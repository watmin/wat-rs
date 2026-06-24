# Arc 291 — Strike 3a: the admin/data facet split (the structural decomplection)

**Goal.** Make `stop` **owner-only by construction**: move it off the client `Op` enum onto the Handle's
admin surface, so its caller arg flips `client-peer` → `Handle` and a client holding only the dial-`Address'`
has no `stop` at all. RED probe: `wat-tests/service-admin-facet.wat` (verified RED at HEAD — `stop`
parameter #1 wants a client `Peer'`, got `Handle`). This is **3a** (the structural facet); `stop → resp`
decouple is **3b**.

## The contract (pinned)
- `stop` (and later `hibernate`) live on an **admin surface reachable ONLY via the `Handle`**, never via the
  dial-`Address'`. The client face has **no** `stop` method — security by construction (an unforgeable,
  never-handed capability), not a runtime "is this admin?" check (the deleted-first-attempt bar; see
  `DESIGN.md` archaeology + avoid-list).
- `stop`'s surface: `<svc>/stop [h <- <svc>::Handle] -> <state-ty>` (3a keeps the state return; 3b decouples it).
- The Handle exposes only the **client** address via `Handle/addr`; the **admin** reach is private to the Handle.
- The avoid-list holds (verified mistakes from the deleted first attempt): no `Mutex` on the admin reach
  (single-owner → `ThreadOwnedCell`/ownership); the admin address is **never exposed** + **pid-stamped**
  (`OnlyThisPeer{minter_pid}`), never "unguessable"; **two channels, never one multiplexed with a runtime
  check**.

## ⚠ The trap (grounded) — the serve loop is single-facet; 3a needs a foundation step first

The serve loop today (`service.wat:482-495`) is a **single-facet `poll'`**:
```clojure
serve [self l clients state]
  (match (poll' self l clients) -> nil
    ((ServiceEvent::Accepted peer)   (serve self l (conj clients peer) state))
    ((ServiceEvent::Message idx req) …dispatch client op…)
    ((ServiceEvent::Disconnected idx) (serve self l (remove-at clients idx) state))
    ((ServiceEvent::Closed idx)       (serve self l (remove-at clients idx) state)))
```
`poll'` watches **one** listener `l` + **one homogeneous** `clients` vector (`Vector<Peer'<Reply,Op>>`). The
facet split needs the loop to wait on **two facets whose `Op` types differ** (`client::Op` ≠ `admin::Op`) —
and a homogeneous vector + single-listener `poll'` cannot hold both. **Both facets mutate the same `State`,
so they MUST serialize through this one loop** (the lock-free-mutex / state-as-self invariant) — the split is
*which channel*, never a second loop. So 3a cannot be "add a listener"; the serve loop's wait must first
grow to multiplex two heterogeneous facets.

## The decomposition (forced by the trap)

- **3a-i — FOUNDATION: the dual-facet serve wait.** Extend the serve loop to wait on the client facet
  (listener + `client::Op` peers) AND an admin facet (`admin::Op`) in one wait, returning a **facet-tagged**
  event, sharing `state`. This is a real primitive-level decision (see the fork below) and must land before
  the macro reshape. Likely touches the `poll'`/`select'` machinery (possibly Rust).
- **3a-ii — the macro reshape (consumes 3a-i).** Split the generated `Op`/`Reply` into a **client** enum
  (user ops) and an **admin** enum (`Stop`); mint the admin reach; the serve loop uses the 3a-i dual-facet
  wait; the `Handle` holds the admin reach; `stop` becomes `<svc>/stop [h <- Handle]` (dials the admin
  surface). Un-ignores the RED probe → GREEN.

## The fork to settle (3a-i) — how the owner reaches the admin facet

| | **A — admin LISTENER + `Address'`** | **B — pre-established admin PEER** |
|---|---|---|
| Handle holds | the admin `Address'` (a portable capability) | a live admin `Peer'` (set up at launch) |
| serve loop | accepts admin connections (dual-listener `poll'`) | watches one admin peer (no admin accept) |
| delegation | **easy** — hand the admin `Address'` (272 `#wat-edn.cap`); the orchestrator←daemon vision needs this | hard — transferring a live peer is harder than an address |
| cost | dual-**listener** accept + dual-facet wait | dual-**facet** wait only (no second accept) |

Four-questions lean: **A** is the vision's target (the admin authority must be a *delegable capability* —
the whole orchestrator/daemon story rests on the daemon handing the admin `Address'` up). **B** is the
smaller first step (no admin accept loop) but defers delegation and still needs the dual-facet wait. My
recommendation: **build the dual-facet wait (3a-i) general enough to serve A**, but the *first* macro
reshape (3a-ii) MAY ship B's single-peer form if A's dual-listener accept proves too big for one strike —
decide at 3a-i. **This is the one decision that needs the builder's confirm before the build is briefed.**

