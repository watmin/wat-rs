# SCORE — STONE 255.89: STOP-3 — the converted corpus, more than twelve mechanisms

HEAD `d45406b02297abb70a623713be9e8b3b2f85ecab`. Not pushed. No cure was started.

## KEEP

Commit `9d85da5a518d4909b76df02c0ffa85549bd53a25`, `wat-scripts/fixes/to-faithful-clojure.keep`, 16 lines.

| reason | n |
|---|---:|
| negative proof of the retired keyword form `:fn(wat::core::i64)->wat::core::i64` | 2 |
| recorded codemod whose replay changes | 14 |

The two proofs are `tests/function/fn_rename_bare_fn_type.wat` and `tests/function/fn_rename_mixed_legacy.wat`.
Twelve of the codemods die after conversion with `alpha 6 of :wat::rete::fire-rules did not compile`. The other two, `to-faithful-clojure-net.wat` and `to-faithful-clojure-rete.wat`, stop rewriting: the replay got-output keeps the keyword fixture, including `:wat::core::Tuple(i64)`.
No tracked `*.wat` contains the text `:wat::core::Tuple(i64)`, so there is no third negative-proof row of that text. It lives in the replay fixtures, which are not `*.wat`.
`to-faithful-clojure.wat` itself is not on the list. It was converted with the rest of the corpus.

## Conversion

Tool copies taken out of the tree before the sweep: `/tmp/wat-pristine-25589` and `/tmp/to-faithful-clojure-25589.wat`. Batches of 40. Log `/tmp/g1-89-convert.log`.

```
CORPUS 2228 KEEP 16 TARGETS 2212 KEEP_MISSING []
HEADS_BEFORE_TOTAL 86840
HEADS_BEFORE benches 3 / crates 34 / docs 516 / examples 26 / tests 32225 / wat-scripts 47329 / wat-tests 6707
HEADS_BEFORE_KEEP 1768
HEADS_BEFORE_TARGETS 85072
HEADS_AFTER_TOTAL 1768
HEADS_AFTER benches 0 / crates 0 / docs 0 / examples 0 / tests 6 / wat-scripts 1762 / wat-tests 0
REMAIN_OUTSIDE_KEEP 0
FAILS 0
PASS2_CHANGED_N 0
```

The 1768 heads that remain are the KEEP files. The six outside `wat-scripts/` are:

```
tests/function/fn_rename_bare_fn_type.wat:2 :wat::core::defn
tests/function/fn_rename_bare_fn_type.wat:3 :wat::core::fn
tests/function/fn_rename_bare_fn_type.wat:8 :wat::core::fn
tests/function/fn_rename_mixed_legacy.wat:2 :wat::core::defn
tests/function/fn_rename_mixed_legacy.wat:3 :wat::core::lambda
tests/function/fn_rename_mixed_legacy.wat:8 :wat::core::fn
```

Commit `d45406b02297abb70a623713be9e8b3b2f85ecab`: `2205 files changed, 61151 insertions(+), 61151 deletions(-)`.
2212 targets, 2205 files in the commit, so 7 targets were byte-identical. Their names were not recorded. No file was refused. `wat/` was not in the corpus. `wat/core.wat` is 2330 lines.

## Census

Pre, unconverted tree: `.census/2026-10-03T22-42-45Z.txt`, 2293 files, `0` 2083, `1` 208, `101` 2.
Post, converted tree, after `cargo build --release --bin wat` (RC 0, 22.94s): `.census/2026-10-03T23-05-48Z.txt`, 2293 files, `0` 2066, `1` 225, `101` 2.
`census.sh --diff` exited 8. Same files both sides. Other rc-to-rc changes: 0.

NEW, rc 0 → 1, 20 files:

| file | check |
|---|---|
| `tests/function/probe_arc237_stone3_guard_ensure.wat` | MalformedForm, `:wat::core::i64::to-string` is retired |
| `tests/macros/probe_arc265_acronym_registry_svc.wat` | MalformedDecl, response type name is LAW, declared `` `:my::aws::Waf::CreateWebACLResponse` `` |
| `tests/rete/probe_arc251_8d_unquote_marker_identity.wat` | UnresolvedReference `:foo::bar` at lines 14 and 18 |
| `tests/services/probe_arc209_c2_defservice_dispatch.wat` | TypeMismatch ×4, `:my::Counter::Op` against `:my::Counter/Op` |
| `tests/services/probe_arc255_14_namespace_join.wat` | MalformedForm, `:wat::core::i64::to-string` is retired |
| `tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_accessor.wat` | UnresolvedReference `:u::Demo::Has::has` |
| `tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_accessor_runs.wat` | UnresolvedReference `:u::Demo::Has::has` |
| `tests/types/probe_arc251_type_the_polymorphic_accessor__named_variant_accessor_truth.wat` | UnresolvedReference `:u::Demo::Has::has` |
| `tests/value/wat_names_are_values.wat` | UnresolvedReference |
| `tests/wat_lang/probe_arc234_stone4_hash_destructure.wat` | MalformedForm, `:wat::core::i64::to-f64` is retired |
| `tests/wat_lang/probe_def_not_special.wat` | UnresolvedReference |
| `tests/wat_lang/wat_arc098_form_matches_runtime.wat` | pattern head must be a struct type keyword |
| `tests/wat_lang/wat_arc098_form_matches_typecheck.wat` | pattern head must be a struct type keyword |
| `tests/wat_lang/wat_arc157_def.wat` | UnresolvedReference |
| `tests/wat_lang/wat_arc157_def_redef_true_ok.wat` | UnresolvedReference |
| `wat-scripts/perf/grid/where-inline-computed.wat` | ReteCheckErrors |
| `wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat` | pattern head must be a struct type keyword |
| `wat-scripts/scratch-pad/probe-arc278-fnforms-walks-a-matches-pattern.wat` | pattern head must be a struct type keyword |
| `wat-scripts/scratch-pad/probe-arc278-reap-serve-event.wat` | `:wat::core::i64::to-string` is retired |
| `wat-tests/service-cache-lru.wat` | TypeMismatch ×2 |

RECOVERY, rc 1 → 0, 3 files. A refusal that stopped refusing:

| file | before |
|---|---|
| `tests/rete/probe_arc278_enum_variant_typo_bad.wat` | UnknownEnumVariant, `:evt::G` has no variant `Hii`. Now `evt.G/Hii` and rc 0 |
| `tests/rete/probe_arc278_enum_variant_typo_tagged.wat` | UnknownField, `:tg::Req` has no field `:tg::P.Hi`. Now rc 0 |
| `tests/types/probe_arc296_p1_annotation_names_a_type__bare_legacy_primitive.wat` | BareLegacyPrimitive on bare `:i64`. Now `wat.type/i64` and rc 0 |

The bare-`:i64` file's comment calls it the control for a retired lowercase primitive. It fits KEEP reason (a). It was converted anyway. Restoring the parent bytes would be a cure, and this score stops before cures.

## Replay

`cargo nextest run --release -E 'test(every_recorded_migration)'` on the committed tree:

```
Summary [   9.000s] 18 tests run: 18 passed, 5974 skipped
```

RC 0. `git status` clean after it. No replay golden changed.

## Floor

`.floor/2026-10-03T23-07-54Z`, `scripts/floor.sh`, exit 100. Not re-run. Tree was clean.

```
Summary [ 368.019s] 5969 tests run: 5676 passed (24 slow), 293 failed, 23 skipped
```

Header: `Starting 5969 tests across 49 binaries (23 tests skipped, including 5 tests via profile.default.default-filter)`.
Against `.floor/2026-10-03T13-09-11Z` at `30cd8b69c` (6412 passed, 24 skipped): 5969 run against 6412, 23 skipped against 24.
Doctests, from `doctest.log`: wat 5 passed, 1 ignored; wat_doc 0 tests. Both `test result: ok`.
`doc-link.log` ends `Finished release profile [optimized] target(s) in 11.25s` and `Generated …/target/doc/console_demo/index.html and 11 other files`. `doc-link-judge.log` is 0 bytes. The process exit is nextest's 100, so the judge's own rc was not printed.
Reachability shards passed: 0 at 22.490s, 1 at 22.535s, 2 at 20.759s, 3 at 20.861s, 4 at 20.799s, 5 at 21.017s.
`keyed_gather_visits_match_the_keyed_prediction` passed at 16.407s.
`retirement_table_is_fully_reachable` passed at 153.639s.
`clean.log` has no line naming `native-matches-oracle` or `native_matches_oracle`. That deftest was not in the 5969.
Clippy was not run. The ignores ledger was not run.

## Mechanisms

