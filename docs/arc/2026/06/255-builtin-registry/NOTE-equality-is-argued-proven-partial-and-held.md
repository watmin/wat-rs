# NOTE — `=`/`not=` are argued, proven `@Totality Partial`, and HELD

> Arc 255 Stone 1c-b-ii landed **four** of its six rows (`<` `>` `<=` `>=`). `:wat::core::=` and
> `:wat::core::not=` are **held**, not abandoned — ruled by the four questions in-chat, 2026-09-03.

## Why they are held

Their grading is correct and empirically proven twice — the counterexample is committed at
`wat-scripts/scratch-pad/probe-core-eq-is-partial.wat`:

```
(:wat::core::= <fn> <fn>)     --check → exit 0        the checker admits it
                              run     → TypeMismatch  "expected matching comparable pair,
                                                       got wat::core::fn"
```

`infer_equality` admits any pair whose types unify; `values_equal` has no `Value::Function` arm.

Registering that honest `Partial` retires `intrinsic_meta`'s by-name placeholder
(`matches!(head, "reduce" | "=" | "not=")`) whose own header says a homed name must leave it — and
the rete fence then correctly refuses `=`, turning four fixtures red. Those fixtures are not wrong:
they compare a `String` field to a string literal, which genuinely cannot raise. **The fence asks
"is this VERB total?" when the answerable question is "is this CALL total?"**

`[[RULING-rete-forged-the-paths-the-registry-claims-the-tools]]` rules the cure: the registry gains
`properties_of(name, arg_types)`, and the fence becomes a consumer. **These two rows land the
moment it can answer, and eight rete rows unblock with them.**

## Stone 1c-b-iii update (2026-09-03) — the domain gate exists now; the grade does not change

Stone 1c-b-iii built `is_type_equatable` (`src/check.rs`, sibling of `is_type_orderable`) and
gated `infer_equality` on it — exactly the cure `RULING-rete-forged-the-paths-the-registry-
claims-the-tools.md` and the "What must be true before they land" section below anticipated via
`properties_of(name, arg_types)` + a rete-fence consumer. **That specific mechanism was not
built; this narrower one supersedes it for `=`/`not=`'s own totality question** (the rete-fence
question — "is this CALL total" vs "is this VERB total" — is untouched and still open for the
eight rete rows named below).

The gate **does** close the hole `probe-core-eq-is-partial.wat` measures: after this stone,
`--check` on that file **rejects** it (previously exit 0). Direct calls comparing two
`:wat::core::fn` values, or any other type `values_equal` has no arm for (a genuine
pre-existing gap also found: `(:wat::core::PersistentMap :- [K V])` has no `values_equal` arm
either — same shape as `Fn`, not previously named), now fail `--check` instead of raising at
runtime.

**But the measured question this stone was built to answer — "is `=` genuinely `Total`, or
`Total`-at-concrete-sites-only?" — comes back `Total`-at-concrete-sites-only.** Built
`wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat`: a generic `eq-generic :- [T] [a <-
:T b <- :T] -> :bool (:wat::core::= a b)`, called with two `:wat::core::fn` arguments.

```
--check  → exit 0            (the checker admits the call)
run      → TypeMismatch      "expected matching comparable pair, got wat::core::fn"
```

Mechanism: `eq-generic`'s own body is checked ONCE, generically — `is_type_equatable` must admit
the bare rigid type param (`Path(":T")`, per `check_function_body`'s "declared type parameters
are RIGID... represented as `Path(\":T\")`", `:1780-1783`) or `wat/test.wat:61`'s `assert-eq`
itself stops compiling (STOP-1). Once admitted, the CALL SITE `(eq-generic <fn> <fn>)` is checked
only against `eq-generic`'s declared signature (`a <- :T`, `b <- :T` — both unify against the
same fresh `T`, which two structurally-identical `Fn` types do) — ordinary call-site argument
unification, a completely different code path from `infer_equality`. Nothing re-invokes the
domain gate under the concrete instantiation. So the hole this stone closes at DIRECT call sites
reopens, unchanged, one level of indirection inside a generic body.

A second, incidental finding along the way: `is_type_orderable`'s own `TypeExpr::Var(_) => true`
line (`check.rs:12871`) does **not** reach this same rigid-type-param case — a declared `:T`
inside a generic body is `Path(":T")`, never `TypeExpr::Var`. Built `/tmp/probe_lt_generic.wat`
(a generic `[T] [a <- :T b <- :T] -> :bool (:wat::core::< a b)`) and confirmed `--check` refuses
it TODAY, independently of this stone — a dormant pre-existing gap in Stone 1c-b-ii's ordering
gate, inert only because no corpus function currently orders two bare `:T` values.
`is_type_equatable` does not repeat it: it defers on both `TypeExpr::Var(_)` and a rigid `Path`
via `is_type_param_letter` (`check.rs:9932`), which is what keeps `assert-eq` compiling.

