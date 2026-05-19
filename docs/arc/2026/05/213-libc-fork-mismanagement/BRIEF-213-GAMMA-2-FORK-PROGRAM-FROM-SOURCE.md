# Arc 213 stone γ-2 — Migrate `fork_program_from_source` to `spawn_lifelined`

**Your ONE concern this spawn:** rebase `src/fork.rs::fork_program_from_source` (line 933, fork call at line 968) on α's `spawn_lifelined` primitive. This site **already has a lifeline** (Phase 1C wired it). γ-2 is canonicalization — replace the manual lifeline-pipe creation + bare libc::fork with the canonical primitive that does both atomically.

**Easier than γ-1:** child_branch_from_source already accepts `lifeline_r_raw + lifeline_r` (line 1134); ChildHandleInner already gets `Some(lifeline_w)` (line 1008); `LifelineWriter::into_owned_fd()` already exists (γ-1 added). The migration is mechanical pattern-matching on γ-1's precedent.

γ-3 (`src/spawn_process.rs:167`) is the remaining canonicalization; not γ-2's concern.

---

## Audit-grounded scope (verified post γ-1 at commit `33d8f2c`)

### The site

`fork_program_from_source` at `src/fork.rs:933-1013` (approximately):

| Line | Current content |
|---|---|
| 943-945 | 3 stdio pipes (`make_pipe`) |
| 947-949 | Capture raw fds before fork |
| 957 | **Manual lifeline pipe creation: `let (lifeline_r, lifeline_w) = make_pipe(OP)?;`** |
| 958 | `let lifeline_r_raw = lifeline_r.as_raw_fd();` |
| 968 | **Bare `libc::fork()`** |
| 981-994 | Child branch call (passes `lifeline_r_raw + lifeline_r`) |
| 998-1001 | Parent closes child-side stdio fds |
| 1005 | Parent drops lifeline_r (its copy) |
| 1008 | `ChildHandleInner::new(pid, Some(lifeline_w))` |

### What changes vs γ-1

| Aspect | γ-1 (eval_kernel_fork_program_ast) | γ-2 (fork_program_from_source) |
|---|---|---|
| Lifeline status pre-stone | None (the gap) | Already Some(lifeline_w) |
| child_branch signature change | Extended +2 params | UNCHANGED — already takes lifeline_r/_raw |
| ChildHandleInner::new(pid, ...) | None → Some | Stays Some |
| LifelineWriter::into_owned_fd | Added by γ-1 | Reused as-is |
| Sister fn precedent | Followed γ-1 itself | Mirrors γ-1's `eval_kernel_fork_program_ast` migration shape |

γ-2's migration is **strictly easier**: no signature extensions, no constructor changes. Just swap the fork mechanism + drop redundant manual lifeline plumbing.

---

## What to migrate

### Single concern: `fork_program_from_source` body (lines ~943-1013)

**Remove:**
- Line 957: `let (lifeline_r, lifeline_w) = make_pipe(OP)?;` — spawn_lifelined creates this
- Line 958: `let lifeline_r_raw = lifeline_r.as_raw_fd();` — spawn_lifelined provides via closure arg
- Line 1005: `drop(lifeline_r);` — spawn_lifelined handles the parent-side drop (LifelineWriter is what parent retains)
- Lines 968-995 (the bare-libc::fork block + child branch call inside `if pid == 0`)

