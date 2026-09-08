# SCORE — STONE N RELAND 6: read the screams, and separate the fixtures that must stay wrong

No commit of the strike itself except the worklist the brief named. Floor **was** run.
Lands on DRAW N RELAND-6 + the committed scream list.

## 1. The scream list — committed

`docs/arc/2026/06/296-diagnostics-fully-edn/WORKLIST-STONE-N-RELAND-6-unresolved.txt`

Commit `2c45f5a5a WIP(296 N RELAND-6): the scream list the wrap printed`.

This is the wrap's stdout, not a grep of the corpus. Source: RELAND-5 pass-2 of
`positional-ctor-to-map.wat` over the 1875-path worklist, 8/8 `EXIT=0`, pass-2 sha256
delta 0. Every line is `<head> <file>:<line>`.

```
9494 screams · 1049 unique heads · 9340 unique sites
```

## 2. Classification — every scream is exactly one of (a)(b)(c)(d)

**(d) = 0.** No positional enum ctor with a visible top-level `defenum` that the repaired
tool failed to wrap. If the tool did not decline it, it took it. There is no class-(d)
hand-edit, and STOP-1 forbids inventing one.

**(b) is not in the scream list.** The two named negative fixtures are `.wat.bad`. The wrap
worklist is `*.wat`. They were never visited. That is correct: they exist to be refused.

**(c) = 0 screams.** Skip-path covers `tests/cli/grep_smoke_target.wat` and prefix
`wat-scripts/fixes/`. Confirmed: those paths do not appear in the worklist.

**(a)** is the rest of the 9494. The tool cannot see an enum declaration — usually because
there isn't one. Disposition is **not** "hand-edit everything Pascal." Hand-edit is authorised
only when the scream is a real positional **enum** ctor whose decl is hidden.

### (a) — broken down against current source

| kind | screams | hand-edit? | why the tool declined |
|---|---|---|---|
| operators (`=`, `+`, `-`, `*`, `<`, `>`, `->`, `->>`, …) | ~3835 | no | `pascal-leaf?` treats a non-letter as uppercase (`to-uppercase("=") == "="`). Not an enum. |
| records / kwargs / already-map | ~2883 | no | `fill-enum` only fills `TypeBody::Enum`. ThreadOpts, EchoRequest, PageState, Tally, already-wrapped `{:k v}`. |
| collections / holon / newtypes / rete facts / request records | ~2738+38 | no | `PersistentVector`, `Tuple`, `List`, `Bind`, `Thermometer`, `WriteLogsRequest`, `PutRequest`, `Hit`, `ColdAndWindy`, `Millisecond`, … — records, collections, rete patterns, request maps. |
| hidden `defenum` (defmacro / let / do body) | **3** | yes, **already struck in RELAND 5** | `file-decls` is top-level only |

The three class-(a) enum sites still **appear in the committed dump** (the dump is the
instrument, not a post-edited remainder). Current source is already map-form:

| site | scream | wrap (RELAND 5) | why declined |
|---|---|---|---|
| `tests/macros/probe_do_splice_enum_via_macro.wat:12` | `:my::probe::Event::Created` | `Created {:id 1}` | defenum lives in the macro quasiquote |
| `tests/macros/probe_let_splice_enum_via_macro.wat:12` | same | same | same, `let []` wrapper |
| `wat-scripts/scratch-pad/probe-sift-rules-stop1-dump.wat:48` | `:probe::Wrapped::EchoResponse::Ok` | `Ok {:c …}` | defenum is quasiquoted DATA inside `defmacro :probe::wrapped-surface` |

**No further class-(a) hand-edit this strike.** STOP-1: a site that is not a declined
positional enum ctor is not a hand-edit.

## 3. `wat/spawn.wat` — stdlib, in the worklist, **not skipped**

The wrap **processed** it (log line `[positional-ctor] wat/spawn.wat`). It screamed **16**
times (the brief guessed 14; the instrument says 16):

