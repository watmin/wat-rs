# Arc 209 — defservice, RE-GROUNDED onto the primed substrate (2026-06-12)

> **Read order:** this is the *current* build target. The original
> [`DESIGN.md`](./DESIGN.md) is preserved as the model-of-record from May 2026 (see its
> time-capsule marker). The *behavior* it specified is correct and unchanged; this doc
> re-grounds the *tooling* beneath it onto the deadlock-free primed substrate the side
> quests (arcs 214 / 259 / 249 / 251 / 256 / 257 / 258) built since.

## ⚠️ Surface update (2026-06-12 very-late) — the admin hierarchy COLLAPSED

This doc was first written mid-session (15:40) and still describes the May surface with an
`:admin`/`:user` capability split. **That split is retired.** Two realizations later in the
same session rewrote it; they are now the authority over the surface sections below.

**(1) The admin tier collapses — the substrate owns permissions.** `Admin` existed for one
job: PERMISSIONS (provision users, validate a server-id witness, gate forgeries). The
substrate now answers that directly, per tier — thread = *you hold the handle*; process =
*your pid is in my `SO_PEERCRED` allow-set* (kernel-vouched, no `/proc`, no namespaces);
remote = *your cert chains to my CA* (mTLS). A hand-rolled permission system on top of a real
one is redundant ceremony. So **`Admin`/`User` caps, `Provision`/`Deprovision`, the server-id
witness, and the `Wire Admin|User` multiplexing all collapse.** The full model is locked in
[`DESIGN-STONE-C0b-SECURITY.md`](./DESIGN-STONE-C0b-SECURITY.md) (visible socket + refuse
untrusted callers; we are NOT using namespaces).

**The flattened surface defservice generates:**

```
(:wat::service::defservice :counter
  :state :wat::core::i64
  :ops   [Get       [s <- :State]                 -> (:Tuple :State :i64)
          Increment [s <- :State n <- :i64]        -> (:Tuple :State :i64)])
```

`:state` + a **flat** `:ops` set + dynamically-connecting **substrate-verified** clients +
(optional) **per-op identity policy** (a privileged op like `Stop` gated by `pid == owner`) +
**ownership lifecycle** (the owner holds the `spawn-program'` handle, stops via RAII
drain/join — not a tier, just a held handle). The handler contract (`[s <- :State ...args] ->
(:Tuple :State ...rest)` = state-as-self) and the mutex framing below are UNCHANGED.

**(2) The service loop is NOT a new verb — it is the existing homogeneous event-loop.** No
`serve'`/`Connected|Message` verb. ONE message enum (the ops as variants); `select'` over a
homogeneous set of client `Peer'`s (plus the `Listener'` at process tier); `match` the
variant; dispatch; grow/shrink the peer vector between iterations; TCO-recur. The reference
already on disk is `crates/wat-lru/wat/lru/CacheService.wat` `loop-step` (`select req-rxs →
(idx, maybe) → match: Ok(Some req)→handle+recur · Ok(None)→remove-at idx+recur [the shrink] ·
Err→done`). The grow-mechanism differs by tier: **thread = grow-by-message**, **process =
grow-by-listener** (a ready `Listener'` → `accept'` → the new peer joins the set).

Everything below that says `:admin`/`:user`/Provision/witness is the prior surface, kept for
the reasoning trail; read it through this banner.

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
4. **The collapsed surface.** *(SUPERSEDED by the surface-update banner — `:admin`/`:user`
   are gone; the flat form is `:state` + `:ops`.)* What survives: signatures reflected from
   the handler defns; the substrate generates the protocol enum, the `select'` dispatch loop,
   and the client wrappers. What's cut: capability structs, the server-id witness, the
   `Admin|User` wire split (substrate identity replaces all three).

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

## The connection primitive — "programs ≠ channels" *(SOLVED — C0b.1 shipped)*

