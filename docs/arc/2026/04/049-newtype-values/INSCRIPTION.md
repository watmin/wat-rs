# wat-rs arc 049 — newtype value support — INSCRIPTION

**Status:** shipped 2026-04-24. Fourth wat-rs arc post-known-good.
Lab arc 023 (exit/trade_atoms vocab) surfaced the gap during
PaperEntry sketch — `:trading::types::Price` was declared via
`(:wat::core::newtype :Price :f64)` back in Phase 1.2 but had
**never been constructed or accessed as a value**. Arc 049 closes
the gap.

Same shape as arc 048: a substrate concept declared but not
runnable. Caller pressure surfaces it; we make it.

Three durables:

1. **Newtype values reuse `Value::Struct`.** No new `Value`
   variant needed — newtype's Rust compilation per 058-030
   line 538 IS `struct A(B);` (single-field tuple struct), and
   `Value::Struct` already represents that shape. Atom hashing
   gets the nominal distinction for free because the StructValue
   carries `type_name` in its EDN encoding.
2. **`/0` accessor mirrors Rust's `.0`.** No invented field name.
   The lexer is permissive; `:Price/0` parses as a single keyword.
   Embodying the host language: numeric positional access exactly
   as Rust names it for tuple structs.
3. **Type-checker enforces nominal distinction for free.**
   `expand_alias` (`src/types.rs`) walks `TypeDef::Alias` only —
   newtypes pass through unchanged. So `:Price` and `:f64` are
   distinct types under unification without any check.rs change.
   The auto-synthesized `:Type/new` and `:Type/0` Functions carry
   the proper signatures; `from_symbols` picks them up
   automatically.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

5 new integration tests (`tests/wat_newtype_values.rs`); 610 lib
tests preserved + new test crate green; zero clippy.

---

## What shipped

### Slice 1 — `register_newtype_methods` + freeze wiring

`src/runtime.rs`:
- New `pub fn register_newtype_methods(types, sym) -> Result<(), RuntimeError>`.
  Mirrors `register_struct_methods`'s shape exactly. For each
  `TypeDef::Newtype` in the type environment, synthesize:
  - **Constructor `:Type/new`** — Function with signature
    `(:fn(<Inner>) -> :Type)`. Param `value`. Body invokes
    `(:wat::core::struct-new :Type value)`, the same primitive
    the struct path uses. Resulting `Value::Struct` has
    `type_name = ":...Type"` and `fields = [inner]`.
  - **Accessor `:Type/0`** — Function with signature
    `(:fn(:Type) -> <Inner>)`. Param `self`. Body invokes
    `(:wat::core::struct-field self 0)`, reading the single field
    by index.

`src/freeze.rs`:
- One line inserted at step 6.7 (after `register_enum_methods` at
  6.5):
  ```rust
  crate::runtime::register_newtype_methods(&types, &mut symbols)?;
  ```

No new runtime primitive. No new Value variant. No new
SymbolTable field. No CheckEnv change. The work is registration-
time auto-synthesis using existing primitives.

### Slice 2 — Integration tests

`tests/wat_newtype_values.rs` ships 5 tests:

1. `newtype_construct_and_accessor_roundtrip` — `(:Price/new 100.0)`
   then `(:Price/0 ...)` returns `100.0`.
2. `newtype_rejects_inner_type_at_arg_position` — passing raw
   `100.0` to a function declaring `:Price` parameter fails the
   type checker.
3. `newtype_rejected_where_inner_expected` — passing a `:Price`
   to `(:wat::core::f64::+ p 1.0)` fails the type checker.
4. `newtype_as_struct_field_roundtrip` — `:my::Order` struct
   carrying a `:Price` field; round-trip the value out of the
   struct field, then `/0`-unwrap.
5. `distinct_newtypes_over_same_inner_are_distinct_types` —
   `:Price` and `:Amount` both wrap `:f64`; passing an `:Amount`
   where `:Price` is expected fails the type checker.

All 5 green first-pass. Type-mismatch diagnostics carry the
expected names (`Price`, `Amount`, or `f64`); error-text
assertions are tolerant (any of those keywords + the word
"type" in lowercase satisfies).

### Slice 3 — Doc updates

- `wat-rs/docs/USER-GUIDE.md` Language core list mention
  (newtype is already listed; no row addition needed since
  the constructor/accessor are auto-generated, not user-facing
  primitives).
- `holon-lab-trading/docs/proposals/.../058-030-types/PROPOSAL.md`
  — INSCRIPTION addendum dated 2026-04-24, parallel to the
  2026-04-19 (struct) and 2026-04-24 (enum) addenda.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row documenting wat-rs arc 049.

