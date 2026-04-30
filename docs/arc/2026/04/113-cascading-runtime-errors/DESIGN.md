# Arc 113 — cascading runtime errors as `Vec<ProgramDiedError>`

## Status

Drafted 2026-04-30 during arc 112 slice 2a closure. Captures the
chained-cause backtrace shape arcs 110 + 111 + 112 collectively
made expressible. Lands AFTER arc 109 § J slice 10d mints the
`ProgramDiedError` supertype + typeclass dispatch infrastructure
(Vec element type lives at the supertype). Without that,
arc 113 ships as `Vec<ThreadDiedError>` only and grows to
`Vec<ProgramDiedError>` post-§J.

User direction (2026-04-30):

> can we just... (conj backtrace err) and return a Vec of
> errors?... that'd be amazing
>
> ProgramDiedError is satisfied by ThreadDiedError and
> ProcessDiedError -- yea?

## The shape

Today (post-arc-111):

```scheme
((Err :wat::kernel::ThreadDiedError)
  ;; one error — the immediate peer that died
  (handle-err err))
```

Post-arc-113:

```scheme
((Err died-chain)
  ;; died-chain :Vec<:wat::kernel::ProgramDiedError>
  ;; head = the immediate peer that died
  ;; tail = whatever killed it, recursively, across hosts
  (handle-chain died-chain))
```

The Vec IS the call chain. No separate "Caused by" construct;
no nested `Err.cause = Some(prev_err)` linked-list shape;
just the simplest data structure that carries the information.

## How it accumulates

Every receiver that gets an `Err` and itself dies (panics,
returns Err, gets disconnected) **conjs** its own death info
onto the incoming Vec before propagating. The accumulation
happens at every hand-off boundary:

```
         Thread A panics
              │
              │ panic-cell publishes ThreadDiedError-A
              ▼
     [ThreadDiedError-A]                           ← initial Vec
              │
              │ Thread B's recv on A's channel returns this Vec
              ▼
        Thread B panics handling the Err
              │
              │ panic-cell publishes; B's senders see:
              ▼
     [ThreadDiedError-B, ThreadDiedError-A]        ← (conj prev B)
              │
              │ Process C's process-recv on B's pipe gets this
              │ via stderr-EDN framing
              ▼
        Process C panics (or just exits non-zero)
              │
              │ child stderr emits the Vec as EDN; parent
              │ Thread D's process-recv parses + appends C's death
              ▼
     [ProcessDiedError-C, ThreadDiedError-B, ThreadDiedError-A]
              │
              ▼
       caller's match arm sees the full chain
```

`(:wat::core::Vector/first chain)` answers "what just died";
walking the Vec answers "what killed it, transitively, across
host boundaries"; nothing in between is lost.

## Element type — `ProgramDiedError` (post-§J)

The Vec's element type is `:wat::kernel::ProgramDiedError` (arc
109 § J slice 10d), the supertype both `ThreadDiedError` and
`ProcessDiedError` satisfy. This means a single chain can hold
deaths from BOTH host kinds — a Thread that died because a
Process it was talking to died because a Thread it was talking
to died gets one `Vec<ProgramDiedError>` containing all three,
each entry's concrete type still pattern-matchable when the
caller wants to discriminate.

```scheme
((Err died-chain)
  ;; died-chain :Vec<:wat::kernel::ProgramDiedError>

  ;; Host-agnostic match — most common
  (:wat::core::for died-chain (fn (entry)
    (log-died (:wat::kernel::ProgramDiedError/message entry))))

  ;; Host-discriminated match — when subject matters
  (:wat::core::for died-chain (fn (entry)
    (:wat::core::match entry
      ((:wat::kernel::ThreadDiedError-Panic msg _failure)
       (log-thread-panic msg))
      ((:wat::kernel::ProcessDiedError-Panic msg _failure)
       (log-process-panic msg))
      (_ (log-other entry))))))
```

