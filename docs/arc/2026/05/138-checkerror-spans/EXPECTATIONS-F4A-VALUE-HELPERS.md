# Arc 138 F4a — Pre-handoff expectations

**Brief:** `BRIEF-F4A-VALUE-HELPERS.md`
**Targets:** src/spawn.rs, src/string_ops.rs, src/io.rs (Value-shaped helpers + their callers).

## Setup — workspace state pre-spawn

- Baseline: F3 commit `6327840`. F1 + F2 + F3 cracks closed. CRACKS-AUDIT updated to decompose F4 into F4a/F4b/F4c.
- 6 Span::unknown() in src/spawn.rs (all rationaled).
- 7 Span::unknown() in src/string_ops.rs (all rationaled).
- 25 Span::unknown() in src/io.rs (15 rationaled; ~10 unmarked are F4b/F4c territory or genuine OS errors).
- 11 Value-shaped helpers across the 3 files.
- 6/6 arc138 canaries pass.

## Hard scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | src/spawn.rs + src/string_ops.rs + src/io.rs. NO others. |
| 2 | All 11 helpers gain span/list_span | confirmed signature update per BRIEF list. |
| 3 | Helper bodies use threaded span | Span::unknown() inside helpers replaced with the threaded span; rationale comments deleted. |
| 4 | Callers updated | Each caller passes appropriate span (Pattern A or B). |
| 5 | Workspace tests pass | All 6 arc138 canaries PASS; `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 6 | No commits | Working tree shows uncommitted modifications only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 7 | Span::unknown() count drops | ≥ 50% drop across the 3 files (specifically: spawn.rs 6 → ≤ 2, string_ops.rs 7 → ≤ 2, io.rs 25 → ≤ 18). |
| 8 | Pattern distribution | A dominates for arg-specific helpers (expect_*); B for arity_2-style. |
| 9 | Honest report | Comprehensive ~400 words; per-file helper + caller counts. |

## Independent prediction

- **Most likely (~70%):** 6/6 hard + 3/3 soft. Sonnet ships in 10-20 min. Pattern matches F2/F3.
- **String_ops/two_strings broadening (~15%):** the `two_strings` helper may need a parameter beyond span (refactor surfaces); honestly named.
- **Test pattern updates (~10%):** existing tests using helpers may need span: Span::unknown() added.
- **Cross-file regression (~5%):** rare; cargo build catches.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 3 files.
3. `grep -c "Span::unknown()" src/spawn.rs src/string_ops.rs src/io.rs` → verify drops.
4. Run all 6 canaries; workspace tests.
5. Score; commit + push; queue F4b (FromWat trait expansion).