**Grade stands: `@Totality Partial`.** The reason is narrower and now precisely located (the
type-var door on a generic body, not an ungated `Fn`/`PersistentMap` domain at every call site),
but a well-typed call can still reach `values_equal`'s raise. The rows below stay held, verbatim,
unregistered — `intrinsic_meta`'s by-name placeholder for `=`/`not=` is untouched, and the four
rete/sift fixtures were re-run unedited and still pass (the fence never got exercised by this
stone — no registration means `lookup_entry` still returns `None` for these two heads). The gate
itself ships regardless, on its own merits: it turns every DIRECT `(= <fn> <fn>)`-shaped call
from a silent runtime raise into a located compile error, which is the majority of how `=` is
actually called in this corpus (`assert-eq`'s own generic indirection is the outlier, not the
rule).

## The argued blocks, kept verbatim so the follow-up stone lifts rather than re-derives

Each carries its five axes with the fn or `file:line` each was grounded on. The `@Totality Partial`
argument in particular cost a built-and-run counterexample; it is not to be re-argued from scratch.

```rust
:wat::core::=

/// `(:wat::core::= a b)` — arc 255 Stone 1c-b-ii, registered `#[wat_intrinsic]`. THIN
/// WRAPPER, not a reimplementation: `eval_eq` (`:5221-5254`, immediately above) takes `head`
/// as its first parameter — not the canonical `#[wat_intrinsic]` shape — so it cannot be
/// annotated in place; this wrapper forwards its own FQDN as `head` and changes nothing else.
/// `eval_eq` itself is untouched.
///
/// **Purity/Determinism ground — `Pure ∧ Deterministic`:** `eval_eq`'s body evaluates each
/// operand by ordinary call-by-value (`eval_inner`, `:5237-5238`) and then only classifies
/// the two already-evaluated values via `values_equal` (`:5297-5529`, a pure structural-match
/// function with no `eval_inner`/`apply_function` on caller-supplied code, no I/O, no
/// entropy/clock read). `Pure ∧ Deterministic`.
///
/// **Totality ground — `Partial`, empirically confirmed reachable:** `eval_eq` raises
/// `TypeMismatch` when `values_equal` returns `None` (`:5240-5250`). `infer_equality`
/// (`src/check.rs:12773-12841`, dispatched from `check.rs:3797-3798`) accepts a call as
/// well-typed whenever the two operand types `unify`, OR one is a subtype of the other, OR
/// both are subtypes of `:wat::core::Record`, OR both are numeric (`:12809-12828`) — it never
/// asks whether `values_equal` actually HAS an arm for the resulting pair. `values_equal`'s
/// own doc says plainly it returns `None` "for pairs whose shapes aren't comparable at all
/// (e.g., comparing a `Value::Function` to anything...)" (`:5300-5302`), and its match falls
/// to a bare `_ => None` (`:5527`) for exactly that case — no arm exists for `Value::Function`
/// anywhere in it. Measured directly, not just read: a program comparing two same-signature
/// `:wat::core::fn` values (`(:wat::core::= (:wat::core::fn [x <- :wat::core::i64] ->
/// :wat::core::i64 x) (:wat::core::fn [y <- :wat::core::i64] -> :wat::core::i64 y))`) passes
/// `target/release/wat --check` cleanly (exit 0 — `unify` on two structurally-identical
/// `TypeExpr::Fn` succeeds, `src/check.rs:15631-15638`) and then raises
/// `#wat.runtime/TypeMismatch {..got wat::core::fn..}` at eval. A well-typed call reaches the
/// raise. `Partial` — the same precedent Stone 1c-a-ii set for `conforms?`.
///
/// **Expand-time ground — `Legal`:** `src/macros/eval.rs`'s residue hand-list names
/// `":wat::core::="` literally (`:486`, the "value/control-flow ops with no per-verb home
/// yet" group) — registering here REPLACES that residue entry, so it must carry the SAME
/// verdict or silently revoke today's legality (arc 255 the `fn` lesson).
///
/// **Category ground — `Probe`:** matches the per-type sibling `:wat::i64::=`'s own
/// registered `@Category Probe` (`src/intrinsic/i64.rs:552`) — interrogates two values
/// and derives a FACT about their relationship; `wat/runtime-meta.wat:113-116`'s `:Probe`
/// doc — "the output is a fact ABOUT the input... NOT 'returns a bool'."
///
/// `@arg`/`@ret` grounded in `infer_equality` (`src/check.rs:12773-12841`), dispatched from
/// `check.rs:3797-3798` — no `TypeScheme` exists.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality      Partial
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     args :wat::core::Value the left operand (position 0) then the right operand
///   (position 1, prose-only — the variadic sniff leaves no second `@arg` slot); the two must
///   be compatible — their types `unify`, one is a subtype of the other, both are subtypes of
///   `:wat::core::Record` (cross-flavor record comparison), or both are numeric
///   (`infer_equality`, `src/check.rs:12773-12841`); raises `TypeMismatch` at runtime if the
///   compatible pair is not one `values_equal` can actually compare (e.g. two `:wat::core::fn`
///   values)
/// @ret     :wat::core::bool true iff position 0 structurally equals position 1
///   (`values_equal`, `src/runtime.rs:5297-5529`)
/// @example (:wat::core::= 1 1) #=> true
/// @see     :wat::core::not=
#[wat_intrinsic(":wat::core::=")]
fn eval_eq_intrinsic(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_eq(":wat::core::=", args, list_span, env, sym)
}

