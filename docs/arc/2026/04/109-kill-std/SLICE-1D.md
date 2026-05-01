# Arc 109 Slice 1d — Mint `:wat::core::unit`; retire `:()` as a type annotation

**Compaction-amnesia anchor.** Read this first if you're picking up
slice 1d mid-flight.

## What this slice does

Mint `:wat::core::unit` as the canonical type name for the unit type
(the type with one inhabitant, `()`). Retire `:()` as a TYPE
annotation in user code and substrate stdlib. The empty-tuple LITERAL
VALUE `()` stays untouched — only the type-position spelling renames.

Per arc 109 § A and INVENTORY.md:

> The unit TYPE moves under `:wat::core::*` (the same home as
> `i64`/`f64`/`bool`/`String`/`u8`). Pre-arc-109 it's spelled `:()`
> (zero-element structural form, parses as empty tuple); slice 1d
> retires that spelling as a type annotation. The empty-tuple
> LITERAL VALUE `()` stays untouched — only the TYPE annotation
> `:()` is renamed.

User direction (2026-04-30): *"i want us to remove () as the type in
109 - it needs to be swapped to :wat::core::unit"*.

## The protocol

Pattern 3 from `docs/SUBSTRATE-AS-TEACHER.md` — same as slice 1c.
Dedicated `CheckError` variant + walker; substrate's diagnostic
stream IS the migration brief; sonnet sweeps from it.

**Distinct from slice 1c**: the unit type is a `TypeExpr::Tuple(vec![])`
(empty tuple), NOT a `TypeExpr::Path`. The walker detects this shape
specifically. FQDN form `:wat::core::unit` parses to
`TypeExpr::Path(":wat::core::unit")` (a typealias the substrate
unifies with the empty tuple), preserving the distinction the walker
needs.

## What to ship

### Substrate (Rust)

1. **Mint `:wat::core::unit` typealias** in `src/types.rs::register_builtin_types`:
   ```rust
   env.register_builtin(TypeDef::Alias(AliasDef {
       name: ":wat::core::unit".into(),
       type_params: vec![],
       expr: TypeExpr::Tuple(vec![]),
   }));
   ```
   Alias resolution unifies `:wat::core::unit` with `:()` at the type
   level. Both forms type-check; user-source distinguishes them.

2. **Add `CheckError::BareLegacyUnitType { span }`** variant in
   `src/check.rs`. Display IS the migration brief naming the rule and
   the canonical FQDN form. `diagnostic()` arm produces structured
   record consumable via `--check-output edn|json`.

3. **Extend the walker.** `walk_type_for_bare` in `src/check.rs`
   already recurses on Tuple's elements; add a guard at the entry of
   the Tuple arm: if `elements.is_empty()`, emit
   `BareLegacyUnitType` with the keyword's span. Don't recurse on
   the (empty) elements list.

   Equivalent shape to slice 1c's Path-based detection. Same
   audit-parse path (`parse_type_expr_audit`), same span propagation,
   same per-keyword span granularity.

### Verification

Probe coverage:
- `(name :())` → fires (outer bare unit)
- `(name :wat::core::unit)` → silent (FQDN)
- `(define (:foo (x :i64) -> :())` ... → fires (return type)
- `(define (:foo (x :i64) -> :wat::core::unit)` ... → silent
- `:Result<(),E>` → fires (inner bare unit)
- `:Result<wat::core::unit,E>` → silent (inner FQDN)
- `:fn(i64)->()` → fires (fn ret bare unit)
- `:Vec<my::pkg::Empty>` → silent (user struct, distinct shape)

False-positive resistance: empty tuples never appear in valid wat
*as values* (the empty list `()` is a list literal, not a type
expression), so detecting `Tuple(vec![])` purely in type-keyword
positions is unambiguous.

### Sweep order

Same four tiers as slice 1c. Substrate stdlib first; binary must
boot clean before anything downstream.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`. Every
   `(define ... -> :())` and `:fn(...)->()` retires.
2. **Lib + early integration tests** — embedded wat strings in
   `src/check.rs::tests`, `src/runtime.rs::tests`, `tests/wat_*.rs`
   that show up as `<test>:N:M` source.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

Verification gate after each tier: `cargo test --release --workspace`
shows zero `BareLegacyUnitType` errors before next tier.

## What does NOT change

- **The empty-list value `()`** — that's a list literal, parses as
  `WatAST::List(vec![], _)`, not a type expression. The walker
  doesn't flag values, only TypeExpr.
- **Internal Rust `TypeExpr::Tuple(vec![])` literals** — the
  canonical internal representation of unit. They stay; only the
  source-form `:()` in `.wat` files retires.
- **`Process<I,O>` / `Thread<I,O>` typed `O = ()`** — when user code
  writes `:Process<i64,()>` the inner `()` is a bare unit and SHOULD
  fire. After sweep: `:Process<wat::core::i64,wat::core::unit>`.
- **The `:None` keyword value** — distinct from `:()`; not a type.
  Slice 1d doesn't touch it.

## Closure (slice 1d step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § A — strike the `:()` row; mark unit type
   as ✓ shipped slice 1d.
2. Update `J-PIPELINE.md` — slice 1d done.
3. Update `SLICE-1D.md` — flip from anchor to durable shipped-record.
4. Add row to `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`.
5. Update `WAT-CHEATSHEET.md` if § 3 (FQDN namespace) needs the unit
   row added.
6. Optional: retire `:()` at the parser level once all consumer code
   is swept (TypeError::BareLegacyUnitType so future user code can't
   reintroduce). Walker stays as the structural rule; parser-level
   rejection is belt-and-suspenders.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` § "Three migration patterns" — Pattern 3.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § A — the unit row.
- `docs/arc/2026/04/109-kill-std/SLICE-1C.md` — the precedent slice;
  the walker shape this slice extends.
- `src/check.rs::validate_bare_legacy_primitives` — the existing
  walker; slice 1d extends its Tuple arm.
- `src/types.rs::parse_type_expr_audit` — the data-driven parse the
  walker consumes.
