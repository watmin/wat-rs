# EXPECTATIONS — Arc 216 Stone 216.4 — Atomizable predicate consolidation + verification

Mode A target: 11/11 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `fn is_atomizable` audit | Confirm primitive baseline (i64, String, bool, keyword, byte, HolonAST, WatAST, char) + three collection arms (HashSet, Vector, HashMap) all present and recursive; document at `src/check.rs:3600`. Type-alias resolution (e.g., `:wat::program::Env` → `HashMap<keyword, HolonAST>`) handled correctly |
| 2 | Comments consolidated | Stale "Stone N future" comments removed (Stones 216.1/216.2/216.3 all shipped); single canonical doc-comment on `fn is_atomizable` names the four categories |
| 3 | `infer_list` special-case verified | `:wat::holon::Atom | :wat::holon::leaf` arm correctly applies `is_atomizable(resolved)` post-inference; diagnostic names offending position |
| 4 | WAT-CHEATSHEET consolidation | Single canonical "Atomizable types" section; per-stone "future" markers updated to "shipped"; reference `fn is_atomizable` mechanism; composition examples included |
| 5 | Probe 1 — Composite HashMap-of-Vector | `:wat::holon::Atom (HashMap keyword (Vector i64))` type-checks and runs |
| 6 | Probe 2 — Composite Vector-of-HashSet | `:wat::holon::Atom (Vector (HashSet i64))` type-checks and runs |
| 7 | Probe 3 — Composite HashSet-of-Vector | `:wat::holon::Atom (HashSet (Vector i64))` type-checks and runs |
| 8 | Probe 4 — Triple-nested composition | `:wat::holon::Atom (HashMap keyword (Vector (HashSet i64)))` — all three collections; type-checks; runs |
| 9 | Probe 5 — Negative: non-atomizable element | `:wat::holon::Atom (Vector Function)` fails at check with diagnostic naming non-atomizable position |
| 10 | Probe 6 — Negative: non-atomizable K | `:wat::holon::Atom (HashMap Function i64)` fails at check with diagnostic naming non-atomizable position |
| 11 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.4.md` — scorecard + deltas + verification summary + elapsed time + audit findings |

## Independent prediction (calibration record)

**Target runtime:** 30-45 min Mode A
**Upper bound:** 60 min
**Confidence:** high

**Rationale:**
- Predicate code was pre-landed in 216.1 Delta 6; this stone is verification + documentation + composite probes
- Composite probes mechanically compose three established patterns (HashSet, Vector, HashMap); no new mechanism
- Doc consolidation is mechanical (find "Stone N future" → "Stone N"); no judgment calls
- Risk: composite probe surfaces a runtime bug masked by individual-type probes (e.g., shape-dispatch ordering interaction in nested composition); STOP-2 triggers
- Risk: audit surfaces a primitive missing from the predicate baseline (unlikely given 216.1's care)

**Per `feedback_stone_briefs_cite_prior_score`:** prior SCORE cites are integral to this stone; all three 216.x SCOREs reference the predicate state. Brief includes all three.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Sandbox-scope walker validation — Stone 216.5
- INSCRIPTION + closure — Stone 216.6
- Polymorphic dispatch for collection ops — arc 214 Slice 4 Stone 4.6 (separate arc deferred)

## Honesty deltas accepted

- If the predicate code is genuinely complete + correct: SCORE honestly says "no code changes needed beyond comment + doc polish" — this is the success case
- If type-alias resolution doesn't naturally hit the predicate (e.g., `:wat::program::Env` is checked before alias expansion): document the gap; defer the fix to a follow-up if structural
- Negative probe diagnostic format — sonnet picks the assertion form (substring match on TypeMismatch message vs. structural check); documents
- If `:wat::core::Function` isn't a valid type to use in negative probes (because it can't be parsed as an `Atom<>` arg syntactically), substitute the simplest equivalently non-atomizable type and document
