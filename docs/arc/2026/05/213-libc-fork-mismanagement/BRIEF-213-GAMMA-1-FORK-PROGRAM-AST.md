# Arc 213 stone γ-1 — Migrate `eval_kernel_fork_program_ast` to `spawn_lifelined`

**Your ONE concern this spawn:** rebase `src/fork.rs::eval_kernel_fork_program_ast` (line 532, fork call at line 600) on α's `spawn_lifelined` primitive. This site is the **load-bearing one** of γ — it currently does `ChildHandleInner::new(pid, None)` (line 631), bypassing the lifeline mechanism entirely. The orphan-leak class arc 213 named exists here. γ-1 closes it.

γ-2 (fork_program_from_source — line 906; has lifeline already) and γ-3 (spawn_process.rs — line 167; has lifeline already) ship in separate spawns post per-stone trust gate. ONE site here.

---

## Audit-grounded scope (verified pre-spawn)

### The site

`eval_kernel_fork_program_ast` at `src/fork.rs:532-672` (approximately):
- Line 600: `let pid = unsafe { libc::fork() };` — bare fork; no lifeline pipe created
- Line 611-623: child branch calls `child_branch(forms, inherit_config, stdin_r_raw, stdout_w_raw, stderr_w_raw, stdin_pair, stdout_pair, stderr_pair)`
- Line 631: `let handle = Arc::new(ChildHandleInner::new(pid, None));` — **None** = no lifeline; this is the gap
- Returns `Value::Struct(Arc<StructValue>)` wrapping `:wat::kernel::Process`

### The child branch

`child_branch` at `src/fork.rs:674` (private fn; ONLY one caller — line 613):
- Returns `!` (never returns; always _exit via child_branch internals)
- Current signature: 8 params (forms, inherit_config, 3 raw fds, 3 OwnedFd pairs)
- **Does NOT call `child_post_fork_init`** — it's the orphan-leak surface (no shutdown-worker registration)

### Sister fn precedent

`fork_program_from_source` (line 906) + `child_branch_from_source` (line 1072) already have the canonical pattern:
- Lifeline pipe created (line 895): `let (lifeline_r, lifeline_w) = make_pipe(OP)?;`
- Child branch receives `lifeline_r_raw + lifeline_r`
- Child branch calls `child_post_fork_init(lifeline_r_raw)` at line 1111
- Parent retains `lifeline_w` via `ChildHandleInner::new(pid, Some(lifeline_w))` at line 946

γ-1's job: mirror this canonical pattern in `eval_kernel_fork_program_ast` + extend `child_branch` to accept + register lifeline.

---

## What to migrate

### 1. `eval_kernel_fork_program_ast` (lines ~595-635)

**Replace** the bare fork + parent setup block (approximately lines 595-631):
```rust
// SAFETY: fork is legal at this call site...
let pid = unsafe { libc::fork() };
if pid < 0 { ... error path ... }

if pid == 0 {
    child_branch(forms, inherit_config, stdin_r_raw, stdout_w_raw, stderr_w_raw,
                 stdin_pair, stdout_pair, stderr_pair);
}

// Parent setup
drop(stdin_r); drop(stdout_w); drop(stderr_w);
let handle = Arc::new(ChildHandleInner::new(pid, None));  // ← the gap
```

**With** spawn_lifelined-based pattern (approximate shape):
```rust
// Move parent-side closes for stdio AHEAD of fork (so they happen
// only in parent — closures capture by move; child gets its own copies).
// Actually: simpler — keep current order; spawn_lifelined's closure
// runs only in child; parent-side drops still happen after spawn_lifelined
// returns. Sonnet decides exact arrangement.

let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw| {
    // Child branch — same as current child_branch but with lifeline_r_raw
    // wired so child_post_fork_init can register it with shutdown worker.
    child_branch(forms, inherit_config,
                 stdin_r_raw, stdout_w_raw, stderr_w_raw,
                 lifeline_r_raw,            // ← NEW: passed in
                 stdin_pair, stdout_pair, stderr_pair);
}).map_err(|err| RuntimeError::MalformedForm {
    head: OP.into(),
    reason: format!("spawn_lifelined: {}", err),
    span: crate::span::Span::unknown(),
})?;

// Parent-side close existing OwnedFds (child has its own copies via fork inheritance)
drop(stdin_r); drop(stdout_w); drop(stderr_w);

// ChildHandleInner gets BOTH the pid AND the lifeline_w (closes the gap).
// pidfd.pid() retrieves the pid; lifeline_writer is the OwnedFd-bearing
// LifelineWriter type from arc 213 α — pass its inner OwnedFd to
// ChildHandleInner::new.
let pid = pidfd.pid();
let lifeline_w = ???;  // sonnet: extract OwnedFd from LifelineWriter
let handle = Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)));
```

