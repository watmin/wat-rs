# BRIEF-STONE-255.SF-ii — flip purity/determinism/category to enum markers

**Read first:** `DESIGN-STONE-special-form-doc-contract.md` § "The enum-marker
convention" (this dir). This brief is the strike path for the flip.

**Context:** Stone 255.SF built the special-form machinery (`wat_special_form!`,
`Kind::SpecialForm`, `src/intrinsic/special/{control_flow,binding}.rs`, registry
`handler: Option`, `@syntax`). It shipped purity as **`pure: bool`** with
`@purity preserving → true` — which DISCARDS "Preserving" (an `if` reports
`pure: true`, indistinguishable from `+`; the honest claim the author wrote is
unreflectable). This strike fixes that: flip to tri-state enums under one grammar.

## The convention (`@<EnumName> <Variant>`, exact case)

One rule for every closed-enum marker: marker name = the Rust enum, value = a
variant verbatim (exact case). Capitalized marker = closed enum; lowercase
(`@arg`/`@ret`/`@example`/`@added`/`@see`/`@syntax`) = structured/freeform.

## The work

1. **Three enums in `crates/wat-doc/src/lib.rs`** (the leaf crate the parser,
   macro, and main crate all reach — like `Arity`):
   - `Purity { Pure, Effectful, Preserving }`
   - `Determinism { Deterministic, Nondeterministic, Preserving }`
   - `Category { Encoding, Reflection, ControlFlow, Binding }`
   Each: exact-case parse (`FromStr` or fn) + `variants()` + `as_str()` (variant
   string == Rust variant name verbatim); derive Debug/Clone/Copy/PartialEq.

2. **Unified grammar in wat-doc**: ONE closed-enum rule — `@<EnumName> <Variant>`,
   value validated ∈ the named enum's `variants()` (error names the enum + lists
   variants). Applies to `@Purity`, `@Determinism`, `@Category`. **ANNIHILATE** the
   old `@pure`/`@deterministic` bool markers and `@category`-as-lowercase-string.
   Both `DocComment` (value path) and `DocSpecialForm` (special path) carry
   `purity: Purity`, `determinism: Determinism`, `category: Category`.

3. **Entry/Submission (`src/intrinsic/mod.rs`)**: `pure: bool → purity: Purity`,
   `deterministic: bool → determinism: Determinism`, `category: &'static str →
   category: Category`. Import from wat-doc; update the `registry()` builder.

4. **Macros (`crates/wat-macros`)**: `wat_intrinsic` + `wat_special_form` emit the
   enum values, validated exact-case at macro time (compile-error on bad variant).
   DROP the `KNOWN_CATEGORIES` hand-list — validate against `Category::variants()`.

5. **Re-fit exemplars**:
   - `bytes.rs`: `@pure true → @Purity Pure`, `@deterministic true → @Determinism
     Deterministic`, `@category Encoding → @Category Encoding`.
   - `control_flow.rs` (if) + `binding.rs` (let): `@Purity Preserving @Determinism
     Preserving @Category ControlFlow` / `Binding`.

6. **Cross-checks (`mod.rs` tests)**:
   - `pure_declared_matches_is_effectful_op`: `is_effectful_op(name)==false ⟺
     purity ∈ {Pure, Preserving}`; `==true ⟺ Effectful`.
   - `purity_mandated_examples`: `purity ∈ {Pure,Preserving} && determinism ∈
     {Deterministic,Preserving}` → ≥1 RUNNABLE `@example` mandatory; else norun.

7. **Reflection**: `render-doc`/`metadata-of` surface the enum so **"Preserving"
   reflects** (render-doc on `if` must show `Purity: Preserving`). Reflection
   parity: if `Kind` has a wat-side defenum in `wat/runtime-meta.wat`, add
   `Purity`/`Determinism`/`Category` defenums the same way (so `Value::Enum`
   carries the right `type_path`). `derive_pure_deterministic`: for REGISTERED
   forms read `entry.purity`/`entry.determinism` (the declared truth); the
   name-derive stays only as the residual for unregistered intrinsics.

8. **Strengthen the probe**: add `(assert-contains rendered "Preserving")` to the
   `render-doc-of-if` deftest in `wat-tests/reflect/special-form-doc-if-let.wat` —
   so the honest claim is WITNESSED, not just stored.

## Gate

- `cargo test --test test render_doc` → `render_doc_of_if`/`_let`/`_bytes_to_hex` ok.
- render-doc on `:wat::core::if` surfaces `Preserving` (the strengthened probe).
- `cargo test --lib` at the 36-fail floor; the 3 cross-checks pass.
- `cargo test -p wat-doc` green (+ new enum-parse tests).
- bytes unchanged. clippy clean in touched files.

## STOP triggers
1. If `pure: bool → purity: Purity` cascades into the checker's type table — STOP.
2. If a wat-side `Purity`/`Determinism`/`Category` defenum conflicts with an
   existing one — STOP, report.
