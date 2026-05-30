# BRIEF — arc-242 stale-nil fixture sweep (unblock Stone 243.3 commit)

You are sonnet. A pre-existing broken integration test surfaced during Stone 243.3 R3-β: `tests/arc112_slice2b_process_send_recv.rs` uses `:wat::core::nil` in VALUE position, which arc 242 Stone 242.2's Doctrine 1 retired (`:wat::core::nil` is a TYPE keyword; value position must use bare `nil`). The arc-242 bulk sed missed this file. This is arc-242 debt, NOT caused by R3-β — but it must be fixed so the tree is green before the R3-β commit (`feedback_no_broken_commits`).

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

**IMPORTANT — working tree state:** the working tree carries the uncommitted R3-β substrate changes (8 modified src/ files). DO NOT revert, stash, or touch them. Your work is ONLY in `tests/`. Orchestrator commits everything atomically afterward.

## What to do

### Step 1 — definitive scan (the orchestrator's grep was display-corrupted; you establish ground truth)

```
grep -rn ':wat::core::nil' tests/
```

Enumerate every occurrence. For each, determine **position**:
- **VALUE position** (a form being evaluated/returned — e.g. an element of a `[...]` vector literal, a `let` body's final form, a fn body's return form, an argument to a call) → must become bare `nil`.
- **TYPE position** (a type annotation — e.g. after a param name `(:x :wat::core::nil)`, in a return-type slot, inside a `:wat::core::Vector<...>` type arg) → STAYS `:wat::core::nil` (it's correctly a type keyword there).

The known site is `tests/arc112_slice2b_process_send_recv.rs` (~lines 37/43/45/46/75 — verify exact lines yourself). Sweep ALL of `tests/`, not just that file.

### Step 2 — migrate value-position occurrences

For each VALUE-position `:wat::core::nil`, replace with bare `nil`. Leave TYPE-position occurrences untouched.

### Step 3 — RUN the affected test(s) — measure, don't assume

For each test file you edited, run it:
```
cargo test --release --test <target_name>
```
(For the known file: `cargo test --release --test arc112_slice2b_process_send_recv`.)

**CRITICAL (the F9 lesson):** the premise is "stale syntax — migrating nil fixes it." That premise may be WRONG. These nil values sit as fn-body returns where the declared return type is `(:wat::core::Vector :i64)`. After migrating to bare `nil`, the program may STILL fail type-checking — because `[nil]` (a vector containing nil) may not conform to `Vector<i64>`, OR because arc 242 changed deeper semantics than just the keyword.

- If the test goes **GREEN** after migration → stale-fixture confirmed; done.
- If the test stays **RED** after migration → **STOP. Do NOT force it.** Capture the new error verbatim. This means the issue is deeper than stale syntax (semantic shift, or the test's own expectation is now wrong). Surface it for orchestrator triage — do not rewrite the test's logic or its assertions on your own judgment.

### Step 4 — verify no collateral

Confirm the gates still hold (your edits are test-only, so these should be unchanged):
```
cargo test --release --lib -p wat 2>&1 | tail -1        # expect 890/0
cargo test --release --test function 2>&1 | tail -1     # expect 8/0
cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -1   # expect 3/0
```

## STOP triggers (REJECTION)

1. Any edited test stays RED after migration — STOP, surface the verbatim error (do NOT rewrite test logic/assertions)
2. Lib < 890 / function < 8 / probe < 3 (your test-only edits shouldn't affect these — if they do, something's wrong; surface it)
3. holon-rs touched (STOP-5)
4. Any src/ file touched (your scope is tests/ ONLY — the R3-β src changes are off-limits)
5. Working tree R3-β changes reverted/stashed
6. A TYPE-position `:wat::core::nil` migrated (that would be wrong — type position is correct as-is)
7. Commit attempted (orchestrator commits atomically)
8. 20 min elapsed

## Discipline

- Sonnet writes (`feedback_sonnet_writes_substrate`); orchestrator commits.
- Measure, don't theorize (`feedback_debugging_approach`): RUN the test; let the result decide stale-vs-deeper.
- DO NOT commit. DO NOT touch src/. DO NOT touch INTERSTITIAL.

## Return paragraph (≤ 150 words)

- The definitive grep result (every `:wat::core::nil` in tests/, with VALUE/TYPE classification)
- Which occurrences you migrated (file:line)
- For each edited test: GREEN or RED after migration; if RED, the verbatim error
- Gate confirmation (lib/function/probe unchanged)
- Any additional findings