Pre-§J: arc 113 ships `Vec<ThreadDiedError>` only (chains stay
within thread space). Once §J slice 10d lands, the Vec generalizes
to `Vec<ProgramDiedError>`. The conj operation is the same; only
the static element type widens.

## Implementation — arc 111's six OnceLock pieces

Arc 111's DESIGN.md (Candidate C — the rich-Err follow-up
pieces) named six OnceLock cells that arc 113 wires into a
chained-cause mechanism. The minimal six:

1. **Per-thread panic cell** — a `OnceLock<ThreadDiedError>` on
   each thread, published by the catch_unwind wrap when the
   thread panics. Idempotent; safe to read after publish.
2. **WatSender's origin back-ref** — every `Sender<T>` carries
   a back-ref to the panic cell of the thread that owns its
   peer Receiver. When a Sender's peer thread dies, send/recv
   on this Sender surfaces the cell's contents.
3. **Spawn-helper re-tag** — when arc 112's `spawn-thread`
   creates a Thread<I,O>, the inner thread's panic cell gets
   wired to the channel pair backing stdin/stdout/stderr.
4. **Channel pair death slot** — each Sender/Receiver pair has
   a shared `OnceLock<Vec<ProgramDiedError>>` populated when
   either side's owning thread/process dies. The first death
   wins; subsequent deaths conj onto the Vec via the `Drop`
   propagator.
5. **Drop propagator at last-drop** — when the last Sender is
   dropped (channel closed cleanly OR catastrophically), Drop
   reads the pair's death slot. If empty: surface `Ok(:None)`
   on subsequent recv (clean disconnect). If non-empty: surface
   `Err(chain)`.
