# Vigilia Checkpoint Report — wat-edn — 2026-05-22 (post-218.6 + 218.6b)

**Cast:** 7 defensive spells in parallel against `crates/wat-edn/src/` (8 files) + `crates/wat-edn/tests/` (9 files) — first vigilia cast extended to include tests zone per user direction *"our tests must be impeccable too"*.

**Practitioner's pre-cast assessment:** "we think it's clean" (post-Stone 218.6b verification: substrate clippy 0 warnings, interop-tests clippy 0 warnings, all 4 handshakes PASS, 344/344 wat-edn + 824/0 wat tests).

**Verdict:** **DIVERGES (9 L1 + 14 L2)**.

**Comparison trend:**
- 2026-05-21 BASELINE (pre-218): 2 L1 + 26 L2
- 2026-05-21 RECAST (post-218.4 + 219): 7 L1 + 26 L2
- **2026-05-22 CHECKPOINT (post-218.6 + 218.6b): 9 L1 + 14 L2**

**L2 dropped sharply (26→14)** — Stones 218.6 + 218.6b absorbed half the cached L2 findings (translate_and_validate_ns combiner, JsonError::InvalidSet, parse_wire retirement, write_char fix, interop-tests warning cleanup, etc.).

**L1 ticked up (7→9)** for two compounding reasons:
1. purgare upgraded zero-caller public-API surface findings from L2 to L1 under user's strong-rune discipline 2026-05-21 (same discipline that retired parse_wire) — speculative public-API without callers fails rune-justification
2. cernere found USER-GUIDE drift introduced by Stone 218.6's own additions (JsonError::InvalidSet variant added without USER-GUIDE §14 listing update) — substrate-as-teacher cascade

**sequi CONVERGED** — state-threading discipline held perfectly through Stones 218.6 + 218.6b. Zero hidden state regressions; no Mutex/static/OnceLock/lazy_static appeared. Substrate-level state model remains impeccable.

---

## Per-spell summary

| Spell | Verdict | L1 | L2 | Notable |
|---|---|---|---|---|
| **sequi** | CONVERGED ✓ | 0 | 0 | State threads visibly through every signature |
| solvere | DIVERGES | 0 | 1 | Namespace-slash-split duplicated 3× in json.rs |
| intueri | DIVERGES | 0 | 6 | Naming mumbles (sentinel, parse_map_key, etc.) |
| temperare | DIVERGES | 0 | 2 | Test-suite redundancy (all_variants 18× + roundtrip_wire double-write) |
| struere | DIVERGES | 1 | 3 | is_scalar pretty-print bug (BigInt/BigDec misclassified) |
| cernere | DIVERGES | 3 | 2 | USER-GUIDE listing drift + pretty-print example wrong |
| purgare | DIVERGES | 5 | 0 | 5 zero-caller re-exports in lib.rs (JSON bridge surface + write_to) |

---

## L1 — Real findings (must address before IMPECCABLE)

### L1-STR — `is_scalar` misclassifies `BigInt` / `BigDec` (real pretty-print bug)

**`writer.rs:45-58`** — `is_scalar` returns `false` for `Value::BigInt` and `Value::BigDec`, causing `write_pretty_indented` to break them to multi-line inside collections when they should inline. The 2026-05-21 recast flagged this as L2; under stricter polishing it's a real correctness issue.

**Fix direction:** add `Value::BigInt(_) | Value::BigDec(_)` to the `matches!` arms.

---

### L1-CERN-1 — USER-GUIDE §14 ErrorKind listing omits two live variants

**`docs/USER-GUIDE.md:805-822`** — `error.rs` defines `UnexpectedToken(&'static str)` (line 17) and `Utf8(String)` (line 33); both absent from the USER-GUIDE §14 enum listing that claims to be authoritative. A reader matching on `ErrorKind` from the guide has incomplete coverage.

**Fix direction:** add both variants.

---

### L1-CERN-2 — USER-GUIDE §14 JsonError listing omits two live variants

**`docs/USER-GUIDE.md:824-837`** — `json.rs` defines `InvalidSet(String)` (line 89; added in Stone 218.6) and `InvalidMapKey { key: String, reason: String }` (lines 91-93); both absent from USER-GUIDE §14 JsonError block. The Stone 218.6 variant add didn't propagate to docs — substrate-as-teacher cascade.