**KEY DESIGN QUESTION sonnet decides:**
- `LifelineWriter` (arc 213 α) wraps an `OwnedFd` privately. ChildHandleInner needs `OwnedFd` for its `lifeline_w` field. Options:
  - (a) `LifelineWriter::into_owned_fd(self) -> OwnedFd` — add a consumption method to LifelineWriter
  - (b) `LifelineWriter` exposes `as_fd() -> &OwnedFd` — but ChildHandleInner needs owned, not borrowed
  - (c) ChildHandleInner field type changes to `Option<LifelineWriter>` instead of `Option<OwnedFd>` — cleaner type-system; LifelineWriter::Drop closes the fd

Option (c) is the most honest — ChildHandleInner stores the canonical LifelineWriter type. But that's a wider blast radius (ChildHandleInner has other callers). Per scope discipline:
- If wider blast radius >5 sites: pick (a), add `into_owned_fd` method
- If 5 or fewer sites: pick (c), migrate type to LifelineWriter

Sonnet greps ChildHandleInner usage; picks based on count; documents the choice in SCORE.

**Pidfd usage:** for γ-1, `pidfd` is only used to retrieve `pid` via `pidfd.pid()`. The actual waitpid/kill calls go through ChildHandleInner (legacy path; stones δ migrates those). So `pidfd` is dropped at end of `eval_kernel_fork_program_ast` — that's intentional and fine for γ-1; δ will migrate ChildHandleInner to hold a Pidfd.

### 2. `child_branch` (lines 674-...)

**Extend signature** to accept lifeline_r_raw:
```rust
fn child_branch(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,                  // ← NEW
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
) -> ! {
    // ... existing parent-side drops + dup2 setup ...

    // NEW: register lifeline_r with shutdown worker before any user-eval runs.
    // Mirror sister fn child_branch_from_source line 1111.
    child_post_fork_init(lifeline_r_raw);

    // ... existing wat-eval body ...
}
```

`child_post_fork_init` is at `src/fork.rs:474` (pub(crate)); it's the canonical Phase 3 helper. Sonnet inserts the call at the equivalent spot in child_branch to what child_branch_from_source does at line 1111.

### What spawn_lifelined gives the child by default

spawn_lifelined (arc 213 α) does:
1. clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND
2. lifeline pipe created pre-fork
3. setpgid(0, 0) in child (becomes own pgrp leader)
4. catch_unwind wraps body; _exit(0) on Ok, _exit(1) on Err
5. Drops the child's inherited lifeline_w copy (parent-only handle)

This means:
- child_branch returns `!` already (always _exit) — catch_unwind never sees Ok return
- If child_branch panics, spawn_lifelined's catch_unwind catches + exits 1 (matches existing behavior; child_branch's existing panic-exit path becomes redundant but harmless)
- setpgid(0,0) is NEW — child of fork-program-ast becomes own pgrp leader (matches sister fn behavior in spawn_process and run_in_fork post-β)

---

## What NOT to do

- **DO NOT** migrate `fork_program_from_source` (line 906). That's γ-2.
- **DO NOT** migrate `spawn_process.rs::libc::fork()` (line 167). That's γ-3.
- **DO NOT** touch `child_branch_from_source` (line 1072). Sister fn; γ-2's territory.
- **DO NOT** migrate `ChildHandleInner::wait_or_cached` or `Drop` (lines 217 + 236) — those still use libc::waitpid/kill. Stone δ migrates them.
- **DO NOT** modify `eval_kernel_wait_child` (line 289). Stone δ.
- **DO NOT** change the public signature of `eval_kernel_fork_program_ast` — wat dispatch arm; signature contract fixed.
- **DO NOT** introduce new types; reuse what α/β shipped.

---

## The proof gate (workspace baseline preservation)

The wat-level `:wat::kernel::fork-program-ast` is exercised by 5 test binaries:

| Test binary | Owner |
|---|---|
| `tests/arc112_slice2b_process_send_recv.rs` | arc 112 Process send/recv suite |
| `tests/wat_arc170_program_contracts.rs` | arc 170 program-entry-points contracts |
| `tests/probe_run_hermetic_ast_stdout_capture.rs` | run-hermetic stdout capture |
| `tests/probe_run_hermetic_no_deadlock.rs` | run-hermetic deadlock probe |
| `crates/wat-cli/tests/wat_cli.rs` | wat-cli end-to-end |

### Pre-flight baselines (orchestrator-verified at commit `2f10dbd`)

ALL of these are GREEN on baseline. ANY regression is γ-1's fault.

