# BRIEF — Stone 241.10 — `src/remedy/` + ranked-remedy schema (substrate teaches with receipts)

You are sonnet. Phase 3 third stone (RENUMBERED — was 241.10 define HARD CUT; user direction 2026-05-29 inserted remedy infrastructure before the bandaid-rip; define HARD CUT moved to 241.11; INSCRIPTION to 241.12).

**Bar: REMARKABLE.** User direction: *"{src,tests}/remedy/*.rs must be remarkable — manifest it."* This is a VIGILIA-GATED namespaced home — L1+L2=0 convergence required pre-commit. SCORE-green is the floor. Cycles allowed.

## What this stone does

Mint `src/remedy/` namespaced home that turns `hint: Option<String>` (flat prose) into `remedies: Vec<Remedy>` (ranked structured data). Substrate refuses + offers ranked candidates with kind annotation. Walks into Lisp condition-system convergence room (#18 candidate).

## What to do

### S1 — Mint `src/remedy/` namespaced home

Four files mirroring `src/argspec/` precedent (Stone 241.1):

- `src/remedy/mod.rs` — public exports (`Remedy`, `RemedyKind`, `nearest_match`, `retirement_lookup`, `remedies_for`)
- `src/remedy/distance.rs` — Levenshtein helper (Wagner-Fischer table; ~25 lines)
- `src/remedy/retirement.rs` — explicit retirement-form → replacement static table
- `src/remedy/rank.rs` — threshold tuning, top-N capping, kind merging

### S2 — Define `Remedy` + `RemedyKind` types

```rust
pub struct Remedy {
    pub form: String,           // ":wat::core::defenum"
    pub score: u32,             // edit distance for Typo; 0 for Retirement
    pub kind: RemedyKind,
}

pub enum RemedyKind {
    Typo,                       // Levenshtein-derived from candidate set
    Retirement,                 // Explicit retirement-table lookup
}

impl Ord for Remedy { /* ascending by score; ties broken lex on form */ }
```

### S3 — APIs

```rust
/// Top-N=5 cap; threshold = max(1, needle.len() / 3)
pub fn nearest_match<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy>;

/// Explicit retirement-table lookup; None if not retired
pub fn retirement_lookup(needle: &str) -> Option<Remedy>;

/// Merge retirement (priority) + typo (filtered + ranked); single call site sugar
pub fn remedies_for<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy>;
```

### S4 — Retirement table seeding

`src/remedy/retirement.rs` ships with arc 241 retirements ONLY (no future-vapor entries):

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum
    (":wat::core::enum",              ":wat::core::defenum"),
    // (Stone 241.11 entry added when 241.11 ships; do NOT pre-emptively add)
];
```

### S5 — Schema upgrade on error variants (HARD CUT)

Per D1 + D2 in DESIGN-STONE-241.10:

Replace `hint: Option<String>` with `remedies: Vec<Remedy>` on:
- `CheckError::MalformedForm` (check.rs)
- `TypeError::MalformedDecl` (types.rs)
- `TypeError::MalformedVariant` (types.rs)
- Any other variant currently carrying `hint:` (grep -n "hint:" src/{types,check}.rs)

Empty `Vec<Remedy>` = no remedy (was `None`). `Option<Vec<Remedy>>` is REJECTED per `feedback_no_semantic_abuse_of_option` (D2).

### S6 — Display formatting

`impl Display for CheckError` / `impl Display for TypeError` extend to render remedies:

- 0 remedies → no remedy section
- 1 remedy → single-line: `  did you mean: :wat::core::defstruct [retirement replacement]`
- ≥2 remedies → multi-line block:
  ```
    did you mean:
      :wat::core::defenum    [typo, distance 2]
      :wat::core::defstruct  [typo, distance 4]
  ```
- Annotation kinds:
  - `[typo, distance N]` for `RemedyKind::Typo`
  - `[retirement replacement]` for `RemedyKind::Retirement`

### S7 — Wire-in to existing error construction sites

Per the substrate-as-teacher discipline: the cascade is the migration brief. Migrate hand-written `hint:` strings to `remedies:` calls.

**Retirement paths (Stone 241.8 + 241.9 HARD-CUT arms):**

At `src/check.rs` Stone 241.8's HARD-CUT arm (currently around line 6936-6946):

```rust
// BEFORE (241.8 hand-written prose):
":wat::core::struct" | ":wat::core::struct-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.8); use ':wat::core::defstruct' instead", k),
        span: head_span.clone(),
    }]);
}

