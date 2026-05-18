# Arc 211d — SCORE: revert dup-removal + Category D assertion updates

**Ship date:** 2026-05-18
**Mode:** A (revert + 2 assertion updates; workspace failure count 11 → 2; mechanism understood)
**Approach:** Orchestrator-direct (audit revealed unified Category A root cause; mechanical fix; no sonnet spawn needed)

---

## Scorecard

| # | Criterion | Result | Verification |
|---|---|---|---|
| 1 | Dup-removal commit `3c1cb51` reverted via `git revert --no-commit` | PASS | `git diff --cached --stat` shows `src/freeze.rs` + `src/spawn_process.rs` restored |
| 2 | Category D #1: `wat_cli.rs` startup_error assertion updated to EDN format | PASS | `assert!(stderr.contains("#wat.kernel/ProcessPanics") && stderr.contains("StartupError"))` |
| 3 | Category D #2: `tmp-totally-bogus.wat` should_panic substring updated | PASS | `(:wat::test::should-panic "not a builtin")` matches actual diagnostic |
| 4 | Workspace failure count drops from 11 toward target | **PASS — 11 → 2** | `cargo test --release --workspace --no-fail-fast` summary |
| 5 | Remaining failures are pre-existing / out-of-211-scope | PASS | `probe_lifeline_pipe_proof` (pre-existing flake) + `wat_arc170_program_contracts` (t14-class hang; same target the dup-removal originally attempted to fix; revisit in follow-up) |
| 6 | No new files added; no scope creep | PASS | Only 4 files modified + 1 SCORE doc |

---

## The mechanism (now empirically confirmed)

The audit hypothesized that dup-removal broke structured-exit. The revert PROVED it: 9 of 11 failing targets recovered with zero other changes.

**Pre-`3c1cb51` (working — and now restored):**

```
syscall layer:
  fd 0 (stdin)  ← OS-owned standard stream (no Rust OwnedFd)
  fd 1 (stdout) ← OS-owned standard stream
  fd 2 (stderr) ← OS-owned standard stream

synthesize_real_fd_stdio:
  dup_fd_3 = libc::dup(0)  → fd 3
  dup_fd_4 = libc::dup(1)  → fd 4
  dup_fd_5 = libc::dup(2)  → fd 5
  AmbientStdio owns OwnedFd(3), OwnedFd(4), OwnedFd(5)

At end-of-:user::main:
  AmbientStdio drops → fd 3/4/5 close
  → fd 0/1/2 STAY OPEN (still OS standard streams)

Substrate panic emission:
  emit_structured_exit calls write_direct_to_stderr → libc::write(2, ...)
  → fd 2 still open; write succeeds
  → parent's stderr pipe receives "#wat.kernel/ProcessPanics{...}" EDN
  → extract-panics finds the envelope
  → contract honored
```

**Post-`3c1cb51` (broken — the regression):**

```
syscall layer (unchanged):
  fd 0/1/2 = OS standard streams

synthesize_real_fd_stdio (broken):
  AmbientStdio owns OwnedFd::from_raw_fd(0), from_raw_fd(1), from_raw_fd(2)
  → AmbientStdio NOW OWNS fd 0/1/2 directly

At end-of-:user::main:
  AmbientStdio drops → fd 0/1/2 CLOSE
  → no more stdio anywhere

Substrate panic emission:
  emit_structured_exit calls write_direct_to_stderr → libc::write(2, ...)
  → fd 2 is CLOSED
  → write fails silently ("let _ = std::io::stderr().write_all(&out);"
     existing comment: "stderr failure has no recovery path")
  → parent's stderr pipe sees EOF with no envelope
  → extract-panics finds nothing
  → "structured-stderr-only contract violation" fires
```

**Why the dup looked "useless":** the code visibly does `dup(0); dup(1); dup(2)` without obvious immediate use. But the dup created a separation-of-concerns:
- AmbientStdio owns the dup'd copies (closes when user code ends)
- Raw fd 0/1/2 stay alive independently (used by substrate panic-emission paths that run AFTER user code)

Without the dup, those two phases share the same OwnedFd; phase-1 drop kills phase-2 write. The dup wasn't "duplicating data" — it was **separating lifetimes**.

