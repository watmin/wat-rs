# wat-rs arc 049 — newtype value support — BACKLOG

**Shape:** four slices. Implementation, tests, INSCRIPTION+docs,
lab unblock.

---

## Slice 1 — `register_newtype_methods` + freeze wiring

**Status: ready.**

`src/runtime.rs`:
- New `pub fn register_newtype_methods(types, sym) -> Result<(), RuntimeError>`.
  Mirrors `register_struct_methods` exactly. For each `TypeDef::Newtype`:
  - Synthesize `:Type/new(value :Inner) -> :Type` Function whose body is
    `(:wat::core::struct-new :Type value)`.
  - Synthesize `:Type/0(self :Type) -> :Inner` Function whose body is
    `(:wat::core::struct-field self 0)`.

`src/freeze.rs`:
- Insert one line after the existing `register_enum_methods` call
  (~freeze.rs:568): `crate::runtime::register_newtype_methods(&types, &mut symbols)?;`

**Sub-fogs:**
- (none expected.)

## Slice 2 — integration tests

**Status: obvious in shape** (once slice 1 lands).

New `tests/wat_newtype_values.rs`. ~5 tests:

1. **Construct + access round-trip** — `(:Price/new 100.0)` then
   `(:Price/0 ...)` returns 100.0.
2. **Nominal distinction in signatures** — function declared as
   `(:f -> :Price)` cannot return raw `:f64` (type-checker error).
3. **Inverse: cannot pass `:f64` where `:Price` expected** —
   `(:f (x :Price))` rejects `(:f 100.0)`.
4. **Atom-hash distinction** — `(Atom (:Price/new 100.0))` and
   `(Atom 100.0)` produce non-coincident vectors (different
   type_name in the StructValue → different EDN serialization →
   different hash).
5. **Newtype as struct field round-trip** — `(:Container/new
   (:Price/new 100.0) "label")` then access the Price field then
   `/0`-unwrap; coincident? against the original.

## Slice 3 — INSCRIPTION + USER-GUIDE + 058-030 addendum

**Status: obvious in shape** (once slices 1 – 2 land).

- `wat-rs/docs/arc/2026/04/049-newtype-values/INSCRIPTION.md` —
  records what shipped, the `/0`-over-`/value` decision, atom-hash
  reuse via Value::Struct.
- `wat-rs/docs/USER-GUIDE.md` — Language core list gains newtype
  construction/accessor mention. Forms appendix grows two rows
  (deferred follow-up as with arc 048 if time-pressed).
- `holon-lab-trading/docs/proposals/.../058-030-types/PROPOSAL.md`
  — INSCRIPTION addendum dated 2026-04-24 mirroring the
  2026-04-19 (struct) and 2026-04-24 (enum) addenda. Records the
  pinned construction + accessor syntax.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row documenting wat-rs arc 049.

## Slice 4 — wat-rs commit + push, lab unblock

**Status: obvious in shape** (once slices 1 – 3 land).

- wat-rs commit: runtime.rs + freeze.rs + tests/wat_newtype_values.rs
  + DESIGN + BACKLOG + INSCRIPTION + USER-GUIDE.
- Push wat-rs.
- Lab's arc 023 (exit/trade_atoms) opens with PaperEntry using
  Price properly. Lab repo's commit ships separately.

**Sub-fogs:**
- (none.)