// AFTER (241.10 structured remedies):
":wat::core::struct" | ":wat::core::struct-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.8)", k),
        remedies: remedies_for(k, std::iter::empty()),  // retirement_lookup hits the table
        span: head_span.clone(),
    }]);
}
```

Stone 241.9's HARD-CUT arm for `:wat::core::enum` (mints in 241.9; you migrate per same pattern).

**Type-unknown paths:**

Where `TypeError::MalformedDecl` or `CheckError::ReturnTypeMismatch` (or similar) currently emits an unknown-type-name error, populate `remedies: nearest_match(needle, TypeEnv.iter_names())`.

**Binding-unknown paths:**

Where unknown-binding errors emit, populate `remedies: nearest_match(needle, SymbolTable.in_scope_names())`.

Sonnet identifies sites via:

```bash
grep -n "hint:" src/types.rs src/check.rs
grep -n "Unknown\|NotFound\|Undeclared" src/types.rs src/check.rs
```

### S8 — Cascade migration of hint-asserting tests

Per substrate-as-teacher: read failure → migrate → re-run. Tests asserting on hint strings convert to remedies-field checks OR (when only Display matters) to substring assertions on the formatted output.

### S9 — Probe verification

`tests/probe_arc241_stone10_remedy.rs` (already committed STRIKE-READY). 8 contracts; pre-stone 2/8; post-stone 8/8.

### S10 — Vigilia cast (8 spells) on `src/remedy/`

Per `feedback_namespaced_home_vigilia_gate` and arc 241 Stone 241.1 precedent. Spells: intueri, solvere, purgare, struere, sequi, temperare (always-apply), complectens, vocare. Acceptance: L1+L2=0 each. Amend cycles allowed. If 3 cycles + still divergent, STOP-11 (escalate to orchestrator).

The vigilia cast also covers `tests/remedy/` if you mint helper tests there (mirror argspec precedent).

## Discipline

- **HARD CUT on `hint:` field** — REPLACE with `remedies:`; no augmentation; no shim alongside
- **`Vec<Remedy>` NOT `Option<Vec<Remedy>>`** per `feedback_no_semantic_abuse_of_option` (D2)
- **No EnumDef/StructDef schema extension** beyond the error-variant field swap
- **`src/argspec/*` UNCHANGED** — canonical parser is stable from Stone 241.1.fix
- **`src/lib.rs`** — add `pub mod remedy;` only (no other changes)
- **holon-rs NEVER touched** (STOP-5; frozen)
- **No new error variants** — schema upgrade is field-shape change on existing variants
- **No VSA / `coincident?` reaches** — string edit-distance is the right geometry; VSA is for semantic similarity
- **Lazy invocation** — `remedies_for` called ONLY at error construction paths, never as defensive pre-compute (per `temperare`)
- **Retirement table is explicit static** — no heuristic; D6 lock
- **Top-N=5 cap** — beyond 5 = noise (D8)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — cascade discipline
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.10.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.10.md` — D1-D10 + T1-T8 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.9.md` — predecessor; cascade migration pattern
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` — vigilia-gate cycle precedent (the 8-spell convergence pattern)
7. `/home/watmin/work/holon/wat-rs/src/argspec/{mod,parse,error}.rs` — namespaced-home structural precedent
8. `/home/watmin/work/holon/wat-rs/src/check.rs` (especially the 241.8 HARD-CUT arm + 241.9 HARD-CUT arm post-241.9-ship)
9. `/home/watmin/work/holon/wat-rs/src/types.rs` — error variants carrying `hint:`
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone10_remedy.rs` — 8-contract probe

## Implementation sketch

1. Baseline confirmation: lib 834 PASS / 0 FAIL; Stone 241.10 probe = 2/8 PASS at HEAD (post-241.9-ship); clippy ≤ 902
2. **S1+S2+S3+S4**: mint `src/remedy/` (4 files); `Remedy`/`RemedyKind`; APIs; retirement-table seed
3. **S5**: schema upgrade — `hint:` → `remedies:` on error variants
4. **S6**: Display formatting for remedies on each affected error variant
5. **S7**: wire-in to existing error paths (retirement arms + type-unknown + binding-unknown)
6. Run lib tests. Cascade BEGINS (hint-asserting tests migrate).
7. **S8**: iterate cascade per substrate-as-teacher
8. **S9**: verify probe 8/8
9. **S10**: 8-spell vigilia cast on `src/remedy/`; converge L1+L2=0 (amend cycles allowed)
10. Final: lib ≥ 834; workspace compile clean; clippy ≤ 902; vigilia CONVERGED
11. Write `SCORE-STONE-241.10.md`
12. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to schema migration / wire-in / cascade
2. Lib < 834 (post-cascade final state)
3. **240 min elapsed** (HARD CUT cascade + vigilia cycle upper bound)
4. holon-rs touched
5. Files outside `src/remedy/*`, `src/types.rs`, `src/check.rs`, `src/runtime.rs` (if hint usage), `src/lib.rs` (`pub mod remedy;` only), hint-asserting test files, `tests/probe_arc241_stone10_*`, `tests/remedy/*` (vigilia tests if any), SCORE doc
6. Scope creep: define HARD CUT (241.11); INSCRIPTION (241.12); new error variants; new VSA tooling; per-error-kind context refactor; unknown-form-head rejection (NOT 241.10's scope; substrate behavior change)
7. Stone 241.10 probe < 8/8
8. Stone 241.1-241.9 probes regress; arc 237/238 probes regress
9. Clippy > 902
10. Adding `hint:` BACK alongside `remedies:` (HARD CUT violation)
11. **Vigilia divergent after 3 amend cycles** (escalate to orchestrator)
12. `feedback_no_semantic_abuse_of_option`: `Option<Vec<Remedy>>` instead of `Vec<Remedy>`
13. Heuristic retirement matching (D6 violation)
14. Eager `remedies_for` pre-compute (D10 violation)

## SCORE doc spec

Mirror `SCORE-STONE-241.9.md` with the vigilia section per `SCORE-STONE-241.1.fix.md`. Include:
- Header (Mode A/B; runtime; one-line summary; cascade size; vigilia status)
- Phase A scorecard (probe + lib + clippy + structural rows)
- Vigilia convergence section (per-spell L0/L1/L2 findings; amend cycles; final state)
- Schema migration audit (per-variant hint→remedies conversion)
- Wire-in site catalog (per-path migration)
- Final `Remedy`/`RemedyKind` shape (verbatim)
- Final `nearest_match` + `retirement_lookup` + `remedies_for` bodies (verbatim)
- Display format examples (single / multi / retirement)
- Honest deltas (anything surfaced)
- Convergence #18 inscription (Lisp condition-system) — provisional pending verification

## Post-strike

Return one-paragraph status: `src/remedy/` minted; schema upgraded; cascade depth; Stone 241.10 probe 8/8; vigilia convergence state; SCORE doc path; any surfaced gaps.

Phase 3 advances. One stone remaining (241.11 `define ⇒ defn` HARD CUT) before INSCRIPTION at 241.12. The bandaid-rip lands on a substrate that teaches with receipts. Strike clean.
