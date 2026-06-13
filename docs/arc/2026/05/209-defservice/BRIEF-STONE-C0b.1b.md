# BRIEF — C0b.1b: `select'` learns the `Listener'` + `SelectEvent<I,O>` (FOLD)

## The work (one paragraph)

Give `:wat::kernel::select'` a **2-arg form** `(select' listener peers) -> :wat::kernel::SelectEvent<I,O>`
that multiplexes a listener (new connections) with the connected client peers, so a service loop
grows/serves/shrinks in one blocking call. The existing **1-arg** `(select' peers) -> Tuple<i64,O>`
is UNTOUCHED (brackets depends on it). **FOLD design:** `select'` uses the **existing** comms
`select()` (blocking, consuming — no `ready()`, no `try_recv`, no comms change). On the won index:
the **listener** (index 0) → `select()` consumed the connect-request → `select'` unpacks + wraps
the new server `Peer'` inline (reuse `accept'`'s unpack, on the service thread → custody holds) →
`:Connection [peer]`; a **peer** (index k>0) → `Ok` → `:Message`, `Disconnected` → `:Closed`. The
committed RED probe `tests/nursery/probe_arc209_c0b1b_select_listener.rs` is the gate (returns 24).

## Read in order (the rooms)

1. **`tests/nursery/probe_arc209_c0b1b_select_listener.rs`** — the gate. The `serve` loop's 4 match
   arms (`:Connection peer`→conj, `:Message`→serve, `:Closed`/`:Lost`→`remove-at`) + the client
   choreography (two clients connect, round-trip n*2, stop → 24). RED at HEAD on `select'` arity
   (1 vs 2) + `SelectEvent::*` unknown.
2. **`docs/arc/2026/05/209-defservice/DESIGN-STONE-C0b.1b-select-grows-listener.md`** § "⚙ FINAL
   SHAPE" — the authoritative `SelectEvent<I,O>` definition + the FOLD mechanic.
3. **`src/runtime.rs` `eval_accept_prime`** (`:17971`) — it recv's a connect-request from the
   listener then **unpacks the `Value::Tuple` `(req_rx, resp_tx)` and wraps `Peer { tx: resp_tx,
   rx: req_rx }` on this thread**. Factor that unpack+wrap (everything AFTER the recv) into a
   helper `wrap_connect_request(cr: Value, span) -> Result<Value, EvalBreak>`; `accept'` becomes
   `recv` + the helper; `select'`'s listener arm calls the helper on the `select()`-consumed value.
4. **`src/runtime.rs` `eval_peer_select_prime`** (`:23049`, the 1-arg form) + its C0 bare-`Peer'`
   arm (`:23318`) — connection peers are `PEER_TYPE_PATH` (bare `Peer { tx, rx }`); select registers
   each peer's `.rx`. The comms `Select` (`src/comms/thread.rs:235/250`) — `recv(rx)` registers
   (index = registration order), `select()` returns `SelectOutcome::Recv { index, result }`
   (consuming) or `Shutdown`. **Use it as-is.**
5. **`src/runtime.rs` `thread_died_error_panic`** (`:19913`) — the precedent for constructing a
   stdlib-defenum `Value::Enum(EnumValue { type_path, variant_name, fields })`. Mirror it for
   `SelectEvent::Connection/Message/Closed`.
6. **`src/check.rs` `infer_select_prime`** (`:10554`) + `infer_accept_prime` (`:9922`, the
   `Listener'<S,R>` match) — mirror for the 2-arg scheme.
7. **`wat/spawn.wat`** — where `SelectEvent` is declared.

## Implementation sketch (fill it; don't invent the shape)

**1 — `wat/spawn.wat`: declare the defenum.**
```
(:wat::core::defenum :wat::kernel::SelectEvent<I,O>
  :Connection [peer  <- :wat::kernel::Peer'<I,O>]
  :Message    [idx   <- :wat::core::i64  msg   <- :O]
  :Closed     [idx   <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64  cause <- :wat::kernel::Failure])
```

**2 — `src/runtime.rs`: factor `wrap_connect_request`** out of `eval_accept_prime` (the unpack +
wrap-on-this-thread, everything after the listener recv). `accept'` = recv + `wrap_connect_request`.

**3 — `src/runtime.rs` `eval_peer_select_prime`: branch on `args.len()`.** 1 → existing path
untouched. 2 → the thread/bare-`Peer'` arm:
- Eval `args[0]` (listener) → `Value::wat__kernel__Receiver` → `&Receiver<Value>`.
- Eval `args[1]` (peers) → `Vec` of `PEER_TYPE_PATH` opaques → guard each → `&peer.rx`.
- Build `comms::thread::Select`; `recv(listener_rx)` FIRST (index 0), then each peer `.rx` (1..=N).
- `sel.select()`:
  - `Shutdown` → existing MalformedForm "interrupted by shutdown".
  - `Recv { index: 0, result }` (listener) → `let cr = result?` (the connect-request `Value`) →
    `wrap_connect_request(cr, span)?` → `SelectEvent::Connection { peer }`.
  - `Recv { index: k, result }`, `k>0` (peer `k-1`) → `result`: `Ok(v)` →
    `SelectEvent::Message { idx: k-1, msg: v }`; `Err(_)` → `SelectEvent::Closed { idx: k-1 }`.
- Construct the `SelectEvent` `Value::Enum` (mirror `thread_died_error_panic`'s `EnumValue {
  type_path: ":wat::kernel::SelectEvent", variant_name, fields }`).
- **`:Lost` is NOT emitted at the thread tier** (remote-only) — do not synthesize it.

**4 — `src/check.rs` `infer_select_prime`: branch on arity.** 2 args → `args[0]` reduces to
`Listener'<S,R>` (mirror `infer_accept_prime:9953`); `args[1]` is `Vector<Peer'<I,O>>`. Return
`Parametric { head: "wat::kernel::SelectEvent", args: vec![I, O] }` — `I`/`O` are the peers'
element type params (`:Connection`'s peer is `Peer'<I,O>`, `:Message`'s msg is `O`). The 1-arg form
returns `O` from the same slot; for the 2-arg form you need BOTH params.