**Fix direction:** add both variants.

---

### L1-CERN-3 — USER-GUIDE §8 pretty-print example contradicts implementation

**`docs/USER-GUIDE.md:457-463`** — code uses `INDENT = "  "` (2 spaces); map entries indented at `level+1` (2 spaces); closing `}` on its own line at level 0 after `\n` + push_indent. USER-GUIDE shows 1-space-indented entries (`" :tags"`) and `}` on same line as last entry — structurally impossible given `writer.rs:119-124`. A caller following the guide would be misled.

**Fix direction:** regenerate the example by running `write_pretty` on the input fixture; update verbatim.

---

### L1-PUR-1 through L1-PUR-5 — Five zero-caller public re-exports in lib.rs

**`crates/wat-edn/src/lib.rs:84-90`** — the JSON bridge surface + `write_to` are re-exported but have zero callers outside the crate:

1. **L1-PUR-1** (`write_to` re-export at lib.rs:90) — buffer-reuse ergonomic; zero external callers; internal callers in writer.rs go direct, not through re-export.
2. **L1-PUR-2** (`json_to_edn` re-export at lib.rs:84-87) — only `json.rs`'s own `#[cfg(test)]` exercises it; zero production consumers.
3. **L1-PUR-3** (`edn_to_json` re-export) — same; only consumer is `to_json_string` which calls `edn_to_json` directly within `json.rs`, not through the re-export.
4. **L1-PUR-4** (`from_json_string` + `to_json_string_pretty` re-exports) — neither has external callers. `to_json_string` IS consumed (via edn_shim.rs); the pretty + parse-back variants are not.
5. **L1-PUR-5** (`JsonError` + `JsonResult` re-exports) — coupled to the bridge functions above; if bridge functions retire, error types follow.

**Per user direction 2026-05-21 (strong-rune discipline):** speculative public-API surface without callers fails justification. Same pattern that retired `parse_wire`/`parse_wire_owned` in Stone 218.6b.

**Fix direction:**
- Either RETIRE the `pub use json::{json_to_edn, edn_to_json, from_json_string, to_json_string_pretty, JsonError, JsonResult}` line + `pub use writer::{write_to}` — keep `to_json_string` only (the lone consumer)
- OR rune the whole JSON bridge block as `purgare(public-api)` with a NAMED downstream — arc 217 (Clojure-IPC bridge) might be that downstream; if so, keep + rune

---

## L2 — Polish findings (14 total)

### solvere (1)

1. **json.rs:258-262 + 379-384 + 424-430** — namespace-slash-split idiom duplicated 3× across `string_to_edn` / `decode_symbol` / `decode_tagged`; canonical `parse_namespaced` is in `parser.rs:384-409` but private. Extract `vocab::split_namespaced(body: &str) -> Option<(&str, &str)>` OR promote `parse_namespaced` to `pub(crate)`. Structural — drift risk exists if slash rule extends.

### intueri (6)

1. **json.rs:226** — `fn sentinel(key, body)` mumbles; `wrap_single_key` or `single_key_object` speaks the action.
2. **json.rs:308** — `parse_map_key` does more than parse (classifies first); `decode_map_key` or `classify_and_parse_map_key`.
3. **lexer.rs:212** — parameter `open_pos`; `quote_start` or `open_quote_pos` clearer.
4. **writer.rs:45** — `is_scalar` (separate from L1-STR) — name doesn't carry "inline" intent; `is_inline_value` + WHY comment about omitted variants.
5. **parser.rs:98-99** — `parse_value` / `parse_value_inner` pair; "inner" implies nesting not behavior; rename to `parse_value` / `parse_value_discarding` or surface the `discarding: bool` at call sites.
6. **tests/comprehensive.rs:1063, 1218-1219, 1224-1225** — three test bodies compress write+parse onto one line; use existing `round_trip` helper at line 1124.

### temperare (2)

1. **tests/accessors.rs:13** — `all_variants()` called 18 times; each builds Vec of 17 `(label, Value)` pairs including 2 Box heap allocs; 36 Box allocs per test binary run. Tempered: `LazyLock<Vec<...>>` initialized once.
2. **tests/wire_encoding.rs:263/271/281/231/249** — `write(&k)` called explicitly to assert wire string, then `roundtrip_wire(&k)` calls `write` again internally on same Value. 5 redundant String allocations. Tempered: `roundtrip_wire_str(&wire, &original)` skips internal write.