6. **Receiver surfaces the death slot** — recv on a closed
   channel checks the death slot; recv result widens from
   `Result<Option<T>, ThreadDiedError>` (arc 111 slice 1's
   placeholder) to `Result<Option<T>, Vec<ProgramDiedError>>`
   (arc 113's chain shape).

For the Process side (arc 112), the equivalent six are:

1'. **Process exit-code + stderr** is the panic cell. The
    child's catch_unwind wrap renders `ProgramDiedError` as
    EDN to stderr before `_exit`-ing; non-zero exit codes
    distinguish "ran but panicked" from "didn't start" from
    "killed externally."
2'. **`Process<I,O>`'s join field IS the back-ref** — the
    parent reads stderr-EDN + exit code via Process/join-result.
3'. **fork-helper re-tag** — fork-program's child branch
    installs a panic hook that emits the EDN-rendered chain
    on stderr before exit.
4'. **Pipe death is implicit** — when the parent's process-recv
    sees stdout EOF + stderr non-empty, the chain comes from
    stderr's framed EDN.
5'. **Parent's Drop on Process** — if the parent drops a
    Process without joining, Drop sends SIGKILL and waitpid;
    no chain (the parent killed it, not catastrophic).
6'. **process-recv parses the chain** — stderr-emitted
    Vec<ProgramDiedError> EDN deserializes into the same
    Vec the channel-side code returns. Same element type;
    same conj at every hand-off.

## Slicing

| Slice | Work |
|---|---|
| **1** | Mint `:wat::kernel::ProgramDiedError` (depends on arc 109 § J slice 10d's typeclass infra). If §J hasn't landed yet, this slice ships as preparatory work — the supertype is declared, satisfaction is asserted in tests, but consumers stay on `ThreadDiedError` until §J wires the dispatch. |
| **2** | Per-thread panic cell + Sender back-ref + channel-pair death slot. recv result type widens from `Result<Option<T>, ThreadDiedError>` to `Result<Option<T>, Vec<ProgramDiedError>>`. Substrate-as-teacher migration hint at every TypeMismatch — same arc-111-style sweep pattern. |
| **3** | fork-program child panic hook + stderr-EDN framing of the chain. process-recv parses stderr into `Vec<ProgramDiedError>`. |
| **4** | Sweep consumers — every `((Err err) ...)` arm becomes `((Err died-chain) ...)`. Most consumers can keep matching the head only via `(:wat::core::Vector/first chain)`; full-chain walk is opt-in. |
| **5** | INSCRIPTION + USER-GUIDE + 058 row. Retire `arc_111_migration_hint` (task #168) — chain-shape supersedes single-error shape. |

## Dependency on arc 109 § J + arc 114

Arc 113's clean shape **requires** arc 109 § J slice 10d's
typeclass dispatch (so `ProgramDiedError` works as the Vec
element type). It also benefits from arc 114's spawn-thread
rename (so all thread-side error sources go through the
`ThreadDiedError` shape with no R-yielding bare-spawn corner
case to special-case).

Recommended landing order:

1. arc 109 § J slices 10a → 10c (mint Program supertype, split
   Thread/Process, rename today's unified `Process<I,O>` →
   `Program<I,O>`).
2. arc 109 § J slice 10d (typeclass dispatch — the
   `ProgramDiedError` machinery is part of this slice).
3. arc 114 slices 1 → 3 (kill spawn's R; spawn-thread; ALL
   thread-side sources now produce the `ThreadDiedError` shape
   uniformly).
4. arc 113 slice 1 → 5 (the chain accumulation).

If §J slips, arc 113 ships as `Vec<ThreadDiedError>` first;
widening to `Vec<ProgramDiedError>` is a one-element-type
change post-§J.

## What this arc does NOT do

- Does NOT add a separate "Caused by" construct or recursive
  Err.cause shape. The Vec IS the chain. Programs that want
  to ignore depth use `Vector/first`; programs that want to
  reason about the full chain walk it.
- Does NOT change recv's three-state shape. `Ok(Some v)` /
  `Ok(:None)` (clean disconnect) / `Err(Vec<...>)` (catastrophic
  disconnect with chain). Same arms arc 111 set up.
- Does NOT touch arc 060's `:wat::kernel::join-result` on a
  bare `ProgramHandle<R>`. That keeps `Result<R,
  ThreadDiedError>` (single err) until arc 114 retires the
  bare-spawn path entirely.
- Does NOT introduce a new error variant. `ProgramDiedError`
  is just the supertype; the three concrete variants (Panic,
  RuntimeError, ChannelDisconnected) remain.

## The four questions

**Obvious?** Yes. `(conj backtrace err)` is the simplest
possible shape; reads as "the Vec IS the chain." Match arms
read the head for "what just died" or walk for "what cascaded."
No new vocabulary.

**Simple?** Yes — Vec<T> + conj. The complexity is in arc 111's
six OnceLock pieces, not in the user-facing shape.

**Honest?** Yes. The chain CROSSES hosts. A Vec<ProgramDiedError>
captures that cross-host causality data-faithfully. No special
encoding for "this entry was a Thread vs a Process" — that's in
the concrete satisfier's type, pattern-matchable when relevant.

**Good UX?** Yes. Most consumers just want to know "did the peer
die?" → check `Vector/first`. Sophisticated consumers walk the
chain. Both are direct on the same shape; nothing has to be
unwrapped.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — Program /
  Thread / Process supertype split + ProgramDiedError mirror.
  Slice 10d mints the typeclass infrastructure arc 113 leans on.
- `docs/arc/2026/04/111-result-option-recv/DESIGN.md` —
  Candidate C's six OnceLock pieces arc 113 wires.
- `docs/arc/2026/04/111-result-option-recv/INSCRIPTION.md` —
  arc 111's slice 1 placeholder (`Err(ChannelDisconnected)`)
  arc 113 replaces with the rich chain.
- `docs/arc/2026/04/112-inter-process-result-shape/DESIGN.md` —
  arc 112's process-side Err mechanism arc 113 generalizes
  via stderr-EDN framing.
- `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` — arc 114's
  spawn-thread + Thread<I,O> shape arc 113's chain mechanism
  serves uniformly.
