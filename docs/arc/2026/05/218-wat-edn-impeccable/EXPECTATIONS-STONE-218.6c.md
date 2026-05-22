# EXPECTATIONS — Arc 218 Stone 218.6c — Toward impeccable: fixes + demotions + rune rebalance

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `is_scalar` BigInt/BigDec arms added | `crates/wat-edn/src/writer.rs:45-58` — `matches!` includes `Value::BigInt(_) \| Value::BigDec(_)`. Pretty-printer now inlines BigInt + BigDec inside small scalar-only collections, matching their atomic nature. |
| 2 | Silent-Null fallback replaced | `crates/wat-edn/src/json.rs:119-123` — `.unwrap_or(JV::Null)` → `.expect("finite f64 must convert to serde_json::Number per from_f64 contract")`. The closed-construction guarantee the struere rune protects now truly holds. |
| 3 | USER-GUIDE ErrorKind listing complete | `crates/wat-edn/docs/USER-GUIDE.md:805-822` — `UnexpectedToken(&'static str)` and `Utf8(String)` added in their natural positions. Listing now matches `error.rs:14-37` enum body. |
| 4 | USER-GUIDE JsonError listing complete | `crates/wat-edn/docs/USER-GUIDE.md:824-837` — `InvalidSet(String)` and `InvalidMapKey { key: String, reason: String }` added. Listing now matches `json.rs:51-95` enum body. |
| 5 | USER-GUIDE pretty-print example regenerated | `crates/wat-edn/docs/USER-GUIDE.md:457-463` — example replaced with verbatim output produced by running the actual `write_pretty` on a parseable fixture. Format: 2-space indentation per level; closing `}`/`]` on its own line at outer level; nested collections break at 2-space increments. |
| 6 | `edn_to_json` demoted to pub(crate) | `crates/wat-edn/src/json.rs:100` — `pub fn` → `pub(crate) fn`. `crates/wat-edn/src/lib.rs:84-87` — `edn_to_json` removed from `pub use json::{...}` list. Function remains usable internally by `to_json_string` + `to_json_string_pretty`. |
| 7 | `json_to_edn` demoted to pub(crate) | `crates/wat-edn/src/json.rs:204` (or correct line) — `pub fn` → `pub(crate) fn`. `crates/wat-edn/src/lib.rs:84-87` — `json_to_edn` removed from `pub use json::{...}`. Function remains usable internally by `from_json_string`. |
| 8 | `purgare(public-api)` rune on `to_json_string_pretty` | `crates/wat-edn/src/json.rs:185` (above `pub fn`) — new rune comment block citing: symmetric pretty variant of actively-consumed `to_json_string`; arc 116 cargo integration; impressive JSON bridges ship both forms; natural API for human-readable output. Placed BEFORE existing struere + temperare runes. |
| 9 | `purgare(public-api)` rune on `write_to` | `crates/wat-edn/src/writer.rs:187` (above `pub fn write_to`) — new rune comment block citing: buffer-reuse ergonomic; symmetric with `write`; documented in `crates/wat-edn/docs/IPC-BRIDGE.md:95`; canonical Rust pattern for output composition. |
| 10 | 2 struere runes deleted | `crates/wat-edn/src/json.rs:167-172` (above `to_json_string`) + `:177-182` (above `to_json_string_pretty`) — both `// rune:struere(invariant-coupling) — ...` 5-line comment blocks deleted. temperare blocks PRESERVED intact at `:175-181` and `:185-191` (or whatever the post-delete line numbers become). |
| 11 | All test suites + clippy green | `cargo build --release -p wat-edn` 0 warnings. `cargo test --release -p wat-edn` PASS at 344 (no count change; all fixes correctness-preserving). `cargo test --release --lib -p wat` PASS 824/0. From `crates/wat-edn/interop-tests/`: `cargo build --release` 0 warnings + `cargo clippy --release --all-targets -- -D warnings` 0 warnings. |
| 12 | Interop-tests 4 handshakes pass | All four handshakes (consume.clj / reader / consume_shapes.clj / shape_matrix_reader) PASS. NOTE: if sub-agent piped-bash permission denies, sonnet ships and reports; orchestrator runs the 4 during independent scoring per Stone 218.6b precedent. |

