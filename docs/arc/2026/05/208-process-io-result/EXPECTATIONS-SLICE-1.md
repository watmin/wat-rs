# EXPECTATIONS — Arc 208 Slice 1

## Mode prediction

**Mode A — clean Result flip ships (~65%).** Sonnet audits substrate state, settles readln sub-decision (likely `Result<:I, ...>` — no Option wrapping needed at process tier), flips both verb signatures + eval handlers, mints test file with ~5-7 cases, workspace baseline preserved. Walker rule deferred to slice 2 (per BRIEF's default position). ~75-95 min wall-clock.

**Mode B — readln Option-wrapping needed (~20%).** Audit reveals substrate distinguishes clean stdin EOF from subprocess panic via the lifeline-pipe mechanism (per FD-multiplex Phase 1B work in arc 170). If clean EOF is distinct from death, `Result<:Option<:I>, :Vector<ProcessDiedError>>` mirrors `Receiver/recv` honestly. Sonnet surfaces, orchestrator confirms shape, sonnet completes. Adds ~10-15 min.

**Mode C — walker rule belongs in slice 1 (~10%).** After substrate flip lands, sonnet audits whether silent-Process-I/O-illegal is a 5-minute mirror of arc 110's walker or a bigger lift. If trivial (just add new ProcessPeer-call patterns to existing walker), absorbs in slice 1 for atomic substrate honesty. Adds ~15-25 min.

**Mode D — substrate prerequisite gap surfaces (~5%).** `ProcessDiedError` Value variant or Vector wrapping has some shape that doesn't compose cleanly into the Result wrapper. Sonnet surfaces; orchestrator decides whether to extend slice 1 scope or queue prerequisite slice.

**Mode E-time-violation — anything past 120 min.** Surface; orchestrator decides kill vs let-finish.

## Sub-decision prediction

Orchestrator's guess on `Process/readln` shape: **plain `Result<:I, :Vector<ProcessDiedError>>`** (no Option). Reasoning:

- At thread tier: `Receiver/recv` returns `Result<Option<T>, ...>` because the Sender side can drop cleanly (Sender::Drop → channel disconnect → recv returns Ok(None)). The "clean close" is a valid lifecycle event distinct from thread death.
- At process tier: the subprocess can't cleanly close its own stdout while continuing to live (or rather, if it does, it's a no-more-data signal that the parent treats as end-of-stream). When the parent's `Process/readln` returns end-of-stream, the subprocess is either about to exit or already gone — substantively the same as subprocess death from the parent's perspective.
- Arc 170's lifeline-pipe + FD-multiplex (Phases 1A-1E) eliminated the orphan-detection race; subprocess death is detected deterministically via FD EOF, not via timing.

So: clean stdin close == subprocess exit == "no more data possible." `Err(Vector<ProcessDiedError>)` is the honest signal; no separate Ok(None) state worth distinguishing.

Sonnet's audit may overturn — trust the audit. If sonnet finds a substrate mechanism that distinguishes clean-EOF-with-subprocess-still-alive (e.g., subprocess does `close(stdout)` then keeps running on stderr), Option is honest. Otherwise plain Result.

## Workspace baseline expected

Pre-existing 3-4 failures unchanged:
- `lifeline_pipe_zero_orphans_across_100_trials` (flaky)
- `deftest_wat_tests_tmp_totally_bogus`
- `t6_spawn_process_factory_with_capture_round_trips`
- `startup_error_bubbles_up_as_exit_3`

Acceptable post-slice-1: same set or any subset. Unacceptable: any NEW failure.

The flip is a SIGNATURE change — every consumer of `Process/readln`/`Process/println` needs to handle the Result. Arc 203 demos (`counter-service-process-N3.wat` + variants) currently call these verbs expecting raw `:I`/`:nil` returns; the signature flip will break them at type-check time. Slice 1 is ONLY substrate; the demo break is EXPECTED and is slice 2's scope. Sonnet should NOT fix the demos in slice 1 — that's consumer ripple.

**Honest expectation:** post-slice-1, arc 203 process-tier demos will FAIL type-check (signature mismatch). This is broken-intermediate per `feedback_no_broken_commits`. Mitigation: orchestrator's atomic-commit holds slice 1 + slice 2 work together if needed, OR slice 1 ships with arc 203 demo's signatures updated minimally (preserving Result-discarding via `match _ -> ... (_ ...)` pattern) as part of slice 1's substrate proof. Sonnet's call which path fits.

Actually — simpler: slice 1 should NOT touch arc 203 demos. The DEMO breakage is the substrate-as-teacher signal that consumer ripple is needed (slice 2). Slice 1's tests are NEW tests in `tests/wat_arc208_*` that exercise the new substrate verbs against the new signatures — they pass standalone without touching arc 203. After slice 1 ships, arc 203 demos will type-check-fail in workspace test; THAT's the slice 2 trigger.

Wait — `feedback_no_broken_commits` rules out shipping a broken commit. So slice 1 MUST leave workspace green. Options:
- (a) Slice 1 includes a minimal arc 203 demo Result-discard patch to keep them green (semantically equivalent: `match (Process/println peer data) -> ... ((:Ok _) ...) ((:Err _) ...)` collapsing both to old behavior)
- (b) Slice 1 includes the full arc 203 demo ripple (slice 2 scope collapses into 1)

Option (a) is cleaner stepping stone: minimal patch to keep workspace green; demo's error handling stays panic-on-Err (semantically same as today, just explicit); slice 2 then properly handles Err to surface ServerDied without crash-test-proc workaround.

Sonnet's call between (a) and (b). Likely (a) for cleaner stepping stones.

## Failure-mode catches

- FM 1 (grep before claiming): verification gate IS the grep audit
- FM 9 (baseline pre-flight): explicit in verification gate
- FM 11 (deferral language): N/A this slice (no INSCRIPTION)
- FM 16 (no tool preamble): BRIEF doesn't preamble Bash/cargo
- `feedback_no_broken_commits`: workspace MUST be green at slice 1 ship; arc 203 demos may need minimal Result-discard patch
- `feedback_no_known_defect_left_unfixed`: walker rule decision made in-slice OR explicit out-of-scope

## Atomic commit shape

NO commit by sonnet. Orchestrator commits all touched files + new test file + SCORE atomically when sonnet returns.

Expected commit shape: 3-5 files (check.rs + runtime.rs + maybe arc 203 demo minimal-patch + new test file + SCORE doc). ~200-400 lines diff.

## Calibration record

Arc 207 progression: slice 1 (audit) 36 min, slice 2 (substantive) 93 min, slice 3 (mechanical) ~30 min, slice 4 (ripple+Mode D) ~30 min, slice 5 (paperwork) ~35 min. Total ~3.5 hours sonnet for 5-slice arc.

Arc 208 prediction: 3 slices total. Slice 1 substantive (75-95 min). Slice 2 ripple (45-60 min). Slice 3 closure (30-45 min). Total ~2.5-3 hours sonnet for full arc 208.

Smaller because shape is settled by precedent (arc 110/111 templates) — no audit-vs-mint split needed at slice level.

Sonnet: trust the precedent; mirror cleanly; surface the readln sub-decision and walker decision honestly; return.
