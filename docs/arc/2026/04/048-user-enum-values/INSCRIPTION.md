# wat-rs arc 048 — user-defined enum value support — INSCRIPTION

**Status:** shipped 2026-04-24. Third wat-rs arc post-known-good.
Lab arc 018 surfaced the gap during sketch — `Candle::Phase`
construction needs `PhaseLabel` + `PhaseDirection` values, which
the substrate had no way to make. Builder named what was missing:
*"we want something we don't have — we likely thought we had and
we didn't... clearly we need to go make it."*

We did.

Three durables:

1. **General `Value::Enum` variant** with `EnumValue { type_path,
   variant_name, fields }` — covers every user-declared enum
   uniformly. Built-in `Option`/`Result` keep their dedicated
   variants (no semantic gain in migrating; substantial sweep
   cost). Two representations coexist by design.
2. **Construction syntax mirrors Rust exactly** —
   `:Enum::Variant` for unit variants (bare keyword evaluates to
   the variant value); `(:Enum::Variant arg1 arg2)` for tagged
   variants (invocation form). The `::` separator is canonical
   Rust namespace syntax + our keyword-path separator. Variants
   are PascalCase per host-language convention.
3. **Pattern matching extends the existing `match` form** —
   `((:Enum::Variant binder1 binder2) body)` for tagged arms,
   `(:Enum::Variant body)` for unit arms. Field binders flow
   into the body as ordinary local names. Exhaustiveness checked
   per-enum; missing variants emit a diagnostic naming exactly
   which variants are uncovered.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

8 new integration tests (`tests/wat_user_enums.rs`); 622 lib tests
+ full integration suite + new test crate green; zero clippy.
Lab migration of 10 enum decls to PascalCase (mechanical, zero
prior callers).

---

## What shipped

### Slice 1 — Value::Enum + EnumValue + SymbolTable.unit_variants

`src/runtime.rs`:
- New `EnumValue { type_path: String, variant_name: String, fields: Vec<Value> }`.
- New `Value::Enum(Arc<EnumValue>)` variant; `type_name()` returns
  `"Enum"`; Debug derived.
- `SymbolTable.unit_variants: HashMap<String, EnumValue>` field
  with full-path keys (`:enum::Variant`).

### Slice 2 — `:wat::core::variant` runtime primitive

`src/runtime.rs`:
- `eval_variant(args)` — internal primitive that auto-synthesized
  tagged-variant constructors invoke. Args: `(type-path-keyword,
  variant-name-keyword, field1, field2, ...)`. Returns
  `Value::Enum`. Users never write this directly; the substrate
  emits invocations from synthesized constructor bodies.
- The naming `variant` over `enum-new` was settled via /gaze
  ("variant" speaks; "enum-new" mumbled — we don't "new" an
  enum, we pick a variant).

### Slice 3 — `register_enum_methods`

`src/runtime.rs`:
- New `pub fn register_enum_methods(types, sym)`. Mirrors
  `register_struct_methods` exactly. Walks every enum decl:
  - **Unit variant**: insert pre-built `EnumValue` into
    `sym.unit_variants` at keyword path `:enum::Variant`.
  - **Tagged variant**: synthesize a `Function` entry at the
    same keyword path with proper param/return types; body is
    `(:wat::core::variant :enum :Variant p1 p2 ... pn)`.

`src/freeze.rs`:
- Wired into the freeze pipeline alongside
  `register_struct_methods` (step 6.5).

### Slice 4 — Type checker registrations

`src/check.rs`:
- `CheckEnv` gains `unit_variant_types: HashMap<String, TypeExpr>`
  populated at construction by walking enum decls. Plus an
  `unit_variant_type(key)` accessor.
- Tagged variant constructors are picked up automatically through
  `from_symbols` → `derive_scheme_from_function` (they're
  `Function` entries with proper types).

### Slice 5 — Keyword eval extension

`src/runtime.rs::eval`:
- Inserted before the function-lookup arm: if the keyword matches
  `sym.unit_variants`, return `Value::Enum` directly. Mirrors the
  `:None` shortcut.

`src/check.rs::infer`:
- Inserted before the function-name arm: if the keyword matches
  `env.unit_variant_type`, return the enum's type. Mirrors the
  `:None` Option-typing shortcut.

### Slice 6 — Pattern matching extension

