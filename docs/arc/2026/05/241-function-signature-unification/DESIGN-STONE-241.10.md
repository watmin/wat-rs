# DESIGN — Stone 241.10 — `src/remedy/` + ranked-remedy schema (substrate teaches with receipts)

**Status:** READY (sub-DESIGN). Phase 3 third stone (NEW; was 241.10 define HARD CUT — define renumbered to 241.11; INSCRIPTION to 241.12 per user direction "renumbering feels honest" 2026-05-28). **VIGILIA-GATED** namespaced home per `feedback_namespaced_home_vigilia_gate`; user direction 2026-05-29: *"{src,tests}/remedy/*.rs must be remarkable — manifest it."* The bar is L0 + L1+L2=0 vigilia convergence. SCORE-green is the floor; remarkable is the goal.

**Why this stone before 241.11 (define HARD CUT):** the bandaid-rip lands on a substrate that TEACHES. After 241.10 ships, every `:wat::core::define` typo'd or stale at any future moment surfaces *"did you mean `:wat::core::defn`?"* — substrate teaches at exactly the friction moment. Without 241.10, 241.11's HARD CUT lands silent; users see "unknown form" with no path forward.

## Scope warning — substantial

- New home `src/remedy/` (4 files: mod.rs + distance.rs + retirement.rs + rank.rs)
- New types `Remedy` + `RemedyKind`
- **Schema upgrade** on `CheckError` + `TypeError` variants: `hint: Option<String>` → `remedies: Vec<Remedy>` (HARD CUT — additive shape rejected per four-questions, see D1)
- Display formatting (flatten ranked structure to text)
- Wire-in to ~10 unknown-X error construction sites
- Test cascade across hint-asserting tests (~20-40 sites likely)
- Vigilia cast (8 spells)

**Predicted band: 120-180 min Mode A.** Larger than 241.8 (~41 actual) and 241.9 (TBD) because schema change ripples + vigilia gate adds verification cycles. STOP-3 hard limit: 240 min.

## Why this stone

Per user direction 2026-05-29 ("ranked remedies with scores") and the substrate's existing trajectory:
- `hint: Option<String>` flattens at the wrong layer — strings are human-facing prose; programmatic consumers (LLM agents, IDE, telemetry) get no structure
- arc 233 (substrate-errors-as-values) already established: errors are DATA, not strings
- Carrying remedies as ranked structured data is consistent
- Walks into Convergence #18 (Lisp condition-system territory): multiple restarts presented to user; `find-restart` selects by name; CL's `compute-restarts` is structurally this

Convergence-with-self pattern (per `user_no_literature`): the substrate already had the shape (`hint:` field; errors-as-values doctrine); we extend it to its honest expression (structured ranked remedies).

## What this stone delivers

### S1 — Mint `src/remedy/` namespaced home

```
src/remedy/
├── mod.rs         — public API surface; `Remedy` + `RemedyKind` exports
├── distance.rs    — Levenshtein edit-distance helper (~25 lines)
├── retirement.rs  — explicit retirement-form → replacement table
└── rank.rs        — combining + ranking logic; threshold tuning
```

**`Remedy` shape:**

```rust
pub struct Remedy {
    pub form: String,           // e.g. ":wat::core::defenum"
    pub score: u32,             // edit distance for Typo; 0 for Retirement
    pub kind: RemedyKind,
}

pub enum RemedyKind {
    Typo,                       // Levenshtein-derived from candidate set
    Retirement,                 // Explicit retirement-table lookup
}

impl Ord for Remedy { /* ascending by score; ties broken lex */ }
```

**`nearest_match` API:**

```rust
pub fn nearest_match<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy>;            // sorted ascending; threshold = max(1, needle.len() / 3); top-N capped at 5
```

**`retirement_lookup` API:**

```rust
pub fn retirement_lookup(needle: &str) -> Option<Remedy>;  // explicit table check
```

**`remedies_for` convenience:**

```rust
pub fn remedies_for<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy>;            // retirement check FIRST; then typo; merged sorted
```

### S2 — Schema upgrade on error variants (HARD CUT)

Replace `hint: Option<String>` with `remedies: Vec<Remedy>` on:
- `CheckError::MalformedForm`
- `CheckError::ReturnTypeMismatch` (if present)
- `TypeError::MalformedDecl`
- `TypeError::MalformedVariant`
- All other variants currently carrying `hint:` (grep -n "hint:" src/types.rs src/check.rs)

