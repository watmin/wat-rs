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

## Amend — the vanished tests run, and the floor still stops

The STOP-3 record above stands, including what that floor did not run. This section is the amend. It continues at `1c9fd486d`. Nothing in the STOP-3 table was regrouped in place; the sentence grouping stays as the record of that floor. The table below is the new floor, grouped by the code site that decides.

### Commits on this amend

Local `main`, not pushed. The floor below ran on `204b9cb98`.

- `2459375cc` — enumerators read a wat head by identity in both spellings. `crates/wat-macros/src/discover.rs` has a local `canonical_identity`. `sanitize_name` strips a leading `:`, replaces every non-alphanumeric with `_`, and collapses runs. The stored site name stays the source spelling.
- `2fe5ff0ed` — `tests/types/probe_arc296_p1_annotation_names_a_type__bare_legacy_primitive.wat` restored with `git checkout 9d85da5a5 --` that path, and a KEEP row appended. Bytes begin `(:wat::core::defn :user::f [x <- :i64] -> :i64 x)`. Restoring those bytes is not a hand-edit.
- `e969cf456` — a nonexistent enum variant is refused in both spellings. The deciding site is `check_operand_field_ref` in `src/rete/validate/typing.rs`. The old enum-typo goldens were not rewritten.
- `091380729` — `run_single_deftest` looks up `canonical_identity` of the written deftest name. Probe `wat-tests/probe-arc255-89-deftest-lookup.wat` holds one deftest in each spelling.
- `5db1ac528` — the `identifier::path` read in `tests/lint/rete_compile_gate.rs` carries a namespace rune.
- `204b9cb98` — the both-spellings refusal is compared as EDN against a first capture of that probe. That golden is not a rewrite of an older golden.

`flat` stays. 5c-iv and 5d were not started. No converted corpus file was hand-edited.

### The floor that scheduled the names and could not look them up

`.floor/2026-10-03T23-51-20Z`. Do not re-run.

```
Summary [ 381.850s] 6421 tests run: 5672 passed (24 slow), 749 failed, 24 skipped
```

The scanner stored the written spelling and `defn` registers `canonical_identity`, so a symbol deftest panicked `not found in frozen symbols` at `src/host/test_runner.rs:473`. The same floor also failed `no_loose_string_assert` on the new probe's `contains` calls and `one_variant_separator` on `identifier::path(`. Those three are this stone's own reds. They were cured in `091380729`, `204b9cb98`, and `5db1ac528`, and a new floor followed. On that new floor the needle `not found in frozen symbols` is 0.

### The set of test names

Against `.floor/2026-10-03T13-09-11Z` (the green 6412 at `30cd8b69c`), the new floor has MISSING 0 and ADDED 11. No rename row.

```
wat-macros discover::tests::one_file_holds_a_deftest_in_each_spelling
wat-macros discover::tests::primed_and_hermetic_symbol_heads_are_deftests
wat-macros discover::tests::sanitized_loader_name_is_stable_across_spellings
wat-macros discover::tests::symbol_alias_is_called_in_both_spellings
wat-macros discover::tests::symbol_head_that_is_not_an_annotation_still_clears_pending
wat::kernel test::deftest_wat_tests_probe25589_lookup_keyword_spelling
wat::kernel test::deftest_wat_tests_probe25589_lookup_symbol_spelling
wat::lint nested_program_starts::deftest_rust_name_is_stable_across_spellings
wat::lint rete_compile_gate::extraction::keyword_and_symbol_defrules_in_one_source_are_both_found
wat::rete probe_arc255_89_enum_variant_both_spellings::a_nonexistent_variant_is_refused_in_both_spellings
wat::rete wat_scripts_grid_axes_live::rewrite_fire_to_spec_sees_both_spellings
```

Each of those 11 passed on the new floor, including the lookup pair at 0.866s and 0.939s, the both-spellings refusal at 0.494s, and `with-loader-example::test deftest_user_with_loader_test_test_loader_wiring` at 0.527s. `the_rule_declaring_population_is_not_vacuous` passed at 0.167s. `only_identifier_rs_spells_the_variant_separator` passed at 0.095s. `ignore_reasons_are_justified` passed at 0.060s. That last test bans phrases; it does not assert that the ignore count is 18.

### Enumerators

Cured, and the probe that shows it passed on this floor:

- Deftest discovery, its annotations, primed and hermetic forms, and aliases (`discover.rs`). One file holds a deftest in each spelling.
- The runtime lookup (`src/host/test_runner.rs`). The same file holds a keyword deftest and a symbol deftest.
- `declared_namespaces` in `tests/lint/rete_compile_gate.rs`.
- `rustify_deftest` in `tests/lint/nested_program_starts.rs`.
- The grid oracle rewrite `rewrite_fire_to_spec_sees_both_spellings`.

Cured off this floor's nextest set: the four shell twins (`run-axis.sh`, `check-grid-three-way.sh`, `check-query-compat.sh`, `check-spec-native.sh`) grew a second substitution for `wat.rete/fire-rules`. `wat-source-derive` heads already go through `form_identity`; the crate is not in `default-members`.

Found and not cured, because they do not match a head spelling:

- `build.rs` `collect_wat_files` (line 198) collects `*.wat` by extension. `every_probe_runs` is generated from that list.
- `src/rete/reachability.rs` replaces the keyword `:wat::rete::fire-rules ` inside an embedded source constant.
- The keyword arms in `crates/wat-reader` `ast.rs` are the parser.

The grid bodies are not green. On this floor `spec_equals_native_on_every_where_family` failed at 75.665s, `grid_axes_run_and_derive_nonvacuously` failed at 52.595s, and `every_grid_axis_native_matches_its_oracle` failed at 23.859s. `nested_program_literals_start_on_the_child_path` failed at 98.670s. Their deciding lines were not separated from the rows below.

### The new floor

`.floor/2026-10-04T00-08-26Z`. Do not re-run.

```
Summary [ 386.688s] 6423 tests run: 6110 passed (25 slow), 313 failed, 24 skipped
```

`Starting 6423 tests across 49 binaries (24 tests skipped, including 5 tests via profile.default.default-filter)`.

Doctests exited 0. wat: 5 passed, 1 ignored (`src/restriction_entry.rs` line 54). wat_doc: 0. wat_edn: 3 passed. wat_macros: 0 passed, 4 ignored. wat_reader: 0. wat_to_edn_derive: 0.

