# EXPECTATIONS — Arc 219 Stone 219.1 — Substrate strict-EDN + constructor translation + test sweep

Mode A target: 11/11 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | vocab.rs strict-EDN char set | `crates/wat-edn/src/vocab.rs:101-122` `is_symbol_continue` removes `b':'` and `b'#'` from the `matches!` pattern. Keeps `b'.'` `b'*'` `b'+'` `b'!'` `b'-'` `b'_'` `b'?'` `b'$'` `b'%'` `b'&'` `b'='` `b'<'` `b'>'` `b'/'` (per EDN spec). |
| 2 | Translation helper added | `crates/wat-edn/src/value.rs` — private fn `translate_wat_to_strict(ns: &str) -> String` (or equivalent) replacing `::` with `.`. Idempotent. |
| 3 | `Symbol::ns` constructor translation | `value.rs:233` — translates namespace via helper BEFORE further construction. `Symbol::ns("wat::core", "X")` stores namespace as `"wat.core"`. |
| 4 | `Symbol::try_ns` constructor translation | `value.rs:257` — translation happens BEFORE `validate_first_char` (so validation runs against strict form). |
| 5 | `Keyword::ns` + `Keyword::try_ns` constructor translation | `value.rs:306` + `:328` — same pattern as Symbol; translation before validation. |
| 6 | `Tag::ns` + `Tag::try_ns` constructor translation | `value.rs:368` + `:385` — same pattern. |
| 7 | `from_parts_unchecked` UNCHANGED | Unchecked paths preserve their UNCHECKED semantic; caller responsibility. NO translation added. Verified by grep + read. |
| 8 | Wat-edn-internal test fixture sweep | All wat-edn-internal test files using `:wat::core::Foo` style literals → flipped to `:wat.core.Foo` (or kept where they're TESTING the rejection of `::`). Known sites: 5 in wire_encoding.rs. Sonnet grep finds the full count; sweeps all. |
| 9 | Three new probes added | Probe 1 `is_symbol_continue_rejects_colon` (vocab unit test). Probe 2 `parser_rejects_double_colon_in_keyword` (parse-level rejection). Probe 3 `keyword_ns_translates_wat_to_strict` (constructor-translation visible via `namespace()` accessor). All in `spec_strict.rs`. |
| 10 | wat-edn test suite: 342/342 PASS | `cargo build --release -p wat-edn` clean. `cargo test --release -p wat-edn` 342 PASS (339 baseline + 3 new probes; fixture sweep updates counted as updates, NOT regressions). `cargo clippy --release -p wat-edn -- -D warnings` 0 warnings. |
| 11 | wat-rs workspace sanity check | `cargo build --release` workspace builds clean. `cargo test --release --lib` library tests pass. STOP-4 trigger if wat-rs callers regress beyond crates/wat-edn/. |

(Plus SCORE doc inscribed at `docs/arc/2026/05/219-wat-edn-strict-edn-keywords/SCORE-STONE-219.1.md`.)

## Independent prediction (calibration record)

**Target runtime:** 45-75 min Mode A
**Upper bound:** 95 min
**Confidence:** medium-high

**Rationale:**
- Three coordinated surfaces: vocab.rs char-set tighten (1 site, 2 byte drops), value.rs constructor translation (6 sites + 1 helper fn), test fixture sweep (5+ sites)
- Substrate-pre-grep dense — all constructor sites + 5 fixture sites confirmed at exact line numbers
- Risk: `from_parts_unchecked` accidentally gets translation added (STOP-1 — easy mistake; orchestrator pre-flagged)
- Risk: wat-rs callers outside crates/wat-edn/ depend on `::` survival post-roundtrip (STOP-4) — orchestrator's analysis says no, but workspace test is the proof
- Risk: a fixture sweep flip MISSES a site (STOP-2 — wider grep catches)
- Calibration trend: four prior stones (218.1-218.4) all at-or-below lower band. 219.1 has broader surface than 218.4 (dual-surface: substrate AND fixture sweep), so band widens but stays consistent with the pattern.

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.4 SCORE (~20 min actual; 9-row substrate + docs work). 219.1 has 11 rows (vocab + 6 constructor sites + helper + fixtures + 3 probes + wat-edn verification + wat-rs sanity). Bigger surface; widened band.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Touching wat-rs `src/` storage — wat-rs keeps `::` internally
- Touching `.wat` source files — wat source syntax keeps `::`
- Wat-rs Rust string literals like `"wat::core::Foo"` (outside crates/wat-edn/) — constructor at the boundary handles translation
- Stones 219.2 / 219.3 / 219.4 — separate stones
- DESIGN-219 amendments — orchestrator-direct
- New error variants beyond what the existing infrastructure provides — reuse existing ErrorKind variants

## Honesty deltas accepted

- Translation helper spelling — `String::replace("::", ".")` vs char-by-char walker; sonnet picks based on simplicity (replace is fine; the strings are short)
- Test fixture sweep count — sonnet greps wider than orchestrator's 5; reports total + per-site classification
- Probe placement: `spec_strict.rs` for all three is the default; sonnet picks alternative placement if a probe genuinely belongs elsewhere (Probe 1 might fit better as a vocab-internal unit test)
- Whether to add `translate_wat_to_strict` to `vocab.rs` instead of `value.rs` — `value.rs` is the constructor home; `vocab.rs` is char-vocab; either is defensible. Sonnet picks; documents.

## Honesty deltas NOT accepted

- Skipping the translation on any of the 6 named constructor sites — STOP. The boundary discipline is the load-bearing invariant.
- Adding translation to `from_parts_unchecked` — STOP-1 trigger. Unchecked paths stay unchecked.
- Skipping `b'<' | b'>'` preservation — STOP-3. EDN-spec chars stay.
- Touching wat-rs `src/` files — out of scope; STOP if tempted.
- Bypassing tests/clippy — never; 342 must hold (339 + 3 additive).
