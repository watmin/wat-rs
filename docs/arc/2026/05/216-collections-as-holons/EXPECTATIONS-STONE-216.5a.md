# EXPECTATIONS — Arc 216 Stone 216.5a — `impl Hash for Value` + `impl PartialEq + Eq for Value`

Mode A target: 14/14 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Value enum audit | Sonnet documented variant list + classification (atomizable vs non-atomizable) in SCORE; based on `is_atomizable` predicate at `src/check.rs:3623` |
| 2 | `impl PartialEq for Value` | Manual impl mirroring HolonAST pattern at `holon-rs/src/kernel/holon_ast.rs:158-192`; structural per-variant; f64 via `to_bits()`; non-atomizable strategy documented (unreachable! or Arc::ptr_eq) |
| 3 | `impl Eq for Value` | Marker trait impl; safe per NaN-bit-pattern equality |
| 4 | `impl Hash for Value` | Mirrors HolonAST pattern at `holon-rs/src/kernel/holon_ast.rs:196-232`; `std::mem::discriminant` tagging first; per-variant payload hashing; f64 via `to_bits()`; non-atomizable → `unreachable!()` with predicate-citation message |
| 5 | HashSet/HashMap arm strategy | Hash uses sorted-payload-hashes (NOT the canonical-key crutch String); bypasses `hashmap_key` entirely for these variants; deterministic via sort |
| 6 | WatAST decision | D1 (impl Hash for WatAST directly) OR D2 (Debug-string DefaultHasher); sonnet picks; documents |
| 7 | Probe 1 — Self-equality | `hash(&v) == hash(&v)` for each atomizable variant (i64, f64, String, bool, keyword, HolonAST, Uuid, HashSet, HashMap, Vec, WatAST) |
| 8 | Probe 2 — Discriminant tagging | `hash(&Value::bool(true)) != hash(&Value::i64(1))`; variants with structurally-identical-looking payloads still distinct via discriminant |
| 9 | Probe 3 — NaN-safety | `Value::f64(NAN) == Value::f64(NAN)` (bit-pattern); hash matches |
| 10 | Probe 6 — Vec composition | Two Vec Values with reversed element order produce DIFFERENT hashes (order preserved) |
| 11 | Probe 7 — HashSet composition | Two HashSet Values with same elements (different insertion order) produce IDENTICAL hashes (set semantics; sort-then-hash) |
| 12 | Probe 8 — HashMap composition | Two HashMap Values with same pairs (different insertion order) produce IDENTICAL hashes (map semantics; sort-then-hash) |
| 13 | Probe 10 — Non-atomizable panic | `std::panic::catch_unwind` confirms hashing a non-atomizable Value variant panics with `unreachable!()` message citing the predicate; OR documented skip if Fn construction not accessible at test layer |
| 14 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5a.md` — scorecard + variant classification + WatAST decision + non-atomizable PartialEq strategy + verification summary + elapsed time |

(Probes 4/5/9 from BRIEF compress into the rows above; row count is the contract.)

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 105 min
**Confidence:** medium-high

**Rationale:**
- HolonAST pattern is the direct template; sonnet just transposes it onto Value's variant set
- No callers touched — foundation-only stone; lowest risk in the antidote sequence
- Risk: Value enum may have many variants (15+?) and the manual impl gets long; mechanical but tedious
- Risk: WatAST D1 path requires adding Hash to WatAST enum — sonnet's call whether that fits in this stone or defers to a follow-up; D2 is safer fallback
- Risk: non-atomizable variant detection — sonnet must correctly identify which variants should `unreachable!()` vs which actually have meaningful equality (Arc-backed handles might want `Arc::ptr_eq`)
- All 216.x probe matrices stay green — the new impls coexist with `hashmap_key` (don't refactor callers)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites 216.5 SCORE (`8a6c12f` — the audit + probe matrix); sonnet reads the lineage.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- HashSet storage refactor — Stone 216.5b
- HashMap storage refactor — Stone 216.5c
- `hashmap_key` deletion — Stone 216.5d
- Any caller refactor — all 18 sites untouched
- WAT-CHEATSHEET update (still documents `hashmap_key` as canonical)
- Sandbox-walker validation — Stone 216.6
- INSCRIPTION — Stone 216.7

## Honesty deltas accepted

- Variant count + classification details — sonnet's audit; documents
- WatAST D1 vs D2 decision — sonnet picks; documents
- Non-atomizable PartialEq strategy (unreachable! vs Arc::ptr_eq vs default false) — sonnet picks per-variant; documents
- Probe 10 skip if Fn construction not accessible at test layer — documented in SCORE with substrate citation

## Honesty deltas NOT accepted (the post-216.4 sharpening)

- **Probe substitution — STOP-3 trigger, not a delta.**
- **Caller refactor — STOP-1 trigger, not a delta.**
- **Storage refactor — STOP-2 trigger, not a delta.**
- **Silent unreachable!() on ambiguous variants — STOP-4 trigger, not a delta.**
