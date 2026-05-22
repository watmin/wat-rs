# EXPECTATIONS — Arc 221 Stone 221.5 — Symbol/String canonical-bytes seed distinction

Mode A target: 7/7 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `PRIM_TAG_SYMBOL` constant | `const PRIM_TAG_SYMBOL: &str = "symbol";` added at PRIM_TAG block (~line 522) alongside PRIM_TAG_STRING/I64/F64/BOOL/CHAR/KEYWORD/NIL/TAG; snake-case mirrors precedent |
| 2 | `canonical_edn_holon` Symbol arm flipped | `HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_SYMBOL, s.as_bytes())` at line ~549; String arm unchanged |
| 3 | `encode` Symbol arm flipped | `HolonAST::Symbol(s) => { let seed = leaf_seed(PRIM_TAG_SYMBOL, ...); }` at line ~626; pattern mirrors existing leaf encoding |
| 4 | Symbol doc comment rewritten (lines 53-71) | Removes "Stone 221.5 resolves it" deferral; states resolved distinctness via PRIM_TAG_SYMBOL; arc 221 doctrine fully closed for Symbol/String pair |
| 5 | 2 new tests | `symbol_string_canonical_bytes_distinct` + `symbol_string_vectors_distinct` — both PASS via assert_ne!; mirror Stone 221.1's `char_distinct_from_string` shape |
| 6 | All test suites + clippy green | `cargo build --release` 0 warnings; `cargo test --release` 289/289 PASS (287 baseline + 2 new); `cargo clippy --release -- -D warnings` 0 warnings |
| 7 | Wat-rs untouched | `git -C /home/watmin/work/holon/wat-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 30-45 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- Stone 221.1 (1 new variant + 8 arms + 1 const + 1 ctor + 3 tests, holon-rs cold) = ~25 min
- Stone 221.5 is SIMPLER: 1 new const + 2 arm flips + 1 doc rewrite + 2 tests
- Pattern internalized from 221.1/221.3
- Risk: cascade test regression (DESIGN open question #4) — pre-flight grep shows zero active Symbol/String equality assertions; risk low but verify via cargo test

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- wat-rs changes
- Stone 221.6 INSCRIPTION (spawn-block blocked on arc 222 + 223)
- Arc 222 + arc 223 work
- New HolonAST variants
- Migration of pre-existing Symbol+String same-content sites (none expected)
- BOOK / USER-GUIDE

## Honesty deltas accepted

- Symbol doc rewrite phrasing — sonnet picks; load-bearing point is "no longer cites Stone 221.5 deferral"
- Test fixture helper choice — `fresh_env()` recommended (used by existing `keyword_vs_string_distinct_by_content`); alternative honest as long as the test runs cleanly
- Adjacent doc comment refreshes (other places mentioning the Symbol/String collision as a pre-existing compromise) — encouraged but not required this stone

## Honesty deltas NOT accepted

- Skipping either distinctness test — STOP. Load-bearing assertions for Stone 221.5 substrate doctrine.
- Touching wat-rs files — STOP. Wat-rs stays clean.
- Modifying other PRIM_TAG constants — STOP. Only adding PRIM_TAG_SYMBOL.
- Adding new HolonAST variants — settled.
- "Pre-existing failure" framing for any test broken by this stone — STOP per Stone 221.3 Delta 1a discipline.

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** holon-rs test regression beyond +2 new
- **STOP-2:** distinctness tests fail (PRIM_TAG_SYMBOL not differentiating)
- **STOP-3:** 60 min elapsed
- **STOP-4:** wat-rs touched accidentally
