# BRIEF — Stone 6.w Strike 1: process/ structural convergence (+ runtime SHUTDOWN)

**Executor:** sonnet (background). **Home:** `src/process/` (+ the `src/runtime.rs`
SHUTDOWN-statics in-scope items + the one out-of-home doc fix F1). **Design substrate:**
`docs/arc/2026/05/214-concurrency-toolkit/SCORE-STONE-6.w-VIGILIA-FINDINGS.md` — READ
IT FIRST; it holds every finding's detail. This brief draws the strike path; the ledger
is the room map. **Greedy stance:** fix every item; no banking.

## The work, in one paragraph
Drive `src/process/` to L1+L2=0 by executing the ledger's PROCESS/ section + the δ-2/δ-3
pidfd SPINE + excusare P-1/P-3/P-4 + circumspicere F2/F4 + the runtime.rs:248 idempotent
side-catch + the F1 doc fix. The spine is the load-bearing structural change; do it FIRST
and verify it before the mechanical items. Nothing user-facing changes; behavior is
preserved exactly (exit-code shell convention, post-fork RAII fd discipline, teardown).

## Read in order (the rooms)
1. `docs/arc/2026/05/214-concurrency-toolkit/SCORE-STONE-6.w-VIGILIA-FINDINGS.md` — the full fix-list (PROCESS/ section + THE SPINE + EXCUSARE VERDICTS + CIRCUMSPICERE VERDICTS).
2. `src/process/handle.rs` (130 lines, whole) — the spine target: `wait_or_cached` (68-84), `Drop` (87-101), `extract_exit_code` (108-118), the `pid` field (29).
3. `src/process/clone.rs:103-210` — the `Pidfd` methods to migrate ONTO: `wait_status()` (159), `send_signal()` (190), `poll_exit()` (130, currently dead), `try_wait()` (166, currently dead), `extract_exit_status_from_siginfo` (the sibling decoder used at :184).
4. `src/process/verbs.rs` + `src/process/child.rs` — the 3 child-branch dup + the emit_panics/emit_structured_exit dup + the conformare span discards + the exigere comments (all cited with line numbers in the ledger PROCESS/ section).

## THE SPINE — δ-2/δ-3 pidfd migration (do this FIRST, verify, then the rest)
The struct already OWNS `self.pidfd: Pidfd` next to the raw `pid` field; the race-path
methods bypass it. Migrate:
- **`wait_or_cached` (handle.rs:73):** replace `libc::waitpid(self.pid, …)` →
  `self.pidfd.wait_status()` (returns `io::Result<ExitStatus>`, reaps atomically). Map
  the `ExitStatus` → `i64` via the EXACT shell convention `extract_exit_code` uses today
  (normal: `WEXITSTATUS`/`.code()`; signal: `128 + signal`). On `Err`, keep the `-1`
  sentinel. Preserve the `cached_exit` OnceLock caching + the `reaped` store.
- **`Drop` (handle.rs:95-99):** replace `libc::kill(self.pid, SIGKILL)` →
  `self.pidfd.send_signal(libc::SIGKILL)` (PID-reuse-safe) and the reaping
  `libc::waitpid(self.pid)` → `self.pidfd.wait_status()`. Ignore errors (best-effort
  teardown), as today.
- **secare TOCTOU:** gate the reap behind `self.reaped.compare_exchange(false, true,
  AcqRel, Acquire)` so exactly ONE caller (Drop vs a concurrent wait_or_cached) performs
  the wait. The winner waits; the loser returns the cached/sentinel. (Today `reaped` is a
  plain load/store — the window is the finding.)
- **Consolidate the decoder (solvere F-PR-3):** once `wait_or_cached` no longer calls
  `waitpid`, `handle.rs::extract_exit_code` (the c_int-status decoder) is redundant with
  clone.rs's siginfo decoder. DELETE `extract_exit_code`; route the ExitStatus→i64 map
  through ONE helper (add it next to `extract_exit_status_from_siginfo` in clone.rs, or
  make `ExitStatus` carry the i64 mapping). One decoder, one shell convention.
- **δ-3 retire the raw `pid` field (handle.rs:29):** after the two paths use `self.pidfd`,
  the `pid` field has only diagnostic/cascade readers. Check every `self.pid` /
  `.pid` reader (grep `ChildHandleInner` + `\.pid\b`). If a reader needs the pid for
  CASCADE interop (killpg), keep it via `self.pidfd.pid()` (the method at clone.rs:208)
  and DROP the struct field. `ChildHandleInner::new` drops `pid: pidfd.pid()`.
