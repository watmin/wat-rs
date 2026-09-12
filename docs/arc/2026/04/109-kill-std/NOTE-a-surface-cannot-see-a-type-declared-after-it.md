# NOTE (arc 109 vocabulary) — a SURFACE cannot see a type declared AFTER it; the load order decides the contract

**Filed 2026-09-12, builder-directed** (*"we can note the capability one"*), found while striking
`the-gate-methods-face-an-outcome` in `docs/excursus/2026/08/001-sns-sqs/`. **CLOSED 2026-09-12 — see the closing section.**

## The fact

`wat/capability.wat` is stdlib manifest position **20**; `wat/service.wat` is **35**
(`src/load/stdlib.rs:156` vs `:341`). `:wat::service::GateOutcome` is declared in `service.wat`, so
**`capability.wat` cannot name it.**

Consequence, shipped deliberately: `Capability` / `TypedCapability`'s `grant` / `revoke` features keep
`-> :wat::core::nil`, and the auto-emitted `grantable-extend` (in `service.wat`) wraps
`:wat::service::require-granted`, which **raises** on `Gone` / `GaveUp`. So:

- the **owner methods** `<S>/grant` and `<S>/revoke` face a `GateOutcome` ✓
- the **capability tier** still raises ✗

⚠ **File this as "a surface cannot see a type declared after it", NOT as "grant still raises."** The
second framing sends the next reader to `grant`, which is already correct. The defect is the ordering.

## Why it is LOW priority — measured, 2026-09-12

**Zero userland callers.** `grep -oE '\(:wat::capability::[A-Za-z]*/(grant|revoke) '` across every
`.wat` finds exactly **2**, and both are inside `wat/service.wat` — the macro's own generated wiring
(`~acc-kw`, `~gw-handles-sym` are splice symbols). Four files mention `capability::Capability` at all,
two of which are the stdlib pair itself; the other two are arc-170 probes that name the *type* without
calling `grant` through it.

So the raising path exists for **uniformity**, not for traffic. Nothing in the corpus reaches it.

## Why the fix is probably SMALL

`GateOutcome` has **no dependency that pins it to `service.wat`**:

```
:Applied []
:Gone    [cause     <- :wat::kernel::LociDiedError]   ← Rust-registered in src/types.rs
:GaveUp  [waited-ms <- :wat::core::i64  last <- :wat::core::String]
```

`LociDiedError` is registered in Rust, so it is visible to every `.wat` regardless of load order. Two
routes, both cheap:

1. **Declare `GateOutcome` in a stdlib file that loads before position 20** (`wat/core.wat`,
   `wat/program.wat` at 19, …). Smallest diff; leaves the enum in wat.
2. **Register it in `src/types.rs`** beside `RecvOutcome` / `SendOutcome`, which is where the other
   cross-cutting outcome enums already live. More principled — an outcome enum every tier must name is
   a kernel-tier type, and this is the argument that it was mis-homed from the start.

★ (2) is the one that generalizes: **`CallOutcome` and `StopOutcome` are in `service.wat` too**, so any
future file loading before 35 hits the same wall. If the wall is going to be hit again, move the types
rather than the files.

## The generalization worth a sweep

⭑ The question this note really asks is **which stdlib types are declared later than the surfaces that
need them.** A cheap census: for each `defenum`/`defrecord` in `wat/*.wat`, compare its file's manifest
position against the earliest position of any file naming it. Any inversion is either already broken or
one refactor away from it. Nobody has run that sweep.

## ⭑ CLOSED 2026-09-12 (`fb31d3323`) — route (2), and the NOTE's "probably small" was HALF right

`:wat::service::GateOutcome` is **registered in `src/types.rs`** — the route this note ranked second and
called "more principled". Verified on disk this session: **1** registration hit in `src/types.rs`, **0**
`defenum :wat::service::GateOutcome` left in `wat/service.wat`, **5** occurrences of the name in
`wat/capability.wat`, and **zero** capability `grant`/`revoke` features still returning
`-> :wat::core::nil`. **The name did not change**, so the 28 references never moved.

