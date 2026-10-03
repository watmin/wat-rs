# WEIGH — STONE 255.85: cutover 5c-i (a) — the conversion tooling — ACCEPTED

**Executor: grok via pulsare, commits `b07f09866` … `85e51366f`, SCORE `fb505b434`.** Weighed by the orchestrator on
2026-10-03.

## Re-run by the orchestrator at `fb505b434`

| row | result |
|---|---|
| release floor | `.floor/2026-10-03T02-01-32Z`: **6394 passed / 24 skipped**, exit 0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 (the IDE's two `dead_code` warnings were stale) |

## What landed

- **The converter's type rule asks the closed set:** only the 24 (and `AST`) become `wat.type/`; `Error` stays
  `wat.core/Error`, `UUID` stays `wat.uuid/UUID`. The same fix in the printer (255.77's finding) moved three goldens from
  `wat.type/Seqable` to `wat.core/Seqable` (a non-member printed under its home: correct).
- **The seven text-reading gates read declarations by identity**, green on the main tree and on a clone with a converted
  stdlib (44/44 there).
- **The delta sample** dropped the file 255.82 deleted (not rebuilt).
- **`wat-fix-rust` is resumable and reads only programs:** a failing batch is retried per literal, failures recorded, the
  run ends with its summary; prose in Markdown backticks is not a candidate.
- **The two `:fn(` fixtures** are negative proofs of the retired form; the converter now leaves that keyword alone instead
  of aborting the file.

## The re-measure (grok, fresh clone at `e935b3093`)

- Corpus: **2219 OK, 0 FAIL** (86,593 heads → 0).
- Embedded dry run: `1301 file(s) scanned, 103 changed, 6559 edit(s) found, 18 refused, 1 codemod-failed`. **All 18
  refusals are `format!` placeholders inside a keyword** (`:{ns}::seed`, `:wat::rete::{fire_fn}`, `:casc::Stage{k}`):
  text cannot respell a name the template builds. The one failure is a `:Any` in `src/freeze.rs` (`AnyBanned`). Both are
  5c-iv's to answer.

## Noted

- On an old replay fixture holding the retired `:wat::core::Tuple(i64)` keyword, the converter emits `wat.core/Tuple(i64)`
  rather than leaving the retired form alone (as it now does for `:fn(`).

## Verdict

Accepted and pushed. Next: 255.86 (G1 and R-a), then 5c-ii.