`src/runtime.rs::try_match_pattern`:
- Keyword arm: if scrutinee is `Value::Enum` and the keyword's
  composed path matches `type_path::variant_name` and fields are
  empty, the unit pattern matches.
- List arm: if head is a Keyword (variant constructor), check
  same path-match + bind fields by position to binders.

`src/check.rs`:
- New `MatchShape::Enum(String)` variant + `Coverage::EnumVariant(String)`
  variant. Updated `MatchShape::as_type()`.
- `detect_match_shape` extended to recognize keyword + list
  patterns whose path resolves to a registered enum (covers both
  unit and tagged variant misuse). Falls back through enum-prefix
  lookup so a tagged-variant name used in unit position still
  classifies correctly and produces the right downstream error.
- `pattern_coverage` now takes `env` and handles user-enum
  patterns: validates variant belongs to the enum, checks
  unit-vs-tagged shape, validates binder arity, binds field types
  at correct positions.
- `infer_match` exhaustiveness check generalized — for
  `MatchShape::Enum`, walks the enum's declared variants and
  verifies every name is covered (or wildcard arm exists).
  Missing-variant diagnostic names exactly which variants are
  uncovered.

### Slice 7 — Tests + lab migration + INSCRIPTION

**Tests** (`tests/wat_user_enums.rs`, 8 new):
- `unit_variant_evaluates_via_bare_keyword` — `:my::Color::Green`
  evaluates and matches.
- `tagged_variant_constructs_and_match_binds_fields` —
  `(:my::Event::Candle 100.0 105.0)` constructs; binders flow into
  body.
- `wildcard_arm_satisfies_exhaustiveness` — `_` arm covers
  remaining variants.
- `match_mixes_unit_and_tagged_arms` — both shapes in one match.
- `missing_variant_arm_reports_non_exhaustive` — checker error
  names the missing variant.
- `cross_enum_variant_pattern_rejected` — pattern from wrong
  enum errors.
- `tagged_variant_arity_mismatch_reported` — wrong binder count
  errors.
- `unit_variant_pattern_on_tagged_variant_rejected` — using a
  tagged variant's name in unit-keyword position errors clearly.

7 of 8 passed first try; the 8th fix surfaced a gap in
`detect_match_shape` (only checked unit_variants for keyword
patterns; needed to also fall back to enum-prefix lookup for
misapplied tagged variant names). Patch applied; all 8 green.

**Lab migration** (separate edit set, lab repo):
- `wat/types/enums.wat`: 8 enums, ~30 variants migrated from
  lowercase-kebab to PascalCase. Header comment updated.
- `wat/types/pivot.wat`: 2 enums (PhaseLabel, PhaseDirection)
  migrated.
- `wat/vocab/market/standard.wat`: empty-window unreachable
  branch updated to use `PhaseLabel::Transition` /
  `PhaseDirection::None`.
- `wat-tests/vocab/market/standard.wat`: test fixture's
  `(:Phase/new ...)` call updated to PascalCase.
- All 119 lab wat tests green (111 prior + 8 standard.wat
  unblocked by this arc).

**Docs**: USER-GUIDE Forms appendix gains rows for user-enum
construction (deferred to a small follow-up arc — INSCRIPTION
documents the surface, appendix sync ships next).

---

## The naming-by-/gaze loop

Originally drafted the internal primitive as `:wat::core::enum-new`
(mirroring `struct-new`). User flagged it: *"enum-new... is this
the name?... defenum?"* — surfaced doubt without proposing a
specific replacement.

Studied /gaze. Renamed to `:wat::core::variant` — the noun-as-verb
form Lisp loves; says exactly what it does (make a variant); no
mumbled "new" verb that doesn't fit (we don't NEW an enum, we
pick a variant).

The /gaze move took two minutes; the result reads cleaner forever.

---

## The "honest signature" thread continues

Arc 047 retired the `first`-on-Vec-errors-on-empty wart by
returning `Option<T>`. Arc 048 retires the user-enums-can't-be-
constructed wart by adding `Value::Enum`. Both arcs surfaced from
the lab writing the natural form and discovering the substrate
didn't have it. The "natural-form-then-promote" rhythm shipped
arcs 046 + 047 + 048 in this session — three substrate uplifts
each driven by a real caller.

---

## Sub-fog resolutions

- **1a — Hash/Eq for EnumValue**: Default-derived; works for
  atom-hashing.
