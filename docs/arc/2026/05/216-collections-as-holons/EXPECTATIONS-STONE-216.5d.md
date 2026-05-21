# EXPECTATIONS — Arc 216 Stone 216.5d — DELETE `hashmap_key`

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Caller audit | Comprehensive grep documents remaining `hashmap_key` callers after 216.5b+c; counts + per-site refactor approach in SCORE |
| 2 | Straggler callers refactored | Any non-internal-recursion caller refactored to use native `Value::Hash` (via DefaultHasher inline OR the new native HashSet/HashMap storage); STOP-1 surfaced any non-obvious cases |
| 3 | `fn hashmap_key` deleted | Function declaration + all 9 arms + doc-comment block + `other =>` TypeMismatch all removed from `src/runtime.rs` |
| 4 | Stone 216.5 throw-away arms deleted | Vec, HashMap, WatAST arms (added in Stone 216.5 as the canonical-key extension) die with the function; ~40 lines paid forward |
| 5 | Imports updated | Any `use ... hashmap_key` references removed |
| 6 | `value_is_hashable` decision | Option α (KEEP — defense-in-depth) OR Option β (RETIRE — risky); decision + rationale documented in SCORE; STOP-3 forced double-check if β |
| 7 | WAT-CHEATSHEET updated | "Hashable types" subsection rewritten to describe `impl Hash for Value` + `is_atomizable` predicate as canonical mechanism; all `hashmap_key` references removed; canonical-key scheme docs removed |
| 8 | `probe_arc216_stone5_hashmap_key_coverage.rs` deleted | Test file removed (its subject — `hashmap_key` — no longer exists); coverage now provided by 216.5b + 216.5c probe matrices |
| 9 | `probe_verify_hashset_of_vector_gap.rs` doc updated | Doc-comment updated to reflect "the gap is closed; canonical-key crutch is gone; this probe is historical evidence." Test still passes. |
| 10 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5d.md` — scorecard + straggler count + value_is_hashable decision + line count deleted + CHEATSHEET deltas + probe file deletion + verification summary + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 105 min
**Confidence:** high

**Rationale:**
- Deletion + cleanup; simpler than 216.5b/c (which were refactors)
- Most callers should already be gone (216.5b refactored HashSet usage; 216.5c refactored HashMap usage; only stragglers remain)
- Risk: a straggler caller has a non-obvious refactor path; STOP-1 surfaces
- Risk: `probe_arc216_stone5_hashmap_key_coverage.rs` tests downstream user-facing behavior (not just hashmap_key directly); STOP-4 forces a pause to adapt rather than delete
- Risk: `value_is_hashable` retirement (Option β) introduces a Hash-panic vulnerability; STOP-3 forces double-check

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites 216.5a/b/c lineage; sonnet has the full antidote-sequence context.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Sandbox-walker validation — Stone 216.6
- INSCRIPTION + closure — Stone 216.7
- Any architectural change beyond `hashmap_key` removal
- Refactor of `is_atomizable` predicate

## Honesty deltas accepted

- Straggler caller count + per-site refactor specifics — sonnet's audit; documents
- `value_is_hashable` decision (α vs β) — sonnet picks; documents rationale
- Probe file deletion specifics (full delete vs adapt-and-keep) — sonnet picks per STOP-4 outcome; documents
- WAT-CHEATSHEET deltas (specific phrasing of the new "Hashable types" subsection) — sonnet drafts; documents
- Line count deleted — informational; honest measurement of poison removed

## Honesty deltas NOT accepted

- **Probe substitution — STOP-5 trigger, not a delta.**
- **Silently leaving a `hashmap_key` caller in place — STOP-1 trigger, not a delta.**
- **Retiring `value_is_hashable` without verifying every Hash-reaching path goes through check.rs — STOP-3 trigger.**
- **Deleting probe coverage that tests downstream behavior — STOP-4 trigger.**
