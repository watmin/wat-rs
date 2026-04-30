# Arc 111 — intra-process `Result<Option<T>, ThreadDiedError>` for kernel comms

## Status

Drafted 2026-04-30. Follows arc 110's grammar rule directly — keeps
the rule, lifts the type. Stays in-memory only; arc 112 generalizes
the same shape to fork-program subprocess pipes.

## The pathology

Arc 110 closed the silent-disconnect class structurally — every comm
call lands in `match` or `option::expect`. But the receiver still
loses information at disconnect time:

```
A panics with message "balanced book lost"
  → A's stack unwinds; A's Sender clone drops
  → channel's last sender went away → recv returns :None
  → B's match :None arm fires (or expect panics with B's diagnostic)
  → A's actual panic message ("balanced book lost") is captured on
    A's spawn-outcome channel — only readable by whoever joins A
    NOT by B, who was the actual collaborator
```

Three states share one signal (`:None`):

- All senders dropped via clean scope exit. Terminal end-of-stream.
- A sender thread panicked, its sender-clone dropped during unwind.
  Catastrophic; message exists somewhere but B can't see it.
- The substrate's spawn-outcome channel itself disconnected (a
  defect; should not happen under the catch_unwind wrap from arc 060).

The current wat program reading from a forked subprocess gets the
same arms it gets from in-memory channel. Arc 110's grammar rule
still applies — but the **information** the recv carries is too
narrow.

## The new return shape

```
:wat::kernel::send sender value
    -> :Result<:(), :wat::kernel::ThreadDiedError>

:wat::kernel::recv receiver
    -> :Result<:Option<T>, :wat::kernel::ThreadDiedError>

:wat::kernel::try-recv receiver
    -> :Result<:Option<T>, :wat::kernel::ThreadDiedError>

:wat::kernel::select receivers
    -> :(i64, :Result<:Option<T>, :wat::kernel::ThreadDiedError>)
```

Three states, three arms:

- `Ok(Some v)` — value flowed. Common case.
- `Ok(:None)` — channel alive but **terminal**. Every sender dropped
  via clean scope exit; protocol's last message; worker recv-loop
  exits cleanly here.
- `Err(ThreadDiedError::Panic msg failure)` — a sender-thread
  panicked; `msg` carries the panic message, `failure` carries the
  structured `:wat::kernel::Failure` (when the panic was an
  `assertion-failed!` per arc 064).

`E` = `:wat::kernel::ThreadDiedError` — the SAME enum
`:wat::kernel::join-result` already returns. The comm error type IS
the join error type. Both surface "the thread you were talking to
died this way," using the variants arc 060 + arc 105 established.
Programs that were already pattern-matching `join-result`'s `Err`
arms don't need a second matching style.

`:wat::kernel::Sent` typealias retires; it conflated the two senses.
The new shape has no shorthand: write `:Result<:(), :wat::kernel::ThreadDiedError>`
where you'd written `:wat::kernel::Sent`.

## Arc 110's grammar rule still applies

`:wat::kernel::send` and `:wat::kernel::recv` calls land in
`:wat::core::match` (handle three arms) or
`:wat::core::result::expect` (panic on `Err`; the recv now returns
`:Option<T>` directly on success, no nested unwrap).

**Match shape grows from 2 arms to 3:**

```scheme
;; Before (arc 110)                  ;; After (arc 111)
(:wat::core::match (:wat::kernel::recv rx) -> :T
  ((Some v) ...recurse...)            ((Ok (Some v)) ...recurse...)
  (:None ()))                         ((Ok :None) ()) 
                                      ((Err died) (handle-peer-died died)))
```

The new third arm is where the receiver decides what to do with the
panic message: re-raise via `assertion-failed!`, write to a log,
update a supervisor state, etc.

**Expect shape — `result::expect` replaces `option::expect`:**

```scheme
;; Before (arc 110)                   ;; After (arc 111)
(:wat::core::option::expect -> :T     (:wat::core::result::expect -> :Option<T>
  (:wat::kernel::recv rx)               (:wat::kernel::recv rx)
  "rx disconnected — peer died?")       "recv: peer thread died")
                                      ;; result then unwrapped into Option<T>;
                                      ;; the typed Some/None handling stays at the
                                      ;; consumer's level
```

