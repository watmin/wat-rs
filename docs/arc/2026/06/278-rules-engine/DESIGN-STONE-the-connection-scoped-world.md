# DESIGN-STONE — the CONNECTION-SCOPED WORLD: a tenant's rules are their own, and the cursor lives with them

> **RULED by the builder 2026-08-08.** *"for a mysql-esque solution we must not allow tenets to
> intermingle with their query statements… each query connection is provisioned a world and then
> they use that for their query work… we can keep this around to handle pagination."*
>
> *"if the peer goes away for any reason we drop the world for it — we'll need to handle timeouts
> later."*
>
> *"the per-connection world holder must be on an ephemeral field as we cannot express this state
> as edn."*

## Why this exists — the defect it closes

`a61056f0` shipped `(:wat::rete::core::defn …)` with its membrane firing at runtime. It registers
**globally**, into one process-wide `sym.functions`. The builder named the defect: *"many users
could define the same funcs… they must not become globals."* Two tenants defining `:usr::big?`
differently collide, and one silently wins.

`register_overlay` (`check/env.rs:311`) would make that collision a **located error** instead of a
silent clobber — which is the right guard for a LOCAL program laying defns over the base world, and
is genuinely valuable there. But loud collision is not isolation. Only a separate world makes two
tenants unable to see each other's names at all.

## The model — three things that want the same object

A connection gets a world. That is not a compromise between two designs; it is the only shape in
which all three of these work at once, and they are the same object seen from three sides:

| | why it needs connection scope |
|---|---|
| **isolation** | a tenant's names exist only in their own world |
| **the prepared statement** | the compiled rete network survives between requests — compiled once, fired many |
| **the cursor** | pagination REQUIRES state between requests; there is nowhere else to hold a position |

The third is the one that forces it. A per-*request* world cannot hold a cursor, so the pagination
design already implied connection scope whether or not anyone said so.

## ★ THIS WAS ALREADY DESIGNED — do not re-derive it

`DESIGN-service-io-budgets.md:319` — *"Output-side streaming — the two-level cursor (the
inference-explosion crux)"*, with `:max-page-bytes 524288` at `:58` (the builder's 512KB, written
down and builder-accepted). Tasks **#19** (reader tooling / lazy `Stream<Value>`) and **#20**
(composite cursor) are its pending halves.

**And it must be a COMPOSITE cursor, not a page token, for a reason specific to a rules engine.**
`wat.query/NextToken {resume-time}` resumes a scan over STORED rows. But derived facts do not exist
until the rules fire: one input page can deduce M results that themselves overflow a frame. So there
are genuinely two sequences and two positions — one in the input scan, one in the derivations from
it. That is the inference explosion, and it is why a single `sk` token cannot express the resume
point.

## The state split — A MAP, not a field (corrected 2026-08-08 by the builder)

