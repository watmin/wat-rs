# BRIEF — C0b.1b: `select'` learns the `Listener'` + `SelectEvent<O>`

## The work (one paragraph)

Give `:wat::kernel::select'` a **2-arg form** `(select' listener peers) -> :wat::kernel::SelectEvent<O>`
that multiplexes a listener (new connections) together with the connected client peers, so a
service loop can grow/serve/shrink in one blocking call. The committed RED probe
`tests/nursery/probe_arc209_c0b1b_select_listener.rs` is the gate (a service that accepts two
dynamically-connecting clients, round-trips `n*2` through each, then stops — returns 24). The
existing **1-arg** `(select' peers) -> Tuple<i64,O>` is UNTOUCHED (brackets depends on it). The
listener mechanic is the crux: `select'` must **peek** the listener (signal `:Connection` WITHOUT
consuming the connect-request — the loop's `accept'` consumes it) while **consuming** a ready
peer's message (→ `:Message`/`:Closed`). That needs a peek-without-consume on the comms Select,
which doesn't exist yet — you add it.

## Read in order (the rooms)

1. **`tests/nursery/probe_arc209_c0b1b_select_listener.rs`** — the gate. The exact `serve` loop
   shape + the 4 match arms (`:Connection`→`accept'`+conj, `:Message`→serve, `:Closed`/`:Lost`→
   `remove-at`) and the client choreography. RED at HEAD on `select'` arity (1 vs 2) +
   `SelectEvent::*` unknown.
2. **`docs/arc/2026/05/209-defservice/DESIGN-STONE-C0b.1b-select-grows-listener.md`** § "⚙ FINAL
   SHAPE" — the authoritative `SelectEvent` definition + the variant rationale.
3. **`src/comms/thread.rs:213-294`** `Select` — `recv(rx)` registers an arm; `select()` builds a
   fresh crossbeam `Select`, wires a shutdown arm first, **consumes** the fired op. You add a
   `ready()` that peeks (no consume).
4. **`src/runtime.rs` `eval_peer_select_prime`** (~`:23049`; the 1-arg form) + its **C0 bare-`Peer'`
   arm** (~`:23318`, "a service select's over the server ends") — the precedent: connection peers
   are `PEER_TYPE_PATH` (bare `Peer { tx, rx }`); select registers each peer's `.rx`. Branch on
   arity for the 2-arg form.
5. **`src/runtime.rs` `eval_listener_prime`/`eval_accept_prime`** (`:17853`/`:17971`) — the
   listener is a `Value::wat__kernel__Receiver` (the rendezvous of connect-requests); a connection
   peer is `make_rust_opaque(PEER_TYPE_PATH, ThreadOwnedCell<Peer{tx,rx}>)`.
6. **`src/runtime.rs` `thread_died_error_panic`** (`:19913`) — the precedent for constructing a
   stdlib-defenum `Value::Enum(EnumValue { type_path, variant_name, fields })`. Mirror it to build
   `SelectEvent::Connection/Message/Closed`.
7. **`src/check.rs` `infer_select_prime`** (`:10554`) + `infer_accept_prime` (`:9922`, the
   `Listener'<S,R>` match) — mirror for the 2-arg type scheme.
8. **`wat/spawn.wat`** — where `SelectEvent` is declared (alongside the concurrency surface).

## Implementation sketch (fill it; don't invent the shape)

**1 — `wat/spawn.wat`: declare the defenum.**
```
(:wat::core::defenum :wat::kernel::SelectEvent<O>
  :Connection
  :Message [idx <- :wat::core::i64  msg   <- :O]
  :Closed  [idx <- :wat::core::i64]
  :Lost    [idx <- :wat::core::i64  cause <- :wat::kernel::Failure])
```

**2 — `src/comms/thread.rs`: add `ready()` to `Select`** (peek, no consume). Mirror `select()`'s
crossbeam build (shutdown arm first, then user arms), but call `inner.ready()` (block until an arm
is ready, returns its arm index — does NOT recv). Map back: shutdown arm → a `Shutdown` outcome;
else `Ready(ReceiverIndex(user_pos))`. Return a small enum (e.g. `ReadyOutcome::{Ready(ReceiverIndex), Shutdown}`).
**Do not consume.** (crossbeam `Select::ready()` exists.)

