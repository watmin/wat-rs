# Arc 170 slice 1f-λ Phase A — SCORE (canonical pattern landed)

**Result:** Mode A clean. 17/17 tests pass in `wat_arc170_program_contracts.rs` on first compile. Phase A's primary deliverable — a canonical pattern reference for Phase B — is grep-able in the file.
**Runtime:** ~15 min opus.
**Files:** 2 modified — `tests/wat_arc104_fork_program.rs` deleted; `tests/wat_arc170_program_contracts.rs` extended with T12 + T13.

**Workspace: 2163/36 → 2165/32** — +2 passes / -4 failures. Net **-6 from the slice 1f-λ scope of 28** (consolidation saved 2 tests by reusing existing T4-T6 round-trip coverage).

## § The consolidation decision

The Phase A BRIEF asked: rewrite `tests/wat_arc104_fork_program.rs` (4 tests) on the canonical surface. Read of the canonical file `wat_arc170_program_contracts.rs` (T4-T6) revealed: **the round-trip scenario is already covered by T4-T6.** Re-running the four questions on consolidation vs separate-file:

1. **Obvious?** YES — one file for arc-170 spawn-process tests
2. **Simple?** YES — delete one file, add 2 tests to canonical home
3. **Honest?** YES — matches slice 1f-θ V3 doctrine: delete poison, write at canonical location
4. **Good UX?** YES — future readers find arc-170 surface tests in one place

Decision: delete `wat_arc104_fork_program.rs` entirely; add only the SCENARIOS that aren't already covered (one-way emit + clean tx-drop exit) to the canonical file as T12 + T13.

## § Scenario inventory + disposition

