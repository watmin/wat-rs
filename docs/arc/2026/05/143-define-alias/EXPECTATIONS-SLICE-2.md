# Arc 143 Slice 2 — Pre-handoff expectations

**Drafted 2026-05-02 (late evening)** for the computed-unquote
substrate change.

**Brief:** `BRIEF-SLICE-2.md`
**Output:** 2-3 Rust files modified (`src/macros.rs` + `src/runtime.rs`
+ possibly `src/check.rs`) + 4-6 new tests + ~250-word report.

## Setup — workspace state pre-spawn

- `src/macros.rs:541-622` enforces "body must be quasiquote template";
  `unquote_argument` (line 785+) handles only Symbol + already-substituted
  literal; arbitrary List arguments fall through.
- `src/runtime.rs:5878` has `value_to_watast` (used by struct->form,
  the precedent for Value → WatAST conversion at expand-time).
- `src/runtime.rs:15400-15470` is the stdlib bootstrap; `expand_all`
  is called BEFORE `register_functions`; bootstrap-time `sym` doesn't
  have user defines populated but substrate primitives dispatch fine
  via Rust match arms.
- Slice 1's primitives (`:wat::runtime::lookup-define`,
  `:wat::runtime::signature-of`, `:wat::runtime::body-of`) ship via
  Rust dispatch and will be available at expand-time after slice 2.
- Workspace baseline: 1 pre-existing failure
  (`deftest_wat_lru_test_lru_raw_send_no_recv` from arc 130 RELAND v1
  hitting the `:reduce` gap — fixed by slice 7); everything else
  passes.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | 2-3 Rust files modified: `src/macros.rs` (mandatory), `src/runtime.rs` (call-site threading at bootstrap), possibly `src/check.rs` (its own `expand_all` call sites). NO wat files. NO new test crate. New tests live in `src/macros.rs::tests` module. |
