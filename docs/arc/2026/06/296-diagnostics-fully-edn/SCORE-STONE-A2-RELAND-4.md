# SCORE — STONE A-2 RELAND-4: option E, and the last three

## Restore

`git log --oneline -20` found `9d5bf198f WIP(296 A-2 RELAND-3): 24 -> 9, and 7 of the 9 are the
fenced ruling — PRESERVED`. Per the brief's own fallback (restoring the probe file from that
commit would overwrite the two newest test rows), I restored only:

```
git checkout 9d5bf198f -- src/types.rs src/declare/register.rs src/check.rs src/freeze/env.rs \
  tests/services/probe_arc278_journal_surface.rs
```

and left `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` at HEAD (which already carried the
two RELAND-4 rows added by `3226a387a`, correctly `#[ignore]`d as vacuous). Removed the 7
now-stale `#[ignore]` attributes by hand (all 7 fixtures already existed on disk; no new fixture
authored). `cargo build --release` was clean before any of my own edits.

## The enum gate — where it lives

`fn container_is_enum(head: &str, types: &TypeEnv) -> bool` in `src/check.rs` (next to
`type_head_args`/`widen_to_enclosing_enum`, ~line 17319):

```rust
fn container_is_enum(head: &str, types: &TypeEnv) -> bool {
    matches!(
        types.get(&crate::types::parametric_head_fqdn(head)),
        Some(crate::types::TypeDef::Enum(_))
    )
}
```

It is the ONLY gate (STOP-3): a plain `TypeEnv::get` + `TypeDef::Enum` match, no hand-listed
"safe container" set. It answers the same for a user's bare `defenum`, a stdlib one
(`Option`/`Result`/`Outcome`), and one of `register_variant_types`'s own one-variant singletons
(a variant is a sum-of-one, so the same derivation applies transitively at every nesting depth).

It gates two sites in `assignable` (`src/check.rs`):
1. The pre-existing "`ah != eh`, head-level `is_subtype` variant<:enum edge" arm — per-arg
   fallback: `unify(x, y) || (container_is_enum(eh) && assignable(x, y))`, on a CLONED subst
   (mirrors `relate_value_to_slot`/`combine_match_arm`'s established pattern — a failed
   recursive `assignable` can leave partial bindings, and this is exactly the class of bug I
   hit and fixed below).
2. The pre-existing "Arc 278 Stone 2, `ah == eh` same-head parametric" arm — same fallback,
   gated on `container_is_enum(ah)` (== `eh` in this arm), also on a cloned subst.

Both fallbacks only ever ADD an acceptance path; the existing `unify`/bottom-top/`Peer'` checks
are untouched, so every prior byte-identical behaviour for a non-enum container is preserved.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'
```

**13 tests run: 13 passed, 0 skipped.** (Acceptance said 13/0 — met exactly.)

| Test | Result |
|---|---|
| `a_nonexistent_variant_is_refused` | PASS |
| `an_enum_value_does_not_flow_into_a_variant_parameter` | PASS |
| `two_sibling_variants_still_join_across_match_arms` | PASS |
| `a_user_defn_parameter_accepts_a_variant` | PASS |
| `a_nested_variant_literal_reaches_a_base_typed_parameter` | PASS (subject) |
| `the_constructor_carries_the_variant_type` | PASS |
| `an_intrinsic_parameter_accepts_a_variant_like_a_user_defn_does` | PASS |
| `a_stdlib_enums_variant_is_a_type_too` | PASS |
| `two_sibling_variants_still_join_in_an_if` | PASS |
| `matching_on_a_variant_still_works` | PASS |
| `a_variant_value_still_flows_where_the_enum_is_expected` | PASS |
| `a_non_enum_container_does_not_widen_its_argument` | PASS — exit 1, invariant held (STOP-1 subject) |
| `a_function_can_take_only_one_variant_and_destructure_it` | PASS |

Earlier stones, unmoved (re-run, not just assumed):

| Filter | Result |
|---|---|
| `p1_annotation` | 10 passed |
| `p1b_a_parametric` | 4 passed |
| `p2prereq` | 4 passed |
| `p3_one_question` | 5 passed |
| `a1_one_rule` | 4 passed |

## STOP triggers

- **STOP-1** — did NOT fire. `a_non_enum_container_does_not_widen_its_argument` (fixture
  `non_enum_container_stays_invariant.wat`, a `defrecord :usr::Holder :- [T]` holding
  `(Option :- [i64])`, constructed with `(Holder (Option::Some {:value 1}))` against a param
  declared `(Holder :- [(Option :- [i64])])`) stays refused, exit 1:
  `":user::takes-holder: parameter #1 expects (:usr::Holder :- [(:wat::core::Option :- [:wat::core::i64])]); got (:usr::Holder :- [(:wat::core::Option::Some :- [:wat::core::i64])])"`.
  `container_is_enum("usr::Holder")` is `false` (it's `TypeDef::Aggregate`), so neither fallback
  arm fires and the pre-existing invariant `unify` path is what refuses it — unchanged from
  before this stone.
- **STOP-2** — did NOT fire. Both join controls (`two_sibling_variants_still_join_in_an_if`,
  `…_across_match_arms`) stayed green throughout, verbatim.
- **STOP-3** — did NOT fire. The gate is `container_is_enum` alone (see above); no hand-list was
  written or needed.
- **STOP-4** — did NOT fire. I did not touch `register_variant_types` or any namespace/prefix
  scoping; `src/types.rs`'s diff is exactly what the restored WIP already contained.
- **STOP-5** — did NOT fire. `src/record/construct.rs` has zero diff (`git status`/`git diff
  --stat` confirm no entry for that path).

## What this closes (the builder's expression)

```
(:user::app-describe
  (:wat::core::Option::Some {:value (:wat::core::Result::Err {:error "inner-boom"})}))
