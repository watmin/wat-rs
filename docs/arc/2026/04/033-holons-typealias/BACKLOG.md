# Arc 033 — `:wat::holon::Holons` typealias — BACKLOG

**Shape:** three slices, mirroring arc 032's rhythm.

---

## Slice 1 — substrate: register the typealias

**Status: ready.**

Target: `src/types.rs::register_builtin_types`. Add the alias
registration right after arc 032's `BundleResult`:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::Holons".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![
            TypeExpr::Path(":wat::holon::HolonAST".into()),
        ],
    },
}));
```

Rust unit tests (mirror arc 032's pattern):
- `holons_alias_registered_with_builtins`
- `holons_alias_expands_to_expected_vec`

## Slice 2 — wat-rs call-site migration

**Status: obvious in shape** (once slice 1 lands).

Python substring-replace. Pattern:
`:Vec<wat::holon::HolonAST>` → `:wat::holon::Holons`
(and bare form for nested contexts).

Files swept:
- `src/runtime.rs` — doc comments
- `src/check.rs` — comments
- `tests/*.rs` — test fixtures that use the type
- `wat/holon/*.wat` — stdlib forms where Bundle's input appears
- `wat-tests/holon/*.wat` — test files
- `wat-tests/std/test.wat` — test-harness examples

**Sub-fogs:**
- **2a — bare-form inside `<>`.** Where `:Vec<wat::holon::HolonAST>`
  appears nested (e.g., `:AST<Vec<wat::holon::HolonAST>>`), the
  inner bare form should become `wat::holon::Facts` (no leading
  `:`). Single-pattern substring replace handles both positions
  because the leading `:` stays outside the match — same trick
  arc 032 used.

## Slice 3 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slice 2 lands).

- `docs/arc/2026/04/033-facts-typealias/INSCRIPTION.md`
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new row
- `docs/README.md` — arc index row
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md` —
  CHANGELOG row

---

## Working notes

- Opened 2026-04-23. Second arc under `/gaze`, same session as 032.
- Lab migration to Facts happens in lab arc 004 alongside the
  Scales + ScaleEmission aliases; see that arc's BACKLOG.
