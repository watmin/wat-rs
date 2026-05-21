# EXPECTATIONS — Arc 218 Stone 218.6 — L1 substrate fixes

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `write_char` supplementary-plane fix | `crates/wat-edn/src/writer.rs:307-318` — function rewritten so codepoint `> 0xFFFF` emits literal Unicode (`out.push(c)`) instead of overflowing `\uXXXX`. BMP control bytes + DEL keep `\uXXXX`. BMP non-control non-printable preserves existing escape behavior (no test regression on existing BMP char output). |
| 2 | Supplementary-plane round-trip probe | New test in `crates/wat-edn/tests/round_trip.rs` (or similar) asserting `parse(write(Value::Char('😀'))) == Value::Char('😀')`. Plus shape matrix extended: `:char-supplementary` shape in `interop-tests/src/bin/shape_matrix.rs` + `clj/consume_shapes.clj` assertion + `clj/produce_shapes.clj` produce + `src/bin/shape_matrix_reader.rs` assertion. |
| 3 | `JsonError::InvalidSet` variant + `decode_set` fix | `crates/wat-edn/src/json.rs:51` — new variant `InvalidSet(String)` added (near `InvalidMap`); any `Display`/`Debug` impl extended. `json.rs:376` — `decode_set` uses `JsonError::InvalidSet` instead of `InvalidMap`. |
| 4 | `writer.rs:78` operand swap | Line reads `} else if items.len() <= 8 && all_scalar(items) {` — O(1) check short-circuits the O(N) walk. |
| 5 | `is_canonical_uuid` parser → vocab | Function body + docstring moved verbatim from `parser.rs:455-471` to `vocab.rs` (near `validate_first_char`); `pub(crate)`. Two import sites updated: `parser.rs:297` (in-module or `use crate::vocab::is_canonical_uuid`) + `json.rs:36` (`use crate::vocab::is_canonical_uuid`). Comment at `parser.rs:473-474` updated or removed. |
| 6 | `translate_and_validate_ns` combiner + 6 paired-call sites | `vocab.rs` adds `pub(crate) fn translate_and_validate_ns(ns: &str) -> Result<String, &'static str>` (translate `::` → `.`, validate first char, return translated). `value.rs:218-220` `translate_wat_to_strict` deleted. 6 paired-call sites in value.rs collapsed to single-call form (panic-flavor uses `.unwrap_or_else(|m| panic!(...))`; try-flavor uses `?`). |
| 7 | rune `temperare(serde-api-shape)` additive | `crates/wat-edn/src/json.rs:165-172` + `:175-184` — second rune line added below the existing `struere(invariant-coupling)` rune on each of `to_json_string` + `to_json_string_pretty`. Wording cites: serde_json API shape, double-materialization trade-off, no caller pressure, simpler-wins-until-measurement disagrees. |
| 8 | `parse_wire` + `parse_wire_owned` retired | `crates/wat-edn/src/lib.rs:130-152` — both function bodies + docstring block deleted. `tests/wire_encoding.rs` — every call site migrated to `Parser::new_wire(x).parse_top()` (or `.parse_top().map(Value::into_owned)`); imports updated; doc comments rephrased. All 23 tests preserved. `crates/wat-edn/docs/USER-GUIDE.md` — `parse_wire`/`parse_wire_owned` removed from imports + paragraph; section heading restored to 3-entry-point form; wire-mode teaching routes reader to `Parser::new_wire`. |
| 9 | wat-edn test suite green | `cargo build --release -p wat-edn` 0 warnings / 0 errors. `cargo test --release -p wat-edn` PASS with expected count delta: **342 baseline** (verified pre-spawn 2026-05-22; 218.4 SCORE cited 339 but arc 219 added 3 spec_strict tests since) + 1 supplementary-plane probe + 1 `InvalidSet` probe (optional sonnet-add) = 343 or 344. Report actual count. |
| 10 | wat downstream test suite green | `cargo test --release --lib -p wat` PASS (824/0 baseline, verified pre-spawn 2026-05-22). No regressions in the downstream substrate. |
| 11 | clippy clean | `cargo clippy --release -p wat-edn -- -D warnings` 0 warnings / 0 errors. |
| 12 | Interop-tests 4 handshakes pass | All four commands from BRIEF "Verification" §5 return PASS. Shape matrix now includes the supplementary-plane char probe (proves the writer fix end-to-end through `clojure.edn/read`). |