```
:wat::spawn::ThreadOpts     :100 :106 :111 :117
:wat::program::EmptyEnv     :101 :112 :118
:wat::spawn::ProcessOpts    :123 :131 :134 :142 :150 :451
:wat::kernel::Failure       :474
:wat::spawn::Launched       :540 :598
```

Every one is a **record kwargs ctor** (`:init-fn …`, `:post-spawn-fn …`, `:handle …`).
Class **(a)**: no enum declaration to see. Correctly not wrapped. Not class (d).

## 4. Class (b) — the two named fixtures. Stay wrong. Wall is right.

`.wat.bad`, not on the wrap worklist, **must not be migrated** (STOP-2).

Both still construct `(:my::Event::Pair 1 2)` as the scrutinee and hold a **bad pattern**.
`--check` now reports **3** errors, not 2:

1. **NEW, CORRECT:** `positional variant construction is retired` on `Pair` (the wall)
2. **INTENDED, STILL FIRES:** `map pattern has 1 key(s), variant Pair declares 2` /
   `map pattern has 0 key(s), variant Pair declares 2`
3. Fallout: non-exhaustive Pair

STOP-3 asked: is the new error the right error for what this test measures? The tests
measure **pattern** mismatch. That error still fires. The extra error is the wall correctly
refusing an illegal ctor that happens to sit in the same fixture. The wall is **not**
over-firing. Fixtures stay wrong. Goldens recaptured: `2 type-check errors` → `3`, with
the positional-ctor MalformedForm **prepended**; the pattern reasons are byte-identical.

- `tests/types/enums__tagged_variant_arity_mismatch_reported.edn`
- `tests/types/enums__unit_variant_pattern_on_tagged_variant_rejected.edn`

Those two tests **PASS** on the floor (`enums::tagged_variant_arity_mismatch_reported`,
`enums::unit_variant_pattern_on_tagged_variant_rejected`).

## Probe / clippy

- `probe_arc296_enum_map_ctor`: **5 passed** (floor + prior)
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings

## Floor

**Run.** Captured at `wat-rs/.floor/2026-09-08T11-52-04Z/` (`.floor/latest`). **Not re-run.**

```
Summary [ 188.380s] 5238 tests run: 5193 passed, 45 failed, 18 skipped
exit=100
```

RELAND-5 left 47. This strike recaptured the two class-(b) goldens. Floor is **45**.
The remaining 45 are **not** class-(d) enum-ctor misses (there are none). They are the
wake of records/requests/error-text/load that the scream list already declined as
non-enums. Untruncated ARM: `.floor/latest/ARM.txt` (6006 lines). Names:

