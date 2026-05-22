# Vigilia FINAL Recast — wat-edn — 2026-05-22 (post-218.6e)

**Cast:** 7 defensive spells in parallel against `crates/wat-edn/src/` (8 files) + `crates/wat-edn/tests/` (9 files).

**Practitioner's pre-cast assessment:** 0 L1 + 0 L2 expected (IMPECCABLE proof).

**Verdict:** **DIVERGES (0 L1 + 3 L2)** — almost there. **6 of 7 spells CONVERGED.** cernere finds 3 doc-drift mumbles that 218.6e missed.

---

## Trajectory across 5 casts

| Cast | Trigger | L1 | L2 | CONVERGED count |
|---|---|---|---|---|
| 2026-05-21 BASELINE | pre-arc-218 | 2 | 26 | 0/7 |
| 2026-05-21 RECAST | post-218.4 + 219 | 7 | 26 | 1/7 (sequi) |
| 2026-05-22 CHECKPOINT | post-218.6 + 218.6b | 9 | 14 | 1/7 (sequi) |
| 2026-05-22 IMPECCABLE | post-218.6c + 218.6d | 1 | 5 | 4/7 |
| **2026-05-22 FINAL** | **post-218.6e** | **0** | **3** | **6/7** |

**Significance: 0 L1 across all 7 spells.** First cast in arc-218's history with zero L1 findings — the substrate is now structurally honest. Remaining 3 L2 are entirely doc-drift (USER-GUIDE + one vocab.rs comment), no code changes needed.

---

## Per-spell summary

| Spell | Verdict | L1 | L2 | Notes |
|---|---|---|---|---|
| sequi | CONVERGED ✓ | 0 | 0 | State-threading discipline impeccable across all 5 casts |
| solvere | CONVERGED ✓ | 0 | 0 | Slash-split discipline + module structure clean |
| struere | CONVERGED ✓ | 0 | 0 | Functions well-built; types enforce; abstractions at caller level |
| temperare | CONVERGED ✓ | 0 | 0 | No redundant work; LazyLock + roundtrip_wire_str hold |
| intueri | CONVERGED ✓ | 0 | 0 | Every name speaks; cascade renames complete |
| purgare | CONVERGED ✓ | 0 | 0 | All `pub` items have live consumers or strongly-justified runes |
| cernere | DIVERGES | 0 | 3 | Three USER-GUIDE/vocab.rs doc-vs-impl drift items |

---

## L2 findings (3 total — all docs)

### cernere L2-1 — USER-GUIDE suite breakdown table stale

**`crates/wat-edn/docs/USER-GUIDE.md:790-792`** — Stone 218.6e fixed the headline test count (313/342 → 344) but the per-suite ATTRIBUTION table wasn't updated:

- Stale: "26 lib unit + 16 json + 7 round_trip + 36 spec_strict"
- Actual: "44 lib unit (includes 18 json tests, not separately counted) + 9 round_trip + 40 spec_strict + 23 wire_encoding"

`wire_encoding` suite isn't mentioned at all. Fix: regenerate the per-suite breakdown from `cargo test --release -p wat-edn` output.

### cernere L2-2 — "~2000 LOC" claim stale (2 sites)

**`crates/wat-edn/docs/USER-GUIDE.md:22, 932`** — Both occurrences claim "~2000 LOC". Actual: 3581 lines across 8 src files (verified via `wc -l crates/wat-edn/src/*.rs`). ~80% undersell. Update to "~3500 LOC" or precise count.

### cernere L2-3 — vocab.rs spec-quote vs implementation contradiction

**`crates/wat-edn/src/vocab.rs:80-87`** — The spec-quote block comment cites the EDN spec verbatim, including `"Additionally, `: #` are allowed as constituent characters."` Then `is_symbol_continue` at line 101 has a doc-comment explaining these are NOT permitted per arc 219's strict-EDN keyword namespace decision.

The two adjacent comments contradict at first read. A reader of the spec-quote block sees `: #` as allowed before reaching the override explanation. The test `is_symbol_continue_rejects_colon` in `spec_strict.rs` locks the override.

Fix: bracket the spec-quote with a note like *"NOTE: arc 219 overrides — `:` and `#` are excluded from wat-edn's `is_symbol_continue` per strict-EDN keyword namespace discipline."* OR remove the spec-quote's `: #` clause entirely.

---

## Existing runes — all CONVERGED CLEAR

4 runes encountered across all 7 spells, all verdicts CLEAR:

1. `json.rs:170` `temperare(serde-api-shape)` on `to_json_string` — CLEAR
2. `json.rs:179` `purgare(public-api)` on `to_json_string_pretty` — CLEAR (218.6e rewrite removed the false consumer claim; honest now)
3. `json.rs:189` `temperare(serde-api-shape)` on `to_json_string_pretty` — CLEAR
4. `writer.rs:195` `purgare(future-fixture)` on `write_to` — CLEAR (218.6e re-categorization with explicit retirement criterion)

Every rune holds strong justification. The discipline is operational.

---

## What this means for arc 218

**Pure IMPECCABLE = zeros across all 7 spells.** We're at 6/7 CONVERGED + 0 L1 + 3 L2 docs. NOT pure-IMPECCABLE yet.

Two honest paths:

**Path A — Close the 3 L2 docs (Stone 218.6f):**
- 3 surgical doc edits in 2 files
- Predicted: ~3-5 min
- Then a vigilia ABSOLUTE recast — if CONVERGED, full IMPECCABLE achieved
- Then arc 218 closure conversation per user direction

**Path B — Accept operational-IMPECCABLE + pivot to streaming:**
- Substrate is structurally honest (0 L1, runes clean, 6/7 CONVERGED)
- 3 L2 are pure docs, deferrable to a maintenance arc
- Move to user's named "deferred streaming optimization" as the actual arc 218 scope
- Risk: violates strict "impeccable = zeros" definition the user named

Per user direction 2026-05-22 ("impeccable is zeros - we still have non-zeros or no?") — Path A is the disciplined move. The discipline says: close the L2s, then proclaim IMPECCABLE, then pivot to streaming.

Stone 218.6f shape (3 items / ~5 min):
1. USER-GUIDE.md:790-792 — regenerate per-suite breakdown table
2. USER-GUIDE.md:22 + :932 — update "~2000 LOC" → actual count (verified via wc -l)
3. vocab.rs:80-87 — bracket spec-quote with arc-219 override note (or remove the `: #` clause)

After 218.6f ships + final vigilia recast comes back CONVERGED → **IMPECCABLE achieved within wat-edn scope.** Then arc 218 actual scope opens (the user-named deferred streaming optimization per `docs/IPC-BRIDGE.md:305` — "What might need work for streaming EDN").

---

## Substrate-as-teacher meta-finding

The cascade continues at the deepest layer. Stone 218.6e fixed the test-count HEADLINE but missed the per-suite TABLE 5 lines below. Stone 218.6e fixed the demoted-function example but didn't audit the "~2000 LOC" claims. Arc 219's `:`/`#` exclusion was inscribed in `is_symbol_continue`'s doc-comment but the spec-quote 20 lines above wasn't bracketed.

The pattern: every time the substrate moves, an adjacent comment ages. cernere is the spell that catches this; it remains diligent.

Per `feedback_any_defect_catastrophic` + user's "impeccable = zeros" framing: 3 L2 remain. Pure IMPECCABLE is one more 5-minute stone away.

*Six guards stood silent. One found three pieces of dust. The whole guards itself. One more sweep and the floor is clean.*
