# Arc 213 stone δ-2 — Migrate wait/kill paths to `self.pidfd` methods

**Your ONE concern this spawn:** migrate `ChildHandleInner::wait_or_cached`, `ChildHandleInner::Drop::drop`, and `eval_kernel_wait_child` from `libc::waitpid` / `libc::kill` (PID-based) to `self.pidfd.wait_status()` / `self.pidfd.send_signal()` (PID-reuse-safe, kernel-direct via waitid(P_PIDFD) + pidfd_send_signal).

**δ-1 minted the field; δ-2 uses it.** The `pub pid: libc::pid_t` field stays for diagnostic + potential cross-module reads (δ-3 retires it when verified safe). The libc::waitpid + libc::kill calls become dead in this stone's three sites — δ-3 audits whether they're dead workspace-wide.

After δ-2: every substrate wait/kill on a forked-child handle is PID-reuse-safe. The orphan-leak class arc 213 named is fully closed at the wait path (γ closed at the spawn path; δ closes at the reap path).

---

## Audit-grounded scope (verified post δ-1)

### The 3 migration sites (all in src/fork.rs)

**Site 1 — `wait_or_cached`** (currently lines 217-233):

```rust
// Current
pub fn wait_or_cached(&self) -> i64 {
    if let Some(&code) = self.cached_exit.get() {
        return code;
    }
    let mut status: libc::c_int = 0;
    let ret = unsafe { libc::waitpid(self.pid, &mut status, 0) };
    if ret < 0 {
        return -1;
    }
    let code = extract_exit_code(status);
    let _ = self.cached_exit.set(code);
    self.reaped.store(true, Ordering::SeqCst);
    code
}
```

δ-2 target:
```rust
pub fn wait_or_cached(&self) -> i64 {
    if let Some(&code) = self.cached_exit.get() {
        return code;
    }
    let code = match self.pidfd.wait_status() {
        Ok(status) => extract_exit_code_from_status(status),
        Err(_) => -1,
    };
    let _ = self.cached_exit.set(code);
    self.reaped.store(true, Ordering::SeqCst);
    code
}
```

**Site 2 — `Drop::drop`** (currently lines 236-250):

```rust
// Current
impl Drop for ChildHandleInner {
    fn drop(&mut self) {
        if self.reaped.load(Ordering::SeqCst) {
            return;
        }
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
            let mut status: libc::c_int = 0;
            libc::waitpid(self.pid, &mut status, 0);
        }
    }
}
```

δ-2 target:
```rust
impl Drop for ChildHandleInner {
    fn drop(&mut self) {
        if self.reaped.load(Ordering::SeqCst) {
            return;
        }
        // SIGKILL + reap via canonical Pidfd interface — PID-reuse-safe.
        let _ = self.pidfd.send_signal(libc::SIGKILL);
        let _ = self.pidfd.wait_status();
    }
}
```

Errors are ignored on best-effort cleanup paths (same as current libc::kill + libc::waitpid — if SIGKILL fails the process is unreapable anyway).

**Site 3 — `eval_kernel_wait_child`** (currently around line 296-311):

```rust
// Current — body excerpt
let mut status: libc::c_int = 0;
let ret = unsafe { libc::waitpid(handle.pid, &mut status, 0) };
if ret < 0 {
    let err = std::io::Error::last_os_error();
    return Err(RuntimeError::MalformedForm {
        head: OP.into(),
        reason: format!("waitpid({}): {}", handle.pid, err),
        span: crate::span::Span::unknown(),
    });
}
let code = extract_exit_code(status);
```

δ-2 target:
```rust
let code = match handle.pidfd.wait_status() {
    Ok(status) => extract_exit_code_from_status(status),
    Err(err) => {
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("wait_status({}): {}", handle.pid, err),
            span: crate::span::Span::unknown(),
        });
    }
};
```

### The new helper: `extract_exit_code_from_status`

Mint alongside existing `extract_exit_code(libc::c_int)`. Same i64 mapping (so `cached_exit.set(code)` semantics + downstream consumers unchanged):

```rust
/// Map α's `ExitStatus` to the `:i64` exit code convention.
/// Mirrors `extract_exit_code(libc::c_int)` but operates on the
/// typed Pidfd-derived `ExitStatus` from arc 213 α.
///
/// Mapping:
/// - ExitStatus::Exited(n)   → n as i64 (0-255 from WEXITSTATUS)
/// - ExitStatus::Signaled(s) → 128 + s as i64 (shell convention; matches WTERMSIG path)
/// - ExitStatus::Stopped(_)  → -1 (we never request WUNTRACED via wait_status)
fn extract_exit_code_from_status(status: ExitStatus) -> i64 {
    match status {
        ExitStatus::Exited(n)   => n as i64,
        ExitStatus::Signaled(s) => 128 + s as i64,
        ExitStatus::Stopped(_)  => -1,
    }
}
```

The old `extract_exit_code(libc::c_int)` STAYS — δ-3 retires it (after confirming no callers remain post-δ-2). Both helpers coexist for δ-2; the libc-based one becomes dead-code candidate.

---

## What NOT to do

