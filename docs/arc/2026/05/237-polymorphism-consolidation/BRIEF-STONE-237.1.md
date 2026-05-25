# BRIEF — Stone 237.1 — mint `:wat::core::typeunion` substrate primitive

**Status:** READY TO SPAWN.

## What to do

Mint `:wat::core::typeunion` as a substrate type-declaration primitive — a NEW type-level grouping concept for "named bounded set of types." Add `TypeDef::Union(UnionDef { name, type_params, members })` parallel to existing `TypeDef::Alias`. Add 4 new `TypeError` variants for validation (`CyclicUnion` / `EmptyUnion` / `SingleMemberUnion` / `InvalidUnionMember`). Extend the unifier (`fn unify` in `src/check.rs`) with **bounded-existential typing** — typeunion unifies against any member; symmetric; member-set intersection for Union/Union.

NO defclause work (Stone 237.2). NO arc 146 Dispatch migration (Stone 237.6). NO arithmetic special-case retirement (Stone 237.7). NO AnyBanned message update (Stone 237.8). 237.1 ships ONLY the typeunion primitive + unifier extension.

TWO files modified: `src/types.rs` + `src/check.rs`. ZERO new files (probe already on disk per FM 2-bis).

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.1.md` — sub-DESIGN with all locked decisions, substrate work breakdown, trap-door audit
2. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN.md` — arc umbrella (especially "Substrate diagnosis findings" section)
3. `tests/probe_arc237_stone1_typeunion_substrate.rs` — **LOAD-BEARING** 14 probes; ALL must PASS
4. `src/types.rs:161` `AliasDef` struct — registration model to mirror exactly
5. `src/types.rs:1406` `CyclicAlias` error pattern — `CyclicUnion` mirrors this
6. `src/types.rs:2629` `expand_alias` resolution pattern — typeunion resolution mirrors this  
7. `src/check.rs:13953` `fn unify` — insertion point for typeunion arms (bounded existential)
8. `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — Rust probe pattern template (parallel substrate-mint shape)
9. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — substrate-mint stone template (polymorphic primitive)

## Implementation sketch

### Types (`src/types.rs`)

```rust
// Add new struct (parallels AliasDef)
pub struct UnionDef {
    pub name: String,
    pub type_params: Vec<String>,  // empty in arc 237; reserved for future parametric typeunions
    pub members: Vec<TypeExpr>,
}

// Add variant to existing TypeDef enum
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
    Union(UnionDef),   // NEW
}

