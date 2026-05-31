# BRIEF — comms ward R5 (intueri wake: 4 stale `pipe(2)` comments after the pipe2 conversion)

**Target:** `src/comms/process.rs` + `src/io.rs` ONLY. **Goal:** close the 4 intueri L1s — comments still naming `pipe(2)` / `libc::pipe` after R4 converted the calls to `pipe2(O_CLOEXEC)`. Stale comment = active lie (intueri's worst class); here the lie is architecturally significant — it tells a reader the fd carries NO O_CLOEXEC when it does. COMMENT-ONLY.
**Mode:** sonnet writes; orchestrator verifies + re-casts intueri. Do NOT commit, push, or stamp. Add to the existing dirty tree (R2+R3+R4 are on disk uncommitted).

## Hard rules
- Anchor: `pwd` MUST be `/home/watmin/work/holon/wat-rs`. Any `.claude/worktrees/` path → cd + `git -C`. Never a worktree.
- Write scope: `src/comms/process.rs` + `src/io.rs` ONLY. COMMENT/DOC text only — ZERO code change. Any logic edit or other file → STOP and report.
- No runes — solvable doc-truth fixes → FIX.
- cargo `-p wat`. Comment-only → comms 32/0, lib 895/0/1 must hold exactly; a count change is impossible → if one appears, STOP.

## The 4 fixes (exact sites, verified by orchestrator read)

**Fix 1 — `src/comms/process.rs:6` (module doc):**
Current: `//! using `libc::pipe` for the transport and `io_uring` for the wake`
The transport now uses `libc::pipe2(O_CLOEXEC)` (pair() @ line 1046). A reader of the module doc forms a false belief the transport fds carry no O_CLOEXEC.
Fix: `//! using `libc::pipe2(O_CLOEXEC)` for the transport and `io_uring` for the wake`

**Fix 2 — `src/comms/process.rs:1050` (SAFETY comment, inside `pair()`):**
Current: `// SAFETY: pipe(2) returned two valid, owned fds. Wrap each as OwnedFd`
This CONTRADICTS the correct SAFETY block 9 lines above it (1041-1045 correctly names `libc::pipe2` + explains O_CLOEXEC). Two SAFETY comments in one function disagree on which syscall ran.
Fix: `// SAFETY: pipe2(O_CLOEXEC) returned two valid, owned fds. Wrap each as OwnedFd`

**Fix 3 — `src/io.rs:439` (section block comment):**
Current: `// a fresh `pipe(2)` pair (parent-side pipe ends). The`
`eval_kernel_pipe` calls `libc::pipe2(O_CLOEXEC)` (line ~1177). Reader believes no CLOEXEC on the `:wat::kernel::pipe` primitive's fds.
Fix: `// a fresh `pipe2(O_CLOEXEC)` pair (parent-side pipe ends). The`

**Fix 4 — `src/io.rs:1187` (SAFETY comment):**
Current: `// SAFETY: libc::pipe returned 0, so fds[0] (read) and fds[1]`
The actual call (line ~1177) is `libc::pipe2`; the error-format string just above (line 1183) already says `pipe2(2)`. The SAFETY argument names the wrong syscall.
Fix: `// SAFETY: libc::pipe2 returned 0, so fds[0] (read) and fds[1]`

## What is ALREADY correct (do NOT touch — verified clean by orchestrator)
- process.rs:1025 doc (`libc::pipe2(2)` with `O_CLOEXEC`) ✓
- process.rs:1041-1045 SAFETY (names pipe2, explains O_CLOEXEC + fork-without-exec) ✓
- io.rs:1161 doc (`libc::pipe2(2)` with `O_CLOEXEC`) ✓
- io.rs:1406 doc (`pipe2(O_CLOEXEC)` pair) ✓
- io.rs:1183 error string (`pipe2(2) syscall failed`) ✓
- fork.rs all clean (intueri ruled the module doc, fn doc, fn name honest)

## Gates
```
cargo build -p wat
cargo test -p wat --test comms      # 32/0 (comment-only)
cargo test -p wat --lib             # 895/0/1
```

## Report (SCORE)
1. Files touched (ONLY process.rs + io.rs).
2. Fix 1-4 with the new comment text.
3. Gate counts.
4. `git status --porcelain` + confirm `git diff` of the two files shows ONLY comment-line changes (no code).
5. Any honest delta.

Do NOT commit. Orchestrator re-casts intueri on the corrected tree; converged L1+L2=0 (with circumspicere also converged) → comms wards (6th/FINAL home).
