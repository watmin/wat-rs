# DESIGN — reader-based inline-wat gate (surface-agnostic)

**Status:** STRIKE-READY. Provenance: `NOTE-test-cleanup-revealed-by-rete.md`.
**Replaces:** the `startup_from_source(` needle in `tests/lint/no_inlined_wat_in_tests.rs` (world-only,
surface-specific — blind to inline *drivers* and to faithful-Clojure `(wat.core/…)`).

## Why

Two blind spots in the current gate: (1) it only catches inline *worlds* (`startup_from_source`), not inline
*drivers* (`let run = format!("(:wat…")`); (2) its `(:` shape is **surface-specific** — 300 converts inline
wat to `(wat.core/defn …)`, which has no `(:` prefix, so the gate goes blind to exactly the code 300 ships
(verified: `(:[^:]` misses `(wat.core/if …)`; 8 files already carry faithful forms).

## The idea — detect by the reader, not by a surface regex

A string is inline wat iff **wat's own reader reads it as a form**. Feed each string literal to
`wat::parser::parse_one_with_file`; if it parses to a **list whose head is a `Keyword` or `Symbol`**, it is a
wat form. Surface-agnostic by construction: the one reader accepts both `(:wat::core::…)` and `(wat.core/…)`
during the dual-surface period, and after the retire `(:wat::core::…)` becomes a parse *error* — so the gate
follows the language automatically, zero maintenance. This dogfoods 300's `VNVS LECTOR` in the gate itself.

## The detector contract (the unit-testable core)

```
fn is_inline_wat_form(literal_content: &str) -> bool:
    let src = replace_placeholders(literal_content)   // {ns} / {} / {fire_fn} → __ph__ (so format! templates parse)
    match parse_one_with_file(&src, "<inline-wat-lint>"):
        Ok(WatAST::List(items, _)) if head_is_keyword_or_symbol(items) => true
        _ => false
```

Unit-test cases (the spec — must all hold):
- `(:wat::core::+ 2 3)` → **true**  (rust-scheme, keyword head)
- `(wat.core/if true 1 2)` → **true**  (faithful-Clojure, symbol head — the surface 300 ships)
- `(:{ns}::run-counts :wat::rete::{fire_fn})` → **true**  (format! template — placeholders substituted)
- `"n::Bad"` → **false**  (a bare query type-string, not a form)
- `hello world` / `expected i32, got String` → **false**  (ordinary Rust string)
- `//! (:wat::core::char) …` content → not reached (comment-skipping handles it — see scan)

## The scan (the gate)

For each `tests/**/*.rs` except the gate's own file:
1. Extract string-literal **contents** — `"…"` (unescaped) and `r#"…"#` (raw), skipping `//` line comments and
   `/* … */` blocks. (This is the fiddly part: escapes, `\`-line-continuations, raw strings.)
2. Run `is_inline_wat_form` on each literal.
3. If any literal is a wat form AND the file lacks `// rune:lint(no-inlined-wat)` → the file is an offender.
4. Fail listing offenders (the campaign meter — expected-red until the sweep drives it to zero, exactly like
   the current gate's "ONE expected-red test" convention; nextest isolates it, a SECOND red is a real regression).

## The one contract decision

**File-level rune, this strike** (matches the existing gate; simplest). Per-site rune (a marker on the
offending line) is the tightening the builder wants ("declare why for *them*") — named as the immediate
follow-on, `inline-wat-per-site-rune`, once the file-level gate + sweep land. Not bundled here.

## Out of scope = rejected (named)

- **The sweep itself** — extracting the flagged files to `.wat` / adding earned runes is the drive-to-zero
  *campaign* the gate meters; it is NOT this strike. This strike ships the GATE (the meter) + its unit tests.
- **Per-site rune granularity** — banked `inline-wat-per-site-rune` (the immediate follow-on).
- Touching the rete engine, `wat/rete.wat`, or any non-test source.

## STOP triggers

- STOP if robust Rust string-literal extraction (escapes + raw + `\`-continuation) proves infeasible cleanly
  → report; do not ship an extractor that silently misses literal forms (a gate that under-detects is worse
  than none — it reads as "covered" when it isn't).
- STOP if `parse_one_with_file` is not callable from `tests/lint/` → report (it is `pub` in `crates/wat-reader`;
  the lint target already depends on `wat`).

## Files

- `tests/lint/no_inlined_wat_in_tests.rs` — the detector fn + its `#[cfg(test)]` unit tests + the rewritten
  corpus scan. (Rename its title/doc to reflect reader-based detection; keep the `no-inlined-wat` rune name.)
- `docs/arc/2026/06/278-rules-engine/NOTE-test-cleanup-revealed-by-rete.md` — the provenance (committed).

## Done = green

- The detector unit tests pass (both surfaces detected, non-wat + template cases correct).
- The corpus scan RUNS and lists offenders — its red count is the campaign meter (expected-red is acceptable
  and documented; do NOT rune-blanket the whole corpus to force green — the red IS the honest state).
- No other test regresses (the scan is isolated; nextest runs the rest green).
