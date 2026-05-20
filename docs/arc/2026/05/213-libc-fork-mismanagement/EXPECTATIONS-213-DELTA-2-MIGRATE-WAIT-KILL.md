# Arc 213 stone δ-2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 45-60 min Mode A. Three method migrations + one new helper. Larger than δ-1 (which was just additive field + 3 line changes); smaller than γ-1 (which mint+sister-fn extended).
- **LOC changed:** ~50-80 (3 method bodies rewritten ~10-20 LOC each + new helper ~10 LOC + doc-comment updates)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** LOW-MEDIUM. Pidfd methods proven by α smoke probe; semantics-parity migration; behavior unchanged.

## Honest-delta watch

### Risk 1 — ExitStatus → i64 mapping parity

`extract_exit_code(libc::c_int)` maps:
- WIFEXITED → WEXITSTATUS as i64
- WIFSIGNALED → 128 + WTERMSIG as i64
- else → -1

`extract_exit_code_from_status(ExitStatus)` must produce IDENTICAL i64 values for forked-child outcomes:
- ExitStatus::Exited(n) → n as i64 (same as WEXITSTATUS)
- ExitStatus::Signaled(s) → 128 + s as i64 (same as 128+WTERMSIG)
- ExitStatus::Stopped(_) → -1 (we don't request WUNTRACED via wait_status, so this branch shouldn't fire)

Risk: ExitStatus type from α may emit signal numbers in a different form (e.g., raw libc value vs platform-int). Sonnet checks α's SCORE + Pidfd implementation; if mismatch surfaces, document.

### Risk 2 — Drop's error-ignoring behavior parity

Current Drop ignores all errors (best-effort cleanup):
```rust
unsafe {
    libc::kill(self.pid, libc::SIGKILL);  // ignore
    let mut status: libc::c_int = 0;
    libc::waitpid(self.pid, &mut status, 0);  // ignore
}
```

δ-2 target:
```rust
let _ = self.pidfd.send_signal(libc::SIGKILL);
let _ = self.pidfd.wait_status();
```

Both `send_signal` and `wait_status` return `io::Result<...>`. Drop ignores via `let _ = ...`. Behavior parity preserved.

### Risk 3 — eval_kernel_wait_child error message format

Current: `format!("waitpid({}): {}", handle.pid, err)`
δ-2: `format!("wait_status({}): {}", handle.pid, err)`

The error message string CHANGES. Any wat-side test asserting on the exact error message format would fail. Sonnet's job: keep the format style consistent + document the change in SCORE.

### Risk 4 — Pidfd's wait_status consumption semantics

Does `wait_status(&self)` borrow self or consume? Per α's SCORE, Pidfd's `wait_status` takes `&self` (borrowing; doesn't consume). After wait_status returns, the Pidfd is still alive (so cached_exit reuse after subsequent calls works). Drop's wait_status fires on the now-dead process; idempotent like waitpid's ECHILD case.

If wait_status consumes self (`fn wait_status(self)`), the migration shape changes — ChildHandleInner would need Pidfd in a Cell or Option to consume on first call. Sonnet checks α's signature first.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `wait_or_cached` migrated to `self.pidfd.wait_status()` | YES |
| 2 | `Drop::drop` migrated to `self.pidfd.send_signal(SIGKILL)` + `self.pidfd.wait_status()` | YES |
| 3 | `eval_kernel_wait_child` migrated to `handle.pidfd.wait_status()` | YES |
| 4 | `extract_exit_code_from_status(ExitStatus) -> i64` helper minted | YES |
| 5 | `extract_exit_code(libc::c_int)` PRESERVED (δ-3 retires after dead-ness audit) | YES |
| 6 | `pub pid: libc::pid_t` field PRESERVED (δ-3 retires) | YES |
| 7 | libc::waitpid / libc::kill imports PRESERVED if other modules use them | YES (likely) |
| 8 | cargo build --release clean | YES |
| 9 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 10 | All 23 baseline test binaries: post-count == pre-count | YES |
| 11 | Zero modifications outside `src/fork.rs` | YES |
| 12 | Drop's error-ignoring behavior parity preserved (best-effort cleanup) | YES |
| 13 | eval_kernel_wait_child error format updated ("wait_status" replaces "waitpid") | YES |
| 14 | cached_exit semantics unchanged (same i64 values cached) | YES |
| 15 | SCORE inscribes ExitStatus → i64 mapping confirmation + any Pidfd semantics surprises | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; substrate's wait/kill paths PID-reuse-safe; orphan-leak class fully closed at reap path
- **Mode B (acceptable):**
  - Pidfd's wait_status consumption semantics differ from prediction (consumes vs borrows); REVERT + inscribe shape choice for orchestrator
  - ExitStatus → i64 mapping has subtle mismatch in a probe's assertion; REVERT + inscribe
- **Mode C:** STOP rule broken (touched δ-3 territory, removed pid field, retired extract_exit_code, scope-crept to γ sites)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (α's Pidfd methods proven by smoke probe; ExitStatus mapping is straightforward; Drop's error-ignoring is identical pattern). MEDIUM-HIGH on first-attempt Mode A (signal-number cross-platform consistency is the main risk; α uses libc::SIGTERM directly so should match).
- **Risk factors:**
  - ExitStatus::Signaled(s) signal-number representation (raw libc value vs WTERMSIG-style)
  - Pidfd::wait_status consumption semantics
  - Any wat-side test asserting on exact error message format
- **Why this matters:** completes the substrate-side PID-reuse race elimination at the reap path. After δ-2: every fork-and-wait flow in the substrate uses kernel-direct syscalls (waitid(P_PIDFD), pidfd_send_signal). δ-3 retires the libc fallback + removes pid field. ε migrates probe-side observation (different domain; orthogonal). ζ enforces L2 module privacy. η ships INSCRIPTION.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Post-δ-1, δ-2 is the obvious next step (sequential δ chain; δ-1 minted the field; δ-2 uses it). No tiebreaker needed within the δ chain.

The broader-arc tiebreaker (δ-2 vs ε vs ζ vs arc 212 ζ-1) deferred until δ-3 (the δ chain's natural endpoint).

## Cross-references

- Arc 213 DESIGN — full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 α SCORE — Pidfd methods (wait_status / send_signal / try_wait / poll_exit) + ExitStatus enum
- Arc 213 δ-1 SCORE (will exist post δ-1) — ChildHandleInner pidfd field mint
- `src/fork.rs:217-233` — wait_or_cached (Site 1 target)
- `src/fork.rs:236-250` — Drop::drop (Site 2 target)
- `src/fork.rs:296-...` — eval_kernel_wait_child (Site 3 target)
- `src/fork.rs:252-267` — extract_exit_code(libc::c_int) (existing helper; reference for the new helper's semantics)
- `feedback_substrate_owns_not_callers_match` — doctrine extends: substrate owns "wait via Pidfd"
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — architectural commitment δ-2 operationalizes