- **poll_exit/try_wait come ALIVE by use (purgare F2/F3):** the migration makes them
  reachable; do NOT delete them. If after the migration they STILL have zero callers,
  that is a real finding — STOP-3 (below).

## STOP triggers (rejection criteria — ship nothing, surface the gap)
- **STOP-1 (δ-3 cascade interop):** if retiring the `pid` field breaks a cascade/killpg
  reader that genuinely needs the raw pid and `pidfd.pid()` cannot serve it — STOP; keep
  the field with an honest rune and report. Do not invent a workaround.
- **STOP-2 (child-branch dedup RAII):** the 3-child-branch dedup (solvere F-PR-1, ~400
  lines: `child_branch` / `child_branch_from_source` / `spawn_process_child_branch`) must
  PRESERVE the post-fork RAII fd ownership exactly (the OwnedFd-drop-closes-parent-ends
  discipline, the dup2-then-_exit order, the lifeline mem::forget). If the shared
  `run_forked_child` kernel cannot preserve every ownership move — STOP; report which move
  resists extraction. A botched dedup that drops an fd wrong is worse than the dup.
- **STOP-3 (still-dead methods):** if after the spine migration `poll_exit`/`try_wait`
  still have zero callers — STOP and report (don't delete, don't force a fake caller).
- **General:** if any change would alter the exit-code shell convention or the
  observable teardown behavior — STOP. Behavior is preserved.

## The mechanical items (after the spine verifies green)
Execute every PROCESS/-section item + excusare + circumspicere F1/F2/F4 + side-catch from
the ledger. Highlights (full detail in the ledger):
- excusare **P-1**: delete the inert `#[allow(non_camel_case_types)]` at clone.rs:43 (no code change; CloneArgs is already CamelCase).
- excusare **P-3/P-4**: doc-text — name the 12 params (verbs.rs:754); re-point the dead-file citation `fork.rs::child_branch_from_source` → `src/process/verbs.rs::…` (verbs.rs:1111).
- circumspicere **F2**: `runtime.rs:334` + `:349` `std::process::exit(1)` → `unsafe { libc::_exit(1) }` (fork-safe; matches the adjacent raw `libc::write`).
- circumspicere **F4**: runtime.rs:309-311 — add the cross-step comment (close_range already closed it; this close's EBADF is expected/benign in the canonical path).
- circumspicere **F1** (out-of-home doc): `docs/ZERO-MUTEX.md:478-489` — `CHILD_PID`→`CHILD_PGID`, `kill(2) forwarding`→`killpg(2) broadcast` (arc 106 generalized PID→PGID; code is `crates/wat-cli/src/lib.rs:118`/`:615`).
- side-catch: `runtime.rs:248` — the `SHUTDOWN_BROADCAST_READ_FD` "Once set, never re-set (idempotent init)" doc is now false (6.4 re-sets it in fork children ~:356); fix the comment to state the fork-rebirth re-set.
- the emit_panics_to_stderr_fork/_spawn + emit_structured_exit merge (intueri L1/solvere/struere/temperare); clone3-name-lie inscription; ChildHandleInner phantom-outer → rename to ChildHandle; input/output→stdin/stdout vocab; _inherit_config WIRED not deleted; LifelineWriter::close DELETE; CloneArgs pub→pub(super); spawn_lifelined_any dedup; wait_or_cached→wait_or_cached_exit (or moot post-spine); Phase-1C stale comment present-tense; perspicere runes; δ-1/δ-2 attested-arc comments become moot post-spine.

## Blast radius
`src/process/{handle,clone,child,verbs,stdio,mod}.rs` + `src/runtime.rs` (the 3 cited
SHUTDOWN sites + :248 doc) + `docs/ZERO-MUTEX.md` (2-word doc). NO changes to
`src/channel/`, `src/kernel/`, or `src/comms/`. NO new public API. NO behavior change.

## Verify (run these; report each)
- `cargo test --release --lib -p wat` — green (baseline will be supplied; match it).
- `cargo clippy --release -p wat 2>&1 | grep -iE "src/process/"` — ZERO warnings (clippy-in-home is the L2 ward bar).
- Enveloped process tests still green: `setsid timeout 120 cargo test --release --test comms -- --ignored 2>&1 | tail` for the gamma/hermetic/lifeline paths (the δ-2 migration must not change teardown/exit behavior).
- Confirm the exit-code shell convention preserved (a child exiting 0 → 0; SIGKILL → 137).

## Deliverable
The edits applied + a structured report: each finding's disposition (FIXED / STOP-n /
rune-with-reason), the spine's before/after, the verify results (each command + real
outcome), honest deltas, line counts (`wc -l`, not estimates). Do NOT commit — the
orchestrator weighs against the disk and commits. Your final message IS the report.
