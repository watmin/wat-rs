# KNOWN-BROKEN tests (surfaced by arc 240, 2026-05-27)

Arc 239 made the workspace test-build compile; the full
`cargo test --workspace` surfaced runtime failures hidden behind the old
`--lib`-only metric. Several exercise the **stdio-trio + spawn/fork/exit
machinery this arc is actively reshaping** (#296 StdIn/Out/Err services,
#309 wat-cli Stone B, #310 spawn.rs deletion). Per user direction 2026-05-27 —
*"G will be addressed when we unwind to it"* + the in-flight-dependency rule —
arc 240 did NOT touch these; they close as arc 170 winds.

**Red tests — ambient-stdio (thread-context / stdio-trio):**
- `deftest_wat_rs_test_test_ambient_stdio_readln_echo` (15s time-limit leak)
- `deftest_wat_rs_test_test_ambient_stdio_println_string`
- `deftest_wat_tests_std_test_test_assert_stderr_matches_fail_reports_pattern`
  (`wat-tests/kernel/services/ambient-stdio.wat`, `wat-tests/test.wat`)

**Red tests — wat-cli fork/exit (re-confirm after arc 240.3 clears the
`WorkUnitLog.wat` startup cascade — these are the residual genuinely-170 ones):**
- `sigterm_to_cli_cascades_via_polling_contract` (`src/fork.rs:167` child exit)
- `check_mode_exits_zero_on_good_program` (exit 1 vs 0)
- `missing_user_main_rejected` (exit 4 vs 3 — exit-code semantics)
- possibly others in `crates/wat-cli/tests/wat_cli.rs` once the cascade clears

**NOTE:** Many `wat_cli` failures in the 2026-05-27 run were *cascade* from
arc 240's root-cause-A (`WorkUnitLog.wat` bundled into CLI startup → 3 extra
diagnostics → "expected 2 records, got 5", exit-code drift). Those clear when
arc 240.3 ships. Re-run `cargo test -p wat-cli` after 240.3 to isolate the
true arc-170 residual before closing this list.

Cross-ref: `docs/arc/2026/05/240-runtime-rot-remediation/DESIGN.md` (root causes G + E / DEFER set).
