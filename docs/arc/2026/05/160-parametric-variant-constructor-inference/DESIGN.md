# Arc 160 — Parametric variant constructor inference

**Status:** opened 2026-05-07 on `arc-159-wip-substrate-and-sweep` branch.

**Gates:** arc 159 closure waits on arc 160 closure (the substrate
inference gap arc 159 surfaced needs fixing before arc 159's user-
visible binding-shape change ships cleanly).

## Background — the discovery vehicle

User direction 2026-05-07: *"this mass refactor work is meant to
catch exactly these kinds of problems."*

Arc 159 (drop per-binding `:T` from `:wat::core::let`) shipped
substrate + sweep on the WIP branch; cleanup brought workspace
to 9 failures concentrated in tests using polymorphic-constructor-
typed bindings:

```clojure
(:wat::core::let
  ((resp (:wat::core::Ok 418)))            ; resp's type post-arc-159
  (:wat::core::match resp -> :String
    ((:wat::core::Ok 200) "ok")            ; FAILS: "int literal pattern in :?29 position"
    ...))
```

Pre-arc-159: legacy annotation `((resp :wat::core::Result<i64,String>) (:Ok 418))`
gave the substrate the binding's full parametric type. Match arms
unified against the known type cleanly.

Post-arc-159: the legacy annotation is gone. Inference must figure
out the parametric type from the constructor application alone.

## The substrate gap

Looking at `src/check.rs` line 4426-4469 (Ok constructor inference
arm):

```rust
if head_is_ok_fqdn {
    let t_ty = infer(&args[0], env, locals, fresh, subst, errors)
        .unwrap_or_else(|| fresh.fresh());
    let e_var = fresh.fresh();
    return Some(TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![t_ty, e_var],
    });
}
```

This LOOKS like it produces `Result<inferred_type_of_value, ?fresh>`
correctly:
- `infer(&args[0])` for `418` returns `TypeExpr::Path(":i64")`
- `t_ty = :i64`
- Return `Result<:i64, ?fresh>`

Yet the failing test surfaces `?29` (a fresh type variable) in the
Ok payload position when the match arm's IntLit pattern checks
against it.

**Open diagnostic question for arc 160 slice 1:** which step
between "constructor returns `Result<:i64, ?>`" and "match-arm
pattern check sees `Result<?29, ?30>`" is the one that introduces
the additional fresh var or strips the i64? Possibilities:

1. The constructor inference is producing different output than
   the code suggests (maybe falling through to a different arm,
   maybe `infer(&args[0])` returns `None` for some reason).
2. `process_let_binding` is generalizing the type or substituting
   it incorrectly when populating `out_scope`.
3. Match scrutinee inference doesn't apply current substitution
   to the looked-up type before pattern check.
4. Pattern-arm type resolution introduces fresh vars instead of
   reading from scrutinee.

## Goal

Make `(:wat::core::Ok value)` and similar parametric variant
constructor applications produce parametric types that propagate
correctly through:
- Let bindings (arc 159 case)
- Function returns
- Match arm pattern checks

This unblocks arc 159 (the cleanup tests fail because the
substrate produces `?fresh` where it should produce `i64`).

## Scope

### Variants in scope

- `:wat::core::Ok` / `:wat::core::Err` (Result<T, E>)
- `:wat::core::Some` / `:wat::core::None` (Option<T>)
- User-defined parametric enum variants (per arc 057 / 109 design)

### What "fix" means concretely

When a polymorphic variant constructor is applied:
1. Inference returns `Parametric { head: ENUM, args: [...] }` where
   each arg is either:
   - The inferred type of the corresponding constructor argument
     (when the type parameter is determined by the application)
   - A fresh type variable (when the type parameter isn't
     determined by this application — e.g., Err type when calling Ok)
2. The returned parametric type propagates through:
   - `process_let_binding`'s `out_scope.insert(name, ty)`
   - Match scrutinee lookup
   - Pattern-arm position resolution
   - `apply_subst` at every relevant point

The fresh vars get unified later by recipient context (match arm
that uses Err's payload as String; downstream use of Result<i64, X>
that pins X).

## Slice plan

### Slice 1 — diagnostic

Sonnet (or orchestrator) reads the failing test path end-to-end,
adds tracing if needed, and identifies the EXACT step where the
type info is lost. Mode A delivers a one-paragraph summary of
which substrate function is responsible.

### Slice 2 — fix

Apply the fix at the identified site. Likely small (1-3 functions).
Verify the 9 currently-failing tests pass; verify the existing
2036 baseline + 12 arc 159 tests stay green.

### Slice 3 — closure

INSCRIPTION + 058 changelog row + cross-references. Closes arc 160.
Unblocks arc 159 closure.

## Cross-references

- **Arc 159 v3** — (the user-visible binding-shape change) waits on this
- **Arc 158a** — already shipped; valid in either world
- **Arc 145** — paid-for lesson "declared type is redundant when
  inference suffices" — arc 160 is the substrate work that makes
  inference SUFFICIENT for parametric constructor cases (where arc
  145's generalization missed)
- **Memory `feedback_v1_backout_dependency_arc.md`** — naming
  pattern: arc 160 ships separately; arc 159 closure cites this
- **WIP branch `arc-159-wip-substrate-and-sweep`** at
  `c6b8d74` — substrate + sweep + 9 cleanup fixes; gates arc 159
  on arc 160 ship

## Why arc 160 is the right shape

User direction: *"we do the hard, honest work for the longest
term solution... option A — strengthen substrate inference."*

Option A (substrate inference fix) wins on the four questions:
- Obvious — fixes the actual gap; no new syntax
- Simple — narrows the existing inference's scope to the variant
  constructor case
- Honest — closes a real substrate limit
- Good UX — best possible (no annotation needed anywhere)

The mass-refactor discipline turned this gap up. Per user
direction: *"this mass refactor work is meant to catch exactly
these kinds of problems."*
