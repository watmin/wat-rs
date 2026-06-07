//! tests/nursery/ — the living test commons (arc 252 test-surface reorg).
//!
//! NOT an archive. This is where tests are RAISED and PROVEN, then GRADUATE to
//! their permanent `src/<home>` test dir (`tests/comms/`, `tests/value/`, …) when
//! a home is warranted. It is the default home for a test that belongs to no single
//! src-home yet, and the place iterative test-development accumulates.
//!
//! The contract (builder, 2026-06-06):
//!   - Accumulate HONESTLY-NAMED good tests here as you go — every test real,
//!     every assertion meaningful (no sentinels).
//!   - Arc/stone/probe-numbered names (`probe_arc216_stone1_hashset_roundtrip`) are
//!     VALID — they are real, they pass, they are useful. Improve a name when
//!     warranted (especially on graduation to a home), not for its own sake.
//!   - PROMOTE a test OUT to `tests/<home>/` (with a behavior name) when it clearly
//!     belongs to one src-home. The nursery raises; the homes keep.
//!
//! One leak-safe `[[test]]` binary (Cargo: `name="nursery"`). Keep only PURE
//! (non-process) tests here so `cargo test --test nursery` never leaks; process
//! tests live in their home group (`tests/comms/`), leak-`#[ignore]`'d as needed.
//!
//! The module list below is GENERATED — do not hand-edit it. Add a `.rs` file here,
//! run `scripts/gen-test-mods.sh`, and it is declared automatically; the `--check`
//! gate (green-gate 1/4) fails loud if the list drifts, so no test is ever lost.
//!
//! Run: `cargo test --release -p wat --test nursery`

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
mod probe_arc237_7b_intrinsic_typing;
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
mod probe_arc254_channel_payload_portable;
mod probe_arc254_process_ownership;
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
