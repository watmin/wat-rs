# Arc 213 stone γ-3 — Migrate `eval_kernel_spawn_process` to `spawn_lifelined`

**Your ONE concern this spawn:** rebase `src/spawn_process.rs::eval_kernel_spawn_process` (line 91, fork call at line 167) on α's `spawn_lifelined` primitive. This is the **third and final fork site** in arc 213's γ phase. The most heavily-used spawn path (19 test binaries exercise it).

**Easier than γ-1** (lifeline already there per Phase 1B) **but wider blast radius than γ-2** (19 test binaries vs 5). Pattern fully proven by γ-1 + γ-2 — γ-3 is the third instance.

After γ-3 ships, ZERO bare `libc::fork()` calls remain in the substrate (other than `spawn_lifelined`'s internal clone3 implementation in α). Stones δ/ε/ζ then proceed.

---

## Audit-grounded scope (verified post γ-1 at commit `562efb9`)

### The site

`eval_kernel_spawn_process` at `src/spawn_process.rs:91-228`:

| Line | Current content |
|---|---|
| 110-136 | Evaluate program-arg to `Vec<WatAST>` (slice 6 contract) |
| 145 | Inherit caller's Config (COW; arc 031) |
| 149-151 | Three pipes (input/output/stderr) |
| 153-155 | Capture raw fds before fork |
| 162 | **Manual lifeline pipe creation: `let (lifeline_r, lifeline_w) = make_pipe(...)?;`** |
| 163 | `let lifeline_r_raw = lifeline_r.as_raw_fd();` |
| 167 | **Bare `libc::fork()`** |
| 179-191 | Child branch call (passes `lifeline_r` + `lifeline_w` to child) |
| 196-202 | Parent closes child-side stdio + drops lifeline_r |
| 204 | `ChildHandleInner::new(pid, Some(lifeline_w))` |
| 219-227 | Return Process struct (4-field: stdin/stdout/stderr + ProgramHandle) |

### The sister child branch

`spawn_process_child_branch` at `src/spawn_process.rs:277-289` (one caller — line 179):
- Signature has 11 params including **`lifeline_w: OwnedFd`** (line 288)
- At line 303: `drop(lifeline_w)` — Arc 170 Phase 1D fix preventing "child is its own lifeline keeper"
- Already calls `child_post_fork_init(lifeline_r_raw)` at line 337

### The KEY simplification γ-3 enables

α's `spawn_lifelined` ALREADY drops the child's inherited lifeline_w internally:

```rust
// In spawn_lifelined (src/fork.rs α):
if pid == 0 {
    unsafe { libc::setpgid(0, 0); }
    drop(lifeline_w);  // ← α already does this
    let outcome = std::panic::catch_unwind(|| child_body(lifeline_r_raw));
    ...
}
```

After γ-3:
- `spawn_lifelined` drops the inherited lifeline_w in child (before child_body runs)
- `spawn_process_child_branch` no longer needs `lifeline_w` param (its `drop(lifeline_w)` at line 303 becomes redundant + dead)
- **Signature simplification:** drop `lifeline_w: OwnedFd` from spawn_process_child_branch (one caller; mechanical)

This is the ONE sister-fn signature change γ-3 makes. γ-1 + γ-2 didn't simplify; γ-3 does because spawn_process is the cleanest expression of "redundant Phase 1D drop is now spawn_lifelined's job."

---

## What to migrate

### 1. `eval_kernel_spawn_process` body (lines ~145-204)

Mirror γ-1 + γ-2's pattern. Approximate shape:

```rust
let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

let (input_r, input_w) = make_pipe(":wat::kernel::spawn-process")?;
let (output_r, output_w) = make_pipe(":wat::kernel::spawn-process")?;
let (stderr_r, stderr_w) = make_pipe(":wat::kernel::spawn-process")?;

let input_r_raw = input_r.as_raw_fd();
let output_w_raw = output_w.as_raw_fd();
let stderr_w_raw = stderr_w.as_raw_fd();

// Convert OwnedFds to raw for closure capture (mirror γ-1/γ-2 pattern)
let input_r_fd = input_r.into_raw_fd();
let input_w_fd = input_w.into_raw_fd();
let output_r_fd = output_r.into_raw_fd();
let output_w_fd = output_w.into_raw_fd();
let stderr_r_fd = stderr_r.into_raw_fd();
let stderr_w_fd = stderr_w.into_raw_fd();

// spawn_lifelined creates the lifeline pipe + atomic clone3+pidfd
let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
    // Child reconstructs OwnedFds for child branch signature
    let input_r = unsafe { OwnedFd::from_raw_fd(input_r_fd) };
    let input_w = unsafe { OwnedFd::from_raw_fd(input_w_fd) };
    let output_r = unsafe { OwnedFd::from_raw_fd(output_r_fd) };
    let output_w = unsafe { OwnedFd::from_raw_fd(output_w_fd) };
    let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_fd) };
    let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_fd) };
    let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };

    spawn_process_child_branch(
        forms,
        inherit_config,
        input_r_raw,
        output_w_raw,
        stderr_w_raw,
        lifeline_r_raw,
        (input_r, input_w),
        (output_r, output_w),
        (stderr_r, stderr_w),
        lifeline_r,
        // NOTE: lifeline_w param dropped — spawn_lifelined handled the child-side drop
    );
}).map_err(|err| RuntimeError::MalformedForm {
    head: OP.into(),
    reason: format!("spawn_lifelined: {}", err),
    span: crate::span::Span::unknown(),
})?;

// Parent — reconstruct parent-side + close child-side
let input_w = unsafe { OwnedFd::from_raw_fd(input_w_fd) };
let output_r = unsafe { OwnedFd::from_raw_fd(output_r_fd) };
let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_fd) };
drop(unsafe { OwnedFd::from_raw_fd(input_r_fd) });
drop(unsafe { OwnedFd::from_raw_fd(output_w_fd) });
drop(unsafe { OwnedFd::from_raw_fd(stderr_w_fd) });

let lifeline_w = lifeline_writer.into_owned_fd();
let pid = pidfd.pid();

let handle = Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)));

let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(input_w));
let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(output_r));
let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

Ok(Value::Struct(Arc::new(StructValue {
    type_name: ":wat::kernel::Process".into(),
    fields: vec![
        Value::io__IOWriter(stdin_writer),
        Value::io__IOReader(stdout_reader),
        Value::io__IOReader(stderr_reader),
        Value::wat__kernel__ProgramHandle(Arc::new(ProgramHandleInner::Forked(handle))),
    ],
})))
```

### 2. `spawn_process_child_branch` signature simplification (line 277)

**Remove `lifeline_w: OwnedFd` param** (line 288) AND **remove `drop(lifeline_w)` call** (line 303) — both become redundant because spawn_lifelined handles the inherited-lifeline_w drop in the child's context BEFORE running the closure body.

The comment block at lines 296-302 documents WHY the drop was needed ("child is its own lifeline keeper" prevention). Update the comment to note that spawn_lifelined now owns this discipline (additive doc update).

ONE caller of spawn_process_child_branch (line 179 — the new closure body) — mechanical update.

### 3. Imports + Pidfd

Reference α's `Pidfd`/`LifelineWriter`/`spawn_lifelined` from `crate::fork::*` (same path γ-1 + γ-2 use). Add imports as needed.

---

## What NOT to do

- **DO NOT** touch `src/fork.rs` at all. γ-1 + γ-2 territory; both shipped.
- **DO NOT** modify `ChildHandleInner::wait_or_cached` or `Drop` (lines in fork.rs:217 + 236) — δ territory.
- **DO NOT** modify `eval_kernel_wait_child` — δ territory.
- **DO NOT** change the public signature of `eval_kernel_spawn_process` — wat dispatch arm.
- **DO NOT** modify `emit_structured_exit` / `emit_panics_to_stderr` — process_stdio module concerns.
- **DO NOT** introduce new types or helpers — γ-1's `LifelineWriter::into_owned_fd` already exists.
- **DO NOT** restructure the slice-6 program-form evaluation block (lines 110-136). Pure refactor of the fork mechanism only.

---

## The proof gate (workspace baseline preservation)

`:wat::kernel::spawn-process` is exercised by **19 test binaries**:

```
tests/arc112_scheme_probe.rs
tests/arc112_slice2b_process_send_recv.rs
tests/probe_closure_body_prelude_lift.rs
tests/probe_counter_actor_process_diag.rs
tests/probe_declaration_form_lift.rs
tests/probe_def_not_special.rs
tests/probe_lifeline_orphan_clean_via_fork_program.rs
tests/probe_lifeline_orphan_clean_via_substrate.rs
tests/probe_pdeathsig_diagnostic.rs
tests/probe_pdeathsig_kills_orphan_child.rs
tests/probe_run_hermetic_no_deadlock.rs
tests/probe_spawn_process_parent_type.rs
tests/probe_spawn_process_stdin.rs
tests/probe_spawn_process_stdio.rs
tests/wat_arc170_program_contracts.rs
tests/wat_arc170_stone_a_drain_and_join.rs
tests/wat_arc208_process_io_result.rs
tests/wat_process_peer_ipc_round_trip.rs
crates/wat-cli/tests/wat_cli.rs
```

Plus α regression: `tests/probe_pidfd_primitive.rs`.

**Total: 20 test binaries to verify pre + post.**

### Pre-flight baselines (orchestrator-verified before this spawn)

Orchestrator records baseline pass/fail counts for ALL 20 binaries in the spawn prompt before sonnet runs. ANY post-γ-3 regression vs baseline IS γ-3's fault.

**KNOWN pre-existing concerns to flag (NOT γ-3 regressions):**
- `probe_lifeline_pipe_proof` — pre-existing 1/100 flake (arc 213 ε territory) — orthogonal; not in the 20-binary set above
- `probe_pdeathsig_diagnostic` + `probe_pdeathsig_kills_orphan_child` — these test the RETIRED PDEATHSIG mechanism (Phase 1B retired it in favor of lifeline pipe). May be `#[ignore]`'d, may be artifacts. Orchestrator records actual baseline state in spawn prompt.

### Verification protocol (post-migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. Re-run each of the 20 cargo test commands; record pass counts
3. Compare each binary's post-count to its pre-count (recorded in spawn prompt)
4. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-3-SPAWN-PROCESS.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **You touch `src/fork.rs`.** γ-1 + γ-2 territory; both shipped. γ-3 stays in spawn_process.rs.

2. **You modify ChildHandleInner / wait_or_cached / Drop / eval_kernel_wait_child.** δ territory. STOP.

3. **A test that PASSED on baseline FAILS post-migration.** STOP. Inscribe which test + diagnostic + your hypothesis.

4. **A test that was already failing/ignored stays in same state.** That's NOT γ-3's concern — note in SCORE but do not investigate.

5. **cargo build FAILS.** STOP. Inscribe error. One syntactic-fix retry allowed.

6. **The spawn_process_child_branch signature simplification has a non-obvious complication.** STOP. The simplification is one-caller mechanical; if it surfaces a non-mechanical issue, REVERT the sig change + inscribe.

7. **You feel the urge to also migrate ChildHandleInner to hold a Pidfd.** STOP — δ territory.

8. **You feel the urge to also retire `emit_panics_to_stderr` or other process_stdio plumbing.** Out of scope; module-architecture concerns.

9. **`Arc<dyn SomeTrait>` / closure UnwindSafe issue** (γ-2 risk 2). Document resolution in SCORE; do NOT silently AssertUnwindSafe-wrap if the deeper issue is the trait bound.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-3-SPAWN-PROCESS.md`:

1. Header: `# Arc 213 stone γ-3 — SCORE: migrate eval_kernel_spawn_process to spawn_lifelined`
2. Summary: site migrated; spawn_process_child_branch signature simplified; substrate fork-canonicalization complete (γ-1+γ-2+γ-3 ship the 3 sites)
3. File changes:
   - `src/spawn_process.rs` — eval_kernel_spawn_process body (lines ~145-204) + spawn_process_child_branch signature + Phase 1D comment update
4. Verification: pre/post pass counts for each of 20 test binaries
5. Notes:
   - Signature simplification (removed lifeline_w from spawn_process_child_branch)
   - Any closure-capture / UnwindSafe subtleties
   - Comparison to γ-1 + γ-2 precedents
6. Mode classification

---

## Constraints

- Edit `src/spawn_process.rs` ONLY (eval_kernel_spawn_process body + spawn_process_child_branch signature + 1 comment block)
- ZERO other code edits — γ-1's `LifelineWriter::into_owned_fd` already exists; no new types/helpers
- ZERO git operations (orchestrator commits)
- Run cargo build + 20 test binaries

---

## Time prediction

45-75 min. Larger than γ-2 because:
- 20 test binaries to verify (vs γ-2's 5)
- Signature simplification on spawn_process_child_branch (γ-2 didn't touch its sister fn)
- spawn_process is the heaviest spawn path — any subtle interaction has broader impact

Smaller than γ-1 because pattern fully established by α/β/γ-1/γ-2.

---

## Mode classification

- **Mode A:** site migrated; spawn_process_child_branch signature simplified; cargo build clean; all 20 baselines preserved; SCORE written; mode-classified
- **Mode B (acceptable):**
  - Closure-capture UnwindSafe / signature simplification has a complication you can describe but not resolve in this stone; REVERT + inscribe + return
  - A test fails in a way that surfaces a behavior mismatch (e.g., timing-sensitive probe affected by setpgid change — already validated safe by γ-1 + γ-2 + β)
- **Mode C:** STOP rule broken (touched γ-1/γ-2/δ territory, changed signature, modified caller sites in fork.rs)

After γ-3 ships, the substrate has **ZERO bare libc::fork() calls** (other than spawn_lifelined's clone3 internally). γ phase complete. δ/ε/ζ proceed.