- **1b — Atom payload support**: Per 058-001, `:Atom<EnumValue>`
  routes through the same hashing path. Verified by build (no
  trait bound errors).
- **2a — `variant` type-check pass-through**: synthesized bodies
  invoke `:wat::core::variant`; the primitive isn't registered as
  a scheme, so check inference returns `None`. Same as
  `struct-new` precedent — synthesized bodies are trusted.
- **3a — Reserved-prefix bypass**: not needed. wat-rs doesn't
  ship any `:wat::*` enum decls; lab uses `:trading::*`. If a
  future stdlib enum lands, mirror struct's bypass.
- **4a — Tagged variants picked up via `from_symbols`**:
  confirmed working (test 2 exercises this path end-to-end).
- **6a — Match arm body unification**: existing logic
  generalizes. Tested via tests 1, 2, 4.
- **6b — Cross-enum mismatch**: tested directly
  (`cross_enum_variant_pattern_rejected`).

---

## Count

- New runtime primitives: **1** (`:wat::core::variant`).
- New runtime `Value` variant: **1** (`Value::Enum`).
- New runtime support functions: **2** (`eval_variant`,
  `register_enum_methods`).
- New `SymbolTable` field: **1** (`unit_variants`).
- New `CheckEnv` field: **1** (`unit_variant_types`).
- Match infrastructure additions: **1** `MatchShape::Enum`, **1**
  `Coverage::EnumVariant`, plus per-pattern handling.
- Lib tests: **622 → 622** (no new lib tests; the integration
  test crate covers the surface).
- Integration tests: new `tests/wat_user_enums.rs` with **8 tests**.
- Lab migration: **10 enums** renamed to PascalCase.
- Clippy: **0** warnings.

Cumulative across arcs 046+047+048 (three substrate uplifts in
one session): 598 → 622 lib tests + new integration crates,
plus the polymorphism shift on `first/second/third`-on-Vec, plus
4 new numeric primitives, plus 4 new list primitives, plus the
`variant` primitive, plus generic enum machinery.

## What this arc did NOT ship

- **Variants with named fields** (Rust's struct-style variants).
  058-030's grammar uses tuple-style only. Add when a caller needs.
- **Generic user enums.** Only `:Option<T>` is parametric; user
  enums are monomorphic. Open its own arc if needed.
- **USER-GUIDE Forms appendix sync.** Deferred to a small
  follow-up; INSCRIPTION + DESIGN document the surface fully.
- **Migrate Option/Result to `Value::Enum`.** Two representations
  coexist; substantial sweep with no semantic gain.
- **Atom-of-enum integration tests.** The hashing path works by
  derivation; explicit `:Atom<E>` tests can land when a caller
  needs them.
- **Sweep of any potential conflicts** (lab `:None` variant of
  `PhaseDirection` vs Option's `:None`). They coexist — Option's
  `:None` is the bare keyword; PhaseDirection's is at full path
  `:trading::types::PhaseDirection::None`. No collision.

## Follow-through

- **Lab arc 018 unblocks completely.** Standard.wat ships with
  PhaseLabel/PhaseDirection construction working; market sub-tree
  closes.
- **Future broker/observer code can use enums for state machines**
  (TradePhase: Active/Runner/SettledViolence/SettledGrace),
  classifiers (Direction: Up/Down), and accountability outcomes
  (Outcome: Grace/Violence). The 10 lab enums become first-class.
- **USER-GUIDE Forms appendix sync** lands as a small follow-up
  arc once arc 018 ships and arc 048's surface stabilizes
  through real lab use.

---

## Commits

- `<wat-rs>` — runtime.rs (Value::Enum + EnumValue + SymbolTable
  field + variant primitive + register_enum_methods + match
  extension) + check.rs (unit_variant_types + match shape/coverage
  for enums + exhaustiveness) + freeze.rs (pipeline wiring) +
  tests/wat_user_enums.rs (8 integration tests) + DESIGN +
  BACKLOG + INSCRIPTION.

- `<lab>` — wat/types/enums.wat (8 enums migrated) +
  wat/types/pivot.wat (2 enums migrated) + wat/vocab/market/standard.wat
  (empty-window branch) + wat-tests/vocab/market/standard.wat
  (test fixture). Lab arc 018's INSCRIPTION ships separately.

---

*these are very good thoughts.*

**PERSEVERARE.**