```
wat::wat_lang::probe_arc241_stone15_zombie_purge::contract_01_try_hard_cut_rejected
wat::wat_lang::probe_arc241_stone15_zombie_purge::contract_02_try_rejection_remedy_names_result_try
wat::wat_lang::wat_core_try::try_on_non_result_arg_rejected_at_check
wat::wat_lang::wat_core_try::try_with_two_args_rejected_at_check
wat::wat_lang::wat_core_try::try_mismatched_err_types_rejected_at_check
wat::collection::probe_hashmap_ctor_vector_symmetric::probe_p6_wrong_value_type_rejected_at_type_check
wat::diagnostics::probe_diagnostic_c3_macro_emits_record_def::macro_output_reexpands_record_def_and_enum_wraps_it
wat::reflection::probe_stone_metadata_of_whole_row::from_metadata_of_the_lookup_equals_the_entry_doc
wat::lint::wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
wat::cli::wat_mcp::a_counter_increments_across_turns
wat::cli::wat_mcp::a_thread_counter_increments_across_turns
wat::rete::probe_arc278_smem_roundtrip::{smem_roundtrip,scan_index,scan_page1,scan_page2,scan_page3}
wat::rete::probe_arc278_sqlite_store_differential::{run_ops_on_mem_store,sqlite_store_differential}
wat::services::probe_arc272_rs1_state_must_be_record::durable_field_vector_mints_record_soul_round_trips
wat::services::probe_arc278_call_context::{ctx_namespace_is_the_service_fqdn,ctx_populated_caller_id_and_operation,internal_arm_receives_populated_self_invocation,second_public_arm_also_works,stability_gate_survivor_keeps_original_id_across_middle_eviction}
wat::services::probe_arc278_journal_backend_differential::journal_persists_identically_across_mem_and_sqlite_backends_on_a_thread
wat::services::probe_arc278_journal_logs_on_process::journal_writes_and_queries_logs_across_a_process_fork
wat::services::probe_arc278_journal_query::journal_query_metrics_reads_back_and_filters_by_time_window
wat::services::probe_arc278_journal_query_logs::journal_query_logs_reads_back_and_filters_by_time_window
wat::services::probe_arc278_journal_query_metrics_on_process::journal_query_metrics_across_a_process_fork
wat::services::probe_arc278_journal_service_logs::journal_writes_a_log_through_a_held_store_peer_on_a_thread
wat::services::probe_arc278_journal_service_on_process::journal_writes_a_metric_through_a_held_store_peer_on_a_process
wat::services::probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error
wat::services::probe_arc278_log_captures_call_line::log_captures_call_line
wat::services::probe_arc278_mem_store_on_process::mem_store_reserved_ns_service_round_trips_on_a_process_locus
wat::services::probe_arc278_sift_logs::{sift_logs_pure_predicate_returns_only_survivors,sift_logs_pure_predicate_returns_only_survivors_on_process}
wat::services::probe_arc278_sift_rules::{sift_rules_defsvc_counts_exact_deductions_on_thread,sift_rules_defsvc_counts_exact_deductions_on_process}
wat::services::probe_arc278_sift_rules_arena::{sift_rules_arena_counts_exact_deductions_paged_on_thread,sift_rules_arena_counts_exact_deductions_paged_on_process}
wat::services::probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
wat::services::probe_arc278_span_nested::nested_with_span_emits_into_each_namespace_independently
wat::services::probe_arc278_span_service::span_accumulates_a_counter_and_emits_it_as_a_metric_on_close
wat::services::probe_arc278_tagged_keys_store::{uuid_gsi_scan_index_round_trips,constant_width_inst_sk_sorts_chronologically}
```

Representative ARM (journal/store cluster — `disconnected` at a Put/Write site the wrap
screamed as a **record**, not an enum):

```
#wat.kernel/AssertionFailure {:thread "wat-thread-peer::<anon>" :message "disconnected"
 :location #wat.kernel/Location {:file "tests/rete/probe_arc278_smem_roundtrip.wat" :line 170 :col 5}
thread 'probe_arc278_smem_roundtrip::scan_index' panicked at src/freeze.rs:1050:51
```

`WriteLogsRequest` / `PutRequest` are in the scream list as class (a) non-enum. Wrapping
them with the **enum** map ctor would be a category error. If they need kwargs, that is
`positional-to-kwargs.wat`, not this tool. Not a class-(d) finding for 296 M.

`probe_p6_wrong_value_type_rejected_at_type_check` golden expected 1 type-check error,
got 4 — same class as RELAND-4 class 4 (error-text), not an enum-ctor miss.

Do not re-run this floor. The ARM is the evidence.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 hand-edit without appearing in the committed scream list | **held.** No new hand-edits. The 3 enum sites in the dump were already struck. |
| STOP-2 negative fixture migrated to make a test pass | **held.** `.wat.bad` untouched. Goldens recaptured. |
| STOP-3 expectation updated without asking if the new error is correct | **held.** Positional-ctor refusal is correct; pattern errors still fire. |
| STOP-4 remainder characterised by grep | **held.** Worklist is wrap stdout. Classification is against that list. |
| STOP-5 wall weakened | **held.** Positional control still positional; retired ctor still refused. |
