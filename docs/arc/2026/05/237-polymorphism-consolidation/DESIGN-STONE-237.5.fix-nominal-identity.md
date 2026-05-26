# DESIGN — Stone 237.5.fix-nominal-identity

**Status:** ACTIVE (2026-05-25 night-late).

**Origin:** Stone 237.5 shipped `:wat::core::conforms?` with a nominal-identity helper (`concrete_type_name_matches`) that, for any non-record value, compares `value.type_name()` to the Path name. Its FM 2-bis probe covered record / primitive / union / vector / alias — but **never enum, newtype, or the `:wat::core::struct` form**. The 237.6 crawl traced a defect: `Value::Enum.type_name()` returns the **generic** `"wat::core::Enum"` (runtime.rs:1254), not the enum's specific FQDN (which lives in the `EnumValue` payload, runtime.rs:1144). So `(conforms? some-color :my::Color)` compares `"wat::core::Enum" == "my::Color"` → **false, always**. Newtype + struct nominal identity are unverified and suspect for the same reason. conforms? is the foundation 237.6's `is-<Name>?` composes over; the foundation must be correct for every nominal form first (any-defect-catastrophic). This stone makes it so. No is-<Name>? work here (that stays 237.6).

---

## Scope

`src/runtime.rs` — the `concrete_type_name_matches` helper (and any per-form identity extraction it needs). Extend the conformance probe with enum / newtype / struct contracts. The probe is the authority: pre-stone it goes red on whichever nominal forms are actually broken; post-stone all green.

ONE file changed (`src/runtime.rs`) + one probe file (NEW). No check.rs change expected (the scheme `∀T × :keyword → :bool` is form-agnostic). No is-<Name>? auto-mint. No holon-rs (STOP-5).

## The defect, precisely

`concrete_type_name_matches` (runtime.rs):
```rust
match value {
    Value::wat__Record { class_fqdn, .. } => class_fqdn.as_str() == stripped,  // ✓ correct
    other => other.type_name() == stripped,                                     // ✗ wrong for Enum
}
```
`type_name()` returns a *generic kind string* for `Value::Enum` (`"wat::core::Enum"`), not the declared FQDN. Same risk for newtype (verify its runtime representation + whether `type_name()` returns the newtype FQDN or the inner type). Struct (`:wat::core::struct` form) likely returns its FQDN via `type_name()` but is unverified.

## Locked decisions

### D1 — per-form identity extraction

`concrete_type_name_matches` must read the **declared FQDN** the value actually carries, per Value kind:

| Value kind | identity source | match against Path |
|---|---|---|
| `Value::wat__Record { class_fqdn }` | `class_fqdn` (already correct) | `== stripped` |
| `Value::Enum(ev)` | the `EnumValue`'s declared enum FQDN (NOT `type_name()`) | `== stripped` |
| newtype value | the newtype's declared FQDN (find its runtime carrier — "nominal distinction via Atom hashing", runtime.rs:3007-3016) | `== stripped` |
| `Value::Struct` | struct FQDN via `type_name()` (verify it's the declared name, not generic) | `== stripped` |
| all other primitives | `type_name()` (already correct — i64/u8/f64/String/bool distinct) | `== stripped` |

The probe defines truth; sonnet finds the right field per form and makes it pass.

### D2 — no behavior change for the green forms

record / primitive / union / vector / alias must stay green (237.5 probe 12/12 must still pass). This stone only *adds* correct handling for enum/newtype/struct; it does not alter the working arms.

## FM 2-bis probe

`tests/probe_arc237_stone5fix_nominal.rs` (NEW) — committed before the BRIEF. Declares an enum, a newtype, and a `:wat::core::struct`, constructs an instance of each, and asserts conformance both ways:

- enum value `conforms?` its own enum type → true ; a *different* enum / non-enum → false
- newtype value `conforms?` its own newtype → true ; the inner type (e.g. `:f64`) and other types → false (the newtype is nominally distinct from its inner)
- struct value `conforms?` its own struct type → true ; a different struct → false
- (regression sentinel) one record + one primitive contract carried over, to prove the green arms stay green

Pre-stone: the enum contract fails (confirmed by trace); newtype/struct contracts reveal their actual state. Post-stone: all green.

## Out of scope (REJECTED — not deferral)

- `is-<Name>?` auto-mint — Stone 237.6.
- The 5-site emission / single-authority question — emerges from 237.6's small steps; not decided here.
- Fn/Var conformance — already errors per 237.5 contract.

## Calibration

ONE helper + per-form field extraction + probe. Smaller than 237.5 (no new dispatch, no walker — just correct the leaf identity check). **Target band: 15–30 min Mode A; 60 STOP.** Mirror the 234.3c.fix shape (tight, single-file).
