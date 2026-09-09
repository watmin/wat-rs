# SCORE — STONE A-2 RELAND-2: the four mechanisms the join does not reach

## Start-here restoration

`git show 60813552a` was read; its `src/` changes (`src/check.rs`, `src/declare/register.rs`,
`src/freeze/env.rs`, `src/types.rs` — the join: `join_types` shared by `join_if_branches`,
`combine_match_arm`, `relate_value_to_slot`; `register_variant_types`; the ctor-narrowing fix in
`register_enum_methods`) were restored onto the current tree with
`git checkout 60813552a -- src/check.rs src/declare/register.rs src/freeze/env.rs src/types.rs`.
Verified byte-identical to `60813552a` before any further edit (`git diff 60813552a -- <file>`
empty for all four). All work below is layered on top of that restoration; `register.rs` and
`freeze/env.rs` are UNTOUCHED by this stone (final diff against `60813552a` is empty for both).

## Mechanisms closed

### ① Two parameter-checking paths (17 of 40) — FIXED

**Root cause.** `infer_option_expect`/`infer_result_expect` (`src/check.rs`, the hand-written
intrinsic checkers for `:wat::core::Option/expect` and `:wat::core::Result/expect`) compared the
argument's type against the expected `(Option :- [T])`/`(Result :- [T E])` shape with a bare
`unify(&opt_ty, &expected_opt, subst, env.types())`. `unify` is exact — it has no notion of the
`Variant <: Enum` subtype edge the restored join relies on. A user `defn`'s ordinary parameter
check instead routes through `assignable`, which does know that edge. Same pair of types, two
checkers, two answers.

**Fix.** Both call sites now call `assignable(&opt_ty, &expected_opt, subst, env)` /
`assignable(&res_ty, &expected_res, subst, env)` instead of `unify`. `assignable`'s
same-head-after-widening arm still falls through to a per-argument `unify` for the inner type
variable, so `t_var`/`e_var` still get bound exactly as before — this is strictly a widening of
acceptance, not a loss of the binding these functions depend on for their return type.

**Diff** (`src/check.rs`, both intrinsics identical shape):
```rust
-    if unify(&opt_ty, &expected_opt, subst, env.types()).is_err() {
+    if !assignable(&opt_ty, &expected_opt, subst, env) {
```
and the `Result` sibling identically.

**Verified.** `an_intrinsic_parameter_accepts_a_variant_like_a_user_defn_does` (the un-ignored
`intrinsic_param_accepts_a_variant` row) goes from RED (exit 1, `Option/expect: parameter opt
expects (:wat::core::Option :- [:?1628]); got (:wat::core::Option::Some :- [:wat::core::i64])`)
to GREEN (exit 0) on the restored tree.

### ③ A channel's message var pins to the first payload (5 of 40) — FIXED

**Root cause.** `relate_value_to_slot` (used by `infer_send_prime`/`infer_try_send_prime` for the
payload-to-`I` check) does: if either side carries a type variable, try `unify(&a, &e, ...)` on a
cloned `subst`; if that binds, commit it. When the channel's `I` type param `e` is still a bare
unbound `Var` (its first-ever use) and the payload `a` is a narrowed variant (e.g.
`:probe::Msg::Setup`), `unify` binds the var to `a` **verbatim** — pinning the channel's message
type to the FIRST payload's own narrow variant instead of the enclosing enum
(`:probe::Msg`). Every later `send'` of a *different* sibling variant on the same channel then
fails: `:wat::kernel::send: parameter payload expects :probe::Msg::Setup; got :probe::Msg::Work`.

