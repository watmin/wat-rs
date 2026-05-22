# EXPECTATIONS — Arc 221 Stone 221.2 — wat-rs `value_to_atom` Char arm + `is_atomizable` Char extension

Mode A target: 5/5 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `value_to_atom` Char arm | `src/runtime.rs:~13823` — `Value::wat__core__Char(c) => HolonAST::char_(c)` placed alongside other primitive leaves (i64 / f64 / bool / String / keyword); doc comment cites Stone 221.1 holon-rs commit `243eded` |
| 2 | `is_atomizable` Char extension | `src/check.rs:~3640` — one line added: `\| ":wat::core::Char"` in the matches!-arm; doc comment cites Stone 221.2's value_to_atom dispatch (this stone) + Stone 220.2's runtime Hash arm at runtime.rs:846 |
| 3 | 3 new probes | `tests/wat_arc221_char_atomization.rs` — Probe 1 `(:wat::holon::Atom \a)` round-trip (Atom holds Char; distinct from Atom of i64), Probe 2 `HashMap<Char, i64>` insert + lookup (char-frequency-tally), Probe 3 `HashSet<Char>` insert + contains? |
| 4 | Uuid arm NOT added (out of scope) | `git diff src/runtime.rs` shows ZERO changes to `Value::wat__core__Uuid` arm of value_to_atom. Uuid stays in false-flag state through Phase A; Stone 221.4 closes it with Tag-based encoding. |
| 5 | All test suites + clippy green | `cargo build --release` 0 NEW warnings (pre-existing 115 wat-clippy warnings stay per arc 170 backlog). `cargo test --release --lib -p wat` 827/0 PASS (baseline preserved). `cargo test --release --test wat_arc220_char` 10/10 PASS (Stone 220.2 unchanged). `cargo test --release --test wat_arc221_char_atomization` 3/3 PASS (new probes). `cargo test --release --test wat_arc220_list` 23/23 PASS (Stone 220.4 unchanged). `cargo test --release -p wat-edn` 1/1 PASS (unchanged). `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings. |

## Independent prediction (calibration record)

**Target runtime:** 20-30 min Mode A
**Upper bound:** 45 min
**Confidence:** very high

**Rationale:**
- Smaller scope than Stone 221.1 (no exhaustive-match cascade in wat-rs; only 2 file edits)
- Sonnet already familiar with wat-rs patterns from Stones 220.2 + 220.4
- Test file template established from `tests/wat_arc220_char.rs` (Stone 220.2 — 10 tests, 312 lines, helper fns)
- Risk: probe wat syntax exact form — sonnet picks from existing precedents (low risk)
- Risk: `HolonAST::char_` constructor not imported — quick fix if needed (low risk)
- 16th stone in below-band series; pattern locked

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 221.1 (~25 min sonnet for HolonAST::Char + 4 cascade arms + 3 tests + SCORE in holon-rs). Stone 221.2 is ~50% of that scope (2 file edits + 3 probes; no cascade because wat-rs's value_to_atom isn't an exhaustive match — it has fallthrough). Band 20-30 reflects probe-writing time as the dominant factor.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Uuid arm (Stone 221.4 — Phase B)
- Keyword/Nil/Tag leaves (Stone 221.3 — Phase B)
- Symbol/String canonical-bytes seed distinction (Stone 221.5 — Phase B)
- Migration ripple (Stone 221.4 — Phase B)
- INSCRIPTION (Stone 221.6 — Phase B)
- arc 220 Slice 5 paperwork (post-Phase-A, separate task)
- Holon-rs build verification (Stone 221.1 already shipped)
- Interop handshakes (wat-edn untouched)

## Honesty deltas accepted

- Exact probe wat syntax — sonnet picks based on existing wat_arc220_char.rs / wat_arc220_list.rs patterns
- Test file naming — `wat_arc221_char_atomization.rs` recommended; sonnet may pick variant if it surfaces a better convention
- Char arm placement in value_to_atom — sonnet picks (alphabetical vs thematic-grouping vs immediately-after-keyword); load-bearing is "primitive cluster"
- `HolonAST::char_` constructor import — if `use holon::HolonAST` already provides it via re-export, no new import; sonnet handles per existing pattern

## Honesty deltas NOT accepted

- Adding Uuid arm — STOP. Deferred to Stone 221.4 per Tag-doctrine-correction (Uuid needs Bind(Tag, payload); Tag doesn't exist until Stone 221.3).
- Skipping any of the 3 probes — STOP. All three are the contract: Atom-able, HashMap-key-able, HashSet-element-able.
- Convention-based Char encoding via String("char:c") or similar — STOP. The point is to USE the Stone 221.1 leaf, not invent scaffolding.
- Touching holon-rs files — STOP. Stone 221.1 already shipped.
- Touching wat-edn — STOP. Not in scope.
- Adding new runes — STOP. No candidates this stone.
- Scope beyond the 2 file edits + 3 probes + SCORE — STOP at the boundary.

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** lib test regression (827/0 baseline must hold)
- **STOP-2:** probe failures (diagnostic surface needed)
- **STOP-3:** 45 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** Uuid arm accidentally added
