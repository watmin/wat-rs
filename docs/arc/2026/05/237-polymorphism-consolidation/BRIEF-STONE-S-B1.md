# BRIEF — Stone S-B.1 — `:wat::core::recordtype` + `TypeDef::Record` (records become types)

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Mint the substrate type-declaration form `:wat::core::recordtype` that makes a
record class a real `TypeDef::Record`, so it inherits the type system's uniform
services autonomously: ∀T `is-<Name>?` synthesis + `typesub` hierarchy membership.

```
(:wat::core::recordtype :my::Circle :wat::Record)   ; declares Circle as a record type,
                                                     ; parent :wat::Record (base flavor)
(:wat::core::recordtype :my::Sphere :wat::holon::Record)  ; holon flavor
```

Make `tests/probe_arc237_sB1_recordtype.rs` go **6/6**. It is committed and pins
the contract.

This is **NOT new-paradigm territory** — it mirrors the `:wat::core::typeunion`
mint (Stone 237.1) almost exactly (new `TypeDef` variant + decl-form parse +
register + cascade), plus one addition: registration also wires the `typesub`
edge via S-A's `register_subtype`.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-B1-recordtype.md`
   — the sub-DESIGN. **Read the three-tier doctrine box at top: `conforms?` stays
   tier-3 (NO parent-walk); B.1's conforms_check Record arm is NOMINAL-EXACT only.**
2. `tests/probe_arc237_sB1_recordtype.rs` — **LOAD-BEARING** 6 contracts. Pre-stone:
   fails (recordtype unknown / `TypeDef::Record` absent). Post-stone: 6/6.
3. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.1.md` — the
   comparable mint (typeunion); **mirror its SCORE structure.**
4. `src/types.rs`:
   - `enum TypeDef` (~185) + `UnionDef` (~177) — add `TypeDef::Record(RecordDef)`;
     `RecordDef { name: String, parent: String }`.
   - `TypeDef::name()` (~196) — add the Record arm.
   - decl-head dispatch `classify_type_decl` (~1803 region; the fn that maps a head
     keyword like `"typeunion"` to its parser) — recognize `recordtype`.
   - `parse_typeunion` (~2145) — the parser shape to MIRROR; write `parse_recordtype`
     → `RecordDef { name, parent }`.
   - `register_with_span` (~263) + `register_stdlib_with_span` (~313) — the Union
     arms (~286, ~330) call `validate_union_members` + `check_union_no_cycle`. For
     Record: after inserting the `TypeDef::Record`, ALSO call
     `self.register_subtype(&name, &parent)` (S-A's method, ~types.rs:372) — wiring
     the edge IS part of registering the record type. Reject if the parent is
     unknown / the edge would cycle (register_subtype already cycle-checks; ALSO
     verify the parent resolves to a known type → else a clean error).
   - `register_builtin_types` (~1267 `:wat::Record`, ~1328 `:wat::holon::Record`
     root edge) — precedent for opaque type registration; do NOT change.
5. `src/runtime.rs`:
   - `register_type_predicates` (~3198; arms ~3208-3212) — add
     `TypeDef::Record(r) => &r.name`. This synthesizes `is-<Name>?` ∀T. THE
     asymmetry-killer.
   - `conforms_check` (~16130; Path arm Struct/Enum/Newtype ~16157) — add
     `Some(TypeDef::Record(_))` → `concrete_type_name_matches` (NOMINAL identity,
     exactly like the Struct arm). NO hierarchy walk here (that's tier-2 `subtype-of?`,
     a different stone).
6. **Cascade:** `cargo build` after adding the variant names every exhaustive
   `match` over `TypeDef` that needs a Record arm (the Union precedent cascaded to
   ~4 sites — expect a similar handful across `types.rs`, `runtime.rs`, `check.rs`,
   possibly `closure_extract.rs`). Add the Record arm mirroring the nearest nominal
   sibling (usually Struct). Iterate build→fix→build until clean. NOT a crisis.

## Implementation sketch

```rust
// src/types.rs
pub struct RecordDef { pub name: String, pub parent: String }
pub enum TypeDef { Struct(..), Enum(..), Newtype(..), Alias(..), Union(..), Record(RecordDef) }
// TypeDef::name(): TypeDef::Record(r) => &r.name