```

against `[o <- (Option :- [(Result :- [i64 String])])]` — `nested_variant_literal.wat` — now
checks clean (exit 0). Verified directly:

```
./target/release/wat --check tests/types/probe_arc296_A2_a_variant_is_a_type__nested_variant_literal.wat
exit=0
```

## Loose end ① — the SUPERSEDED probe row

`probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused` pinned
P-2a's monomorphic-only fence: `:wat::core::Option::Some` (bare, no type args) as an annotation
must stay refused as `UnknownNamedType`. Measured directly — after this stone it now checks
clean, exit 0 (the fixture `generic_variant_stays_refused.wat` uses exactly that annotation on a
`defn` parameter).

**Disposition chosen: SUPERSEDED** (the third of the seam's three — staleness/finding/superseded),
not staleness-capture and not a finding. Reasoning: P-2a itself was reverted
(`82bacfba0`, "sibling variants have no JOIN — and my rows could not see it"); its monomorphic-only
scope was that stone's own boundary, not a law of the type system. A-2 (this campaign) was
explicitly ruled to cover every enum — `register_variant_types` carries no monomorphic guard, and
the RELAND-4 probe's own `an_intrinsic_parameter_accepts_a_variant_like_a_user_defn_does` /
`a_stdlib_enums_variant_is_a_type_too` rows exercise exactly this population (a `wat::core::Option`
family variant, parametric) and are meant to pass. So the old row isn't wrong about what USED to
be true, and it isn't a defect A-2 introduced — it's a design boundary a later, ruled stone
deliberately moved past. I marked it `#[ignore = "SUPERSEDED by arc 296 A-2 (option E): …"]`
rather than deleting it (keeps the historical record of the boundary) and rather than rewriting
its assertion to the opposite (a silent rewrite would read as though the row always tested for
acceptance, erasing the fact that P-2a's own fence is what it used to pin).

## Loose end ② — `match` refusing a scrutinee already typed as its own variant

Reproduced exactly as the brief quoted, via the EXISTING (not new) fixture
`tests/function/recursive_patterns_t3.wat` / test `recursive_patterns::nested_options_three_levels`
(named in the WIP's own commit message as the 9th failure):

```
malformed :wat::core::match form: :wat::core::Option::Some pattern in
 (:wat::core::Option::Some :- [:wat::core::i64]) position
```

**Root cause, precisely isolated by bisection (see "a wrong turn" below):** the outer scrutinee's
own arg (an `Option::Some<i64>` nested inside another `Option::Some`) gets bound, via ordinary
`unify`, into the match's fresh shape variable — narrow and unwidened. That narrow type then
becomes the FIELD type a nested `(Some …)`/`(Ok …)`/`(Err …)`/`:enum::Variant` sub-pattern is
checked against, and the dispatch requires the BARE enum head, so it refuses. Fixed by widening
to the enclosing enum (`widen_to_enclosing_enum`, mechanism ③, already existed) at the THREE
dispatch-decision points inside the recursive sub-pattern checkers, and nowhere else:

- `check_nested_variant_map` (src/check.rs ~line 7427) — widened ONCE at entry (safe: this
  function only ever recurses via `bind_fields` -> `check_subpattern`, or errors; it never
  stores `expected_ty` itself into a binding).
- `check_subpattern`'s `:wat::core::Option::None` dispatch and its bare-Keyword
  `<enum>::<Variant>` dispatch (~line 7794, ~7834) — widened LOCALLY for each dispatch decision
  only (a Unit variant introduces no field binding, so nothing is stored).
- `check_subpattern`'s `WatAST::List` builtin `(Some _)`/`(Ok _)`/`(Err _)` dispatch and its
  Keyword-headed tagged-variant dispatch (~line 7944, ~8046) — widened LOCALLY for the dispatch
  match and the field-type args handed to the recursive calls; the bare-`Symbol` binder arm
  (`WatAST::Symbol(s, _) => bindings.insert(…, expected_ty.clone())`) is UNTOUCHED — it still
  stores the caller's own `expected_ty` exactly as received.

Verified:

```
./target/release/wat --check tests/function/recursive_patterns_t3.wat   # exit=0
cargo nextest run --release -E 'test(recursive_patterns)'               # 10 passed, 0 skipped
```

### A wrong turn, caught before it shipped — the regression and its fix

My FIRST attempt widened the shared per-arg `unify` inside `assignable`'s "`ah != eh`" arm
whenever the expected side still carried an unresolved type var (mirroring
`relate_value_to_slot`'s own pinning guard). That fixed `nested_options_three_levels` but broke
`probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error`
(`cargo nextest run --release -E 'test(arc278_journal_surface)' --test-threads=1`), whose golden
pins the NARROW variant name in a `TypeMismatch`'s `got` field:

```
thread 'probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error' panicked:
no check error matched `CheckErrorKind::TypeMismatch { expected, got, .. } if
expected == ":wat::telemetry::Journal::WriteMetricsResponse" && got ==
":wat::query::Store::PutResponse::Success"`; errors were:
#wat.check/CheckErrors {:message "3 type-check errors" ... :errors [#wat.check/TypeMismatch
{:message ":wat::telemetry::Journal::Reply::WriteMetrics: parameter resp expects
:wat::telemetry::Journal::WriteMetricsResponse; got :wat::query::Store::PutResponse" ...
:callee ":wat::telemetry::Journal::Reply::WriteMetrics" :param "resp" :expected
":wat::telemetry::Journal::WriteMetricsResponse" :got ":wat::query::Store::PutResponse"
:remedies []} ... (x3)]}
```

I bisected by disabling each new branch in turn (`if false && …`) and rebuilding, then added
targeted `eprintln!` tracing to pin the exact call. Root cause: `:wat::service::Outcome :- [S R
O]` is ITSELF an enum (`wat/service.wat`), so `defservice`'s auto-generated dispatcher's own
`(match handler-result [:wat::service::Outcome::Reply {:state s :reply resp} …])` — matching each
handler's OWN return value — runs through the exact same machinery. Widening there pinned the
running `R` type-var to the bare `Store::PutResponse` instead of the narrow `::Success` variant,
and a bare-Symbol pattern binder (`resp`) then stored that WIDENED type into `arm_locals`, which
a completely unrelated downstream check (the wire-protocol's own `Reply::WriteMetrics {:resp
resp}` constructor call) read back out and reported in ITS error message. I reverted that attempt
entirely (both the `assignable` pinning guard and a second wrong turn — widening
`infer_match`'s own shared `shape` value broke the SAME test via the SAME mechanism, and a THIRD
attempt widening `check_subpattern`'s `expected_ty` at its true top — including the bare-Symbol
arm — reproduced it again) before landing on the scoped fix above, which never touches a stored
binding, only a dispatch decision. Re-ran `probe_arc278_journal_surface` (both tests,
`--test-threads=1` per its own doc comment) after the final fix: **2 passed, 0 failed.**

## Regression sweep (beyond the acceptance filter)

Run after the final state, all green:

- `wat::types` (whole binary): 601 passed, 5 skipped (pre-existing, unrelated).
- `wat::services` (whole binary): 133 passed, 2 skipped (pre-existing, unrelated).
- `wat::wat_lang` (whole binary): 252 passed, 2 skipped.
- `wat::function` (whole binary): 220 passed, 1 skipped.
- Targeted filters: `match_` (118), `enum_` (71), `service` (63), `defservice` (6), `variant`
  (124, after the loose-end-① disposition), `recursive_patterns` (10), `record`/`record_` (220 /
  163), `fn_` (211), `closure` (39), `generic` (37) — all fully green.

I did not run `scripts/floor.sh`, an unfiltered `cargo nextest run`, or `cargo clippy` per the
brief's tier — those are the orchestrator's to run centrally.

## Deltas, honestly

- `src/check.rs`: +the `container_is_enum` gate; two new covariant-fallback arms in `assignable`
  (both on cloned substs); five scoped `widen_to_enclosing_enum` call sites across
  `check_nested_variant_map`/`check_subpattern` for loose end ②. No existing arm's prior
  behaviour was altered — every addition is a NEW acceptance path or a widen applied only at a
  point that was previously an unconditional refusal.
- `tests/types/probe_arc296_a2_a_variant_is_a_type.rs`: removed the 7 stale `#[ignore]`s (no
  fixture or assertion text changed).
- `tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type.rs`: one test re-marked
  `#[ignore]` with a SUPERSEDED reason (assertion body untouched, so the historical claim it made
  is still legible in the diff).
- `tests/services/probe_arc278_journal_surface.rs`: restored to the WIP's version (part of the
  brief's own restore instructions); no further edits by me — its golden already expected the
  narrow variant, which is exactly what my final, scoped fix preserves.
- Two loose ends closed, one SUPERSEDED disposition recorded, zero STOPs fired, zero known
  regressions after a sweep of ~1600 tests across the areas most likely to be touched by an
  `assignable`/match-pattern change.
