# EXPECTATIONS — Arc 218 Stone 218.6d — L2 sweep (13 mechanical fixes)

Mode A target: 14/14 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `vocab::split_namespaced` extracted (solvere L2) | `crates/wat-edn/src/vocab.rs` — new `pub(crate) fn split_namespaced(body: &str) -> Option<(&str, &str)>` added (alongside `validate_first_char` + `translate_and_validate_ns` + `is_canonical_uuid`). `crates/wat-edn/src/json.rs:258-262, 379-384, 424-430` — all 3 slash-split sites collapsed to single call. |
| 2 | `sentinel` → `single_key_object` rename (intueri L2) | `crates/wat-edn/src/json.rs:222` — function renamed + all internal call sites updated (likely 8-10 sites within json.rs based on grep). |
| 3 | `parse_map_key` → `decode_map_key` rename (intueri L2) | `crates/wat-edn/src/json.rs:304` — function renamed + call site at `:585` (test name) + any other internal references updated. |
| 4 | `open_pos` → `quote_start` param rename (intueri L2) | `crates/wat-edn/src/lexer.rs:212` — `lex_string_escaped` parameter renamed; body usages updated; the call site at `:206` updated. |
| 5 | `is_scalar` → `is_inline_value` rename + Tagged WHY (intueri L2) | `crates/wat-edn/src/writer.rs:47` — function renamed; WHY comment added explaining `Value::Tagged` is absent because the tagged variant has a dedicated arm in `write_pretty_indented`. Caller(s) within writer.rs updated (likely `all_scalar` at `:64` + `write_pretty_indented` at `:78`). |
| 6 | `parse_value` / `parse_value_inner` restructure (intueri L2) | `crates/wat-edn/src/parser.rs:98-107` — either rename `_inner` → `_discarding` (Option α) OR inline + surface `discarding: bool` at call sites (Option β). Docstring preserved on whichever fn carries the contract. |
| 7 | tests/comprehensive.rs compressed write+parse fixed (intueri L2) | `crates/wat-edn/tests/comprehensive.rs:1063, 1218-1219, 1224-1225` — 3 sites use existing `round_trip` helper at `:1124` instead of inline `let s = write(&v1); let v2 = parse(&s).unwrap();`. |
| 8 | `all_variants` → `LazyLock` (temperare L2) | `crates/wat-edn/tests/accessors.rs:13` — function replaced (or backed) by `static ALL_VARIANTS: LazyLock<Vec<(&'static str, Value<'static>)>>`; 18 call sites borrow the static. Net effect: 36 Box allocs → 2 (one Box<BigInt> + one Box<BigDecimal>) per binary run. |
| 9 | tests/wire_encoding.rs double-write fixed (temperare L2) | `crates/wat-edn/tests/wire_encoding.rs:263, 271, 281, 231, 249` — 5 sites restructured to avoid the redundant internal `write` in `roundtrip_wire`. Either signature changes (`roundtrip_wire_str`) OR sites inline the assertion. Test count preserved at 23. |
| 10 | lexer.rs:309 double-peek fixed (struere L2) | `crates/wat-edn/src/lexer.rs:296-313` — `lex_char` captures `first` at the first peek site; second `self.peek().unwrap()` replaced by direct use of captured value. |
| 11 | parser.rs:111 pos-lag WHY comment added (struere L2) | `crates/wat-edn/src/parser.rs:111` — comment added explaining the pos capture invariant (peeked-token lag; Keyword arm uses body_start; other arms accept the trade-off). |
| 12 | USER-GUIDE i64 range row fixed (cernere L2) | `crates/wat-edn/docs/USER-GUIDE.md:383` — row updated from `i64 (> 2^53)` to `i64 (out of ±2^53 range)` or equivalent symmetric wording; example range-honest. |
| 13 | USER-GUIDE serde claim rephrased (cernere L2) | `crates/wat-edn/docs/USER-GUIDE.md:740` — aspirational serde claim either removed or rephrased as a future consideration (no present-tense "is available"). |
| 14 | All tests + clippy + handshakes green | `cargo build --release -p wat-edn` 0 warnings. `cargo test --release -p wat-edn` 344 PASS. `cargo test --release --lib -p wat` 824/0. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0. From `interop-tests/`: clippy clean + 4 handshakes PASS (orchestrator-side if sub-agent permission wall hits, per 218.6c precedent). |

## Independent prediction (calibration record)

**Target runtime:** 25-40 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- 13 items spanning 5 source files + 3 test files + USER-GUIDE
- Mostly mechanical (renames cascade within single file; sites confirmed)
- Largest mechanical step: A.1 (helper extract + 3-site update) + C.8 (LazyLock conversion of 18 call sites)
- Substrate-pre-grep dense: all line numbers verified post-218.6c via grep (parse_value at :98, parse_value_inner at :107, sentinel at :222, parse_map_key at :304, is_scalar at :47, open_pos at :212, all_variants at :13, roundtrip_wire at :40, USER-GUIDE rows at :383 + :740)
- Calibration nine-for-nine below band: 218.1-219.1 ~15-25, 218.6 ~8, 218.6b ~6, 218.6c [TBD]. This is largest item count yet (13 vs 218.6c's 10). Band raised to 25-40 conservatively

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 218.6c shipped at minutes-not-hours; 218.6d has 30% more items + 1 LazyLock conversion + 1 helper extract. Confidence high; band 25-40.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- INSCRIPTION + arc 218 closure — deferred per user direction
- New runes — discipline forbids; all 13 items get root-fixes
- Touching existing runes (2 temperare + 2 purgare) — they stay intact
- New public surface
- Encoding doctrine / syntax changes
- Performance optimization beyond C.8 + C.9 (the temperare-surfaced items)

## Honesty deltas accepted

- B.2 / B.3 / B.4 / B.5 / B.6 exact rename target — sonnet picks among the suggested alternatives based on what reads cleanest at the call sites
- B.5 WHY comment exact wording — sonnet preserves the "Tagged has its own arm in write_pretty_indented" intent
- B.6 Option α vs β — sonnet picks; both honest
- C.8 `LazyLock<Vec<...>>` precise shape — slice borrowing pattern may need `&'static` vs `'static` lifetime gymnastics; sonnet picks the cleanest Rust
- C.9 Option α vs β — sonnet picks; both eliminate the double-write
- D.10 lex_char refactor exact shape — sonnet picks the cleanest single-peek form
- D.11 pos-lag comment wording — sonnet preserves the surfaced invariant
- E.12 wording for the i64 range row — sonnet picks symmetric phrasing
- E.13 wording for the serde claim — sonnet picks future-tense rephrasing OR full removal

## Honesty deltas NOT accepted

- Skipping any L2 — STOP. All 13 must close for vigilia recast to have a chance at CONVERGED.
- Adding NEW runes — STOP. User direction: significant justification required; these L2s don't meet that bar.
- Touching the 4 existing runes (2 temperare + 2 purgare) — STOP. They stay intact.
- Skipping the interop handshakes — STOP. If permissions block, defer to orchestrator per 218.6c precedent; do NOT skip silently.
- Adding new public surface — STOP. Retire-then-mint-on-demand discipline.
- Touching `to_json_string` / `from_json_string` / `write` / `write_pretty` — STOP. Live consumer APIs stay exactly as they are.
- Bypassing tests/clippy/handshakes — never.
- Scope beyond the 13 substantive items — STOP at the boundary.