**Fix.** Added `widen_to_enclosing_enum(t, env)` — widens `t` to its enclosing enum (same type
args) via `TypeEnv::enclosing_enum`, mirroring `join_types`'s own parent-lookup for sibling
variants; a no-op when `t` names no enum or is already bare. `relate_value_to_slot` now widens the
**actual** (payload) side before the var-binding `unify`:
```rust
-        let mut trial = subst.clone();
-        if unify(&a, &e, &mut trial, env.types()).is_ok() {
+        let widened = widen_to_enclosing_enum(&a, env);
+        let mut trial = subst.clone();
+        if unify(&widened, &e, &mut trial, env.types()).is_ok() {
```
A slot that is already concrete never reaches this branch (its `contains_type_var` guard is
false) — it falls straight to `assignable`, unchanged, which still accepts a narrow payload into a
concrete enum slot via the head-level `Variant <: Enum` edge. Only the first-binding case changes.

**Verified.** `tests/services/probe_arc170_c1_kwargs_bracket.wat` (RED: 3 `TypeMismatch`s, exactly
the brief's verbatim shape, `:probe::Msg::Setup` vs `:probe::Msg::Work`) → GREEN.
`wat-scripts/probes/arc-170/probe-m1-worker-setup.wat` (`wat --check`, same shape) → GREEN
(`EXIT=0`). Confirmed via `cargo nextest run --release -E 'test(wat_scripts_fixes_load)'`
(160s, PASS) that no other corpus fixture regressed or newly needs this mechanism.

### ④ A unit/under-consuming variant inherits an unused type param (3 of 40) — FIXED

**Root cause.** `TypeEnv::register_variant_types` builds each variant's singleton `EnumDef` with
`type_params: e.type_params.clone()` — the PARENT enum's full param list, copied verbatim,
regardless of whether the one variant's own fields use all of them. This is inert in the parent
process (nothing calls `check_type_params_consumed` on a programmatically-built singleton — traced
exhaustively: its one call site is `parse_type_decl`, reached only for source-parsed
declarations). But `closure_extract::type_def_to_ast`'s `TypeDef::Enum` arm reconstructs this
EXACT `EnumDef` as real `defenum` **source text** (`decl_name_siblings` splices `type_params` as
the declaration's own `:- […]` binder) whenever a closure needs to carry a type across a spawned
worker (`:wat::bracket::collect-loop`/`map-worker`, used by `each`/`map` over a service handle).
The worker's own startup re-parses that text through the ordinary declare pipeline, which DOES
enforce consumption — surfacing `UnconsumedTypeParam` at the **worker**, never visible at the
parent. E.g. a plain (non-generic) `:probe::counter` service's generated `Status::Stopped
[resp <- resp-ty]` carries `Status`'s transport-marker `T` (needed by the sibling `Started`
variant, which the whole-enum check validates fine) even though `resp-ty` never uses `T`.

**Fix.** `register_variant_types` now computes, per variant, the free type-vars its OWN member
types (its fields for a `Tagged` variant; none for `Unit`) actually mention, via the existing
`crate::declare::typevar::collect_free_type_vars_in`, and filters the parent's `type_params` down
to that set before building the singleton:
```rust
let member_types: Vec<TypeExpr> = match v {
    EnumVariant::Unit(_) => Vec::new(),
    EnumVariant::Tagged { fields, .. } => fields.iter().map(|(_, t)| t.clone()).collect(),
};
let consumed = crate::declare::typevar::collect_free_type_vars_in(&member_types);
let variant_type_params: Vec<String> =
    e.type_params.iter().filter(|p| consumed.contains(p)).cloned().collect();
```
The **parent** `EnumDef` (`e`) is untouched — its own `type_params` stay exactly as declared and
are still validated collectively at its own parse time. Only the per-variant singleton's stored
list narrows. This does not change the constructor's return TYPE EXPRESSION arity anywhere:
`register_enum_methods` builds that via `parametric_decl_type(&constructor_path,
&enum_def.type_params)`, reading the **parent's** `enum_def.type_params` (unaffected), never the
singleton's own stored def.

**Verified.** Before the fix: `probe_arc170_gapj_each_kwargs`, `probe_arc170_c2_mixed_macro`,
`probe_arc170_c2_strike1_mixed` (all spawn `:wat::bracket::collect-loop` workers over a generated
`Status` enum) failed with, verbatim:
```
#wat.kernel/AssertionFailure {:thread "probe_arc170_gapj_each_kwargs::each_with_kwargs_tail_fires_every_side_effect_and_returns_nil" :message "bracket collect-loop: runner 0 crashed: [#wat.kernel/LociDiedError.StartupError {:error #wat.type/UnconsumedTypeParam {:message \"type parameter \\\"T\\\" in :probe::counter::Status::Stopped's param-spec is declared but never used — every parameter in a type declaration's param-spec must be consumed by a field, variant, or body type (direct, e.g. `x <- T`, or nested, e.g. `x <- (Vector <- [T])`) — an unused parameter still discriminates types, but that discrimination must be written, not inferred. Remove \\\"T\\\" from :probe::counter::Status::Stopped's param-spec, or use it.\" :location #wat.core/Span {:file \"src/closure_extract.rs\" :line 2926 :col 16 :end #wat.core/Option.None {}} :causes [] :decl \":probe::counter::Status::Stopped\" :param \"T\"}}]" :location #wat.kernel/Location {:file "wat/bracket.wat" :line 750 :col 13} ...}
```
and (a second instance found the same way) `:probe::s1::Status::Stopped`, `:probe::s4::Status::PeersDenied`
in `probe_arc170_c2_strike1_mixed`/`probe_arc170_c2_mixed_macro`. After the fix, all three: GREEN.
`cargo nextest run --release -E 'binary_id(=wat::services)'` — 133/133 pass except the one
pre-existing unrelated failure (below).

⚠ Note on the brief's illustrative FQDN: the brief's verbatim example named
`:probe::counter::Status::Hibernated`; the actual corpus fixtures that reproduce this mechanism
name the failing singleton `Status::Stopped` or `Status::PeersDenied` (both generated
unconditionally by every `defservice`, same as `Status::Hibernated` — all are variants whose own
field types, or lack of fields, don't consume the transport-marker `T` that a sibling variant like
`Started` does). Mechanism, cause, and fix are identical regardless of which generated variant
trips it; `Hibernated` itself never appeared in a corpus grep (only in `wat/service.wat`'s macro
body, and the one live hibernate-resume fixture's Status enum passes because its snapshot record
also happens not to need this closure-extraction path in its test).

## Mechanism ② — nested widening (7 of 40) — STOP-3 FIRED, NOT FIXED

Per STOP-3, mechanism ① was applied FIRST and mechanism ② was re-measured against the result, as
instructed. It is **not** cleared. Minimal reproduction (`/tmp/mech2-repro.wat`, a plain user
`defn` — the same shape as the brief's intrinsic example, since ① already fixed the intrinsic
path and the residual failure is structural, not intrinsic-specific):

```wat
(:wat::core::defn :usr::takes-nested [o <- (:wat::core::Option :- [(:wat::core::Result :- [:wat::core::i64 :wat::core::String])])] -> :wat::core::i64
  (:wat::core::Option/expect o "nope"))

(:wat::core::def :usr::x
  (:usr::takes-nested (:wat::core::Option::Some {:value (:wat::core::Result::Err {:error "boom"})})))
```

`wat --check` output, verbatim (post-①-fix tree):
```
#wat.check/CheckErrors {:message "3 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":usr::takes-nested: parameter #1 expects (:wat::core::Option :- [(:wat::core::Result :- [:wat::core::i64 :wat::core::String])]); got (:wat::core::Option::Some :- [(:wat::core::Result::Err :- [:?1 :wat::core::String])])" ... :callee ":usr::takes-nested" :param "#1" :expected "(:wat::core::Option :- [(:wat::core::Result :- [:wat::core::i64 :wat::core::String])])" :got "(:wat::core::Option::Some :- [(:wat::core::Result::Err :- [:?1 :wat::core::String])])" :remedies []} ...]}
```

**Why ① doesn't reach it.** `assignable`'s Parametric/Parametric arm for `ah != eh` (here
`Option::Some` vs `Option`) requires, after confirming `is_subtype(ah, eh)` (the head-level
`Variant <: Enum` edge), that **every argument pairwise-`unify`s** — plain `unify`, not
`assignable` — deliberately invariant per Arc 278 Stone 2. The inner argument pair here is
`Result::Err<?N,String>` vs `Result<i64,String>` — different heads again, one level down. `unify`
requires exact head equality (`src/check.rs`'s `unify`, the `(Parametric,Parametric)` arm: `if h1
!= h2 || a1.len() != a2.len() { return Err(UnifyError) }`), so this inner pair fails, and the
outer `assignable` call returns `false`. This is exactly the argument-position question STOP-3
names: `assignable`'s per-argument check would need to become recursively `assignable` (permitting
widening one level down) rather than plain `unify`, and that is a language decision — whether an
argument position may WIDEN — the builder has not made. **STOPPED. Not fixed. No relaxation of
argument-position invariance was attempted.**

## STOP triggers — verbatim disposition

- **STOP-1** (no scoping by namespace/prefix): did not fire. `register_variant_types` is
  unconditional, exactly as restored from `60813552a`; not touched.
- **STOP-2** (join controls must stay green): did not fire.
  `two_sibling_variants_still_join_in_an_if` and `two_sibling_variants_still_join_across_match_arms`
  are GREEN throughout every build in this session (verified after each of the three fixes and in
  the final consolidated run below).
- **STOP-3** (mechanism ② not cleared by ①): **FIRED**. Reported above; not fixed.
- **STOP-4** (join must not search `subtype_edges`): did not fire. No new code in this stone
  touches `subtype_edges`; `widen_to_enclosing_enum` and the mechanism-④ filter both go through
  `TypeEnv::enclosing_enum`/`collect_free_type_vars_in`, neither of which consults it.
- **STOP-5** (`src/record/construct.rs` must not change): did not fire. That file is untouched
  (not in `git diff --stat` for this stone).

## Acceptance — every row, `tests/types/probe_arc296_a2_a_variant_is_a_type.rs`

`cargo nextest run --release -E 'test(a2_a_variant)'` → **11 tests run: 11 passed, 0 skipped.**

| Row | Result |
|---|---|
| `a_variant_value_still_flows_where_the_enum_is_expected` (widest control) | PASS |
| `matching_on_a_variant_still_works` | PASS |
| `a_nonexistent_variant_is_refused` | PASS |
| `an_intrinsic_parameter_accepts_a_variant_like_a_user_defn_does` (mechanism ①, un-ignored) | PASS |
| `a_user_defn_parameter_accepts_a_variant` (paired control) | PASS |
| `a_stdlib_enums_variant_is_a_type_too` (scope-cut catcher, un-ignored) | PASS |
| `two_sibling_variants_still_join_in_an_if` (⛔⛔ STOP-2 control) | PASS |
| `two_sibling_variants_still_join_across_match_arms` (⛔ STOP-2 control) | PASS |
| `a_function_can_take_only_one_variant_and_destructure_it` (subject, un-ignored) | PASS |
| `the_constructor_carries_the_variant_type` (subject, un-ignored) | PASS |
| `an_enum_value_does_not_flow_into_a_variant_parameter` (subject, un-ignored) | PASS |

Earlier stones unmoved — `cargo nextest run --release -E 'test(p1_annotation) or
test(p1b_a_parametric) or test(p2prereq) or test(p3_one_question) or test(a1_one_rule)'` →
**27 tests run: 27 passed** (`p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 ·
`p3_one_question` 5 · `a1_one_rule` 4 — matches the brief's stated counts exactly).

## Classification of everything else touched by these fixes, by reason (verbatim, one block each)

Per the tier note, a falling count is a progress meter, not a gate. Beyond the four named
mechanisms, targeted corpus sweeps (`binary_id(=wat::services)`, `binary_id(=wat::types) or
binary_id(=wat::comms) or binary_id(=wat::process) or binary_id(=wat::channel) or
binary_id(=wat::function) or binary_id(=wat::program)`, `test(wat_scripts_fixes_load)`,
`test(every_ungated_wat_checks)`, `test(hibernate)`) surfaced three further clusters. All three
were confirmed, by stashing this stone's `src/check.rs`/`src/types.rs` edits and testing against
the bare `60813552a` restoration, to be **pre-existing at `60813552a`, unaffected by any of this
stone's three fixes** (identical failures, byte-for-byte, before and after) — they are NOT part of
mechanisms ①/③/④, were not introduced by this work, and are out of this stone's scope to fix.

**Cluster A — `wat_arc148_ord_buildout` (12 tests), ordering intrinsics vs a narrowed
Option/Result variant.** Verbatim (one representative; all 12 share this shape):
```
thread 'wat_arc148_ord_buildout::result_ok_gt_err' panicked at tests/types/wat_arc148_ord_buildout.rs:30:41:
startup: #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::>: parameter #2 expects (:wat::core::Result::Ok :- [:wat::core::i64 :?3002]); got (:wat::core::Result::Err :- [:?3003 :wat::core::String])" ... :callee ":wat::core::>" :param "#2" :expected "(:wat::core::Result::Ok :- [:wat::core::i64 :?3002])" :got "(:wat::core::Result::Err :- [:?3003 :wat::core::String])" :remedies []}]}
```
`:wat::core::<`/`>`/`<=`/`>=`'s own hand-written ordering checker compares two SIBLING variants of
the same enum directly (`Result::Ok` vs `Result::Err`) rather than widening both to `Result` first
— structurally close to mechanism ①'s family (another hand-written intrinsic not routing through
the join's widening) but a distinct site (`:wat::core::<`/`>`/`<=`/`>=`, not `Option/Result
expect`) not named in the brief's four mechanisms, and not touched by this stone's fix (which was
scoped exactly to `infer_option_expect`/`infer_result_expect`).

**Cluster B — `recursive_patterns::nested_options_three_levels` (1 test), `match` refuses a
variant-narrowed scrutinee.**
```
startup: #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::match form: :wat::core::Option::Some pattern in (:wat::core::Option::Some :- [:wat::core::i64]) position" ...} #wat.check/MalformedForm {:message "malformed :wat::core::match form: :None pattern in (:wat::core::Option::Some :- [:wat::core::i64]) position" ...}]}
```
A nested nested-Option nested-match scrutinee stays narrowed to `Option::Some` (rather than
widening to `Option`) before reaching an inner `match`, whose pattern-shape checker then refuses
`Some`/`None` patterns against the narrow variant type. A `match`-side gap, not a parameter-check
or channel-var one; outside this stone's four mechanisms.

**Cluster C — `probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused`
(1 test), a superseded stone's own assertion.**
```
thread 'probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused' panicked at tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type.rs:89:5:
assertion `left == right` failed
  left: 0
 right: 1
```
This is P-2a's own fixture asserting that a GENERIC enum's variant reference stays refused
(`code == 1`) — true before RELAND-1, false now that RELAND-1 registers variants of every enum,
generic included, as real types (`code == 0`, correctly accepted). RELAND-1 supersedes P-2a's
scope by design (P-2a's variants-registered-but-uninhabitable state was explicitly reverted per
this file's own header); this assertion is stale relative to that supersession, not a defect this
stone introduces or is scoped to fix.

**Cluster D — `probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error`
(1 test), a golden keyed to the pre-join bare-enum error text.**
```
thread 'probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error' panicked at tests/services/probe_arc278_journal_surface.rs:62:5:
no check error matched `CheckErrorKind::TypeMismatch { expected, got, .. } if
expected == ":wat::telemetry::Journal::WriteMetricsResponse" && got ==
":wat::query::Store::PutResponse"`; errors were:
#wat.check/CheckErrors {:message "3 type-check errors" ... :errors [#wat.check/TypeMismatch {:message ":wat::telemetry::Journal::Reply::WriteMetrics: parameter resp expects :wat::telemetry::Journal::WriteMetricsResponse; got :wat::query::Store::PutResponse::Success" ... :got ":wat::query::Store::PutResponse::Success" ...} ...]}
```
The compile IS refused, as the test's own name expects — three genuine `TypeMismatch`s are
present — but the test's `.rs` assertion looks for the bare-enum spelling
`:wat::query::Store::PutResponse` in `got`, and the restored join's ctor-narrowing
(`register_enum_methods`) now correctly reports the narrow variant
`:wat::query::Store::PutResponse::Success` instead. This is a golden that needs updating for the
join's more precise error text — a direct, intended consequence of RELAND-1's own fix, not a
regression, and not this stone's tier (editing test goldens outside the four named mechanisms).

## Final consolidated verification

- `cargo nextest run --release -E 'test(a2_a_variant)'` — 11/11 PASS.
- `cargo nextest run --release -E 'test(p1_annotation) or test(p1b_a_parametric) or
  test(p2prereq) or test(p3_one_question) or test(a1_one_rule)'` — 27/27 PASS.
- `cargo nextest run --release -E 'binary_id(=wat::services)'` — 133/133 PASS except Cluster D
  (pre-existing, out of scope).
- `cargo nextest run --release -E 'binary_id(=wat::types) or binary_id(=wat::comms) or
  binary_id(=wat::process) or binary_id(=wat::channel) or binary_id(=wat::function) or
  binary_id(=wat::program)'` — 1011 tests, 997 pass; the 14 failures are exactly Clusters A+B+C,
  confirmed pre-existing and unchanged before/after this stone's fixes.
- `cargo nextest run --release -E 'test(wat_scripts_fixes_load)'` — PASS (160s; the whole
  `wat-scripts/` corpus, incl. `probe-m1-worker-setup.wat`, loads and type-checks clean).
- `cargo nextest run --release -E 'test(every_ungated_wat_checks)'` — PASS.
- `cargo nextest run --release -E 'test(hibernate)'` — 3/3 PASS (both loci of
  `wat-tests/service-hibernate-resume.wat`, plus the telemetry-bridge hibernate fixture).
- `binary_id(=wat::process) or binary_id(=wat::comms) or binary_id(=wat::channel)` — 131/131 PASS.

Per the tier, `scripts/floor.sh`, an unfiltered `cargo nextest run`, and `clippy` were **not** run
— the orchestrator runs those centrally.

## Files changed

- `src/check.rs` — mechanisms ① and ③ (`infer_option_expect`, `infer_result_expect`,
  `relate_value_to_slot`, new `widen_to_enclosing_enum`).
- `src/types.rs` — mechanism ④ (`TypeEnv::register_variant_types`).
- `src/declare/register.rs`, `src/freeze/env.rs` — restored from `60813552a` verbatim, untouched
  beyond that (empty diff against `60813552a`).
- `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` — removed the five `#[ignore = "arc 296
  A-2…"]` attributes RELAND-1's revert had left in place, so the acceptance filter runs all 11
  rows (0 skipped) as the brief requires. No assertions changed.
- `tests/types/probe_arc296_A2_a_variant_is_a_type__intrinsic_param_accepts_a_variant.wat`,
  `…__user_defn_param_accepts_a_variant.wat` — pre-existing fixtures from RELAND-2's own PROBE
  commit (`09904a68d`), not authored by this stone; carried forward unchanged.
