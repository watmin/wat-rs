# Arc 145 — Typed `let` (`-> :T` declaration) + collapse `let*` → `let`

**Status:** drafted 2026-05-03 (mid-arc-144-slice-2-closure). User
direction:

> *"we need a new arc to land before 109 closes - we need let to be
> typed... we need to declare let as value bearing explciitly...
> some -> :T declaration like if, match, cond, and so on do..."*
>
> *"(and we'll kill let* in favor of the the current let* being
> let...) if there's an existing let... it'll assume let*'s behavior
> at the end of 109"*

## Goal

Two paired changes:

1. **Add `-> :T` declaration to `:wat::core::let`** so the form is
   explicitly value-bearing — readers see the result type at the
   form, not inferred from context.
2. **Kill `:wat::core::let*`.** Collapse to ONE binding form named
   `:wat::core::let` that uses today's `let*`'s sequential
   semantics (each RHS sees the previous bindings). Today's
   `:wat::core::let` (parallel; RHSs see only outer scope) gets
   absorbed into the sequential semantics at the end of arc 109.

After arc 145, the substrate has ONE binding form (`let`) with
EXPLICIT `-> :T` and sequential bindings — the same shape readers
already use today as `let*`, just with one fewer name to remember
and a head-anchored result type.

This is the expansion of task #185 (arc 109 follow-up: rename
`:wat::core::let*` → `:wat::core::let`) — the rename PLUS the
typing.

## Current state

`src/check.rs:5184-5208` (`infer_let`): PARALLEL semantics. All
RHSs see only outer locals; bindings are extended after the whole
binding-list is processed. Result type is the body's type.

`src/check.rs:5946+` (`infer_let_star`): SEQUENTIAL semantics. Each
RHS sees the PREVIOUS bindings via a `cumulative` env that grows as
each binding is processed. Result type is the body's type.

Runtime (`src/runtime.rs:2402-2403`): `eval_let` + `eval_let_star`
mirror the check semantics. Tail-call paths
(`src/runtime.rs:1969-1970`): same.

There's also an incremental-evaluator step (`runtime.rs:14852+`):
`step_let_star` exists; `step_let` may not exist as separate.

Surface count of usage sites:
- `let*` is the pervasive form across the substrate stdlib and lab.
- `let` (parallel) is RARE in user code per the grep observation.

Today's typing for both: bindings, body. NO `-> :T` declaration —
result type is purely inferred from body.

## Target shape

```scheme
(:wat::core::let
  (((<name> :Type) <expr>)
   ((<name> :Type) <expr>)
   ...)
  -> :ResultType
  <body-expr>)
```

OR (open question — see below):

```scheme
(:wat::core::let -> :ResultType
  (((<name> :Type) <expr>) ...)
  <body-expr>)
```

