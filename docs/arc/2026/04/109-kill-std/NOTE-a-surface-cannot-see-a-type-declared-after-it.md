# NOTE (arc 109 vocabulary) — a SURFACE cannot see a type declared AFTER it; the load order decides the contract

**Filed 2026-09-12, builder-directed** (*"we can note the capability one"*), found while striking
`the-gate-methods-face-an-outcome` in `docs/excursus/2026/08/001-sns-sqs/`. **NOT STARTED.**

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

## Provenance

- Found: `docs/excursus/2026/08/001-sns-sqs/the-gate-methods-face-an-outcome/SCORE.md`
  (§CAPABILITY SURFACE STAYS `nil`, and the orchestrator's grading §"a constraint the strike found").
- Kin: `NOTE-an-outcome-variant-no-primitive-can-construct.md` (this arc) — the other place where a
  type is *present* and the thing it promises is not reachable. Both are "the type says more than the
  substrate delivers", from opposite directions.
