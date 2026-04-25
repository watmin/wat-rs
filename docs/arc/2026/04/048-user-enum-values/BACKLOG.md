# wat-rs arc 048 — user-defined enum value support — BACKLOG

**Shape:** seven slices. Substrate change + new value rep + new
primitive + register + check + match extension + lab migration
+ docs/INSCRIPTION. Substantial; multiple cargo-test gates.

---

## Slice 1 — Value::Enum + EnumValue + SymbolTable extension

**Status: ready.**

`src/runtime.rs`:
- New `EnumValue` struct: `{ type_path: String, variant_name: String, fields: Vec<Value> }`.
- New `Value::Enum(Arc<EnumValue>)` variant in the `Value` enum.
  Update `type_name()` and `Debug` impl.
- New `SymbolTable.unit_variants: HashMap<String, EnumValue>` field.
  Populated by Slice 3's `register_enum_methods`. Keyword keys
  like `:trading::types::PhaseLabel::Valley` map to pre-built
  EnumValues for fast eval-time lookup.
- Update `SymbolTable::Default` impl + Debug field listing.

**Sub-fogs:**
- **1a — Hash/Eq for EnumValue.** Required for atom-hashing.
  Default-derive should work (String + Vec<Value> already Hash+Eq).
- **1b — Atom payload support.** Per 058-001, `:Atom<EnumValue>`
  should work. Hashing via the derived Hash impl is the path.

## Slice 2 — `:wat::core::variant` runtime primitive

**Status: ready** (after Slice 1).

`src/runtime.rs`:
- Add `:wat::core::variant` to keyword dispatch.
- New `eval_enum_new(args, env, sym)` helper:
  - Args: type-path keyword, variant-name keyword, then the
    field values (variadic — count determined by variant decl).
  - Returns `Value::Enum(Arc::new(EnumValue { type_path, variant_name, fields }))`.

`src/check.rs`:
- Register `:wat::core::variant` as a polymorphic special-cased
  primitive (returns the enum-typed value matching the first
  keyword arg). Body of synthesized variant constructors uses it.

