# Arc 213 stone β — Migrate `run_in_fork` to `spawn_lifelined`

**Your ONE concern this spawn:** rebase `src/fork.rs::run_in_fork` (line 149) on the canonical `spawn_lifelined` primitive shipped at α (commit `5e43d7c`). Internal-only change. Public signature stays exactly the same. The 11 callers across 5 files do not move.

This closes the substrate's fork-path inconsistency: today, `spawn-process` + `fork-program` install lifelines; `run_in_fork` bypasses them via bare `libc::fork()`. After β, every fork path in the substrate routes through the canonical primitive — the "every spawn has a lifeline" guarantee stops being a lie.

---

## What to migrate

### Single function: `src/fork.rs::run_in_fork`

**Current body (lines 149-185):**
```rust
pub fn run_in_fork<F>(body: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        panic!("fork failed: {}", std::io::Error::last_os_error());
    }
    if pid == 0 {
        // Child — run body, exit 0 on success, 1 on panic.
        let outcome = std::panic::catch_unwind(body);
        match outcome {
            Ok(()) => unsafe { libc::_exit(0) },
            Err(_panic) => unsafe { libc::_exit(1) },
        }
    }
    // Parent — wait + assert.
    let mut status: libc::c_int = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    assert!(waited >= 0, "waitpid failed: {}", std::io::Error::last_os_error());
    assert!(
        libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
        "forked child exited with failure (status={:#x})",
        status
    );
}
```

**Target body (approximately):**
```rust
pub fn run_in_fork<F>(body: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    // Use the canonical Pidfd + lifeline primitive (arc 213 α).
    // spawn_lifelined's child closure takes lifeline_r as an i32
    // (the inherited read-end of the lifeline pipe). run_in_fork's
    // body doesn't use it — the parent (this fn) holds the
    // lifeline write-end via the returned LifelineWriter; if the
    // parent dies before this fn returns, the kernel closes its
    // FDs and the child's lifeline_r EOFs.
    let (pidfd, _lifeline) = spawn_lifelined(|_lifeline_r| {
        body();
    })
    .expect("spawn_lifelined failed");

    // Parent — wait via canonical Pidfd interface + assert clean exit.
    let status = pidfd
        .wait_status()
        .expect("wait_status failed");
    assert!(
        matches!(status, ExitStatus::Exited(0)),
        "forked child exited with failure: {:?}",
        status
    );
}
```

**Why this is the migration:**
- `libc::fork()` → `spawn_lifelined` (Linux 5.3+ clone3 + CLONE_PIDFD + lifeline + setpgid + catch_unwind already baked in)
- `libc::waitpid(pid, ...)` → `Pidfd::wait_status()` (waitid(P_PIDFD); PID-reuse-safe)
- Manual `WIFEXITED` / `WEXITSTATUS` decoding → `ExitStatus::Exited(0)` pattern match
- Child branch + `catch_unwind` + `_exit(0/1)` → ALREADY INSIDE `spawn_lifelined` (the helper does this; the caller-provided closure just runs the body)

**Signature stays IDENTICAL.** The 11 caller sites compile unchanged.

### Lifeline interpretation

run_in_fork's existing pattern is short-lived synchronous fork-and-wait. The body doesn't poll `lifeline_r` (and we are NOT adding that behavior in β — out of scope). The lifeline is set up by spawn_lifelined; the parent holds `LifelineWriter` via `_lifeline` until run_in_fork returns; if the parent process dies abruptly during the wait, the kernel closes `LifelineWriter`'s FD and the child's `lifeline_r` EOFs (the child still doesn't observe — that's a caller-body change for a later stone, not β).

The β value: substrate consistency. Every fork path uses the canonical primitive; future caller-body migrations can leverage `lifeline_r` if they want; the substrate's "every spawn has a lifeline" guarantee is honest.

---

## What NOT to do

- **DO NOT** change `run_in_fork`'s public signature. 11 callers depend on `F: FnOnce() + UnwindSafe` (no args).
- **DO NOT** add `lifeline_r` polling to `run_in_fork`'s body. That's a caller-by-caller behavior change; not β's scope.
- **DO NOT** touch the 2 other `libc::fork()` sites in fork.rs (lines 614 + 920 per arc 213 DESIGN — `fork_program_ast` + `fork_program_from_source`). Those are stone γ.
- **DO NOT** touch any of the 11 caller sites. Migration is internal-only.
- **DO NOT** change `eval_kernel_wait_child` or any other waitpid caller (stone δ).
- **DO NOT** modify error type returned to caller — `run_in_fork` still panics on fork failure / exit failure (its existing pattern).
- **DO NOT** mint new types, helpers, or modules. Pure refactor.

