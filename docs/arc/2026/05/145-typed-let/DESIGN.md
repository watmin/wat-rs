# Arc 145 — Typed `let` + `let*` (`-> :T` declaration on both)

**Status:** drafted 2026-05-03 (mid-arc-144-slice-2-closure).
**Revised** 2026-05-03 after user clarified the scope.

## User direction (verbatim)

> *"we need a new arc to land before 109 closes - we need let to be
> typed... we need to declare let as value bearing explciitly...
> some -> :T declaration like if, match, cond, and so on do..."*

Initial draft also included a `let*` → `let` rename. User
clarified after seeing the design:

> *"oh... i didn't realize we have both let and let* defined.. then
> the arc is... both will be typed.. we just typcally only use let*
> in our examples because we need the sequential binding.. but
> users can make their own choice.. that's a good stance to have"*

Further clarification 2026-05-03 evening (DESIGN re-revised):

> *"the ret val of a let statement /must be declared/ .. the
> 'user's choice' is whether or not to use let or let* -- both
> must have a ret val declared.. the let's ret val can be bound
> to something and used later - just like if, match etc"*

**`-> :T` is REQUIRED on both forms.** The "user's choice" is the
BINDING STRATEGY — parallel (`let`) vs sequential (`let*`). Both
forms produce values; both forms declare their return type. This
matches `if` / `match` / `cond` / `try` / `Option/expect` /
`Result/expect` — every value-bearing form in the substrate
declares `-> :T`. `let`/`let*` are the only outliers; arc 145
fixes that.

## Goal

Add `-> :T` declaration support to BOTH `:wat::core::let` (parallel)
and `:wat::core::let*` (sequential). Both forms remain — users
choose the binding strategy that fits the call site (parallel
when bindings are independent; sequential when later bindings need
earlier ones). The typing addition makes both forms explicitly
value-bearing, matching the substrate's `if` / `match` / `cond` /
`Option/expect` / `Result/expect` convention.

After arc 145, both forms read like:
```scheme
(:wat::core::let -> :T (((<n> :Type) <expr>) ...) <body>)
(:wat::core::let* -> :T (((<n> :Type) <expr>) ...) <body>)
```

(Exact `-> :T` placement matches whatever the substrate's existing
convention dictates — see Q1 below.)

## What this arc does NOT do

- **Does NOT rename `let*` → `let`.** Both forms stay distinct; the
  user direction "users can make their own choice" preserves
  parallel-vs-sequential as a deliberate decision at each call site.
- **Does NOT kill either form.** Task #185 (the original arc 109
  follow-up to rename `let*` → `let`) is SUPERSEDED by this arc's
  "both stay" stance.
- **Does NOT change runtime semantics.** Parallel-let stays
  parallel; sequential-let stays sequential. Only the type-checker
  surface changes (an optional `-> :T` annotation that gets
  validated against the body's inferred type).

## Current state

`src/check.rs:5184-5208` (`infer_let`): PARALLEL semantics. All
RHSs see only outer locals. Result type = body's type (no `-> :T`
today).

`src/check.rs:5946+` (`infer_let_star`): SEQUENTIAL semantics. Each
RHS sees the previous bindings via a `cumulative` env. Result type
= body's type (no `-> :T` today).

Runtime (`src/runtime.rs:2402-2403`): `eval_let` + `eval_let_star`
mirror the check semantics. Tail-call paths
(`src/runtime.rs:1969-1970`): same. Incremental evaluator
(`runtime.rs:14852+`): `step_let_star` exists.

## Target shape

```scheme
;; let — parallel binding (RHS expressions can't see each other)
(:wat::core::let -> :ResultType
  (((<n> :Type) <expr>) ...)
  <body>)

;; let* — sequential binding (later RHS expressions see earlier bindings)
(:wat::core::let* -> :ResultType
  (((<n> :Type) <expr>) ...)
  <body>)
```

