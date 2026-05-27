# SCORE — Stone S-B.1 — mint `:wat::core::recordtype` + `TypeDef::Record`

**Date:** 2026-05-26
**Status:** COMPLETE — 6/6 PASS (LOAD-BEARING); all regression guards green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -3` | 0 errors |
| 2 | **S-B.1 probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | arc 237.1 typeunion regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 5 | arc 237.5 conforms? regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | arc 237.6 is-predicate regression | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 7 | arc S-A hierarchy regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 8 | holon-rs untouched | (not touched) | STOP-5 not triggered |

---

## Final API shape

Matches BRIEF sketch exactly — no naming adjustments.

### New type

```rust
/// Stone S-B.1 — record class declaration. Minimal holder: name + parent.
/// Fields live in the macro's emitted accessors (S-B.2). NOT fed to
/// register_struct_methods — dedicated kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub parent: String,
}
```

### New TypeDef variant

```rust
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
    Union(UnionDef),    // Stone 237.1
    Record(RecordDef),  // Stone S-B.1
}
```

### New private functions in `src/types.rs`

| Function | Purpose |
|----------|---------|
| `parse_recordtype` | Parser: `(:wat::core::recordtype :Name :Parent)` → `TypeDef::Record` |

### Registration path (in `register_with_span` + `register_stdlib_with_span`)

After `TypeDef::Record` is inserted:
1. Verify `parent` is a known type via `self.types.contains_key(&parent)` → `MalformedDecl` if not.
2. Insert `TypeDef::Record` into the types map.
3. Call `self.register_subtype(&name, &parent)` — wires the `typesub` edge (S-A's method; cycle-checked there).

### New arms in `src/runtime.rs`

| Function | Arm added |
|----------|-----------|
| `register_type_predicates` | `TypeDef::Record(r) => &r.name` — synthesizes `is-<Name>?` ∀T |
| `conforms_check` (Path arm) | `Some(TypeDef::Record(_))` → `concrete_type_name_matches` (nominal exact, mirror Struct) |
| `typedef_to_signature_ast` | Early-return for Record (no type_params field) |
| `typedef_to_define_ast` | `TypeDef::Record(_) => ":wat::core::recordtype"` |

### Surface form

```wat
(:wat::core::recordtype :my::Circle :wat::Record)
(:wat::core::recordtype :my::Sphere :wat::holon::Record)
```

Two positional args after the head: class FQDN + parent type. No type params (minimal holder).

---

## Line counts

| File | Pre-stone | Post-stone | Net |
|------|-----------|------------|-----|
| `src/types.rs` | 3771 | 3900 | +129 |
| `src/runtime.rs` | 33357 | 33373 | +16 (+3 header/-0) |
| `src/closure_extract.rs` | 2475 | 2488 | +13 |
| `src/check.rs` | 21256 | 21256 | 0 (untouched) |

Total net: ~158 lines. Within the expected band for a new TypeDef variant + decl form + cascade.

---

## Cascade depth

**2 rounds.**

1. `src/types.rs` — adds `RecordDef` struct + `TypeDef::Record` variant + `TypeDef::name()` arm + `classify_type_decl` arm + `parse_type_decl` arm + `parse_recordtype` function + Record registration logic in `register_with_span` + `register_stdlib_with_span`. Compile reveals 4 non-exhaustive pattern errors.

2. `src/closure_extract.rs` + `src/runtime.rs` — mandatory match exhaustiveness fixes caused by new `TypeDef` variant. 2 locations each. Compile clean. `src/check.rs` required no cascade arm (no exhaustive `TypeDef` match in check.rs; Union arms use `if let` guards).

### Cascade sites (forced TypeDef exhaustiveness)

| File | Location | Function | Record arm added |
|------|----------|----------|-----------------|
| `src/closure_extract.rs` | `~1274` | `def_inner_typeexprs` | `TypeDef::Record(_) => vec![]` (no inner TypeExprs) |
| `src/closure_extract.rs` | `~2168` | `type_def_to_ast` | Reconstructs `(:wat::core::recordtype :Name :Parent)` |
| `src/runtime.rs` | `~13028` | `typedef_to_signature_ast` | Early-return `WatAST::List([name_kw])` (no type_params) |
| `src/runtime.rs` | `~13058` | `typedef_to_define_ast` | `TypeDef::Record(_) => ":wat::core::recordtype"` |

All four arms are minimal and correct; none add new logic beyond what the variant semantically requires.

---

## Honest deltas

### `check.rs` untouched

`src/check.rs` required no cascade arm. Its TypeDef matches use `if let` guards (e.g. `if let TypeDef::Union(u) = ...`) rather than exhaustive `match`, so no Record arm was forced. This is a difference from the 237.1 cascade where check.rs needed unify extension arms. For S-B.1 there is no new unification logic.

### `typedef_to_signature_ast` early-return pattern

`RecordDef` has no `type_params` field (BRIEF-correct: minimal holder). The `typedef_to_signature_ast` function destructures `(base, type_params)` from all other variants, but Record has no `type_params` to borrow. The cleanest solution is an early-return for the Record arm before the destructuring match — avoids adding a `type_params` field to `RecordDef` (which would be a lie) and avoids a `static EMPTY` in a match arm.

### `:my::is-Circle?` auto-synthesis (THE asymmetry-killer)

`register_type_predicates` synthesizes `:my::is-Circle?` and `:my::is-Sphere?` automatically when these types are registered via `recordtype`. The probe confirms: `(:my::is-Circle? 42)` → `false` (∀T, no type error). Pre-stone, the macro's hand-emitted predicate narrowed `[v <- :wat::Record]` and type-errored on non-record values. The asymmetry is dead.

### Nominal `conforms?` Record arm — unexercised in B.1

The `conforms_check` Record arm (`Some(TypeDef::Record(_)) => concrete_type_name_matches`) is correct and in place, but probe B.1 does not exercise its TRUE-path (requires a real `:my::Circle` value from the macro — S-B.2 territory). The FALSE-path (via `is-Circle?` probe 2) is covered. The arm is the obvious-correct mirror of the Struct arm; its TRUE-path is proven end-to-end in S-B.2.

### No `TypeError::UnknownType` added

The BRIEF said "clean error" for unknown parent. `TypeError::MalformedDecl { head: "recordtype", reason: "parent '...' is not a known type" }` is used — already exists, surfaces clearly, does not require a new variant. Probe 6 only checks `is_err()`.

### Parent validation scope

Parent validity is checked via `self.types.contains_key(&parent)`. This is correct because `:wat::Record` and `:wat::holon::Record` are both registered as `TypeDef::Struct` in `register_builtin_types` (confirmed at types.rs:1370, 1388). Any type that has been declared (builtin or user) will pass this gate. Types that exist only in the subtype_edges (e.g. bare hierarchy nodes with no TypeDef) cannot be used as parents — a reasonable constraint.

---

## Working tree on return

```
 M src/closure_extract.rs
 M src/runtime.rs
 M src/types.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-B1.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit — orchestrator commits after scoring.
