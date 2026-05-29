# BRIEF — Stone 242.1 — bare `nil` + `:wat::core::Char` HARD CUT + doctrine inscription

You are sonnet. Stone 242.1 is the substantive stone of arc 242 (`lexeme-role-doctrine`). Arc 242 is a spawn-block child of arc 241 (paused at Stone 241.12 STRIKE-READY); arc 241 resumes after arc 242 closes.

## The two doctrines this stone inscribes

**Doctrine 1 — bare lexeme = value; keyword lexeme (`:wat::core::*`) = type.**

**Doctrine 2 — scalar types lowercase; non-scalar/container types PascalCase.**

Read `docs/arc/2026/05/242-lexeme-role-doctrine/DESIGN.md` and `DESIGN-STONE-242.1.md` for full doctrine framing + examples.

## CRITICAL discipline (pre-authorized per `feedback_hard_cut_admits_no_bypasses`)

**HARD CUT IS TOTAL.** `:wat::core::Char` dies EVERYWHERE in the substrate. No privileged paths. No substrate-internal bypasses.

When you encounter substrate-internal `:wat::core::Char` use: it migrates to `:wat::core::char`. The ONLY acceptable references are:
- HARD-CUT-rejection error text at `src/check.rs`
- `RETIREMENT_TABLE` entry
- Historical comments
- Probe test source testing the HARD CUT
- Retirement_lookup test fixtures

If you classify a use as "privileged path" — STOP. That framing is a doctrine violation. The use migrates. Stone 241.11.fix round 2 was killed for this framing; do not repeat.

## What to do

### S1 — Audit bare `nil` lexer state

The probe shows bare `nil` may already work (C01 passes at HEAD). Investigate:
```
grep -n "\"nil\"\|nil_value\|parse_nil" src/parser.rs src/edn_shim.rs
```

If bare `nil` parses as a SYMBOL (per edn_shim.rs:1802 comment), determine if it's already treated as primitive nil value in context OR if the substrate coerces in type-check.

If lexer work is needed: make bare `nil` a primitive nil literal (same shape as bare `true`/`false`/numeric literals). If lexer work is NOT needed (already operational): document the finding in SCORE.

### S2 — Migrate `:wat::core::nil` VALUE-position uses → bare `nil`

Audit:
```
grep -rn ":wat::core::nil" src/ tests/ wat/
```

Classify each per Doctrine 1:
- **Type position** (signature returns, parameter types, type annotations): PRESERVE (lowercase scalar type stays)
- **Value position** (expression returns, dispatch values, argument values): MIGRATE to bare `nil`

When ambiguous: PREFER PRESERVE (don't break working code).

Type-position indicators:
- After `->` in signatures
- After `<-` in argspec
- In type-annotation positions

Value-position indicators:
- Body expressions
- Return values
- Argument values in function calls

### S3 — HARD CUT `:wat::core::Char` → `:wat::core::char`

Per Doctrine 2 (scalar types lowercase). Mirror Stone 241.8/9/11 pattern:

**S3.1 — Append RETIREMENT_TABLE entry:**

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 — defn replaces define
    (":wat::core::define",            ":wat::core::defn"),
    // Stone 242.1 — char (lowercase) replaces Char (per Doctrine 2; scalar types lowercase)
    (":wat::core::Char",              ":wat::core::char"),
];
```

**S3.2 — Mint check.rs HARD-CUT-rejection arm** mirroring existing struct/enum/define arms; populate `remedies: remedies_for(k, std::iter::empty())`.

**S3.3 — Mint `:wat::core::char` as the live type** at appropriate substrate dispatch (parser/types/check).

**S3.4 — Cascade migrate** all active `:wat::core::Char` references to `:wat::core::char`.

### S4 — Reflection emitters

Audit:
```
grep -n "Keyword.*wat::core::Char\|Keyword.*wat::core::nil" src/runtime.rs
```

For each AST construction:
- `:wat::core::nil` emissions: judge value vs type position; preserve type-position emissions; migrate value-position emissions to bare `nil` keyword (or whatever the lexer expects post-S1)
- `:wat::core::Char` emissions: migrate to `:wat::core::char`

### S5 — Doctrine inscription

Two files to inscribe:

1. **`docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`** — append substantive realization entry naming arc 242's contribution (per `feedback_sonnet_no_realization_voice`: this is orchestrator-direct content — but you can DRAFT it during SCORE writing, orchestrator reviews + finalizes during commit)

2. **`~/.claude/projects/-home-watmin-work-holon/memory/project_lexeme_role_doctrine.md`** — NEW memory:
   ```
   ---
   name: lexeme-role-doctrine
   description: ...
   metadata:
     type: project
   ---

   Doctrine 1 + Doctrine 2 verbatim with examples.
   Inscribed at arc 242 close.
   How to apply: future case-audit arcs consume this rule.
   ```

3. Update `MEMORY.md` index with the new memory entry.

### S6 — Pre-INSCRIPTION grep (light version for this stone)

After all migrations, verify:
```
grep -rn ":wat::core::Char" src/ tests/ wat/
```

Goal: 0 active uses (only HARD-CUT-rejection text + retirement-table entry + historical comments + probe).

```
grep -rn ":wat::core::nil" src/ tests/ wat/
```

Inspect distribution: type-position (preserve) vs value-position (should be 0 post-migration).

### S7 — Probe verification

`tests/probe_arc242_stone1_lexeme_role.rs` (already committed STRIKE-READY). 4 contracts; pre-stone 3/4 PASS (C03 disconfirms cleanly); post-stone 4/4 PASS.

## Discipline

- HARD CUT total for `:wat::core::Char` — no internal bypasses (per `feedback_hard_cut_admits_no_bypasses`)
- `:wat::core::nil` TYPE-position uses PRESERVED (Doctrine 1: lowercase scalar type stays)
- Bare `nil` is a VALUE per Doctrine 1
- src/argspec/*, src/lib.rs UNCHANGED
- src/remedy/retirement.rs MODIFIED (single-line append for 5th entry)
- Stone 241.x probes preserved; arc 237/238 probes preserved
- holon-rs NEVER touched
- Auto-fixer crate (if minted) must be EPHEMERAL — DELETED before commit per Stone 241.10/241.11 precedent
- Stone 242.1 probe 4/4 PASS

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/BRIEF-STONE-242.1.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/DESIGN.md` — arc-level + doctrine framing
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/DESIGN-STONE-242.1.md` — D1-D8 + T1-T7 + STOP
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.11.md` — cascade pattern + auto-fixer ephemeral discipline
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — retirement-table append discipline + remedy infrastructure
8. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — current RETIREMENT_TABLE (4 entries; you append 5th)
9. `/home/watmin/work/holon/wat-rs/src/check.rs` — find existing HARD-CUT-rejection arms; mirror for `:wat::core::Char`
10. `/home/watmin/work/holon/wat-rs/src/parser.rs` + `/home/watmin/work/holon/wat-rs/src/edn_shim.rs` — bare nil lexer state audit
11. `/home/watmin/work/holon/wat-rs/tests/probe_arc242_stone1_lexeme_role.rs` — 4-contract probe (3/4 PASS at HEAD; C03 disconfirms)