**Sub-fogs:**
- **2a — Type-check special-casing.** `variant` returns a value
  whose type is determined by its first arg (a type-path keyword).
  Most synthesized constructors will have known return types
  registered separately (Slice 4's variant-constructor registry),
  so the special case is mostly a path-through.

## Slice 3 — `register_enum_methods` (parallel to register_struct_methods)

**Status: ready** (after Slices 1+2).

`src/runtime.rs`:
- New `pub fn register_enum_methods(types, sym) -> Result<(), RuntimeError>`.
- Walks every `TypeDef::Enum` in the TypeEnv.
- For each unit variant: insert `EnumValue` into `sym.unit_variants`
  at keyword path `:enum::Variant`.
- For each tagged variant: synthesize a `Function` entry at keyword
  path `:enum::Variant` whose body is `(:wat::core::variant
  :enum-path :Variant p1 p2 ... pn)`.
- Wire into freeze pipeline (`src/freeze.rs`) — call alongside
  `register_struct_methods`.

**Sub-fogs:**
- **3a — Reserved-prefix bypass.** `register_struct_methods` has
  a self-trust bypass for `:wat::holon::*` (built-in struct
  methods). `register_enum_methods` may need similar — none of
  wat-rs's own decls use enums, but the bypass shape should mirror
  for consistency.

## Slice 4 — Type checker — variant constructor registration

**Status: ready** (after Slices 1+2+3).

`src/check.rs`:
- When walking `TypeDef::Enum` in `CheckEnv::from_types`, register
  each tagged variant as a callable: `:enum::Variant` with
  `:fn(field-types...) -> :enum`.
- Unit variants are NOT functions in the type system; their values
  flow via the keyword-eval shortcut (Slice 5). The type checker
  needs a parallel mechanism: a `unit_variant_types: HashMap<String, EnumType>`
  registry that `infer_keyword` consults before the unknown-keyword
  fallback.

**Sub-fogs:**
- **4a — `variant` type pass-through.** When the synthesized
  function body invokes `variant`, the type checker accepts the
  first-arg keyword as a type marker and returns that type. Bypass
  via `variant`'s special inference arm.

## Slice 5 — Keyword eval extension for unit variants

**Status: ready** (after Slices 1+3).

`src/runtime.rs::eval`:
- Insert before the existing function-lookup arm:
  ```rust
  if let Some(ev) = sym.unit_variants.get(k) {
      return Ok(Value::Enum(Arc::new(ev.clone())));
  }
  ```
- Mirrors the `:None` shortcut.

`src/check.rs::infer_keyword`:
- Insert before unknown-keyword fallback:
  ```rust
  if let Some(enum_ty) = unit_variant_types.get(k) {
      return Some(enum_ty.clone());
  }
  ```

## Slice 6 — Pattern matching extension

**Status: ready** (after Slices 1+5).

`src/runtime.rs::try_match_pattern`:
- New arm: `Value::Enum(ev)` matched against pattern shapes:
  - `WatAST::Keyword(":enum::Variant")` — matches if `ev.type_path`
    + `ev.variant_name` align AND `ev.fields` is empty.
  - `WatAST::List([WatAST::Keyword(":enum::Variant"), binders...])`
    — matches if path/variant align AND `binders.len() == ev.fields.len()`,
    binds each binder to the corresponding field.
- Wildcard `_` arm continues to match anything.

`src/check.rs::infer_match` + `pattern_coverage`:
- When scrutinee type is a user enum, walk arm patterns and verify:
  - Each pattern's variant-path matches one of the enum's variants.
  - Binder count matches the variant's field count.
  - Every variant is covered (or `_` arm exists).
- Emit clear error messages for missing variants / arity mismatch.

**Sub-fogs:**
- **6a — Match arm body unification.** Arm bodies must unify to
  the declared `-> :Type` return. Existing logic for Option
  generalizes — it already unifies arm body types.
- **6b — Cross-enum mismatch.** `(:wat::core::match phase ((:Event::Candle ...)))`
  — variant doesn't belong to PhaseLabel's enum. Type checker
  rejects with `CheckError::TypeMismatch` naming the wrong enum.

## Slice 7 — Tests + lab migration + docs

**Status: ready** (after slices 1 – 6 land + green).

**Tests** (`src/runtime.rs::tests` inline):
- `unit_variant_constructs` — `:my::E::A` evaluates to Value::Enum
- `tagged_variant_constructs` — `(:my::E::B 42)` evaluates to
  Value::Enum with fields populated
- `match_dispatches_unit` — match on E selects the right unit arm
- `match_dispatches_tagged_with_binders` — match on E binds tagged
  fields and uses them in the body
- `match_exhaustiveness_required` — missing variant → CheckError
- `match_arity_mismatch` — wrong binder count → CheckError
- `match_cross_enum_rejected` — pattern from wrong enum → CheckError
- `enum_value_in_atom` — `:Atom<E>` round-trips with stable hash
- Plus end-to-end integration test at `tests/wat_user_enums.rs`.

**Lab migration** (separate commit-ready edit set):
- Rewrite 10 enum decls in `wat/types/enums.wat` + `wat/types/pivot.wat`
  to PascalCase variants (`:valley` → `:Valley`, etc.).
- Verify zero call sites break (no current callers — first-time
  use surfaces in arc 018).
- Lab cargo test green.

**Docs**:
- `docs/USER-GUIDE.md` §15 Forms appendix gains rows for enum
  construction (unit + tagged) and match-arm patterns.
- `docs/USER-GUIDE.md` §3 mental-model overview gains a sentence
  about user-enum support.
- `docs/CONVENTIONS.md` — variant naming convention pinned to
  PascalCase ("we embody host language").

**INSCRIPTION** + lab repo `058 FOUNDATION-CHANGELOG` row + push.

---

## Working notes

- Opened 2026-04-24 mid-arc-018. Lab arc 018 sketch surfaced
  `Candle::Phase` construction as the first-ever user-enum value
  call site. The 058-030 spec described enum DECLARATION fully
  but never pinned construction syntax; only Option ever shipped
  with value support. Arc 048 closes the gap.
- Substantial scope: Value enum addition, runtime keyword
  dispatch, register_enum_methods, type checker, match extension,
  tests, lab migration. Multiple cargo-test gates between slices.
- Lab migration is mechanical (10 decls × variant renames; zero
  current callers) but worth being deliberate about.
- After arc 048 ships, lab arc 018 resumes with substrate-direct
  use of PhaseLabel/PhaseDirection in test fixtures; market
  sub-tree completes.