`cargo doc` finished: `Finished release profile [optimized] target(s) in 11.42s`. `doc-link.log` is 38459 bytes and ends on unresolved-link warnings. `doc-link-judge.log` is 0 bytes. Nextest was red, so the floor did not print a judge rc. The judge is not claimed.

Nextest prints each failure twice. The counts below are the first block of each of the 313 names. A test is counted on every needle its block contains. The needles are not a partition, and their sum is not 313.

| needle | tests |
| --- | ---: |
| fact-shaped cond has no minted alpha | 65 |
| cond did not compile | 20 |
| is not total | 38 |
| is not pure | 6 |
| is not deterministic | 1 |
| is not a rete primitive | 3 |
| cannot lower head | 5 |
| pattern head must be a struct type keyword | 19 |
| is retired | 19 |
| holding a Keyword | 3 |
| UnknownEnumVariant | 1 |
| UnknownField | 10 |
| unresolved reference | 28 |
| TypeMismatch | 28 |
| left == right | 115 |
| response type name is LAW | 3 |
| malformed match arm | 3 |
| head spelling was never swapped | 3 |
| verbatim | 6 |
| not declared in :peers | 2 |
| sift- | 3 |
| not found in frozen symbols | 0 |
| i64(0) | 21 |
| a comm carries only pure data | 1 |

`left == right` and `i64(0)` are symptoms. They are not a deciding site. Many of them are a program printing 0 because an alpha was not minted.

### Time limits on this floor

The fuzz deftest failed fast. The 90s limit did not fire.

```
        FAIL [   4.953s] ( 393/6423) wat::kernel test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle
  stdout ───

    running 1 test
    test test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle ... FAILED

    failures:

    failures:
        test::deftest_wat_tests_rete_fuzz_test_native_matches_oracle

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 638 filtered out; finished in 4.94s

  stderr ───

    thread 'wat-test::wat-tests.rete.fuzz/test-native-matches-oracle' (2629634) panicked at src/host/test_runner.rs:496:17:
    test differential-fuzz.wat :: wat-tests.rete.fuzz/test-native-matches-oracle
      failure: #wat.runtime/MalformedForm {:message "malformed :wat::rete::fire-rules form: fact-shaped cond has no minted alpha — cannot compile driver: (wat-tests.rete.fuzz/W (?w :- :k))" :location #wat.core/Span {:file "wat-tests/rete/differential-fuzz.wat" :line 140 :col 47 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 140 :col 81}}} :causes [] :head ":wat::rete::fire-rules" :reason "fact-shaped cond has no minted alpha — cannot compile driver: (wat-tests.rete.fuzz/W (?w :- :k))"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Reachability shards passed: shard 0 at 22.930s, shard 1 at 22.807s, shard 4 at 20.992s, shard 5 at 20.928s, shard 2 at 20.706s, shard 3 at 20.556s. `retirement_table_is_fully_reachable` passed at 165.581s. `keyed_gather_visits_match_the_keyed_prediction` passed at 16.329s.

### Deciding sites

One row is one site. The Rust Debug `left` / `right` lines are omitted from the long EDN mismatches; they repeat the same two values and they are in the floor log. A block is cut at the next test's `PASS` line when the extractor overran.

**1. `src/rete/kernel/arm.rs:118`.** `alpha_by_text.get(&cond_text(cond))` else the minted-alpha sentence. `cond_text` is `src/rete/kernel/node.rs:214`, `wat_edn::write` of the cond. A keyword-minted alpha and a symbol cond are different text. 65 tests. The fuzz block above is this site. The explain-order block is the same sentence:

```
        FAIL [   1.031s] ( 651/6423) wat::rete probe_arc278_explain_order::or_control_a_single_arm_agrees
  stdout ───

    running 1 test
    test probe_arc278_explain_order::or_control_a_single_arm_agrees ... FAILED

    failures:

    failures:
        probe_arc278_explain_order::or_control_a_single_arm_agrees

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 1.00s

  stderr ───

    thread 'probe_arc278_explain_order::or_control_a_single_arm_agrees' (2634279) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_explain_order.rs:59:67:
    or fixture must run: #wat.runtime/MalformedForm {:message "malformed :wat::rete::fire-rules form: fact-shaped cond has no minted alpha — cannot compile driver: (orx/A1 (?k :- :k))" :location #wat.core/Span {:file "tests/rete/probe_arc278_explain_order.wat" :line 83 :col 12 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 83 :col 31}}} :causes [] :head ":wat::rete::fire-rules" :reason "fact-shaped cond has no minted alpha — cannot compile driver: (orx/A1 (?k :- :k))"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**2. `src/rete/kernel/arm.rs:374`.** `compile_condition_local` returning none becomes `alpha {aid} cond did not compile`. 20 tests contain that sentence. This block's exit code is the assertion; the stderr is that sentence.

```
        FAIL [   0.873s] (2359/6423) wat::cli wat_grep::g4_balanced_file_is_silent_and_clean
  stdout ───

    running 1 test
    test wat_grep::g4_balanced_file_is_silent_and_clean ... FAILED

    failures:

    failures:
        wat_grep::g4_balanced_file_is_silent_and_clean

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 90 filtered out; finished in 0.87s

  stderr ───

    thread 'wat_grep::g4_balanced_file_is_silent_and_clean' (2652554) panicked at /home/john/work/holon/wat-rs/tests/cli/wat_grep.rs:135:5:
    assertion `left == right` failed: balanced input must exit 0; stderr: [#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::rete::fire-rules form: alpha 6 cond did not compile — setup should compile every fact-shaped alpha\" :location #wat.core/Span {:file \"tests/cli/wat_grep__any_symbol_rule.wat\" :line 9 :col 10 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 9 :col 82}}} :causes [] :head \":wat::rete::fire-rules\" :reason \"alpha 6 cond did not compile — setup should compile every fact-shaped alpha\"}"}]

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**3. `wat/rete/compile.wat:345` prints the sentence. The witness attributes `wat/rete/compile.wat:609`.** Line 345 formats `expr is not total`. Line 609 is the accumulator fence calling `axis-violation-message`. The false-total decision is upstream of the printer. 38 tests contain `is not total`. This block names head `':wat::core::length'`.

```
        FAIL [   0.879s] ( 354/6423) wat::kernel test::deftest_wat_tests_rete_fuzz_test_accumulate_kinds_are_not_vacuous
  stdout ───

    running 1 test
    test test::deftest_wat_tests_rete_fuzz_test_accumulate_kinds_are_not_vacuous ... FAILED

    failures:

    failures:
        test::deftest_wat_tests_rete_fuzz_test_accumulate_kinds_are_not_vacuous

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 638 filtered out; finished in 0.86s

  stderr ───
    #wat.kernel/AssertionFailure {:thread "wat-thread-peer::<anon>" :message "compile-condition: accumulator expr is not total — ':wat::core::length' is not total" :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 609 :col 46} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1145 :symbol ":wat::rete::compile-condition"} #wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1180 :symbol ":wat::rete::compile-query"} #wat.kernel/Frame {:file "wat-tests/rete/differential-fuzz.wat" :line 463 :symbol ":wat::rete::compile-all"} #wat.kernel/Frame {:file "wat-tests/rete/differential-fuzz.wat" :line 481 :symbol ":wat-tests::rete::fuzz::nv-rows"} #wat.kernel/Frame {:file "wat/spawn.wat" :line 375 :symbol ":wat::type::Fn"}] :upstream-chain nil}

    thread 'wat-test::wat-tests.rete.fuzz/test-accumulate-kinds-are-not-vacuous' (2629623) panicked at src/host/test_runner.rs:496:17:
    test differential-fuzz.wat :: wat-tests.rete.fuzz/test-accumulate-kinds-are-not-vacuous
      failure: compile-condition: accumulator expr is not total — ':wat::core::length' is not total
        at:       wat/rete/compile.wat:609:46
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**4. `src/rete/expr_ir/mod.rs:745`.** `cannot lower head {head}`. 5 tests. This block expected `is not a rete primitive` for `:wat::i64::>` and got the lowerer.

