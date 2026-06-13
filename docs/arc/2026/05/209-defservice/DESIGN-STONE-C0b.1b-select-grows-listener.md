# DESIGN-STONE C0b.1b — `select'` learns the `Listener'` + `SelectEvent<O>`

> Prerequisite for Stone C (defservice). Thread tier only (rides C0b.1; the process
> `Listener'` arrives at C0b.2/.3). Mechanism: [`DESIGN-STONE-C0b-host-parametric-connection.md`](./DESIGN-STONE-C0b-host-parametric-connection.md).
> Inquisitor draws; Shadowdancer (sonnet) executes; Inquisitor weighs.

## Why this exists (the four-questions reveal, 2026-06-12)

A defservice loop must **grow** (a new client connects) and **shrink** (a client leaves) while
it serves existing clients — all from one blocking `select'`. The honest way to admit the
listener into that `select'` was settled by the four questions:

- **Modelling the door as a `Peer'<ConnectRequest,nil>`** dropped into the homogeneous
  `Vector<Peer'<I,O>>` **FAILS Honest + Obvious** — the `nil` half is a dead half
  (`feedback_optional_is_a_smell`), and to share the vector you must collapse connect-events
  and ops into one anonymous union message enum, which wat (an ADT language, no anon unions)
  can't express. **Rejected.**
- **Teaching `select'` to take the `Listener'` as a distinct arg and return a NAMED event
  sum** clears all four. This is what the builder originally described — *"admin event and
  user events — the server does an io select, gets a caller, works it, recurs"* (two event
  kinds from one select = a named sum); what the builder rejected was a new `serve'` **verb**,
  not the two-event shape. The loop stays plain wat.

## What this delivers — the 2-arg `select'` + `SelectEvent`

```
;; NEW — declared in the kernel stdlib alongside the peer types
;; (home: wat/spawn.wat or wat/kernel/ — confirm at probe time)
(:wat::core::defenum :wat::kernel::SelectEvent<O>
  :Connection                                                     ;; the listener fired — a client is dialing
  :Message [idx <- :wat::core::i64  msg    <- :O]                 ;; peer[idx] delivered msg
  :Closed  [idx <- :wat::core::i64]                               ;; peer[idx] left GRACEFULLY (clean EOF)
  :Crashed [idx <- :wat::core::i64  reason <- :wat::core::String]);; peer[idx] DIED unexpectedly — + reason

;; NEW 2-arg form (ADDITIVE — the 1-arg (select' peers) -> Tuple<i64,O> is UNTOUCHED)
(:wat::kernel::select' listener peers) -> :wat::kernel::SelectEvent<O>
```

`listener : Listener'<S,R>`, `peers : Vector<Peer'<R,S>>` (the accepted server ends —
`accept' : Listener'<S,R> -> Peer'<R,S>`), result `SelectEvent<S>` (the service receives `S`).
Minimal honest requirement: `O` is the peers' recv-type, extracted exactly as the 1-arg
`infer_select_prime` extracts it; tightening `listener.S == peers.recv` is optional polish, not
required for the gate — do not let it block the probe.

The service loop (plain wat, written by hand in the Stone-C gate, later macro-generated):

```
:Connection            -> (accept' listener) -> push the new Peer' -> recur        ;; GROW
:Message  idx msg      -> handle msg, reply on peers[idx], recur
:Closed   idx          -> (remove-at peers idx) -> recur                            ;; graceful SHRINK
:Crashed  idx reason   -> (remove-at peers idx) -> [surface/log reason] -> recur    ;; SHRINK + diagnostics
```

`:Connection` carries no payload — `select'` does **not** mint the peer; the loop reuses the
**already-shipped `accept'`**. Deprovision is implicit: a client drops its handle → its
`Peer'` output EOFs → `:Closed`/`:Crashed`. No explicit "remove" message (the admin collapse).

## The one contract decision (pinned)

