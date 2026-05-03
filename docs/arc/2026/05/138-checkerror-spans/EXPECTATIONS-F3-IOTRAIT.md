# Arc 138 F3 — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-F3-IOTRAIT.md`
**Targets:** src/io.rs (trait def + 7 impls + 13 eval_io_* callers), src/runtime.rs (3 spawn plumbing callers).

## Setup — workspace state pre-spawn

- Baseline: F2 commit (TBD when F2 ships).
- 16 leftover `// arc 138 slice 3b: span TBD` markers in src/io.rs from slice 3b.
- 6/6 arc138 canaries pass.

## Hard scorecard (7 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | 2 files modified: src/io.rs + src/runtime.rs. NO others (harness.rs, fork.rs, compose.rs only have type refs, should not be touched). |
| 2 | Trait expansion | WatReader's 4 error-returning methods gain `span: Span`. WatWriter's 4 error-returning methods gain `span: Span` (snapshot unchanged — no error path). |
| 3 | All 7 implementors updated | RealStdin/RealStdout/RealStderr/StringIoReader/StringIoWriter/PipeReader/PipeWriter all updated. Each error-emitting body uses threaded span. |
| 4 | 16 caller sites updated | 13 in src/io.rs eval_io_*, 3 in src/runtime.rs spawn plumbing. Each passes a real span. |
| 5 | Slice 3b markers cleared | `grep -c "arc 138 slice 3b" src/io.rs` drops to 0 (or near). Any leftover Pattern E carries `// arc 138: no span — <reason>`. |
| 6 | Workspace tests pass | All 6 arc138 canaries PASS. `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 7 | No commits | Working tree shows uncommitted modifications only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 8 | Span source quality | Caller sites use list_span/call-form span (Pattern B dominates — trait method failures are about call site, not specific args). |
| 9 | close() decision documented | Whether `close` gains span or stays unchanged is named in the report. |
| 10 | Honest report | Comprehensive ~400 words; per-impl + per-caller distribution; helper sigs broadened named; Pattern E rationales (if any) listed. |

## Independent prediction

- **Most likely (~55%):** 7/7 hard + 3/3 soft. Sonnet ships in 30-45 min. Pattern B dominates; close() gets span for consistency; PipeWriter close() error path threaded properly.
- **Helper-sig broadening surfaces (~20%):** some eval_io_* shims don't have list_span; sonnet adds `list_span: &Span` parameter or uses args[0].span(). All in-file. Hard 7/7 + soft 10 PASS+.
- **PipeReader/PipeWriter complexity (~10%):** the pipe impls have specific error paths (closed / disconnected) that need careful threading. Adds 5-10 min.
- **Test regression (~10%):** WatReader/WatWriter signature change breaks tests in spawn/process suites. Sonnet investigates; mechanical fix.
- **Cross-file regression (~5%):** sonnet accidentally edits harness.rs/fork.rs/compose.rs (only type refs there); cargo build catches.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 2 files (src/io.rs + src/runtime.rs).
3. `grep -c "arc 138 slice 3b" src/io.rs` → drops to 0 (or near).
4. `grep -c "Span::unknown()" src/io.rs` → drops substantially.
5. Run all 6 canaries; workspace tests.
6. Spot-check 5 caller sites + 5 impl method bodies.
7. Score; commit + push; queue F4 (Value-shaped APIs).

## What this slice tells us

- All clean → trait expansion pattern is durable for traits with multi-method, multi-implementor surfaces. F4 (Value-shaped APIs) dispatches with confidence.
- close() handling clarifies whether default-impl methods get span — pattern decision for future trait expansions.
