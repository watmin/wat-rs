# Arc 170 slice 1i EXPECTATIONS — substrate-wide structured-exit protocol

**Single substrate slice.** Enforce existing `structured-stderr-only` doctrine ([`TIERS.md`](./TIERS.md):75) at every wat-process child exit path. wat-cli is benefactor, not implementor.

## Runtime band

**60-90 min sonnet.** Hard cap 180 min. Wakeup at T+10800s (substrate slice — extra headroom for Rust + wat-side + 3 probes).

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Custom panic hook installed in spawn_process_child_branch + fork.rs::child_branch_from_source; suppresses Rust default panic output to fd 2 in wat-process children | grep + read; `probe_no_default_rust_panic_noise_on_stderr` passes |
| B | `emit_structured_exit` (extended `emit_panics_to_stderr` or new helper) emits structured `#wat.kernel/ProcessPanics` EDN for ALL child exit paths: runtime error, plain panic, AssertionPayload panic, startup error, entry-form failure, bad-return, main-signature | `grep -rn "write_direct_to_stderr" src/` shows only helper itself + internal call from emit_structured_exit; no direct callers from exit paths |
| C | `ProcessDiedError` enum (`src/runtime.rs`) has variants for all kinds emitted (Panic / RuntimeError / StartupError / EntryFormFailure / BadReturn / MainSignature); any newly-minted variants documented in honest deltas | grep + read |
| D | `tests/probe_runtime_error_produces_structured_edn.rs` PASSES — runtime-error path produces structured EDN; `failure.message` is actual runtime error text, NOT "forked program exited N" | cargo test |
| E | `tests/probe_plain_panic_produces_structured_edn.rs` PASSES — plain panic path produces structured EDN | cargo test |
| F | `tests/probe_no_default_rust_panic_noise_on_stderr.rs` PASSES — Rust default panic handler output absent from `RunResult.stderr` lines | cargo test |
| G | Existing `tests/probe_runtime_err_stderr_visibility.rs` still PASSES — may need expectations tightened (Rust noise lines should now be absent) | cargo test |
| H | Wat-side harness `(None chain)` fallback retired in `run-hermetic-driver` + 3 siblings; replaced with structural-contract-violation panic; workspace failures from svc-tests now show ACTUAL diagnostic in `Failure.message`, not "exited 3" | grep + workspace test output |

**8 rows. All must PASS.**

### Row D + E + F + G — path-honesty discipline (Row G from Gap K carries forward)

Every new probe body MUST exercise the path its filename names. No silent path-switching:
- `probe_runtime_error_produces_structured_edn` — exercises a body that hits `Ok(Err(runtime_err))` in spawn-process child
- `probe_plain_panic_produces_structured_edn` — exercises a body that panics with a NON-AssertionPayload payload (bare String / &str / a raised non-payload value)
- `probe_no_default_rust_panic_noise_on_stderr` — exercises ANY panic path; asserts the stderr_lines do NOT contain Rust's default panic handler output

Manual review by orchestrator post-spawn: read each probe body against its filename + Row G claim.

### Row H — the user-facing impact

Currently the 5 svc-test workspace failures all show `Failure.message = "forked program exited 3"`. After this slice, those same 5 tests will STILL fail (this slice does not fix the underlying defect they hit) but their `Failure.message` will carry the ACTUAL runtime error text from the child. The test_runner's output (cargo test stdout/stderr) becomes useful diagnostic, not just an exit-code summary.

The workspace count probably stays at 167 pass / 7 fail (or close). The CONTENT of those 7 failures' diagnostics is what changes.

## Discipline mirror (orchestrator-side)

- FM 9: independent verification of each load-bearing row (not just "tests pass" — verify probe bodies match the BRIEF's claimed paths)
- FM 12: `model: "sonnet"` explicit on the Agent call
- FM 16: no Bash/tool-availability preamble in BRIEF (don't trigger meta-skepticism)
- FM 17: pre-action sweep before commit — does each test body verify what its filename claims (Row G); does the harness retirement preserve correctness (Row H)
- Atomic commit after scoring
- ScheduleWakeup at 2× upper-bound (180 min = 10800s; clamped by runtime to 3600s) — accept the clamp; first wakeup at 60 min is the orphan-check + halfway-point sniff
- Working tree pre-spawn: clean (just probe + new BRIEFs committed)

## Mode B trigger

If the substrate restructure ships but `extract-panics` still returns `None` on some exit path → STOP. That's a Class still violating the doctrine. Surface which path + why.

If `ProcessDiedError` enum needs variants that aren't minted: mint them; document in honest deltas. Don't deferral-language them ("future arc could...").

If the harness `(None chain)` retirement reveals dependencies in test code that rely on the old behavior: surface them honestly; either fix in this slice or document affirmatively as out-of-scope with clear naming (no deferral).

## Hard constraints (mirror BRIEF)

- DO NOT modify `src/check.rs`
- DO NOT add wall-clock timeouts ANYWHERE
- DO NOT touch deftest macro (V5 retry shape stays)
- DO NOT touch `docs/arc/` or `~/.claude/` (FM 11 + boundaries)
- DO NOT use `cd <subdir> && ...` — use absolute paths or `git -C` (FM 7)
- DO NOT commit / push / git add — orchestrator atomic-commits
- DO use `timeout -k 5 N` on cargo invocations (N=30 probe, N=90 workspace)
- DO NOT name probe files that don't match what they test (Row G)