---

## The proof gate (workspace baseline preservation)

The 11 caller sites are the load-bearing tests. They must continue to pass.

### Pre-flight baseline (orchestrator-verified before this spawn)

The following tests pass on baseline (verified post-α at commit `5e43d7c`):
- `cargo test --release --test wat_harness_deps` — 3 run_in_fork sites
- `cargo test --release --test probe_shutdown_cascade_crossbeam` — 1 run_in_fork site
- `cargo test --release --test probe_shutdown_cascade_pipefd` — 1 run_in_fork site
- `cargo test --release -p wat-cli --test wat_cli` — 1 run_in_fork site
- `cargo test --release -p wat --lib` (subset touching runtime.rs fork tests) — 5 run_in_fork sites

### Verification protocol (you run, after the migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. `cargo test --release --test probe_pidfd_primitive 2>&1 | tail -10` — α's smoke probe still green (sanity)
3. Run each affected test binary:
   ```bash
   cargo test --release --test wat_harness_deps 2>&1 | tail -10
   cargo test --release --test probe_shutdown_cascade_crossbeam 2>&1 | tail -10
   cargo test --release --test probe_shutdown_cascade_pipefd 2>&1 | tail -10
   cargo test --release -p wat-cli --test wat_cli 2>&1 | tail -10
   ```
4. Run the lib tests that exercise run_in_fork (in src/runtime.rs — sonnet identifies the 5 sites via grep `run_in_fork` in src/runtime.rs and runs the surrounding `#[test]` fns):
   ```bash
   # Sonnet identifies test fn names via grep + module structure; runs:
   cargo test --release -p wat --lib <test_module>::<test_name> 2>&1 | tail
   ```
5. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-BETA-MIGRATE-RUN-IN-FORK.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **The migration touches any file outside `src/fork.rs`.** STOP. β is internal-only. Revert + report.

2. **Any test that passes on baseline FAILS post-migration.** STOP. Inscribe which test + the diagnostic. The migration broke something; do not investigate beyond what the test output tells you.

3. **You feel the urge to also migrate `fork_program_ast` OR `fork_program_from_source` (the other 2 libc::fork sites).** STOP. ONE stone. γ is the next stone, not this one.

4. **You feel the urge to add lifeline_r polling to run_in_fork's body.** STOP. Caller-body change is a separate concern; not β.

5. **The public signature of `run_in_fork` needs to change to make the migration work.** STOP — this means the design hypothesis is wrong. Inscribe the surface mismatch + return.

6. **cargo build FAILS.** STOP. Inscribe the error. If obvious syntactic fix (typo, missing import), correct once + retry. Otherwise STOP.

7. **You see a failing test that was ALREADY failing on baseline.** That's not β's concern. Note it in SCORE but do not investigate.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-BETA-MIGRATE-RUN-IN-FORK.md`:

1. Header: `# Arc 213 stone β — SCORE: migrate run_in_fork to spawn_lifelined`
2. Summary: internal refactor; signature unchanged; all 11 caller sites verified
3. File changes:
   - `src/fork.rs` — only `run_in_fork` body modified (lines 149-185 approximate; sonnet reports exact)
4. Verification: each of the 5 test binaries' pass count (e.g., "wat_harness_deps: 3/3 passed")
5. Notes: any subtleties (e.g., panic message format change from "waitpid failed" to "wait_status failed"; if a caller depends on the exact panic string, migration deltas matter)
6. Mode classification

---

## Constraints

- Edit `src/fork.rs` (only `run_in_fork` body; lines ~149-185)
- ZERO other code edits — no new types, no new helpers, no imports needed beyond what α already added
- ZERO git operations (orchestrator commits)
- Run cargo build + 5 test binaries

---

## Time prediction

20-40 min. Single-function refactor using primitives α just shipped. Most of the time is running + observing the 5 affected test binaries.

---

## Mode classification

- **Mode A:** migration shipped; cargo build clean; all 5 test binaries' affected tests still pass; α probe still green; SCORE written
- **Mode B (acceptable):** a test fails in a way that surfaces an honest signature/behavior mismatch sonnet can describe but not resolve in this stone (e.g., a caller asserts on the exact panic message string that changed); REVERT + inscribe + return
- **Mode C:** STOP rule broken (touched γ territory, changed signature, modified caller sites, scope-crept to add lifeline_r polling)

The substrate teaches; α minted the canonical primitive; β proves it in production usage; γ/δ/ε/ζ migrate the rest of the substrate.