## Independent prediction (calibration record)

**Target runtime:** 30-45 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- Seven substantive items spanning 5 source files + tests/ + USER-GUIDE + interop-tests bins/clj
- Items (b), (c), (f) are tiny (variant mint + operand swap + comment additions)
- Items (d), (e) are mechanical moves with locked target shape
- Item (a) has the most surprise risk (BMP test regression — STOP-1 — pre-grep confirms recommended shape preserves BMP behavior, but real-world tests can surprise)
- Item (g) is the largest mechanical edit (23 test sites in wire_encoding.rs + USER-GUIDE restructure + lib.rs delete)
- Interop-tests four handshakes are a deterministic verification gate
- Substrate-pre-grep dense: all line numbers confirmed; JsonError location confirmed (line 51, not error.rs); 6 paired sites confirmed via grep; wire_encoding.rs structure confirmed (35 lines hit by `parse_wire|Parser::new_wire|fn `)
- Calibration five-for-five below band: 218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20, 219.1 below band. This stone is heavier — band raised to 30-45 (vs 218.4's 20-40)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.4 SCORE (~20 min ship; 9 items including 2 substrate + 2 probes). 218.6 has ~1.7× the substantive items + an interop-tests probe + a 23-test migration. Confidence high; band 30-45.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- L2 findings (struere extracts, solvere placement-2/3, cernere docs + escape-case, temperare perf rune candidates, intueri renames, purgare public-API runes) — Stone 218.7
- INSCRIPTION + re-cast vigilia — Stone 218.5 (redefined)
- Touching tagged-literal naming or `:wat::` syntax — arc 216.8/.9/.10 territory; locked per encoding doctrine
- New enum variants beyond `JsonError::InvalidSet` — preserve scope discipline
- New public surface — retire-then-mint discipline; arc 217 mints back when caller surfaces
- Performance optimization beyond the operand-swap (Item 4) — Stone 218.7 weighs the L2 perf candidates
- DESIGN.md / INTERSTITIAL / CLIFFNOTES amendments — orchestrator-direct after sonnet ships

## Honesty deltas accepted

- `write_char` exact shape — sonnet picks a clean form that fixes the overflow AND preserves BMP behavior; recommended structure in BRIEF Part (a) is one valid form, not the only one
- Probe placement (round_trip.rs vs new spec_supplementary.rs vs writer's internal `#[cfg(test)]`) — sonnet picks based on existing conventions in `crates/wat-edn/tests/`
- Shape matrix probe naming (`:char-supplementary`, `:emoji-round-trip`, etc.) — sonnet picks
- `is_canonical_uuid` import form — `use crate::vocab::is_canonical_uuid` at top of parser.rs vs inline `crate::vocab::is_canonical_uuid` at call site; sonnet picks
- `translate_and_validate_ns` panic-message wording — sonnet preserves diagnostic intent; exact prose may shift
- USER-GUIDE wording for the section restructure — sonnet picks the cleanest explanation that preserves Stone 218.4's wire-mode teaching
- Test count: report actual. May be 342 + 1 (just supplementary-plane probe) or 342 + 2 (supplementary-plane + an `InvalidSet` decode probe to lock the variant)
- Rune wording for `temperare(serde-api-shape)` — sonnet preserves the named axis (serde-API constraint, double-mat trade-off, no caller pressure); exact phrasing may shift

## Honesty deltas NOT accepted

- Skipping any L1 fix — STOP. Substrate trust is binary; partial closure isn't IMPECCABLE.
- Skipping the interop-tests handshakes — STOP. Per `feedback_wat_edn_touch_runs_interop_tests`, this is the cross-language proof gate, not optional.
- Changing the rune-discipline disposition (e.g. runing parse_wire instead of retiring; deleting the double-mat code instead of runing it) — these were user-charged decisions; respect them.
- Adding NEW public surface beyond what's already there — retire-only-then-restore-on-demand discipline.
- Renaming `is_canonical_uuid` during the move — that's a Stone 218.2 territory rename; preserve the name.
- Bypassing tests/clippy/handshakes — never.
- Touching scope beyond the 7 substantive items — STOP at the boundary.
