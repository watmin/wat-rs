# Arc 209 — defservice, RE-GROUNDED onto the primed substrate (2026-06-12)

> **Read order:** this is the *current* build target. The original
> [`DESIGN.md`](./DESIGN.md) is preserved as the model-of-record from May 2026 (see its
> time-capsule marker). The *behavior* it specified is correct and unchanged; this doc
> re-grounds the *tooling* beneath it onto the deadlock-free primed substrate the side
> quests (arcs 214 / 259 / 249 / 251 / 256 / 257 / 258) built since.

## Why a re-grounding exists

defservice was designed in May 2026 against the old concurrency tooling and then shelved
while the substrate it stood on was rebuilt. The rebuild — the month-long deadlock
annihilation that started with arc 170 and ran through 214 and 259 — produced exactly the
idealized tooling defservice's design assumed it would have to hand-roll. The behavior we
want is tool-agnostic; this doc keeps the behavior and swaps the tooling.

**Migration discipline (load-bearing):** we do **not** rewrite in place. New work is
authored **adjacent**; the legacy services stay inertly correct on their old primitives
until the prime-drop mass refactor retires them. defservice's internals will drop the `'`
char in that same refactor. (See `feedback`-banked: author-adjacent / prime-drop strategy.)

## What is preserved (the behavior — confirmed correct)

From the FINAL locked surface (DESIGN.md § "Surface settled 2026-05-18"):

1. **State-as-self is the mutex.** Handler contract, uniform: `[s <- :State, ...args] ->
   (:Tuple :State ...rest)`. The dispatch loop owns the live state; handlers are pure
   transforms. A single loop serializing access through provisioned handles **is** mutual
   exclusion — no locks. (Rust `&mut self` / Haskell `s→(s,a)` / Erlang `handle_call`.)
2. **The select-loop + dynamic handle-set + TCO.** The service monitors a set of handles
   for admin tasks and user tasks; provision TCO-recurs with a handle **added**, deprovision
   TCO-recurs with it **removed**.
3. **The agnostic-interface invariant.** `(:counter::get user!)` returns the same `Result`
   whether the service runs on a thread, a process, or (future) a remote host. The substrate
   hides the transport.
4. **The collapsed surface.** `(:wat::service::defservice :counter :state :T :admin [Op
   :handler …] :user [Op :handler …])` — signatures reflected from the handler defns; the
   substrate generates the protocol enums, capability structs, dispatch loop, and client
   wrappers.

## What changed — the tooling beneath it

### Stones A & B already shipped (in idealized form)

| Old plan (DESIGN.md) | Status today | Shipped as |
|---|---|---|
| **Stone A** — unified `spawn-program` entry, `:tier :service state` dispatch | **subsumed** | arc 259 `spawn-program'` host-type defclause `(host prog)` — more general (any prog; dispatch on host type). defservice's generated `-start-thread`/`-start-process` become thin callers of `spawn-program' (thread)`/`(process)`. |
| **Stone B** — restrict raw `spawn-*` to substrate-internal | **shipped** | arc 259 S2d — `spawn-thread'`/`spawn-process'`/`close'` are `#[restricted_to(":wat::kernel::")]`. |

### The idealized tooling defservice (Stone C) now builds on