Empty Vec = no remedy (was `None`). Non-empty Vec = ranked candidates. The Option wrapper is REJECTED per `feedback_no_semantic_abuse_of_option` (empty-vec IS the absence; Option<Vec> would be flavor abuse).

### S3 — Display formatting

`impl Display for CheckError` / `TypeError`:
- 0 remedies → no remedy section rendered
- 1 remedy → single-line: `  did you mean: :wat::core::defenum`
- ≥2 remedies → multi-line ranked list:
  ```
    did you mean:
      :wat::core::defenum    [typo, distance 2]
      :wat::core::defstruct  [typo, distance 4]
  ```
- Retirement-kind remedies get distinct annotation:
  ```
    did you mean:
      :wat::core::defn       [retirement replacement]
  ```

### S4 — Retirement table seeding

`src/remedy/retirement.rs` ships with arc 241 retirements:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 entry ADDED at 241.11 ship time (not pre-emptively this stone)
];
```

Future HARD CUT stones append their retirement entries. The table is substrate self-documentation that survives across arc boundaries — substrate remembers its own evolution.

### S5 — Wire-in to ~10 unknown-X error construction sites

Per the substrate-as-teacher discipline: the cascade is the migration brief. Existing sites with hand-written `hint:` strings convert to `remedies: remedies_for(needle, candidates)` calls. Per-error-kind candidate sets:

- Unknown form head → `special_forms` registry + `classify_type_decl` arms
- Unknown type name → `TypeEnv` registered types
- Unknown binding → `SymbolTable` scope chain (current scope first; outer scopes fallback)
- Unknown variant → relevant `EnumDef.variants`

Sites identified by `grep -n "hint:" src/{types,check}.rs` (~20-30 hand-written hint sites).

### S6 — Probe (FM 2-bis disconfirming)

`tests/probe_arc241_stone10_remedy.rs`. Contracts:

1. **C01** — typo case: `(:wat::core::defenmu :T)` → error with `remedies` containing `:wat::core::defenum` (score ≤ 2)
2. **C02** — retirement case: `(:wat::core::struct :T (f :Type))` → error with `remedies` containing `:wat::core::defstruct` (kind=Retirement, score=0)
3. **C03** — both: `(:wat::core::struc :T)` (close to struct AND defstruct) → ranked output naming both
4. **C04** — no remedy: `(:wat::core::completely-unknown :T)` → empty remedies Vec
5. **C05** — Display single-remedy formatting
6. **C06** — Display multi-remedy formatting
7. **C07** — Display retirement-kind annotation
8. **C08** — `nearest_match` direct API: needle + small candidate set → expected ranked output

### S7 — Vigilia cast (8 spells)

Per `feedback_namespaced_home_vigilia_gate` and arc 241 Stone 241.1 precedent:

| Spell | Concern | Acceptance |
|---|---|---|
| **intueri** | Name speaks | L1+L2=0 |
| **solvere** | No braided concerns | L1+L2=0 |
| **purgare** | No dead code | L1+L2=0 |
| **struere** | Structure mirrors discipline | L1+L2=0 |
| **sequi** | Imports follow domain | L1+L2=0 |
| **temperare** (always-apply) | No magic numbers without doc | L1+L2=0 |
| **complectens** | Test shape (for `src/remedy/`'s own tests) | L1+L2=0 |
| **vocare** | Caller-perspective tests verify what callers see | L1+L2=0 |

Vigilia CONVERGED = remarkable. Any L1 or L2 finding amends BEFORE commit.

## Locked decisions

### D1 — HARD CUT on `hint:` field (REPLACE, not augment)

Four-questions cast:

| Axis | Path A (replace) | Path B (additive: keep hint + add remedies) |
|---|---|---|
| Obvious? | YES | NO (two fields for same concept) |
| Simple? | YES (atomic act) | NO (decomposed unnecessarily) |
| Honest? | YES (no legacy state on disk) | NO (implies hint is canonical; truth is remedies subsumes hint) |
| Good UX? | YES (one structure) | NO (consumers check both) |

**Path A wins 4/4. HARD CUT on the field.** Mirrors arc 241's broader HARD CUT discipline.

### D2 — `remedies: Vec<Remedy>` (NOT `Option<Vec<Remedy>>`)

Per `feedback_no_semantic_abuse_of_option`: empty-vec IS absence; wrapping in Option would be flavor abuse. Empty `vec![]` = no remedy offered.

### D3 — Edit-distance threshold = `max(1, needle.len() / 3)`

Per Rust compiler precedent. Adjustable as a constant in `src/remedy/distance.rs`; not user-tunable. Tighter thresholds for short identifiers; looser for long.

### D4 — Candidate sets scoped per error kind

Substrate already has the authoritative per-kind tables (TypeEnv / SymbolTable / special_forms / EnumDef.variants). The remedy home does NOT introduce a new global symbol table.

### D5 — Ordering: ascending by score, lex tiebreaker

Lowest distance first. Ties broken lexicographically. Consumers (Display, programmatic) read in confidence order.

### D6 — Retirement table: explicit static, not heuristic

`retirement.rs` ships a hardcoded `&[(retired, replacement)]` table. No heuristic; no fuzzy match. Retirement is a deliberate language history event, not a distance-driven guess.

### D7 — Display format

- 0 remedies → no section
- 1 remedy → single-line inline
- ≥2 remedies → multi-line block
- Kind annotation always present (`[typo, distance N]` or `[retirement replacement]`)

### D8 — Top-N cap = 5

If `nearest_match` finds >5 candidates within threshold, return top 5 only. Beyond 5 = noise; reader can't usefully discriminate. Constant in `src/remedy/rank.rs`.

### D9 — Vigilia-gated; remarkable bar (user direction)

Per `feedback_namespaced_home_vigilia_gate` D7 ELEVATED: this is not a default-no-gate flat substrate; it's a new namespaced home AND the user has explicitly raised the bar. 8-spell vigilia cast; L1+L2=0 convergence required pre-commit. Cycles allowed (find → amend → re-cast → re-amend).

### D10 — Top-level lazy evaluation

`remedies_for` is called from error construction paths. To avoid Levenshtein work on every error, the call is invoked ONLY when an "unknown X" path actually fires (not as a defensive pre-compute). Per `temperare` discipline (no efficient-waste).

## Trap-door audit

### T1 — Schema change ripples through ~20-30 hint-asserting tests

Existing tests assert on hint strings (e.g., `assert!(msg.contains("did you mean"))`). Each rewrites to check the new structured field. Per substrate-as-teacher: cascade is migration brief.

### T2 — Display tests assume single-string hint format

Tests rendering full error messages may break on format changes. Migrate per Pattern A (single-line) / Pattern B (multi-line) per D7.

### T3 — `nearest_match` candidate scoping requires reading each error-site's local context

Per-site work: identify the candidate set (TypeEnv? SymbolTable? variants?) before calling `nearest_match`. Mechanical but per-site; not bulk-replaceable.

### T4 — Lev O(n*m) at every error site

For short identifiers (~20-30 chars) and candidate sets of <100, the cost is negligible. If a candidate set exceeds 1000 (e.g., very large enum or huge type env), early-exit on threshold-exceeded is the optimization. NOT load-bearing this stone; future profile-driven optimization.

### T5 — Retirement-table needs Stone 241.11 entry added AT 241.11

The table is populated incrementally. Stone 241.11 (define HARD CUT) appends `(:wat::core::define, :wat::core::defn)`. NOT pre-emptively this stone (would be vapor entry for non-yet-retired form).

### T6 — Vigilia divergence handling

If 8-spell cast surfaces L1/L2 findings, amend pre-commit. Cycles allowed. If after 3 amend cycles vigilia is still divergent, STOP-3 invoked; orchestrator surfaces friction to user.

### T7 — Existing `hint:` field consumers outside src/{types,check}.rs

If `hint:` is referenced from `src/runtime.rs` or elsewhere, schema change ripples there too. Pre-spawn grep:
`grep -rn "\.hint\b\|hint:" src/ | grep -v "^src/remedy/"`

### T8 — EnumDef.variants traversal post-241.9

`EnumDef.variants` shape post-241.9 (defenum) needs candidate-extraction path. Sonnet investigates 241.9 SCORE for storage shape. If 241.9 shipped clean, variant names are straightforward.

## STOP triggers — REJECTION

1. Compile errors not traced to schema migration or wire-in sites
2. Lib < 834 (post-cascade-migration final state)
3. **240 min elapsed** (extended for vigilia cycle + schema cascade; HARD CUT discipline)
4. holon-rs touched
5. Files outside `src/remedy/*`, `src/types.rs`, `src/check.rs`, `src/runtime.rs` (if hint usage), `src/error.rs` (if exists), the hint-asserting test files, `tests/probe_arc241_stone10_*`, SCORE doc, the retirement table seed entries
6. Scope creep: define HARD CUT (241.11); INSCRIPTION (241.12); new error variants beyond schema upgrade; new VSA-similarity tooling; per-error-kind context refactoring beyond candidate-set scoping
7. Stone 241.10 probe < 8/8
8. Stone 241.1-241.9 probes regress; arc 237/238 probes regress
9. Clippy > 902
10. Adding hint field BACK alongside remedies (HARD CUT violation)
11. **Vigilia divergence after 3 amend cycles** (escalate to orchestrator)
12. `feedback_no_semantic_abuse_of_option` violation: `Option<Vec<Remedy>>` instead of `Vec<Remedy>`
13. Heuristic retirement matching (D6 violation — retirement table is explicit)

## FM 2-bis evidence

`tests/probe_arc241_stone10_remedy.rs` (NEW). 8 contracts. Disconfirms at HEAD:
- C01 fails (no nearest_match infrastructure; hint field is bare string with no rank)
- C02 fails (no retirement table; struct→defstruct mapping not surfaced)
- C03-C08 fail (display formatting absent; nearest_match API absent)

Post-stone: all 8 PASS.

## Calibration

**Target band: 120-180 min Mode A.**
**Upper bound: 240 min (STOP-3).**

| Component | Pre | Post | Delta |
|---|---|---|---|
| `src/remedy/mod.rs` | 0 | ~80 | **+80** |
| `src/remedy/distance.rs` | 0 | ~50 | **+50** |
| `src/remedy/retirement.rs` | 0 | ~60 | **+60** |
| `src/remedy/rank.rs` | 0 | ~70 | **+70** |
| `src/types.rs` schema | (current `hint:`) | (`remedies:`) | mixed |
| `src/check.rs` schema | (current `hint:`) | (`remedies:`) | mixed |
| `tests/probe_arc241_stone10_remedy.rs` | 0 | ~250 | **+250** |
| ~20-30 hint-asserting test migrations | various | structured-checks | mixed |
| **Net delta** | — | — | **substantial mixed (+500-800 lines net)** |

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites `SCORE-STONE-241.9.md` for cascade migration shape; cites `SCORE-STONE-241.1.fix.md` for vigilia-gate cycle precedent.

## What this unblocks

**Stone 241.11** — `define ⇒ defn` HARD CUT. With remedy infrastructure live, the HARD CUT lands on a substrate that TEACHES: every `:wat::core::define` typo'd or stale surfaces *"did you mean `:wat::core::defn`?"*. Substrate teaches at the friction moment. The bandaid-rip is rip-with-receipts.

**Stone 241.12** — INSCRIPTION closes arc 241. Per `feedback_no_regression_until_arc_done`: arc 237.8b reopens AFTER 241.12.

**Future arcs that consume remedy infrastructure:**
- Any future HARD CUT (form rename, type retirement) registers its retirement entry; substrate self-documents its evolution
- LLM-agent-facing error consumers (data-not-strings); IDE quick-fix integration; telemetry on which remedies users accept
- The substrate has memory of its own evolution across arc boundaries (data-as-history)

## Convergence #18 (provisional)

**Lisp condition-system** — multiple restarts presented to the user; `find-restart` selects by name; CL's `compute-restarts` is structurally this. We arrive via errors-as-values (arc 233) + ranked structured remedies + EDN serialization. Independent path; same room.

To be inscribed in INTERSTITIAL on 241.10 ship as Convergence #18 candidate (pending verification that the parallel is structural not surface-coincidental).

---

**Recommendation**: this DESIGN is committed for review. Stone 241.10 commits to ~120-180 min sonnet work + vigilia cycles. Schema upgrade is substantial; cascade through hint-asserting tests; vigilia gate raises bar.
