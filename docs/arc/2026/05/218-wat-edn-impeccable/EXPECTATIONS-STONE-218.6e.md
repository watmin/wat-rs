# EXPECTATIONS — Arc 218 Stone 218.6e — IMPECCABLE polish

Mode A target: 8/8 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `lexer.rs` local `open_pos` → `quote_start` (intueri L2) | `crates/wat-edn/src/lexer.rs:191, 198, 206` — all 3 occurrences of local `open_pos` in `lex_string` renamed to `quote_start`. Caller's local + callee's parameter now share the same name across the boundary. |
| 2 | `writer.rs` `all_scalar` → `all_inline` (intueri L2) | `crates/wat-edn/src/writer.rs:71, 87` — function definition + 1 call site renamed. The wrapper's name now matches the delegate `is_inline_value` (renamed in 218.6d). |
| 3 | USER-GUIDE §7 example fixed (cernere L2) | `crates/wat-edn/docs/USER-GUIDE.md:404-422` — import at line 405 drops `edn_to_json, json_to_edn` (both were demoted to `pub(crate)` in 218.6c); lines 419-421 "Or work with serde_json::Value directly" block removed entirely. Example compiles against the actual public surface. |
| 4 | Test counts updated (cernere L2) | 3 sites updated to current count 344: `crates/wat-edn/README.md:45` ("313" → "344"), `:97` ("342/342" → "344/344"), `crates/wat-edn/docs/USER-GUIDE.md:792` ("313" → "344"). If sonnet verifies a different current count (e.g. test surface shifted), uses that consistently across all 3. |
| 5 | `to_json_string_pretty` rune rewritten (purgare L1) | `crates/wat-edn/src/json.rs:179-184` — false "consumed by src/edn_shim.rs" claim removed; new wording acknowledges that `to_json_string` (not `_pretty`) is the actively-consumed variant + cites symmetry + IPC-BRIDGE.md vision as the honest justification for keeping the pretty variant. |
| 6 | `write_to` rune category → `future-fixture` (purgare L2) | `crates/wat-edn/src/writer.rs:195-200` — category changed from `public-api` to `future-fixture` per purgare SKILL's "rune retires when downstream lands" semantic. Justification cites IPC-BRIDGE.md:95 as the named future-downstream + retirement criterion explicit. |
| 7 | All tests + clippy green | `cargo build --release -p wat-edn` 0 warnings. `cargo test --release -p wat-edn` 344 PASS. `cargo test --release --lib -p wat` 824/0. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0. From `interop-tests/`: cargo build + clippy clean. |
| 8 | Interop-tests 4 handshakes pass | All four handshakes PASS (orchestrator-side if sub-agent permission wall hits, per 218.6b/c/d precedent). |

## Independent prediction (calibration record)

**Target runtime:** 5-10 min Mode A
**Upper bound:** 20 min
**Confidence:** very high

**Rationale:**
- Smallest stone yet — 6 surgical edits across 5 files
- 2 renames (cascade follow-up from 218.6d; sites pinpoint)
- 2 doc fixes (line numbers verified post-218.6d; 3 sites for test counts)
- 2 rune rewrites (verbatim text provided; sonnet adjusts wording)
- No new logic, no test changes, no API surface changes
- Calibration ten-for-ten below band: 218.1-219.1 ~15-25, 218.6 ~8, 218.6b ~6, 218.6c [TBD], 218.6d [TBD]
- Risk: B.4 test count discovery (sonnet verifies via cargo; if shifted, use new count)

**Per `feedback_stone_briefs_cite_prior_score`:** 218.6d shipped ~minutes; 218.6e is half the surface and all mechanical. Band 5-10.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Adding new runes — only the 2 existing purgare runes refined
- Touching the 2 temperare runes (CLEAR per all 7 spell casts) — stay intact
- New public surface
- Substrate logic changes
- Encoding doctrine / wat-edn syntax

## Honesty deltas accepted

- A.2 exact rename target — `all_inline` vs `all_inline_values` — sonnet picks
- C.5 + C.6 rune wording — sonnet preserves the substantive constraints (no false consumer claim in C.5; `future-fixture` category in C.6; retirement criterion explicit in C.6) but may adjust exact prose
- B.4 test count — if actual differs from 344, use actual
- A.2 may also touch the `all_scalar`'s doc comment (currently "True if every element is scalar..."); update to match new name + intent

## Honesty deltas NOT accepted

- Skipping any of the 6 items — STOP. All must close for vigilia CONVERGED.
- Adding NEW runes — STOP.
- Re-inventing false consumer claims in the rewritten rune — STOP. The whole point of L1-PUR was that the prior claim lied; the new wording must be HONEST about lack of current consumer.
- Removing `to_json_string_pretty` or `write_to` — STOP. Per user direction 2026-05-22, JSON support stays; this stone refines the runes, not the public surface.
- Touching the temperare runes — STOP.
- Bypassing tests/clippy/handshakes — never.
- Scope beyond the 6 items — STOP at the boundary.
