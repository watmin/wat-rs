# Arc 143 Slice 3 — Pre-handoff expectations

**Drafted 2026-05-02 (late evening)** — same session as slice 2;
parallel-prepared while slice 2 was sweeping.

**Brief:** `BRIEF-SLICE-3.md`
**Output:** 2 Rust files modified (`src/runtime.rs` + `src/check.rs`) +
1 new test file + ~250-word report.

## Setup — workspace state pre-spawn

- Slice 1 + slice 2 shipped. All 11 + 26 of those tests pass.
- HolonAST is a `pub enum` in `holon-rs/src/kernel/holon_ast.rs:51`;
  `Bundle(Vec<HolonAST>)` is open-access via pattern matching from
  wat-rs.
- Slice 1's helpers (`function_to_signature_ast` at `runtime.rs:5974+`,
  etc.) are the placement precedent for sonnet's new helpers.
- The synthesized AST shape from `signature-of` (per slice 1's tests):
  `Bundle [Symbol "name<T>", Bundle [Symbol "_a0", Symbol "T0"], ...,
  Symbol "->", Symbol "Ret"]`.
- Workspace baseline: 1 pre-existing failure
  (`deftest_wat_lru_test_lru_raw_send_no_recv` from arc 130 RELAND v1);
  everything else passes.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | Exactly 2 Rust files modified (`src/runtime.rs` + `src/check.rs`) + 1 NEW test file (`tests/wat_arc143_manipulation.rs`). NO wat files. |
| 2 | `eval_rename_callable_name` present | Function in runtime.rs near slice 1's helpers (`runtime.rs:6111+`). Validates 3 args, returns `Value::holon__HolonAST`. |
| 3 | `eval_extract_arg_names` present | Function in runtime.rs near slice 1's helpers. Validates 1 arg, returns `Value::Vec<keyword>`. |
| 4 | Rust helpers added | Bundle-destructuring helper + string-split-at-`<` helper. Mirrors slice 1's helper-placement pattern. |
| 5 | Dispatch arms in runtime.rs | Two new arms in the dispatch (near slice 1's at 2411-2413): `:wat::runtime::rename-callable-name` + `:wat::runtime::extract-arg-names`. |
| 6 | Scheme registrations in check.rs | Two new schemes registered alongside slice 1's at 10997-11021: rename-callable-name takes `(HolonAST, keyword, keyword) -> HolonAST`; extract-arg-names takes `HolonAST -> Vec<keyword>`. |
| 7 | Type-checker special-case extended | The slice 1 special-case at `check.rs:3126-3163` extends to include both new primitives (same arg-bypass treatment as slice 1's three primitives). |
| 8 | **`cargo test --release --workspace`** | Exit=0 except the 1 pre-existing arc 130 failure. Same baseline + new test file's 6-9 tests. ZERO new regressions. |
| 9 | New tests cover all cases | 6-9 tests in the new test file: rename-callable-name (with type-params, without type-params, error-not-bundle, error-name-mismatch); extract-arg-names (3+ arg head, zero-arg head, head with arrow + return type, error-not-bundle). |
| 10 | Honest report | 250-word report includes: file:line refs, helper functions, dispatch arms, scheme registrations, verbatim AST output for a rename test, test totals (1 pre-existing failure unchanged + 0 new regressions), honest deltas. |

**Hard verdict:** all 10 must pass. Row 8 is load-bearing for
non-regression. Row 7 (type-checker special-case extension) is
load-bearing for the primitives type-checking correctly at call sites.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 150-300 LOC. >400 LOC = re-evaluate. |
| 12 | Style consistency | New eval funcs follow existing arg-validation pattern; helpers placed adjacent to slice 1's helpers; pattern-match shape on `HolonAST::Bundle` matches existing usage in runtime.rs. |
| 13 | Test coverage breadth | Tests cover both happy path AND error path. Error tests use direct EDN AST construction or `from-watast` → broken-shape input. |
| 14 | Compose with slice 1 | At least ONE test invokes both slice 1 and slice 3 primitives in sequence (e.g., `(:rename-callable-name (:signature-of :foldl) :foldl :reduce)`) — proves the slices integrate. |

## Independent prediction

- **Most likely (~70%)**: Mode A clean ship. Slice 1 + 2 calibration is
  fresh; sonnet has the pattern. The HolonAST `Bundle` access is
  open via pub enum; the string-split is trivial. ~12-18 min runtime.
- **Soft drift on test-construction approach (~15%)**: sonnet picks a
  test-AST-construction shape that differs from the brief's suggestion
  (e.g., uses `from-watast` literal vs direct Bundle construction).
  Honest delta; outcome still committable.
- **Helper-placement surprise (~8%)**: sonnet finds an existing helper
  in runtime.rs (e.g., a Bundle pattern-matcher) that the brief
  missed; reuses it. Cleaner than predicted.
- **String-split edge case (~5%)**: type-params suffix has nested
  generics (`<T, Vec<U>>`); naive split-at-first-`<` does the right
  thing? Probably yes (we want everything from FIRST `<` onward), but
  worth verifying.
- **Mode B regression (~2%)**: rare given the pattern is well-trodden.

## Methodology

After sonnet returns:

1. Read this file FIRST.
2. Score each row of both scorecards.
3. Diff via `git diff --stat` — verify file scope.
4. Read the new test file; count tests; verify happy + error paths
   covered.
5. Run `cargo test --release --workspace` locally; confirm row 8.
6. Verify row 14 (compose with slice 1) by reading at least one test
   that chains the primitives.
7. Score; commit `SCORE-SLICE-3.md`.

## Why this slice matters for the chain

After slice 3, the SUBSTRATE side of arc 143's macro layer is
COMPLETE: slice 1 (point lookups), slice 2 (computed unquote), slice 3
(HolonAST manipulation). Slice 6's `:wat::runtime::define-alias`
defmacro becomes pure wat — a ~15 LOC quasiquote body using the three
substrate primitives.

This is the substrate-as-teacher cascade in motion: slice 6 was
attempted first and surfaced the gaps; slices 2 + 3 close them; slice
6 relands trivially. The discipline the project was built on,
restored after the compaction-amnesia incident.
