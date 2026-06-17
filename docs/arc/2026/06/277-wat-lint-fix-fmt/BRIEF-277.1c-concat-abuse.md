# BRIEF — Arc 277.1c: the `concat-abuse` lint rule

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently.

## The work (one paragraph)

Add one **report-only** lint rule to `wat/lint.wat`: `concat-abuse` detects a `string::concat` chain
that interleaves string literals with non-literal args (a hand-rolled template) and suggests `format`.
It mirrors the existing `nested-if-=-ladder` rule exactly in shape. Wire it into `lint-file`, add two
deftests, un-ignore the Rust probe.

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1c-concat-abuse.md` § "The detection"** and
implement it verbatim: helpers `concat-head?`, `concat-arg-counts` (→ `Tuple(n-lits, n-vals)`),
`concat-abuse?`, `make-concat-finding`, and the recursive `rule-concat-abuse-form`. Report-only:
`fix` is `""`; severity `"warn"`; rule name exactly `"concat-abuse"`.

## Read in order (the rooms)

1. `docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1c-concat-abuse.md` — THE SPEC.
2. `wat/lint.wat:48-58` — `lint-structural?`, `kw-or-sym?` (reuse these).
3. `wat/lint.wat:228-281` — `make-ladder-finding` (copy the `ast-span` → `:line`/`:col` shape) and
   `rule-nested-if-=-ladder-form` (copy the report-or-recurse structure EXACTLY).
4. `wat/lint.wat:286-300` — `lint-file`: add `rule-concat-abuse-form` to the per-form fold, **after**
   the ladder rule's findings (so existing "first finding is the ladder" assertions hold):
   `(:wat::core::concat acc (:wat::core::concat (rule-nested-if-=-ladder-form form path) (rule-concat-abuse-form form path)))`.
5. `wat-tests/lint.wat` — add two deftests after Case 4 (copy the Case 1 / Case 2 shapes):
   - **Case 5 `detects-concat-abuse`:** a SourceFile body `(:wat::core::string::concat \"x: \" a \" of \" b)`
     inside a defn → assert `lint-source` yields a finding whose `Finding/rule` == `"concat-abuse"`.
     (There must be at least one such finding; if other findings also appear, filter/scan for it — but
     the fixture has no ladder, so the concat finding will be present.)
   - **Case 6 `no-false-positive-concat`:** two clean files — `(:wat::core::string::concat \"a\" \"b\")`
     (all-literal) and `(:wat::core::string::concat a b)` (all-value) — → `lint-source` length == 0.
6. `tests/probe_arc277_lint_concat_abuse.rs` — remove the one `#[ignore = "arc 277.1c …"]` attribute.

## Implementation sketch (you fill it; the shape is fixed by the DESIGN + the ladder rule)

`concat-arg-counts` walks `(:wat::core::drop children 1)` with a `foldl` over a `Tuple(i64,i64)`
accumulator: `(if (= (ast-kind arg) "string") (Tuple (+ lits 1) vals) (Tuple lits (+ vals 1)))`.
`concat-abuse?` = `concat-head?` AND `(i64::>= n-lits 1)` AND `(i64::>= n-vals 1)`.
`rule-concat-abuse-form`: `(if (concat-abuse? form) <one-finding-vector> (if (lint-structural? form) <foldl-recurse> <empty>))`.

## STOP triggers (halt + report, do not improvise)
1. If reading a `Tuple(i64,i64)` via `first`/`second` fails at runtime — STOP, report the exact error.
2. If wiring the second rule changes the existing Case 1/Case 2 deftest results (they must stay green) —
   STOP, report (likely the fold order — the concat rule must come AFTER the ladder rule).
3. If `ast-span` on a concat form does not yield `:line`/`:col` like `make-ladder-finding` uses — STOP,
   report (do not fabricate a span).

## Blast radius
`wat/lint.wat` + `wat-tests/lint.wat` + `tests/probe_arc277_lint_concat_abuse.rs` (un-ignore) ONLY.
No Rust source edits. No new files. No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc277_lint_concat_abuse          # 1/1 GREEN
cargo test --release -p wat --test probe_arc277_lint_if_ladder             # still GREEN (ladder rule intact)
cargo test --release --test test 2>&1 | grep "test result"                 # deftest binary: 259 passed / 1 failed (was 257; +2 new lint deftests; the 1 = run_string_entry_direct)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result      # deporder gate: 1 passed / 0 failed
```
Report: a summary of the new helpers/rule + the `lint-file` wiring change, the four command outputs
verbatim, and any delta from expected. Do not claim green you did not see.