| 2 | `substitute_bindings` helper added | Helper function in `src/macros.rs` that recursively walks WatAST replacing Symbols-matching-binding-keys with bound ASTs. ~30 LOC. Tested independently. |
| 3 | `unquote_argument` extended | Handles List arg by substituting bindings, calling `eval()`, calling `value_to_watast()`. Existing Symbol path unchanged. Existing already-substituted literal path preserved (verify via heuristic OR by reading existing macros to confirm no regression). |
| 4 | `splice_argument` extended | Handles List-as-expression analogously to unquote_argument; result Value must convert to a list; splice elements. |
| 5 | Threading complete | `expand_template`, `walk_template`, `unquote_argument`, `splice_argument` all carry `env: &Environment` + `sym: &SymbolTable`. Callers (`expand_macro_call`, `expand_form`, `expand_once`, `expand_all`) updated to thread through. Bootstrap call sites (`runtime.rs:15400+`, `check.rs` similar) pass `&Environment::default()` + `&SymbolTable::default()`. |
| 6 | Backward-compat heuristic documented | Sonnet's report names the heuristic chosen for "List = expression vs List = literal." Likely "head-is-Keyword → eval, else literal." Decision MUST be explicit + justified. |
| 7 | Existing macro tests UNCHANGED | All tests in `src/macros.rs::tests` (line 856+) PASS WITHOUT MODIFICATION. The existing `make-deftest`, `deftest`, `Concurrent`, etc. macros must continue to expand identically. Zero behavior break for existing macros. |
| 8 | New computed-unquote tests added | 4-6 new tests in `src/macros.rs::tests` covering: Symbol unquote still works, List literal in unquote stays literal (if heuristic), computed unquote evaluating substrate primitive call, computed unquote substituting macro params before eval, computed unquote-splicing, computed unquote inside nested quasiquotes. ALL pass. |
| 9 | **`cargo test --release --workspace`** | Exit=0. Same baseline pass count + new tests. Same 1 pre-existing failure (`deftest_wat_lru_test_lru_raw_send_no_recv`); ZERO new regressions. |
| 10 | Honest report | 250-word report covers: file:line refs for all changes, backward-compat heuristic + justification, 2-3 new test bodies verbatim, test totals confirming 1 pre-existing failure unchanged + 0 new regressions, honest deltas (env type, any sym shape required, anything brief didn't predict). |

**Hard verdict:** all 10 must pass. Rows 7 + 9 are load-bearing for
non-regression. Row 6 is load-bearing for the discipline (the
heuristic must be explicit, not implicit).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 100-200 LOC. >300 LOC = re-evaluate (likely scope creep). |
| 12 | Style consistency | New code follows existing patterns: error types via `MacroError`, span discipline via call-site span pattern (per arc 016 slice 1), recursive walk shape mirrors `walk_template`. |
| 13 | Heuristic minimal | The "List = expression vs literal" heuristic is the SIMPLEST possible (e.g., "head is Keyword"). If sonnet invents a complex distinguishing rule, that's a hint it's over-engineered. |
| 14 | Doctring discipline | Public/extended functions get a brief doc-comment naming the new behavior + the slice that added it. |

## Independent prediction

Before reading sonnet's output, the orchestrator predicts:

- **Most likely (~60%):** Mode A clean ship. Brief is substrate-
  informed; all assumptions verified pre-spawn. Sonnet ships in
  15-25 min. The only real design choice is the literal-vs-expression
  heuristic; sonnet picks the obvious one (head-is-Keyword) and
  surfaces the choice in the report.
- **Second-most-likely (~20%):** Mode A with soft drift on
  backward-compat handling. The existing-already-substituted-literal
  case has more nuance than the brief anticipated; sonnet finds an
  edge case and adapts. Outcome still committable.
- **Threading-extent surprise (~10%):** the env/sym threading touches
  more call sites than the brief identified (e.g., a sub-crate's
  expand_all). Sonnet reports the additional sites; orchestrator
  scores PASS with honest delta.
- **value_to_watast type-assumption surprise (~5%):** the existing
  `value_to_watast` may have constraints (e.g., specific Value
  variants only); sonnet has to handle Values that can't convert
  cleanly. Surfaces in honest deltas; minor reland or extension.
- **Mode B at existing-test regression (~5%):** sonnet's heuristic
  breaks an existing macro test. Reland with sharper heuristic.

## Methodology

After sonnet returns, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` — verify file scope.
4. Read sonnet's heuristic justification (row 6).
5. Run `cargo test --release --workspace` locally; confirm row 9.
6. Run `cargo test --release -p wat --lib macros::tests::` (or
   equivalent module path) to confirm existing macro tests pass.
7. Read the 2-3 new test bodies sonnet quoted; verify they exercise
   the new path.
8. Score; commit `SCORE-SLICE-2.md` as a sibling.

## Why this slice matters for the chain

Slice 2 is the GATING substrate change for arc 143's macro layer.
Without computed unquote, slice 6's `define-alias` defmacro can't be
written. With it shipped:
- Slice 6's brief becomes trivial (pure wat, ~15 LOC defmacro)
- Slice 7's apply becomes trivial (one line)
- Arc 130's stepping stone unblocks (post-slice-7)
- Arc 109 v1 closes (post all post-109 arcs)

This slice ALSO unlocks every FUTURE reflective macro — sweep
generators, spec validators, doc extractors. The substrate gains a
foundational metaprogramming capability for ~100 LOC of focused
extension.

## What we learn

- **Mode A clean:** the substrate-informed brief discipline propagates
  cleanly to sonnet. The orchestrator's pre-spawn crawl was the
  enabling discipline. Cadence restored.
- **Mode B at row 7 (existing macros break):** the literal-vs-expression
  distinction has more nuance than the brief captured; reland with
  sharper heuristic.
- **Mode B at row 9 (workspace not green):** the threading touched
  something that broke; reland with sharper scope on the threading.
- **Sonnet refuses to spawn or reports impossibility:** unlikely
  given the pre-spawn verification, but if it happens, the brief had
  a hidden gap; orchestrator investigates.
