# Arc 138 Slice 2 — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-2.md`
**Target:** `src/types.rs` (and optionally one new canary test file)

## Setup — workspace state pre-spawn

- Baseline (commit `23a30fe`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 1 finish shipped (8/8+4/4); 203 of 206 CheckError emission sites use real spans.
- Pre-sweep `Span::unknown()` count in `src/types.rs`: **TBD via grep before spawn** (likely 0; the file currently has only the `MalformedVariant` variant carrying span, and that arrives from real source. Sonnet will report BEFORE/AFTER).
- TypeError enum has 12 variants total. 1 (`MalformedVariant`) already carries span (arc 130 follow-up). 10 need spans. (One more, `MalformedField`, technically has 3 but only 1 emission site — we count it as 1 for the variant retrofit list.)
- ~26 emission sites in `src/types.rs` per `grep -c "Err(TypeError" src/types.rs`.
- The pattern is well-established by slice 1: variant + Display + threading. Slice 2 does the FULL retrofit (variant + Display + emissions + canary) since it's smaller scale than slice 1 (10 variants × 1-3 sites ≈ 26 sites total vs slice 1's 200).

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff (or +1 test file) | `src/types.rs` modified; optionally a new canary test file. No other Rust files. No docs. |
| 2 | All 10 variants gain `span: Span` | `DuplicateType`, `ReservedPrefix`, `MalformedDecl`, `MalformedName`, `MalformedField`, `MalformedTypeExpr`, `AnyBanned`, `CyclicAlias`, `AliasArityMismatch`, `InnerColonInCompoundArg`. `MalformedVariant` unchanged. |
| 3 | All 10 Display arms prefix coords | When span is non-unknown, the rendering starts with `<file>:<line>:<col>:`. `MalformedVariant`'s existing `at <span>` shape unchanged. |
| 4 | Emission sites thread real spans | ≥ 22 of 26 sites use real spans (≥ 85%). Each leftover `Span::unknown()` carries `// arc 138: no span — <reason>` rationale. |
| 5 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 6 | Canary added + passes | A new test asserts a TypeError's rendered Display contains `<test>:` (file:line:col). The test passes. |
| 7 | No commits | Working tree shows uncommitted modifications only. |
| 8 | Honest report | ~400 words; counts before/after; variant list; Display arms; helper signatures broadened (substrate observation if any); canary location; verification commands; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Span source quality | Each emission site uses the MOST RELEVANT local span (per the BRIEF's "best source" table); not always the parent form's span when a more specific token is available. |
| 10 | Helper signatures broadened deliberately | If sonnet adds `decl_span: Span` to `parse_struct` / `parse_enum` / etc., the parameter is uniform across the helper family (per slice 1 pattern). |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 10%. |
| 12 | Honest delta on AliasArityMismatch | The variant fires from `parse_type_expr` which is called from check.rs context, not from a clean type-decl boundary. If sonnet leaves it as `Span::unknown()` with rationale, that's PASS. If sonnet threads a span through, even better. Either way: explicitly named in honest deltas. |

## Independent prediction

Slice 1 finish landed 8/8+4/4 with one substrate observation (helper signatures + SchemeCtx trait gap). Slice 2 is structurally larger (full retrofit, not just emission threading) but smaller in scope (~26 sites vs ~200).

- **Most likely (~65%):** 8/8 hard + 4/4 soft. Sonnet ships in 25-40 min. ~22-25 sites real-spanned; 1-4 leftover with rationale.
- **Helper signatures broadened (~25%):** sonnet adds `decl_span: Span` to 3-5 parse_* helpers. Substrate observation flagged in honest deltas. Hard 8/8 + Soft 10 PASS+.
- **Display arm shape divergence (~5%):** sonnet adds the prefix differently in some arms (e.g., embeds the span in the message body rather than prefixing). Hard row 3 partial; we verify and re-baseline if cosmetic.
- **Canary test placement issue (~3%):** sonnet places the test in a location that doesn't compile or doesn't run. Re-locate; minor issue.
- **AliasArityMismatch surfaces re-architecture (~2%):** the parse_type_expr call chain doesn't have any span to thread. Sonnet explicitly leaves Span::unknown() with rationale; PASS.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat src/types.rs` → 1 file (or +1 test) modified, ~50-150 line edits estimate.
3. `grep -c "span: Span::unknown()" src/types.rs` → measure ≤ 4 leftover.
4. `grep -c "// arc 138: no span" src/types.rs` → matches the leftover-unknown count.
5. Read each modified variant def — confirm `span: Span` field added with doc comment.
6. Read each Display arm — confirm prefix shape correct.
7. Spot-check 5-10 emission sites — confirm best-source span used.
8. `cargo test --release --workspace 2>&1 | grep -E "FAILED" | grep -v trading | head -5` → empty.
9. Run sonnet's canary test by name — passes.
10. Score each row; write `SCORE-SLICE-2.md`.
11. If clean → commit + push, queue slice 3 (RuntimeError).

## What this slice tells us

- All clean → arc 138's pattern propagates across files. Slice 3 (RuntimeError, larger) dispatches with confidence.
- Helper signatures broadened → continued substrate observation about parse helpers' value-consumption pattern. Maybe a refactor follow-up surfaces.
- AliasArityMismatch genuinely uns-spannable → real substrate gap; document in slice 3 BRIEF.
- Hard fail — investigate. Likely cause: the full retrofit (variant + Display + emit) was too much for one engagement; split slice 2 into 2a (variant + Display) and 2b (emissions) for re-spawn.

## What follows

- Score → commit slice 2 → write slice 3 BRIEF (RuntimeError, ~22 user-facing variants in `src/runtime.rs`) → spawn sonnet → continue.