**Replace with** spawn_lifelined-based pattern (approximate shape, mirroring γ-1's eval_kernel_fork_program_ast at lines ~584-693):

```rust
// stdio raw fds captured pre-closure (kernel inherits across clone3)
let stdin_r_raw = stdin_r.as_raw_fd();
let stdout_w_raw = stdout_w.as_raw_fd();
let stderr_w_raw = stderr_w.as_raw_fd();

// Convert OwnedFds to raw for closure capture
let stdin_r_fd = stdin_r.into_raw_fd();
let stdin_w_fd = stdin_w.into_raw_fd();
let stdout_r_fd = stdout_r.into_raw_fd();
let stdout_w_fd = stdout_w.into_raw_fd();
let stderr_r_fd = stderr_r.into_raw_fd();
let stderr_w_fd = stderr_w.into_raw_fd();

let owned_source = source.to_string();
let owned_canonical = canonical.map(|s| s.to_string());

// spawn_lifelined creates the lifeline pipe + atomic clone3+pidfd
let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
    // Child reconstructs OwnedFds for child_branch_from_source signature
    let stdin_r = unsafe { OwnedFd::from_raw_fd(stdin_r_fd) };
    let stdin_w = unsafe { OwnedFd::from_raw_fd(stdin_w_fd) };
    let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_fd) };
    let stdout_w = unsafe { OwnedFd::from_raw_fd(stdout_w_fd) };
    let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_fd) };
    let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_fd) };
    let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };

    child_branch_from_source(
        owned_source,
        owned_canonical,
        loader,
        argv,
        stdin_r_raw,
        stdout_w_raw,
        stderr_w_raw,
        lifeline_r_raw,
        (stdin_r, stdin_w),
        (stdout_r, stdout_w),
        (stderr_r, stderr_w),
        lifeline_r,
    );
}).map_err(|err| RuntimeError::MalformedForm {
    head: OP.into(),
    reason: format!("spawn_lifelined: {}", err),
    span: crate::span::Span::unknown(),
})?;

// Parent — reconstruct + close child-side fds, retain parent-side
let stdin_w = unsafe { OwnedFd::from_raw_fd(stdin_w_fd) };  // parent writes
let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_fd) }; // parent reads
let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_fd) }; // parent reads
// Close child-side (parent's copies — kernel duplicated them across clone3)
drop(unsafe { OwnedFd::from_raw_fd(stdin_r_fd) });
drop(unsafe { OwnedFd::from_raw_fd(stdout_w_fd) });
drop(unsafe { OwnedFd::from_raw_fd(stderr_w_fd) });

let lifeline_w = lifeline_writer.into_owned_fd();
let pid = pidfd.pid();

Ok(ForkedProgramHandles {
    child_handle: Arc::new(ChildHandleInner::new(pid, Some(lifeline_w))),
    stdin_w,
    stdout_r,
    stderr_r,
})
```

Sonnet adapts the exact shape to match γ-1's precedent (the closure-side OwnedFd reconstruction + parent-side reconstruction + child-side closure). The PATTERN is what γ-1 established; γ-2 mirrors.

### child_branch_from_source — DO NOT TOUCH

Already has lifeline_r + lifeline_r_raw params (line 1134); already calls `child_post_fork_init(lifeline_r_raw)` (mirror per γ-1 precedent at child_branch). γ-2 does NOT modify child_branch_from_source.

### Pidfd usage

`pidfd` is used only to retrieve `pid` via `pidfd.pid()`. The Pidfd is dropped at function end — that's intentional for γ-2; δ migrates ChildHandleInner to hold a Pidfd properly.

---

## What NOT to do

- **DO NOT** touch `eval_kernel_fork_program_ast` (line 584). γ-1 territory; already shipped.
- **DO NOT** touch `child_branch` (line ~684). γ-1 territory.
- **DO NOT** touch `spawn_process.rs::libc::fork()` (line 167). γ-3 territory.
- **DO NOT** touch `child_branch_from_source` (line 1134). Already correct; no changes needed.
- **DO NOT** modify `ChildHandleInner::wait_or_cached` or `Drop` (lines 217 + 236) — stone δ.
- **DO NOT** modify `eval_kernel_wait_child` (line 289). Stone δ.
- **DO NOT** change the public signature of `fork_program_from_source` — wat-cli + eval_kernel_fork_program depend on it.
- **DO NOT** introduce new types or helpers; γ-1's `LifelineWriter::into_owned_fd` already exists.

---

## The proof gate (workspace baseline preservation)

`fork_program_from_source` is exercised by:

| Test binary | Purpose |
|---|---|
| `tests/probe_lifeline_orphan_clean_via_fork_program.rs` | Tests lifeline mechanism via fork-program specifically (HIGH-RELEVANCE) |
| `tests/wat_arc170_stone_b_walker_collapse.rs` | Walker collapse exercises fork-program |
| `tests/wat_arc170_program_contracts.rs` | Includes fork-program path tests |
| `crates/wat-cli/tests/wat_cli.rs` | wat-cli IS the primary user of fork_program_from_source |

Plus α regression: `tests/probe_pidfd_primitive.rs`.

### Pre-flight baselines (orchestrator-verified post γ-1 at commit `33d8f2c`)

**ALL of these are GREEN on baseline.** ANY failure post-γ-2 IS regression.

| Binary | Pre-γ-2 baseline |
|---|---|
| `cargo test --release --test probe_pidfd_primitive` (α regression) | **2/2 PASS** |
| `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` | (orchestrator verifies pre-spawn) |
| `cargo test --release --test wat_arc170_stone_b_walker_collapse` | (orchestrator verifies pre-spawn) |
| `cargo test --release --test wat_arc170_program_contracts` | **24/24 PASS** (γ-1 verified) |
| `cargo test --release -p wat-cli --test wat_cli` | **15/15 PASS** (γ-1 verified) |

(Orchestrator records the 2 unknown counts in the spawn prompt + scorecard.)

### Verification protocol (post-migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. Re-run each cargo test command above; record pass counts
3. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-2-FORK-PROGRAM-FROM-SOURCE.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **You touch `eval_kernel_fork_program_ast` OR `child_branch`.** γ-1 territory. STOP.

2. **You touch `child_branch_from_source`.** Already correct. STOP — γ-2 does NOT modify it.

3. **You touch `spawn_process.rs::libc::fork()` (line 167) OR anything else in spawn_process.rs.** γ-3 territory. STOP.

4. **You modify ChildHandleInner's `wait_or_cached` OR `Drop` impl.** STOP — δ territory.

5. **A test that PASSED on baseline FAILS post-migration.** STOP. Inscribe which test + diagnostic.

6. **cargo build FAILS.** STOP. Inscribe error. One syntactic-fix retry allowed.

7. **You feel the urge to also migrate ChildHandleInner to hold a Pidfd.** STOP — δ territory.

8. **`probe_lifeline_orphan_clean_via_fork_program` was failing on baseline.** If yes (orchestrator reports actual count in the spawn prompt), note pre-existing — post-γ-2 may or may not still fail; do NOT investigate beyond confirming pre-existing status.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-2-FORK-PROGRAM-FROM-SOURCE.md`:

1. Header: `# Arc 213 stone γ-2 — SCORE: migrate fork_program_from_source to spawn_lifelined`
2. Summary: site migrated; manual lifeline-pipe creation removed; canonical primitive owns lifeline
3. File changes:
   - `src/fork.rs` — `fork_program_from_source` body (lines ~933-1013 approximate; sonnet reports exact)
4. Verification: pre/post pass counts for each of the 5 test binaries listed
5. Notes: any subtleties (e.g., stdio fd ownership pattern, redundant child-side close after spawn_lifelined returns, comparison to γ-1's precedent)
6. Mode classification

---

## Constraints

- Edit `src/fork.rs` ONLY (only `fork_program_from_source` body; lines ~933-1013)
- ZERO other code edits — γ-1's `LifelineWriter::into_owned_fd` already exists; child_branch_from_source already correct
- ZERO git operations (orchestrator commits)
- Run cargo build + 5 test binaries

---

## Time prediction

30-50 min. Smaller than γ-1 because:
- No child_branch signature extension (sister fn already correct)
- No ChildHandleInner::new change (already Some(lifeline_w))
- No new type/helper (γ-1 added LifelineWriter::into_owned_fd)
- Pattern fully established by γ-1's precedent

---

## Mode classification

- **Mode A:** site migrated; manual lifeline plumbing removed; cargo build clean; all 5 baselines preserved; SCORE written; mode-classified
- **Mode B (acceptable):**
  - Pattern adaptation has a non-obvious complication sonnet describes but can't resolve (e.g., closure-capture lifetime; OwnedFd reconstruction ordering); REVERT + inscribe + return
  - A test fails in a way that surfaces a real behavior mismatch (e.g., wat-cli stdio proxy timing sensitive to setpgid)
- **Mode C:** STOP rule broken (touched γ-1/γ-3/δ, changed public signature, modified caller sites, child_branch_from_source touched)

The substrate teaches; α minted the primitive; β proved it on run_in_fork; γ-1 closed the no-lifeline gap; γ-2 canonicalizes the second of three fork sites.
