# Arc 161 — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

`infer_list` in `src/check.rs` now infers the result type for
function-value applications — calls of the shape `(t arg1 arg2 ...)`
where `t` is a Symbol (or arbitrary inline expression) whose
inferred type is `Fn(P1, P2, ...) -> R`.

Pre-fix, lines 4606-4613 were a no-op fall-through:

```rust
// Non-keyword head (bare symbol or inline expression). Not typed
// at this layer pending your call on explicit let-binding type
// annotations. Recurse into args so nested keyword-headed calls
// still get checked.
for item in items {
    let _ = infer(item, env, locals, fresh, subst, errors);
}
None
```

The legacy `((t :Fn(...)) (rhs))` let-annotation typed `t`
directly via the binder's declared type; downstream `(t arg)`
didn't need application inference because the binding was
annotated, and the post-call type was unused (or filled by the
next legacy annotation in chain). Arc 159 dropped the annotation
— exposing the no-op.

The fix: at the fall-through, infer the head, reduce to canonical
form, match `TypeExpr::Fn`, do application (arity check + per-arg
unify + return `apply_subst(*ret, subst)`). Mirrors `infer_spawn`'s
existing value-head branch (`src/check.rs:7556-7589`).

The non-Fn `_ =>` arm silently returns `None` and recurses into
args — does NOT emit `TypeMismatch`. This matches `infer_spawn`'s
precedent and avoids false positives at every other site that
recurses into a non-keyword list (retired-form params-lists,
type-annotation lists, etc.).

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1 (sonnet ran) | `f6ab13f` | `infer_list` no-op replaced with value-head Fn-application branch (~55 LOC); single file edited |
| 2 | (this commit) | Closure paperwork |

## Substrate impact

| Test outcome | Pre-arc-161 (post-arc-160) | Post-arc-161 |
|---|---|---|
| Workspace failures | 1 | 0 |

The single failing test was
`deftest_wat_telemetry_test_svc_tel_null_translator` — a
let-binding chain where `result` was bound to the application of
a let-bound Fn value:

```clojure
(:wat::core::let
  ((t (:test::svc-tel-null-translator))     ; t : Fn(Stats)->Vector<i64>
   (result (t (:wat::telemetry::Stats/new 1 2 3))))
  (:wat::test::assert-eq (:wat::core::length result) 0))
```

Pre-fix: `(t (Stats/new 1 2 3))` returned `None` from `infer_list`;
`result` not in scope; `length result` → `<unresolved>` →
dispatch error. Post-fix: `result` resolves to `Vector<i64>`;
length dispatch matches `Vec<T>` arm; clean.

## Settled design

### Why "(value head)" as the callee label

Keyword-headed paths use `k.clone()` — the FQDN string — as the
`callee` field on diagnostic errors. Value-head application has
no single canonical name (head can be Symbol / inline expression
/ anything). `"(value head)"` is the stable label; if a future
arc wants more descriptive labels (e.g., the Symbol's name), that
shape is purely ergonomic.

### Why silent return on non-Fn head

The first draft (mirroring the BRIEF template literally) emitted
`TypeMismatch` when the head's inferred type wasn't a `Fn`. This
broke `multiple_lambda_sites_post_retirement_silently_alias`:
retired `(:wat::core::lambda ...)` forms lack a checker arm;
the schema-lookup fall-through recurses into the lambda's args;
the lambda's params-list `(() -> :wat::core::i64)` has head `()`
(empty list) which infers to `Some(TypeExpr::Tuple([]))` (unit).
The value-head branch saw unit-as-head and emitted a false-
positive `TypeMismatch`.

Sonnet's correction: silent `return None` + arg recursion. The
value-head branch is an INFERENCE OPPORTUNITY at a fall-through
site, not a strict call-site checker. The fix is purely additive
— `None → correct type when head IS Fn`; never `None → false-
positive error`. Matches `infer_spawn`'s precedent (returns a
placeholder for non-Fn heads, no error).

This is the exemplary discipline: catch your own regression
mid-sitting, apply the principled fix consistent with prior
precedent, surface the chain of reasoning in the report.

### Why this gap existed

Pre-arc-159, `let`-bound names typed via legacy
`((name :T) rhs)` annotation; the substrate stored `:T` directly
in scope. Downstream `(name arg)` falling into the no-op was
fine — every value-head call site either had its result
annotated by the next binding, or used positional accessors
(`first` / `second`) that tolerate unresolved.

Post-arc-159, `let`-bound names type via inference of the RHS.
RHS like `(:test::null-translator)` correctly produces
`Fn(...)->R`. But applying that bound Fn (`(t arg)`) requires
application inference — which arc 161 fills.

This is the third substrate inference gap arc 159 surfaced:
- arc 158a: walker reads declared `:T` → migrated to RHS
  pattern-match
- arc 160: variant constructor FQDN keyword paths fell through →
  hoisted into Region A
- arc 161: Symbol-headed application returned None → fixed to
  mirror `infer_spawn`'s value-head path

The mass-refactor discipline turned this gap up. Per user
direction 2026-05-07: *"this mass refactor work is meant to
catch exactly these kinds of problems."*

## Honest deltas surfaced by sonnet

1. **The mid-flight regression-and-correction.** Documented
   above and in SCORE-SLICE-1. Sonnet caught and fixed before
   reporting; no orchestrator rework needed.

2. **Inline-expression heads work via the unified `infer(head)`
   path.** No special handling for Symbol-vs-list heads — the
   recursive `infer` dispatches on AST shape internally.

3. **`reduce` is the right canonicalization helper for this site.**
   No separate `apply_subst` pre-step needed (`reduce` covers
   Var-walk + alias expansion in one normalization pass).

4. **No unification surprises at this layer.** Same `unify` shape
   the keyword-headed branch uses (line 4481).

## Tests

No new tests added by arc 161. The 1 currently-failing workspace
test verified the fix end-to-end:

- `deftest_wat_telemetry_test_svc_tel_null_translator`

Workspace post-fix: 2047 passed / 0 failed.

## Out of scope (arc 162 is queued separately)

- **Internal `lambda` identifier rename** — `Value::wat__core__lambda`,
  `parse_lambda_signature*`, `<lambda@span>` debug strings,
  `tests/wat_spawn_lambda.rs` test naming. Surface lambda is dead
  (arc 155); internal Rust-level identifier rename is queued as
  arc 162 (DESIGN at
  `docs/arc/2026/05/162-lambda-internal-rename/DESIGN.md`).

## Cross-references

- **Arc 159** — discovery vehicle; this arc's gap was masked by
  arc 159's pre-state (legacy let-annotations typed bound names
  directly, bypassing application inference)
- **Arc 158a** — sibling pattern (substrate adapts to arc 159's
  shape change; walker side of the same family)
- **Arc 160** — sibling substrate fix (variant constructors);
  same arc 159 discovery pattern, different gap
- **Memory `feedback_v1_backout_dependency_arc.md`** — the
  cascading-arcs naming pattern (arc 159 v3 ships when both
  dependency arcs land)
- **`infer_spawn`'s value-head branch** at `src/check.rs:7556-7589`
  — the precedent arc 161 mirrors

## Commit chain

- `d221de1` arc 161 opens (DESIGN + BRIEF + EXPECTATIONS)
- `f6ab13f` arc 161 slice 1: Symbol-headed application inference
- (this commit) arc 161 slice 2: closure paperwork