`result::expect`'s panic message gets prefixed with the
`ThreadDiedError`'s own message (ditto how
`:wat::kernel::join-result`'s panic messages render today — the
caller's diagnostic + the dead thread's message).

## What this arc does NOT change

- Does NOT change arc 110's grammar rule. send/recv calls still land
  in match-discriminant or expect-value position.
- Does NOT add cross-process panic propagation. Arc 112 generalizes
  the same return type to fork-program subprocess pipes (stdin =
  Sender; stdout = `Ok(Some T)` payload; stderr = `Err(E)` payload).
- Does NOT touch `:wat::kernel::join-result`. Its `Err` arm already
  returns `ThreadDiedError`. Arc 111 just uses the same type at recv
  for symmetry — programs match the same shape whether they're
  joining a thread or recv'ing from one.
- Does NOT add a "select among supervised channels" primitive. That
  would land if explicit supervision becomes a separate concern;
  for now, every channel is supervised when its sender's origin
  thread is known.

## Implementation

### Slice 1 — type-level shape change (the structural arc)

Net surface change: every `Value::Option(Some(v))` returned by send
/ recv / try-recv becomes `Value::Result(Ok(Value::Option(Some(v))))`.

`src/runtime.rs` (~5 functions):

- `eval_kernel_send` — `Ok(())` becomes `Ok(Result::Ok(()))`;
  `Err(_)` (disconnect) becomes `Ok(Result::Err(thread_died_error_*))`
  per slice-2 wiring.
- `eval_kernel_recv` — `Ok(v)` becomes `Ok(Result::Ok(Option::Some(v)))`;
  `Err(_)` becomes `Ok(Result::Err(thread_died_error_*))`.
- `eval_kernel_try_recv` — `Ok(v)` and `Err(Empty)` and
  `Err(Disconnected)` get the right Result-wrap.
- `eval_kernel_select` — the second tuple element grows from
  `:Option<T>` to `:Result<:Option<T>, :ThreadDiedError>`.

`src/check.rs` (~5 schemes):

- Update the registered schemes for send/recv/try-recv/select.
- The `:wat::kernel::Sent` typealias drops out of `wat/kernel/queue.wat`
  (replaced by the verbose `:Result<:(), :wat::kernel::ThreadDiedError>`).

Slice 1 ships with `Err` always carrying
`:ChannelDisconnected` (a stand-in variant) regardless of cause —
the **type** is right; the **information** in `Err` will be
specific in slice 2. This keeps slice 1 honest and small.

### Slice 2 — populate `Err` with the originating thread's panic

To distinguish "all senders dropped clean" from "a sender thread
panicked," the substrate tracks sender → spawning-thread
relationships and consults the spawning-thread's panic state at
disconnect-detection time.

**Decomposition into single-concern pieces (Hickey: simple, not
easy — composed of simple things, not complected). Substrate
state lives in `OnceLock<T>` cells where multiple actors read
write-once data; never `Mutex` per the zero-Mutex doctrine.**

All shared cells use `OnceLock<T>` — write-once, multi-read,
lock-free. The `OnceLock` is listed in `docs/ZERO-MUTEX.md` §
"Honest caveats" as a legitimate substrate primitive: it does a
different job than `Mutex` (one-time write; not scar tissue on
shared mutable state).

#### Piece 1 — per-thread panic capture cell

Every spawn-thread gets one `Arc<OnceLock<ThreadDiedInfo>>`,
created by the spawn-helper. The thread's `catch_unwind` calls
`set(info)` on panic BEFORE the body's locals drop. One concern:
*did this thread die catastrophically, and with what message?*

```rust
struct ThreadDiedInfo {
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
}

// In eval_kernel_spawn, before std::thread::spawn:
let panic_cell: Arc<OnceLock<ThreadDiedInfo>> = Arc::new(OnceLock::new());
let cell_for_thread = Arc::clone(&panic_cell);

std::thread::spawn(move || {
    let outcome = std::panic::catch_unwind(...);
    if let Err(payload) = &outcome {
        let (message, assertion) = extract_panic_payload_borrow(payload);
        let _ = cell_for_thread.set(ThreadDiedInfo { message, assertion });
    }
    // ... existing SpawnOutcome handling ...
});
```

`OnceLock::set` returns `Err` if already set (idempotent — no
write-write race). The cell stays alive as long as any sender
back-reference holds an `Arc::clone` of it.