The capability tier now **returns a value**, proven by
`wat-scripts/scratch-pad/probe-capability-grant-returns-gateoutcome.wat` (printing
`#wat.service.GateOutcome/Applied []` through both `Capability/grant` and `TypedCapability/revoke`) —
because with **zero userland callers** a green floor never executes this surface and is therefore silence,
not proof. Full grading: `docs/excursus/2026/08/001-sns-sqs/the-gate-outcome-outlives-its-file/SCORE.md`.

⚠ **Route (1) — "declare it in a file that loads before position 20" — was the cheaper-looking option and
it is a trap.** The strike had to touch `wat/core.wat` (position **0**) anyway, and that file cannot name
`:wat::service::require-granted` (position 34) even inside a quasiquote — the load-order gate caught it by
name, `left: 2, right: 0`. Adding lines to `core.wat` then broke **five EDN goldens** that pin macro-body
spans through it. So the "smallest diff" route pays in the file with the tightest hidden constraint. See
`NOTE-a-golden-that-pins-a-rust-line-number.md` (wat-side sibling) for that constraint.

`CallOutcome` and `StopOutcome` are **still `defenum`s in `service.wat`** — deliberately. No file needs
them earlier today; this note's ★ argument (move the types, not the files) stands as the ruling for
**when** one does.

### ⛔ The requested sweep: HALF of it is a standing gate, and the failing half is the half that bit us

This note asked for a census — *"for each `defenum`/`defrecord` in `wat/*.wat`, compare its file's manifest
position against the earliest position of any file naming it"* — and said **nobody has run that sweep**.
Measured: **that census already exists as a gate that runs on every floor.**

`:wat::deporder::verify` classifies `defenum` as eval-dep and `collect-kwds` gathers **every**
`::`-qualified keyword anywhere in a form tree — which includes a *type annotation*. Confirmed by probe
rather than by reading, because "it should see it" is the tell:
`wat-scripts/scratch-pad/probe-deporder-sees-a-type-annotation.wat`, two cells —

```
CELL-A (must fire)     a.wat@0 names :t::b::Thing in a RETURN TYPE, b.wat@1 defenums it  →  1 violation
  [#wat.deporder/Violation {:referencer "a.wat" :referencer-pos 0
                            :definer "b.wat" :definer-pos 1 :symbol ":t::b::Thing"}]
CELL-B (must not fire)  the same two files in the opposite order                          →  0
```

and `test_stdlib_load_order::verify_stdlib_has_no_load_order_violations` **PASS** at position 3009/5237 of
`.floor/2026-09-12T03-04-48Z/`. So the inversion this note describes is gated, and the stdlib holds **0**.

★★ **But the gate could never have caught THIS defect, and the reason is the finding.** The gate reads
inversions that were **written down**. `capability.wat` never wrote `:wat::service::GateOutcome` — it wrote
`-> :wat::core::nil` *instead*, because the honest type was unnameable. **A workaround leaves no reference
for a reference-checker to read.** The degraded contract was the only evidence, and it looked like a
design choice.

⭑ So the sweep worth running is the **opposite** of the one this note asked for: not *which types are
named too early* (gated, zero) but **which surfaces return a weaker type than they mean because the honest
one loads later**. That one is a judgement census over `-> :wat::core::nil` on a fallible boundary, not a
mechanical one — and it is unrun. `[[NOTE-io-boundary-outcome-enum]]` is its doctrine.

## Provenance

- Found: `docs/excursus/2026/08/001-sns-sqs/the-gate-methods-face-an-outcome/SCORE.md`
  (§CAPABILITY SURFACE STAYS `nil`, and the orchestrator's grading §"a constraint the strike found").
- Kin: `NOTE-an-outcome-variant-no-primitive-can-construct.md` (this arc) — the other place where a
  type is *present* and the thing it promises is not reachable. Both are "the type says more than the
  substrate delivers", from opposite directions.
