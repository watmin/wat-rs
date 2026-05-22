# SCORE — Arc 221 Stone 221.5 — Symbol/String canonical-bytes seed distinction in holon-rs

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 7/7 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `PRIM_TAG_SYMBOL` constant | PASS | `const PRIM_TAG_SYMBOL: &str = "symbol";` added at the PRIM_TAG block ahead of PRIM_TAG_STRING; snake-case mirrors PRIM_TAG_CHAR/KEYWORD/NIL/TAG precedent |
| 2 | `canonical_edn_holon` Symbol arm flipped | PASS | `HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_SYMBOL, s.as_bytes())` — String arm unchanged at PRIM_TAG_STRING; two lines that were previously identical now differ |
| 3 | `encode` Symbol arm flipped | PASS | `let seed = leaf_seed(PRIM_TAG_SYMBOL, s.as_bytes(), vm.global_seed())` — String arm unchanged; mirrors the canonical_edn_holon flip exactly |
| 4 | Symbol doc comment rewritten | PASS | Lines 53-63 replaced — "Stone 221.5 resolves it" deferral removed; affirmative statement: PRIM_TAG_SYMBOL = "symbol" seeds distinct identity from PRIM_TAG_STRING = "String"; pre-arc-216 Symbol/String canonical-bytes collision declared resolved; pre-arc-221 keyword/nil conventions cited as retired |
| 5 | 2 new tests | PASS | `symbol_string_canonical_bytes_distinct` — `assert_ne!(canonical_edn_holon(Symbol("x")), canonical_edn_holon(String("x")))` PASS; `symbol_string_vectors_distinct` — `assert_ne!(encode(Symbol("x"), ...), encode(String("x"), ...))` PASS; both use `fresh_env()` helper per BRIEF recommendation |
| 6 | All test suites + clippy green | PASS | `cargo build --release` — 0 warnings, OK; `cargo test --release` — 270 unit + 19 doc = 289/289 PASS (287 baseline + 2 new); `cargo clippy --release -- -D warnings` — 0 warnings |
| 7 | Wat-rs untouched | PASS | `git -C /home/watmin/work/holon/wat-rs/ diff --name-only` — empty (no existing wat-rs files modified; SCORE doc is a new file in docs/) |

## Deltas from EXPECTATIONS

### Delta 1 — No cascade surprises

Pre-flight grep confirmed zero active Symbol/String equality assertions in holon-rs. The change is a constant rename in exactly 2 arms (canonical_edn_holon + encode) — Rust's exhaustive-match compiler requires no additional arms. STOP-1 did not trigger. The +2 test delta matches the prediction exactly (287 → 289).

### Delta 2 — char_distinct_from_symbol pre-existing test now stronger

The Stone 221.1 test `char_distinct_from_symbol` asserted `Char('a')` differs from `Symbol("a")` in canonical bytes. Before Stone 221.5, both Symbol and String used PRIM_TAG_STRING, so the test was implicitly relying on the PRIM_TAG_CHAR vs PRIM_TAG_STRING distinction. After this stone, PRIM_TAG_CHAR vs PRIM_TAG_SYMBOL distinction also holds — the test continues to PASS and is now backed by two distinct constants rather than one. No action required; noting for calibration record.

## Verification summary

```
holon-rs/ (working dir):
  cargo build --release                         — OK (0 warnings)
  cargo test --release                          — 289/289 PASS (270 unit + 19 doc)
  cargo clippy --release -- -D warnings         — 0 warnings

wat-rs/ contamination check:
  git -C wat-rs/ diff --name-only               — empty (no wat-rs files touched)
```

New tests confirmed passing:
```
test kernel::holon_ast::tests::symbol_string_canonical_bytes_distinct ... ok
test kernel::holon_ast::tests::symbol_string_vectors_distinct          ... ok
```

## Files changed (1 file)

Holon-rs:
- `holon-rs/src/kernel/holon_ast.rs` (~+20 lines): PRIM_TAG_SYMBOL constant + canonical_edn_holon Symbol arm flip + encode Symbol arm flip + Symbol doc comment rewrite + 2 new tests

SCORE doc (wat-rs docs dir, code changes in holon-rs):
- `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.5.md` (this file)

**Total: 1 modified source file + 1 new SCORE doc.**

## STOP triggers

- **STOP-1 (existing holon-rs test regression):** DID NOT TRIGGER. 289 tests PASS; +2 new exactly matches prediction.
- **STOP-2 (distinctness tests fail):** DID NOT TRIGGER. `symbol_string_canonical_bytes_distinct` and `symbol_string_vectors_distinct` both PASS — PRIM_TAG_SYMBOL creates distinct canonical bytes byte-for-byte from PRIM_TAG_STRING.
- **STOP-3 (60 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (wat-rs touched accidentally):** DID NOT TRIGGER. `git -C wat-rs/ diff` is empty.

## Calibration check

- **Target runtime:** 30-45 min
- **Actual sonnet duration:** ~15 min (reading 4 docs + targeted file reads + 5 edits + build/test/clippy verification + SCORE)
- **Within prediction band?** UNDER lower bound — consistent with Stone 221.1 (~25 min) and 221.3 (~35 min) pattern. Stone 221.5 is the simplest of the three: no new variant, no cascade sites, no external consumer flips. Pattern fully internalized; only 2 arms to flip + 1 doc rewrite + 2 tests.

## Substrate state

- `PRIM_TAG_SYMBOL = "symbol"` minted — snake-case, distinct from `PRIM_TAG_STRING = "String"` at byte level
- `Symbol(s)` canonical bytes now seed from `"symbol"` tag; `String(s)` from `"String"` tag
- `Symbol("x")` and `String("x")` produce distinct VSA vectors — the pre-arc-216 collision is closed
- Arc 221 Phase B substrate work complete: Symbol, String, I64, F64, Bool, Char, Keyword, Nil, Tag each have distinct PRIM_TAG constants and distinct canonical-bytes seeds
- `HolonAST::Sixteen variants. All distinct at both type level and canonical-bytes level.`

## Unblocks

- Stone 221.6 INSCRIPTION (spawn-block blocked on arc 222 + arc 223 — not on holon-rs substrate; this stone removes the last holon-rs deferral)
- Arc 222 + arc 223 can proceed knowing the full Symbol/String/Keyword/Nil/Tag leaf algebra is substrate-honest
