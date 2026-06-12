# Stone C0 — the connection primitive (`peer-pair'`) + `select'` over bare `Peer'`

> Prereq for Stone C (the `defservice` defmacro). Re-grounded plan:
> [`DESIGN-REGROUNDED-2026-06-12.md`](./DESIGN-REGROUNDED-2026-06-12.md). Disconfirming
> probe (committed RED): `tests/nursery/probe_arc209_connection_primitive.rs` (`69f63053`).

## Why

defservice provisions a client by minting a **net-new, connected `Peer'` pair adjacent to
the admin channel** — the service keeps the server end in its `select'` set; the grantee
gets the client end; deprovision drops the server end from the TCO loop. "Programs ≠
channels" (builder, 2026-06-12): `spawn-program'` makes a peer by *spawning a new program*;
provisioning needs a channel to the *already-running* service.

The probe proved the gap precisely: `UnknownFunction(":wat::kernel::peer-pair'")` on exactly
the mint line — every other primitive (`send'`/`recv'`/`select'`) resolves. The only way to
get a `Peer'` today is to spawn.

## What it delivers (thread tier)

Two small, additive substrate changes:

1. **`peer-pair'` — mint two connected, crossed `Peer'` ends without spawning.**
2. **`select'` over bare `Peer'`** — add the `PEER_TYPE_PATH` branch (today `select'` accepts
   only `Thread'` / `Process'`; `send'` / `recv'` already accept bare `Peer'`).

When both land, the probe round-trips a request/response over a non-spawned pair and `select'`s
the server end → green. defservice's `provision` (Stone C) wraps `peer-pair'`.

## The one contract decision (pinned)

```
(:wat::kernel::peer-pair' :S :R) -> (:wat::core::Tuple :wat::kernel::Peer'<S,R> :wat::kernel::Peer'<R,S>)
```

- Returns **both ends, crossed**: end A is `Peer'<S,R>` (sends `S`, recvs `R`); end B is
  `Peer'<R,S>` (sends `R`, recvs `S`). A's send reaches B's recv and vice-versa. For the
  symmetric counter (`S = R = i64`), both ends are `Peer'<i64,i64>`.
- **No spawn, no thread, no join handle, no crash channel** — a bare connected pair has no
  worker behind it. (Construction mirrors `spawn_thread_peer` *minus* the spawn.)
- **Thread tier only** in this stone (crossbeam). See § Out of scope.
- Name is intueri-pending — `peer-pair'` is the working name for the raw primitive; the
  defservice-level `provision` is the higher-level op that wraps it.

## Construction (grounded — mirror `spawn_thread_peer` minus the spawn)

`Peer<S,R>` is `{ tx: comms::thread::Sender<S>, rx: comms::thread::Receiver<R> }`
(`src/kernel/peer.rs`). `spawn_thread_peer` (`src/kernel/spawn.rs`) makes two crossbeam pairs
and crosses them: the worker's `self_peer = Peer { tx: output_tx, rx: input_rx }`; the parent
`Thread { input: input_tx, output: output_rx, … }`.

`peer-pair'` does the same crossover, both ends in hand, no thread:

```text
let (a_tx, a_rx) = comms::thread::pair::<Value>();   // end A → end B
let (b_tx, b_rx) = comms::thread::pair::<Value>();   // end B → end A
let end_a = Peer { tx: a_tx, rx: b_rx };             // sends on A, recvs on B
let end_b = Peer { tx: b_tx, rx: a_rx };             // sends on B, recvs on A
// wrap each: make_rust_opaque(PEER_TYPE_PATH, Arc::new(ThreadOwnedCell::new(Some(end))))
// return Value::Tuple([wrapped_a, wrapped_b])
```

`select'` over `Peer'`: add a third `first_type_path == PEER_TYPE_PATH` branch to
`eval_peer_select_prime` (`src/runtime.rs:22682`), mirroring the `THREAD_PEER_TYPE_PATH`
branch — the thread-tier crossbeam-select machinery reads each peer's receiver; for a bare
`Peer'` that receiver is `Peer.rx` (vs. the `Thread.output` the Thread' branch reads).

## Files touched

| File | Change |
|---|---|
| `src/runtime.rs` | register `:wat::kernel::peer-pair'` eval arm (near the spawn/peer verbs ~4538) + the new fn; add the `PEER_TYPE_PATH` branch to `eval_peer_select_prime` (:22682) + update its error enum string to `Thread'<I,O> \| Process'<I,O> \| Peer'<I,O>` |
| `src/check.rs` | register the `peer-pair'` TypeScheme: `(:S :R) -> (Tuple Peer'<S,R> Peer'<R,S>)` (generic, mirror the `select'` / `spawn-program'` scheme registration sites) |
| `tests/nursery/probe_arc209_connection_primitive.rs` | already committed RED — this is the gate (goes green) |

`Peer'<S,R>` is already a registered wat type (deftest' bodies bind `self <- Peer'<i64,i64>`),
so no new type registration is needed — only the verb scheme.

## Out of scope = affirmatively rejected (not deferred)

- **Process / remote connection.** A bare pair with *both ends in hand* is a shared-memory
  (thread) concept. For process (pipe) and remote (socket) the far end lives in **separate
  memory** — provisioning hands the far end *across* the boundary, a genuinely **different
  mechanism** (pipe-fd handoff / socket accept), not this verb extended. That is the
  `:remote`-class forcing function — built toward, perpetually away. Stone C0 ships the thread
  tier, where the counter proof and the stdio services live; the separate-memory connection
  model arrives with `:remote`. (`[[feedback_dont_build_the_forcing_function]]`.)
- **The defservice dispatch loop / `provision` wrapper / TCO-grow of the `select'` set.** That
  is Stone C — it *consumes* `peer-pair'` + `select'`. Not here.

## Gate (run each, READ output, then commit — never chain test+commit)

1. `cargo test --release -p wat --test nursery probe_arc209_connection_primitive -- --test-threads=1` → **GREEN** (round-trips to 84).
2. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds (arc-255 ×2, undefined-builtin ×2), zero new.
3. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken (242/1 baseline).
4. `cargo build --release` clean; `cargo clippy` clean in `src/kernel/` + the touched runtime arms.

## Estimate

~30–50 lines Rust (the mint fn + the select' branch + the scheme). Additive; bounded; every
primitive it composes is grounded above. A clean single sonnet strike behind the committed
probe — or inline; orchestrator's call at fire time.