`-> :T` is REQUIRED on both forms. Body's inferred type must unify
with `:T`; mismatch surfaces a clean `TypeMismatch` at the body's
position. Bare `(:wat::core::let bindings body)` without `-> :T`
is a parse error post-arc-145.

**Existing call sites must be migrated.** Arc 145 sweeps every
`(:wat::core::let ...)` and `(:wat::core::let* ...)` call site in
the repo (substrate `wat/`, tests, examples, lab consumers if
referenced) to add `-> :T` annotations. This is a foundation-
correction breaking change; per arc 109's "no bridges" doctrine,
ship the consistency cleanly. The migration is mechanical —
sonnet sweep with substrate-informed brief.

## Slice plan (3 slices — sweep 1a + 1b atomic; slice 2 closure)

### Slice 1a — Substrate: `-> :T` REQUIRED on `infer_let` + `infer_let_star` + eval counterparts

Extend `infer_let` (check.rs:5184) and `infer_let_star`
(check.rs:5946) to REQUIRE `-> :T` arrow + return-type keyword
AT THE PLACEMENT THE SUBSTRATE'S CONVENTION DICTATES (see Q1).
Validate body's inferred type unifies with `:T`. Bare
`(:wat::core::let bindings body)` without `-> :T` becomes a
parse error.

Mirror at runtime: `eval_let` (runtime.rs:2402+) and `eval_let_star`
(runtime.rs:2403+). The `-> :T` token doesn't affect runtime
evaluation; it's a no-op at the eval layer. Tail-call paths
(`eval_let_tail`, `eval_let_star_tail`) and incremental-step paths
(`step_let_star`) similarly thread the required arm-result.

Update arc 144 slice 2's special-form registry (`src/special_forms.rs`)
to reflect the required `-> :T` slot in the let / let* sketches.

~150-300 LOC substrate + 6-10 unit tests covering: typed parallel,
typed sequential, type mismatch on typed parallel, type mismatch
on typed sequential, parse error on untyped parallel (REQUIRED),
parse error on untyped sequential (REQUIRED).

**This sweep BREAKS the workspace** — every existing
`(:wat::core::let ...)` and `(:wat::core::let* ...)` call site
fails to parse post-substrate-change. Sweep 1b fixes them.
Per `feedback_no_broken_commits.md`, working tree stays dirty
between sweep 1a and sweep 1b; orchestrator commits both
atomically when workspace = 0-failed (per recovery doc § 7
atomic-commit-across-coordinated-sweeps pattern).

### Slice 1b — Consumer sweep: add `-> :T` to every existing call site

Workspace-wide sweep across:
- `wat/` substrate sources
- `wat-tests/` workspace test files
- `crates/*/wat/` per-crate substrates
- `crates/*/wat-tests/` per-crate tests
- Examples (`examples/`)
- Embedded wat strings in Rust test files (`tests/wat_*.rs`)

Each existing call site gains `-> :T` after the form name, where
T is the body's inferred return type. Sonnet uses cargo test's
type-mismatch error messages as the brief (the substrate's
diagnostic stream IS the work list).

Per "simple is uniform composition" (`feedback_simple_is_uniform_composition.md`):
N identical one-line additions IS simple.

Lab consumers (`holon-lab-trading/`) — separate workspace; out of
scope for this arc; lab handles its own migration when consuming
the new substrate version.

### Slice 2 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry documenting the new
`-> :T` shapes for both forms + cross-references to arc 108 (the
`-> :T` precedent) + arc 144 slice 2 (special-form registry sketch
update). End-of-work ritual review of COMPACTION-AMNESIA-RECOVERY.

## Open questions

### Q1 — `-> :T` placement

Per arc 108's INSCRIPTION:
> match and if put `-> :T` AFTER the first arg (scrutinee / cond)
> because that arg is a dispatch-determiner that doesn't itself
> produce the result. expect's value expression DOES produce the
> result (Some-/Ok-arm yields its inner). The honest position for
> `-> :T` is HEAD — declared before any value producer.