```
        FAIL [   0.842s] ( 905/6423) wat::rete probe_fence_names_the_head::core_op_where_names_law_a_not_total
  stdout ───

    running 1 test
    test probe_fence_names_the_head::core_op_where_names_law_a_not_total ... FAILED

    failures:

    failures:
        probe_fence_names_the_head::core_op_where_names_law_a_not_total

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.83s

  stderr ───

    thread 'probe_fence_names_the_head::core_op_where_names_law_a_not_total' (2637354) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_fence_names_the_head.rs:123:5:
    startup error did not match `StartupError::Runtime(e) if
    matches!(e.kind(), RuntimeErrorKind::AssertionFailed { message, .. } if
    message ==
    "compile-condition: where expr is not a rete primitive — ':wat::i64::>' is not a rete primitive; a where admits only :wat::rete:: ops")`:
    #wat.runtime/MalformedForm {:message "malformed :wat::rete::lower form: cannot lower head :wat::i64::>" :location #wat.core/Span {:file "tests/rete/probe_fence_names_the_head_core_op.wat" :line 13 :col 20 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 13 :col 36}}} :causes [] :head ":wat::rete::lower" :reason "cannot lower head :wat::i64::>"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**5. `src/check.rs:14664`.** A `wat.form/matches?` pattern head that is not a `Keyword` is `pattern head must be a struct type keyword`. 19 tests. The twin string at `src/reflect/match.rs:131` is a different function. This block is the type-check.

```
        FAIL [   0.839s] (5633/6423) wat::wat_lang wat_arc098_form_matches_typecheck::valid_simple_binding_and_comparison
  stdout ───

    running 1 test
    test wat_arc098_form_matches_typecheck::valid_simple_binding_and_comparison ... FAILED

    failures:

    failures:
        wat_arc098_form_matches_typecheck::valid_simple_binding_and_comparison

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 254 filtered out; finished in 0.83s

  stderr ───

    thread 'wat_arc098_form_matches_typecheck::valid_simple_binding_and_comparison' (2691339) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/wat_arc098_form_matches_typecheck.rs:35:12:
    startup should succeed for valid patterns: #wat.check/CheckErrors {:message "3 type-check errors" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "tests/wat_lang/wat_arc098_form_matches_typecheck.wat" :line 12 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 12 :col 24}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []} #wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "tests/wat_lang/wat_arc098_form_matches_typecheck.wat" :line 21 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 21 :col 24}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []} #wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "tests/wat_lang/wat_arc098_form_matches_typecheck.wat" :line 34 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 34 :col 24}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**6. `src/check.rs:6177`.** `remedies_for` score 0 emits `'{k}' is retired; use '{form}' instead`. 19 tests contain `is retired`. The load gate below shows the converted call that canonicalizes to `':wat::core::i64::to-string'` and is refused with remedy `':wat::i64::to-string'`. The lambda block further down is a span column on a message that already matches; it is row 12, not this emitter's cure.

**7. `src/intrinsic/rete.rs:301`.** `vocabulary-admitted?` matches only `WatAST::Keyword`. 3 tests contain `holding a Keyword`.

```
        FAIL [   0.827s] ( 418/6423) wat::rete probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head
  stdout ───

    running 1 test
    test probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head ... FAILED

    failures:

    failures:
        probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.82s

  stderr ───

    thread 'probe_arc278_55_slice_one_vocabulary::admission_admits_a_rete_module_head' (2631307) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_55_slice_one_vocabulary.rs:19:78:
    eval: #wat.runtime/TypeMismatch {:message ":wat::rete::vocabulary-admitted?: expected :wat::WatAST holding a Keyword (a quoted head name), got wat::type::String `\"Symbol(Identifier { name: \"wat.rete.i64/>\", scopes: {} }, Span { file: \"tests/rete/probe_arc278_55_slice_one_vocabulary.wat\", line: 160, col: 50, end: Some(Pos { line: 160, col: 64 }) })\"`" :location #wat.core/Span {:file "tests/rete/probe_arc278_55_slice_one_vocabulary.wat" :line 160 :col 34 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 160 :col 65}}} :causes [] :op ":wat::rete::vocabulary-admitted?" :expected ":wat::WatAST holding a Keyword (a quoted head name)" :got {:type "wat::type::String" :rendered "\"Symbol(Identifier { name: \"wat.rete.i64/>\", scopes: {} }, Span { file: \"tests/rete/probe_arc278_55_slice_one_vocabulary.wat\", line: 160, col: 50, end: Some(Pos { line: 160, col: 64 }) })\"" :provenance nil}}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**8. `src/types.rs:4228`.** The required response name is `format` of `surface_base` plus `kebab_to_pascal_with_acronyms` (`src/types.rs:4221`). The panic wrapper is `src/check.rs:24427`. 3 tests contain `response type name is LAW`. This one is `CreateWebACLResponse` against `CreateWebAclResponse`. It is not a slash-versus-colon failure.

