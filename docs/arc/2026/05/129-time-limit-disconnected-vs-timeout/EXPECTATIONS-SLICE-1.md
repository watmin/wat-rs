# Arc 129 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning the sonnet agent and
BEFORE its first deliverable. Durable scorecard.

**Brief consumed:** `BRIEF-SLICE-1.md` + `DESIGN.md`
**Output channel:** `crates/wat-macros/src/lib.rs` modifications
+ ~150-word written report.

## Setup — what the workspace looks like before sonnet starts

- Slice 1 of arc 126 (the `ChannelPairDeadlock` check) is
  COMMITTED at `2b6d053`.
- Slice 2 of arc 126 (the 6 deadlock-class tests' `:ignore` →
  `:should-panic` conversion) is in WORKING TREE, uncommitted.
  The 3 wat-test files have the new annotations in place.
- Workspace is currently RED because of slice 2's pending
  edits: `cargo test --release --workspace` fails with 5
  `panic did not contain expected string` failures (the bug
  arc 129 fixes).

This is intentional. Slice 2's working-tree state IS the
verification harness for arc 129.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff | Only `crates/wat-macros/src/lib.rs` modified beyond what's already in working tree (the 3 slice-2 wat-test files). `git diff --stat` shows 4 files modified total: 3 wat-test (slice 2) + 1 Rust (slice 1's lib.rs). |
| 2 | JoinHandle kept | The line `let _ = ::std::thread::spawn(...)` becomes `let __wat_handle = ::std::thread::spawn(...)`. Verifiable via `grep -n "let __wat_handle" crates/wat-macros/src/lib.rs`. |
| 3 | Match arms split | `Err(_)` becomes two arms: `Err(::std::sync::mpsc::RecvTimeoutError::Timeout)` and `Err(::std::sync::mpsc::RecvTimeoutError::Disconnected)`. Verifiable via `grep -n "RecvTimeoutError" crates/wat-macros/src/lib.rs`. |
| 4 | resume_unwind on Disconnected | The Disconnected arm calls `__wat_handle.join()` and on `Err(payload)` calls `::std::panic::resume_unwind(payload)`. Verifiable via `grep -n "resume_unwind" crates/wat-macros/src/lib.rs`. |
| 5 | **Workspace green** | `cargo test --release --workspace` exit=0. The 6 deadlock-class tests in slice 2's working-tree edits NOW PASS via `:should-panic` matching. |
| 6 | **Six tests now PASS** | Each of the 6 named in slice 2 (`test-cache-service-put-then-get-round-trip`, `test-step3-put-only` through `test-step6-lru-eviction-via-service`, `step_B_single_put`) reports as `... ok` in cargo test output. |
| 7 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 8 | Honest report | 150-word report with: file:line refs for the two edits, the exact final form of the match block, workspace test totals, runtime-per-test data, confirmation of arc 126 slice 2's six passing test names. |

**Hard verdict:** all 8 must pass for arc 129 to ship clean.
Row 5 + row 6 are the load-bearing rows — they validate that
the bug fix actually fixes the bug, with slice 2 as the
test bed.

## Soft scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | 15-30 LOC change in `crates/wat-macros/src/lib.rs`. >50 LOC = re-evaluate scope. |
| 10 | Comments preserved or improved | The DESIGN's load-bearing comments (explaining WHY the new shape is correct) appear in the implementation verbatim or with prose improvements. Not deleted. |
| 11 | No new public API | No `pub fn`, no new exported types, no new helpers in `lib.rs`. |
| 12 | The else-branch is unchanged | The no-`:time-limit` path (lines 682-699) compiles identically. |
| 13 | Inner thread no longer leaks on Disconnected | The fix joins the handle on Disconnected — the thread doesn't leak in this case (only in the real Timeout case, which still leaks per arc-123's existing UX). |
| 14 | Race-condition observation | If sonnet observes any flaky-test behavior (intermittent Timeout panics on what should be Disconnected paths), the report surfaces it as a follow-on arc concern. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~60%):** all 8 hard + 5-6 soft pass cleanly.
  The fix is well-scoped, the DESIGN's two scenarios are
  precise, and `std::panic::resume_unwind` is a well-known
  idiom. Slice 2's 6 tests pass via :should-panic.
- **Second-most-likely (~25%):** all 8 hard pass, one soft
  drift (e.g. fewer comments than the DESIGN suggested, or
  LOC slightly under because the fix is shorter than predicted).
- **Substring still doesn't propagate (~10%):** row 5 + 6 fail
  because some OTHER layer between resume_unwind and cargo
  libtest mangles the message. Would surface arc 130. Less
  likely because the panic chain post-resume_unwind is
  Rust-stdlib territory; std::panic::resume_unwind is
  documented to preserve the payload verbatim.
- **Race condition fires (~5%):** with slice 2's 200ms budget,
  the panic-fast path takes <10ms; the race shouldn't fire.
  But surprises happen.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 4 files: 3 wat + 1 Rust).
4. Verify hard row 2 by `grep -n "let __wat_handle"`.
5. Verify hard row 3 by `grep -n "RecvTimeoutError"`.
6. Verify hard row 4 by `grep -n "resume_unwind"`.
7. Verify hard row 5 by reading the cargo-test totals from the
   agent's report.
8. Verify hard row 6 by reading the per-test outcomes.
9. Score; commit SCORE-SLICE-1.md as a sibling.

## What we learn

**If clean:** the failure-engineering chain is fully validated
across THREE substrate-fix arcs (128, 129) plus one structural-
rule arc (126). The artifacts-as-teaching discipline ships
small substrate fixes through sonnet sweeps with the same
fidelity as larger structural rules. The next time a substrate
gap surfaces, the path is known: DESIGN + BRIEF + EXPECTATIONS
+ delegate.

**If row 5/6 fails:** another substrate layer is mangling the
panic substring. Surface arc 130; the diagnosis becomes the
next teaching artifact.

**If row 9 (LOC) drifts well over 50:** the brief was unclear
about scope; sonnet expanded to address something the DESIGN
didn't ask. Tighten the brief for next iteration.

## Why this slice matters for the broader chain

Arc 129 is the second substrate fix in the failure-engineering
record (the first was arc 128). Both surfaced through the
arc 126 chain. After arc 129 ships:

- Arc 126 slice 2 is committable on a green workspace.
- Arc 126 slice 3 (closure: INSCRIPTION + doc updates) becomes
  the natural next step.
- The substrate's `:should-panic` + `:time-limit` combination
  is correct for all future tests — not just the 6 in arc 126.

The four-questions framing the user invoked ("phenomenal" UX)
is real: `:should-panic("substring") + :time-limit("Xms")`
should compose. Currently it doesn't. Arc 129 makes it.
