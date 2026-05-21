# EXPECTATIONS — Arc 218 Stone 218.3 — Contract precision

Mode A target: 11/11 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Pretty-print map symmetric with vector | `crates/wat-edn/src/writer.rs:106-125` — map arm matches vector pattern: open `{` + `\n`, `push_indent` before EVERY entry, `\n` + `push_indent(level)` + `}` to close. Output structurally consistent with vector pretty-print. |
| 2 | Pretty-print test expectations updated | All pretty-print map snapshots in `crates/wat-edn/tests/` updated to the new symmetric format. Test count holds at 336/336 (snapshot updates counted, not regressions). |
| 3 | `to_json_string` rune annotation | `crates/wat-edn/src/json.rs:162-163` — `// rune:struere(invariant-coupling)` annotation explaining the `.expect()` is structurally unreachable per edn_to_json's closed construction |
| 4 | `to_json_string_pretty` rune annotation | `crates/wat-edn/src/json.rs:167-170` — matching rune on the pretty variant (same invariant) |
| 5 | `parse_map_key` strict mode | `crates/wat-edn/src/json.rs:279-294` — EDN-looking keys that fail to parse return `Err(JsonError::InvalidMapKey { ... })` instead of silently falling through to `Value::String`. New variant added to `JsonError` enum if not present. |
| 6 | Probe for parse_map_key strict mode | One test added (json-side test file of sonnet's choice — `comprehensive.rs` / `spec_conformance.rs` / new file iff needed): EDN-looking key that fails to parse → returns `JsonError::InvalidMapKey`, not `Value::String` |
| 7 | Closer-token diagnostic split | `crates/wat-edn/src/parser.rs:158-160` — `Token::Eof` keeps `UnexpectedEof`; `Token::RParen` / `Token::RBracket` / `Token::RBrace` get separate `UnexpectedToken` (or `UnexpectedByte` if cleaner per existing patterns). Each closer surfaces a distinct diagnostic. |
| 8 | `lexer.rs:213` allocation tightened | `crates/wat-edn/src/lexer.rs:213` — `String::with_capacity(self.pos - body_start)` (already-consumed body length) instead of `self.input.len() - body_start` (whole-input length) |
| 9 | Identifier suffix scan fold | `crates/wat-edn/src/parser.rs:382-391` — two-scan (`body.find('/')` + `name.contains('/')`) folded into single `splitn(3, '/')` pass; all four error messages preserved exactly ("empty prefix in", "empty name in", "more than one / in", per-side validate_first_char wrapping) |
| 10 | wat-edn test suite: zero regressions | `cargo test --release -p wat-edn` 336/336 PASS (pretty-print snapshot updates counted as updates not regressions); `cargo clippy --release -p wat-edn -- -D warnings` clean |
| 11 | SCORE doc inscribed | `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.3.md` — scorecard + deltas (vigilia file/line citation corrections; pretty-print snapshot update count; variant decisions) + verification + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 40-65 min Mode A
**Upper bound:** 90 min
**Confidence:** medium-high

**Rationale:**
- 6 substantive items, each independent
- Pretty-print map symmetrization is the largest single change (code edit + snapshot update sweep across pretty.rs + maybe comprehensive.rs)
- Two `JsonError` / `ErrorKind` variant additions (Items 5 + 7) — small enum additions; sonnet picks shape based on existing patterns
- Items 3 + 4 + 8 are one-liner rune/code adjustments
- Item 9 is a single-function refactor with strict error-text preservation
- Substrate-pre-grep is dense (all line numbers confirmed; off-by-one notes documented; vigilia file path correction noted)
- Risk: pretty-print snapshot sweep is larger than apparent (STOP-1 trigger if behavior tests break)
- Risk: `JsonError::InvalidMapKey` variant addition ripples to multiple call sites (STOP-2)
- Risk: identifier suffix fold subtle error-text drift (STOP-4)
- Calibration trend: 218.1 actual ~20 (band 25-45); 218.2 actual ~15 (band 30-50). This stone has more design-touch surface than naming sweep; widening to 40-65.

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.2 SCORE (~15 min ship; substrate-pre-grep + mechanical sweep pattern). 218.3 adds two enum variant additions + a behavioral format change (pretty-print map) on top of the mechanical baseline. Confidence band widens proportionally.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- UUID strictness changes — Stone 218.4
- USER-GUIDE map format claim correction — Stone 218.4 (interacts with this stone's symmetrization; 218.4 picks up after 218.3 lands)
- `parse_wire` / `parse_wire_owned` docs — Stone 218.4
- Public-API runes for forward-declared re-exports — Stone 218.5
- INSCRIPTION + re-cast vigilia — Stone 218.5
- Touching tagged-literal naming (Some/None/Ok/Err) — those are arc 216.8/.9 territory; out of arc 218 entirely
- Performance work beyond allocation cap + scan fold — surfaced items only
- DESIGN.md / INTERSTITIAL amendments — orchestrator-direct

## Honesty deltas accepted

- Variant naming: `JsonError::InvalidMapKey` vs `JsonError::InvalidKey` etc. — sonnet picks based on sibling variants; documents
- `ErrorKind::UnexpectedToken` shape: `(&'static str)` vs `(String)` vs reuse `UnexpectedByte(u8)` — sonnet picks based on existing enum patterns; documents
- Pretty-print snapshot update count: however many tests assert the map format — sonnet sweeps all; documents the count
- Vigilia citation deltas (`writer.rs:162-170` actually in `json.rs:162-170`; `parser.rs:158` actually `:159`) — already documented in BRIEF; SCORE rolls them up
- Test file placement for the new probe (Item 6) — sonnet picks an existing test file or mints a new one if the test family is genuinely new

## Honesty deltas NOT accepted

- Skipping the pretty-print map symmetrization — STOP. Decision locked via four-questions YES×4 in BRIEF.
- Skipping the parse_map_key strict mode — STOP. Decision locked via four-questions in BRIEF.
- Bypassing tests/clippy — never; 336/336 must hold (snapshot updates counted)
- Changing identifier suffix error TEXT — STOP-4 trigger; error messages must match exactly
- Removing the rune annotations — they're the load-bearing artifact for Items 3 + 4
- Touching scope beyond the 6 items — STOP at the boundary
