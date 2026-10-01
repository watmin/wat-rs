# WEIGH — STONE 255.80: the recorded codemods reach wat inside Rust string literals — ACCEPTED

**Executor: a Sonnet subagent, commits `e28d03236` (probe + shared module), `b184330b3` (driver + apply), `0982145d2`
(cures), `5b4d963b2` (SCORE).** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator at `5b4d963b2`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T21-57-10Z`: **6362 passed / 24 skipped**, exit 0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| the driver over every tracked `.rs` (dry run) | `./target/release/wat-fix-rust wat-scripts/fixes/types-to-wat-type.wat --dry-run --list <1218 paths>` → `1218 file(s) scanned, 15 changed, 64 edit(s) applied, 0 refused`, tree unchanged: the 15 are exactly the files deliberately held back (below) |

## What landed

- **`wat::embedded_wat`** (`src/embedded_wat.rs`): the lint's tested literal extractor, factored out, plus a decoded→raw
  offset map and a length-preserving `format!` placeholder stand-in. `no_inlined_wat_in_tests` uses it; its unit tests are
  unchanged and green.
- **`wat::codemod_driver`** and the `wat-fix-rust` binary: runs **any** recorded codemod over every wat-shaped literal in
  a set of `.rs` files, one batch per file, verifies each region's raw slice before splicing, refuses otherwise.
- **Applied:** 506 edits in 39 files, 0 refused. The probe (committed before the tool) proved the composition through
  three line continuations and an escaped placeholder in `src/macros/parse.rs` (the brief's suggested file had no such
  literal; the code won).
- A test-only `check()` helper in `src/check.rs` skipped `freeze.rs`'s `normalize_symbol_refs` step; cured in order.

## Held back for 4b (the 15 files, 64 edits)

Literals that byte-match something still old-spelled: 12 compare against `format_type`'s rendered `CheckError` text, 2
call rete `lower()` directly on raw un-normalized AST (whose dispatch is keyword-only: **the K1 door-bypass class**, a
finding for 4b), 1 mirrors a doc comment. They convert when 4b makes types print as written.

## Notes

- The dry-run summary says "edit(s) applied" when nothing was written: a label that states the wrong conclusion. Fix in
  4b's touch of the driver.
- Four `rune:lint(loose-assert)` exemptions were added on new code, each with a reason.

## Verdict

Accepted and pushed. The cutover's remaining stones (4b, 5, 7) and 251's terminal cut can now reach embedded wat.