| Original arc-104 test | Disposition |
|---|---|
| `fork_program_round_trip_via_pipes` | DELETE (T4-T6 already cover the round-trip scenario in canonical home) |
| `fork_program_child_writes_stdout_parent_reads_line` | REPLACE with T12 — child emits without recv'ing first (one-way pattern, distinct from T4-T6) |
| `fork_program_clean_exit_code_via_wait_child` | REPLACE with T13 — child exits clean on parent tx-drop (distinct exit-trigger pattern from T4-T6) |
| `fork_program_parse_error_surfaces_as_exit_3` | DELETE (scenario obsolete — under spawn-process(fn), the worker fn is wat-compiled in PARENT's world before spawning; no parse-error-on-spawn path exists) |

**Net:** 4 deletions, 2 fresh tests. The 4 BareLegacy* failures close because the file's gone.

## § Scorecard

| Row | What | Result |
|-----|------|--------|
| A | Arc-104 file's BareLegacy* failures close | ✓ file deleted; 4 failures gone |
| B | Each surviving scenario has its own test; obsolete deleted with rationale | ✓ T12 (one-way emit) + T13 (tx-drop exit); 2 deletions documented in disposition table |
| C | All Phase A tests pass | ✓ 17/17 in `wat_arc170_program_contracts.rs` |
| D | Workspace failure count drops | ✓ 36 → 32 (-4) |
| E | `cargo check --release` green | ✓ clean compile, first try |
| F | Canonical pattern grep-able for Phase B | ✓ T12 + T13 land beside T4-T6 in canonical file; sonnet can grep `(:wat::kernel::spawn-process` for the call shape and `:wat::kernel::Receiver<` / `:Sender<` for worker signatures |
| G | Honest deltas surfaced | ✓ 4 categories below |

**7/7 rows pass.** Mode A clean.

## § Honest deltas (4 categories)

1. **Consolidation pivoted the BRIEF mid-flight.** Phase A's BRIEF said "rebuild `tests/wat_arc104_fork_program.rs`." Reading T4-T6 in the canonical file revealed the round-trip scenario was already covered; consolidating into the canonical home + writing only NEW-SCENARIO tests was the honest move. The BRIEF's "Phase A delivers 4 fresh tests" became "Phase A delivers 2 fresh tests + 4 deletions in the canonical home." Lesson: re-running the four questions during execution surfaces consolidation opportunities the pre-flight crawl missed.

2. **Worker fn shape: keyword defn vs inline lambda.** Both T12 and T13 use the `(:wat::core::defn :my::name [rx tx] -> :nil ...)` keyword-define shape (mirrors T4's pattern). Inline-lambda variant exists (T5) but is more verbose; for tests that aren't specifically exercising the inline-lambda path, the keyword form is cleaner.

3. **Process struct field access via Rust helpers.** Parent-side I/O uses the file-local helpers `process_tx_field` / `process_rx_field` / `process_handle_field` / `drive_typed_recv` / `unwrap_*` — these are the canonical accessor pattern for the Process Value's struct fields (idx 2=stderr, 3=ProgramHandle, 4=tx, 5=rx). Sonnet reading this file finds the helpers next to T4-T6; replication is direct.

4. **`drop(process)` triggers tx-disconnect.** T13's clean-exit mechanism relies on dropping the Process Value to drop its embedded Sender, which signals the child's recv to surface a Disconnected variant. The match arms cover Ok(None) (clean disconnect under arc 111 contract) + Ok(Some(_)) (impossible here but contract requires coverage) + Err(_) (peer-died fallback). The match-on-recv shape satisfies arc 110's "recv in match-or-expect position" rule.

## § Phase B path (sonnet sweep)

The remaining 24 failures live in:
- `tests/wat_arc103_spawn_program.rs` (6 tests; `spawn-program` substrate-retired)
- `tests/wat_fork.rs` (10 tests; `wait_child` / `fork_child` / legacy main)
- `crates/wat-cli/tests/wat_cli.rs` (10 tests; wat-cli echo, programs-are-atoms, sigterm cascade, panic-marker)
- `examples/with-loader/tests/smoke.rs` (1 test)
- `examples/with-lru/tests/smoke.rs` (1 test)

Phase B's sonnet BRIEF will:
- Point sonnet at T4-T6, T12, T13 as canonical references
- For each failing test file: inventory scenarios; for each scenario, decide rebuild-as-fresh OR delete-as-obsolete (mirror Phase A's disposition-table approach)
- Surface honest deltas per file

**Predicted Phase B runtime:** 60-120 min sonnet. The wat-cli + examples tests may have a different shape than the kernel-API tests (they exercise wat-cli's stdout pipeline rather than direct typed-channel access); Phase B's BRIEF surfaces this.

## § Files modified

- `tests/wat_arc104_fork_program.rs` — DELETED (146 lines; arc-104-substrate poison)
- `tests/wat_arc170_program_contracts.rs` — +73 lines (T12 + T13 at canonical home)

**Net: -73 lines** (146 deleted, 73 added; canonical home grew by 2 tests; arc-104 file zero).

## § What's next

1. **Atomic-commit Phase A** (this turn) — deletion + canonical extension + SCORE; push
2. **Phase B BRIEF** authored, citing Phase A's canonical pattern
3. **Phase B EXPECTATIONS** with sonnet runtime prediction
4. **Spawn sonnet** for Phase B with `model: "sonnet"`, `run_in_background: true`, ScheduleWakeup at 2× upper bound
5. **Triage** the 4 `slice4_*` heterogeneous failures (independent of arc 170)
6. **Arc 170 INSCRIPTION** when baseline is clean

## § Cross-references

- BRIEF: [`BRIEF-SLICE-1F-LAMBDA.md`](./BRIEF-SLICE-1F-LAMBDA.md)
- Canonical home: [`tests/wat_arc170_program_contracts.rs`](../../../../../tests/wat_arc170_program_contracts.rs) (now 17 tests; T1-T13 + 3 sub-tests T1b/T2b/T8b)
- Prior precedent: [`SCORE-SLICE-1F-THETA-V3.md`](./SCORE-SLICE-1F-THETA-V3.md) — delete-and-rewrite-at-canonical-vantage pattern
- Substrate diagnostic: `src/check.rs:732+` — the BareLegacy* messages that point at this migration