## Blast radius

`src/runtime.rs` (factor `wrap_connect_request` + the `eval_peer_select_prime` 2-arg arm + the
`SelectEvent` construction), `src/check.rs` (`infer_select_prime` 2-arg arm), `wat/spawn.wat` (the
defenum). **Do NOT touch `src/comms/thread.rs`** (the existing `select()` is reused as-is), the
1-arg `select'` path, `listener'`/`connect'`, the process tier, or `comms::process`.

## STOP triggers (surface the gap; do not improvise past them)

1. If `wrap_connect_request` cannot be cleanly factored from `eval_accept_prime` (the unpack/wrap
   is entangled with the recv) — STOP and report; do not duplicate the wrap logic into select'.
2. If the listener (`wat__kernel__Receiver`) and the bare `Peer'` `.rx` are not both
   `comms::thread::Receiver<Value>` (can't co-register in one `Select`) — STOP, report the types.
3. If constructing a `Value::Enum` for the stdlib `SelectEvent` defenum has no clean path (the
   defenum isn't registered at eval time) — STOP; do not fall back to a Tuple.
4. If the parametric `SelectEvent<I,O>` scheme fights inference — note exactly what you relaxed; the
   goal is the probe green, but do not collapse to a single param (the `:Connection` peer needs both).

## Gate (run each, READ the output, report the verbatim final line — do NOT commit)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`
   → **1 passed** (returns 24).
2. `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1`
   → **1 passed** (C0b.1 connection + `accept'` factoring intact).
3. `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1`
   → **1 passed**.
4. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the known reds (4
   baseline); zero NEW.
5. `cargo test --release --test test 2>&1 | tail -3` → 242/1 (the 1 pre-existing).
6. `cargo build --release` clean; `cargo clippy` clean in the touched files.

## Report back

The `wrap_connect_request` factor + the 2-arg eval arm + the infer scheme + the defenum, as
written. The `SelectEvent` representation chosen. Verbatim final line of each gate row. Any STOP
hit + exact error. Honest deltas vs this sketch. Do **NOT** commit — the Inquisitor weighs against
its own re-run and commits.