> ⚠ **The first draft of this stone wrote `:ephemeral [world, network]` — SINGULAR — as if a service
> held one world. It holds MANY.** The builder: *"need a map of `(wat.type/HashMap [Connection
> World])` or something such that concurrent connections never stomp on each other… we create an
> entry on connection creation/rules statement and we destroy the entry on connection close."*
> A single field is the same cross-tenant stomping this stone exists to prevent, moved one level in.

```clojure
:durable   [specs  <- HashMap<ConnId, ConnSpec>]   ;; per-connection SPEC — EDN: defs + cursor position
:ephemeral [worlds <- HashMap<ConnId, World>]      ;; per-connection RESOURCES — frozen world + network
```

**`:durable` is the thunk; `:ephemeral` is the forcing of it** — R5 at the connection layer, and the
same reason the telemetry sink holds a backend *spec* durably and the live `Store` ephemerally. A
world is built from its connection's `defs`; it is never serialized, because it cannot be.

That split is not stylistic. IPC is EDN-only, so a resource has no wire representation — the
builder's *"we cannot express this state as edn"* is the whole argument, and it lands the worlds map
in `:ephemeral` by necessity rather than preference.

## ★★ THE KEY MUST BE STABLE — `idx` IS NOT

The lifecycle events already exist: `ServiceEvent::Connection peer` · `Closed idx` ·
`Lost idx cause`. **But they identify a client by POSITION, and the position is not stable.** Every
eviction path is `(:wat::std::list::remove-at selectables idx)` — `service.wat:1058`, `:1061`,
`:1352`, `:1364`. Remove client #2 of five and every client above it **shifts down one**.

**So a `HashMap<idx, World>` silently hands client #3's compiled rules and cursor to client #4 after
any disconnect.** Not a crash — a CROSS-TENANT LEAK, which is precisely the defect this whole stone
exists to close, reintroduced by the bookkeeping. This is the single most dangerous way to build
this and it is the obvious way.

**Required: a `ConnId` minted at `Connection` and NEVER reused**, with `idx` demoted to a transient
routing detail that never names anything. The substrate does not hand us this today; minting it is
part of the stone.

**And the ORDER is the correctness argument:** on `Closed`/`Lost`, resolve *idx → ConnId* against the
live list **BEFORE** the `remove-at`, then drop that entry. Resolve after the eviction and you have
resolved against a shifted list — the same bug wearing a different hat.

*Neighbourhood:* `remove-at`'s idx handling is ALREADY a tracked open item (the `service.wat:958/961`
idx-shift, the item-c ouroboros tail). This map would be its second consumer, which argues for fixing
identity properly rather than coding around the shift.

## Lifetime — ruled

**A client goes away ⇒ that client's entry is destroyed.** Created on `Connection` (or on the rules
statement), destroyed on `Closed`/`Lost`.

⚠ **The first draft got this wrong and the builder corrected it:** *"the ephemeral record is lost
when the service goes down… but this does not mean anything about a client going away."* Those are
UNRELATED events. `:ephemeral` dying with the SERVICE says nothing about per-CLIENT cleanup — a live
service leaking one world per departed connection is exactly the defect, and no ambient RAII covers
it. The create/destroy is explicit, per entry, and must be built.

**Timeouts are explicitly NOT NOW** (the builder: *"we'll need to handle timeouts later"*). Do not
add one. 24y's `NO TIMEOUT` ruling stands and its reasoning applies: the number would be a guess, and
a wedged connection should be visible rather than silently reaped.

## The four questions

| | |
|---|---|
| **Obvious?** | **YES** — one connection, one world. A reader asks "whose names are these?" and the answer is the connection they arrived on. |
| **Simple?** | **YES** — it is ONE mechanism (a world built from a definition set) at a different LIFETIME, not a second mechanism. `eval-with-defs!` already builds exactly this world per call; this lifts its lifetime. |
| **Honest?** | **YES** — and it is the point. Today's global registration LIES to a second tenant about whose `:usr::big?` they are calling. A separate world cannot tell that lie. |
| **Good UX?** | **YES** — the local streaming case is UNCHANGED (top-level `defn`s still land global, as today); the foreign case gets isolation without asking for it. |

**4 × YES.**

## What is NOT in scope

- **The wire, the transport, the service itself.** This stone is the world's LIFETIME and its state
  split. The chaos engine (#7) is the consumer.
- **Timeouts** — ruled out above.
- **The `cargo test` deadlock** — reproduced twice (`ee6770c8`, `9f6340a1`), green under `nextest`,
  tracked separately at the builder's direction.

## ⛔ STOPs

1. **STOP-1 — the local path does not change.** A top-level `(:wat::rete::core::defn …)` in an
   ordinary program keeps registering globally, exactly as `a61056f0` ships it. If this stone starts
   rewriting that path, STOP — the builder's own streaming-app case depends on it.
2. **STOP-2 — ⚠ REWRITTEN. Prove a DEPARTED CLIENT's entry is destroyed, on a LIVE service, BY A RUN.**
   The first draft asked to prove "peer-death drops the world" and gestured at `:ephemeral` dying
   with the service — the WRONG EVENT (builder-corrected). Service teardown and a client leaving are
   unrelated, and no ambient RAII covers the second. The gate: connect N clients, disconnect one,
   and show that exactly its entry is gone, the service still serves, and **the survivors' worlds
   are still their own** (a count alone would pass while the entries were silently re-keyed).
6. **STOP-6 — THE MAP IS NOT KEYED ON `idx`.** If the key is a position into `selectables`, STOP:
   `remove-at` shifts it and tenants inherit each other's worlds. Prove the key is stable across an
   eviction by disconnecting a MIDDLE client and showing a higher-indexed survivor still resolves to
   its own world. This is the defect most likely to ship green — nothing crashes when it is wrong.
7. **STOP-7 — resolve *idx → ConnId* BEFORE the `remove-at`.** After it, the list has shifted and the
   resolution is against the wrong world.
3. **STOP-3 — ⚠ THE EPHEMERAL WALL IS UNCONFIRMED.** The record claims 293.W makes "an impure field
   can only live in `:ephemeral`" compiler-enforced. A grep for that enforcement did NOT locate it.
   Before relying on it as a WALL rather than a CONVENTION, confirm it — and if it is only a
   convention, say so plainly rather than describing it as structural.
4. **STOP-4 — the cursor is COMPOSITE.** If the design collapses to a single resume token, STOP: it
   cannot express a position inside the derivations from one input page, which is the whole
   inference-explosion problem `DESIGN-service-io-budgets.md` already named.
5. **STOP-5 — do not re-derive #19/#20.** They are the read surface of this object and are already
   designed. Reconcile INTO them; do not open a parallel plan.

## The one number worth having, and it is no longer a gate

A per-connection freeze happens ONCE per connection, so `freeze_forms`'s cost leaves the hot path
and stops shaping the design. Worth measuring eventually for connection-establishment latency; NOT a
blocker, and explicitly not a reason to delay. (It WOULD have been a gate under per-request scope —
which is one more reason connection scope is the right answer.)