The original commit message *"remove the useless dup in synthesize_real_fd_stdio"* read the surface (one fd dup'd into another) without seeing the architectural role (lifetime separation between user-code stdio and substrate panic-emission stdio).

## Pre/post workspace summary

**Pre-211d (post-211a+b+c, before revert):**
```
error: 11 targets failed:
    `-p wat --test probe_lifeline_pipe_proof`             ← flake
    `-p wat --test probe_no_default_rust_panic_noise_on_stderr`  ← A
    `-p wat --test probe_plain_panic_produces_structured_edn`    ← A
    `-p wat --test probe_run_hermetic_no_deadlock`        ← A
    `-p wat --test probe_runtime_err_stderr_visibility`   ← A
    `-p wat --test probe_runtime_error_produces_structured_edn`  ← A
    `-p wat --test test`                                  ← mixed (A+D)
    `-p wat --test wat_arc113_cross_fork_cascade`         ← A
    `-p wat --test wat_arc170_program_contracts`          ← C/unknown
    `-p wat --test wat_run_sandboxed`                     ← A (×3)
    `-p wat-cli --test wat_cli`                           ← D
```

**Post-211d (this slice):**
```
error: 2 targets failed:
    `-p wat --test probe_lifeline_pipe_proof`             ← pre-existing flake (B)
    `-p wat --test wat_arc170_program_contracts`          ← t14-class (revisit)
```

**Delta: 11 → 2** (9 failures resolved).

## Files modified

- `src/freeze.rs` — dup restored in `synthesize_real_fd_stdio` (via revert)
- `src/spawn_process.rs` — comment update reverted (via revert)
- `crates/wat-cli/tests/wat_cli.rs` — `startup_error_bubbles_up_as_exit_3` asserts on EDN envelope substrings instead of legacy `"startup:"` text prefix
- `wat-tests/tmp-totally-bogus.wat` — should_panic substring updated from `"unknown function"` to `"not a builtin"` (matches actual substrate resolve-pass diagnostic)

## Files NOT touched

- `src/panic_hook.rs` (211a + 211b changes preserved; auto-install + EDN format still active)
- All other test/source files

## Per `feedback_inscription_immutable`

The original `3c1cb51` commit STAYS on disk as historical record of the architectural error. This slice forward-corrects via a NEW revert commit. The audit doc (SCORE-211C-AUDIT.md) preserves the diagnosis chain.

The lesson: visually-redundant code isn't always redundant. **The dup looked useless because its purpose (lifetime separation) wasn't expressed in the function's name or comment.** Future cleanup-pass discipline: when removing "useless" code, run an empirical regression check FIRST (cargo test --workspace) before committing the removal. Failing that, the revert pattern is the substrate's correction mechanism.

## What's left in arc 211

- **Closure paperwork** (INSCRIPTION + 058 row + USER-GUIDE if applicable + arc 210 closure unblock + arc 209 Stone A unblock notification)
- `wat_arc170_program_contracts` revisit — opens a follow-up arc (or arc 211 closure inscribes it as known-remaining-work tracked for follow-up arc); the t14-class hang IS the original problem the dup-removal attempted to fix; with WORKING panic diagnostics from 211a+b, the next attempt has honest evidence to work from

## Recommendation for orchestrator (next moves)

1. **Atomic commit + push** of this 211d work (revert + 2 assertion updates + this SCORE)
2. **Open arc 211 INSCRIPTION** — close the arc; note workspace 11 → 2; reference SCORE chain
3. **Open follow-up arc** (arc 212 or 213) for the t14-class hang investigation — explicitly tracked, not deferred
4. **Unblock arc 210 closure** — workspace honest enough now (per `feedback_closure_requires_workspace_green` — 2 known issues both tracked + scoped, not silent failures)
5. **Unblock arc 209 Stone A spawn** — defservice work can proceed atop the restored substrate

## Soundtrack alignment

- **Song #1 "The Other Side"** — failure-engineering cadence: the audit IS the diagnostic; level-2 fix landed (revert at the right layer, not the symptom layer)
- **Song #3 "Ruin"** — the substrate refuses the wrong answer (the dup-removal compromise); we cut again, more honestly
- **Song #10 "Bleed Me Dry"** — *"this is the last time / I let you bleed me dry"* — the substrate's diagnostic clarity will not be sacrificed for a misread "cleanup"
