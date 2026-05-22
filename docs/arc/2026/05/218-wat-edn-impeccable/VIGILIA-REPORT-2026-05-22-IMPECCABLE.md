# Vigilia IMPECCABLE Recast — wat-edn — 2026-05-22 (post-218.6c + 218.6d)

**Cast:** 7 defensive spells in parallel against `crates/wat-edn/src/` (8 files) + `crates/wat-edn/tests/` (9 files). The "IMPECCABLE check" — verifying that Stones 218.6c + 218.6d closed the cascade from VIGILIA-REPORT-2026-05-22-CHECKPOINT.md (9 L1 + 14 L2 → 0 + 0).

**Practitioner's pre-cast assessment:** 0 L1 + 0 L2 expected.

**Verdict:** **DIVERGES (1 L1 + 5 L2)** — close, but the cascade isn't done.

---

## Trajectory across 4 casts

| Cast | Trigger | L1 | L2 |
|---|---|---|---|
| 2026-05-21 BASELINE | pre-arc-218 | 2 | 26 |
| 2026-05-21 RECAST | post-218.4 + 219 | 7 | 26 |
| 2026-05-22 CHECKPOINT | post-218.6 + 218.6b | 9 | 14 |
| **2026-05-22 IMPECCABLE** | **post-218.6c + 218.6d** | **1** | **5** |

**4 of 7 spells CONVERGED:** sequi (consistent since 2026-05-21), solvere, struere, temperare. Three spells still find honest dust.

---

## Per-spell summary

| Spell | Verdict | L1 | L2 | Notes |
|---|---|---|---|---|
| sequi | CONVERGED ✓ | 0 | 0 | State-threading discipline impeccable; held through every stone |
| solvere | CONVERGED ✓ | 0 | 0 | Slash-split duplication closed via `vocab::split_namespaced` |
| struere | CONVERGED ✓ | 0 | 0 | All prior 1 L1 + 3 L2 confirmed closed by 218.6c + 218.6d |
| temperare | CONVERGED ✓ | 0 | 0 | LazyLock + roundtrip_wire_str both confirmed |
| intueri | DIVERGES | 0 | 2 | Cascade-rename incompleteness from 218.6d |
| cernere | DIVERGES | 0 | 2 | USER-GUIDE drift exposed by 218.6c demotions + test-count staleness |
| purgare | DIVERGES | **1** | 1 | Stone 218.6c rune wording lies about consumer + write_to category mismatch |

---

## L1 — The load-bearing finding (must close)

### L1-PUR — `to_json_string_pretty` rune cites a non-existent consumer

**`crates/wat-edn/src/json.rs:179-184`** — the `purgare(public-api)` rune added in Stone 218.6c claims:

> *"consumed by src/edn_shim.rs for WAT_TEST_OUTPUT cargo integration per arc 116"*

**This claim is FALSE.** `src/edn_shim.rs:92` calls `wat_edn::write_pretty` (EDN pretty-print), NOT `wat_edn::to_json_string_pretty` (JSON pretty-print). The orchestrator conflated the two pretty functions when justifying the rune in Stone 218.6c.

**Verified consumer audit (2026-05-22 recast):**
- `to_json_string` — live at `src/edn_shim.rs:105` + `:166` (WAT_TEST_OUTPUT cargo integration) + `crates/wat-edn/interop-tests/src/bin/json_producer.rs:12`
- `to_json_string_pretty` — **zero current consumers** anywhere in `src/`, `crates/`, or `interop-tests/`
- `write_pretty` — live at `src/edn_shim.rs:92` (this is what 218.6c rune mis-cited)

**The rune is a LIE.** Per `feedback_inscription_immutable` we don't amend past INSCRIPTIONs, but runes are LIVE annotations — they must hold under scrutiny or they're suppressing findings rather than naming exemptions.

**Honest fix paths:**
- **(α) Rewrite the rune** — drop the false edn_shim consumer claim; honest justification becomes: symmetric pretty variant of actively-used `to_json_string` + IPC-BRIDGE.md vision (line 95) + impressive JSON bridges ship both compact and pretty forms. The function stays per user direction 2026-05-22 ("we keep the JSON support").
- **(β) Retire** — symmetry-alone justification arguably doesn't meet user's high-bar "runes require significant justification"; if so, retire `to_json_string_pretty` per the parse_wire pattern.

User direction 2026-05-22 was: *"we keep the JSON support - we just need to make sure its fucking impressive."* The JSON bridge IS impressive partly BECAUSE it ships both compact + pretty variants symmetrically. Retiring the pretty variant would make the bridge less impressive. So path (α) is the load-bearing fix — but the rune must be HONEST about the lack of current consumer.

---

## L2 — Polish findings (5 total)

### intueri (2) — cascade-rename incompleteness from Stone 218.6d

1. **`crates/wat-edn/src/lexer.rs:191`** — local variable `open_pos` in `lex_string` was NOT renamed when 218.6d renamed the callee `lex_string_escaped`'s parameter to `quote_start`. The caller's local + the callee's parameter now have different names for the same concept (position of opening `"` byte). Rename `open_pos` → `quote_start` in `lex_string` to complete the 218.6d intent.

