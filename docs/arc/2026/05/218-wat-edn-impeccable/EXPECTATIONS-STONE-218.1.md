# EXPECTATIONS — Arc 218 Stone 218.1 — L1 fixes + cross-spell convergence

Mode A target: 9/9 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | cernere L1.A — USER-GUIDE.md:159 rewritten | `crates/wat-edn/docs/USER-GUIDE.md` example replaces `p.parse_next()?` loop with `Parser::new(input).parse_all()?` vec-iteration; surrounding teaching intent preserved; no other API names changed |
| 2 | cernere L1.B — IPC-BRIDGE.md:212 rewritten | `crates/wat-edn/docs/IPC-BRIDGE.md` prose accurately describes the real Parser API (`new` / `new_wire` / `parse_top` / `parse_all`); phantom `parse_next` reference gone |
| 3 | temperare L1 — lexer.rs:346-347 single-iterator | `crates/wat-edn/src/lexer.rs` char-literal single-char arm uses one `chars()` iterator with `.next()` then `.next().is_none()` check; semantically identical to original; one traversal instead of two |
| 4 | escapes.rs gains `write_keyword_body_to` | `crates/wat-edn/src/escapes.rs` adds `pub(crate)` or `pub` fn `write_keyword_body_to<W: std::fmt::Write>(seg: &str, w: &mut W) -> std::fmt::Result`; docstring cites arc 218 stone 218.1 extraction + arc 170 REALIZATIONS pass 14 lineage |
| 5 | value.rs collapses to shared helper | `crates/wat-edn/src/value.rs` deletes local `write_keyword_segment`; callers at lines ~440, ~443 route through `escapes::write_keyword_body_to` |
| 6 | writer.rs collapses to shared helper | `crates/wat-edn/src/writer.rs` deletes local `write_keyword_body`; callers at lines ~163, ~166 route through `escapes::write_keyword_body_to`; `String` infallibility honored cleanly |
| 7 | `display_equivalence.rs` test still PASSES | The byte-identical lock between Display + writer paths survives the extraction — structural proof of semantic preservation |
| 8 | wat-edn test suite: zero regressions | `cargo test --release -p wat-edn` green; `cargo clippy --release -p wat-edn -- -D warnings` no new warnings |
| 9 | SCORE doc inscribed | `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.1.md` — scorecard + deltas + verification + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 25-45 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- Three concrete pieces; each mechanical
- L1.A + L1.B: doc rewrites with surrounding-context preservation; ~10 min combined
- temperare L1: single 5-line code swap; ~3 min
- Cross-spell extraction: helper signature is dictated by `fmt::Write` unification; two call-site collapses are mechanical; `display_equivalence.rs` provides the verification mechanism; ~15 min including verification iteration
- Substrate citations all confirmed pre-spawn (orchestrator grep at 2026-05-21); no STOP-1/2/3 expected
- Risk: `fmt::Write` lifetime issue (low — both `Formatter` and `String` are mature impls; STOP-3 trigger if it surfaces)
- Risk: doc example intent unclear after context read (low — STOP-4 trigger if it surfaces)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites VIGILIA-REPORT-2026-05-21.md (the worklist) + SCORE-STONE-216.7.md (most recent calibration: 45 min actual, 45-75 min predicted band, lower end). Pattern lineage: mechanical-extraction + doc-rewrite; smaller surface than 216.7's substrate edits.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- `escapes.rs` rename to `vocab.rs` — Stone 218.2
- Lexer variable renames (`e` → `escape_byte` etc.) — Stone 218.2
- All L2 findings from vigilia report — distributed across 218.2/218.3/218.4
- DESIGN.md amendments — orchestrator-direct (none expected for this stone)
- INSCRIPTION-218.md — Stone 218.5

## Honesty deltas accepted

- Helper visibility (`pub` vs `pub(crate)`) — sonnet picks based on call-site scope; documents
- Exact spelling of the lexer single-iterator pattern — sonnet picks cleanest; documents if it deviates from BRIEF's suggested shape
- Rune annotation choice on writer.rs `.expect()` — sonnet picks (optional one-liner; not load-bearing)
- Doc example body shape — sonnet preserves teaching intent; documents if structural change was needed (e.g., a loop-shape that became a vec-iter changes the prose flow)

## Honesty deltas NOT accepted

- Probe / row substitution — STOP-3 trigger (no test subjects to substitute here; structural)
- Touching `escapes.rs` rename — Stone 218.2 territory; STOP if tempted
- Renaming variables in lexer.rs beyond the L1 fix — Stone 218.2 territory
- Bypassing the `display_equivalence.rs` failure — STOP-1 trigger; honest report of the regression, not a workaround
- Extending scope to any L2 finding — STOP at the L1 boundary; surface and pause