#### Piece 2 — `WatSender` carries an origin back-ref

```rust
pub struct WatSender {
    inner: Arc<crossbeam_channel::Sender<Value>>,
    origin: Arc<OnceLock<ThreadDiedInfo>>,
    death_slot: Arc<OnceLock<ThreadDiedInfo>>,  // Piece 4
}
```

`Value::crossbeam_channel__Sender` now wraps `WatSender`. The
`origin` is an `Arc::clone` of Piece 1's cell from whichever
thread currently owns this sender. One concern: *this sender came
from that thread's panic cell.*

`Clone` on `WatSender` clones the three Arcs — origin and
death_slot propagate naturally to clones made inside the spawn
body.

#### Piece 3 — spawn-helper re-tags sender args

When `eval_kernel_spawn` collects `arg_values`, walk for
`Value::crossbeam_channel__Sender` variants and replace each
sender's `origin` with an `Arc::clone` of the new spawn's panic
cell. The original (caller-thread) `WatSender` is unchanged; the
spawn body sees a freshly-tagged copy. One concern: *senders
crossing into a new thread re-tag to that thread's panic cell.*

#### Piece 4 — channel pair shares a death slot

`make-bounded-queue` creates an `Arc<OnceLock<ThreadDiedInfo>>`
shared between sender and receiver ends — the channel's
death-info slot. One concern: *channel-level "did the protocol
die catastrophically."*

```rust
let death_slot: Arc<OnceLock<ThreadDiedInfo>> = Arc::new(OnceLock::new());
// passed into both WatSender and the new WatReceiver (Piece 6)
```

#### Piece 5 — `WatSender::Drop` propagates origin → channel

```rust
impl Drop for WatSender {
    fn drop(&mut self) {
        // Last sender for this channel? Use Arc::strong_count
        // on inner — only this WatSender is holding it now.
        if Arc::strong_count(&self.inner) == 1 {
            if let Some(info) = self.origin.get() {
                // Idempotent — first writer wins.
                let _ = self.death_slot.set(info.clone());
            }
        }
        // inner Arc decrement happens automatically when self drops.
    }
}
```

`OnceLock::get()` returns `Option<&T>` lock-free. `OnceLock::set`
is the atomic-test-and-set; returns `Err(value)` on already-set
(harmless here — first panic wins). One concern: *at the last
drop of a sender for a channel, propagate origin's panic cell to
the channel's death slot.*

#### Piece 6 — receiver surfaces death slot on disconnect

```rust
pub struct WatReceiver {
    inner: Arc<crossbeam_channel::Receiver<Value>>,
    death_slot: Arc<OnceLock<ThreadDiedInfo>>,
}

fn eval_kernel_recv(...) -> Result<Value, RuntimeError> {
    // ...
    match receiver.inner.recv() {
        Ok(v) => Ok(Value::Result(Ok(Some(v)))),
        Err(_) => match receiver.death_slot.get() {
            Some(info) => Ok(Value::Result(Err(thread_died_error_panic(
                info.message.clone(),
                info.assertion.clone(),
            )))),
            None => Ok(Value::Result(Ok(None))),
        },
    }
}
```

One concern: *surface the channel's death slot through recv's
return type.*

### How the pieces compose (linear, no braiding)

```
Thread panic
  → captured in Piece 1's cell (catch_unwind writes)
    → linked to senders via Piece 2's origin field, Piece 3's re-tag
      → propagated by Piece 5 to Piece 4's channel slot at last-drop
        → surfaced by Piece 6 in recv's Result
```

Each piece does ONE thing. Adjacent pieces share data through Arcs
(the same data structure passed through), not through control flow
or hidden state. Each piece can be reasoned about, tested, and
debugged independently. The composition is the system; the
implementation is plumbing.

**Why not Candidates A/B (explicit primitive / TLS ambient):**

- **A (explicit `make-supervised-pair`)** ties the receiver to ONE
  driver-handle. The canonical fan-in pattern (HandlePool with N
  senders from N threads) has multiple supervisors per channel; A
  can attribute disconnect to one of them and silently miss panics
  from the others. **Fails honest.**
