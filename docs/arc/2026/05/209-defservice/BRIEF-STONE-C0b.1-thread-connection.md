# BRIEF — Stone C0b.1: thread-tier connection (`listener'` / `connect'` / `accept'`)

## The work

Mint three thread-tier kernel verbs that let a service accept dynamic client connections over
an in-memory crossbeam rendezvous: `listener'` (create the rendezvous), `connect'` (a client
dials it, gets a `Peer'`), `accept'` (the service takes the next connection, gets a `Peer'`).
Each side's `Peer'` end is wrapped **on its own thread** so `ThreadOwnedCell` custody holds.
The committed RED probe `tests/nursery/probe_arc209_c0b1_thread_connection.rs` is the gate: it
round-trips a protected scalar (`5 * 2 = 10`) through a miniature service. Make it GREEN.

Full design + the settled address/custody call: `DESIGN-STONE-C0b.1-thread-connection.md`.

## Read in order (the rooms)

1. **`tests/nursery/probe_arc209_c0b1_thread_connection.rs`** — the gate; the exact verb shapes
   and the round-trip you must make pass.
2. **`src/runtime.rs` `eval_peer_pair_prime`** (grep it; arc 209 C0, commit `137362fe`) — **the
   precedent to mirror.** It mints crossbeam pairs via `comms::thread::pair::<Value>()`, builds
   `Peer { tx, rx }`, and wraps via `make_rust_opaque(PEER_TYPE_PATH, Arc::new(ThreadOwnedCell::new(Some(Peer{..}))))`.
   `connect'`/`accept'` build their `Peer'` ends the identical way.
3. **`src/runtime.rs` `eval_make_channel`** (~:17754) — how a crossbeam pair becomes wat values
   (`sender_from_comms` / `receiver_from_comms`); the rendezvous + the connect-request use these.
4. **`src/channel/inner.rs`** (`sender_from_comms`/`receiver_from_comms`, `SenderInner::Comms`/
   `ReceiverInner::Comms`) — to put a raw half into a `Value` (ship it over the rendezvous) and
   to extract the raw half back out on the other side.
5. **`src/check.rs` `infer_peer_pair_prime`** (arc 209 C0) — the type-keyword → parametric
   scheme to mirror for the three new verbs.
6. **`src/runtime.rs` dispatch** (~:4540, near `select'` / `peer-pair'`) — where the eval arms go.

## Implementation sketch (the path — fill it, don't invent the shape)

**`listener'(:thread, :S, :R)`** → `let (tx, rx) = comms::thread::pair::<Value>()` (the
rendezvous, carries a connect-request) → return `Value::Tuple([Listener', Address'])` where
`Listener'` wraps `rx` and `Address'` wraps `tx`.

**`connect'(addr)`** →
```
let (req_tx,  req_rx)  = comms::thread::pair::<Value>(); // client→server (S)
let (resp_tx, resp_rx) = comms::thread::pair::<Value>(); // server→client (R)
let client = make_rust_opaque(PEER_TYPE_PATH, Arc::new(ThreadOwnedCell::new(Some(Peer{ tx: req_tx, rx: resp_rx }))));
// connect-request = the SERVER's halves, packed into a Value Tuple:
let cr = Value::Tuple(Arc::new(vec![ receiver_from_comms(req_rx), sender_from_comms(resp_tx) ]));
addr.send(cr);                                          // one-way; no return leg
return client;
```

**`accept'(listener)`** → `let cr = listener.recv()` → unpack the Tuple → extract the raw
`req_rx` (from `ReceiverInner::Comms`) + `resp_tx` (from `SenderInner::Comms`) → return
`Peer{ tx: resp_tx, rx: req_rx }` wrapped on **this** thread.

**`Listener'` / `Address'` representation — your call, note it in the SCORE.** Cleanest typing:
new opaque newtypes (`LISTENER_TYPE_PATH`/`ADDRESS_TYPE_PATH` in `spawn.rs`, mirroring
`PEER_TYPE_PATH`). Simplest: reuse the `Sender`/`Receiver` *values* directly (the rendezvous
ends). Either is fine if the probe type-checks + runs; pick the one that's cleaner with less
new surface, and say which in the SCORE.

**Type schemes (`check.rs`):** mirror `infer_peer_pair_prime`. `listener'` :
`(host, :S, :R) -> Tuple<Listener'<S,R>, Address'<S,R>>`; `connect'` : `Address'<S,R> -> Peer'<S,R>`;
`accept'` : `Listener'<S,R> -> Peer'<R,S>`. Get the probe type-checking; if the parametric
detail fights you, the goal is the probe green — note any shape you had to relax.

## Blast radius

`src/runtime.rs` (3 eval fns + 3 dispatch arms), `src/check.rs` (3 schemes), `src/kernel/spawn.rs`
(type-path consts, only if you choose newtypes). No new files except as the design lists. Do
not touch `peer-pair'`, `send'`/`recv'`/`select'`, or the process tier.

## STOP triggers (surface the gap; do not improvise past them)

1. If a `Peer'` end cannot be wrapped on its owning thread with the `eval_peer_pair_prime`
   pattern (e.g. the raw half won't pack into a `Value`/unpack) — STOP, surface the exact type,
   do **not** route a peer cell across a thread to make it compile.
2. If the rendezvous handshake genuinely needs a return leg (a response channel) to work — STOP
   and surface it; the design says one-way (`connect'` mints, `accept'` only receives), and a
   return leg is a design change for the Inquisitor, not a fix to apply.
3. If making the verbs type-check requires changing `peer-pair'`/`send'`/`recv'`/`select'` —
   STOP; that's out of blast radius.

## Gate (run each, READ the output, report the verbatim final line — do NOT chain a commit)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` → **1 passed**.
2. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds (arc-255 ×2, undefined-builtin ×2), zero new.
3. `cargo test --release --test test 2>&1 | tail -3` → 242/1 (the 1 = `test_run_string_entry_direct`, pre-existing).
4. `cargo build --release` clean; `cargo clippy` clean in the files you touched.

## Report back

The three eval fns + schemes as written. Which `Listener'`/`Address'` representation you chose
and why. Verbatim final line of each gate row. Any STOP hit + the exact error. Any honest delta
vs. this sketch. Do **not** commit — the Inquisitor weighs against its own re-run and commits.