The one genuinely new piece, and it is now built. `spawn-program'` makes a peer by **spawning
a new program**; connecting a client needs a channel to the **already-running** service. The
answer to the original open probe (below) came back: **not a bare `Peer'`-pair constructor —
the listen/accept/connect model**, host-parametric.

- **A spawned service starts with zero clients.** It calls
  `(:wat::kernel::listener' (host) :S :R)` → `(Listener', Address')`. A client calls
  `(:wat::kernel::connect' addr)` → its client-side `Peer'`; the service `(:wat::kernel::accept'
  listener)` → the server-side `Peer'`, **each end wrapped on its own side** (custody by
  construction). Shipped for the **thread tier** at C0b.1 (`f304fa2e`); process tier
  (abstract-UDS + `SO_PEERCRED`) is C0b.2/C0b.3, not yet built.
- The **accepted `Peer'` joins the service's `select'` set** (the dynamic handle-set; grow
  TCO-recurs with it added). A client dropping → its peer **leaves the loop** (the `select'`
  set shrinks — `Ok(None)`/`remove-at`, exactly CacheService's existing shrink arm).
- There is **no separate provision/deprovision admin channel** — connection *is* provision
  (the substrate verifies the caller), disconnection *is* deprovision. The channel *is* the
  client identity; no tid-tagging.

Full mechanism + the sockets-emergent reasoning + the security model:
[`DESIGN-STONE-C0b-host-parametric-connection.md`](./DESIGN-STONE-C0b-host-parametric-connection.md)
+ [`DESIGN-STONE-C0b-SECURITY.md`](./DESIGN-STONE-C0b-SECURITY.md).

### The original open probe — ANSWERED

*Q: bare `Peer'`-pair constructor, or wrap the transport-pair makers?* Answer: neither was the
shape. C0 shipped `peer-pair'` (the same-thread degenerate case, `137362fe`); C0b reframed it
to **listen/accept/connect** because that is the only mechanism identical across thread /
process / remote. The C0b.1 probe (`probe_arc209_c0b1_thread_connection`, green at `f304fa2e`)
already proves a service `select'`ing over a *grown* set of accepted client peers. Stone C is
unblocked.

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
  **op enum** (one variant per `:ops` entry) + the **`select'` dispatch loop** (homogeneous
  over the accepted client `Peer'` set — plus the `Listener'` at process tier — `match`ing the
  op variant, grow on accept, shrink on client-drop, TCO-recur; modeled on CacheService.wat
  `loop-step`) + the **client wrappers** (one per op), over `spawn-program'` / `Peer'` /
  `listener'` / `accept'` / `select'`. Handler contract validated at expand time via
  reflection. **No `:admin`/`:user`, no Provision/Wire/witness codegen** — substrate identity
  (handle / `SO_PEERCRED` / cert) replaces them. Optional per-op identity policy (e.g. `Stop`
  gated by `pid == owner`) is the only authorization the macro emits. **Unblocked** —
  C0b.1's thread connection is the foundation; the thread tier is provable today, the process
  tier follows when C0b.2/C0b.3 land.
- **Stone D — the proof.** Author a *new* counter service as a `defservice` (adjacent; the
  legacy `counter-*` proofs stay inert). Prove the **entire loop on the thread tier**: spawn
  the service → a client `connect'`s (the set grows) → the client `Increment`s a **protected**
  scalar through its `Peer'` → the client drops (the set shrinks) → owner stops via RAII. The
  counter mechanism is trivial; the point is that the whole control+data-plane loop is now
  trivially built and deadlock-free. Ride `deftest'`. (Process-tier proof via
  `deftest-hermetic'` rides on C0b.2/C0b.3.)
- **Later — stdio on defservice.** Rebuild the universe-resident stdio services as `defservice`
  declarations; retire the hand-rolled `spawn_service_peer`.

## Scope / arc home

defservice is arc **209 reactivated**, building on arc 259's spawn substrate. Whether Stone C/D
land under a reopened 209 or fold into 259's tail is a paperwork call to settle when Stone C is
drawn; the design is the same either way.
