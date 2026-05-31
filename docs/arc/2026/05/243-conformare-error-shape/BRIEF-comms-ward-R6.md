# BRIEF — comms ward R6 (circumspicere wake: stale fork.rs docs naming a retired type + wrong syscall)

**Target:** `src/fork.rs` ONLY. **Goal:** close circumspicere's 2 claim-vs-code L1s (widened by orchestrator grep to the true site count) — fork.rs docs ship `:wat::kernel::ForkedChild` (a type RETIRED arc 112, 2026-04-30) and `libc::fork(2)` (the real syscall is `clone3` via `spawn_lifelined`, arc 213). FM 14 surface-retirement leftovers — code moved, docs didn't. COMMENT-ONLY (doc comments).
**Mode:** sonnet writes; orchestrator verifies + re-casts. Do NOT commit, push, or stamp. Add to the existing dirty tree (R2-R5 on disk uncommitted).

## Hard rules
- Anchor: `pwd` MUST be `/home/watmin/work/holon/wat-rs`. Any `.claude/worktrees/` path → cd + `git -C`. Never a worktree.
- Write scope: `src/fork.rs` ONLY. DOC-COMMENT text only — ZERO code change. Any logic edit or other file → STOP and report.
- No runes — doc-truth fixes → FIX.
- cargo `-p wat`. Comment-only → all suites must hold exactly (lib 895/0/1, comms 32/0); a count change is impossible → if one appears, STOP.

## Code-truth (orchestrator-verified by read — what the docs MUST say)
- The fork primitives RETURN `:wat::kernel::Process` (verified: `type_name: ":wat::kernel::Process"` at fork.rs:729 + 1224; comments at 724 "fork-program-ast returns the same :wat::kernel::Process" + 1224 "fork-program returns Process<I,O>").
- `ForkedChild` was RETIRED 2026-04-30 arc 112 (src/types.rs:997 "`:wat::kernel::ForkedChild` RETIRED 2026-04-30 (arc 112)"). It is a DEAD type name — no doc should present it as the current return.
- The real syscall is `clone3` via `spawn_lifelined` (fork.rs:160 calls it; 155/639/1005 say "instead of bare libc::fork()"). Bare `libc::fork(2)` is NOT what runs.

## Fix A — `ForkedChild` → `Process` at all 6 sites (the retired-type leftover)
Sites (orchestrator `git grep` verified — all 6, do not miss one; a partial fix re-creates the half-purge that left this debt):
- **fork.rs:8** (module doc): `//! \`:wat::kernel::ForkedChild\` struct holding the child's pid plus` → name `:wat::kernel::Process` (the parent receives a `:wat::kernel::Process` struct holding the child handle + the three parent-side pipe ends). Adjust the surrounding sentence (lines 7-9) so it reads true.
- **fork.rs:559** (fn doc return-type for `eval_kernel_fork_program_ast`): `/// :wat::kernel::ForkedChild\`.` → `/// :wat::kernel::Process\`.`
- **fork.rs:564** (same fn doc body): `/// gets the ForkedChild struct (handle + stdin writer + stdout` → `/// gets the Process struct (...)`.
- **fork.rs:947** (fn doc for `fork_program_from_source`/helper): `/// \`:wat::kernel::ForkedChild\` struct value.` → `/// \`:wat::kernel::Process\` struct value.` NOTE: this doc ALREADY self-contradicts — line 950 says "wraps them into the Process struct". Fixing 947 resolves the contradiction.
- **fork.rs:1124** (fn doc return-type for `eval_kernel_fork_program`): `/// -> :wat::kernel::ForkedChild\`.` → `/// -> :wat::kernel::Process\`.`
- **fork.rs:1128** (same fn doc body): `/// \`:wat::kernel::ForkedChild\` Value::Struct so wat callers see the` → `/// \`:wat::kernel::Process\` Value::Struct so wat callers see the`

## Fix B — `libc::fork(2)` → `clone3` at the module-doc CLAIM site
- **fork.rs:3** (module doc): `//! Creates three pipe pairs, calls \`libc::fork(2)\`, redirects the` → name the real mechanism, e.g. `//! Creates three pipe pairs, forks via \`clone3\` (through \`spawn_lifelined\`), redirects the`. Keep it accurate + brief.
- **DO NOT touch fork.rs:155, 639, 1005** — those say "instead of bare libc::fork()" describing the *history/rationale* (correct: they explain spawn_lifelined replaced bare fork). Those are honest. Only the line-3 claim "calls libc::fork(2)" is the lie.

## What NOT to touch
- types.rs:997/1003/1004, check.rs:16812/16927 — those are RETIREMENT-RECORD comments (historical context naming the retired type on purpose). Per FM 14 Bucket C: KEEP. They're in other files anyway (out of scope).
- Any code. Zero logic change.

## Gates
```
cargo build -p wat
cargo test -p wat --lib            # 895/0/1
cargo test -p wat --test comms     # 32/0
cargo test -p wat --test '*'       # fork/spawn suites still green (doc-only, must be unchanged)
```

## Report (SCORE)
1. Files touched (ONLY fork.rs).
2. Fix A (6 sites) + Fix B (1 site) with new text each.
3. Gate counts.
4. `git status --porcelain` + confirm `git diff src/fork.rs` shows ONLY doc-comment lines changed (no code).
5. Confirm zero `ForkedChild` remains as a CURRENT-return claim in fork.rs (`git grep ForkedChild src/fork.rs` → empty, since all 6 were live-claim sites).
6. Any honest delta.

Do NOT commit. Orchestrator re-casts intueri + circumspicere on the corrected tree; converged L1+L2=0 across all 9 spells → comms wards (6th/FINAL home) + /proc-purge commit → homes-walk CLOSES.
