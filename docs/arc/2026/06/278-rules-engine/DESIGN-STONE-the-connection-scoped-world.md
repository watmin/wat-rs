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

## The state split — and the builder's ruling makes it structural

Established convention, exactly the telemetry service's shape
(`DESIGN-telemetry-service-and-query-surface.md:218`, `:ephemeral [store <- …]`):

```clojure
:durable   [defs         <- :wat::core::Vector<wat::WatAST>   ;; the SPEC — EDN, hibernates, ships
            cursor       <- (:Option Cursor)]                 ;; a POSITION is data
:ephemeral [world        <- <the frozen world>                ;; a live resource — NOT EDN
            network      <- <the compiled rete network>]      ;; likewise
```

**`:durable` is the thunk; `:ephemeral` is the forcing of it** — R5 at the connection layer, and the
same reason the telemetry sink holds a backend *spec* durably and the live `Store` ephemerally. The
world is rebuilt from `defs` in `:init`; it is never serialized, because it cannot be.

That split is not stylistic. IPC is EDN-only, so a resource has no wire representation — the
builder's *"we cannot express this state as edn"* is the whole argument, and it lands the world in
`:ephemeral` by necessity rather than by preference.

## Lifetime — ruled

**Peer dies ⇒ the world is dropped.** Nothing else. A `defservice`'s `:ephemeral` is thread-owned
and dies with the service, so this should be close to free rather than a mechanism to build — but it
is a claim to PROVE by a run, not to assume (STOP-2).

**Timeouts are explicitly NOT NOW** (the builder). Do not add one. 24y's `NO TIMEOUT` ruling stands
and its reasoning applies here too: the number would be a guess, and a wedged connection should be
visible rather than silently reaped.

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
2. **STOP-2 — prove peer-death drops the world BY A RUN.** "`:ephemeral` dies with the service" is
   the expectation, not the evidence. A leaked world per dead connection is the defect this stone
   would be creating.
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
