# BRIEF — reader-based inline-wat gate

**The work (one paragraph).** Rewrite the inline-wat lint (`tests/lint/no_inlined_wat_in_tests.rs`) so it
detects inline wat by feeding string literals to wat's **own reader** (`parse_one_with_file`), not by the
current `startup_from_source(` substring. This makes the gate catch inline *drivers* (not just worlds) AND be
**surface-agnostic** — it catches both `(:wat::core::…)` and `(wat.core/…)` because the one reader reads both,
and it follows 300's convert-then-retire automatically. Full design + rationale: `DESIGN-STONE-inline-wat-reader-gate.md`.

## Read in order

1. `DESIGN-STONE-inline-wat-reader-gate.md` — the detector contract, the scan, the unit-test cases, the STOPs,
   the scope (GATE only; the sweep is NOT this strike). YOUR MARCHING ORDERS.
2. `tests/lint/no_inlined_wat_in_tests.rs` — the current gate (the file you rewrite): `collect_rs` walk, the
   `startup_from_source` needle, the `// rune:lint(no-inlined-wat)` file-level skip, the offender-list assert.
3. `crates/wat-reader/src/parser.rs:212` — `pub fn parse_one_with_file(src, file) -> Result<WatAST, ParseError>`
   (the reader you call). And `crates/wat-reader/src/ast.rs:58` — `enum WatAST` (`List(items, span)`,
   `Keyword(..)`, `Symbol(..)` — for the head-check).

## The detector (unit-testable core)

```rust
fn is_inline_wat_form(literal_content: &str) -> bool {
    let src = replace_placeholders(literal_content); // {ns}/{}/{fire_fn} → __ph__ so format! templates parse
    matches!(
        wat::parser::parse_one_with_file(&src, "<inline-wat-lint>"),
        Ok(WatAST::List(items, _)) if matches!(items.first(), Some(WatAST::Keyword(..)) | Some(WatAST::Symbol(..)))
    )
}
```
Write `#[cfg(test)]` unit tests asserting EXACTLY the DESIGN's cases: `(:wat::core::+ 2 3)`→true,
`(wat.core/if true 1 2)`→true (the faithful surface — this is the load-bearing new capability),
`(:{ns}::run-counts :wat::rete::{fire_fn})`→true, `"n::Bad"`→false, an ordinary Rust string→false.

## The scan (the gate)

Rewrite the corpus walk: for each `tests/**/*.rs` (skip the gate's own file), extract **string-literal
contents** (handle `"…"` with escapes + `\`-line-continuation, and `r#"…"#` raw; skip `//` and `/* */`
comments), run `is_inline_wat_form` on each; if any is a wat form AND the file has no
`// rune:lint(no-inlined-wat)` marker → offender. `assert!` offenders empty, listing them (the campaign meter).

## STOP triggers (halt + report; ship nothing)

- STOP if robust string-literal extraction (escapes / raw / continuation) can't be done cleanly → report. Do
  NOT ship an extractor that silently misses literal forms — a gate that under-detects reads as "covered" when
  it isn't.
- STOP if `parse_one_with_file` isn't callable from `tests/lint/` → report (it's `pub`; the lint target deps `wat`).
- Do NOT extract any test's inline wat to `.wat` and do NOT add runes across the corpus — that is the SWEEP, a
  separate campaign. This strike ships the GATE + its unit tests ONLY.
- Do NOT touch the rete engine, `wat/rete.wat`, or any non-test source.

## How to work

Build/run with `cargo test --release -p wat --test lint`. To iterate on the detector, run its unit tests
first (they're the spec). Then run the corpus scan — it WILL report a large offender count; that is the
expected-red meter, NOT a failure of your work.

## Done = green (for the detector) + honest-red (for the meter)

- The detector unit tests PASS (both surfaces + template + non-wat cases correct).
- The corpus scan RUNS and lists offenders. Report the offender COUNT + a breakdown if you can (how many files
  hit, roughly how many are format!-drivers vs parse_one bodies vs faithful-surface). Do NOT force it green by
  blanket-runing the corpus — the red is the honest state the sweep will drive down.
- No other lint test regresses.

Report: the detector unit-test results, the corpus offender count, files changed, and any STOP hits.
