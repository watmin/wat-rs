# 296 Quarantine — IGNORE LEDGER — RETIRED 2026-08-16 (Stone K, row 7)

> **STATUS: RETIRED IN PLACE.** Not deleted — this is the record of a 241-test
> quarantine and its drain, six waves, **115 → 0**. The GOLDEN population this
> ledger tracks (the 240-row table below + the 1 already-fixed lint row) is
> **measured 0** today: zero rows still assert the pre-stone-B rust-debug face,
> zero `#[ignore = "296-recapture-pending"]` in `tests/`. What `#[ignore]` means
> from here on — and why a *remaining* `#[ignore]` is no longer a quarantine
> signal — is answered by
> `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-K-ignore-means-one-thing.md`,
> this arc's closure stone.
>
> **⚠ ONE HONEST EXCEPTION, measured, not glossed over.** The GATE as originally
> written (below, preserved unedited) also demanded zero
> `(:wat::test::ignore "296-recapture-pending")` occurrences in `wat-tests/`.
> That is **not** true today: `wat-tests/lint.wat:72` still carries
> `(:wat::test::ignore "296-recapture-pending: lint-stdlib times out (>5s) after
> stone B; unlock: 296 recapture or perf fix")` on
> `deftest_wat_tests_lint_lint_stdlib_runs` — the side-car TIMEOUT entry
> documented below, never actually unlocked. Stone K did not touch it: fixing
> `lint-stdlib`'s performance (or giving this one a real exclusion mechanism,
> the same move 3 pattern Stone K applied to the Rust-side boundary probe) is
> out of Stone K's scope (it relocates/re-declares `#[ignore]`'s two ON-DEMAND
> kinds; this is neither — it is exactly the debt kind, just wat-native instead
> of Rust-native, and undercounted by any grep that only checks `.rs` files).
> Recorded here so the next hand does not read "gate satisfied" as "everything
> in this file resolved."

**Purpose**: Tracked mute of 241 pre-existing failing tests established by the
arc 296 stone B quarantine commit. These tests are NOT silenced as bugs — they
are *recapture-pending*: stone B flipped the diagnostic surface from Rust
`{:?}` debug strings to structured EDN (the `:wat.core/Span` record + per-error
tagged maps), and these goldens still assert the pre-stone-B rust-debug face.

**Clean baseline**: after this ledger's ignores are applied,
`cargo nextest run` reports **0 failed** (only skipped + passed).

**Gate** (original wording, preserved): this ledger MUST be EMPTY before arc
296 closes. Each row below is a debt entry that requires a recapture strike to
retire. **See the retirement note above for the one row that is not actually
empty.**

---

## Unlock

Recapture each test via `assert_edn_matches_file!` + `UPDATE_EDN` (the
pretty-printing EDN update path introduced by stone B). Update the golden
fixture to match the new structured EDN surface, un-ignore the test (remove
its `#[ignore]` attribute), run it green, remove this ledger row.

The GATE: zero rows in this table + zero `#[ignore = "296-recapture-pending"]`
occurrences in `tests/` + zero `(:wat::test::ignore "296-recapture-pending")`
in `wat-tests/` before arc 296 can close.

---

## Side-car entries (non-golden)

Two additional ignores added in the same commit with distinct reasons:

| test | file | reason | unlock |
|------|------|--------|--------|
| `wat::function tco::self_recursion_via_if_at_million_depth` | `tests/function/tco.rs` | TIMEOUT (>30s) — million-depth TCO benchmark; not a golden mismatch | Re-evaluate when TCO performance permits; remove `#[ignore = "boundary-slow"]` |
| `wat::kernel test::deftest_wat_tests_lint_lint_stdlib_runs` | `wat-tests/lint.wat` | TIMEOUT (>5s internal limit) — `(:wat::lint::lint-stdlib)` is too slow post-stone-B | Fix lint-stdlib performance or skip via separate mechanism; remove `(:wat::test::ignore "296-recapture-pending")` |

