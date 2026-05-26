# SCORE — Stone 237.6 — auto-mint `is-<Name>?` (named convenience over conforms?)

**Date:** 2026-05-25
**Status:** COMPLETE — 10/10 probe PASS. All scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **is-predicate probe 10/10 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Stone 237.5 regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 5 | Stone 237.5.fix regression | `cargo test --release --test probe_arc237_stone5fix_nominal 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 8 | Stone 237.2 regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 9 | Stone 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 10 | Stone 237.4 regression | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| tail -3` | `10 passed; 0 failed` |

---

## Predicate-naming rule

Given a type FQDN (e.g. `my::Shape`, `ns::sub::Foo`):

1. Strip leading `:` if present.
2. Split on `::`.
3. Take the last segment as `base`.
4. Build `:<prefix>::is-<base>?` where `<prefix>` is all segments except the last joined by `::`.
5. If no prefix (bare name), emit `:is-<base>?`.

Examples:

| FQDN | Predicate name |
|------|----------------|
| `my::Shape` | `:my::is-Shape?` |
| `my::Circle` | `:my::is-Circle?` |
| `my::Color` | `:my::is-Color?` |
| `my::Price` | `:my::is-Price?` |
| `my::Point` | `:my::is-Point?` |
| `ns::sub::Foo` | `:ns::sub::is-Foo?` |

Mirrors `Record.wat`'s naming (split/take/concat with `"is-"` + `"?"`).

---

## is-<Name>? bodies — all compose conforms?

Every `is-<Name>?` body — both TypeEnv-born and Record.wat-born — now resolves via the one mechanism:

| Source | Body before this stone | Body after this stone |
|--------|------------------------|----------------------|
| TypeEnv forms (struct/enum/newtype/union) | did not exist | `(:wat::core::conforms? v :<FQDN>)` |
| Record.wat macro | `(:wat::core::= (:wat::core::type v) "fqdn")` | `(:wat::core::conforms? v ~fqdn)` |

The `Record.wat` change replaces the standalone `(= (type v) "fqdn")` re-computation with `(conforms? v :FQDN)`. This is the one-way unify the DESIGN called "the actual one-way smell."

---

## New public function (src/runtime.rs)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `register_type_predicates` | `(types: &TypeEnv, sym: &mut SymbolTable) -> Result<(), RuntimeError>` | Walk every non-Alias TypeDef; synthesize `:<ns>::is-<Name>?` Function with body `(conforms? v :<FQDN>)` and `type_params: ["T"]` (∀T param). |

### Type param discipline

The predicate's `param_types: vec![TypeExpr::Path("T".into())]` with `type_params: vec!["T".into()]` — the checker's `instantiate` generates a fresh `TypeExpr::Var` for `T` at each call site, making the predicate accept any value without an `Any` escape hatch. Same pattern used by parametric struct/enum synthesized functions.

---

## Line counts

| File | Post-stone lines | Net added |
|------|-----------------|-----------|
| `src/runtime.rs` | 33,261 | +83 (new `register_type_predicates` fn + doc comment) |
| `src/freeze.rs` | 2,033 | +8 (call + comment for step 6.9) |
| `wat/Record.wat` | 232 | -2 (predicate body: 3 lines → 1 line) |

Total net: +89 lines.

---

## Honest deltas

### Files outside scope

None. Exactly `src/runtime.rs` + `src/freeze.rs` + `wat/Record.wat` touched. No `check.rs` changes needed — `conforms?` already has its own special-case inference arm (arc 237 Stone 237.5); the synthesized predicates delegate to it at eval time. No new `Value` variant. No `holon-rs` touch.

### Record.wat predicate signature unchanged

The `defn` signature `[v <- :wat::Record] -> :wat::core::bool` is unchanged; only the body was switched. The Record.wat predicate still accepts `:wat::Record`-typed values at check time (type-narrowed to the macro context). The TypeEnv-born predicates accept ∀T (broader, correct for the general case where the value's static type may be opaque).

### Union contracts (probe_07/08/09) — the payload

`probe_07` and `probe_08` test `:my::is-Shape?` on Circle and Square instances respectively. These pass because `conforms?` walks the union's member list and finds the instance's declared type (`my::Circle`, `my::Square`) among `my::Shape`'s members. The old `(= (type v) "Shape")` body could never have passed these — the instance's `declared_type_name` is `my::Circle`, not `my::Shape`. This is the payload the DESIGN named: "conforms? unwraps union membership."

### typealias: no predicate

`TypeDef::Alias` is skipped in `register_type_predicates` per doctrine. Typealias names a type, it does not introduce one; `(conforms? v :Alias)` works directly via the alias-unwrapping arm in `conforms_check`.

---

## Working tree on return

```
 M src/runtime.rs
 M src/freeze.rs
 M wat/Record.wat
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.6.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