| Binary | Pre-γ-1 baseline |
|---|---|
| `cargo test --release --test arc112_slice2b_process_send_recv` | **1/1 PASS** |
| `cargo test --release --test wat_arc170_program_contracts` | **24/24 PASS** (t6 fixed by arc 212-α; t14 fixed by arc 211d; both now green) |
| `cargo test --release --test probe_run_hermetic_ast_stdout_capture` | **1/1 PASS** |
| `cargo test --release --test probe_run_hermetic_no_deadlock` | **2/2 PASS** |
| `cargo test --release -p wat-cli --test wat_cli` | **15/15 PASS** |
| `cargo test --release --test probe_pidfd_primitive` (α regression) | **2/2 PASS** |

**Total: 45/45 GREEN pre-γ-1.** Post-migration must hold the same counts.

### Verification protocol (post-migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. Re-run each of the 6 cargo test commands above
3. Compare post-counts to pre-counts (recorded in step 0 of your work)
4. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-1-FORK-PROGRAM-AST.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **You touch `fork_program_from_source` (line 906) OR `child_branch_from_source` (line 1072).** STOP — γ-2 territory.

2. **You touch `spawn_process.rs::libc::fork()` (line 167) OR anything else in spawn_process.rs.** STOP — γ-3 territory.

3. **You modify ChildHandleInner's `wait_or_cached` OR `Drop` impl.** STOP — δ territory; those still use libc::waitpid + kill by design until δ migrates them.

4. **A test that PASSED on baseline FAILS post-migration.** STOP. Inscribe which test + the diagnostic + your hypothesis. Do not iterate beyond what the test output tells you.

5. **You see a failing test.** **ALL 45 baselines are GREEN pre-γ-1.** Any failure post-γ-1 IS γ-1's regression. STOP. Inscribe the test + diagnostic + your hypothesis.

6. **cargo build FAILS.** STOP. Inscribe the error. If obvious syntactic fix (typo, missing import), correct once + retry. Otherwise STOP.

7. **The ChildHandleInner type-design choice (option (a) `into_owned_fd` vs option (c) field-type-change) has a wider blast radius than you expected.** STOP. Inscribe what you found + your preferred option. Orchestrator decides.

8. **You feel the urge to also migrate ChildHandleInner to hold a Pidfd.** STOP — δ territory; γ-1 keeps ChildHandleInner's existing `pid: pid_t` field.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-1-FORK-PROGRAM-AST.md`:

1. Header: `# Arc 213 stone γ-1 — SCORE: migrate eval_kernel_fork_program_ast to spawn_lifelined`
2. Summary: site migrated; child_branch extended with lifeline_r_raw; lifeline gap closed
3. File changes:
   - `src/fork.rs` — eval_kernel_fork_program_ast (lines ~595-635) + child_branch signature extension + child_post_fork_init call (sonnet reports exact lines)
4. Verification: pre/post pass counts for each of the 6 test binaries (e.g., "wat-cli wat_cli: 15/15 → 15/15 PASS")
5. ChildHandleInner type-design choice: which option (a/b/c) selected + why + blast radius
6. Notes: any subtleties (e.g., dup2 + close_inherited_fds_above_stdio ordering relative to child_post_fork_init; whether catch_unwind double-wraps with spawn_lifelined's catch_unwind)
7. Mode classification

---

## Constraints

- Edit `src/fork.rs` ONLY:
  - `eval_kernel_fork_program_ast` body (lines ~595-635 approximate)
  - `child_branch` signature + call to `child_post_fork_init` (lines ~674-...)
  - Possibly `ChildHandleInner` if option (c) selected (sonnet reports)
- ZERO other code edits
- ZERO git operations (orchestrator commits)
- Run cargo build + 6 test binaries listed above

---

## Time prediction

45-75 min. Larger than β because:
- child_branch signature extends (1 caller; mechanical)
- LifelineWriter ↔ OwnedFd plumbing needs a small design choice
- 6 test binaries to verify (vs β's 5)
- Wat-eval child branch has more moving parts than run_in_fork's pure-Rust body

---

## Mode classification

- **Mode A:** site migrated; child_branch extended; lifeline gap closed (ChildHandleInner now Some(lifeline_w)); cargo build clean; all 6 baselines preserved (pre-existing failures unchanged; previously-passing still pass); SCORE written; mode-classified
- **Mode B (acceptable):**
  - The LifelineWriter ↔ OwnedFd plumbing is ambiguous (sonnet describes both options + picks honestly + REVERTS if can't verify)
  - A test fails in a way sonnet describes but can't resolve in this stone (signature contract mismatch, dup2 ordering, etc.); REVERT + inscribe + return
- **Mode C:** STOP rule broken (touched γ-2/γ-3/δ territory, changed public signature, scope-crept to migrate ChildHandleInner, modified unrelated tests)

The substrate teaches; α minted the canonical primitive; β proved it on run_in_fork; γ-1 closes the actual orphan-leak gap (no-lifeline) by using α's primitive + mirroring sister fn's canonical pattern.