```
        FAIL [   0.484s] ( 941/6423) wat check::tests::declared_types_acronym_create_web_acl
  stdout ───

    running 1 test
    test check::tests::declared_types_acronym_create_web_acl ... FAILED

    failures:

    failures:
        check::tests::declared_types_acronym_create_web_acl

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1400 filtered out; finished in 0.47s

  stderr ───

    thread 'check::tests::declared_types_acronym_create_web_acl' (2637796) panicked at src/check.rs:24427:33:
    register: #wat.type/MalformedDecl {:message "malformed :wat::core::defsurface declaration: op `create-web-acl` in surface :my::aws::Waf: response type name is LAW — declared `:my::aws::Waf::CreateWebACLResponse`, required `:my::aws::Waf::CreateWebAclResponse` (arc 278 #74, builder ruling 2026-08-05: an op's response type IS `<Op>Response`; rename the declaration to match)" :location #wat.core/Span {:file "src/check.rs:24425" :line 15 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 24 :col 83}}} :causes [] :head ":wat::core::defsurface" :reason "op `create-web-acl` in surface :my::aws::Waf: response type name is LAW — declared `:my::aws::Waf::CreateWebACLResponse`, required `:my::aws::Waf::CreateWebAclResponse` (arc 278 #74, builder ruling 2026-08-05: an op's response type IS `<Op>Response`; rename the declaration to match)"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**9. `src/resolve/normalize.rs:991`.** Context `namespaced symbol ref — not a builtin, not a registered function (arc 251)`. 28 tests contain `unresolved reference`. The same string at `src/resolve/mod.rs:383` is a unit-test assert, not this emitter.

```
        FAIL [   0.436s] (5707/6423) wat::wat_lang wat_arc157_def::def_basic_float_literal
  stdout ───

    running 1 test
    test wat_arc157_def::def_basic_float_literal ... FAILED

    failures:

    failures:
        wat_arc157_def::def_basic_float_literal

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 254 filtered out; finished in 0.42s

  stderr ───

    thread 'wat_arc157_def::def_basic_float_literal' (2692132) panicked at /home/john/work/holon/wat-rs/tests/wat_lang/wat_arc157_def.rs:35:9:
    expected startup success for tests/wat_lang/wat_arc157_def.wat; got: #wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":t::pi" :context "namespaced symbol ref — not a builtin, not a registered function (arc 251)" :span #wat.core/Span {:file "tests/wat_lang/wat_arc157_def.wat" :line 28 :col 45 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 28 :col 49}}}}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**10. `src/edn/render.rs:3757`.** A string that already contains `::` and does not start with `:` or `(` is returned as `:{s}`. That is why `:wat::cache::Cache/GetResult` stays distinct from `:wat::cache::Cache::GetResult`. Proven on this test. The other TypeMismatch tests (28 contain the word) were not shown to be this line.