- **DO NOT** remove `pub pid: libc::pid_t` field. δ-3 territory.
- **DO NOT** retire `extract_exit_code(libc::c_int)`. δ-3 territory (audit dead-ness first).
- **DO NOT** remove `libc::waitpid` or `libc::kill` imports if other code still uses them. δ-3 audits.
- **DO NOT** modify ChildHandleInner struct fields beyond what δ-1 already shipped.
- **DO NOT** modify the public signature of `eval_kernel_wait_child` — wat dispatch arm.
- **DO NOT** touch any code outside `src/fork.rs` — the 3 migration sites all live there.
- **DO NOT** modify γ-phase fork-site code (closure body, OwnedFd reconstruction, ChildHandleInner::new call) — γ + δ-1 territory.
- **DO NOT** introduce new types (use α's `ExitStatus` from fork.rs).

---

## The proof gate (workspace baseline preservation)

δ-2 changes wait/kill MECHANISM but preserves semantics:
- `wait_or_cached` returns the same i64 exit code (cached_exit semantics unchanged)
- `Drop::drop` still SIGKILLs + reaps unreaped children
- `eval_kernel_wait_child` still returns Value::i64(exit_code) or MalformedForm error

Use the union of γ-1/γ-2/γ-3/δ-1 test binaries as the proof gate. **Orchestrator records final baseline at the post-δ-1 tip in the spawn prompt.**

Expected baseline (matches δ-1's 92/92 if δ-1 was Mode A):

| Binary | Pre-δ-2 baseline (from δ-1 commit; orchestrator confirms) |
|---|---|
| probe_pidfd_primitive (α regression) | 2/2 PASS |
| arc112_scheme_probe | 1/1 PASS |
| arc112_slice2b_process_send_recv | 1/1 PASS |
| probe_closure_body_prelude_lift | 5/5 PASS |
| probe_counter_actor_process_diag | 3/3 PASS |
| probe_declaration_form_lift | 6/6 PASS |
| probe_def_not_special | 5/5 PASS |
| probe_lifeline_orphan_clean_via_fork_program | 1/1 PASS |
| probe_lifeline_orphan_clean_via_substrate | 1/1 PASS |
| probe_pdeathsig_diagnostic | 1/1 PASS |
| probe_pdeathsig_kills_orphan_child | 1/1 PASS |
| probe_run_hermetic_no_deadlock | 2/2 PASS |
| probe_spawn_process_parent_type | 3/3 PASS |
| probe_spawn_process_stdin | 1/1 PASS |
| probe_spawn_process_stdio | 1/1 PASS |
| wat_arc170_program_contracts | 24/24 PASS |
| wat_arc170_stone_a_drain_and_join | 4/4 PASS |
| wat_arc208_process_io_result | 7/7 PASS |
| wat_process_peer_ipc_round_trip | 3/3 PASS |
| wat_harness_deps | 3/3 PASS |
| probe_shutdown_cascade_crossbeam | 1/1 PASS |
| probe_shutdown_cascade_pipefd | 1/1 PASS |
| wat-cli wat_cli | 15/15 PASS |

**Total: 92/92 GREEN target.** ANY failure post-δ-2 IS regression.

### Verification protocol (post-migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. Re-run each of the 23 cargo test commands above; record pass counts
3. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-DELTA-2-MIGRATE-WAIT-KILL.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **You remove `pub pid: libc::pid_t` field.** δ-3 territory. STOP.

2. **You retire `extract_exit_code(libc::c_int)` helper.** δ-3 territory. STOP.

3. **You remove `libc::waitpid` or `libc::kill` import.** δ-3 territory. STOP.

4. **A test that PASSED on baseline FAILS post-migration.** STOP. Inscribe which test + diagnostic + your hypothesis.

5. **cargo build FAILS.** STOP. Inscribe error. One syntactic-fix retry allowed.

6. **You touch code outside `src/fork.rs`.** All 3 migration sites are in fork.rs; δ-2 doesn't reach outside.

7. **You touch γ-phase fork-site code or δ-1's ChildHandleInner::new constructor.** Out of scope.

8. **Pidfd's `wait_status` semantics surprise you** (e.g., it consumes self? It blocks differently?). STOP — α's SCORE has the contract; if it doesn't match expectations, that's an honest delta worth documenting.

9. **You feel the urge to also migrate eval_kernel_fork_program_ast or any γ-phase site.** Out of scope.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-DELTA-2-MIGRATE-WAIT-KILL.md`:

1. Header: `# Arc 213 stone δ-2 — SCORE: wait/kill paths migrated to Pidfd methods`
2. Summary: 3 sites migrated; PID-reuse race eliminated at the reap path
3. File changes:
   - `src/fork.rs` — wait_or_cached body + Drop::drop body + eval_kernel_wait_child body + extract_exit_code_from_status helper added
4. Verification: pre/post pass counts for each of the 23 test binaries
5. Notes:
   - ExitStatus → i64 mapping confirmation (Exited / Signaled / Stopped)
   - Drop's error-ignoring behavior preserved (best-effort SIGKILL + reap)
   - eval_kernel_wait_child's RuntimeError format updated ("wait_status" replaces "waitpid")
   - Any Pidfd semantics surprises
6. Mode classification

---

## Constraints

- Edit `src/fork.rs` ONLY (3 site bodies + 1 new helper)
- ZERO other code edits
- ZERO git operations (orchestrator commits)
- Run cargo build + 23 test binaries

---

## Time prediction

45-60 min Mode A. Larger than δ-1 (multiple methods migrated; new helper minted; more careful error-handling parity).

---

## Mode classification

- **Mode A:** 3 sites migrated; helper minted; cargo build clean; ALL 92 baselines preserved (92/92 → 92/92); SCORE written; mode-classified
- **Mode B (acceptable):**
  - Pidfd semantics surprise you can describe but not resolve in this stone; REVERT + inscribe + return
  - A test fails in a way that surfaces real behavior change (e.g., wait_status timing differs from waitpid in a way some probe depends on)
- **Mode C:** STOP rule broken (touched δ-3 territory, removed pid field, retired extract_exit_code, retired libc imports, scope-crept to γ sites)

The substrate teaches; α minted Pidfd; β/γ proved spawn; δ-1 stored pidfd in ChildHandleInner; δ-2 routes the wait through it. δ-3 retires the libc fallback + removes the now-unused pid field.