- **Unified `Peer'`** + `spawn-program'` + `send'` / `recv'` / **`select'`** (arc 214 #191–195) —
  transport-blind, deadlock-free, RAII close.
- **The macro engine** (arc 249 total-pure macros) + **reflection** (`signature-of-defn`,
  `extract-arg-types`/`-names`) + **generics** (251.7 / 256) + **EDN-native collections** (257).
- **Real general TCO** (the runner-loop / dispatch loop self-recurses, constant stack).

The hardest part of the old design — the **per-tier transport asymmetry** (thread
`Sender`/`Receiver` vs. process `WireResp` stdio multiplex) — is **gone.** The unified
`Peer'` provides transport-agnosticism structurally. Both tiers are `spawn-program' (host)` +
`Peer'` + `select'`; the closure-vs-forms split (thread vs. process) is the same one
`deftest'` / `deftest-hermetic'` already ship.

## The connection primitive — "programs ≠ channels"

The one genuinely new piece. `spawn-program'` makes a peer by **spawning a new program**;
provisioning needs a channel to the **already-running** service. So:

- **A spawned service starts with zero users.** Granting access (`Provision`) mints a
  **net-new transport pair** — crossbeam (thread) / pipe (process) / socket (remote, when it
  arrives) — **adjacent to the admin channel** (the admin channel controls lifecycle only).
- The **far end joins the service's `select'` set** (the dynamic handle-set; provision
  TCO-recurs with it added). The **near end is the grantee's service-client** handle, to use
  as it needs.
- **Deprovision revokes the client**: the far-side handle **leaves the TCO loop** (the
  `select'` set shrinks).

This is the handle-set model — **per-client bidirectional channels**, `select'`-multiplexed.
The channel *is* the client identity; no tid-tagging needed.

### Open probe for Stone C (the disconfirming test to write first)

Is there a bare **`Peer'`-pair constructor** (a "connect to a running service without
spawning" primitive), or does `Provision` wrap the existing transport-pair makers
(`make-channel` for crossbeam — still live; a pipe pair for process; socket accept for
remote)? Write the 10-line probe that establishes a fresh client↔service `Peer'` pair on the
thread tier and proves the service can `select'` over a *grown* set. Build Stone C only after
this is green.

## The stdio convergence — the proof beyond the counter

The universe-resident stdio services (`src/services/`) **are defservice, hand-rolled in Rust**:

| `spawn_service_peer` (peer.rs) | defservice equivalent |
|---|---|
| `ServiceMsg::{Req, Register, Deregister}` | user op / **provision** / **deprovision** |
| `ReplyRegistry: HashMap<tid, reply_tx>` | the dynamic **handle-set** |
| `resource` (loop-owned, threaded per call, never in the req) | `s <- :State` (state-as-self) |
| "EVERY Req gets a reply — the lock is the loop body, the RELEASE is the ack send" | **the mutex** |
| panicking handle kills the loop; blocked callers get `Err` | structured failure |

The stdio shape is the **older variant**: a *shared* request channel + *tid-tagged* routing +
a `HashMap` registry + an **ambient (thread-local)** client (`THREAD_IO`). defservice's model
is the cleaner generalization: per-client channels + `select'` + an **explicit** client handle.

**End state:** once defservice is proven on the counter, the stdio services rebuild on it —
the ambient `THREAD_IO` layer becomes a thin wrapper that installs a provisioned service-client
into the thread-local, and the hand-rolled `spawn_service_peer` retires into a generated one.

## The re-grounded stone plan

- **Stone C — mint `:wat::service::defservice` (pure wat, `wat/service.wat`).** Generates the
  protocol enums + capability structs + the `select'` dispatch loop (over the admin handle +
  the provisioned client handle-set, TCO-recurring on provision/deprovision) + the client
  wrappers, over `spawn-program'` / `Peer'` / `select'`. Handler contract validated at expand
  time via reflection. **Gated by the connection-primitive probe above.**
- **Stone D — the proof.** Author a *new* counter service as a `defservice` (adjacent; the
  legacy `counter-*` proofs stay inert). Prove the **entire loop**: spawn → admin grants a
  user access → the user increments a **protected** scalar (server-id witness) → deprovision →
  stop. The counter mechanism is trivial; the point is that the whole control+data-plane loop
  is now trivially built. Ride `deftest'`.
- **Later — stdio on defservice.** Rebuild the universe-resident stdio services as `defservice`
  declarations; retire the hand-rolled `spawn_service_peer`.

## Scope / arc home

defservice is arc **209 reactivated**, building on arc 259's spawn substrate. Whether Stone C/D
land under a reopened 209 or fold into 259's tail is a paperwork call to settle when Stone C is
drawn; the design is the same either way.
