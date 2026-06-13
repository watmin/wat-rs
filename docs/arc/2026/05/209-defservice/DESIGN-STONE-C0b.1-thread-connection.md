# DESIGN-STONE C0b.1 — thread-tier connection: `listener'` / `connect'` / `accept'`

> First strike of the C0b connection campaign. Mechanism design:
> [`DESIGN-STONE-C0b-host-parametric-connection.md`](./DESIGN-STONE-C0b-host-parametric-connection.md);
> security model: [`DESIGN-STONE-C0b-SECURITY.md`](./DESIGN-STONE-C0b-SECURITY.md).
> Inquisitor draws; Shadowdancer (sonnet) executes; Inquisitor weighs.

## What this delivers

The three connection verbs on the **thread tier only** (crossbeam rendezvous; shared memory,
so the handle *is* the capability — **no `SO_PEERCRED`** here, that's the process tier):

- `(:wat::kernel::listener' (:wat::spawn::thread) :S :R) -> (:wat::core::Tuple Listener'<S,R> Address'<S,R>)`
- `(:wat::kernel::connect' Address'<S,R>) -> :wat::kernel::Peer'<S,R>`
- `(:wat::kernel::accept'  Listener'<S,R>) -> :wat::kernel::Peer'<R,S>`

All `#[restricted_to(":wat::kernel::")]` (defservice-generated; not user surface).

## The design call (settled) — address is custody-free; only `Peer'` ends carry custody

`ThreadOwnedCell` custody binds a **`Peer'` cell** to its minting thread. It does **not** bind a
raw `comms::thread::Sender`/`Receiver` — those are freely `Send` (the old counter service
shipped them across threads inside `AdminResp::Provisioned`; precedent). So:

- The **rendezvous** is a `make-channel`-style crossbeam channel of *connect-requests*. Its
  **Receiver** is the `Listener'` (the service accepts on it); its **Sender** is the `Address'`
  (clients connect to it). Both are raw, freely-`Send` — **no custody**, shareable anywhere.
- The **connection** is two crossbeam pairs (req: client→server, resp: server→client). Each end
  is wrapped into a `Peer'` cell **on its own thread** — client end in `connect'`, server end in
  `accept'`. Raw halves cross threads (`Send`); **no `Peer'` cell ever crosses.** Custody holds
  by construction.

So the flagged "address/custody" tension dissolves: the address is a `Sender` (custody-free),
the `Peer'` ends are wrapped locally. One-message handshake (client mints, ships the server's
halves — no return leg, simpler than the old Provision/Provisioned two-step).

## Mechanics (concrete)

`listener'(:thread, :S, :R)`:
- `let (tx, rx) = comms::thread::pair::<Value>()` — the rendezvous (carries a connect-request).
- Return `Tuple[ Listener' = wrap(rx), Address' = wrap(tx) ]`. (Listener'/Address' are thin
  newtypes over the rendezvous Receiver/Sender — raw, no `ThreadOwnedCell`.)

`connect'(addr: Address'<S,R>)`:
1. `let (req_tx, req_rx) = pair::<Value>()`  — client→server (carries S)
2. `let (resp_tx, resp_rx) = pair::<Value>()` — server→client (carries R)
3. **client `Peer'<S,R>`** = `make_rust_opaque(PEER_TYPE_PATH, ThreadOwnedCell::new(Some(Peer{ tx: req_tx, rx: resp_rx })))` — wrapped **on this (client) thread**.
4. send a connect-request carrying `(req_rx, resp_tx)` (the server's halves) over `addr`.
5. return the client `Peer'`.

`accept'(l: Listener'<S,R>)`:
1. recv a connect-request from `l`'s rendezvous Receiver → `(req_rx, resp_tx)`.
2. **server `Peer'<R,S>`** = `Peer{ tx: resp_tx, rx: req_rx }`, wrapped **on this (service) thread**.
3. return the server `Peer'`.

Result: client sends S / recvs R (`Peer'<S,R>`); server recvs S / sends R (`Peer'<R,S>`); a
`send'` on one reaches a `recv'` on the other. `send'`/`recv'` already handle bare `Peer'`.

The connect-request payload `(Receiver<S>, Sender<R>)` rides inside a `Value` over the rendezvous
— exactly as `AdminResp::Provisioned` shipped `(Sender<Wire>, Receiver<UserResp>)`. Precedent
holds; `Value` is `Send`.

## The one contract decision (pinned)

`connect'` mints both crossbeam pairs and ships the **server's** halves one-way over the
rendezvous; `accept'` only receives + wraps. No return handshake. (The alternative —
`accept'` mints + ships back, the old two-step — is rejected: it needs a return channel in
every request for no benefit.)

## Files touched

| File | Change |
|---|---|
| `src/runtime.rs` | eval arms + fns for `listener'` / `connect'` / `accept'` (mirror `eval_make_channel` for the pairs, `spawn.rs`'s `make_rust_opaque(PEER_TYPE_PATH, …)` for the `Peer'` wrap). New `Listener'`/`Address'` opaque newtypes (or reuse `Sender`/`Receiver` values directly — Shadowdancer picks the cleaner of the two and notes it). |
| `src/check.rs` | TypeSchemes: `listener'` (`(host,:S,:R) -> Tuple<Listener'<S,R>,Address'<S,R>>`), `connect'` (`Address'<S,R> -> Peer'<S,R>`), `accept'` (`Listener'<S,R> -> Peer'<R,S>`). Mirror `infer_peer_pair_prime`'s type-keyword → parametric shape. |
| `src/kernel/spawn.rs` | type-path consts `LISTENER_TYPE_PATH` / `ADDRESS_TYPE_PATH` if newtypes are used (mirror `PEER_TYPE_PATH`). |
| `tests/nursery/probe_arc209_c0b1_thread_connection.rs` | the RED probe (Inquisitor writes it STRIKE-READY; the gate). |

## Out of scope = affirmatively rejected (later strikes)

- **`select'` over a `Listener'`** (the multiplexed accept-loop) — Strike 1b. Strike 1's accept
  is blocking; the probe accepts one client.
- **Process tier (UDS) + `SO_PEERCRED` grant** — Strikes 2–3.
- **The defservice dispatch loop / provision / state** — Stone C.

## Gate (Inquisitor re-runs each; Shadowdancer reports, Inquisitor weighs)

1. `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` → GREEN (round-trip).
2. `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known reds, zero new.
3. `cargo test --release --test test 2>&1 | tail -3` → wat-tests unbroken (242/1).
4. `cargo build --release` clean; `cargo clippy` clean in the touched homes.

## Estimate

~80–150 lines Rust (3 eval fns + 3 schemes + the newtypes). Bounded; every primitive grounded
above. One Shadowdancer strike behind the committed RED probe.