Verify by reading actual implementations:
- `infer_if` (check.rs around 2956 dispatch + the impl) — does it
  put `-> :T` at HEAD or after cond?
- `infer_match` (check.rs around 1420) — does it put `-> :T` at
  HEAD or after scrutinee?
- `infer_option_expect` / `infer_result_expect` — HEAD position
  per arc 108.

For `let`: bindings don't produce the result; the body does. So
either:
- HEAD position (matches `Option/expect` — value producer is the
  body, not the bindings):
  `(:let -> :T <bindings> <body>)`
- AFTER bindings (matches `match` — bindings are the
  dispatch-determiner analog):
  `(:let <bindings> -> :T <body>)`

Slice 1 brief MUST resolve this by reading the actual `infer_if` /
`infer_match` / `infer_option_expect` placements + matching whichever
convention is most consistent.

### Q2 — Required or optional `-> :T`? **RESOLVED: REQUIRED**

User direction 2026-05-03 evening (verbatim, captured in Status
section above):

> *"the ret val of a let statement /must be declared/ .. the
> 'user's choice' is whether or not to use let or let* -- both
> must have a ret val declared.. the let's ret val can be bound
> to something and used later - just like if, match etc"*

**`-> :T` is REQUIRED on both forms.** The "user's choice" is
the BINDING STRATEGY (parallel `let` vs sequential `let*`), NOT
the optionality of the type annotation.

Consistency rationale (now load-bearing):
- `Option/expect` + `Result/expect` REQUIRE `-> :T` per arc 108
- `if` + `match` + `cond` REQUIRE `-> :T`
- `try` REQUIRES `-> :T`
- `let` + `let*` were the ONLY outliers; arc 145 fixes that

Existing call sites must be migrated (sweep 1b's scope). Per
arc 109's "no bridges" doctrine + FM 11's no-deferral discipline,
ship the consistency cleanly rather than papering over with
"optional forever."

Earlier draft of this section said "OPTIONAL forever" — that
draft was Q2-wrong. Per `feedback_inscription_immutable.md` the
correction is captured here in the live DESIGN, with the
prior-state preservation note that the early draft (committed in
git history) was the misinterpretation that the user's evening
clarification corrected.

### Q3 — Special-form registry sketch

Arc 144 slice 2 registered both `:wat::core::let` and
`:wat::core::let*` in the special-form registry with sketches:
```
(:wat::core::let <bindings> <body>+)
(:wat::core::let* <bindings> <body>+)
```

Slice 1 should update these sketches to reflect the new optional
`-> :T` slot. Exact format depends on Q1's resolution.

## Why this arc must land before arc 109 v1 closes

Arc 109's wind-down rule: arc 109 v1 doesn't close until all
post-109 arcs implement (no deferrals). Closing arc 109 with
let / let* untyped (when every other value-bearing form has
explicit `-> :T`) would lock in the inconsistency.

## Cross-references

- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  — the prior arc that established the `-> :T` declaration pattern
  for value-bearing special forms.
- `docs/arc/2026/05/144-uniform-reflection-foundation/SCORE-SLICE-2.md`
  — the slice that registered `let` + `let*` in the special-form
  registry (slice 1 here updates those sketches).
- Task #185 (SUPERSEDED): "rename :wat::core::let* → :wat::core::let"
  is no longer in scope; user direction preserved both forms.
- `src/check.rs:5184-5208` (current `infer_let` parallel — DOES NOT
  CHANGE in this arc; only gains optional `-> :T`)
- `src/check.rs:5946+` (current `infer_let_star` sequential — same)
- `src/runtime.rs:2402-2403` (eval dispatch)

## Status notes

- DESIGN drafted (revised after user clarification).
- Implementation deferred until arc 144 ships (in flight: slices
  3-5 remaining).
- Arc 109 v1 closure blocks on arc 144 + arc 130 + arc 145.
