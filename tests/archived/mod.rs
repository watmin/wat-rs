//! tests/archived/ — the archived test corpus (arc 252 test-surface reorg).
//!
//! These are SPENT design-probes: FM-2-bis disconfirming probes from CLOSED arcs
//! whose result is now a permanent property of the substrate. They did their job
//! at the arc — they proved a composition worked (or didn't) and steered the build.
//! We keep them RUNNABLE rather than delete them: if they still compile and pass,
//! they are zero-cost regression coverage, and they are the historical record of
//! how the substrate was proven. "If they work and were useful, don't lose them."
//!
//! NAMING: archived tests KEEP their arc-numbered names on purpose — the archive
//! IS the record of that arc, so the arc number is the right identifier here. The
//! behavior-renaming (`tests/<home>/<behavior>.rs`) applies only to LIVE tests.
//!
//! This is ONE leak-safe `[[test]]` binary (Cargo: `name="archived"`). Only PURE
//! (non-process) probes live here so `cargo test --test archived` never leaks;
//! living self-enforcing gates and process probes stay elsewhere.
//!
//! The module list below is GENERATED — do not hand-edit it. Add a `.rs` file to
//! this dir, run `scripts/gen-test-mods.sh`, and it is declared automatically; the
//! `--check` gate fails loud if the list drifts, so no file can be silently lost.
//!
//! Run: `cargo test --release -p wat --test archived`

// BEGIN GENERATED MODS (scripts/gen-test-mods.sh) — do not hand-edit below
mod probe_arc214_slice4_stone1_program_env_typealias;
mod probe_arc214_slice4_stone2_env_get_trio;
mod probe_arc214_slice4_stone3_env_dig_trio;
mod probe_arc215_collection_literal_inference;
mod probe_arc215_stone2;
mod probe_arc216_stone1_hashset_roundtrip;
mod probe_arc216_stone2_vector_roundtrip;
mod probe_arc216_stone3_hashmap_roundtrip;
mod probe_arc216_stone4_predicate_composition;
mod probe_arc216_stone5a_value_hash;
mod probe_arc216_stone5b_hashset_native_storage;
mod probe_arc216_stone5c_hashmap_native_storage;
mod probe_arc216_stone6_process_collection_roundtrip;
mod probe_arc216_stone7_tuple_roundtrip;
mod probe_arc226_stone1_type_predicates;
mod probe_arc227_stone2_defrecord;
mod probe_arc234_stone15_namespace_promotion;
mod probe_arc234_stone1_wat_record_variant;
mod probe_arc234_stone2a_record_primitives;
mod probe_arc234_stone2b_defrecord_macro;
mod probe_arc234_stone2c_accessor_class_safety;
mod probe_arc234_stone3a_record_read_verbs;
mod probe_arc234_stone3b_record_assoc;
mod probe_arc234_stone3c_fix_narrow_fallthrough;
mod probe_arc234_stone3c_keyword_accessor;
mod probe_arc234_stone4_hash_destructure;
mod probe_arc234_stone4_match_hash_destructure;
mod probe_arc234_stone5_holon_auto_dispatch;
mod probe_arc236_stone0_check_result;
mod probe_arc237_7a_length_intrinsic;
mod probe_arc237_7c_assoc_polymorphic;
mod probe_arc237_8a_no_implicit_coercion;
mod probe_arc237_8b_defclause_arithmetic;
mod probe_arc237_8c_equality_grid;
mod probe_arc237_8d_equality_intrinsic;
mod probe_arc237_sA1_assignable;
mod probe_arc237_sA_hierarchy;
mod probe_arc237_sB1_recordtype;
mod probe_arc237_sB2_defrecord_recordtype;
mod probe_arc237_sC2ab_field_order;
mod probe_arc237_sC2c_base_record;
mod probe_arc237_sC2d_same_data;
mod probe_arc237_sC3_macro_split;
mod probe_arc237_stone1_typeunion_substrate;
mod probe_arc237_stone2_defclause_substrate;
mod probe_arc237_stone3_guard_ensure;
mod probe_arc237_stone4_rich_errors;
mod probe_arc237_stone5_conforms;
mod probe_arc237_stone5fix_nominal;
mod probe_arc237_stone6_is_predicate;
mod probe_arc238_eq_completeness;
mod probe_arc241_stone10_remedy;
mod probe_arc241_stone11_define_hard_cut;
mod probe_arc241_stone12_defalias;
mod probe_arc241_stone13_define_dispatch_hard_cut;
mod probe_arc241_stone14_restricted_absorbed;
mod probe_arc241_stone15_zombie_purge;
mod probe_arc241_stone16_define_eval_residue;
mod probe_arc241_stone17_defmacro_canonical;
mod probe_arc241_stone1_argspec_canonical;
mod probe_arc241_stone2_fn_parser_migration;
mod probe_arc241_stone3_defclause_parser_migration;
mod probe_arc241_stone5_defclause_rest_dispatch;
mod probe_arc241_stone6_def_metadata_map;
mod probe_arc241_stone7_metadata_of_reflection;
mod probe_arc241_stone8_defstruct;
mod probe_arc241_stone9_defenum;
mod probe_arc242_stone1_lexeme_role;
mod probe_arc242_stone2_value_position_doctrine;
mod probe_arc243_stone3_1_checkenv_borrow;
mod probe_arc243_stone3_typeerror_pattern_a;
mod probe_arc243_stone5_register_subtype_span;
mod probe_arc243_stone6_checkerror_pattern_a;
mod probe_arc243_stone7b_signal_split;
mod probe_arc243_stone7c_runtimeerror_pattern_a;
mod probe_arc247_hof_fn_first;
mod probe_arc249_4_rehome_in_wat;
mod probe_arc249_macro_engine;
mod probe_arc249_threading;
mod probe_arc249_threading_in_wat;
mod probe_brace_map_literal;
mod probe_channel_primitive;
mod probe_collection_transform_ops;
mod probe_diagnostic_bundle_result_compose;
mod probe_diagnostic_defprotocol_dispatch;
mod probe_diagnostic_dynamic_keyword_invocation;
mod probe_diagnostic_macro_splice_from_let;
mod probe_diagnostic_polymorphic_type;
mod probe_diagnostic_typed_entities_reflection;
mod probe_diagnostic_value_snapshot_in_errors;
mod probe_do_splice_def;
mod probe_do_splice_define;
mod probe_do_splice_enum;
mod probe_do_splice_struct;
mod probe_eval_signature_returns_tracked_value;
mod probe_hashmap_ctor_vector_symmetric;
mod probe_let_splice_def;
mod probe_let_splice_define;
mod probe_let_splice_enum;
mod probe_let_splice_struct;
mod probe_nil_return_value_position_bug;
mod probe_panic_hook_auto_installed;
mod probe_register_types_splice_aware;
mod probe_resolver_quote_awareness;
mod probe_stone_233_2_e_ast_derived_provenance;
mod probe_stone_233_2_j_producer_migration;
mod probe_stone_233_2_l_wat_value_seal;
mod probe_stone_233_3_runtime_error_edn;
mod probe_substrate_symmetry_list_span_threading;
mod probe_tracked_value_mint_contract;
mod probe_verify_hashset_of_vector_gap;
// END GENERATED MODS