2. **`crates/wat-edn/src/writer.rs:71`** — `all_scalar` wraps the (now-renamed) `is_inline_value` but kept the "scalar" naming. The function name now claims "all scalar" but delegates to `is_inline_value`, which includes `Value::Inst` (a timestamp) and `Value::Uuid` (structured ID) — neither is a scalar in any conventional sense. Rename to `all_inline` (or `all_inline_values`) to match the delegate.

### cernere (2) — Stone 218.6c demotion not propagated to docs + test-count drift

1. **`crates/wat-edn/docs/USER-GUIDE.md:404-421`** — USER-GUIDE §7 code example uses `edn_to_json` + `json_to_edn` as if public:
   ```rust
   use wat_edn::{to_json_string, to_json_string_pretty,
                 from_json_string, edn_to_json, json_to_edn};
   let jv: serde_json::Value = edn_to_json(&v);
   let back: OwnedValue = json_to_edn(&jv)?;
   ```
   But Stone 218.6c demoted both to `pub(crate)` and removed them from `lib.rs:84-87`. **The example would FAIL TO COMPILE.** Remove the section OR replace with a note that these are internal mechanics.

2. **`crates/wat-edn/README.md:45 + :97` and `docs/USER-GUIDE.md:792`** — test counts stale and mutually inconsistent:
   - README:45: "313 Rust tests + 39 Clojure tests, all green"
   - README:97: "342/342 passing"
   - USER-GUIDE:792: "313 Rust + 39 Clojure (96 assertions)"
   - **Reality:** 344 Rust tests (verified post-218.6d ship)
   
   Update all three sites to the actual count.

### purgare (1) — write_to rune category mismatch

3. **`crates/wat-edn/src/writer.rs:195`** — `write_to` carries `purgare(public-api)` rune but the stated justification is forward-looking ("future Clojure-IPC bridge", "documented in IPC-BRIDGE.md:95"). Zero current external callers. Per SKILL.md category definitions:
   - `public-api` — "exported for downstream consumers outside this codebase"
   - `future-fixture` — "test fixture for a planned test not yet written. The rune retires when the test lands"
   
   The category should be `future-fixture` (or equivalent "future-vision" semantics) — the IPC-BRIDGE.md citation IS a planned-but-not-yet-built downstream. Re-rune to honest category.

---

## Existing runes — verification

5 runes encountered across all 7 spell casts:

1. `json.rs:170` `temperare(serde-api-shape)` on `to_json_string` — CLEAR (all spells confirmed)
2. `json.rs:179` `purgare(public-api)` on `to_json_string_pretty` — **QUESTIONABLE** per L1-PUR above (false consumer claim)
3. `json.rs:187` `temperare(serde-api-shape)` on `to_json_string_pretty` — CLEAR
4. `writer.rs:195` `purgare(public-api)` on `write_to` — **QUESTIONABLE** per purgare L2 above (wrong category)

The 2 temperare runes hold strongly. The 2 purgare runes (added Stone 218.6c) both have wording issues that need correction. Substrate-as-teacher: my justifications were not as strong as I claimed in Stone 218.6c.

---

## What this means for arc 218

Per `feedback_any_defect_catastrophic` + IMPECCABLE = zeros — arc 218 NOT YET CLOSED.

**But the trajectory is unmistakable:**
- 2 L1 + 26 L2 → 1 L1 + 5 L2 over 4 casts and 5 substantive stones
- 4 of 7 spells CONVERGED (was 1)
- The remaining 6 findings are all mechanical cascade + rune-honesty

**Stone 218.6e shape (6 items):**
- intueri L2 ×2: rename `open_pos`→`quote_start` in lex_string body; rename `all_scalar`→`all_inline`
- cernere L2 #1: remove demoted-functions section from USER-GUIDE §7 (or replace with internal-mechanics note)
- cernere L2 #2: fix README + USER-GUIDE test counts to 344
- purgare L1: rewrite `to_json_string_pretty` rune (drop false edn_shim claim; rephrase as symmetry + vision)
- purgare L2: re-rune `write_to` from `public-api` to `future-fixture`

Predicted runtime: ~5-10 min (smallest stone yet; all mechanical or rune wording).

Then ANOTHER vigilia recast. If CONVERGED across all 7 spells → IMPECCABLE achieved within wat-edn scope.

---

## Substrate-as-teacher meta-finding

The IMPECCABLE recast confirmed the cascade pattern at the deepest layer yet: the discipline I applied in Stone 218.6c had two soft spots:
1. The `to_json_string_pretty` rune was based on a CONFLATION (mistook write_pretty for to_json_string_pretty during the consumer audit)
2. The `write_to` rune used the wrong category per the SKILL definitions

Both are honest failures — purgare exists exactly to catch them. The discipline that found them is the same discipline that demanded the runes in the first place. The substrate's checks held; the cascade continues; one more stone closes it.

*Four guards stood silent. Three found dust the others missed. The whole guards itself.*