## Cadence

1. Baseline: `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0)
2. Probe: `cargo test --release --test probe_arc242_stone1_lexeme_role 2>&1 | tail -3` (expect 3/4)
3. **S1**: audit bare nil lexer; implement if needed
4. **S2**: audit `:wat::core::nil` uses; migrate value-position to bare nil
5. **S3**: HARD CUT `:wat::core::Char` (append RETIREMENT_TABLE; mint check.rs arm; mint `:wat::core::char`; cascade migrate)
6. **S4**: reflection emitter audit + migration
7. **S5**: doctrine inscription (memory + INTERSTITIAL draft)
8. **S6**: pre-Stone-cleanup grep verification
9. **S7**: probe 4/4 PASS verification
10. Final: lib ≥ 890; workspace build clean; clippy ≤ 902
11. Write `SCORE-STONE-242.1.md` at `docs/arc/2026/05/242-lexeme-role-doctrine/` (NOT repo root)
12. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers

Per DESIGN-STONE-242.1.md § STOP triggers. Key:

1. Lib < 890
2. 180 min elapsed
3. holon-rs touched
4. `:wat::core::nil` TYPE-position uses incorrectly migrated (broken signature types)
5. `:wat::core::Char` use classified as "privileged path" without migration
6. Auto-fixer crate survives commit
7. Stone 242.1 probe < 4/4
8. Stone 241.x or arc 237/238 probes regress
9. Clippy > 902
10. Doctrine inscription incomplete (no memory + no INTERSTITIAL draft)

## SCORE doc spec

Mirror SCORE-STONE-241.11.md. Include:
- Header (Mode A; runtime; cascade size; auto-fixer? deleted?; doctrine inscription confirmed?)
- Phase A scorecard
- Migration cascade audit (per-file count; value vs type position distribution for `:wat::core::nil`)
- Final RETIREMENT_TABLE (verbatim; 5 entries)
- HARD CUT arm verbatim
- Bare nil lexer state (already operational? added? findings?)
- Doctrine inscription verification (memory file exists; MEMORY.md updated; INTERSTITIAL draft prepared)
- Honest deltas
- NO Vigilia section (D6 — no namespaced home)

## Post-strike

Return one paragraph:
- Bare nil lexer state (was operational? added support?)
- `:wat::core::nil` cascade (value migrated vs type preserved counts)
- `:wat::core::Char` HARD CUT + cascade depth
- Reflection emitters migrated
- Pre-stone grep verification result
- Stone 242.1 probe 4/4 status
- Doctrine inscription status (memory + INTERSTITIAL)
- Auto-fixer status (built? used? DELETED?)
- Baselines
- SCORE doc path (at arc dir, NOT repo root)

Arc 242 closes with Stone 242.2 INSCRIPTION (orchestrator-direct paperwork). Arc 241 resumes after that. Strike clean.
