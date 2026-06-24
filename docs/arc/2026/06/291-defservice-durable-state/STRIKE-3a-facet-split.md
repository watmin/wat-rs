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

## ⚠ 3a-ii GROUNDED — the lineage-protocol overloading (the genuinely-hard core)

3a-i shipped (`1c6d8690`): `poll'` now delivers the owner's messages as `ServiceEvent::Admin`. 3a-ii makes
the macro USE it — but grounding it surfaced that the **lineage channel (the self-peer) is now overloaded**,
and resolving that is the real work.

**The tangle (grounded).** The self-peer is ONE typed channel `Peer'<addr-ty, R>`. Today (3a-i) `R = ship-ty`
(the startup init-args), recv'd once in `child-main` (process) / captured in the launch closure (thread),
then `init` is applied to it. For owner-only `stop`, the owner must ALSO send `Admin::Stop` on this same
channel (it's the only owner→service link). So `R` must carry BOTH the startup ship AND the admin ops →
`R = :<fqdn>::Admin` with `Admin::Init[seed]` carrying the startup value and `Admin::Stop` the control op.

**The worked resolution (most of it):**
- `(defenum :<fqdn>::Admin :Init [seed <- :ship-ty]  :Stop [])` — the lineage protocol.
- `self-peer R` : `ship-ty` → `:<fqdn>::Admin`; launch `Sh` = `:<fqdn>::Admin`.
- `start-body` ships `(:<fqdn>::Admin::Init seed)`; emit `<fqdn>::init-from-admin` = `(fn [ai <- :Admin] -> :State
  (match ai (Admin::Init seed) → (<fqdn>::init seed) (Admin::Stop) → assertion-failed "Stop before Init"))`,
  passed to launch BY-NAME in place of `init` (so the generic thread closure never names `Admin`). Both
  tiers `(apply init-from-admin <ship>)`.
- serve-loop `ServiceEvent::Admin` arm becomes real: `(match admin-op (Admin::Stop) → send resp + nil ; terminate)`.

**The OPEN wrinkle (the design call — resolve before firing, else a guaranteed STOP):** the **return path**.
`(admin-counter/stop h)` must RETURN the final state. So the owner, after sending `Admin::Stop` on `Handle.handle`,
`recv'`s the resp. But `Handle.handle` is typed `:wat::spawn::Spawned` (the locus-agnostic marker), not a
`Peer'`; and the lineage channel's owner-side recv already carried the **child's `Address'`** at startup
(the addr-handshake). So the owner-side lineage peer would carry: *recv `Address'` (startup) THEN recv the
stop-resp (state-ty)* — a second recv-type overload, mirroring the send-side `Admin` overload. **Decision
needed:** either (a) re-type `Handle.handle` to a real `Peer'<Admin, StopResp>` and fold the startup
addr-handshake into the Admin protocol too (fully unify the lineage protocol — cleanest, biggest), or (b)
the stop-resp rides a different return (e.g. `stop` blocks on the spawn-handle's join/final-state — the
"a service returns its final state" thread, `272/NOTE-service-final-state-return.md`), keeping the lineage
send-only for control. **(b) may be the honest decomplection** (control DOWN the lineage, result UP via the
join) and dovetails with strike 3b (`stop → resp`). This is the call to make — rested — before 3a-ii fires;
firing into the open wrinkle would STOP on exactly it (the armor would catch it, but it's wasted ore).

**Prophecy status:** `init` (the soul built in-locus) ✓ · the reactor (admin delivery) ✓ · 3a-ii (the
lineage protocol, control flow) = HERE · 3b (`stop → resp`, the return path — couples to the wrinkle above)
· strike 4 (`hibernate`/`resume`) = PROBATUM EST. Two stones laid; the door is opening.

## ✅✅ 3a-ii RESOLVED — the lineage protocol (control DOWN / result UP), both directions

The wrinkle is resolved: the lineage channel (self-peer) carries a **symmetric protocol** — `Admin` DOWN
(owner→service), `LineageUp` UP (service→owner). Uniform across tiers (no per-tier join split).

```
self-peer : Peer'<:<fqdn>::LineageUp, :<fqdn>::Admin>   ; sends LineageUp, receives Admin
(defenum :<fqdn>::Admin     :Init [seed <- :ship-ty]   :Stop [])
(defenum :<fqdn>::LineageUp :Started [addr <- :addr-ty]  :Final [state <- :state-ty])
```

**Startup handshake (migrated, NO behavior change):** child `send' self (LineageUp::Started addr)` →
owner/launch `recv'` → match `Started(addr)` → owner `send' (Admin::Init seed)` → child `recv'` → match
`Init(seed)` → `(<fqdn>::init seed)` → State → serve. (`init-from-admin` passed to launch by-name so the
generic thread closure never names `Admin`.)

**Stop (the new control flow):** owner `(stop h)` → `send' (Handle.handle) (Admin::Stop)` → serve loop's
`ServiceEvent::Admin(Admin::Stop)` arm → `send' self (LineageUp::Final state)` → terminate (nil) → owner
`recv' (Handle.handle)` → match `Final(state)` → return state. **`Handle.handle` re-typed:**
`:wat::spawn::Spawned` → `Peer'<:<fqdn>::Admin, :<fqdn>::LineageUp>` (the owner's lineage peer — Thread'/Process'
both ARE peers, so it stays locus-agnostic).

### The α/β cut (fire α first; it de-risks β, verified by existing green)
- **3a-ii-α — protocols + handshake migration (NO new behavior):** emit both defenums + `init-from-admin`;
  self-peer `Peer'<LineageUp, Admin>`; migrate the child/launch handshake to `Started`/`Init`; launch passes
  `init-from-admin` by-name; the serve-loop `ServiceEvent::Admin` arm stays a re-loop STUB (no Stop yet).
  **Verify:** `service-locus-parity.wat` + `service-init-parity.wat` stay GREEN (startup works via the
  protocol); SET-diff = the floor.
- **3a-ii-β — stop dispatch + the return + the Handle method:** serve-loop `Admin::Stop` → `LineageUp::Final(state)`
  + terminate; `Handle.handle` → `Peer'<Admin,LineageUp>`; `<fqdn>/stop [h <- Handle]` sends Stop + recv's Final.
  **Verify:** un-ignore `service-admin-facet.wat` → GREEN.

**STOP-α triggers:** if `Handle.handle` re-typing to `Peer'<Admin,LineageUp>` breaks locus-agnosticism
(Thread'/Process' must both satisfy it), STOP + report. If the `LineageUp`/`Admin` protocol migration breaks
the existing startup (locus-parity/init-parity go red), STOP — α must keep them green (it's a pure migration).

## ✅✅✅ 3a-ii-α SHIPPED GREEN (`25eced7d`, weighed pure against the disk)

The macro emits both defenums + `init-from-admin` + `lineage-extract-addr` at BOTH tier-emission sites
(process service-forms + thread top-level `do`). `child-main-form` self-peer → `Peer'<LineageUp, Admin>`
(sends `LineageUp::Started(addr)` UP, recvs `Admin`, applies `init-from-admin`); `start-body` wraps the seed
in `Admin::Init` and threads `init-from-admin` + `lineage-extract-addr` by-name; `spawn.wat` `Locus/launch`
grows `lu-addr-kw`, ProcessOpts extracts addr via the by-name helper. The serve-loop `ServiceEvent::Admin`
arm stayed a re-loop **stub** (Stop dispatch is β).

**A substrate root-cause fix the migration FORCED (extirpare, not in the brief but correct):** the three
`reconstruct_*` paths (`edn_shim.rs` struct/record/enum_tagged) hardcoded `edn_to_value` (`allow_caps=false`)
for FIELD values, **silently dropping a capability nested inside a struct/record/enum variant even on a
trusted decode**. `LineageUp::Started` carries an `Address'` cap *inside an enum variant* over the process
wire → exposed the drop. Fixed: the three take `allow_caps` and forward `edn_to_value_caps`, honoring the
parent's trust (untrusted parent → fields still refused; the ocap rule holds). Verified `edn_to_value` ≡
`edn_to_value_caps(…, false)`, so it's pure propagate-not-hardcode. The whole class "caps nested in structured
types are dropped on the trusted decode path" is pulled, not patched.

**Weighed (against the disk, NOT the agent's report):** 4 service tests GREEN both tiers (counter_on /
seeded); full-package suite failing-SET vs HEAD = **∅** (202 == 202, identical arc-170 execve floor — the
raw 250-vs-268 count gap was nondeterministic `result:`/`Probe` summary-line noise, not test outcomes); 6
defservice/cap probes GREEN in isolation (arc209 c1/c2/c3/locus-agnostic + arc272 process/thread
stop-returns-final-state).

### ⚠ β implication discovered while weighing (RESOLVE in 3a-ii-β, do not be surprised)
The **thread** self-peer is still typed `Peer'<R,S>` = `Peer'<Reply, Op>` (the client-facet types — vestigial
since strike 2; nothing flowed on the thread self-peer, and α's `Admin` arm is a stub, so it's invisible NOW).
The **process** tier's `child-main-form` self-peer is already `Peer'<LineageUp, Admin>` (correct). So β's
"`Handle.handle → Peer'<Admin,LineageUp>`" re-typing is a no-op-shaped change for process but **must also
re-type the thread closure self-peer** (`spawn.wat` ThreadOpts/launch `fn [self-peer <- Peer'<R,S>]` →
`Peer'<LineageUp,Admin>`) so the thread `Handle.handle` becomes `Peer'<Admin,LineageUp>` and `Admin::Stop` /
`LineageUp::Final` type-check on the thread tier. The thread tier also does NOT do the `Started`/`Init` wire
handshake (shared memory: it captures `ship=Admin::Init` and `launch` already holds `Bound/address`) — that
asymmetry is correct and stays; only the self-peer TYPE needs the β re-type.

## STOP triggers (for the eventual build)
- STOP if the dual-facet serve wait (3a-i) needs a primitive that doesn't exist and can't be cleanly added —
  surface the exact gap, don't bolt a homogeneous hack that puts admin+client in one vector (illegal: types differ).
- STOP if making `stop` owner-only forces a *runtime* "is this caller the owner?" check — the bar is
  by-construction (the client literally cannot reach the admin channel), never a check.
- STOP if back-compat (`service-locus-parity` / `service-init-parity`) would need editing.