`select'` reports readiness; it does **not** fold `accept'` in. `:Connection` is payload-free
and the loop calls the shipped `accept'`. (Alternative — `select'` accepts internally and
returns `:Accepted [peer]` — rejected: it couples minting into `select'` for no gain and
duplicates `accept'`.) **Graceful vs. crashed is MANDATORY** (builder: *"we absolutely must
know graceful vs unexpected disconnect — not having that is willfully leaving diagnostics on
the ground"*) — `:Closed` and `:Crashed [reason]` are distinct variants.

## Mechanics (the strike path — fill it, don't invent the shape)

**Grounded foundation (verified this session):**
- `Listener'` is a raw `Value::wat__kernel__Receiver` (the rendezvous; `eval_listener_prime`
  `runtime.rs:17878`); `accept'` already consumes one (`eval_accept_prime:17984`).
- A thread `Peer'` cell holds `Thread<I,O>` with `output` AND **`crash: Receiver<String>`**
  (`peer.rs:86`). `Thread::recv` reads `output`, then on EOF reads `crash` —
  `Ok(reason)→Crashed`, `Err→Disconnected` (graceful) — `peer.rs:108-123`.
- `eval_peer_select_prime` (`runtime.rs:23049`) already downcasts each peer to its
  `ThreadOwnedCell`, takes a `RefGuard`, and registers `&peer.output` into a
  `comms::thread::Select`. It **collapses** a closed peer to one raise (`:23180`) — the
  diagnostic-loss this strike fixes.

**`infer_select_prime` (check.rs) — branch on arity:**
- 1 arg → existing path (`Tuple<i64,O>`), untouched.
- 2 args → `args[0]` must reduce to `Listener'<S,R>` (mirror `infer_accept_prime:9953`'s
  Listener' match); `args[1]` must be `Vector<Thread'<I,O>>` (existing extraction). Return
  `Parametric{ "wat::kernel::SelectEvent", [O] }`.

**`eval_peer_select_prime` (runtime.rs) — branch on arity (2-arg = thread tier only):**
1. Eval `listener` → expect `Value::wat__kernel__Receiver` (else TypeMismatch, mirror
   `eval_accept_prime`). Eval `peers` → `Vec` of `THREAD_PEER_TYPE_PATH` opaques (reuse the
   existing downcast+guard loop).
2. Build one `comms::thread::Select`. Register the **listener receiver at index 0**, then each
   peer's `&peer.output` at indices `1..=N`. (Keep the peer guards alive for step 4.)
3. `sel.select()`:
   - index `0` → construct `SelectEvent::Connection`.
   - index `k>0`, `result = Ok(value)` → `SelectEvent::Message { idx: k-1, msg: value }`
     (subtract the listener offset so `idx` indexes the *peers* vector).
   - index `k>0`, `result = Err(_)` (output EOF) → read `guards[k-1]`'s `Thread.crash` exactly
     as `peer.rs:121`: `crash.recv()` `Ok(reason)→Crashed{ idx:k-1, reason }` /
     `Err(_)→Closed{ idx:k-1 }`.
   - `Shutdown` → the existing MalformedForm raise (unchanged).
4. Construct the `SelectEvent` variant `Value` (mirror how the Rust side builds other kernel
   enum variants — e.g. the `Result`/`Option` constructors in `recv'`). Return it.

**The crash read is a reuse of `Thread::recv`'s second half — not a new mechanism.** `select'`
already holds the guarded cell; on EOF it does what `recv'` already does.

## ⚙ DESIGN RESOLUTIONS (2026-06-12, from the structured-peer-death campaign + the probe)

**`:Crashed`'s reason — reconstruct from the envelope, uniform across tiers.** structured-peer-death
shipped the crash channel carrying an EDN envelope STRING (forced: the process tier is a pipe — it
can only carry serialized strings; the unified `Peer'` keeps both tiers the same shape). So
`select'`'s `:Crashed` arm reads `.crash` (the envelope string) and `edn::read`s it into a
structured value — the SAME way the process tier already parses EDN off its pipe. No tier-specific
struct handling. The reconstruction target type (a `Failure` value vs. the parsed envelope value)
is pinned at build time by grounding what `edn::read` of `#wat.kernel/AssertionFailure {…}` yields;
the contract is `:Crashed [idx <- :i64  reason <- <the reconstructed structured value>]`.

**Service termination — an explicit `Stop` op, NOT owner-drop (for this stone's probe).** A service
loop blocked in `select'` over `(listener, clients)` is NOT watching its `spawn-program'` self-peer,
so the owner merely *dropping* the handle would leave the loop blocked → the RAII join would hang.
Clean owner-initiated RAII shutdown is a Stone C/D lifecycle concern. For C0b.1b, the service
exits the loop on an explicit `Stop` op (gen_server `:stop`) — the probe sends it to terminate so the
join is clean. **The service IGNORES its self-peer** (legal — unused self-peer; the C0 precedent);
it watches only the listener + the connected clients. (`Stop` being a privileged owner-only op is
the per-op identity policy — Stone C.)

**Client departure stays implicit.** A client dropping its `Peer'` → `:Closed [idx]` → `remove-at`
(shrink). Only *service shutdown* is the explicit `Stop`; *client* leave is the implicit EOF.

## Files touched

| File | Change |
|---|---|
| `wat/spawn.wat` *(or kernel stdlib — confirm)* | declare `:wat::kernel::SelectEvent<O>` defenum |
| `src/check.rs` | `infer_select_prime` — 2-arg arm → `SelectEvent<O>` |
| `src/runtime.rs` | `eval_peer_select_prime` — 2-arg thread arm (listener+peers, index map, `.crash` read, variant construction) |
| `tests/nursery/probe_arc209_c0b1b_select_listener.rs` | the RED probe (Inquisitor writes it STRIKE-READY) |

Blast radius: the 1-arg `select'` path, `listener'`/`connect'`/`accept'`, and the process tier
are **not** touched.

## Out of scope = affirmatively rejected

- **Process-tier 2-arg `select'`** — the process `Listener'` does not exist yet (C0b.2/.3
  builds it); the process arm lands in that strike, when there is a process listener to pass.
  Not a half-verb: the 2-arg form covers every listener that currently exists.
- **The 1-arg `select'` contract** — unchanged (brackets depends on `Tuple<i64,O>` +
  raise-on-drain). This strike is additive.
- **The defservice macro / the hand-rolled service proof** — Stone C and its gate; this strike
  only delivers the verb they consume.
- **`:Crashed` reason as a structured type** — the crash channel is `Receiver<String>`; `String`
  is the grounded reason type. A richer error type is a separate decision.

## Gate (Inquisitor re-runs each; Shadowdancer reports, Inquisitor weighs)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`
   → GREEN. The probe must exercise all four variants: a connect → `:Connection` → `accept'`
   grows the set; a message → `:Message`; a client dropping cleanly → `:Closed`; a client
   panicking → `:Crashed` carrying the reason.
2. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds, zero new.
3. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken.
4. `cargo build --release` clean; `cargo clippy` clean in the touched homes.

## Estimate

~120–200 lines Rust (one infer arm + one eval arm + the variant construction) + the defenum.
Every primitive grounded above. One Shadowdancer strike behind the committed RED probe.