Lint fix also applied in the same commit: `src/check.rs` line ~20963 — the
`rune:lint(loose-assert)` exemption comment ended with `;`, causing the lint
scanner to reset `stmt_has_rune` before reaching the `assert!` call.
Fixed by rewording the comment to not end with `;`. The lint test
`no_loose_string_assert::tests_carry_no_loose_string_assert` now PASSES.

---

## Recapture-pending test list (240 entries)

| binary :: file :: test | #[ignore] file |
|------------------------|----------------|
| `wat::comms probe_arc293_W2a_struct_no_cross::struct_rejected_at_wire_SEND` | `tests/comms/probe_arc293_W2a_struct_no_cross.rs` |
| `wat::diagnostics probe_arc237_stone4_rich_errors::probe_03_no_matching_clause_edn_tag_clean` | `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` |
| `wat::diagnostics probe_arc237_stone4_rich_errors::probe_04_postcondition_failed_edn_tag_clean` | `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` |
| `wat::diagnostics probe_arc237_stone4_rich_errors::probe_08_postcondition_edn_carries_ensure_and_returned` | `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` |
| `wat::diagnostics probe_arc237_stone4_rich_errors::probe_09_no_matching_clause_edn_round_trips` | `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` |
| `wat::diagnostics probe_arc237_stone4_rich_errors::probe_10_attempt_list_count_preserved_through_edn` | `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_01_typo_remedy_on_variant_constructor` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_02_retirement_remedy_for_hard_cut_form` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_03_ranked_multi_candidate_variant_typo` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_05_single_remedy_single_line_format` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_06_multi_remedy_multi_line_format` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc241_stone10_remedy::contract_07_retirement_kind_annotation_canonical` | `tests/diagnostics/probe_arc241_stone10_remedy.rs` |
| `wat::diagnostics probe_arc242_stone2_value_position_doctrine::contract_03_keyword_type_in_body_rejected_with_remedy` | `tests/diagnostics/probe_arc242_stone2_value_position_doctrine.rs` |
| `wat::diagnostics probe_arc243_stone6_checkerror_pattern_a::checkerror_display_elides_unknown_secondary_span` | `tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs` |
| `wat::diagnostics probe_arc243_stone6_checkerror_pattern_a::checkerror_display_elides_unknown_span` | `tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs` |
| `wat::diagnostics probe_arc243_stone6_checkerror_pattern_a::edn_elides_unknown_span` | `tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs` |
| `wat::diagnostics probe_arc243_stone7c_runtimeerror_pattern_a::runtimeerror_freeze_pair_elides_unknown_span` | `tests/diagnostics/probe_arc243_stone7c_runtimeerror_pattern_a.rs` |
| `wat::diagnostics probe_arc296_2_to_edn_trait::probe_2_span_to_edn_is_structured_map` | `tests/diagnostics/probe_arc296_2_to_edn_trait.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_alias_arity_mismatch_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_any_banned_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_cyclic_alias_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_cyclic_subtype_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_cyclic_union_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_duplicate_type_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_empty_union_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_impure_field_in_pure_aggregate_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_impure_variant_field_in_pure_enum_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_inner_colon_in_compound_arg_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_invalid_union_member_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_malformed_decl_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_malformed_field_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_malformed_name_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_malformed_type_expr_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_malformed_variant_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_reserved_prefix_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3a_typeerror_derive_identical::probe_single_member_union_known_span` | `tests/diagnostics/probe_arc296_3a_typeerror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_cycle_detected_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_duplicate_load_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_fetch_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_malformed_load_form_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_parse_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_setter_in_loaded_file_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3b_loaderror_derive_identical::probe_verification_failed_known_span` | `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` |
| `wat::diagnostics probe_arc296_3_holdout_edn::probe_1_parse_startup_error_to_edn_is_structured_not_detail` | `tests/diagnostics/probe_arc296_3_holdout_edn.rs` |
| `wat::diagnostics probe_arc296_3_holdout_edn::probe_4_check_startup_error_emits_structured_vector_not_detail` | `tests/diagnostics/probe_arc296_3_holdout_edn.rs` |
| `wat::diagnostics probe_arc296_4_check_error_to_edn::arity_mismatch_to_edn_is_byte_identical` | `tests/diagnostics/probe_arc296_4_check_error_to_edn.rs` |
| `wat::diagnostics probe_arc296_4_check_error_to_edn::comm_call_out_of_position_to_edn_is_byte_identical` | `tests/diagnostics/probe_arc296_4_check_error_to_edn.rs` |
| `wat::diagnostics probe_arc296_4_check_error_to_edn::type_mismatch_to_edn_is_byte_identical` | `tests/diagnostics/probe_arc296_4_check_error_to_edn.rs` |
| `wat::diagnostics probe_arc296_4_check_error_to_edn::unknown_callee_to_edn_is_byte_identical` | `tests/diagnostics/probe_arc296_4_check_error_to_edn.rs` |
| `wat::diagnostics probe_arc296_d1_structured_not_prose::probe_1_return_type_mismatch_remedies_field_is_vector_not_prose` | `tests/diagnostics/probe_arc296_d1_structured_not_prose.rs` |
| `wat::diagnostics probe_arc296_d1_structured_not_prose::probe_3_no_matching_clause_at_call_site_is_structured` | `tests/diagnostics/probe_arc296_d1_structured_not_prose.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_bad_arity_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_bad_type_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_bad_value_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_duplicate_field_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_malformed_setter_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_required_field_missing_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_setter_after_non_setter_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_derive_configerror_identical::probe_unknown_setter_known_span` | `tests/diagnostics/probe_arc296_derive_configerror_identical.rs` |
| `wat::diagnostics probe_arc296_macro_error_is_structured_edn::probe_1_startup_error_to_edn_is_tagged` | `tests/diagnostics/probe_arc296_macro_error_is_structured_edn.rs` |
| `wat::diagnostics probe_arc296_macro_error_is_structured_edn::probe_2_macro_error_to_edn_leaf_cause_is_not_string` | `tests/diagnostics/probe_arc296_macro_error_is_structured_edn.rs` |
| `wat::diagnostics probe_arc296_n3_per_phase_namespaces::error_families_tag_under_their_phase_namespace` | `tests/diagnostics/probe_arc296_n3_per_phase_namespaces.rs` |
| `wat::diagnostics probe_arc296_raise_gate::raise_bare_integer_is_compile_error` | `tests/diagnostics/probe_arc296_raise_gate.rs` |
| `wat::diagnostics probe_arc296_remediation_collapse::probe_1_type_mismatch_retired_callee_emits_remedies_not_hint` | `tests/diagnostics/probe_arc296_remediation_collapse.rs` |
| `wat::diagnostics probe_arc296_remediation_collapse::probe_2_type_mismatch_arc114_shape_emits_spawn_thread_remedy_not_hint` | `tests/diagnostics/probe_arc296_remediation_collapse.rs` |
| `wat::diagnostics probe_arc296_remediation_collapse::probe_3_return_type_mismatch_retired_callee_emits_remedies_not_hint` | `tests/diagnostics/probe_arc296_remediation_collapse.rs` |
| `wat::diagnostics probe_arc296_typed_causes::s1_macro_expansion_failed_carries_typed_cause_not_reason_string` | `tests/diagnostics/probe_arc296_typed_causes.rs` |
| `wat::diagnostics probe_arc296_typed_causes::s1_macro_expansion_failed_fixpoint_site_carries_depth_exceeded_cause` | `tests/diagnostics/probe_arc296_typed_causes.rs` |
| `wat::diagnostics probe_arc296_typed_causes::s2_runtime_error_wire_edn_is_structured_not_prose` | `tests/diagnostics/probe_arc296_typed_causes.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_arity_mismatch` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_arity_too_few` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_duplicate_macro` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_expansion_depth_exceeded` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_macro_eval_runtime_failed` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_malformed_defmacro` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_malformed_template` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_program_body_eval_failed` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_program_body_introduces_name` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_refused_in_macro` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_reserved_prefix` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_splice_not_sequence` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_macro_derive_identical::probe_unbound_macro_param` | `tests/diagnostics/probe_arc298_3_macro_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_arity_mismatch` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_assertion_failed_both_some` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_assertion_failed_expected_none` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_bad_condition` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_channel_disconnected` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_declaration_in_expression_position` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_division_by_zero` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_duplicate_define` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_edn_coerce_mismatch` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_edn_coerce_mismatch_empty_path` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_effectful_in_step` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_eval_forbids_mutation_form` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_eval_verification_failed` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_macro_abort` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_macro_expansion_failed` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_malformed_form` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_no_encoding_ctx` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_no_macro_registry` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_no_matching_clause` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_no_source_loader` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_no_step_rule` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_not_callable` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_param_shadows_builtin` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_pattern_match_failed` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_postcondition_failed` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_reserved_prefix` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_sandbox_scope_leak` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_service_not_running` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_type_mismatch` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_unbound_symbol` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_unknown_field` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_unknown_function` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_arc298_3_runtime_derive_identical::probe_user_main_missing` | `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_1_not_callable_renders_offending_keyword` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_2_not_callable_renders_runtime_built_keyword` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_3_type_mismatch_renders_non_keyword_head` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_4_type_mismatch_renders_non_vector_spread` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_6_runtime_built_keyword_renders_producer_info` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_7_from_holon_produces_tagged_value` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_8_edn_read_produces_tagged_value` | `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` |
| `wat::diagnostics probe_stone_233_3_runtime_error_edn::probe_1_not_callable_serializes_to_tagged_edn` | `tests/diagnostics/probe_stone_233_3_runtime_error_edn.rs` |
| `wat::diagnostics probe_stone_233_3_runtime_error_edn::probe_2_type_mismatch_carries_all_struct_fields` | `tests/diagnostics/probe_stone_233_3_runtime_error_edn.rs` |
| `wat::diagnostics probe_stone_233_3_runtime_error_edn::probe_3_assertion_failed_with_optional_fields` | `tests/diagnostics/probe_stone_233_3_runtime_error_edn.rs` |
| `wat::diagnostics probe_stone_233_3_runtime_error_edn::probe_4_tuple_variant_serializes` | `tests/diagnostics/probe_stone_233_3_runtime_error_edn.rs` |
| `wat::diagnostics probe_stone_233_3_runtime_error_edn::probe_5_provenance_variants_render_with_tags` | `tests/diagnostics/probe_stone_233_3_runtime_error_edn.rs` |
| `wat::function defn::defn_body_type_mismatch_surfaces` | `tests/function/defn.rs` |
| `wat::function fn_rename::bare_fn_type_post_retirement_walker_silent` | `tests/function/fn_rename.rs` |
| `wat::function fn_rename::both_legacy_walkers_retired_silently_alias` | `tests/function/fn_rename.rs` |
| `wat::function fn_rename::lambda_post_retirement_silently_aliases_to_fn` | `tests/function/fn_rename.rs` |
| `wat::function fn_rename::multiple_lambda_sites_post_retirement_silently_alias` | `tests/function/fn_rename.rs` |
| `wat::function fn_signature::fn_body_type_mismatch_surfaces` | `tests/function/fn_signature.rs` |
| `wat::function fn_signature::malformed_args_vector_clear_error` | `tests/function/fn_signature.rs` |
| `wat::function recursive_patterns::nonexhaustive_partial_pattern_rejected` | `tests/function/recursive_patterns.rs` |
| `wat::kernel probe_arc259_deftest_prime::deftest_prime_failing_raises_with_message` | `tests/kernel/probe_arc259_deftest_prime.rs` |
| `wat::kernel test::deftest_wat_tests_lint_lint_stdlib_runs` | `wat-tests/lint.wat` (via `(:wat::test::ignore ...)`) |
| `wat::kernel wat_arc198_def_restricted::defn_metadata_restricted_enforces_for_caller_outside_whitelist` | `tests/kernel/wat_arc198_def_restricted.rs` |
| `wat::kernel wat_arc198_def_restricted::def_restricted_caller_outside_allowed_namespace_fails` | `tests/kernel/wat_arc198_def_restricted.rs` |
| `wat::kernel wat_arc198_def_restricted::def_restricted_exact_fqdn_match_only_allows_named_caller` | `tests/kernel/wat_arc198_def_restricted.rs` |
| `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert` | FIXED (not muted) — rune comment in `src/check.rs` repaired |
| `wat::macros probe_arc209_macro_param_type_enforced::lying_macro_param_type_is_rejected_at_macro_def` | `tests/macros/probe_arc209_macro_param_type_enforced.rs` |
| `wat::macros probe_arc249_threading::witness_thread_first_empty_step_panics_at_expansion` | `tests/macros/probe_arc249_threading.rs` |
| `wat::macros probe_arc249_threading::witness_thread_last_empty_step_desugars_to_call_on_acc` | `tests/macros/probe_arc249_threading.rs` |
| `wat::macros probe_arc258_stone2b_macro_error::contract_02_non_exhaustive_cond_names_else` | `tests/macros/probe_arc258_stone2b_macro_error.rs` |
| `wat::macros probe_arc258_stone2b_macro_error::contract_03_macro_error_surfaces_its_message` | `tests/macros/probe_arc258_stone2b_macro_error.rs` |
| `wat::macros probe_arc279_format::format_strict_missing_kwarg_is_macro_error` | `tests/macros/probe_arc279_format.rs` |
| `wat::macros probe_arc279_format::format_strict_unused_kwarg_is_macro_error` | `tests/macros/probe_arc279_format.rs` |
| `wat::process probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes` | `tests/process/probe_supervisor_select_lost.rs` |
| `wat::process wat_arc202_process_join_holds_stdin::process_join_without_stdin_extraction_fails_check` | `tests/process/wat_arc202_process_join_holds_stdin.rs` |
| `wat::process wat_arc202_process_join_holds_stdin::process_join_with_stdin_present_does_not_fire_stdin_rule` | `tests/process/wat_arc202_process_join_holds_stdin.rs` |
| `wat::process wat_arc208_process_io_result::arc208_t6_walker_rejects_process_println_in_body_position` | `tests/process/wat_arc208_process_io_result.rs` |
| `wat::process wat_arc208_process_io_result::arc208_t7_walker_rejects_process_readln_in_body_position` | `tests/process/wat_arc208_process_io_result.rs` |
| `wat::program wat_arc170_program_contracts::t8b_fork_program_ast_callsite_fires_walker` | `tests/program/wat_arc170_program_contracts.rs` |
| `wat::program wat_arc170_program_contracts::t8_fork_program_callsite_fires_walker` | `tests/program/wat_arc170_program_contracts.rs` |
| `wat::program wat_arc170_program_contracts::t9b_spawn_program_ast_callsite_fires_walker` | `tests/program/wat_arc170_program_contracts.rs` |
| `wat::program wat_arc170_program_contracts::t9_spawn_program_callsite_fires_walker` | `tests/program/wat_arc170_program_contracts.rs` |
| `wat::program wat_arc170_slice_1e_user_main_nil::t2_arc170_slice_2_main_fires_walker` | `tests/program/wat_arc170_slice_1e_user_main_nil.rs` |
| `wat::reflection wat_arc201_extract_arg_types::extract_arg_types_errors_on_non_bundle_input` | `tests/reflection/wat_arc201_extract_arg_types.rs` |
| `wat::reflection wat_arc201_holon_ast_accessors::bundle_children_errors_on_atom_input` | `tests/reflection/wat_arc201_holon_ast_accessors.rs` |
| `wat::reflection wat_arc201_holon_ast_accessors::bundle_first_errors_on_empty_bundle` | `tests/reflection/wat_arc201_holon_ast_accessors.rs` |
| `wat::reflection wat_arc201_holon_ast_accessors::bundle_first_errors_on_leaf_input` | `tests/reflection/wat_arc201_holon_ast_accessors.rs` |
| `wat::reflection wat_arc201_signature_of_fn::signature_of_fn_errors_on_non_fn_input` | `tests/reflection/wat_arc201_signature_of_fn.rs` |
| `wat::services probe_arc209_c0b3bb_verbs::thread_listener_allow_errors_with_tier_message` | `tests/services/probe_arc209_c0b3bb_verbs.rs` |
| `wat::services probe_arc209_c0b3bc_post_spawn::accessor_typechecks_at_parse_time` | `tests/services/probe_arc209_c0b3bc_post_spawn.rs` |
| `wat::types enums::cross_enum_variant_pattern_rejected` | `tests/types/enums.rs` |
| `wat::types enums::missing_variant_arm_reports_non_exhaustive` | `tests/types/enums.rs` |
| `wat::types enums::tagged_variant_arity_mismatch_reported` | `tests/types/enums.rs` |
| `wat::types enums::unit_variant_pattern_on_tagged_variant_rejected` | `tests/types/enums.rs` |
| `wat::types newtype::distinct_newtypes_over_same_inner_are_distinct_types` | `tests/types/newtype.rs` |
| `wat::types newtype::newtype_rejected_where_inner_expected` | `tests/types/newtype.rs` |
| `wat::types newtype::newtype_rejects_inner_type_at_arg_position` | `tests/types/newtype.rs` |
| `wat::types probe_arc214_lexer_primed_generic_head::primed_two_param_with_space_fails_same_as_unprimed` | `tests/types/probe_arc214_lexer_primed_generic_head.rs` |
| `wat::types probe_arc227_stone2_defrecord::probe_constructor_rejects_wrong_typed_field` | `tests/types/probe_arc227_stone2_defrecord.rs` |
| `wat::types probe_arc227_stone2_defrecord::probe_defrecord_constructor_typed_rejects_wrong_type` | `tests/types/probe_arc227_stone2_defrecord.rs` |
| `wat::types probe_arc227_stone2_defrecord::probe_defrecord_field_type_check_bool_rejected` | `tests/types/probe_arc227_stone2_defrecord.rs` |
| `wat::types probe_arc227_stone2_defrecord::probe_two_arg_form_only_one_arg_errors` | `tests/types/probe_arc227_stone2_defrecord.rs` |
| `wat::types probe_arc234_stone3c_fix_narrow_fallthrough::probe_1_concrete_receiver_fails_at_check_time` | `tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.rs` |
| `wat::types probe_arc258_stone1_if_inference::contract_03_branch_mismatch_rejected_for_the_right_reason` | `tests/types/probe_arc258_stone1_if_inference.rs` |
| `wat::types probe_arc293_holder_bound::core_record_rejected_by_holon_holder_bound` | `tests/types/probe_arc293_holder_bound.rs` |
| `wat::types probe_arc293_holder_substitution::core_record_rejected_where_holon_wanted` | `tests/types/probe_arc293_holder_substitution.rs` |
| `wat::types probe_arc293_W2b_enum_purity::pure_enum_with_struct_field_rejected` | `tests/types/probe_arc293_W2b_enum_purity.rs` |
| `wat::types probe_arc293_W_containment::a_record_cannot_declare_a_struct_field` | `tests/types/probe_arc293_W_containment.rs` |
| `wat::types probe_diagnostic_defprotocol_dispatch::probe_3_missing_impl_raises_observable_error` | `tests/types/probe_diagnostic_defprotocol_dispatch.rs` |
| `wat::types struct_destructure::empty_brace_form_is_clean_malformed_form` | `tests/types/struct_destructure.rs` |
| `wat::types struct_destructure::non_struct_subject_is_clean_type_mismatch` | `tests/types/struct_destructure.rs` |
| `wat::types struct_destructure::non_symbol_inside_brace_form_is_clean_malformed_form` | `tests/types/struct_destructure.rs` |
| `wat::types struct_destructure::unknown_field_name_is_clean_malformed_form` | `tests/types/struct_destructure.rs` |
| `wat::types struct_restricted::struct_restricted_ctor_restriction_fires_on_illegal_caller` | `tests/types/struct_restricted.rs` |
| `wat::types struct_restricted::struct_restricted_empty_sections_honored` | `tests/types/struct_restricted.rs` |
| `wat::types struct_restricted::struct_restricted_malformed_shapes_rejected` | `tests/types/struct_restricted.rs` |
| `wat::types struct_restricted::struct_restricted_per_field_restriction_fires_on_illegal_caller` | `tests/types/struct_restricted.rs` |
| `wat::types tuple::legacy_tuple_lowercase_redirects_via_pattern2_poison` | `tests/types/tuple.rs` |
| `wat::types wat_arc148_ord_buildout::enum_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::types wat_arc148_ord_buildout::hashmap_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::types wat_arc148_ord_buildout::hashset_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::types wat_arc148_ord_buildout::holon_ast_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::types wat_arc148_ord_buildout::struct_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::types wat_arc148_ord_buildout::unit_ord_raises_type_mismatch` | `tests/types/wat_arc148_ord_buildout.rs` |
| `wat::value probe_arc242_stone1_lexeme_role::contract_03_legacy_char_hard_cut_with_remedy` | `tests/value/probe_arc242_stone1_lexeme_role.rs` |
| `wat::value wat_arc220_char::char_literal_supplementary_plane_rejected` | `tests/value/wat_arc220_char.rs` |
| `wat::wat_lang probe_arc234_stone4_hash_destructure::probe_5_unknown_field_errors` | `tests/wat_lang/probe_arc234_stone4_hash_destructure.rs` |
| `wat::wat_lang probe_arc241_stone11_define_hard_cut::contract_03_retirement_remedy_names_defn` | `tests/wat_lang/probe_arc241_stone11_define_hard_cut.rs` |
| `wat::wat_lang probe_arc241_stone11_define_hard_cut::contract_04_retirement_kind_annotation_present` | `tests/wat_lang/probe_arc241_stone11_define_hard_cut.rs` |
| `wat::wat_lang probe_arc241_stone11_define_hard_cut::contract_05_retirement_table_includes_define_entry` | `tests/wat_lang/probe_arc241_stone11_define_hard_cut.rs` |
| `wat::wat_lang probe_arc241_stone12_defalias::contract_05_rejection_remedy_names_defalias` | `tests/wat_lang/probe_arc241_stone12_defalias.rs` |
| `wat::wat_lang probe_arc241_stone13_define_dispatch_hard_cut::contract_02_rejection_remedy_names_defclause` | `tests/wat_lang/probe_arc241_stone13_define_dispatch_hard_cut.rs` |
| `wat::wat_lang probe_arc241_stone14_restricted_absorbed::contract_06_rejection_remedies_name_replacements` | `tests/wat_lang/probe_arc241_stone14_restricted_absorbed.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_01_try_hard_cut_rejected` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_02_try_rejection_remedy_names_result_try` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_03_option_expect_lowercase_hard_cut_rejected` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_04_option_expect_lowercase_rejection_remedy_names_pascal` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_05_result_expect_lowercase_hard_cut_rejected` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone15_zombie_purge::contract_06_result_expect_lowercase_rejection_remedy_names_pascal` | `tests/wat_lang/probe_arc241_stone15_zombie_purge.rs` |
| `wat::wat_lang probe_arc241_stone16_define_eval_residue::contract_01_define_rejection_carries_stone_241_16_marker` | `tests/wat_lang/probe_arc241_stone16_define_eval_residue.rs` |
| `wat::wat_lang probe_arc241_stone16_define_eval_residue::contract_02_retirement_remedy_preserves_defn_replacement` | `tests/wat_lang/probe_arc241_stone16_define_eval_residue.rs` |
| `wat::wat_lang probe_arc257_keys_destructure::probe_3_bare_symbol_brace_form_rejected` | `tests/wat_lang/probe_arc257_keys_destructure.rs` |
| `wat::wat_lang probe_def_not_special::probe_define_rejected_at_startup_check` | `tests/wat_lang/probe_def_not_special.rs` |
| `wat::wat_lang wat_arc072_letstar_parametric::whitespace_inside_angle_brackets_raises_clean_lex_error` | `tests/wat_lang/wat_arc072_letstar_parametric.rs` |
| `wat::wat_lang wat_arc136_do_form::do_empty_form_is_malformed` | `tests/wat_lang/wat_arc136_do_form.rs` |
| `wat::wat_lang wat_arc136_do_form::do_recipient_mismatch_fires_type_mismatch` | `tests/wat_lang/wat_arc136_do_form.rs` |
| `wat::wat_lang wat_arc143_define_alias::define_alias_retired_form_rejected_at_startup` | `tests/wat_lang/wat_arc143_define_alias.rs` |
| `wat::wat_lang wat_arc153_nil_rename::bare_legacy_unit_name_walker_retired` | `tests/wat_lang/wat_arc153_nil_rename.rs` |
| `wat::wat_lang wat_arc153_nil_rename::reverse_mixed_nil_body_with_retired_unit_sig_post_retirement` | `tests/wat_lang/wat_arc153_nil_rename.rs` |
| `wat::wat_lang wat_arc153_nil_rename::type_position_unit_post_retirement_is_unknown_fqdn` | `tests/wat_lang/wat_arc153_nil_rename.rs` |
| `wat::wat_lang wat_arc153_nil_rename::value_position_nil_against_i64_recipient_fires_type_mismatch` | `tests/wat_lang/wat_arc153_nil_rename.rs` |
| `wat::wat_lang wat_arc154_kill_let_star::let_body_type_mismatch_surfaces` | `tests/wat_lang/wat_arc154_kill_let_star.rs` |
| `wat::wat_lang wat_arc154_kill_let_star::let_star_post_retirement_silently_aliases_to_let` | `tests/wat_lang/wat_arc154_kill_let_star.rs` |
| `wat::wat_lang wat_arc154_kill_let_star::multiple_let_star_sites_post_retirement_silently_alias` | `tests/wat_lang/wat_arc154_kill_let_star.rs` |
| `wat::wat_lang wat_arc157_def::def_redef_default_flag_off_strict_default` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc157_def::def_redef_forbidden_strict_default` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc157_def::def_redef_set_redef_false_strict_default` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc157_def::def_redef_set_redef_true_type_change_fires` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc157_def::def_type_error_in_expr` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc157_def::def_type_mismatch_via_registered_type` | `tests/wat_lang/wat_arc157_def.rs` |
| `wat::wat_lang wat_arc168_let_flat_shape::multi_form_let_body_typecheck` | `tests/wat_lang/wat_arc168_let_flat_shape.rs` |
| `wat::wat_lang wat_arc168_let_flat_shape::odd_count_vector_errors` | `tests/wat_lang/wat_arc168_let_flat_shape.rs` |
| `wat::wat_lang wat_idempotent_redeclare::define_divergent_body_errors` | `tests/wat_lang/wat_idempotent_redeclare.rs` |
| `wat::wat_lang wat_idempotent_redeclare::typealias_divergent_errors` | `tests/wat_lang/wat_idempotent_redeclare.rs` |
| `wat::wat_lang wat_not_eq::not_eq_f64_cross_numeric_coerce` | `tests/wat_lang/wat_not_eq.rs` |