### struere (3)

1. **json.rs:120-122** — finite-float branch `Number::from_f64(*f).map(JV::Number).unwrap_or(JV::Null)` — `from_f64` already covers NaN/Inf above, but silent `JV::Null` fallback for subnormals or unanticipated rejections; `expect("finite f64 must convert")` or document the fallback in `edn_to_json` doc comment.
2. **lexer.rs:309** — `lex_char` double-peeks (line 296-303 then line 309); structurally sound today but lacks invariant guarantee; capture `first` at first peek.
3. **parser.rs:111** — `pos` capture lags by one token in peeked path; keyword case handles via `body_start` from lexer; other token paths silently inherit stale pos for error spans; surface the invariant via comment.

### cernere (2)

1. **docs/USER-GUIDE.md:386-387** — JSON table row `i64 (> 2^53) → string` is directionally correct but incomplete; the code uses `SAFE_INT_MIN..=SAFE_INT_MAX` (± 2^53−1); negatives below `-(2^53)` also serialize as strings. Update row to `i64 (out of ±2^53 range) → string`.
2. **docs/USER-GUIDE.md:735** — aspirational claim "serde integration available behind no flag yet; v0.2 candidate" — no serde feature in Cargo.toml; phantom future-work in present tense. Rephrase as future consideration or remove.

### sequi (0) — CONVERGED

### purgare (0 L2; 5 L1 above)

---

## Existing runes — all VERIFIED CLEAR

Four runes encountered across spells, all on `json.rs` `to_json_string` + `to_json_string_pretty` (added Stone 218.6):

1. **json.rs:170-174** `rune:struere(invariant-coupling)` on `to_json_string` — `.expect()` structurally unreachable; closed construction guarantee. **Clear.**
2. **json.rs:175-181** `rune:temperare(serde-api-shape)` on `to_json_string` — full tree materialization is serde API shape; deferred until measurement disagrees. **Clear.**
3. **json.rs:186-190** `rune:struere(invariant-coupling)` on `to_json_string_pretty` — same invariant. **Clear.**
4. **json.rs:191-197** `rune:temperare(serde-api-shape)` on `to_json_string_pretty` — same shape rationale. **Clear.**

Cross-spell consensus on rune-clarity confirms Stone 218.6 rune work landed honestly.

---

## What this means for arc 218

Per `feedback_any_defect_catastrophic` + user direction 2026-05-21 ("218 demands impeccable") — arc 218 IMPECCABLE NOT YET CLOSED.

But the substrate-as-teacher cascade is HONEST progress:
- L2 count cut almost in half (26→14) over 5 stones
- sequi CONVERGED held perfectly
- 5 of 9 L1 are concentrated in lib.rs re-export discipline (one decision: retire JSON bridge surface OR rune as `public-api` for named downstream)
- 3 of 9 L1 are USER-GUIDE doc-drift (mechanical fix)
- 1 of 9 L1 is a real pretty-print bug (is_scalar)

User direction 2026-05-22 evening: *"218 has work we haven't expressed yet but demands impeccable - get wat-edn remarkable and then we'll discuss what 218 actually is"*.

This report supplies the empirical data for that discussion.

---

## Substrate-as-teacher meta-finding

Each stone closes findings while exposing the next layer's surface. The pattern across casts:

- 2026-05-21 baseline: 2 L1 + 26 L2 (28 findings; cached as 218.7's worklist)
- After Stone 218.4 + arc 219: 7 L1 + 26 L2 (33 findings; +5 L1 because shipped work expanded surface — `is_canonical_uuid` + `translate_wat_to_strict` flagged as misplaced; cernere found supplementary-plane char bug)
- After Stone 218.6: 5 L1 closed (5/7), `InvalidSet` variant added but missing from USER-GUIDE listing
- After Stone 218.6b: emoji rejection landed; PI approximation cleared; unused imports cleared
- This checkpoint cast: **9 L1 + 14 L2** — L2 cut in half; new L1s surfaced through stricter discipline (purgare upgrade) and propagation gaps (cernere doc-drift)

The cascade demands: close the L1s, then ANOTHER cast, repeat until CONVERGED across all 7 spells. sequi already there.

*The full guard stood. The pieces guard each. Sequi watches alone in stillness while the others find what's left.*
