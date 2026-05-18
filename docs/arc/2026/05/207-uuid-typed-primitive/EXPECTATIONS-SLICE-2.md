# EXPECTATIONS — Arc 207 Slice 2

## Mode prediction

**Mode A — clean ship of all 20+1 items + 8 tests pass + workspace preserved (~65%).** Sonnet works through the checklist in suggested order; each item lands without surprise; tests pass; workspace baseline preserved. ~70-90 min wall-clock.

**Mode B — one or two items need adjustment (~25%).** Most likely:
- `values_equal` arm placement requires reordering nearby arms (low risk)
- `edn_shim` `value_to_edn_with` arm has a specific positional constraint (e.g., must be in a particular match block) that the SCORE didn't surface
- `Uuid/from-string` canonical-strict check edge case (e.g., what about all-zero nil-uuid? — that IS canonical; should be `Some(nil)`. Test should cover.)

Sonnet surfaces, orchestrator approves the adjustment, sonnet completes. Adds ~15-30 min.

**Mode C — surface drift since slice 1 audit (~7%).** A file:line ref drifted because some unrelated commit landed between slice 1 audit and slice 2 spawn. Sonnet re-audits the specific line, surfaces what's actually there, proceeds.

**Mode D-time-violation — anything past 120 min.** Surface; orchestrator decides kill vs let-finish.

## Workspace baseline expected

Pre-existing 3-4 failures (must persist; slice 2 introduces none):
- `lifeline_pipe_zero_orphans_across_100_trials` (flaky; may pass)
- `deftest_wat_tests_tmp_totally_bogus` (intentional canary)
- `t6_spawn_process_factory_with_capture_round_trips` (Stone D2 known delta)
- `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)

Acceptable post-slice-2: same set or any subset (lifeline may toggle). Unacceptable: any NEW failure.

## Item-by-item expectations

Per SCORE-SLICE-1 checklist:

- Items 1, 2, 3 (runtime.rs Value variant + type_name + values_equal): trivial additions; arm placement is the main concern
- Items 4, 5 (no values_compare, no hashmap_key): intentionally NOT added; SCORE row should affirm "not added — correct per slice 1 audit"
- Items 6, 7 (edn_shim read + write): 2-line read fix + ~3-line write arm; both straightforward given Value variant exists
- Items 8–12 (5 check.rs schemes): each ~3 lines; mechanical
- Items 13–17 (5 eval handlers): each ~10-15 lines; uniform pattern; `eval_uuid_v5` removes the existing panic foot-gun
- Item 18 (dispatch wiring): grep + add 5-6 arms
- Item 19 (no types.rs): nothing to do; SCORE row affirms
- Item 20 (test file): ~8 test cases, ~80-120 lines total

## Test coverage check

The 8 test cases per item 20:
1. `Uuid/v4` returns `:wat::core::Uuid` not `:String`
2. `Uuid/v5` with typed ns + name returns `:Uuid`
3. `Uuid/from-string` valid canonical → `Some(uuid)`; invalid → `None`
4. `Uuid/to-string` roundtrips `Uuid/v4` → canonical 36-char
5. `Uuid/nil` returns nil-uuid; `to-string` produces `"00000000-..."`
6. Equality: 2 `Uuid/v4` differ; `Uuid/v5` same args equal
7. Cross-type inequality (String UUID vs Uuid value) — checks check-time rejection (likely a compile-time test or a runtime type assertion)
8. `(= u1 u2)` works via new `values_equal` arm

Plus orchestrator suggestion: add EDN roundtrip cases (write then read; assert structural equality) covering item 7 (write arm). 2-3 more cases. Total ~10 cases.

## Failure-mode catches

- FM 1 (proposing options without grep): every claim in SCORE-2 should cite file:line for evidence
- FM 9 (load-bearing tests verified): the 8+ test cases ARE the load-bearing evidence for the new surface
- FM 11 (deferral language): N/A this slice (no INSCRIPTION ships)
- FM 16 (no tool preamble): BRIEF doesn't preamble Bash/cargo
- FM 17 (discipline-after-pushback): sonnet should fire FM checks before action

## Atomic commit shape

NO commit by sonnet. Orchestrator independently verifies SCORE + commits all touched files atomically when sonnet returns.

Expected commit: ~7-8 files touched (1 runtime.rs, 1 edn_shim.rs, 1 check.rs, 1 string_ops.rs or new uuid_ops.rs, 1 new test file, 1 SCORE doc). ~250-400 lines diff total.

## Calibration record

Slice 1 (audit) ran 36 min — well under the 60 min cap. Slice 2 is the substantive substrate work; expected to use most of the 90-min predicted upper-bound. Honest range: 60-120 min.

Sonnet: trust SCORE-SLICE-1 as ground truth; trust the disk; ship the 20 items + edn write arm + 8 tests; return summary with file:line for every code addition.