`-> :ResultType` placement matches the substrate's existing pattern
for value-bearing forms with a "dispatch-determiner" arg (the
bindings list is the let-equivalent of `match`'s scrutinee — sets
up the context but doesn't produce the result).

Per arc 108's analysis: forms whose first arg DOESN'T itself
produce the result put `-> :T` AFTER that arg (like `match` /
`if`). Forms whose first value-position DOES produce the result
put `-> :T` at HEAD (like `Option/expect`). Bindings don't produce
the result; the body does. So `-> :T` belongs AFTER the bindings.

## Slice plan (sketch — ~3-4 slices)

### Slice 1 — Add `-> :T` parsing to `infer_let_star` + `eval_let_star`

Extend `infer_let_star` (check.rs:5946) + `eval_let_star`
(runtime.rs) to OPTIONALLY accept a `-> :T` arrow + return-type
keyword between the bindings and the body. When present, validate
the body's inferred type unifies with `:T`; surface a clean
`TypeMismatch` if not. When absent, today's behavior preserved.

This is purely additive — no existing call site breaks. Sets up the
syntax for slice 2's rename.

~80-150 LOC + 4-6 unit tests.

### Slice 2 — Sweep wat sources to use `-> :T` (optional adoption)

Mid-arc 109's wind-down: encourage (not require) substrate +
stdlib code to declare `-> :T` on let* call sites. This is a
documentation pass — making the form's value-bearing nature
explicit at every site.

Optional — slice 1's optionality means existing call sites work
unchanged.

~100-200 LOC across substrate + stdlib wat files.

### Slice 3 — Rename `:wat::core::let*` → `:wat::core::let`

The big sweep. Per task #185 + arc 109 follow-up rename pattern:

1. Rename the dispatch site at check.rs:2959 + runtime.rs:2403
   to point at `:wat::core::let` (the existing parallel-let dispatch).
2. RETIRE `infer_let` (parallel) — it's superseded; merge its
   behavior into `infer_let_star` OR ship a deprecation poison that
   redirects + assumes sequential semantics.
3. Sweep ALL wat sources: `:wat::core::let*` → `:wat::core::let`.
4. Sweep ALL Rust sources for the same rename.
5. Add deprecation poison for any remaining `:wat::core::let*` call
   sites (Pattern 2 — synthetic TypeMismatch + redirect to new
   name).

Per the user's "if there's an existing let... it'll assume let*'s
behavior at the end of 109" — the parallel-let semantics get
absorbed. Most call sites don't rely on the parallel-vs-sequential
distinction.

~300-500 LOC sweep + grammar + tests.

### Slice 4 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry + amnesia doc end-of-work
review.

## Open questions

### Q1 — `-> :T` placement: after bindings, or at HEAD?

Per the substrate's own pattern:
- `match <scrutinee> -> :T <arms>+` — after dispatch-determiner
- `if -> :T <cond> <then> <else>` — at HEAD (or after cond?)
- `Option/expect -> :T <opt> <msg>` — at HEAD
- `Result/expect -> :T <res> <msg>` — at HEAD

Actually per check.rs:2956 `infer_if`: `if` puts `-> :T` at HEAD.
The "after dispatch-determiner" interpretation may be wrong.

DEFER to slice 1 implementation: orchestrator (or sonnet during
slice 1 audit) verifies the actual placement convention by reading
`infer_if`/`infer_match`/`infer_option_expect`. The choice for
`let` should match the established convention.

### Q2 — Slice 2 optionality: required or encouraged?

Slice 1 makes `-> :T` optional. Slice 2 sweeps the substrate +
stdlib to ADOPT it. Question: should arc 145 close with `-> :T`
REQUIRED (every let* call site MUST declare result type), or
OPTIONAL (forever encouraged but not required)?

Per arc 110's discipline (silent kernel-comm illegal): the substrate
has a track record of making "the right way" the only legal way
once the migration completes. If `-> :T` is the right way for `let`,
ARC 145 should close with it required.

DEFER to slice 3: the rename slice decides whether deprecation
poison applies to `(let bindings body)` (no `-> :T`) or whether
both shapes coexist forever.

### Q3 — Parallel-let semantics: deprecate or absorb?

User said "it'll assume let*'s behavior at the end of 109." Two
interpretations:
- **Absorb**: existing `:wat::core::let` call sites continue to type-check
  and run, but with sequential semantics (potentially changing
  behavior IF the call site relied on parallel-ness).
- **Deprecate-with-poison**: existing `:wat::core::let` call sites get a
  Pattern 2 poison ("use :wat::core::let* until the rename") so they
  surface the inconsistency before the rename lands.

Per arc 110's discipline: deprecate-with-poison is the calibrated
move. Absorb-silently risks a runtime behavior change for code that
relies on parallel-let semantics.

DEFER to slice 3: confirm via grep whether ANY user code relies on
parallel-ness; absorb if zero matches, deprecate-with-poison
otherwise.

## Why this arc must land before arc 109 v1 closes

Arc 109's wind-down rule: arc 109 v1 doesn't close until all
post-109 arcs implement (no deferrals). The `let*` → `let` rename
was already on the list (task #185); the typed-let addition expands
that task into a proper arc.

Closing arc 109 with both `let` and `let*` AND no `-> :T`
declaration on either would lock in the inconsistency.

## Cross-references

- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  — the prior arc that established the `-> :T` declaration pattern
  for value-bearing special forms.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  Pattern 2 poison + deprecation discipline this arc uses.
- Task #185 — "arc 109 follow-up: rename :wat::core::let* →
  :wat::core::let" — superseded by this arc.
- `src/check.rs:5184-5208` (current `infer_let` parallel)
- `src/check.rs:5946+` (current `infer_let_star` sequential)
- `src/runtime.rs:2402-2403` (current dispatch)

## Status notes

- DESIGN drafted. Implementation not started.
- Arc 144 takes priority (in flight: slices 3-5 remaining).
- Arc 130 (in flight) closure required for arc 109 v1.
- This arc joins the queue: arc 109 v1 closure now blocks on
  arc 144 + arc 130 + arc 145.
