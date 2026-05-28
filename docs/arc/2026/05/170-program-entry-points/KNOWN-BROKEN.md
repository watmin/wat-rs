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

**Red tests — wat-telemetry-sqlite log daemon (added 2026-05-27, user-assigned to 170):**
The sqlite log sink is a daemon (auto-spawned `Service`, arc 089/095) living in the
spawn/Service/process layer this arc reworks. Arc 240's `.wat` drift sweep cleared
its check errors and uncovered two pre-existing runtime bugs (6 `reader` tests):
- `deftest_wat_telemetry_sqlite_reader_*` (6 tests) — `crates/wat-telemetry-sqlite/`
  - **Error 1 (blocker):** `src/cursor.rs` `decode_notag_holon` EDN-rejects
    `::`-namespaced keywords on log-row read-back ("keyword begins with ::";
    likely arc-230 keyword→Bind ripple). **Fix the decode path when correcting the
    daemon.** If 170's rework doesn't touch row-decode, re-home to arc 219b (#445,
    wat-edn EDN conformance).
  - **Error 2 (downstream):** `wat-tests/telemetry/reader.wat:235,270` call
    `:wat::test::assertion-failed` (only `:wat::kernel::assertion-failed!` exists).
- Full detail: `docs/arc/2026/05/240-runtime-rot-remediation/FINDING-sqlite-reader-bugs.md`.

Cross-ref: `docs/arc/2026/05/240-runtime-rot-remediation/DESIGN.md` (root causes G + E + the sqlite-daemon finding / DEFER set).
