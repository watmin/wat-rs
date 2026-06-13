# BRIEF — C0b.1b: `select'` is the service multiplexer + `SelectEvent<I,O>` (3-input, deadlock-free)

## The work (one paragraph)

Give `:wat::kernel::select'` a **3-arg form** `(select' self-peer listener peers) ->
:wat::kernel::SelectEvent<I,O>` that multiplexes a service's three inputs in one blocking call: the
**self-peer** (owner/supervisor link → `:Shutdown`), the **listener** (new connections), and the
**connected client peers** (requests). This is the deadlock-free service loop: when the owner drops
the service handle, RAII **drain** disconnects the self-peer's `input_rx` → `select'` returns
`:Shutdown` → the loop exits → `join` completes. No cooperative stop; dropping the handle IS the
shutdown. The existing **1-arg** `(select' peers) -> Tuple<i64,O>` is UNTOUCHED (brackets depends
on it). `select'` uses the existing comms `select()` (no `ready`/`try_recv`, **no `comms/thread.rs`
change**). The committed RED probe `tests/nursery/probe_arc209_c0b1b_select_listener.rs` is the gate
(returns 24, then terminates cleanly on handle-drop).

> WHY 3-arg (not 2): a 2-arg `(listener, clients)` loop has no termination path — the owner's RAII
> drop drains the self-peer, but a loop not watching the self-peer never wakes, and the join hangs.
> gdb + a prior strike both proved this deadlock. Watching the self-peer makes the drain the wake.
> Vended primitives never deadlock — that is the contract this enforces.

## Read in order (the rooms)

1. **`tests/nursery/probe_arc209_c0b1b_select_listener.rs`** — the gate. The `serve` loop is
   `[self l clients]`; it `(select' self l clients)` and matches 5 arms (`:Shutdown`→exit,
   `:Connection peer`→conj, `:Message`→serve, `:Closed`/`:Lost`→remove-at). The owner DROPS the
   handle at scope-exit (no Stop) → must terminate. RED at HEAD on `select'` arity (1 vs 3) +
   `SelectEvent::*` unknown.
2. **`docs/arc/2026/05/209-defservice/DESIGN-STONE-C0b.1b-select-grows-listener.md`** § "⚙ FINAL
   SHAPE" — the authoritative `SelectEvent<I,O>` + the 3-input mechanic.
3. **`src/runtime.rs` `eval_accept_prime`** (`:17971`) — recv's a connect-request then unpacks the
   `Value::Tuple (req_rx, resp_tx)` and wraps `Peer { tx: resp_tx, rx: req_rx }` on this thread.
   Factor that unpack+wrap (everything AFTER the recv) into `wrap_connect_request(cr: Value, span)
   -> Result<Value, EvalBreak>`. `accept'` = recv + helper; `select'`'s listener arm = helper on the
   `select()`-consumed value. ONE helper, TWO callers.
4. **`src/runtime.rs` `eval_peer_select_prime`** (`:23049`, 1-arg) + its C0 bare-`Peer'` arm
   (`:23318`) — connection peers AND the self-peer are `PEER_TYPE_PATH` (`Peer { tx, rx }`); the
   self-peer's `.rx` = `input_rx`. The comms `Select` (`src/comms/thread.rs:235/250`): `recv(rx)`
   registers (index = registration order), `select()` returns `SelectOutcome::Recv { index, result }`
   (consuming) / `Shutdown`. USE AS-IS.