- **B (TLS ambient)** captures supervision context implicitly at
  channel construction. Channels passed across thread boundaries
  carry origin from where they were born, not where they're used.
  The model breaks the moment a channel travels — and channels
  travel constantly in CSP. **Fails obvious + honest.**
- **C** scales naturally: every sender knows its own origin; clones
  inherit; spawn re-tags; multiple panicked threads each leave
  their mark; the death slot records the first one (or could
  accumulate). The mental model maps to reality at every step.

### Slice 3 — `:wat::kernel::Sent` typealias retires

Arc 110's sweep replaced `:wat::kernel::Sent` with `:()` in expect
binding sites. Arc 111's slice 1 widens send's return type, so the
alias's expansion (`:Option<()>`) is wrong. Drop the alias from
`wat/kernel/queue.wat`. Anywhere it still appears in user code
becomes a compile error pointing at the new shape.

### Slice 4 — sweep the substrate + lab to comply

The old `(:wat::core::match (:wat::kernel::recv rx) -> :T (Some v) ... (:None ...))`
becomes `(:wat::core::match (:wat::kernel::recv rx) -> :T ((Ok (Some v)) ...) ((Ok :None) ...) ((Err died) ...))`.

Same scope as arc 110's sweep — substrate wat sources, all crate
wat-tests, lab, doc examples, USER-GUIDE, SERVICE-PROGRAMS. Each
file's send/recv calls get a third arm.

### Slice 5 — INSCRIPTION + USER-GUIDE update + 058 row

Same closure shape as arc 110 — INSCRIPTION captures the design,
USER-GUIDE explains the new third arm, FOUNDATION-CHANGELOG row
goes in.

## The four questions

**Obvious?** Yes. The Result wrapping turns the disconnect signal
into the same kind of data every wat program already handles via
`join-result`. Programmers who matched on `Err
(ThreadDiedError::Panic msg _)` from `join-result` write the
identical code at recv sites — no new mental model.

**Simple?** Slice 1 is small (~5 functions in runtime, ~5 schemes
in check). Slice 2 is the larger scope; Candidate A keeps the
substrate change to one new primitive (`make-supervised-pair`) +
one new value type (`SupervisedReceiver`). No flow analysis. No
type-system extension.

**Honest?** Yes. The current substrate type lies — `:Option<T>`
collapses three distinct states into two arms. The Result shape
exposes the third state to the receiver as the data it always was
internally. Programs that didn't care about the difference still
ignore the `Err` arm via `result::expect`; programs that did care
gain a way to express it.

**Good UX?** Yes.
- `Result<Option<T>, ThreadDiedError>` reads exactly like
  `join-result`'s return type. Same arm names. Same matching style.
- The new third arm is opt-in for diagnosis: programs that don't
  care write `result::expect`; programs that do care match
  `(Err died)` and use `ThreadDiedError`'s accessors per arc 105.
- Cross-thread panic propagation lands at the call site that needs
  it — no out-of-band lookup, no separate "did the supervisor see
  a panic" question.

## Slicing summary

| Slice | Work |
|---|---|
| **1** | Type-level shape change in runtime + check schemes. `Err` always `:ChannelDisconnected` (placeholder). `:wat::kernel::Sent` retires. |
| **2** | Wire `Err(Panic)` via Candidate A (`make-supervised-pair` + `SupervisedReceiver`). |
| **3** | Substrate wat sources sweep — every comm call gets the third arm or migrates to `result::expect`. |
| **4** | Lab sweep — same shape, all wat-tests + wat-tests-integ. |
| **5** | INSCRIPTION + USER-GUIDE + 058 row. |

Each slice ends green. Slice 1 alone is a coherent type-honesty
shipment; slices 2-5 progressively make the `Err` carry real
information.

## Cross-references

- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  grammar rule arc 111 builds on.
- `docs/arc/2026/04/110-kernel-comm-expect/DESIGN.md` § "Follow-up"
  — the `Result<Option<T>, E>` shape framed as the next arc.
- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` (or its
  equivalent) — `:wat::kernel::ThreadDiedError` enum and the
  death-as-data discipline arc 111 reuses.
- `docs/arc/2026/04/105-spawn-error-as-data/INSCRIPTION.md` —
  `ThreadDiedError::Panic`'s widened `(message, failure)` shape
  preserved for arc 064's structured panic info.
- arc 112 (queued) — same return type generalized to fork-program
  subprocess pipes.
