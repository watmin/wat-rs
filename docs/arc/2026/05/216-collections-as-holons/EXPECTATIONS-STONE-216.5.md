# EXPECTATIONS — Arc 216 Stone 216.5 — `hashmap_key` full coverage

Mode A target: 16/16 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `hashmap_key` audit | Sonnet verified gap matches {Vec, HashMap, WatAST}; surfaced any additional gaps via STOP-3 before extending |
| 2 | `Value::Vec` arm added | `src/runtime.rs` — match arm produces `"Vec:[k1,k2,...]"` (or chosen scheme); order preserved; recursive `hashmap_key` per element; collision-safe |
| 3 | `Value::wat__std__HashMap` arm added | `src/runtime.rs` — match arm produces `"Map:{(k1=v1),(k2=v2),...}"` (or chosen scheme); sorted by k for determinism; both K and V recursive |
| 4 | `Value::wat__WatAST` arm added | `src/runtime.rs` — match arm mirrors HolonAST pattern at lines 9337-9343; DefaultHasher; `"W:{hash}"` (or chosen scheme) |
| 5 | Diagnostic message updated | `other =>` arm enumerates new accepted set honestly: includes Vec, HashMap, WatAST in the "expected" string |
| 6 | All `hashmap_key` callers audited | Sonnet grepped for callers; confirmed uniform benefit; surfaced any pre-filtering blockers |
| 7 | `tests/probe_verify_hashset_of_vector_gap.rs` flips GREEN | Same probe, same assertion; no longer panics; documents the gap that existed |
| 8 | Probe 1 — HashSet<Vector<i64>> round-trip | Forward + reverse; length preserved; dedupe semantic via canonical key |
| 9 | Probe 2 — HashSet<HashMap<keyword, i64>> round-trip | Same shape; HashMap as element |
| 10 | Probe 3 — HashSet<WatAST> round-trip OR documented skip | If WatAST values not constructible at WAT surface, document why with the substrate citation |
| 11 | Probe 4 — HashMap<Vector<i64>, String> round-trip | Vector as K; both directions |
| 12 | Probe 5 — HashMap<HashMap<keyword, i64>, String> round-trip | HashMap as K |
| 13 | Probe 6 — HashMap<WatAST, String> OR documented skip | Same WatAST consideration as Probe 3 |
| 14 | Probe 7 — Nested HashSet<Vector<HashSet<i64>>> round-trip | Three-deep nesting; all canonical keys compose recursively |
| 15 | Probe 11 — Collision-safety test | Two distinct Vecs that would collide under naive comma-join produce DIFFERENT canonical keys |
| 16 | 216.4 Probe 3 relanded + SCORE | `probe_arc216_stone4_predicate_composition.rs` Probe 3 flipped back to `HashSet<Vector<i64>>` per original BRIEF; 11/11 still passes; SCORE-STONE-216.5.md inscribed |

(Probes 8/9/12 from BRIEF compress into the rows above; row count is the contract for shipping.)

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 105 min
**Confidence:** medium-high

**Rationale:**
- Three new arms in one function; mechanical pattern (recursive `hashmap_key` calls + canonical string composition)
- HolonAST pattern at lines 9337-9343 is the template for WatAST arm
- HashSet arm at lines 9351-9355 (sort + join) is the template for HashMap arm
- Vector arm is novel only in that it has no sort step (order preserved) — straightforward
- Risk: collision-safety scheme choice is genuinely subtle; probe 11 is the gate. Sonnet may pick comma-join and fail probe 11; the recovery is to switch to length-prefix or hash-of-serialized scheme. Document the iteration in SCORE.
- Risk: WatAST constructibility at WAT surface uncertain; if probes 3+6 can't be written, document the skip with substrate citation (probably needs `:wat::core::quote` form or similar).
- New STOP triggers (1+2) tighten the gate. Sonnet has explicit guidance against the 216.4 failure modes.

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites all four prior 216.x SCOREs by commit; sonnet sees the full lineage and the failure modes named.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Sandbox-scope walker validation — Stone 216.6
- INSCRIPTION + closure — Stone 216.7
- Any predicate refactor (`is_atomizable` unchanged in this stone — it's already correct; the gap is in the runtime)

## Honesty deltas accepted

- Canonical-key scheme choice (comma-join with escaping vs length-prefix vs hash-based) — sonnet picks; documents in doc-comment + SCORE
- WatAST constructibility at WAT surface — if probes 3+6 can't be written, document the skip with substrate citation
- If audit surfaces additional gaps beyond {Vec, HashMap, WatAST} — STOP-3 fires; orchestrator absorbs or opens new stone
- Probe count: 16 is the contract; if WatAST probes (3+6) skip, replace with two additional collision-safety probes or composition probes to hold the row count

## Honesty deltas NOT accepted (the post-216.4 sharpening)

- **Probe substitution (changing what's tested to make it pass) — STOP-2 trigger, not a delta.** Stone 216.4 Delta 2 was the failure mode; this EXPECTATIONS explicitly disallows the move.
- **Pre-emptive code beyond stone's scope — STOP-1 trigger, not a delta.** Stone 216.1 Delta 6 was the original drift; this EXPECTATIONS explicitly disallows the move.
- **"Future arc" labeling for in-scope gaps — STOP-3 trigger, not a delta.** Surfaced gaps stay in this arc unless orchestrator says otherwise.
