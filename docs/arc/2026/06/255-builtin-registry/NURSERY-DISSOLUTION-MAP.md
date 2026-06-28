# `tests/nursery/` dissolution map — partire verdict (2026-06-27)

> **Source:** a `partire` cast (background agent `a3815708b1a63b028`, ~456s, weighed against the disk).
> **Verdict: SPLIT (Level 1).** `nursery` is 15 independent domains fused under a non-domain name
> ("dumping ground" — builder's own words). **Zero new groups; zero unclassifiable.** All 179 test
> files re-home into existing sibling groups. The `nursery` binary retires.
>
> **This is a MAP, not yet executed.** Part (B) of the test-infra annihilation campaign (see the
> breadcrumb `CURRENT-STATE.md`). Each file moves into its target `tests/<group>/` dir; `build.rs`
> auto-globs `tests/<group>/*.rs` and regenerates each `mod.rs`, so a move is the whole wiring — no
> manual mod-line edits. Co-located `.wat` fixture migration (campaign part A) rides the same motion
> per file.

## Mechanics (proven by the agent against the disk)
- nursery is wired by `Cargo.toml:123-125` (`[[test]] name="nursery" path="tests/nursery/mod.rs"`)
  and `build.rs` auto-glob. Moving a `.rs` into `tests/<group>/` auto-wires it into that group's binary.
- After the 179 files move: `tests/nursery/` holds only `mod.rs` → delete `mod.rs`, remove the
  `Cargo.toml:123-125` `[[test]]` entry, `rmdir tests/nursery/`. The `nursery` binary retires; all 179
  tests survive, each now recompiling only with its true domain (narrower blast radius).
- Sanity: 180 dir entries − `mod.rs` = 179 files; counts reconcile to 179, no orphans.

## The decomposition (179 files → 15 existing homes)

| Home | Files | Reason-to-change (the seam) |
|---|---|---|
| **types** | 31 | the record/type/enum/struct/protocol/generic type system |
| **collection** | 17 | map/vector/set/tuple containers + holon bundle algebra |
| **macros** | 16 | defmacro/splice/threading-macro/hygiene expansion |
| **kernel** | 15 | VM/stdlib internals, spawn-program', bracket pools, deftest harness |
| **diagnostics** | 15 | structured EDN error output (Check/Type/RuntimeError), pretty-print |
| **comms** | 13 | IPC peer verbs `send'/recv'/select'/poll'`, frames, listeners |
| **function** | 13 | fn/defn/defclause/argspec dispatch + polymorphic intrinsics |
| **wat_lang** | 12 | core special forms — define/defalias cuts, destructuring, assert |
| **process** | 10 | OS process lifecycle — spawn-process, stdio, fork, shutdown, crash |
| **reflection** | 10 | metadata-of / signature / examples-seam reflection surface |
| **value** | 9 | low-level `Value` reprs — wat__Record, Hash/Eq, TrackedValue, seal |
| **program** | 7 | program-env record / user.program slot / per-peer env install |
| **channel** | 6 | the channel substrate — make-channel, payload, connection primitive |
| **services** | 4 | defservice / std-io-service record + handle design |
| **resolve** | 1 | the arc251 source-producer migration family |
| **Total** | **179** | |

### types — 31
`probe_arc214_lexer_primed_generic_head`, `probe_arc214_stone46i_typed_peer`, `probe_arc226_stone1_type_predicates`, `probe_arc227_stone2_defrecord`, `probe_arc234_stone15_namespace_promotion`, `probe_arc234_stone2a_record_primitives`, `probe_arc234_stone2b_defrecord_macro`, `probe_arc234_stone2c_accessor_class_safety`, `probe_arc234_stone3a_record_read_verbs`, `probe_arc234_stone3b_record_assoc`, `probe_arc234_stone3c_fix_narrow_fallthrough`, `probe_arc234_stone3c_keyword_accessor`, `probe_arc234_stone5_holon_auto_dispatch`, `probe_arc237_8a_no_implicit_coercion`, `probe_arc237_8c_equality_grid`, `probe_arc237_8d_equality_intrinsic`, `probe_arc237_sA1_assignable`, `probe_arc237_sA_hierarchy`, `probe_arc237_sB1_recordtype`, `probe_arc237_sB2_defrecord_recordtype`, `probe_arc237_sC2ab_field_order`, `probe_arc237_sC2d_same_data`, `probe_arc237_sC3_macro_split`, `probe_arc237_stone1_typeunion_substrate`, `probe_arc237_stone5_conforms`, `probe_arc237_stone5fix_nominal`, `probe_arc237_stone6_is_predicate`, `probe_arc241_stone8_defstruct`, `probe_arc241_stone9_defenum`, `probe_diagnostic_defprotocol_dispatch`, `probe_diag_typealias_leniency`

### collection — 17
`probe_arc215_collection_literal_inference`, `probe_arc215_stone2`, `probe_arc216_stone1_hashset_roundtrip`, `probe_arc216_stone2_vector_roundtrip`, `probe_arc216_stone3_hashmap_roundtrip`, `probe_arc216_stone4_predicate_composition`, `probe_arc216_stone5b_hashset_native_storage`, `probe_arc216_stone5c_hashmap_native_storage`, `probe_arc216_stone6_process_collection_roundtrip`, `probe_arc216_stone7_tuple_roundtrip`, `probe_arc257_native_map_set`, `probe_brace_map_literal`, `probe_collection_transform_ops`, `probe_hashmap_ctor_vector_symmetric`, `probe_nth`, `probe_verify_hashset_of_vector_gap`, `probe_diagnostic_bundle_result_compose`

### macros — 16
`probe_arc241_stone17_defmacro_canonical`, `probe_arc249_4_rehome_in_wat`, `probe_arc249_macro_engine`, `probe_arc249_threading_in_wat`, `probe_arc249_threading`, `probe_diagnostic_macro_splice_from_let`, `probe_do_splice_define`, `probe_do_splice_def`, `probe_do_splice_enum`, `probe_do_splice_struct`, `probe_let_splice_define`, `probe_let_splice_def`, `probe_let_splice_enum`, `probe_let_splice_struct`, `probe_register_types_splice_aware`, `probe_resolver_quote_awareness`

### kernel — 15
`bootstrap_wat_vm_process`, `probe_arc214_stone82w_quarry_dead`, `probe_arc259_bracket_runner`, `probe_arc259_brackets_each`, `probe_arc259_brackets_map`, `probe_arc259_brackets_worker`, `probe_arc259_deftest_hermetic_prime`, `probe_arc259_deftest_prime`, `probe_arc259_s2cii_a_applyloop_purged`, `probe_arc259_s2cii_b_defclause`, `probe_arc259_s2ci_spawn_thread_prime`, `probe_arc259_s2d_internal_only`, `probe_arc259_s2d_raii_hinge`, `probe_build_rs_autodiscovery`, `probe_time_duration_readout`

### diagnostics — 15
`probe_arc236_stone0_check_result`, `probe_arc237_stone4_rich_errors`, `probe_arc241_stone10_remedy`, `probe_arc242_stone2_value_position_doctrine`, `probe_arc243_stone3_1_checkenv_borrow`, `probe_arc243_stone3_typeerror_pattern_a`, `probe_arc243_stone5_register_subtype_span`, `probe_arc243_stone6_checkerror_pattern_a`, `probe_arc243_stone7b_signal_split`, `probe_arc243_stone7c_runtimeerror_pattern_a`, `probe_arc255_epprintln`, `probe_arc255_pprintln`, `probe_diagnostic_value_snapshot_in_errors`, `probe_stone_233_3_runtime_error_edn`, `probe_substrate_symmetry_list_span_threading`

### comms — 13
`probe_arc209_c0b1b_select_listener`, `probe_arc209_c0b1_thread_connection`, `probe_arc209_c0b2b_socket_peer`, `probe_arc209_c0b_uds_abstract_spike`, `probe_arc209_structured_peer_death`, `probe_arc214_stone46aii_peer_verbs`, `probe_arc214_stone46b_select_prime`, `probe_arc258_recv_infers_from_consumer`, `probe_arc259_comms_recv_multiline_frame`, `probe_arc259_s2a_thread_self_peer`, `probe_edn_value_framing`, `probe_ioreader_read_frame`, `probe_readln_max_buffer_kwarg`

### function — 13
`probe_arc237_7a_length_intrinsic`, `probe_arc237_7b_intrinsic_typing`, `probe_arc237_7c_assoc_polymorphic`, `probe_arc237_8b_defclause_arithmetic`, `probe_arc237_stone2_defclause_substrate`, `probe_arc237_stone3_guard_ensure`, `probe_arc241_stone1_argspec_canonical`, `probe_arc241_stone2_fn_parser_migration`, `probe_arc241_stone3_defclause_parser_migration`, `probe_arc241_stone5_defclause_rest_dispatch`, `probe_arc247_hof_fn_first`, `probe_arc259_s2cii0_record_dispatch`, `probe_diagnostic_dynamic_keyword_invocation`

### wat_lang — 12
`probe_arc234_stone4_hash_destructure`, `probe_arc234_stone4_match_hash_destructure`, `probe_arc257_keys_destructure`, `probe_arc241_stone11_define_hard_cut`, `probe_arc241_stone12_defalias`, `probe_arc241_stone13_define_dispatch_hard_cut`, `probe_arc241_stone14_restricted_absorbed`, `probe_arc241_stone15_zombie_purge`, `probe_arc241_stone16_define_eval_residue`, `probe_assert_true_false`, `probe_nil_return_value_position_bug`, `probe_undefined_builtin_resolves`

### process — 10
`probe_arc209_structured_peer_death_process`, `probe_arc214_stone63_fork_dead`, `probe_arc254_process_ownership`, `probe_arc259_thread_crash_reason`, `spawn_process_parent_type`, `spawn_process_stdin`, `spawn_process_stdio`, `shutdown_cascade_memory`, `shutdown_cascade_pipefd`, `probe_panic_hook_auto_installed`

### reflection — 10
`probe_arc241_stone6_def_metadata_map`, `probe_arc241_stone7_metadata_of_reflection`, `probe_arc255_ivb1_structured_doc`, `probe_arc255_ivb2a_examples_seam`, `probe_arc255_ivb2b_verify_examples`, `probe_arc255_ivc_metadata_plain_values`, `probe_arc255_reflection_parity`, `probe_arc255_spec_complete`, `probe_diagnostic_polymorphic_type`, `probe_diagnostic_typed_entities_reflection`

### value — 9
`probe_arc216_stone5a_value_hash`, `probe_arc234_stone1_wat_record_variant`, `probe_arc237_sC2c_base_record`, `probe_arc238_eq_completeness`, `probe_arc242_stone1_lexeme_role`, `probe_eval_signature_returns_tracked_value`, `probe_stone_233_2_e_ast_derived_provenance`, `probe_stone_233_2_l_wat_value_seal`, `probe_tracked_value_mint_contract`

### program — 7
`probe_arc259_cpu_count`, `probe_arc259_env_identity`, `probe_arc259_env_peer_kind`, `probe_arc259_peer_env_install`, `probe_arc259_program_cpu_count`, `probe_arc259_program_init_fn`, `probe_arc259_user_program_slot`

### channel — 6
`probe_arc209_connection_primitive`, `probe_arc214_stone51_channel_substrate_flip`, `probe_arc214_stone61_typed_channel_dead`, `probe_arc254_channel_payload_portable`, `probe_arc254_make_channel`, `probe_channel_primitive`

### services — 4
`gate_arc214_service_record_field_order`, `probe_arc214_stone81b_stderr_no_handle_passing`, `probe_arc214_stone81_stdout_no_handle_passing`, `probe_arc214_stone82_stdin_no_handle_passing`

### resolve — 1
`probe_stone_233_2_j_producer_migration`

## Refused cuts (accidental seams — do NOT re-propose as groups)
- **`probe_diagnostic_*` / `gate_*` / `probe_*` prefixes are NOT domains** — probe-naming conventions.
  The seven `probe_diagnostic_*` scatter across collection/types/function/macros/reflection/diagnostics
  by actual content (e.g. `probe_diagnostic_bundle_result_compose` → collection, holon Bundle algebra).
- **arc-number and `stoneNN` are NOT domains** — arc241 alone splits across function + types + macros +
  reflection + wat_lang + diagnostics (a cross-cutting "define unification" campaign). Grouping by arc
  re-creates a junk drawer with a different label.

## Practitioner's-call splits (graph admits two readings — decided once, recorded here)
- `probe_arc214_stone61_typed_channel_dead` → **channel**; `_stone82w_quarry_dead` → **kernel** (placed by
  which substrate's *removal* each asserts: typed-channel vs thread_io).
- stdio `*_no_handle_passing` trio → **services** (assert `Std{In,Out,Err}Service` shape), not process.
- `probe_arc259_s2cii0_record_dispatch` → **function** (defclause dispatch), not types.
- `probe_arc259_peer_env_install` → **program** (env record), not services.
- `probe_arc216_stone5a_value_hash` → **value** (`Value` Hash/Eq), not collection.
- arc237 `7a/7b/7c/8b` intrinsics → **function** (polymorphic dispatch), not types.

## Pairs
`CURRENT-STATE.md` (the campaign breadcrumb — test-infra annihilation, part B) ·
`feedback_test_wat_is_colocated_fixture` (part A: the co-located `.wat` fixture scheme rides each move) ·
`/partire/SKILL.md` (the spell that derived this).