## RED probe (committed, verified)
`wat-tests/service-admin-facet.wat` — a counter; `Increment` is a client/data op; `stop` is admin/control.
A client (dial-`Address'`) increments; the Handle-holder calls `(admin-counter/stop h)`. RED at HEAD:
`stop` wants `client-peer-ty`, got `Handle` (both tiers, `:40:46` + `:53:46`). Ignore-marked; strike-3a-ii
un-ignores it green.

## Done = the gate (3a)
`service-admin-facet.wat` (un-ignored) GREEN both tiers: `(admin-counter/stop h)` takes the Handle, dials
the admin surface a client cannot reach, returns the final state (count 7). `service-locus-parity.wat` +
`service-init-parity.wat` stay green (back-compat). Full wat-tests SET-diff vs HEAD ⊆ the known floor.

## ✅ Fork RESOLVED + mechanism grounded (build opened 2026-06-23)

**Fork → Option B, specialized: the admin channel IS the spawn lineage peer.** When a service is spawned,
272's lineage handshake establishes an owner↔service channel only the spawner holds (the inherited
capability — owner-only by construction). Grounded: the spawn handle is `Thread'<R,S>` / `Process'<I,O>` —
**a peer** (`spawn.rs:331-336`, `ProcessSelectable::Spawned`), already carried as `Handle.handle`. So the
admin reach is **not a new listener** — it is the lineage peer the spawn already minted, kept live in the
Handle. No admin accept loop; delegation (a connectable admin `Address'`) defers until a remote caller forces
it (don't build the forcing function). `stop`/`hibernate` route on this peer.

**3a-i mechanism, grounded.** Today the serve loop is `(poll' self l clients)` → `ServiceEvent<I,O>` — a
3-arg, single-facet, homogeneous multiplexer (`check.rs:11448-11567`). `select'`
(`runtime.rs:24542` `eval_peer_select_prime`) is the heterogeneous N-peer wait, but peer-only (no accept).
3a-i = the serve loop must wait on the **client facet** (`poll'`: listener + `Vector<Peer'<client::Op>>`)
AND the **admin lineage peer** (one `Peer'<admin::Op>`) together, one loop, shared `State`. Two candidate
shapes (decide at the strike): (a) **extend `poll'`** to a 4-arg facet-tagged wait `(poll' self l clients
admin-peer)` → a `ServiceEvent` tagged client|admin; (b) a **heterogeneous serve loop** over `select'` +
accept handling. (a) is the smaller delta (one extra arm on the existing `ServiceEvent` machinery); lean (a).
**This is a reactor-level Rust change** (io_uring/crossbeam `poll'` + the `ServiceEvent` type + the macro).

**Honest pacing note (slow is smooth):** the build is open, scoped, fork-resolved, mechanism-grounded, RED
probe committed — but 3a-i is a delicate reactor change (the comms multiplexer), the biggest of the arc, and
the right kind of build to FIRE rested, not at the tail of a marathon prose session. STRIKE-READY for a
fresh fire.

## ✅✅ 3a-i GROUNDED + BRIEFED (the reactor change is surgical) — fire this first

**The keystone grounding:** `eval_poll_prime` arg0 is the **self-peer** (the owner↔service link), and it is
**already index 0 in the blocking wait** (`runtime.rs:25214-25261`). Today index-0 fire → `ServiceEvent::
Shutdown` *unconditionally* (`:25451-25463`, "Do NOT inspect result"). The admin channel is already watched;
we just stop discarding its messages. **A stop from the owner is a `recv'` Ok(msg) on the self-peer.**

**3a-i contract:** at index 0, inspect `result` — **`Ok(msg)` → `ServiceEvent::Admin{msg}`** (owner sent an
admin op), **`Err(_)` → `ServiceEvent::Shutdown`** (owner dropped — unchanged). Mirror at the process tier
(`~25530`, `ReactorClass::Fd`). The admin msg is a **third type** `A` (the self-peer's receive type,
independent of the client `I,O`), so `ServiceEvent<I,O>` → **`ServiceEvent<I,O,A>`** with `Admin[msg <- :A]`.

**Why the cascade is bounded (verified):** `ServiceEvent` is `(defenum :wat::spawn::ServiceEvent<I,O> …)`
at `spawn.wat:125`. The ~89 `ServiceEvent` hits are mostly **match patterns** (`ServiceEvent::Message idx
op`) that do NOT carry type params → unaffected by a 3rd param. The Rust constructions build
`EnumValue{type_path, variant_name, fields}` by NAME → unaffected. The real edits: (1) the defenum
(`spawn.wat:125`, add `,A` + `Admin [msg <- :A]`); (2) `infer_poll_prime` (`check.rs:5034`/`:11448`) — type
`A` from arg0's (self-peer) receive type, expose `Admin`; (3) `eval_poll_prime` thread tier
(`runtime.rs:25451-25463`) + process tier (`~25530`) — the `Ok(msg)→Admin` / `Err→Shutdown` split.

**Rooms (read in order):** `runtime.rs:25205-25261` (poll' arg parsing) → `:25440-25518` (thread event
construction, the index-0 branch) → `:25521-~25560` (process tier mirror) → `wat/spawn.wat:125` (the
defenum) → `check.rs:5034` + `:11448-11567` (`infer_poll_prime` + the `ServiceEvent` typing).

**Verify 3a-i (Rust probe — it has no clean wat surface until 3a-ii):** a `tests/` probe that builds a
self-peer pair, sends an admin `Value` from the owner end, calls the poll'/service-loop path, asserts the
event is `ServiceEvent::Admin{msg}` (and a dropped-owner still yields `Shutdown`). Model on
`tests/probe_arc209_c0b3aii_process_service_loop.rs`. ALL existing tests stay green (the SET-diff floor).

**STOP-i:** if adding the 3rd param `A` breaks existing `ServiceEvent::*` **match sites** (it should NOT —
patterns are param-free), STOP and report which — the design assumes matches are unaffected. Do NOT widen
`Admin`'s msg to `:wat::core::Value` to dodge the param (that reintroduces a down-cast — R7/255's checked
firewall; the typed `A` is the point).

**3a-ii (next strike, after 3a-i weighs green):** the macro reshape — emit `<fqdn>::Admin` enum
(`Stop`; later `Hibernate`), self-peer R = `Admin`, child-main `recv'`s the first admin (the ship/init is
`Admin::Init(ship)` — unify the startup handshake with the admin channel), serve loop adds a
`ServiceEvent::Admin` arm dispatching the admin op, `stop` → `<fqdn>/stop [h <- Handle]` sending `Admin::Stop`
on `Handle.handle`. Un-ignores `service-admin-facet.wat` → GREEN.

## STOP triggers (for the eventual build)
- STOP if the dual-facet serve wait (3a-i) needs a primitive that doesn't exist and can't be cleanly added —
  surface the exact gap, don't bolt a homogeneous hack that puts admin+client in one vector (illegal: types differ).
- STOP if making `stop` owner-only forces a *runtime* "is this caller the owner?" check — the bar is
  by-construction (the client literally cannot reach the admin channel), never a check.
- STOP if back-compat (`service-locus-parity` / `service-init-parity`) would need editing.