5. **`src/kernel/spawn.rs` `spawn_thread_peer`** — confirms the prog's `self` is
   `Peer { tx: output_tx, rx: input_rx }`, and RAII `drain_and_join` drops `input_tx` (so the
   self-peer's `input_rx` disconnects on owner-drop — the `:Shutdown` trigger).
6. **`src/runtime.rs` `thread_died_error_panic`** (`:19913`) — precedent for constructing a
   stdlib-defenum `Value::Enum(EnumValue { type_path, variant_name, fields })`. Mirror for
   `SelectEvent::{Shutdown,Connection,Message,Closed}`.
7. **`src/check.rs` `infer_select_prime`** (`:10554`) + `infer_accept_prime` (`:9922`) — mirror for
   the 3-arg scheme.
8. **`wat/spawn.wat`** — where `SelectEvent` is declared.

## Implementation sketch (fill it; don't invent the shape)

**1 — `wat/spawn.wat`:**
```
(:wat::core::defenum :wat::kernel::SelectEvent<I,O>
  :Shutdown
  :Connection [peer  <- :wat::kernel::Peer'<I,O>]
  :Message    [idx   <- :wat::core::i64  msg   <- :O]
  :Closed     [idx   <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure])
```

**2 — `src/runtime.rs`: factor `wrap_connect_request`** out of `eval_accept_prime`. `accept'` =
recv + helper.

**3 — `src/runtime.rs` `eval_peer_select_prime`: branch on `args.len()`.** 1 → existing path
untouched. 3 → the service-multiplexer arm:
- Eval `args[0]` (self-peer) → `PEER_TYPE_PATH` opaque → guard → `&peer.rx` (= `input_rx`).
- Eval `args[1]` (listener) → `Value::wat__kernel__Receiver` → `&Receiver<Value>`.
- Eval `args[2]` (peers) → `Vec` of `PEER_TYPE_PATH` opaques → guard each → `&peer.rx`.
- Build `comms::thread::Select`; `recv` the self-peer rx FIRST (index 0), then the listener rx
  (index 1), then each client `.rx` (index 2..=N+1).
- `sel.select()`:
  - `Shutdown` (substrate cascade) → existing MalformedForm "interrupted by shutdown".
  - `Recv { index: 0, .. }` (self-peer) → `SelectEvent::Shutdown` (the owner dropped — the RAII drain
    disconnected `input_rx`; do NOT inspect the result, the self-peer is the supervisor link).
  - `Recv { index: 1, result }` (listener) → `let cr = result?` → `wrap_connect_request(cr, span)?`
    → `SelectEvent::Connection { peer }`.
  - `Recv { index: k, result }`, `k≥2` (client `k-2`) → `Ok(v)` → `SelectEvent::Message { idx: k-2,
    msg: v }`; `Err(_)` → `SelectEvent::Closed { idx: k-2 }`.
- Construct the `SelectEvent` `Value::Enum` (mirror `thread_died_error_panic`). `:Lost` is NOT
  emitted at the thread tier — do not synthesize it.

**4 — `src/check.rs` `infer_select_prime`: branch on arity.** 3 args → `args[0]` is any `Peer'<_,_>`
(the self-peer — its params don't constrain the result); `args[1]` reduces to `Listener'<S,R>`
(mirror `infer_accept_prime:9953`); `args[2]` is `Vector<Peer'<I,O>>`. Return `Parametric { head:
"wat::kernel::SelectEvent", args: vec![I, O] }` (from the peers' element type).

## Blast radius

`src/runtime.rs` (factor `wrap_connect_request` + the `eval_peer_select_prime` 3-arg arm +
`SelectEvent` construction), `src/check.rs` (`infer_select_prime` 3-arg arm), `wat/spawn.wat` (the
defenum). **Do NOT touch `src/comms/thread.rs`** (existing `select()` reused), the 1-arg `select'`
path, `listener'`/`connect'`/`accept'`'s recv, the process tier, or `comms::process`.

## STOP triggers (surface the gap; do not improvise past them)

1. If `wrap_connect_request` can't be cleanly factored from `eval_accept_prime` — STOP; do not
   duplicate the wrap logic.
2. If the self-peer's `.rx`, the listener `Receiver`, and the client `.rx` are not all
   `comms::thread::Receiver<Value>` (can't co-register) — STOP, report the types.
3. If constructing a `Value::Enum` for the stdlib `SelectEvent` has no clean path — STOP; do NOT
   fall back to a Tuple.
4. If making the probe green appears to need a `comms/thread.rs` change (a `ready`/peek) — STOP and
   surface it; the design says the existing `select()` suffices (self-peer index 0 → `:Shutdown`,
   the drain is the wake). Reintroducing `try_recv`/`ready` is the aborted PEEK path.

## Gate (run each, READ the output, report the verbatim final line — do NOT commit)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`
   → **1 passed** (returns 24 AND terminates — the handle-drop `:Shutdown` must not hang).
2. `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1`
   → **1 passed** (C0b.1 + `accept'` factor intact).
3. `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1`
   → **1 passed**.
4. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 baseline reds; zero
   NEW. **If the run hangs, the self-peer `:Shutdown` wiring is wrong — that is the bug; surface it.**
5. `cargo test --release --test test 2>&1 | tail -3` → 242/1 (pre-existing).
6. `cargo build --release` clean; `cargo clippy` clean in the touched files.

## Report back

The `wrap_connect_request` factor (one helper, two callers); the 3-arg eval arm (esp. the index-0
`:Shutdown` mapping); the infer scheme; the defenum; verbatim final line of each gate row; any STOP
hit + exact error; honest deltas. Do **NOT** commit — the Inquisitor weighs against its own re-run.