95 rows. The n column sums to 293. STOP-3 is more than 12.
A row shares one error sentence. An assertion row is the panic sentence of a failure that did not carry one of those sentences.
The quote is the arm text, cut at 420 characters. The whole block starts at the ARM line in `.floor/2026-10-03T23-07-54Z/ARM.txt`.

| n | mechanism | witness | ARM |
|---:|---|---|---:|
| 50 | fact-shaped cond has no minted alpha | `wat::rete probe_arc278_7a_negation_oracle::negation_blocks_when_present_matching` | 753 |
| 21 | accumulator expr is not total | `wat::rete probe_arc251_8d_make_rule_quote_boundary::the_make_rule_quote_boundary_reads_one_identity_in_both_spellings` | 442 |
| 19 | pattern head must be a struct-type keyword | `wat::wat_lang wat_arc098_form_matches_runtime::comparison_lt_gt_le_ge` | 29522 |
| 18 | TypeMismatch | `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_3_type_mismatch_renders_non_keyword_head` | 23384 |
| 15 | call canonicalizes to a retired :wat::core verb | `wat::rete probe_arc278_nested_wall::a_nested_multi_arg_positional_construction_is_refused` | 2813 |
| 9 | unresolved reference :t::pi | `wat::wat_lang wat_arc157_def::def_basic_float_literal` | 29974 |
| 8 | UnknownField diagnostic | `wat::rete probe_arc278_field_span::a_bind_clause_names_the_field_keyword_not_the_whole_bind` | 1832 |
| 7 | purity or determinism classifier | `wat::rete probe_arc278_55_slice_one_vocabulary::rete_fn_return_type_slot_is_not_classified_as_an_expression_deterministic` | 574 |
| 6 | reserved-prefix refusal | `wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defmacro_may_not_claim_a_wat_name` | 26449 |
| 5 | arrow or type conversion counted zero | `wat::rete probe_arc300_2_fix_defrule::left_arrow_deduces_arrowconv` | 3308 |
| 5 | restricted-namespace reader wants keyword spelling | `wat::kernel wat_arc198_def_restricted::def_restricted_caller_outside_allowed_namespace_fails` | 24487 |
| 5 | unresolved reference :my-app::tag::user-event | `wat::value wat_names_are_values::named_define_is_a_function_value` | 29180 |
| 4 | CheckErrors, inner kind not separated above | `wat::function recursive_patterns::nonexhaustive_partial_pattern_rejected` | 24252 |
| 4 | NotCallable render | `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_2_not_callable_renders_runtime_built_keyword` | 23432 |
| 4 | cannot lower head | `wat::rete probe_arc278_6b_ii_a_where_oracle::fence_rejects_impure_where_at_compile` | 690 |
| 4 | peers bijection ProgramBodyEvalFailed | `wat::services probe_arc278_peers_bijection::peers_bijection_form_spelling_missing_ephemeral_is_rejected` | 27086 |
| 4 | sift counts | `wat::services probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors` | 27352 |
| 4 | symbol spelling of an enum typo is admitted | `wat::rete probe_arc278_enum_variant_typo::a_bare_tagged_enum_variant_in_a_rete_constraint_is_refused` | 1397 |
| 3 | acronym LAW on the declared response type | `wat check::tests::declared_types_acronym_create_web_acl` | 3854 |
| 3 | dotted-name refusal | `wat::resolve probe_arc255_register_variant_is_its_own_door::a_defn_with_a_dotted_name_is_still_refused_end_to_end` | 26481 |
| 3 | dual-spelling pair collapsed to one line | `wat::resolve probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares` | 26350 |
| 3 | malformed match arm | `wat::rete probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires` | 2866 |
| 3 | vocabulary-admitted? expects a Keyword | `wat::rete probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head` | 500 |
| 2 | BareLegacyLambda golden | `wat::function fn_rename::lambda_post_retirement_fires_bare_legacy_lambda` | 23712 |
| 2 | assertion: [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :locat | `wat::rete probe_arc278_nested_wall::a_nested_single_positional_arg_against_a_wider_record_is_refused` | 2707 |
| 2 | assertion: assertion `left == right` failed | `wat::macros probe_arc265_acronym_registry::namespace_scoped_acronym_conversion_restores_casing` | 25112 |
| 2 | comparison head is not a rete primitive | `wat::rete probe_arc278_inline_constraint_law_a::untyped_ordering_constraint_is_refused` | 2338 |
| 2 | deftest scanner wants the keyword form | `wat::kernel test::wat_no_deftests_found` | 24416 |
| 2 | first on a short sequence | `wat::rete probe_arc278_D6_constraint_omission::a_tagged_enum_operand_is_named_not_dropped` | 1112 |
| 2 | format macro diagnostic | `wat::macros probe_arc279_format::format_strict_missing_kwarg_is_macro_error` | 24990 |
| 2 | keyword accessor | `wat::types probe_arc234_stone3c_keyword_accessor::probe_3_unknown_field_on_record_errors` | 27715 |
| 2 | nested variant pattern | `wat::rete probe_arc278_sqlite_interop::sqlite_interop` | 3125 |
| 2 | text must still contain raw :: | `wat::rete probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon` | 1244 |
| 2 | unresolved reference (path not in the arm) | `wat::wat_lang probe_def_not_special::probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end` | 29619 |
| 2 | unresolved reference :my::my-answer | `wat::wat_lang probe_def_not_special::probe_def_at_expression_position_emits_position_error_at_runtime` | 29459 |
| 2 | unresolved reference :u::Demo::Has::has | `wat::types probe_arc251_instantiate_the_type_argument::parametric_variant_accessor_yields_the_argument` | 27916 |
| 2 | where returned Ok instead of the gate | `wat::rete probe_arc278_6b_ii_a_where_oracle::where_passes_when_predicate_true` | 713 |
| 1 | allow is a process-tier gate | `wat::services probe_arc209_c0b3bb_verbs::thread_listener_allow_errors_with_tier_message` | 26963 |
| 1 | assertion: 21 of 21 grid axes FAILED the native-vs-oracle port check (0 axes agreed before the failures below): | `wat::rete wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle` | 3632 |
| 1 | assertion: arm 1 (surplus collides with slot 1; pre-fix ACCEPTED, 0 hits — a silent wrong answer): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` c | `wat::rete probe_arc278_export::arity_refuses_a_surplus_that_collides_with_a_declared_slot` | 1732 |
| 1 | assertion: arm 2 (surplus past a 1-wide frame; pre-fix ACCEPTED, 2 hits — the argument was dropped): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` | `wat::rete probe_arc278_export::arity_refuses_a_surplus_that_falls_past_the_frame` | 1691 |
| 1 | assertion: arm 3 (two arguments into a 0-param callee; pre-fix ACCEPTED, 2 hits — slot 0 fabricated): the call was ACCEPTED and the fence answered i64(0). `exec_program_on | `wat::rete probe_arc278_export::arity_refuses_arguments_to_a_zero_parameter_callee` | 1773 |
| 1 | assertion: arm 4 (zero args to a 1-param callee; pre-fix `UnboundSymbol: slot 1`, not an arity error): the call was ACCEPTED and the fence answered i64(0). `exec_program_o | `wat::rete probe_arc278_export::arity_refuses_a_call_with_no_arguments_at_all` | 1671 |
| 1 | assertion: arm 5 (one arg to a 2-param callee, evaluating path; pre-fix `UnboundSymbol: slot 1`): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` co | `wat::rete probe_arc278_export::arity_refuses_too_few_arguments_on_the_evaluating_path` | 1752 |
| 1 | assertion: assertion `left != right` failed: presence? must not silent-miss identity | `wat::rete probe_arc278_vsa_where_native_differential::differential_presence_four_row_native_eq_oracle` | 3545 |
| 1 | assertion: assertion `left != right` failed: where-accum-from-left: rewrite produced no fire-rules$oracle call — the family never fires? | `wat::rete wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family` | 3611 |
| 1 | assertion: assertion `left == right` failed: :user::native-coincident-id must Guess "identity" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_coincident_identity` | 3416 |
| 1 | assertion: assertion `left == right` failed: :user::native-coincident-not must Guess "not" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_coincident_not` | 3437 |
| 1 | assertion: assertion `left == right` failed: :user::native-const-false must Guess "const-false" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_const_false` | 3589 |
| 1 | assertion: assertion `left == right` failed: :user::native-const-true must Guess "const-true" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_const_true` | 3524 |
| 1 | assertion: assertion `left == right` failed: :user::native-id must Guess "identity" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_identity` | 3503 |
| 1 | assertion: assertion `left == right` failed: :user::native-not must Guess "not" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_not` | 3567 |
| 1 | assertion: assertion `left == right` failed: :user::native-presence-self must Guess "identity" exactly once (four-row catalog); got "count=0" | `wat::rete probe_arc278_vsa_where_native_differential::differential_presence_self` | 3482 |
| 1 | assertion: assertion `left == right` failed: ANCHOR: both engines must raise Ok2 to 1 — the term they share | `wat rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch` | 4756 |
| 1 | assertion: assertion `left == right` failed: Export must survive edn write/read and still fire | `wat::rete probe_arc278_export::edn_write_read_import_fires` | 1586 |
| 1 | assertion: assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only reco | `wat rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle` | 4675 |
| 1 | assertion: assertion `left == right` failed: Temp 10 is cool, Temp 30 is not | `wat::rete probe_arc278_export::source_session_derives_one_hit` | 1792 |
| 1 | assertion: assertion `left == right` failed: Temp 10 is cool, Temp 30 is not — one Hit | `wat::rete probe_arc278_expr_ir::compiled_where_fires_the_cool_rule` | 1973 |
| 1 | assertion: assertion `left == right` failed: `:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the fixture drifted and the probe below proves n | `wat::rete probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches` | 1498 |
| 1 | assertion: assertion `left == right` failed: all three keyword-constant routes must still compile AND fire — a count below 3 means the new refusal, or a widening of it, at | `wat::rete probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords` | 1521 |
| 1 | assertion: assertion `left == right` failed: and CONVERGE at 501 — the seed plus every step up to the bound. Admitting a rule set that then hangs would be worse than refus | `wat::rete probe_arc278_fixpoint_round_cap::a_fence_bounded_counter_is_admitted_and_its_wrong_way_twin_is_not` | 2127 |
| 1 | assertion: assertion `left == right` failed: and the imported session must actually DERIVE — a run that completes without doing the work proves nothing about the ceiling's | `wat::rete probe_arc278_import_accounting::import_refuses_a_build_that_outgrows_the_session_ceiling` | 2399 |
| 1 | assertion: assertion `left == right` failed: degenerate cosine must take caller :undefined (-1.0 and 7.0), not a constant; got [0, 0] | `wat::rete probe_arc278_vsa_where_native_differential::differential_degenerate_takes_caller_undefined` | 3460 |
| 1 | assertion: assertion `left == right` failed: first fire parks one Hit | `wat::rete probe_arc278_query_harvest_protocol::query_is_last_fire_harvest_insert_does_not_refresh` | 2986 |
| 1 | assertion: assertion `left == right` failed: import(export(import(e))) must fire the same Hit | `wat::rete probe_arc278_export::reexport_import_fires` | 1710 |
| 1 | assertion: assertion `left == right` failed: imported Export must fire the same as the source Session | `wat::rete probe_arc278_export::imported_export_derives_the_same_hit` | 1608 |
| 1 | assertion: assertion `left == right` failed: native ≥1 → 1 | `wat::rete probe_arc278_7exists_native_differential::native_exists_passes_once_and_blocks` | 895 |
| 1 | assertion: assertion `left == right` failed: the 3-arity `reduce` is total and must keep firing — the two fixtures differ ONLY in the `init` operand, so if this fails the  | `wat::rete probe_arc278_reduce_arity_totality::the_total_three_arity_form_still_fires` | 3007 |
| 1 | assertion: assertion `left == right` failed: the `where`-body boundary is the one this fix mirrors; a regression there would otherwise only show up somewhere far away | `wat::rete probe_arc278_then_is_an_expansion_boundary::the_lhs_path_that_already_worked_still_works` | 3230 |
| 1 | assertion: assertion `left == right` failed: the fixture fence is `(?c < 20)` over Temp 10 and Temp 30 | `wat::rete probe_arc278_export::untampered_export_answers_one_hit` | 2221 |
| 1 | assertion: assertion `left == right` failed: the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted one, or one that changed its row-driver  | `wat rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter` | 4784 |
| 1 | assertion: assertion `left == right` failed: the span should point at `?missing` itself, not at the enclosing fact-form or rule | `wat::rete probe_arc278_rhs_unbound_span::fire_time_unbound_var_points_at_the_users_operand` | 3064 |
| 1 | assertion: assertion `left == right` failed: the synthetic fence is constantly `10 < 20`, so both Temps pass | `wat::rete probe_arc278_export::a_well_formed_user_call_still_runs` | 1649 |
| 1 | assertion: assertion `left == right` failed: wat-scripts/probes/arc-170/probe-type-splice.wat must run clean (exit 0) — ruling E3, "the floor runs every probe and requires | `wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_type_splice` | 25271 |
| 1 | assertion: expected count=7 via the oracle; got Err(#wat.runtime/MalformedForm {:message "malformed :wat::rete::eval-insert form: ':then' item head 'cg/make-rate' names ne | `wat::rete probe_construction_headline::construct_and_return_derives_via_oracle` | 3395 |
| 1 | assertion: import-and-hits defn | `wat::rete probe_arc278_export::export_without_arm_refusal_names_the_wat_line` | 1885 |
| 1 | bare :i64 control no longer reports BareLegacyPrimitive | `wat::types probe_arc296_p1_annotation_names_a_type::a_bare_legacy_primitive_is_already_refused_by_its_own_wall` | 28806 |
| 1 | call head must be a keyword, symbol, or list | `wat::macros probe_arc249_threading::witness_thread_last_empty_step_desugars_to_call_on_acc` | 24926 |
| 1 | cond else diagnostic | `wat::macros probe_arc258_stone2b_macro_error::contract_02_non_exhaustive_cond_names_else` | 24867 |
| 1 | doc-row byte golden | `wat::cli pprintln_doc_row::doc_row_pprintln_matches_byte_golden` | 5463 |
| 1 | incomplete triple still says name <- :T | `wat::function fn_signature::malformed_args_vector_clear_error` | 23895 |
| 1 | macro-alias hash | `wat::macros probe_hash_scope_renumber::macro_alias_expands_to_same_hash_as_direct_primitive` | 25135 |
| 1 | macro-error sentinel | `wat::macros probe_arc258_stone2b_macro_error::contract_03_macro_error_surfaces_its_message` | 24809 |
| 1 | program edn golden | `wat::program probe_arc213_program_edn_roundtrip::t1_program_to_edn_is_plain_edn` | 25348 |
| 1 | rule-population walk found 18 files | `wat::lint rete_compile_gate::the_rule_declaring_population_is_not_vacuous` | 333 |
| 1 | thread-first first on empty list | `wat::macros probe_arc249_threading::witness_thread_first_empty_step_panics_at_expansion` | 24726 |
| 1 | unresolved reference :foo::bar | `wat::rete probe_arc251_8d_unquote_marker_identity::the_quasiquote_markers_are_identities_and_a_near_miss_is_still_not_one` | 422 |
| 1 | unresolved reference :my::app::totally-bogus | `wat::resolve probe_arc255_the_type_position_has_its_own_authority::a_bogus_call_head_carrying_a_type_binder_is_still_refused` | 26704 |
| 1 | unresolved reference :probe::dbl | `wat::lint nested_program_starts::nested_program_literals_start_on_the_child_path` | 24450 |
| 1 | unresolved reference :t::a | `wat::wat_lang wat_arc157_def::def_redef_set_redef_true_same_type_succeeds` | 30083 |
| 1 | unresolved reference :usr::nope | `wat::cli wat_repl::a_bad_line_does_not_end_the_session` | 22668 |
| 1 | unresolved reference :wat::core::Option/Some | `wat::resolve probe_arc255_the_blanket_hides_a_phantom_head::the_colon_spelling_no_longer_resolves` | 26761 |
| 1 | unresolved reference :wat::rete::f64::>X | `wat::resolve probe_arc255_the_blanket_hides_a_phantom_head::bogus_rete_head_is_refused_at_check_the_blanket_is_dead` | 26733 |
| 1 | unresolved reference :wat::spawn::ProcessLaunch/bogus-field | `wat::services probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time` | 26910 |
| 1 | wire purity wall | `wat::comms probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND` | 23110 |

### 1. fact-shaped cond has no minted alpha

n=50. `wat::rete probe_arc278_7a_negation_oracle::negation_blocks_when_present_matching`.

```
thread 'probe_arc278_7a_negation_oracle::negation_blocks_when_present_matching' (1504377) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_7a_negation_oracle.rs:39:5:
Maintenance at Oslo → 0 Unattended; got Err(#wat.runtime/MalformedForm {:message "malformed :wat::rete::fire-rules form: fact-shaped cond has no minted alpha — cannot compile driver: (ops/Maintenance (?loc :- :location))" :location #wat.core/Span {:file "tests/rete/probe_arc278_7a_negation_oracle.wat" :line 11 :col 18 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 11 :col 55}}} :causes [] :head ":wat::rete:: …
```

### 2. accumulator expr is not total

n=21. `wat::rete probe_arc251_8d_make_rule_quote_boundary::the_make_rule_quote_boundary_reads_one_identity_in_both_spellings`.

```
#wat.kernel/AssertionFailure {:thread "probe_arc251_8d_make_rule_quote_boundary::the_make_rule_quote_boundary_reads_one_identity_in_both_spellings" :message "compile-condition: accumulator expr is not total — ':wat::core::length' is not total" :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 609 :col 46} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1124 :s …
```

### 3. pattern head must be a struct-type keyword

n=19. `wat::wat_lang wat_arc098_form_matches_runtime::comparison_lt_gt_le_ge`.

```
thread 'wat_arc098_form_matches_runtime::comparison_lt_gt_le_ge' (1555541) panicked at src/freeze.rs:1176:9:
call_beside_value: fixture beside "/home/john/work/holon/wat-rs/tests/wat_lang/wat_arc098_form_matches_runtime.rs" failed to freeze: #wat.check/CheckErrors {:message "27 type-check errors" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "tests/wat_lang/wat_arc098_form_matches_runtime. …
```

### 4. TypeMismatch

n=18. `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_3_type_mismatch_renders_non_keyword_head`.

```
#wat.runtime/TypeMismatch {:message ":wat::core::apply: expected wat::type::keyword, got wat::type::String `\"not-a-keyword\"`" :location #wat.core/Span {:file "tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p3.wat" :line 1 :col 65 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 1 :col 80}}} :causes [] :op ":wat::core::apply" :expected "wat::type::keyword" :got {:type "wat::type::String" :rendere …
```

### 5. call canonicalizes to a retired :wat::core verb

n=15. `wat::rete probe_arc278_nested_wall::a_nested_multi_arg_positional_construction_is_refused`.

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/RhsPositionalConstructionRetired {:message \"defrule `nwp::r`: `:then` insert nests a raw positional construction of `:nwp::Inner` with 2 argument(s) — positional construction at a bare aggregate name is retired; use kwarg …
```

### 6. unresolved reference :t::pi

n=9. `wat::wat_lang wat_arc157_def::def_basic_float_literal`.

```
thread 'wat_arc157_def::def_basic_float_literal' (1556558) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/wat_arc157_def.rs:35:9:
expected startup success for tests/wat_lang/wat_arc157_def.wat; got: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":t::pi" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/wat_lang/wat_arc157_def.wat" :line 28 :col 45 :end #wat.core/Option.Some { …
```

### 7. UnknownField diagnostic

n=8. `wat::rete probe_arc278_field_span::a_bind_clause_names_the_field_keyword_not_the_whole_bind`.

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/UnknownField {:message \"defrule `fsb::r`: `:fsb/Src` has no field `:nofield`; available fields: [k]\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_field_span_bind.wat\" :line 12 :col 37 :end #wat.core/Option. …
```

### 8. purity or determinism classifier

n=7. `wat::rete probe_arc278_55_slice_one_vocabulary::rete_fn_return_type_slot_is_not_classified_as_an_expression_deterministic`.

```
thread 'probe_arc278_55_slice_one_vocabulary::rete_fn_return_type_slot_is_not_classified_as_an_expression_deterministic' (1502750) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_55_slice_one_vocabulary.rs:239:5:
assertion failed: is_true(":user::rete-fn-return-type-slot-not-classified-as-expr-det")
```

### 9. reserved-prefix refusal

n=6. `wat::resolve probe_arc255_the_reserved_prefix_wall_is_not_the_blanket::a_user_defmacro_may_not_claim_a_wat_name`.

```
#wat.macro/ReservedPrefix {:message "cannot declare macro :wat::core::sneakym — reserved prefix (:wat::, :rust::, :$bound::); user macros must use their own prefix" :location #wat.core/Span {:file "tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defmacro_wat.wat" :line 2 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 75}}} :causes [] :name ":wat::core::sneakym"}
```

### 10. arrow or type conversion counted zero

n=5. `wat::rete probe_arc300_2_fix_defrule::left_arrow_deduces_arrowconv`.

```
thread 'probe_arc300_2_fix_defrule::left_arrow_deduces_arrowconv' (1509281) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc300_2_fix_defrule.rs:58:5:
assertion `left == right` failed: '<-' deduces one ArrowConv; got i64(0)
```

### 11. restricted-namespace reader wants keyword spelling

n=5. `wat::kernel wat_arc198_def_restricted::def_restricted_caller_outside_allowed_namespace_fails`.

```
thread 'wat_arc198_def_restricted::def_restricted_caller_outside_allowed_namespace_fails' (1534125) panicked at /home/john/work/holon/wat-rs/tests/kernel/wat_arc198_def_restricted.rs:52:13:
assertion `left == right` failed: whitelist mismatch for tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat
```

### 12. unresolved reference :my-app::tag::user-event

n=5. `wat::value wat_names_are_values::named_define_is_a_function_value`.

```
thread 'wat_names_are_values::named_define_is_a_function_value' (1554811) panicked at /home/john/work/holon/wat-rs/tests/value/wat_names_are_values.rs:40:41:
startup: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":my-app::tag::user-event" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/value/wat_names_are_values.wat" :line 47 :col 13 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 47 :col 34} …
```

### 13. CheckErrors, inner kind not separated above

n=4. `wat::function recursive_patterns::nonexhaustive_partial_pattern_rejected`.

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::match form: non-exhaustive: (:wat::core::Option :- [T]) needs arms for both :None and (Some _), or a wildcard. (Arc 055 — narrowing patterns like `(Some (1 _))` are partial; add a fallback `_` arm.)" :location #wat.core/Span {:file "tests/function/recursive_patterns_nonexha …
```

### 14. NotCallable render

n=4. `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_2_not_callable_renders_runtime_built_keyword`.

```
#wat.runtime/NotCallable {:message "not callable: expected Function, got wat::type::keyword `:ns::nonexistent-verb` (built by :wat::keyword::from-string at tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p2.wat:3:13)" :location #wat.core/Span {:file "src/runtime.rs" :line 11274 :col 17 :end #wat.core/Option.None {}} :causes [] :got {:type "wat::type::keyword" :rendered ":ns::nonexistent-verb" :provenance  …
```

### 15. cannot lower head

n=4. `wat::rete probe_arc278_6b_ii_a_where_oracle::fence_rejects_impure_where_at_compile`.

```
thread 'probe_arc278_6b_ii_a_where_oracle::fence_rejects_impure_where_at_compile' (1504113) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_6b_ii_a_where_oracle.rs:96:5:
startup error did not match `StartupError::Runtime(e) if
```

### 16. peers bijection ProgramBodyEvalFailed

n=4. `wat::services probe_arc278_peers_bijection::peers_bijection_form_spelling_missing_ephemeral_is_rejected`.

```
#wat.macro/ProgramBodyEvalFailed {:message "macro :wat::service::defservice — program body eval failed" :location #wat.core/Span {:file "tests/services/probe_arc278_peers_bijection_case4_form_missing.wat" :line 44 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 70 :col 63}}} :causes [] :macro-name ":wat::service::defservice" :cause #wat.macro/MalformedTemplate {:message "malformed template: probe::call …
```

### 17. sift counts

n=4. `wat::services probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors`.

```
thread 'probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors' (1544487) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc278_sift_logs.rs:20:5:
expected sift-logs with a pure `level = :error` predicate to return exactly 1 survivor; got i64(-1)
```

### 18. symbol spelling of an enum typo is admitted

n=4. `wat::rete probe_arc278_enum_variant_typo::a_bare_tagged_enum_variant_in_a_rete_constraint_is_refused`.

```
thread 'probe_arc278_enum_variant_typo::a_bare_tagged_enum_variant_in_a_rete_constraint_is_refused' (1506355) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:71:5:
SILENT WRONG ANSWER: a rule constraining on the BARE tagged variant `:tg::P::Hi` — which has no bare value form at all, `(:tg::P::Hi 7)` is the only way to write one — compiled, fired, and printed "0" with exit 0 and no diagnostic. Core REFUSES the identical expression at check time (`parameter #2 expects [:wat::core::i64 :-> :tg::P]`), so the two engines disagree about the same input and rete ships the wrong answer.
```

### 19. acronym LAW on the declared response type

n=3. `wat check::tests::declared_types_acronym_create_web_acl`.

```
thread 'check::tests::declared_types_acronym_create_web_acl' (1509971) panicked at src/check.rs:24427:33:
register: #wat.type/MalformedDecl {:message "malformed :wat::core::defsurface declaration: op `create-web-acl` in surface :my::aws::Waf: response type name is LAW — declared `:my::aws::Waf::CreateWebACLResponse`, required `:my::aws::Waf::CreateWebAclResponse` (arc 278 #74, builder ruling 2026-08-05: an op's response type IS `<Op>Response`; rename the declaration to match)" :location #wat.core/Span {:file "src/check.r …
```

### 20. dotted-name refusal

n=3. `wat::resolve probe_arc255_register_variant_is_its_own_door::a_defn_with_a_dotted_name_is_still_refused_end_to_end`.

```
#wat.runtime/DottedName {:message "name ':my::Shape.Circle' contains a '.' in its name segment — reserved: a dot in a tag's NAME half means \"this is an enum variant\" (`#ns/Enum.Variant`), so a registered name may not contain one, or it could forge that tag; rename without the dot" :location #wat.core/Span {:file "tests/resolve/probe_arc255_register_variant_is_its_own_door__dotted_defn.wat" :line 5 :col 1 :end #wat. …
```

### 21. dual-spelling pair collapsed to one line

n=3. `wat::resolve probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares`.

```
thread 'probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares' (1541810) panicked at /home/john/work/holon/wat-rs/tests/resolve/probe_arc251_stone9_symbol_head_declaration.rs:50:5:
assertion `left != right` failed: declares: the pair's first lines are identical — the head spelling was never swapped
```

### 22. malformed match arm

n=3. `wat::rete probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires`.

```
thread 'probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires' (1508133) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_match_arm_is_not_a_call.rs:232:5:
the refusal must name the TRUE Stone-C reason (form-level arm exhaustiveness vs. head-level fence totality), not a fabricated arity error
```

### 23. vocabulary-admitted? expects a Keyword

n=3. `wat::rete probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head`.

```
thread 'probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head' (1502312) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_55_slice_one_vocabulary.rs:19:78:
eval: #wat.runtime/TypeMismatch {:message ":wat::rete::vocabulary-admitted?: expected :wat::WatAST holding a Keyword (a quoted head name), got wat::type::String `\"Symbol(Identifier { name: \"wat.rete.i64/>\", scopes: {} }, Span { file: \"tests/rete/probe_arc278_55_slice_one_vocabulary.wat\", line: 160, col: 50, end: Some(Pos { line: 160, col: 64 }) })\"`" :location #wat.core/Span {:file "tests/rete/probe_arc278_55_s …
```

### 24. BareLegacyLambda golden

n=2. `wat::function fn_rename::lambda_post_retirement_fires_bare_legacy_lambda`.

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/BareLegacyLambda {:message "':wat::core::lambda' is retired (arc 155); canonical FQDN is ':wat::core::fn'. Clojure-faithful single-letform vocabulary: lowercase 'fn' for function values (matches Clojure's user-facing `fn`). Rename ':wat::core::lambda' -> ':wat::core::fn' at the offending site." :location #wat.core/Span  …
```

### 25. assertion: [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :locat

n=2. `wat::rete probe_arc278_nested_wall::a_nested_single_positional_arg_against_a_wider_record_is_refused`.

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/RhsArityMismatch {:message \"defrule `nwa::r`: `:then` insert of `:nwa::Inner` expects 2 positional argument(s); got 1\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_nested_wall_arity.wat\" :line 18 :col 34 :e …
```

### 26. assertion: assertion `left == right` failed

n=2. `wat::macros probe_arc265_acronym_registry::namespace_scoped_acronym_conversion_restores_casing`.

```
thread 'probe_arc265_acronym_registry::namespace_scoped_acronym_conversion_restores_casing' (1535940) panicked at /home/john/work/holon/wat-rs/tests/macros/probe_arc265_acronym_registry.rs:40:5:
assertion `left == right` failed
```

### 27. comparison head is not a rete primitive

n=2. `wat::rete probe_arc278_inline_constraint_law_a::untyped_ordering_constraint_is_refused`.

```
thread 'probe_arc278_inline_constraint_law_a::untyped_ordering_constraint_is_refused' (1507798) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_inline_constraint_law_a.rs:93:5:
the refusal must NAME the offending head ":wat::core::>" so the diagnostic teaches; got:
```

### 28. deftest scanner wants the keyword form

n=2. `wat::kernel test::wat_no_deftests_found`.

```
thread 'test::wat_no_deftests_found' (1534119) panicked at /home/john/work/holon/wat-rs/tests/kernel/test.rs:17:1:
wat::test! found no `(:wat::test::deftest ...)` forms under /home/john/work/holon/wat-rs/wat-tests
```

### 29. first on a short sequence

n=2. `wat::rete probe_arc278_D6_constraint_omission::a_tagged_enum_operand_is_named_not_dropped`.

```
thread 'probe_arc278_D6_constraint_omission::a_tagged_enum_operand_is_named_not_dropped' (1505370) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_D6_constraint_omission.rs:100:5:
the D6 tagged-variant fixture must run to completion
```

### 30. format macro diagnostic

n=2. `wat::macros probe_arc279_format::format_strict_missing_kwarg_is_macro_error`.

```
#wat.macro/ProgramBodyEvalFailed {:message "macro :wat::core::format — program body eval failed" :location #wat.core/Span {:file "tests/macros/probe_arc279_format_missing_kwarg.wat" :line 5 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 5 :col 41}}} :causes [] :macro-name ":wat::core::format" :cause #wat.macro/MalformedTemplate {:message "malformed template: format: placeholder {y} has no matching kwa …
```

### 31. keyword accessor

n=2. `wat::types probe_arc234_stone3c_keyword_accessor::probe_3_unknown_field_on_record_errors`.

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :nonexistent form: keyword accessor: field \"nonexistent\" is not declared on :u::Plain (declared fields: x)" :location #wat.core/Span {:file "tests/types/probe_arc251_type_the_polymorphic_accessor__unknown_field.wat" :line 3 :col 52 :end #wat.core/Option.Some {:value #wat.core/Pos {:l …
```

### 32. nested variant pattern

n=2. `wat::rete probe_arc278_sqlite_interop::sqlite_interop`.

```
thread 'probe_arc278_sqlite_interop::sqlite_interop' (1508887) panicked at src/freeze.rs:1064:51:
sqlite_interop deftest must pass (real sqlite open/execute-ddl/execute/select round-trip + Constraint/Fatal fault classification)
```

### 33. text must still contain raw ::

n=2. `wat::rete probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon`.

```
thread 'probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon' (1506078) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_ast_to_source.rs:39:5:
ast->source must print raw `::` token text, not the `.`-dialed write-forms notation
```

### 34. unresolved reference (path not in the arm)

n=2. `wat::wat_lang probe_def_not_special::probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end`.

```
#wat.kernel/AssertionFailure {:thread "probe_def_not_special::probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end" :message "[#wat.kernel/LociDiedError.StartupError {:error #wat.resolve/UnresolvedReferences {:message \"1 unresolved reference\" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path \":h::local-answer\" :context \"namespaced symbol ref — not a builtin, not a registered fu …
```

### 35. unresolved reference :my::my-answer

n=2. `wat::wat_lang probe_def_not_special::probe_def_at_expression_position_emits_position_error_at_runtime`.

```
thread 'probe_def_not_special::probe_def_at_expression_position_emits_position_error_at_runtime' (1555291) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/probe_def_not_special.rs:80:41:
startup: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":my::my-answer" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/wat_lang/probe_def_not_special.wat" :line 12 :col 46 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 12 :col 58}}}}]}
```

### 36. unresolved reference :u::Demo::Has::has

n=2. `wat::types probe_arc251_instantiate_the_type_argument::parametric_variant_accessor_yields_the_argument`.

```
thread 'probe_arc251_instantiate_the_type_argument::parametric_variant_accessor_yields_the_argument' (1548367) panicked at /home/john/work/holon/wat-rs/tests/types/probe_arc251_instantiate_the_type_argument.rs:84:5:
assertion `left == right` failed: got: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":u::Demo::Has::has" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_accessor.wat" :line 6 : …
```

### 37. where returned Ok instead of the gate

n=2. `wat::rete probe_arc278_6b_ii_a_where_oracle::where_passes_when_predicate_true`.

```
thread 'probe_arc278_6b_ii_a_where_oracle::where_passes_when_predicate_true' (1504290) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_6b_ii_a_where_oracle.rs:65:5:
where (> 5 0) true → 1 Gate; got Ok(i64(0))
```

### 38. allow is a process-tier gate

n=1. `wat::services probe_arc209_c0b3bb_verbs::thread_listener_allow_errors_with_tier_message`.

```
#wat.runtime/MalformedForm {:message "malformed :wat::kernel::allow form: allow is a process-tier service gate; a thread listener's handle IS the grant" :location #wat.core/Span {:file "tests/services/probe_arc209_c0b3bb_verbs_thread.wat" :line 6 :col 29 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 6 :col 30}}} :causes [] :head ":wat::kernel::allow" :reason "allow is a process-tier service gate; a thread l …
```

### 39. assertion: 21 of 21 grid axes FAILED the native-vs-oracle port check (0 axes agreed before the failures below):

n=1. `wat::rete wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle`.

```
thread 'wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle' (1509808) panicked at /home/john/work/holon/wat-rs/tests/rete/wat_scripts_grid_port_check.rs:565:5:
21 of 21 grid axes FAILED the native-vs-oracle port check (0 axes agreed before the failures below):
```

### 40. assertion: arm 1 (surplus collides with slot 1; pre-fix ACCEPTED, 0 hits — a silent wrong answer): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` c

n=1. `wat::rete probe_arc278_export::arity_refuses_a_surplus_that_collides_with_a_declared_slot`.

```
thread 'probe_arc278_export::arity_refuses_a_surplus_that_collides_with_a_declared_slot' (1506717) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1015:18:
arm 1 (surplus collides with slot 1; pre-fix ACCEPTED, 0 hits — a silent wrong answer): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` compared no arity: a 2-argument call ran against a 1-parameter program.
```

### 41. assertion: arm 2 (surplus past a 1-wide frame; pre-fix ACCEPTED, 2 hits — the argument was dropped): the call was ACCEPTED and the fence answered i64(0). `exec_program_on`

n=1. `wat::rete probe_arc278_export::arity_refuses_a_surplus_that_falls_past_the_frame`.

```
thread 'probe_arc278_export::arity_refuses_a_surplus_that_falls_past_the_frame' (1506718) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1015:18:
arm 2 (surplus past a 1-wide frame; pre-fix ACCEPTED, 2 hits — the argument was dropped): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` compared no arity: a 2-argument call ran against a 1-parameter program.
```

### 42. assertion: arm 3 (two arguments into a 0-param callee; pre-fix ACCEPTED, 2 hits — slot 0 fabricated): the call was ACCEPTED and the fence answered i64(0). `exec_program_on

n=1. `wat::rete probe_arc278_export::arity_refuses_arguments_to_a_zero_parameter_callee`.

```
thread 'probe_arc278_export::arity_refuses_arguments_to_a_zero_parameter_callee' (1506720) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1015:18:
arm 3 (two arguments into a 0-param callee; pre-fix ACCEPTED, 2 hits — slot 0 fabricated): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` compared no arity: a 2-argument call ran against a 0-parameter program.
```

### 43. assertion: arm 4 (zero args to a 1-param callee; pre-fix `UnboundSymbol: slot 1`, not an arity error): the call was ACCEPTED and the fence answered i64(0). `exec_program_o

n=1. `wat::rete probe_arc278_export::arity_refuses_a_call_with_no_arguments_at_all`.

```
thread 'probe_arc278_export::arity_refuses_a_call_with_no_arguments_at_all' (1506716) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1015:18:
arm 4 (zero args to a 1-param callee; pre-fix `UnboundSymbol: slot 1`, not an arity error): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` compared no arity: a 0-argument call ran against a 1-parameter program.
```

### 44. assertion: arm 5 (one arg to a 2-param callee, evaluating path; pre-fix `UnboundSymbol: slot 1`): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` co

n=1. `wat::rete probe_arc278_export::arity_refuses_too_few_arguments_on_the_evaluating_path`.

```
thread 'probe_arc278_export::arity_refuses_too_few_arguments_on_the_evaluating_path' (1506723) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1015:18:
arm 5 (one arg to a 2-param callee, evaluating path; pre-fix `UnboundSymbol: slot 1`): the call was ACCEPTED and the fence answered i64(0). `exec_program_on` compared no arity: a 1-argument call ran against a 2-parameter program.
```

### 45. assertion: assertion `left != right` failed: presence? must not silent-miss identity

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_presence_four_row_native_eq_oracle`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_presence_four_row_native_eq_oracle' (1509205) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:127:5:
assertion `left != right` failed: presence? must not silent-miss identity
```

### 46. assertion: assertion `left != right` failed: where-accum-from-left: rewrite produced no fire-rules$oracle call — the family never fires?

n=1. `wat::rete wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family`.

```
thread 'wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family' (1509805) panicked at /home/john/work/holon/wat-rs/tests/rete/wat_scripts_grid_axes_live.rs:655:9:
assertion `left != right` failed: where-accum-from-left: rewrite produced no fire-rules$oracle call — the family never fires?
```

### 47. assertion: assertion `left == right` failed: :user::native-coincident-id must Guess "identity" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_coincident_identity`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_coincident_identity' (1509181) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-coincident-id must Guess "identity" exactly once (four-row catalog); got "count=0"
```

### 48. assertion: assertion `left == right` failed: :user::native-coincident-not must Guess "not" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_coincident_not`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_coincident_not' (1509187) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-coincident-not must Guess "not" exactly once (four-row catalog); got "count=0"
```

### 49. assertion: assertion `left == right` failed: :user::native-const-false must Guess "const-false" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_const_false`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_cosine_const_false' (1509194) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-const-false must Guess "const-false" exactly once (four-row catalog); got "count=0"
```

### 50. assertion: assertion `left == right` failed: :user::native-const-true must Guess "const-true" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_const_true`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_cosine_const_true' (1509197) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-const-true must Guess "const-true" exactly once (four-row catalog); got "count=0"
```

### 51. assertion: assertion `left == right` failed: :user::native-id must Guess "identity" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_identity`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_cosine_identity' (1509198) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-id must Guess "identity" exactly once (four-row catalog); got "count=0"
```

### 52. assertion: assertion `left == right` failed: :user::native-not must Guess "not" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_cosine_not`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_cosine_not' (1509200) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-not must Guess "not" exactly once (four-row catalog); got "count=0"
```

### 53. assertion: assertion `left == right` failed: :user::native-presence-self must Guess "identity" exactly once (four-row catalog); got "count=0"

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_presence_self`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_presence_self' (1509208) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:43:5:
assertion `left == right` failed: :user::native-presence-self must Guess "identity" exactly once (four-row catalog); got "count=0"
```

### 54. assertion: assertion `left == right` failed: ANCHOR: both engines must raise Ok2 to 1 — the term they share

n=1. `wat rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch`.

```
thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch' (1514812) panicked at src/rete/kernel/tests/stratify_numbers.rs:167:5:
assertion `left == right` failed: ANCHOR: both engines must raise Ok2 to 1 — the term they share
```

### 55. assertion: assertion `left == right` failed: Export must survive edn write/read and still fire

n=1. `wat::rete probe_arc278_export::edn_write_read_import_fires`.

```
thread 'probe_arc278_export::edn_write_read_import_fires' (1506724) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:314:5:
assertion `left == right` failed: Export must survive edn write/read and still fire
```

### 56. assertion: assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only reco

n=1. `wat rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle`.

```
thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle' (1514815) panicked at src/rete/kernel/tests/stratify_numbers.rs:241:5:
assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only recognises a top-level `:not` head
```

### 57. assertion: assertion `left == right` failed: Temp 10 is cool, Temp 30 is not

n=1. `wat::rete probe_arc278_export::source_session_derives_one_hit`.

```
thread 'probe_arc278_export::source_session_derives_one_hit' (1507082) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:12:5:
assertion `left == right` failed: Temp 10 is cool, Temp 30 is not
```

### 58. assertion: assertion `left == right` failed: Temp 10 is cool, Temp 30 is not — one Hit

n=1. `wat::rete probe_arc278_expr_ir::compiled_where_fires_the_cool_rule`.

```
thread 'probe_arc278_expr_ir::compiled_where_fires_the_cool_rule' (1507142) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_expr_ir.rs:22:5:
assertion `left == right` failed: Temp 10 is cool, Temp 30 is not — one Hit
```

### 59. assertion: assertion `left == right` failed: `:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the fixture drifted and the probe below proves n

n=1. `wat::rete probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches`.

```
thread 'probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches' (1506377) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:41:5:
assertion `left == right` failed: `:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the fixture drifted and the probe below proves nothing
```

### 60. assertion: assertion `left == right` failed: all three keyword-constant routes must still compile AND fire — a count below 3 means the new refusal, or a widening of it, at

n=1. `wat::rete probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords`.

```
thread 'probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords' (1506374) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:164:5:
assertion `left == right` failed: all three keyword-constant routes must still compile AND fire — a count below 3 means the new refusal, or a widening of it, ate a legitimate constant
```

### 61. assertion: assertion `left == right` failed: and CONVERGE at 501 — the seed plus every step up to the bound. Admitting a rule set that then hangs would be worse than refus

n=1. `wat::rete probe_arc278_fixpoint_round_cap::a_fence_bounded_counter_is_admitted_and_its_wrong_way_twin_is_not`.

```
thread 'probe_arc278_fixpoint_round_cap::a_fence_bounded_counter_is_admitted_and_its_wrong_way_twin_is_not' (1507417) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_fixpoint_round_cap.rs:396:5:
assertion `left == right` failed: and CONVERGE at 501 — the seed plus every step up to the bound. Admitting a rule set that then hangs would be worse than refusing it
```

### 62. assertion: assertion `left == right` failed: and the imported session must actually DERIVE — a run that completes without doing the work proves nothing about the ceiling's

n=1. `wat::rete probe_arc278_import_accounting::import_refuses_a_build_that_outgrows_the_session_ceiling`.

```
thread 'probe_arc278_import_accounting::import_refuses_a_build_that_outgrows_the_session_ceiling' (1507510) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_import_accounting.rs:112:5:
assertion `left == right` failed: and the imported session must actually DERIVE — a run that completes without doing the work proves nothing about the ceiling's headroom
```

### 63. assertion: assertion `left == right` failed: degenerate cosine must take caller :undefined (-1.0 and 7.0), not a constant; got [0, 0]

n=1. `wat::rete probe_arc278_vsa_where_native_differential::differential_degenerate_takes_caller_undefined`.

```
thread 'probe_arc278_vsa_where_native_differential::differential_degenerate_takes_caller_undefined' (1509203) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_vsa_where_native_differential.rs:91:5:
assertion `left == right` failed: degenerate cosine must take caller :undefined (-1.0 and 7.0), not a constant; got [0, 0]
```

### 64. assertion: assertion `left == right` failed: first fire parks one Hit

n=1. `wat::rete probe_arc278_query_harvest_protocol::query_is_last_fire_harvest_insert_does_not_refresh`.

```
thread 'probe_arc278_query_harvest_protocol::query_is_last_fire_harvest_insert_does_not_refresh' (1508282) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_query_harvest_protocol.rs:19:5:
assertion `left == right` failed: first fire parks one Hit
```

### 65. assertion: assertion `left == right` failed: import(export(import(e))) must fire the same Hit

n=1. `wat::rete probe_arc278_export::reexport_import_fires`.

```
thread 'probe_arc278_export::reexport_import_fires' (1506920) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:334:5:
assertion `left == right` failed: import(export(import(e))) must fire the same Hit
```

### 66. assertion: assertion `left == right` failed: imported Export must fire the same as the source Session

n=1. `wat::rete probe_arc278_export::imported_export_derives_the_same_hit`.

```
thread 'probe_arc278_export::imported_export_derives_the_same_hit' (1506824) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:58:5:
assertion `left == right` failed: imported Export must fire the same as the source Session
```

### 67. assertion: assertion `left == right` failed: native ≥1 → 1

n=1. `wat::rete probe_arc278_7exists_native_differential::native_exists_passes_once_and_blocks`.

```
thread 'probe_arc278_7exists_native_differential::native_exists_passes_once_and_blocks' (1504712) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_7exists_native_differential.rs:136:5:
assertion `left == right` failed: native ≥1 → 1
```

### 68. assertion: assertion `left == right` failed: the 3-arity `reduce` is total and must keep firing — the two fixtures differ ONLY in the `init` operand, so if this fails the 

n=1. `wat::rete probe_arc278_reduce_arity_totality::the_total_three_arity_form_still_fires`.

```
thread 'probe_arc278_reduce_arity_totality::the_total_three_arity_form_still_fires' (1508405) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_reduce_arity_totality.rs:113:5:
assertion `left == right` failed: the 3-arity `reduce` is total and must keep firing — the two fixtures differ ONLY in the `init` operand, so if this fails the refusal is too wide
```

### 69. assertion: assertion `left == right` failed: the `where`-body boundary is the one this fix mirrors; a regression there would otherwise only show up somewhere far away

n=1. `wat::rete probe_arc278_then_is_an_expansion_boundary::the_lhs_path_that_already_worked_still_works`.

```
thread 'probe_arc278_then_is_an_expansion_boundary::the_lhs_path_that_already_worked_still_works' (1508909) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_then_is_an_expansion_boundary.rs:105:5:
assertion `left == right` failed: the `where`-body boundary is the one this fix mirrors; a regression there would otherwise only show up somewhere far away
```

### 70. assertion: assertion `left == right` failed: the fixture fence is `(?c < 20)` over Temp 10 and Temp 30

n=1. `wat::rete probe_arc278_export::untampered_export_answers_one_hit`.

```
thread 'probe_arc278_export::untampered_export_answers_one_hit' (1507124) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1051:5:
assertion `left == right` failed: the fixture fence is `(?c < 20)` over Temp 10 and Temp 30
```

### 71. assertion: assertion `left == right` failed: the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted one, or one that changed its row-driver 

n=1. `wat rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter`.

```
thread 'rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter' (1515126) panicked at src/rete/kernel/tests/where_tree_branch_differential.rs:521:5:
assertion `left == right` failed: the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted one, or one that changed its row-driver shape all land here
```

### 72. assertion: assertion `left == right` failed: the span should point at `?missing` itself, not at the enclosing fact-form or rule

n=1. `wat::rete probe_arc278_rhs_unbound_span::fire_time_unbound_var_points_at_the_users_operand`.

```
thread 'probe_arc278_rhs_unbound_span::fire_time_unbound_var_points_at_the_users_operand' (1508556) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_rhs_unbound_span.rs:66:5:
assertion `left == right` failed: the span should point at `?missing` itself, not at the enclosing fact-form or rule
```

### 73. assertion: assertion `left == right` failed: the synthetic fence is constantly `10 < 20`, so both Temps pass

n=1. `wat::rete probe_arc278_export::a_well_formed_user_call_still_runs`.

```
thread 'probe_arc278_export::a_well_formed_user_call_still_runs' (1506684) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:1075:5:
assertion `left == right` failed: the synthetic fence is constantly `10 < 20`, so both Temps pass
```

### 74. assertion: assertion `left == right` failed: wat-scripts/probes/arc-170/probe-type-splice.wat must run clean (exit 0) — ruling E3, "the floor runs every probe and requires

n=1. `wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_type_splice`.

```
thread 'every_probe_runs::probe_wat_scripts_probes_arc_170_probe_type_splice' (1537888) panicked at /home/john/work/holon/wat-rs/target/release/build/wat-69bde1333b909ae2/out/every_probe_runs.rs:41:5:
assertion `left == right` failed: wat-scripts/probes/arc-170/probe-type-splice.wat must run clean (exit 0) — ruling E3, "the floor runs every probe and requires exit 0"; stdout:
```

### 75. assertion: expected count=7 via the oracle; got Err(#wat.runtime/MalformedForm {:message "malformed :wat::rete::eval-insert form: ':then' item head 'cg/make-rate' names ne

n=1. `wat::rete probe_construction_headline::construct_and_return_derives_via_oracle`.

```
thread 'probe_construction_headline::construct_and_return_derives_via_oracle' (1509465) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_construction_headline.rs:70:5:
expected count=7 via the oracle; got Err(#wat.runtime/MalformedForm {:message "malformed :wat::rete::eval-insert form: ':then' item head 'cg/make-rate' names neither a known fact-type constructor nor a registered fn — the rule-compile fence should have refused this" :location #wat.core/Span {:file "tests/rete/probe_construction_headline_green.wat" :line 19 :col 10 :end #wat.core/Option.Some {:value #wat.core/Pos {:li …
```

### 76. assertion: import-and-hits defn

n=1. `wat::rete probe_arc278_export::export_without_arm_refusal_names_the_wat_line`.

```
thread 'probe_arc278_export::export_without_arm_refusal_names_the_wat_line' (1506732) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:285:10:
import-and-hits defn
```

### 77. bare :i64 control no longer reports BareLegacyPrimitive

n=1. `wat::types probe_arc296_p1_annotation_names_a_type::a_bare_legacy_primitive_is_already_refused_by_its_own_wall`.

```
thread 'probe_arc296_p1_annotation_names_a_type::a_bare_legacy_primitive_is_already_refused_by_its_own_wall' (1551451) panicked at /home/john/work/holon/wat-rs/tests/types/probe_arc296_p1_annotation_names_a_type.rs:101:5:
assertion `left == right` failed: `:i64` bare is BareLegacyPrimitive — a named diagnostic naming its FQDN replacement
```

### 78. call head must be a keyword, symbol, or list

n=1. `wat::macros probe_arc249_threading::witness_thread_last_empty_step_desugars_to_call_on_acc`.

```
#wat.runtime/MalformedForm {:message "malformed int form: call head must be a keyword, symbol, or list" :location #wat.core/Span {:file "tests/macros/probe_arc249_threading_witness_tl_empty.wat" :line 2 :col 17 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 18}}} :causes [] :head "int" :reason "call head must be a keyword, symbol, or list"}
```

### 79. cond else diagnostic

n=1. `wat::macros probe_arc258_stone2b_macro_error::contract_02_non_exhaustive_cond_names_else`.

```
#wat.macro/ProgramBodyEvalFailed {:message "macro :wat::core::cond — program body eval failed" :location #wat.core/Span {:file "tests/macros/probe_arc258_stone2b_macro_error_c02.wat" :line 5 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 7 :col 28}}} :causes [] :macro-name ":wat::core::cond" :cause #wat.macro/MalformedTemplate {:message "malformed template: cond: non-exhaustive — needs a terminal :els …
```

### 80. doc-row byte golden

n=1. `wat::cli pprintln_doc_row::doc_row_pprintln_matches_byte_golden`.

```
thread 'pprintln_doc_row::doc_row_pprintln_matches_byte_golden' (1521243) panicked at /home/john/work/holon/wat-rs/tests/cli/pprintln_doc_row.rs:32:5:
assertion `left == right` failed: byte golden: #wat.doc/Row from a record value
```

### 81. incomplete triple still says name <- :T

n=1. `wat::function fn_signature::malformed_args_vector_clear_error`.

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::fn form: triple is incomplete; expected `name <- :T` but ran out of items" :location #wat.core/Span {:file "tests/function/fn_signature_malformed_args.wat" :line 6 :col 36 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 6 :col 37}}} :causes [] :head ":wat::core::fn" …
```

### 82. macro-alias hash

n=1. `wat::macros probe_hash_scope_renumber::macro_alias_expands_to_same_hash_as_direct_primitive`.

```
thread 'probe_hash_scope_renumber::macro_alias_expands_to_same_hash_as_direct_primitive' (1536248) panicked at /home/john/work/holon/wat-rs/tests/macros/probe_hash_scope_renumber.rs:124:5:
assertion `left == right` failed: macro alias (:test::MyAlias 42 99) and direct call (:my::prim 42 99 1 -1) must hash EQUAL after expansion — the hash-IS-identity claim in src/macros/mod.rs:4-8 requires macro-transparent canonical hashing.
```

### 83. macro-error sentinel

n=1. `wat::macros probe_arc258_stone2b_macro_error::contract_03_macro_error_surfaces_its_message`.

```
#wat.macro/ProgramBodyEvalFailed {:message "macro :user::boom — program body eval failed" :location #wat.core/Span {:file "tests/macros/probe_arc258_stone2b_macro_error_c03.wat" :line 6 :col 42 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 6 :col 53}}} :causes [] :macro-name ":user::boom" :cause #wat.macro/MalformedTemplate {:message "malformed template: kaboom-sentinel-9173" :location #wat.core/Span {:file …
```

### 84. program edn golden

n=1. `wat::program probe_arc213_program_edn_roundtrip::t1_program_to_edn_is_plain_edn`.

```
#wat.ast/Program {:origins ["probe_arc213.wat"] :forms [#wat.ast/Spanned {:node (#wat.ast/Spanned {:node wat.config/set-capacity-mode! :origin 0 :line 9 :col 2 :end-line 9 :end-col 31} #wat.ast/Spanned {:node :error :origin 0 :line 9 :col 32 :end-line 9 :end-col 38}) :origin 0 :line 9 :col 1 :end-line 9 :end-col 39} #wat.ast/Spanned {:node (#wat.ast/Spanned {:node wat.core/defstruct :origin 0 :line 10 :col 2 :end-lin …
```

### 85. rule-population walk found 18 files

n=1. `wat::lint rete_compile_gate::the_rule_declaring_population_is_not_vacuous`.

```
thread 'rete_compile_gate::the_rule_declaring_population_is_not_vacuous' (1500488) panicked at /home/john/work/holon/wat-rs/tests/lint/rete_compile_gate.rs:338:5:
found only 18 rule-declaring file(s) under wat-scripts/ — under the floor of 100, so the walk is not reaching the population it claims to guard and a green verdict from the shards above would mean nothing
```

### 86. thread-first first on empty list

n=1. `wat::macros probe_arc249_threading::witness_thread_first_empty_step_panics_at_expansion`.

```
#wat.macro/ProgramBodyEvalFailed {:message "macro :wat::core::-> — program body eval failed" :location #wat.core/Span {:file "tests/macros/probe_arc249_threading_witness_tf_empty.wat" :line 2 :col 3 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 2 :col 21}}} :causes [] :macro-name ":wat::core::->" :cause #wat.macro/MacroEvalRuntimeFailed {:message "macro_eval: runtime::eval failed" :location #wat.core/Span { …
```

### 87. unresolved reference :foo::bar

n=1. `wat::rete probe_arc251_8d_unquote_marker_identity::the_quasiquote_markers_are_identities_and_a_near_miss_is_still_not_one`.

```
thread 'probe_arc251_8d_unquote_marker_identity::the_quasiquote_markers_are_identities_and_a_near_miss_is_still_not_one' (1500730) panicked at src/freeze.rs:1176:9:
call_beside_value: fixture beside "/home/john/work/holon/wat-rs/tests/rete/probe_arc251_8d_unquote_marker_identity.rs" failed to freeze: #wat.resolve/UnresolvedReferences {:message "2 unresolved references" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":foo::bar" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/re …
```

### 88. unresolved reference :my::app::totally-bogus

n=1. `wat::resolve probe_arc255_the_type_position_has_its_own_authority::a_bogus_call_head_carrying_a_type_binder_is_still_refused`.

```
#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":my::app::totally-bogus" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_bogus_head_carrying_a_binder.wat" :line 13 :col 24 :end  …
```

### 89. unresolved reference :probe::dbl

n=1. `wat::lint nested_program_starts::nested_program_literals_start_on_the_child_path`.

```
thread 'nested_program_starts::nested_program_literals_start_on_the_child_path' (1498964) panicked at /home/john/work/holon/wat-rs/tests/lint/nested_program_starts.rs:614:5:
10 nested program(s) failed child startup:
```

### 90. unresolved reference :t::a

n=1. `wat::wat_lang wat_arc157_def::def_redef_set_redef_true_same_type_succeeds`.

```
thread 'wat_arc157_def::def_redef_set_redef_true_same_type_succeeds' (1556630) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/wat_arc157_def.rs:47:45:
startup: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":t::a" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/wat_lang/wat_arc157_def_redef_true_ok.wat" :line 5 :col 47 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 5 :col 50}}}}]}
```

### 91. unresolved reference :usr::nope

n=1. `wat::cli wat_repl::a_bad_line_does_not_end_the_session`.

```
#wat.core/Fault {:message "1 unresolved reference" :location #wat.kernel/Location {:file "<runtime>" :line 0 :col 0} :causes [#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":usr::nope" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "<read-string>" :lin …
```

### 92. unresolved reference :wat::core::Option/Some

n=1. `wat::resolve probe_arc255_the_blanket_hides_a_phantom_head::the_colon_spelling_no_longer_resolves`.

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::core::Option/Some" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__colon_spelling_is_refus …
```

### 93. unresolved reference :wat::rete::f64::>X

n=1. `wat::resolve probe_arc255_the_blanket_hides_a_phantom_head::bogus_rete_head_is_refused_at_check_the_blanket_is_dead`.

```
[#wat.kernel/LociDiedError.StartupError {:error #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::rete::f64::>X" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat" :line  …
```

### 94. unresolved reference :wat::spawn::ProcessLaunch/bogus-field

n=1. `wat::services probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time`.

```
#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::spawn::ProcessLaunch/bogus-field" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat" :line 19 :col 72 :end #wat.core/Option.Some {:value …
```

### 95. wire purity wall

n=1. `wat::comms probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND`.

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::program::self-peer form: a comm carries only pure data — type :w2c::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)." :location #wat.core/Span {:fil …
```

## What was not run

Clippy `--release --all-targets -- -D warnings` was not run. The SEAM ignores ledger was not run. No second floor was run. No `.wat` was hand-edited. No golden was re-captured.

## STOP

STOP-3. 95 mechanisms in the table above, the floor is `.floor/2026-10-03T23-07-54Z`, and no cure was started.
The bare-`:i64` recovery and the enum-typo admission are rows 18, 77. They were not restored onto the KEEP list.
5c-iv and 5d were not started.