**3 — `src/runtime.rs` `eval_peer_select_prime`: branch on `args.len()`.** 1 → existing path. 2 →
the thread/bare-`Peer'` 2-arg arm:
- Eval `args[0]` (listener) → expect `Value::wat__kernel__Receiver` → its inner `&Receiver<Value>`.
- Eval `args[1]` (peers) → `Vec` of `PEER_TYPE_PATH` opaques → guard each → `&peer.rx`.
- Build a `comms::thread::Select`; register the **listener rx first (user index 0)**, then each
  peer `.rx` (user indices `1..=N`).
- `sel.ready()`:
  - `Shutdown` → the existing MalformedForm "interrupted by shutdown".
  - `Ready(0)` (listener) → construct `SelectEvent::Connection` — **do not recv** the listener
    (the loop's `accept'` consumes the connect-request).
  - `Ready(k)`, `k>0` (peer `k-1`) → `peers[k-1].rx.try_recv()`:
    - `Ok(v)` → `SelectEvent::Message { idx: k-1, msg: v }`.
    - `Err(Disconnected)` → `SelectEvent::Closed { idx: k-1 }`.
    - `Err(Empty)` → spurious readiness; loop back to `ready()` (rare but possible — do not
      treat as an error).
- Construct the `SelectEvent` variant `Value::Enum` (mirror `thread_died_error_panic`'s
  `EnumValue { type_path: ":wat::kernel::SelectEvent", variant_name, fields }`).
- **`:Lost` is NOT emitted at the thread tier** (crossbeam can't reset) — it exists in the enum
  for the remote tier. Do not synthesize it here.

**4 — `src/check.rs` `infer_select_prime`: branch on arity.** 2 args → `args[0]` reduces to
`Listener'<S,R>` (mirror `infer_accept_prime:9953`); `args[1]` is `Vector<Peer'<I,O>>` (existing
extraction). Return `Parametric { head: "wat::kernel::SelectEvent", args: vec![O] }` where `O` is
the peers' recv-type (the same slot the 1-arg form returns).

## Blast radius

`src/comms/thread.rs` (add `ready()` + its outcome enum), `src/runtime.rs` (`eval_peer_select_prime`
2-arg arm + `SelectEvent` construction), `src/check.rs` (`infer_select_prime` 2-arg arm),
`wat/spawn.wat` (the defenum). **Do NOT** touch the 1-arg `select'` path, `listener'`/`connect'`/
`accept'`, the process tier, or `comms::process`.

## STOP triggers (surface the gap; do not improvise past them)

1. If `crossbeam_channel::Select::ready()` cannot be used to peek without consuming (e.g. the
   borrow shape fights the fresh-per-call build) — STOP and surface it; do NOT make `select'`
   consume the listener (that breaks the `accept'` handshake — a contract change for the Inquisitor).
2. If the listener (`wat__kernel__Receiver`) and the bare `Peer'` `.rx` are not both
   `comms::thread::Receiver<Value>` (so they can't co-register in one `Select`) — STOP and report
   the exact types.
3. If constructing a `Value::Enum` for the stdlib `SelectEvent` defenum has no clean path (the
   defenum isn't registered when `eval` runs) — STOP; do not fall back to a Tuple.

## Gate (run each, READ the output, report the verbatim final line — do NOT commit)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`
   → **1 passed** (returns 24).
2. `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1`
   → **1 passed** (C0b.1 connection precedent intact).
3. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the known reds (the 4
   baseline + the 2 new structured-peer-death probes now green); zero NEW reds.
4. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken.
5. `cargo build --release` clean; `cargo clippy` clean in the touched files.

## Report back

The `ready()` signature + the 2-arg eval arm + the infer scheme + the defenum, as written. The
`SelectEvent` representation chosen. Verbatim final line of each gate row. Any STOP hit + exact
error. Honest deltas vs this sketch. Do **NOT** commit — the Inquisitor weighs against its own
re-run and commits.