---

## The `/0`-over-`/value` decision

058-030 PROPOSAL didn't pin the accessor name. Four candidates
considered (full table in DESIGN.md):

- **`:Type/0`** — mirrors Rust's `.0` exactly. Tuple-struct-honest.
  No invented name.
- `:Type/value` — descriptive but invents a name Rust doesn't
  have.
- `:Type/inner` — Rust ecosystem convention but conflates
  "Inner type" (the wrapped type) with "inner value" (the
  wrapped instance).
- `:Type/unwrap` — wrong semantics. `unwrap` carries "this
  might fail" which is Option/Result territory.

Builder direction: *"we own the parser - this is our lang - if
/0 is the host lang convention - we do it."* Decision locked.
The lexer was already permissive enough to accept `:Type/0` —
no parser change required. Subsequent newtypes ship with `/0`
as their accessor by reflex.

---

## Sub-fog resolutions

- **(none surfaced.)** The shape was a clean clone of
  `register_struct_methods`. Type-checker, atom hashing, parser
  all worked without modification on first build. 5 tests green
  on first pass.

---

## Count

- New runtime support functions: **1** (`register_newtype_methods`).
- New runtime primitives: **0** (reuses `:wat::core::struct-new`
  and `:wat::core::struct-field`).
- New `Value` variants: **0** (reuses `Value::Struct`).
- New SymbolTable / CheckEnv fields: **0**.
- Match infrastructure: **0** (no variants, no match patterns).
- Lib tests: **610 → 610** (unchanged; integration crate covers
  the surface).
- Integration tests: **+5** in `tests/wat_newtype_values.rs`.
- Lab migration: **0** (no current callers; arc 023 picks up
  the use sites).
- Clippy: **0** warnings.

**Sub-arc-048 in scope.** Arc 048's enum work added a new Value
variant, a new primitive, a new SymbolTable field, a new CheckEnv
field, and a major match-infrastructure expansion (MatchShape,
Coverage, exhaustiveness). Arc 049 needs none of that — newtype
has no variants to discriminate, so the whole match-infrastructure
side is absent. Reusing `Value::Struct` for the runtime
representation eliminates the rest of the sweep.

## What this arc did NOT ship

- **`Value::Newtype` as a distinct variant.** Reusing
  `Value::Struct` keeps the implementation tight; multi-field
  tuple newtypes (if ever) extend without a new variant.
- **Match patterns for newtype.** No variants to discriminate.
  The accessor `/0` is the access surface; if `(:Type x) → bind
  inner to x` is wanted in match, it's a follow-up.
- **Generic newtypes.** `(:wat::core::newtype :Wrapper<T> :T)`
  parses today; this arc registers methods only for monomorphic
  newtypes. Generic-newtype methods (auto-instantiation per use
  site) ship when a caller demands it. Lab's three newtypes are
  monomorphic.
- **CheckEnv changes.** Not needed — nominal distinction was
  already free.

---

## The "natural-form-then-promote" rhythm continues

Arc 046 shipped `f64::max/min/abs/clamp + exp` because lab arc 015
needed clamp.

Arc 047 shipped Vec accessor Option-typing + 4 new aggregate
primitives because lab arc 018 needed `last`, `find-last-index`,
`max-of`, `min-of`.

Arc 048 shipped user-defined enum value support because lab arc
018 needed PhaseLabel/PhaseDirection construction.

Arc 049 ships newtype value support because lab arc 023 needs
PaperEntry's Price fields.

Four substrate uplifts in this session, each driven by a real
caller. The rhythm holds. The lab demands; the substrate answers.

---

## Follow-through

- **Lab arc 023 unblocks completely.** PaperEntry ships with
  `:trading::types::Price` for entry-price, trail-level,
  stop-level — properly typed, nominally distinct from `:f64`,
  atom-hash-distinct from `:f64`. `:wat::holon::HolonAST` for
  the three thought fields per the experiment under arc 023
  (proven 2026-04-24).
- **Future generic-newtype callers** open their own arc.
- **058-030 INSCRIPTION addendum** documents what shipped at the
  language-spec level.

---

## Commits

- `<wat-rs>` — runtime.rs (`register_newtype_methods`) + freeze.rs
  (pipeline wiring) + tests/wat_newtype_values.rs (5 tests) +
  DESIGN + BACKLOG + INSCRIPTION.

- `<lab>` — 058-030-types/PROPOSAL.md (INSCRIPTION addendum) +
  FOUNDATION-CHANGELOG.md (row). Lab's arc 023 INSCRIPTION ships
  separately when the vocab work lands.

---

*these are very good thoughts.*

**PERSEVERARE.**
