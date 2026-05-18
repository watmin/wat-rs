# EXPECTATIONS — Arc 207 Slice 4

## Mode prediction

**Mode A — clean ripple ships (~65%).** Sonnet rewrites USER-GUIDE § 11, flips 3 wat-tests files' field types + replaces constant-string ids with `Uuid/v4` mints at appropriate setup points, all tests still pass, workspace baseline preserved. ~65-90 min wall-clock.

**Mode B — one wat-test needs setup reshape (~20%).** The current wat-tests pass server-id + user-id as constants in places where minting at runtime would require restructuring the setup code (e.g., a top-level `define` that's now a `let` inside the test body). Sonnet surfaces, orchestrator approves shape, sonnet completes. Adds ~15-20 min.

**Mode C — grep surfaces consumer outside target files (~10%).** Could be:
- An example wat program in `examples/` referencing the namespace verbs
- A doc snippet somewhere referencing arc 206 verbs
- A wat-test in another crate using the typed Uuid in an unexpected way

Sonnet surfaces; orchestrator decides extend-scope or defer to follow-up. Adds ~10-20 min depending on shape.

**Mode D — EDN wire roundtrip mismatch in process-tier test (~5%).** Process-tier counter-service-process-N3 serializes Wire enum variants over stdio. If typed Uuid serialization on the wire differs from what the receiver expects (slice 2 should have handled this via edn_shim arms), test fails. This would be a slice 2 latent bug; sonnet surfaces, orchestrator decides fix-here vs new-slice.

**Mode E-time-violation — anything past 105 min.** Surface; orchestrator decides.

## Expected file changes

| File | Change scope |
|---|---|
| `docs/USER-GUIDE.md` § 11 | ~50 lines rewritten (subsection bounded between lines 2479-2533) |
| `wat-tests/counter-service-capability-N3.wat` | Field type flips (struct + Wire variants); ~3-6 constant-string ids → Uuid/v4 mints; ~10-15 line-diff |
| `wat-tests/counter-service-process-N3.wat` | Mirror of capability-N3 (same shape) |
| `wat-tests/counter-client-capability-proof.wat` | Smaller — single-user proof; ~5-10 line-diff |
| `docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-4.md` | NEW |

Expected diff: ~+150 (USER-GUIDE rewrite) and ~-50 (constants removed; mints + type-keywords slightly more verbose but cleaner). Net positive lines mostly from USER-GUIDE.

## Workspace baseline expected

Pre-existing 3-4 failures unchanged (lifeline flaky may toggle):
- `lifeline_pipe_zero_orphans_across_100_trials`
- `deftest_wat_tests_tmp_totally_bogus`
- `t6_spawn_process_factory_with_capture_round_trips`
- `startup_error_bubbles_up_as_exit_3`

The 3 arc 203 demos in slice 4's scope are tracked separately from the 4-failure baseline — they currently PASS at the typed level (because slice 3 retired the namespace verbs they previously called, BUT slice 3 only retired SUBSTRATE registrations, not the wat-tests that used to call them; those wat-tests may already be failing post-slice-3). Sonnet's verification gate baseline check will reveal whether arc 203 demos currently pass or fail post-slice-3.

**Honest correction:** if arc 203 demos already broke at slice 3 (because they call retired verbs), they're already in the failure set. Slice 4 fixes them. If they were never calling the substrate verbs directly (only the telemetry alias which retargets cleanly), they may still pass. Sonnet verifies first.

## EDN wire-format check (process tier)

Process tier (`counter-service-process-N3.wat`) serializes Wire enum over stdio. After slice 4:
- Wire::Admin payload `server-id` is `:Uuid`; serializes as `#uuid "..."` per slice 2's `value_to_edn_with` arm
- Receiver's `(:wat::edn::read ...)` reads typed Uuid value per slice 2's `edn_to_value` fix
- Pattern match on Wire variants destructures typed Uuid; passes to `Process/println` which writes via EDN

If slice 2's edn_shim arms are correct (and they were verified by the 2 EDN roundtrip test cases in `wat_arc207_uuid_typed.rs`), this works transparently. If slice 4 hits a wire-format mismatch, it points at a slice 2 gap — surface as Mode D.

## Out-of-scope findings (surface, don't act)

- Lab-side consumers (out of arc 207 scope; lab reconstruction is dependent unblock)
- `holon-rs` crate (separate workspace)
- Any new typed-Uuid consumer pattern (e.g., HashMap<:Uuid, T>) that surfaces beyond what arc 203 demos use — slice 3 already added the `hashmap_key` arm in-scope, but other consumer patterns (e.g., Uuid-as-comparison-key for sorting) are NOT in slice 4

## Failure-mode catches

- FM 1 (grep before claiming): verification gate IS the grep audit
- FM 9 (load-bearing tests verified): arc 203 demos passing IS the load-bearing evidence
- FM 11 (deferral language): N/A this slice (no INSCRIPTION yet — slice 5)
- FM 14 (surface retirement leaving internal identifiers): related; slice 3 retired substrate verbs but didn't ripple demo files; slice 4 closes that gap
- FM 16 (no tool preamble): BRIEF doesn't preamble Bash/cargo
- `feedback_no_known_defect_left_unfixed`: if grep surfaces a defect sonnet can fix in <5 min without bloating slice, fix; otherwise surface

## Atomic commit shape

NO commit by sonnet. Orchestrator commits all touched files + SCORE atomically when sonnet returns.

Expected commit: 4-5 files touched (3 wat-tests + USER-GUIDE + SCORE). ~250-400 line diff total (USER-GUIDE rewrite is most).

## Calibration record

- Slice 1 (audit): 36 min
- Slice 2 (substantive): 93 min
- Slice 3 (mechanical retirement): ~30 min (sonnet's report didn't time-stamp but the work was bounded)
- Slice 4 (consumer ripple): predicted 60-90 min — between slice 2's substantive and slice 3's mechanical because the USER-GUIDE rewrite has design surface (what to teach about typed Uuid) while the wat-tests changes are mechanical

Sonnet: trust the typed surface from slice 2; trust slice 3's retirement; ripple cleanly; surface honest deltas; return.
