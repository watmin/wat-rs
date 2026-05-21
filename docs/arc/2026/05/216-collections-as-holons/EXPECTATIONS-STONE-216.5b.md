# EXPECTATIONS — Arc 216 Stone 216.5b — HashSet storage refactor

Mode A target: 13/13 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Value enum variant updated | `Value::wat__std__HashSet(Arc<HashSet<Value>>)` in `src/runtime.rs`; canonical-key String storage gone |
| 2 | Stone 216.5a PartialEq arm simplified | Native `a == b` on `Arc<HashSet<Value>>` (replaces manual iter-and-compare) |
| 3 | Stone 216.5a Hash arm simplified | Iterates `s.iter()` (Values directly); sort-then-hash semantics preserved |
| 4 | `eval_hashset_ctor` refactored | Direct `set.insert(v)` on `HashSet<Value>`; `hashmap_key` call removed from constructor |
| 5 | Accessor verb refactor — `contains?` | `set.contains(&v)` native; `hashmap_key` call removed |
| 6 | Accessor verb refactor — `conj` | Native insert; Arc::make_mut OR new-Arc strategy chosen + documented |
| 7 | Accessor verb refactor — `dissoc` | Native remove; same Arc strategy |
| 8 | `value_to_atom` HashSet arm refactored | Iterates `s.iter()` (Values); Bundle output unchanged |
| 9 | `hashmap_key` HashSet arm refactored | Iterates `s.iter()` (Values); recursive `hashmap_key(op, v)` per element; sorted+joined string output unchanged |
| 10 | Caller sweep complete | All HashSet-internal `hashmap_key` callers refactored; HashSet-as-key callers untouched (their hashmap_key call still works) |
| 11 | Probes 1-10 from BRIEF | All new probes pass; nested HashSet works; HashMap-of-HashSet works (both as value AND as key); round-trip through Atom preserved |
| 12 | Prior probe suites GREEN | 216.1 (10/10), 216.2 (12/12), 216.3 (14/14), 216.4 (6/6), 216.5 (12/12), 216.5a (22/22), verify-probe (1/1), arc 214 slice 4, arc 215 — all unchanged |
| 13 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5b.md` — scorecard + caller refactor count + Arc strategy + deltas + verification summary + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 75-105 min Mode A
**Upper bound:** 120 min
**Confidence:** medium

**Rationale:**
- Storage refactor with many small touches (constructor + accessors + dispatch arms + value_to_atom + hashmap_key arm)
- Pattern: native Rust HashSet operations replace canonical-key-string operations; mechanical translation
- 216.5a's impl Hash makes the native operations work; the refactor leverages the foundation
- Risk: `Arc<HashSet<Value>>` mutation — Arc::make_mut requires Clone bound on HashSet<Value> (which requires Clone on Value — already true); OR build new Arc each time (cleaner but allocates). Sonnet picks.
- Risk: HashSet of nested HashSet — Stone 216.5a's recursive Hash via sort-then-hash needs to work end-to-end; probe 7 + 10 gate this
- Risk: dispatch site count larger than expected — STOP-5 fires; sub-decomposition decision

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites 216.5 (caller audit), 216.5a (impl Hash), 216.1 (HashSet round-trip contract). Sonnet has the full lineage.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- HashMap storage refactor — Stone 216.5c
- `hashmap_key` deletion — Stone 216.5d
- Sandbox-walker validation — Stone 216.6
- INSCRIPTION — Stone 216.7
- Any Value variant OTHER than `wat__std__HashSet`
- Any `hashmap_key` arm OTHER than the HashSet arm

## Honesty deltas accepted

- Arc::make_mut vs new-Arc for conj/dissoc — sonnet picks; documents
- Caller count surprises (more or fewer than 216.5 audit suggested) — sonnet surfaces; documents
- Polymorphic dispatch arm refactor specifics (which file each arm lives in) — sonnet audits; documents
- If a HashSet caller has a subtle invariant requiring it to keep using `hashmap_key` even after the refactor — surface + document; not blocking
- If `Arc<HashSet<Value>>` mutation requires more careful handling than Arc::make_mut (e.g., shared ownership scenarios) — surface; document

## Honesty deltas NOT accepted

- **Probe substitution — STOP-3.**
- **HashMap storage refactor leak — STOP-1.**
- **`hashmap_key` deletion leak — STOP-2.**
- **Caller behavior change — STOP-4.** (Returns different type, different errors, different ordering.)
- **Silently skipping a dispatch site — STOP-5.**