// Add 4 variants to TypeError enum
pub enum TypeError {
    // ... existing variants ...
    CyclicUnion { name: String, span: Span },
    EmptyUnion { name: String, span: Span },
    SingleMemberUnion { name: String, span: Span },  // diagnostic recommends typealias
    InvalidUnionMember { union_name: String, member_form: String, reason: String, span: Span },
}
```

### Parser (`src/types.rs`)

Surface form (Vector literal for members per `feedback_clojure_not_scheme`):

```wat
(:wat::core::typeunion :Name [:T1 :T2 :T3])
```

- Add `parse_typeunion` mirroring `parse_typealias` (around `src/types.rs:1674`)
- Wire into the decl-form dispatch table where `typealias` is handled
- Members come from the Vector literal `[...]` payload; reject if not a Vector literal
- Empty Vector → `EmptyUnion`; single-element Vector → `SingleMemberUnion`; member-shape validation per the table below

### Registration (`src/types.rs`)

`register_union` parallel to `register_alias` — walks the member graph; checks for cycles through any registered `TypeDef::Union` name; calls `register` to install in TypeEnv.

| Member shape | Verdict |
|---|---|
| `TypeExpr::Path` (concrete OR another typeunion OR alias) | ACCEPT |
| `TypeExpr::Parametric` (e.g., `Vector<i64>`) | ACCEPT |
| `TypeExpr::Tuple` | ACCEPT |
| `TypeExpr::Fn` | REJECT `InvalidUnionMember` |
| `TypeExpr::Var` | REJECT `InvalidUnionMember` |

Cycle detection: walk member graph; any cycle through registered typeunion names → `CyclicUnion` error.

### Unifier extension (`src/check.rs:13953`)

Add typeunion arms to `unify`. Conceptual rules:

- `unify(:Numeric, :i64)` where `:Numeric` resolves to `TypeDef::Union { members: [:i64, :f64] }` → SUCCEED; resolved type = `:i64` (bound to subst)
- `unify(:Numeric, :String)` → FAIL (`UnifyError`)
- `unify(:Numeric, :Numeric)` → SUCCEED; resolved type = `:Numeric` (identity)
- `unify(:U1, :U2)` where both resolve to typeunions → intersect member sets; FAIL if empty intersection
- `unify(:i64, :Numeric)` MUST succeed symmetrically (mirror of first case)

**Insertion point:** the `reduce` function at the start of `unify` (around `src/check.rs:13953`). When `reduce` encounters a `Path` resolving via `TypeEnv` lookup to `TypeDef::Union`, surface the union-reference for the unify-on-children logic to special-case via new match arms.

**Substitution semantics:** when `unify(Union, Member)` succeeds, record the SPECIFIC matched member in `subst` so downstream `unify(union, OtherMember)` correctly FAILS (already bound). Recursive typeunion expansion walks the graph; cycle-check at registration bounds the walk.

### Doctrine compliance

- typeunion is BOUNDED (explicit members; finite); preserves `:Any` ban (058-030)
- typeunion is type-only (no runtime artifact; no new Value variant)
- DEPARTURE from AnyBanned's existing "named enum for closed heterogeneous sets" recommendation is justified by arithmetic UX (Stone 237.7 territory; not 237.1's concern)

## Discipline

- Modify ONLY `src/types.rs` + `src/check.rs`
- DO NOT touch: any existing function except `unify` + `reduce` (parser sites + registration are NEW additions, not modifications)
- DO NOT touch: holon-rs (STOP-4)
- DO NOT touch: any other file in src/ or tests/
- DO NOT commit (orchestrator commits)
- DO NOT update `AnyBanned` error message (that's Stone 237.8)
- DO NOT add `defclause` or any other primitive (that's Stone 237.2+)
- DO NOT mint runtime Value variant (typeunion is type-only)
- DO NOT short-circuit any of the 14 probe contracts

## STOP triggers (REJECTION — NOT permission to defer)

1. **Unexpected compile errors** that don't trace to a probe-named contract
2. **Lib baseline drops below 827** PASS
3. **Clippy exceeds 54** warnings
4. **180 min elapsed** (STOP-3 below Mode A upper)
5. **240 min elapsed** (STOP-4 hard kill; surfaces as partial-state-grading per `feedback_partial_state_grading`)
6. **holon-rs touched** (STOP-5)
7. **Files outside src/types.rs + src/check.rs touched**
8. **Probe doesn't 14/14 PASS** — partial is acceptable mid-flight but ship gate is 14/14
9. **Arc 234 or arc 236 regression** (probe failures in those arcs indicate substrate damage)
10. **Recursive typeunion expansion infinite-loops** — cycle check at registration MUST prevent this; if loop appears, fix at registration not at unify

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.1.md` (NEW). 12-row scorecard verbatim + final API shape (any naming adjustments from sketch) + line counts per file + cascade depth + honest deltas.

## FM 2-bis evidence

The probe at `tests/probe_arc237_stone1_typeunion_substrate.rs` (already committed at `63657d95`) IS the design substrate. 14 contracts test:
- TypeDef::Union registration + read-back (Probes 1, 7, 8, 9)
- Cycle detection (Probe 2, 10)
- Validation errors (Probes 3, 4, 5, 6)
- wat-source integration via `startup_from_source` (Probes 11, 12, 13, 14)

Pre-stone: 10 compile errors naming exactly the substrate pieces this stone mints. Post-stone: 14/14 PASS.

## Calibration anchor

Stone 236.0 (parallel substrate-mint pattern; mint CheckResult<T> + Rust probe) shipped at ~25 min in band. Stone 234.0 (mint `:wat::core::type` polymorphic primitive) shipped at ~38 min in band. Stone 237.1 is HEAVIER than either due to the unifier extension (bounded existential typing is new substrate machinery, not just registration plumbing). Target band: 60-120 min Mode A; 240 STOP.

Per `feedback_stone_briefs_cite_prior_score`: sonnet may mirror the structural shape of those SCOREs for its 237.1 SCORE; the honest-deltas + cascade-depth + line-count sections inherit the discipline.
