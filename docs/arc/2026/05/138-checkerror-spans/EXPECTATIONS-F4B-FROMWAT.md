# Arc 138 F4b — Pre-handoff expectations

**Brief:** `BRIEF-F4B-FROMWAT.md`
**Targets:** src/rust_deps/marshal.rs + crates/wat-macros/src/codegen.rs.

## Setup — workspace state pre-spawn

- Baseline: F4a commit `ec4b465`. F1+F2+F3+F4a closed.
- 17 Pattern E sites in marshal.rs (all rationaled).
- Single proc-macro emit caller in codegen.rs (line 165).
- 6/6 arc138 canaries pass.

## Hard scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | 2 files modified: marshal.rs + codegen.rs. NO others. |
| 2 | Trait + 10 impls updated | from_wat gains span; impls use threaded span. |
| 3 | Recursive calls pass span.clone() | Option<T>/tuples/Result<T,E>/Vec<T>. |
| 4 | Proc-macro emit produces compiling code | codegen.rs:165 quote! adds args[#idx].span().clone(). |
| 5 | Span::unknown() in marshal.rs | 17 → 0. All rationale comments deleted. |
| 6 | Workspace tests + 6 canaries pass | empty FAILED. |

## Soft scorecard (3 rows)

| # | # | # |
|---|---|---|
| 7 | Honest report | per-impl confirmation + recursive call list. |
| 8 | Calibration | ≤ 15 min sonnet (matches F2 shape). |
| 9 | No commits | working tree only. |

## Independent prediction

- **Most likely (~80%):** 6/6 + 3/3, sonnet 8-15 min. Same shape as F2.
- **Recursive call gotcha (~10%):** sonnet misses one of the inner calls; cargo build catches.
- **Test pattern updates (~5%):** rare.
- **Cross-file regression (~5%):** rare.

## Methodology

After sonnet reports back: standard verify (diff stat, grep counts, canaries, workspace) → score → commit + push → queue F4c.
