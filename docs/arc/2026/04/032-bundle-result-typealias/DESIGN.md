# Arc 032 — `:wat::holon::BundleResult` typealias

**Status:** opened 2026-04-23. Small substrate arc, pure ergonomic
win — naming an existing type.

**Motivation.** `:wat::holon::Bundle` returns
`:Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>`
(arc 019 INSCRIPTION). Every downstream caller that threads
through a Bundle inherits the same Result shape. The type appears
44 characters wide at 30+ call sites across wat-rs source files,
wat stdlib files, wat-rs tests, wat-rs wat-tests, plus the lab's
`rhythm.wat` + `wat-tests/encoding/rhythm.wat`.

`/gaze` applied (2026-04-23): **`:wat::holon::BundleResult`**
— two-word, explicit, Level-2-safe (the `Result` suffix speaks
to a cold reader before they grep).

---

## Semantics

Structural typealias. `:A = :B` means `A` and `B` unify as the
same type at the checker layer; no wrapping, no coercion.

```
typealias :wat::holon::BundleResult
  = :Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>
```

Call sites that currently write the long form can write
`:wat::holon::BundleResult` instead. The checker sees both as
the same type. Existing code continues to work unchanged; the
migration is purely textual.

Non-parametric. Bundle's Ok arm is always `HolonAST`; Bundle is
the only operation producing `CapacityExceeded`. No other
`Result<_, CapacityExceeded>` exists or is planned.
Parameterizing would be speculative.

---

## Why this lives at the substrate

The type belongs to `:wat::holon::*` — same tree as `HolonAST`
and `CapacityExceeded`. Lab-local typealias would work
mechanically but leave every other wat consumer (future labs,
downstream crates, `wat-holon` if it ever ships) restating the
long form. One declaration at the substrate covers every caller.

---

## Registration

`TypeEnv::register_builtin` accepts `TypeDef::Alias(AliasDef {
name, type_params, expr })`. The alias goes next to the
existing `:wat::holon::CapacityExceeded` registration in
`src/types.rs::register_builtin_types`, mirroring the pattern:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::BundleResult".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            TypeExpr::Path(":wat::holon::HolonAST".into()),
            TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
        ],
    },
}));
```

`AliasDef` with empty `type_params` is a non-parametric alias.

---

## Non-goals

- **No renaming of `CapacityExceeded`.** The struct name stays.
  Only the Result-wrapper gets the alias.
- **No change to Bundle's actual signature.** Bundle still
  returns the expanded Result type; callers can choose to write
  either form.
- **No parametric `Bundled<T>`.** Ship-what-a-caller-needs.
  Every real use has `HolonAST` in the Ok arm.
- **No alias for other Result shapes** (e.g., `EvalError`'s
  Result). Each fallibility story gets its own name when its
  usage crosses the "worth naming" threshold — Bundle's has;
  the others haven't yet.

---

## Sweep targets

### wat-rs source (Rust + wat)

- `src/runtime.rs` — type annotations in error messages +
  internal docs where the long form appears.
- `src/check.rs` — existing scheme registrations that construct
  the Result type programmatically can stay as-is (they're
  Rust-side); comments referencing the type by long name get
  swept.
- `tests/wat_run_sandboxed.rs` — test fixtures.
- `tests/wat_bundle_capacity.rs` — test fixtures.
- `wat/holon/Trigram.wat` — stdlib algebra scheme annotation.
- `wat/holon/Ngram.wat` — same.
- `wat-tests/holon/Trigram.wat` — test.
- `wat-tests/holon/coincident.wat` — test.
- `wat-tests/std/test.wat` — test.

### Lab

- `wat/encoding/rhythm.wat` — source module.
- `wat-tests/encoding/rhythm.wat` — tests (shipped minutes ago
  as arc 003).

### Docs

- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row under the `:wat::holon::*` section.
- `docs/USER-GUIDE.md` — if any example uses the long form.
- `docs/README.md` — arc index row.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

---

## What does NOT get touched

- BOOK.md — narrative prose. The long form in historical
  chapters stays; it's what was on disk at that time.
- 058 proposals — historical records. Their INSCRIPTIONs
  already describe the substrate as it shipped; rewriting them
  to use a name that arrived later would falsify the record.

The alias is the CURRENT canonical form from arc 032 onward.
Prior text stays as prior text.