========================================

:wat::core::not=

/// `(:wat::core::not= a b)` — arc 255 Stone 1c-b-ii, registered `#[wat_intrinsic]`. THIN
/// WRAPPER over `eval_not_eq` (`:5267-5287`, immediately above), which itself delegates to
/// `eval_eq` and inverts the result — same reasoning as `:wat::core::=`'s wrapper above,
/// re-measured against this name; `eval_not_eq` is untouched.
///
/// **Purity/Determinism ground — `Pure ∧ Deterministic`:** identical to `:wat::core::=`
/// above — `eval_not_eq` calls `eval_eq` (itself `Pure ∧ Deterministic`, grounded above) and
/// inverts a `Value::bool`; no further evaluation of caller-supplied code.
///
/// **Totality ground — `Partial`, same mechanism as `:wat::core::=`:** `infer_equality`
/// (`src/check.rs:12773-12841`) handles `=` and `not=` at the SAME call site
/// (`check.rs:3797-3798`, `":wat::core::=" | ":wat::core::not="`) with no distinction between
/// them, and `eval_not_eq` raises the identical `TypeMismatch` `eval_eq` would (`:5279-5285`,
/// by propagating `eval_eq`'s own `?`). The `:wat::core::=` wrapper's empirical counterexample
/// (two `:wat::core::fn` values) applies unchanged: swap `=` for `not=` in that program and the
/// same well-typed-call-reaches-raise shape holds. `Partial`.
///
/// **Expand-time ground — `Legal`:** `src/macros/eval.rs`'s residue hand-list names
/// `":wat::core::not="` literally (`:487`, the same group `=` sits in) — registering here
/// REPLACES that residue entry, so it must carry the SAME verdict.
///
/// **Category ground — `Probe`:** matches the per-type sibling `:wat::i64::not=`'s own
/// registered `@Category Probe` (`src/intrinsic/i64.rs:589`) — same reasoning as `=`
/// above, inverted.
///
/// `@arg`/`@ret` grounded in `infer_equality` (`src/check.rs:12773-12841`), dispatched from
/// `check.rs:3797-3798` — no `TypeScheme` exists.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality      Partial
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     args :wat::core::Value the left operand (position 0) then the right operand
///   (position 1, prose-only — the variadic sniff leaves no second `@arg` slot); same
///   compatibility rule as `:wat::core::=` (`infer_equality`, `src/check.rs:12773-12841`);
///   raises `TypeMismatch` at runtime under the same condition `=` does
/// @ret     :wat::core::bool true iff position 0 does not structurally equal position 1
/// @example (:wat::core::not= 1 2) #=> true
/// @see     :wat::core::=
#[wat_intrinsic(":wat::core::not=")]
fn eval_not_eq_intrinsic(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_not_eq(":wat::core::not=", args, list_span, env, sym)
}```

## What must be true before they land

- `properties_of(name, arg_types)` exists and the rete fence consults it.
- `intrinsic_meta`'s `matches!(head, "reduce" | "=" | "not=")` placeholder retires — its own
  header requires that of any homed name.
- The four fixtures (`probe_arc278_foreign_pred_purity`, `probe_arc278_sift_logs` ×2,
  `probe_arc278_sift_arena`) go green **without being edited** — if they need editing, the
  interface answered the wrong question.