fn parse_recordtype(args: Vec<WatAST>, span: Span) -> Result<TypeDef, TypeError> {
    // (:wat::core::recordtype :Name :Parent) — exactly 2 keyword args (mirror parse_typeunion's
    // arg-count + name-keyword parsing; parent is a plain type keyword/path).
    // → TypeDef::Record(RecordDef { name, parent })
}

// register_with_span, after inserting TypeDef::Record:
//   - verify `parent` is a known type (env.contains OR builtin) → else clean error.
//   - self.register_subtype(&name, &parent)?;   // wires the typesub edge (S-A)
```

```rust
// src/runtime.rs register_type_predicates: add arm
TypeDef::Record(r) => &r.name,
// conforms_check Path arm: add
Some(TypeDef::Record(_)) => Ok(concrete_type_name_matches(value, name)),
```

## Discipline

- Modify `src/types.rs` + `src/runtime.rs` (+ `src/check.rs` / `src/closure_extract.rs`
  ONLY for forced `TypeDef`-exhaustiveness cascade arms).
- NO Record.wat / defrecord changes (that's S-B.2).
- NO new `Value` variant. NO `subtype-of?` / `exact-type?` (separate stones).
- NO hierarchy walk in `conforms?` (tier-3 stays nominal/union/structural).
- NO holon-rs (STOP-5).
- `TypeDef::Record` must NOT be fed to `register_struct_methods` (it would emit a
  spurious `:my::Circle/new` + accessors). It is a dedicated kind; only Struct goes
  to that pass — verify register_struct_methods matches `TypeDef::Struct` only.

## STOP triggers (REJECTION — not permission to defer)

1. Compile errors not traced to a probe contract or the expected `TypeDef` cascade.
2. Lib baseline drops below 827.
3. 100 min elapsed (STOP-3); 130 min (STOP-4 hard kill).
4. holon-rs touched (STOP-5).
5. Files touched outside `src/types.rs` / `src/runtime.rs` / `src/check.rs` /
   `src/closure_extract.rs` (the last two ONLY for cascade arms). Record.wat is OUT.
6. Probe doesn't reach 6/6.
7. Any arc-237 predecessor probe regresses (237.1 typeunion / 237.5 conforms? /
   237.6 is-predicate / S-A hierarchy).
8. You find yourself: registering the record class as `TypeDef::Struct`; adding a
   hierarchy walk to `conforms?`; feeding Record to `register_struct_methods`;
   touching Record.wat — STOP, none is in scope.

## FM 2-bis evidence

`tests/probe_arc237_sB1_recordtype.rs` (committed) — 6 contracts. Pre-stone: fails
(`recordtype` unknown form / `TypeDef::Record` absent). Post-stone: 6/6.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-B1.md` (NEW). Mirror
SCORE-STONE-237.1: scorecard (compile clean; **S-B.1 probe 6/6 LOAD-BEARING**; lib
827; 237.1/237.5/237.6/S-A regression guards; holon-rs untouched) → Final API shape
→ Line counts → Cascade depth (list the forced `TypeDef` sites) → Honest deltas
(incl. the expected `:my::is-Circle?` auto-synthesis + the nominal conforms? Record
arm being unexercised-in-B1) → Working tree. DO NOT commit (orchestrator commits).

## Calibration

New `TypeDef` variant + decl form (parse+register, mirror typeunion) + edge-wiring
+ 2 synthesis/conformance arms + bounded cascade. Comparable to 237.1. **Target
band: 45–75 min Mode A; 100 STOP-3; 130 STOP-4. Cascade: 2–3 rounds, handful of
forced sites, 0 new files.** Per `feedback_stone_briefs_cite_prior_score`:
SCORE-STONE-237.1 is the shape.