```
        FAIL [   0.796s] (3611/6423) wat::kernel test::deftest_wat_tests_service_cache_lru_multi_client_on_thread
  stdout ───

    running 1 test
    test test::deftest_wat_tests_service_cache_lru_multi_client_on_thread ... FAILED

    failures:

    failures:
        test::deftest_wat_tests_service_cache_lru_multi_client_on_thread

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 638 filtered out; finished in 0.78s

  stderr ───

    thread 'wat-test::wat-tests.service/cache-lru-multi-client-on-thread' (2668098) panicked at src/host/test_runner.rs:467:13:
    test-runner: /home/john/work/holon/wat-rs/wat-tests/service-cache-lru.wat: startup: #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::core::foldl: parameter #1 expects [(wat.type/Vector :- [wat.type/String]) (:wat::cache::Cache::GetResult :- [wat.type/i64]) :-> (wat.type/Vector :- [wat.type/String])]; got [(wat.type/Vector :- [wat.type/String]) (:wat::cache::Cache/GetResult :- [wat.type/i64]) :-> (wat.type/Vector :- [wat.type/String])]" :location #wat.core/Span {:file "wat-tests/service-cache-lru.wat" :line 81 :col 19 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 84 :col 80}}} :causes [] :callee ":wat::core::foldl" :param "#1" :expected "[(wat.type/Vector :- [wat.type/String]) (:wat::cache::Cache::GetResult :- [wat.type/i64]) :-> (wat.type/Vector :- [wat.type/String])]" :got "[(wat.type/Vector :- [wat.type/String]) (:wat::cache::Cache/GetResult :- [wat.type/i64]) :-> (wat.type/Vector :- [wat.type/String])]" :remedies []} #wat.check/TypeMismatch {:message ":wat-tests::cache-svc::result-label: parameter #1 expects (:wat::cache::Cache::GetResult :- [wat.type/i64]); got (:wat::cache::Cache/GetResult :- [wat.type/i64])" :location #wat.core/Span {:file "wat-tests/service-cache-lru.wat" :line 84 :col 74 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 84 :col 77}}} :causes [] :callee ":wat-tests::cache-svc::result-label" :param "#1" :expected "(:wat::cache::Cache::GetResult :- [wat.type/i64])" :got "(:wat::cache::Cache/GetResult :- [wat.type/i64])" :remedies []}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**11. Test-text kind: an exact EDN golden pins the keyword spelling of a fact type, a field, or a span.** The old enum-typo goldens were not rewritten. Unknown-enum is 1 test. Unknown-field is 10 tests; those counts overlap other rows.

```
        FAIL [   0.578s] ( 641/6423) wat::rete probe_arc278_enum_variant_typo::the_misspelled_variant_refusal_names_the_enum_and_its_real_variants
  stdout ───

    running 1 test
    test probe_arc278_enum_variant_typo::the_misspelled_variant_refusal_names_the_enum_and_its_real_variants ... FAILED

    failures:

    failures:
        probe_arc278_enum_variant_typo::the_misspelled_variant_refusal_names_the_enum_and_its_real_variants

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.57s

  stderr ───

    thread 'probe_arc278_enum_variant_typo::the_misspelled_variant_refusal_names_the_enum_and_its_real_variants' (2634253) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:101:5:
    assertion `left == right` failed: EDN data mismatch (the refusal must be `#wat.rete/UnknownEnumVariant` carrying `:enum-path "evt::G"`, `:variant "Hii"` and `:available-variants ["Hi" "Lo"]` — captured whole, so a reordered field, an appended remedy, or a fallback to `UnknownField` all fail here)
    --- actual (raw) ---
    [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/UnknownEnumVariant {:message \"defrule `evt::good` (`:evt/Req`): `:evt::G` has no variant `Hii`; available variants: [Hi, Lo]\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_enum_variant_typo_bad.wat\" :line 27 :col 30 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 27 :col 69}}} :causes [] :rule \"evt::good\" :fact-type \"evt/Req\" :enum-path \"evt::G\" :variant \"Hii\" :available-variants [\"Hi\" \"Lo\"]}]}" :location nil :causes [] :errors [#wat.rete/UnknownEnumVariant {:rule "evt::good" :fact-type "evt/Req" :enum-path "evt::G" :variant "Hii" :available-variants ["Hi" "Lo"] :span #wat.core/Span {:file "tests/rete/probe_arc278_enum_variant_typo_bad.wat" :line 27 :col 30 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 27 :col 69}}}}]}}]
    --- expected (raw) ---
    [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/UnknownEnumVariant {:message \"defrule `evt::good` (`:evt::Req`): `:evt::G` has no variant `Hii`; available variants: [Hi, Lo]\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_enum_variant_typo_bad.wat\" :line 27 :col 32 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 27 :col 79}}} :causes [] :rule \"evt::good\" :fact-type \"evt::Req\" :enum-path \"evt::G\" :variant \"Hii\" :available-variants [\"Hi\" \"Lo\"]}]}" :location nil :causes [] :errors [#wat.rete/UnknownEnumVariant {:rule "evt::good" :fact-type "evt::Req" :enum-path "evt::G" :variant "Hii" :available-variants ["Hi" "Lo"] :span #wat.core/Span {:file "tests/rete/probe_arc278_enum_variant_typo_bad.wat" :line 27 :col 32 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 27 :col 79}}}}]}}]

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
```
        FAIL [   0.668s] ( 640/6423) wat::rete probe_arc278_enum_variant_typo::the_bare_tagged_variant_keeps_the_unknown_field_route
  stdout ───

    running 1 test
    test probe_arc278_enum_variant_typo::the_bare_tagged_variant_keeps_the_unknown_field_route ... FAILED

    failures:

    failures:
        probe_arc278_enum_variant_typo::the_bare_tagged_variant_keeps_the_unknown_field_route

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.66s

  stderr ───

    thread 'probe_arc278_enum_variant_typo::the_bare_tagged_variant_keeps_the_unknown_field_route' (2634185) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:144:5:
    assertion `left == right` failed: EDN data mismatch (the tagged arm's landing, captured whole and NOT endorsed: `#wat.rete/UnknownField` with `:available-fields ["k" "grade"]`. Naming it is row 4 of the scorecard; fixing it is a separate strike)
    --- actual (raw) ---
    [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/UnknownField {:message \"defrule `tg::good`: `:tg/Req` has no field `:tg/P.Hi`; available fields: [k, grade]\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_enum_variant_typo_tagged.wat\" :line 26 :col 58 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 26 :col 65}}} :causes [] :rule \"tg::good\" :fact-type \"tg/Req\" :field \"tg/P.Hi\" :available-fields [\"k\" \"grade\"]}]}" :location nil :causes [] :errors [#wat.rete/UnknownField {:rule "tg::good" :fact-type "tg/Req" :field "tg/P.Hi" :available-fields ["k" "grade"] :span #wat.core/Span {:file "tests/rete/probe_arc278_enum_variant_typo_tagged.wat" :line 26 :col 58 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 26 :col 65}}}}]}}]
    --- expected (raw) ---
    [#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/UnknownField {:message \"defrule `tg::good`: `:tg::Req` has no field `:tg::P.Hi`; available fields: [k, grade]\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_enum_variant_typo_tagged.wat\" :line 26 :col 65 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 26 :col 74}}} :causes [] :rule \"tg::good\" :fact-type \"tg::Req\" :field \"tg::P.Hi\" :available-fields [\"k\" \"grade\"]}]}" :location nil :causes [] :errors [#wat.rete/UnknownField {:rule "tg::good" :fact-type "tg::Req" :field "tg::P.Hi" :available-fields ["k" "grade"] :span #wat.core/Span {:file "tests/rete/probe_arc278_enum_variant_typo_tagged.wat" :line 26 :col 65 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 26 :col 74}}}}]}}]

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**12. Test-text kind: the reason text matches and the golden pins a column.** Peers: end column 63 against 66. The reason string is produced at `wat/service.wat:927`. The lambda retirement golden: end column 20 against 23, same `BareLegacyLambda` message. Comms: the purity-wall sentence is identical, columns 35–40 against 38–45. That sentence is produced at `src/check.rs:10541`. One test contains `a comm carries only pure data`. Two tests contain `not declared in :peers`.

```
        FAIL [   0.378s] (4451/6423) wat::services probe_arc278_peers_bijection::peers_bijection_form_spelling_undeclared_peer_names_the_surface
  stdout ───

    running 1 test
    test probe_arc278_peers_bijection::peers_bijection_form_spelling_undeclared_peer_names_the_surface ... FAILED

    failures:

    failures:
        probe_arc278_peers_bijection::peers_bijection_form_spelling_undeclared_peer_names_the_surface

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 156 filtered out; finished in 0.37s

  stderr ───

    thread 'probe_arc278_peers_bijection::peers_bijection_form_spelling_undeclared_peer_names_the_surface' (2679287) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc278_peers_bijection.rs:160:5:
    assertion `left == right` failed: EDN data mismatch (form spelling, undeclared ephemeral peer must match the bijection's "extra" diagnostic and must name probe::Echo)
    --- actual (raw) ---
    #wat.macro/ProgramBodyEvalFailed {:message "macro :wat::service::defservice — program body eval failed" :location #wat.core/Span {:file "tests/services/probe_arc278_peers_bijection_case5_form_extra.wat" :line 44 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 68 :col 63}}} :causes [] :macro-name ":wat::service::defservice" :cause #wat.macro/MalformedTemplate {:message "malformed template: probe::caller: :ephemeral holds a dialed Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency DAG)" :location #wat.core/Span {:file "wat/service.wat" :line 927 :col 27 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 935 :col 110}}} :causes [] :reason "probe::caller: :ephemeral holds a dialed Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency DAG)"}}
    --- expected (raw) ---
    #wat.macro/ProgramBodyEvalFailed {
      :message "macro :wat::service::defservice — program body eval failed"
      :location #wat.core/Span {
        :file "tests/services/probe_arc278_peers_bijection_case5_form_extra.wat"
        :line 44
        :col 1
        :end #wat.core/Option.Some {
          :value #wat.core/Pos {
            :line 68
            :col 66
          }
        }
      }
      :causes []
      :macro-name ":wat::service::defservice"
      :cause #wat.macro/MalformedTemplate {
        :message "malformed template: probe::caller: :ephemeral holds a dialed Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency DAG)"
        :location #wat.core/Span {
          :file "wat/service.wat"
          :line 927
          :col 27
          :end #wat.core/Option.Some {
            :value #wat.core/Pos {
              :line 935
              :col 110
            }
          }
        }
        :causes []
        :reason "probe::caller: :ephemeral holds a dialed Peer<probe::Echo::Op,…::Reply> but surface :probe::Echo is not declared in :peers — add :peers [… :probe::Echo …] (the explicit s2s dependency DAG)"
      }
    }

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
```
        FAIL [   0.361s] (2943/6423) wat::function fn_rename::lambda_post_retirement_fires_bare_legacy_lambda
  stdout ───

    running 1 test
    test fn_rename::lambda_post_retirement_fires_bare_legacy_lambda ... FAILED

    failures:

    failures:
        fn_rename::lambda_post_retirement_fires_bare_legacy_lambda

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 245 filtered out; finished in 0.35s

  stderr ───

    thread 'fn_rename::lambda_post_retirement_fires_bare_legacy_lambda' (2659814) panicked at /home/john/work/holon/wat-rs/tests/function/fn_rename.rs:72:5:
    assertion `left == right` failed: EDN data mismatch (fnr1: BareLegacyLambda golden)
    --- actual (raw) ---
    #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/BareLegacyLambda {:message "':wat::core::lambda' is retired (arc 155); canonical FQDN is ':wat::core::fn'. Clojure-faithful single-letform vocabulary: lowercase 'fn' for function values (matches Clojure's user-facing `fn`). Rename ':wat::core::lambda' -> ':wat::core::fn' at the offending site." :location #wat.core/Span {:file "tests/function/fn_rename_legacy_lambda.wat" :line 3 :col 5 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 3 :col 20}}} :causes [] :retired ":wat::core::lambda" :fqdn ":wat::core::fn"}]}
    --- expected (raw) ---
    #wat.check/CheckErrors {
      :message "1 type-check error"
      :location nil
      :causes []
      :errors [
        #wat.check/BareLegacyLambda {
          :message "':wat::core::lambda' is retired (arc 155); canonical FQDN is ':wat::core::fn'. Clojure-faithful single-letform vocabulary: lowercase 'fn' for function values (matches Clojure's user-facing `fn`). Rename ':wat::core::lambda' -> ':wat::core::fn' at the offending site."
          :location #wat.core/Span {
            :file "tests/function/fn_rename_legacy_lambda.wat"
            :line 3
            :col 5
            :end #wat.core/Option.Some {
              :value #wat.core/Pos {
                :line 3
                :col 23
              }
            }
          }
          :causes []
          :retired ":wat::core::lambda"
          :fqdn ":wat::core::fn"
        }
      ]
    }

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
The comms block, with the Rust Debug lines omitted:

```
        FAIL [   0.791s] (2687/6423) wat::comms probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND
  stdout ───

    running 1 test
    test probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND ... FAILED

    failures:

    failures:
        probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 93 filtered out; finished in 0.77s

  stderr ───

    thread 'probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND' (2658341) panicked at /home/john/work/holon/wat-rs/tests/comms/probe_arc293_W2a_struct_no_cross.rs:96:5:
    assertion `left == right` failed: EDN data mismatch (check error must match arc 293 §7 purity wall golden)
    --- actual (raw) ---
    #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::program::self-peer form: a comm carries only pure data — type :w2c::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)." :location #wat.core/Span {:file "tests/comms/probe_arc293_W2c_compile_time_send.wat" :line 25 :col 35 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 25 :col 40}}} :causes [] :head ":wat::program::self-peer" :reason "a comm carries only pure data — type :w2c::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)." :remedies []}]}
    --- expected (raw) ---
    #wat.check/CheckErrors {
      :message "1 type-check error"
      :location nil
      :causes []
      :errors [
        #wat.check/MalformedForm {
          :message "malformed :wat::program::self-peer form: a comm carries only pure data — type :w2c::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)."
          :location #wat.core/Span {
            :file "tests/comms/probe_arc293_W2c_compile_time_send.wat"
            :line 25
            :col 38
            :end #wat.core/Option.Some {
              :value #wat.core/Pos {
                :line 25
                :col 45
              }
            }
          }
          :causes []
          :head ":wat::program::self-peer"
          :reason "a comm carries only pure data — type :w2c::S is not pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. Redesign I/O as records, scalars, or pure enums (no Sender, Receiver, or handle fields)."
          :remedies []
        }
      ]
    }

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**13. Test-text kind: the reader requires the refusal to contain the keyword spelling of the head.**

```
        FAIL [   0.586s] ( 724/6423) wat::rete probe_arc278_inline_constraint_law_a::untyped_equality_constraint_is_refused
  stdout ───

    running 1 test
    test probe_arc278_inline_constraint_law_a::untyped_equality_constraint_is_refused ... FAILED

    failures:

    failures:
        probe_arc278_inline_constraint_law_a::untyped_equality_constraint_is_refused

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.57s

  stderr ───

    thread 'probe_arc278_inline_constraint_law_a::untyped_equality_constraint_is_refused' (2635371) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_inline_constraint_law_a.rs:93:5:
    the refusal must NAME the offending head ":wat::core::=" so the diagnostic teaches; got:
    #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/NonReteConstraint {:message \"defrule `probe::untyped-equality` (`:probe/Reading`): `wat.core/=` is not a rete primitive — a rule condition admits only :wat::rete:: ops. Use the per-type spelling, e.g. `:wat::rete::core::i64::wat.core/=`: the rete surface is per-type so the comparison is TOTAL (the generic form has no answer for operands that are not comparable)\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_inline_constraint_untyped_equality.wat\" :line 13 :col 39 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 13 :col 61}}} :causes [] :rule \"probe::untyped-equality\" :fact-type \"probe/Reading\" :head \"wat.core/=\" :twin \":wat::rete::core::i64::wat.core/=\"}]}" :location nil :causes [] :errors [#wat.rete/NonReteConstraint {:rule "probe::untyped-equality" :fact-type "probe/Reading" :head "wat.core/=" :twin ":wat::rete::core::i64::wat.core/=" :span #wat.core/Span {:file "tests/rete/probe_arc278_inline_constraint_untyped_equality.wat" :line 13 :col 39 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 13 :col 61}}}}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**14. Test-text kind: a reader requires two source spellings to differ.** 3 tests contain `head spelling was never swapped`. The file was already converted, so the two lines are the same symbol form.

```
        FAIL [   0.013s] (4270/6423) wat::resolve probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares
  stdout ───

    running 1 test
    test probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares ... FAILED

    failures:

    failures:
        probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 165 filtered out; finished in 0.00s

  stderr ───

    thread 'probe_arc251_stone9_symbol_head_declaration::a_symbol_headed_declaration_actually_declares' (2677101) panicked at /home/john/work/holon/wat-rs/tests/resolve/probe_arc251_stone9_symbol_head_declaration.rs:50:5:
    assertion `left != right` failed: declares: the pair's first lines are identical — the head spelling was never swapped
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**15. Test-text kind: a reader requires verbatim `::` in printed source.** 6 blocks contain the substring `verbatim`. This is the one that was read.

```
        FAIL [   0.743s] ( 610/6423) wat::rete probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon
  stdout ───

    running 1 test
    test probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon ... FAILED

    failures:

    failures:
        probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.73s

  stderr ───

    thread 'probe_arc278_ast_to_source::ast_to_source_is_verbatim_colon_colon' (2633810) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_ast_to_source.rs:39:5:
    ast->source must print raw `::` token text, not the `.`-dialed write-forms notation
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**16. Sift.** 3 tests contain `sift-`. The producer line was not isolated.

```
        FAIL [   0.879s] (4491/6423) wat::services probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors
  stdout ───

    running 1 test
    test probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors ... FAILED

    failures:

    failures:
        probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 156 filtered out; finished in 0.86s

  stderr ───

    thread 'probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors' (2679947) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc278_sift_logs.rs:20:5:
    expected sift-logs with a pure `level = :error` predicate to return exactly 1 survivor; got i64(-1)
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**17. The load gate is a witness, not a new mechanism.** `every_wat_scripts_file_loads_on_the_current_runtime` failed at 386.681s. Four of 782 files do not load: a `MalformedClause` on `(wat.rete.core/cond …)` in `wat-scripts/perf/grid/where-inline-computed.wat`; two pattern-head errors (row 5); and the retired `:wat::core::i64::to-string` (row 6). The `MalformedClause` deciding line was not isolated.

```
        FAIL [ 386.681s] (6423/6423) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
  stdout ───

    running 1 test
    test wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime has been running for over 60 seconds
    test wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime ... FAILED

    failures:

    failures:
        wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 377 filtered out; finished in 386.68s

  stderr ───

    thread 'wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime' (2619677) panicked at /home/john/work/holon/wat-rs/tests/lint/wat_scripts_fixes_load.rs:64:5:
    4 of 782 wat-scripts/ files do not load on the current runtime (rotted):
      wat-scripts/perf/grid/where-inline-computed.wat
          #wat.rete/ReteCheckErrors {:message "#wat.rete/ReteCheckErrors {:message \"1 rete rule validation error\" :location nil :causes [] :errors [#wat.rete/MalformedClause {:message \"defrule `wic::inline-cond` (`:wic/Req`): malformed rete clause `(wat.rete.core/cond ((wat.rete.i64/> :k 100) true) (:else false))` — not a recognized :when shape\" :location #wat.core/Span {:file \"wat-scripts/perf/grid/where-inline-computed.wat\" :line 147 :col 12 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 147 :col 77}}} :causes [] :rule \"wic::inline-cond\" :fact-type \"wic/Req\" :clause \"(wat.rete.core/cond ((wat.rete.i64/> :k 100) true) (:else false))\"}]}" :location nil :causes [] :errors [#wat.rete/MalformedClause {:rule "wic::inline-cond" :fact-type "wic/Req" :clause "(wat.rete.core/cond ((wat.rete.i64/> :k 100) true) (:else false))" :span #wat.core/Span {:file "wat-scripts/perf/grid/where-inline-computed.wat" :line 147 :col 12 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 147 :col 77}}}}]}
      wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat
          #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat" :line 19 :col 10 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 19 :col 27}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []} #wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat" :line 25 :col 10 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 25 :col 27}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []}]}
      wat-scripts/scratch-pad/probe-arc278-fnforms-walks-a-matches-pattern.wat
          #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::form::matches? form: pattern head must be a struct type keyword" :location #wat.core/Span {:file "wat-scripts/scratch-pad/probe-arc278-fnforms-walks-a-matches-pattern.wat" :line 43 :col 6 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 43 :col 17}}} :causes [] :head ":wat::form::matches?" :reason "pattern head must be a struct type keyword" :remedies []}]}
      wat-scripts/scratch-pad/probe-arc278-reap-serve-event.wat
          #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::i64::to-string form: ':wat::core::i64::to-string' is retired; use ':wat::i64::to-string' instead" :location #wat.core/Span {:file "wat-scripts/scratch-pad/probe-arc278-reap-serve-event.wat" :line 61 :col 46 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 61 :col 68}}} :causes [] :head ":wat::core::i64::to-string" :reason "':wat::core::i64::to-string' is retired; use ':wat::i64::to-string' instead" :remedies [#wat.kernel/Remedy {:form ":wat::i64::to-string" :kind :retirement :score 0 :note nil}]}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

────────────
```

**Controls, not a new site.** `a_real_enum_variant_in_a_rete_constraint_matches` prints `0` against `1`. `legitimate_keyword_constants_are_still_keywords` prints `1` against `3`. The wall was not widened to chase them. `resolve_operand_type` still treats a non-`?` symbol as unbound in this rule.

```
        FAIL [   0.839s] ( 642/6423) wat::rete probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches
  stdout ───

    running 1 test
    test probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches ... FAILED

    failures:

    failures:
        probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.83s

  stderr ───

    thread 'probe_arc278_enum_variant_typo::a_real_enum_variant_in_a_rete_constraint_matches' (2634174) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:41:5:
    assertion `left == right` failed: `:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the fixture drifted and the probe below proves nothing
    0

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
```
        FAIL [   0.921s] ( 643/6423) wat::rete probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords
  stdout ───

    running 1 test
    test probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords ... FAILED

    failures:

    failures:
        probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.92s

  stderr ───

    thread 'probe_arc278_enum_variant_typo::legitimate_keyword_constants_are_still_keywords' (2634181) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_enum_variant_typo.rs:164:5:
    assertion `left == right` failed: all three keyword-constant routes must still compile AND fire — a count below 3 means the new refusal, or a widening of it, ate a legitimate constant
    1

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`malformed match arm` is 3 tests. One block:

```
        FAIL [   0.953s] ( 761/6423) wat::rete probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires
  stdout ───

    running 1 test
    test probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires ... FAILED

    failures:

    failures:
        probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 526 filtered out; finished in 0.95s

  stderr ───

    thread 'probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires' (2635651) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_match_arm_is_not_a_call.rs:232:5:
    the refusal must name the TRUE Stone-C reason (form-level arm exhaustiveness vs. head-level fence totality), not a fabricated arity error
    [#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::rete::lower form: malformed match arm\" :location #wat.core/Span {:file \"tests/rete/probe_arc278_match_arm_body_ok.wat\" :line 23 :col 21 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 23 :col 53}}} :causes [] :head \":wat::rete::lower\" :reason \"malformed match arm\"}"}]

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### Fourteen blocks matched none of the needles

These fourteen first-blocks contain none of the needles in the table, so they were not filed on a row above. They stay uncollapsed, which keeps the site count above twelve. Five of them were read. `pure_cond_is_pure` panics at `tests/rete/probe_arc278_6a_purity.rs:82` with the message `pure cond`. `export_without_arm_refusal_names_the_wat_line` panics at `tests/rete/probe_arc278_export.rs:285` with `import-and-hits defn`. `nested_options_three_levels` and `sqlite_interop` both carry the nested-variant reason emitted at `src/runtime.rs:9006`, the arm taken when a vector pattern is not a keyword that `is_namespaced_variant` accepts (`src/runtime.rs:8996`). `construct_and_return_derives_via_oracle` carries the `eval-insert` miss at `src/rete/eval_insert.rs:254` for head `cg/make-rate`. The other nine were not read. `src/rete/purity.rs` was not claimed as their site.

```
wat::rete probe_arc278_55_slice_one_vocabulary::rete_fn_return_type_slot_is_not_classified_as_an_expression_deterministic
wat::rete probe_arc278_55_slice_one_vocabulary::rete_fn_return_type_slot_is_not_classified_as_an_expression_pure
wat::rete probe_arc278_6a_purity::pure_cond_is_pure
wat::rete probe_arc278_D6_constraint_omission::a_tagged_enum_operand_is_named_not_dropped
wat::rete probe_arc278_D6_constraint_omission::a_unit_enum_constraint_reaches_the_explain_payload
wat::rete probe_arc278_accessor_purity::record_accessor_is_pure
wat::rete probe_arc278_accessor_purity::record_accessor_is_deterministic
wat::rete probe_arc278_export::export_without_arm_refusal_names_the_wat_line
wat::rete probe_arc278_foreign_pred_purity::foreign_pred_is_deterministic
wat::rete probe_arc278_foreign_pred_purity::foreign_pred_is_pure
wat::rete probe_arc278_sqlite_interop::sqlite_interop
wat::rete probe_construction_headline::construct_and_return_derives_via_oracle
wat::rete probe_arc278_vsa_where_native_differential::differential_presence_four_row_native_eq_oracle
wat::function recursive_patterns::nested_options_three_levels
```

`src/rete/purity.rs` matches `WatAST::Keyword` at lines 1132, 1254, 1262, 1283, 1311, 1356, 1411, and 2097. That reader was not proven to be the deciding site of those fourteen.

### Clippy and the ignore ledger

Clippy `--release --all-targets -- -D warnings` was run before this section. The log is 7 lines, ends `Finished release profile [optimized] target(s) in 13.65s`, and prints no warning and no error.

`cargo nextest list --release --run-ignored only --ignore-default-filter` listed 19 names. The amend's expected figure is 18. No `#[ignore]` was added or removed to force 18. The floor's 24 skipped tests are these 19 plus the 5 default-filter skips.

```
wat edn::render::comments_survive_the_round_trip::eyeball_the_real_output
wat::kernel probe_arc259_started_at_boot::peer_started_at_is_after_started_at
wat::kernel probe_arc259_started_at_boot::started_at_is_the_primed_boot_not_the_seam
wat::kernel test::deftest_wat_tests_lint_lint_stdlib_runs
wat::macros probe_arc260_keyword_args::user_fn_keyword_args_reorder_to_positional
wat::reflection probe_arc255_ivb2b_verify_examples::verify_examples_reports_no_failures
wat::reflection probe_arc255_reflection_parity::user_form_carries_guaranteed_baseline
wat::services probe_arc278_self_scheduling::self_tick_fires_rearms_and_reactor_serves_process
wat::services probe_arc278_self_scheduling::self_tick_fires_rearms_and_reactor_serves_thread
wat::types probe_arc255_54_classes::collect_compared_types
wat::types probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused
wat::types probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_monomorphic_variant_is_accepted_in_annotation_position
wat::types probe_arc296_p2a_a_monomorphic_variant_is_a_type::an_enum_value_does_not_flow_into_a_variant_parameter
wat::types probe_arc296_p2a_a_monomorphic_variant_is_a_type::is_type_answers_true_for_a_variant
wat::types probe_diag_typealias_leniency::probe_undeclared_field_type_keyword_rejected_or_lenient
wat::value clj_expr_parity::wat_expr_matches_clj_oracle
wat::wat_lang probe_undefined_builtin_resolves::bogus_leaf_under_known_namespace_is_a_check_error
wat::wat_lang probe_undefined_builtin_resolves::wrong_operator_leaf_is_a_check_error
wat-reader::clj_oracle_source_parity source_reader_tracks_the_clj_oracle
```

`doc_link_ledger_matches_the_captured_log` is not in that list.

### STOP

Rows 1 through 15 are more than twelve deciding sites. The sift producer, the `MalformedClause` on `wat.rete.core/cond`, and the fourteen unfiled tests were not given a line, so they do not shrink the count. No corpus cure was started. The alpha door, the totality classifier, the `matches?` keyword gate, the retirement canonicalization, the vocabulary `Keyword` arm, the lowerer, the old goldens, the acronym law, sift, peers, and `normalize.rs` were not edited.

5c-iv and 5d were not started.