## Independent prediction (calibration record)

**Target runtime:** 20-30 min Mode A
**Upper bound:** 45 min
**Confidence:** high

**Rationale:**
- 10 items spanning 4 source files (writer.rs, json.rs, lib.rs, USER-GUIDE.md)
- Part A is mechanical except A.3c (regen pretty-print example — requires actually running the writer)
- Part B.6+7 (visibility demotion) is 2-keyword change + 1 line edit in lib.rs
- Part B.8+9 (rune addition) is verbose-but-mechanical comment insertion
- Part C.10 is delete after A.2 verified
- Substrate-pre-grep dense: all line numbers confirmed; lib.rs `pub use` block at :84-87 verified; json.rs silent-Null fallback at :119-123 verified; writer.rs is_scalar at :45-58 verified
- Calibration eight-for-eight below band: 218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20, 219.1 below, 218.6 ~8, 218.6b ~6, [218.6c TBD]. Pattern: substrate-pre-grep + locked-decisions + mechanical = below band. This stone's surprise risk is Part A.3c (pretty-print regen could surface fixture choice ambiguity)

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 218.6b SCORE shipped at ~6 min combined (sonnet + orchestrator handshakes). 218.6c has ~50% more items + one regen step. Band 20-30 conservative.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- L2 sweep (struere L2 × 3, intueri L2 × 6, temperare L2 × 2, cernere L2 × 2, solvere L2 × 1) — Stone 218.6d
- INSCRIPTION + arc 218 closure conversation — deferred per user direction
- Touching `to_json_string` or `from_json_string` (the live consumer-facing APIs) — they stay exactly as they are
- Adding NEW public surface beyond the two rune-justified items
- `JsonError` + `JsonResult` — they STAY in the `pub use json::{...}` re-export list; they're the error type surface that `from_json_string`'s `JsonResult<OwnedValue>` return forces consumers to import
- Touching encoding doctrine / wat-edn syntax
- Performance optimization beyond surfaced items

## Honesty deltas accepted

- `is_scalar` recommended addition is `Value::BigInt(_) \| Value::BigDec(_)` per pattern; sonnet may add an explanatory comment if it makes the matches! block clearer
- `.expect()` message wording — sonnet preserves the "finite f64 must convert" + "per from_f64 contract" intent; exact phrasing may shift
- USER-GUIDE pretty-print example fixture choice — BRIEF suggests one approach; sonnet picks the cleanest illustrative case (must produce REAL output from `write_pretty`, not hand-crafted)
- Rune wording for `purgare(public-api)` — sonnet preserves the named justification (symmetric variant + named downstream + engineering completeness); exact phrasing may shift
- `to_json_string_pretty` rune placement: BRIEF says "BEFORE existing struere + temperare"; sonnet may verify that ordering is what the spell expects (placement is "immediately preceding the construct" per SKILL; multiple rune blocks can stack)

## Honesty deltas NOT accepted

- Skipping any Part A fix — STOP. The L1s must close.
- Skipping the silent-Null fix (Part A.2) — STOP. It's what makes Part C.10 honest.
- Skipping Part C.10 — STOP. The struere runes are LIES today; either Part A.2 truthifies them (and they can be removed since `.expect()` speaks for itself) OR they stay as lies (unacceptable).
- Keeping `edn_to_json` or `json_to_edn` as `pub` "just in case" — STOP. The discipline is retire-then-mint-on-demand; both have zero external consumers.
- Deleting the temperare runes — STOP. They name a real architectural trade-off per user direction 2026-05-21.
- Adding NEW runes beyond Parts B.8 + B.9 — STOP. The high-bar discipline limits new runes to those two strongly-justified additions.
- Touching `to_json_string` or `from_json_string` (live consumer APIs) — STOP. These stay exactly as they are.
- Bypassing tests/clippy/handshakes — never.
- Scope beyond the 10 substantive items — STOP at the boundary.
