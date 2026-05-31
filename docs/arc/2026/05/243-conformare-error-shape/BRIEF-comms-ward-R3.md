# BRIEF — comms/ ward R3 (verification-recast wake: 2 doc-lies from Fix 1)

**Target:** `src/comms/process.rs` ONLY. **Goal:** close the 2 doc-comment lies the R2 Clone-removal left in its wake, so comms/ converges L1+L2=0 and earns its vigilatum stamp — the 6th and FINAL home.
**Mode:** sonnet writes; orchestrator re-casts + stamps. **Do NOT commit, push, or write a vigilatum stamp.** Leave the tree dirty (the R2 fixes are already on disk uncommitted — ADD these doc fixes to that same dirty tree).

## Hard rules
- Anchor: `pwd` MUST be `/home/watmin/work/holon/wat-rs`. Any `.claude/worktrees/` path → `cd` + `git -C`. Never a worktree.
- **Write scope: `src/comms/process.rs` ONLY.** Two doc-comment edits, NO code/logic change. ANY edit to logic or to another file → STOP and report.
- No runes — both are solvable doc-truth fixes → FIX.
- cargo, `-p wat`. (These are comment-only; build/test must stay exactly green: comms 32/0, lib 895/0/1.)

## Context: the wake
R2's Fix 1 removed the manual `Clone` impl from `process::Sender` (single-writer; frame-corruption now type-impossible — verified by struere). But two doc comments in `process.rs` still describe the deleted multi-writer/Clone world, AND Fix 1's own new `send()` comment over-claims atomicity. Two lenses caught them (intueri 2 L1; struere 1 L2 — the close() one). Both are doc-only.

## Fix A — `send()` comment over-claims a single syscall (intueri L1, process.rs:171-172)
Current (lines 171-172):
```rust
        // Frame: EDN bytes + '\n'. Single allocation; single write syscall
        // (single-writer endpoint — no concurrent interleave possible).
```
The `while written < framed.len()` loop immediately below (lines 180-204) calls `libc::write` repeatedly, retrying on short-write and `continue`-ing on EINTR. "Single write syscall" is false — a reader could believe any-size frame is written atomically in one call. The TRUE invariants worth stating: single allocation; a write LOOP (EINTR-retried, short-write-resumed); single-writer endpoint so there is no concurrent interleave.
**Fix — rewrite the comment to match the loop:**
```rust
        // Frame: EDN bytes + '\n'. One allocation, then a write loop
        // (short writes resumed; EINTR retried). Single-writer endpoint —
        // no concurrent interleave; writes ≤ PIPE_BUF (4096) are POSIX-atomic.
```
(Adjust wording as you see fit, but it MUST NOT claim a single syscall, and MUST reflect the retry loop. Comment only — do not touch the loop.)

## Fix B — `close()` doc describes nonexistent Sender clones (intueri L1 + struere L2, process.rs:210-214)
Current (lines 210-214):
```rust
    /// Signal end-of-stream from this sender. Consumes self so the
    /// endpoint is gone after close. Other cloned `Sender` handles
    /// (if any) remain valid. Peer receivers see EOF on their next
    /// recv only after ALL `Sender` clones close (the pipe's write
    /// reference count hits zero; kernel signals EOF on the read-end).
```
`process::Sender` no longer implements `Clone` (R2 Fix 1). No clone can exist; the "ALL Sender clones close" / "reference count" story is impossible state. A reader of `close()` forms the false belief that Sender fan-in exists.
**Fix — rewrite to the single-writer truth:**
```rust
    /// Signal end-of-stream from this sender. Consumes self so the
    /// endpoint is gone after close. This is the SOLE write-end —
    /// `process::Sender` is not `Clone` (single-writer by design, so
    /// oversized frames cannot interleave). The peer sees EOF immediately
    /// on its next recv: closing this sender drops the only write-end fd,
    /// the pipe's write reference count hits zero, and the kernel signals
    /// EOF on the read-end.
```
(Keep the infallible/Drop/double-close-compile-error lines that follow at 216-217 — they're correct. Comment only.)

## Gates
```
cargo build -p wat
cargo test -p wat --test comms        # MUST stay 32 passed / 0 failed (comment-only)
cargo test -p wat --lib               # MUST stay 895 / 0 / 1
```
A count change from comment-only edits is impossible — if one appears, something is wrong; STOP and report.

## Report (the SCORE)
1. Files touched (must be ONLY `src/comms/process.rs`).
2. Fix A + Fix B disposition with the new comment text.
3. Gate counts (comms, lib).
4. `git status --porcelain` — must be the R2 dirty set + your process.rs edit (no new files; no logic diff — confirm `git diff src/comms/process.rs` shows ONLY comment lines changed).
5. Any honest delta.

Do NOT commit. The orchestrator re-casts struere + intueri (the divergent spells) AND circumspicere (cast last, the surround) on the corrected tree; converged L1+L2=0 → hashless ISO8601 vigilatum stamp + ONE atomic ward commit = the 6th and FINAL warded home.
