# 293.R2 — the repr collapse: ONE `Value::Aggregate`, holder is the only variance (R2 FULFILLMENT)

> **Status: STRIKE — the annihilation. Supersedes `DESIGN-293.R2-aggregate-codegen-merge.md`** (which was patching
> stems: the codegen is split ONLY because the repr is split). This is R2's *FRANGE UT UNUM FIAT* fulfilled at the
> value level — the Layer-2 collapse the base-struct NOTE wrongly deferred as "the optional horizon." Builder
> (2026-06-28): *"annihilate the variance — the restrictions are limitations on what the base holder can hold … i
> cannot understand why you are making many things … i break shit because it's already broken, successfully — fuck
> the preservation of 'works but wrong' — your hesitation is illogical."*

## The bug — three reprs for one thing (grounded `src/value/value.rs`)
```
Value::Struct(StructValue{ type_name, fields })              // no holon
Value::wat__Record{ class_fqdn, struct_form }                // no holon — type_name≡class_fqdn, fields≡struct_form
Value::wat__holon__Record{ class_fqdn, struct_form, holon_form }   // + the hologram
```
`(class, positional fields)` written three times, the only real difference a hologram present on one. Every consumer
special-cases by variant; the codegen is split (struct=Rust, record=wat macro, `register_record_methods` vestigial);
generic records silently drop their accessors. All of it is **variance that should not exist.**

## The contract decision (pinned)
ONE aggregate value. The `holder` enum is the required label; `holon` is a field present on every aggregate, the
`Empty` variant unless the holder is `HolonRecord`. The struct/record/holon "kinds" are **restrictions on the one
holder**, enforced as policy keyed on the label — never separate reprs, codegen, or macros.

```rust
// src/value/  — replaces the three variants
pub enum HolonForm { Empty, Hologram(Arc<HolonAST>) }    // "the enum who holds no value" — illegal-states-unrepresentable
pub struct AggregateValue {
    pub class:  String,          // the type fqdn (was type_name / class_fqdn)
    pub fields: Arc<Vec<Value>>, // positional (was StructValue.fields / struct_form)
    pub holder: Holder,          // {Struct, Record, HolonRecord} — the required label (src/types.rs, exists)
    pub holon:  HolonForm,       // Empty for Struct/Record; Hologram(..) for HolonRecord
}
// Value::Struct + Value::wat__Record + Value::wat__holon__Record  →  Value::Aggregate(Arc<AggregateValue>)
```

### The policy — holder is the ONLY variance (every difference is a restriction on the one holder)
| policy | Struct | Record | HolonRecord | enforced |
|---|---|---|---|---|
| crosses comms / edn-repr | ❌ never | ✅ must | ✅ must | `is_portable_type` → `holder != Struct` (exists, check.rs:13313) |
| carries hologram | `holon = Empty` | `holon = Empty` | `holon = Hologram(..)` | the constructor sets it per holder |
| identity (Eq/Hash) | `(class, fields)` | `(class, fields)` | the hologram (canonical) | one match on `holder`/`holon` |
| assignable where core wanted | only Struct slot | itself | ✅ `holon <: core` | the lattice edge (exists, PASSES) |

The hologram is **derived from `fields`** (a pure function — verified in `NOTE-base-struct-horizon.md`), so it is a
cache the constructor computes when `holder == HolonRecord`, never a separate substance.

## Decomposition (the cascade — depth-first, the fail-count is the meter)
- **R2.1 — the repr (THIS strike, the keystone).** Mint `HolonForm` + `AggregateValue`; replace the three `Value`
  variants with `Value::Aggregate`. **Ride every exhaustive `match` on the three variants to zero** — arms that did
  the same thing collapse to one `Value::Aggregate(a)`; arms that differed branch on `a.holder` / `a.holon`. The
  judgment sites (handle with care, see STOP triggers in the brief): **EDN encode/decode** (keys off the variant
  today → keys off `holder`); **Eq/Hash** (holon → hologram, else `(class, fields)`); **`is_portable`/wire gate**
  (→ `holder != Struct`); **`closure_extract`**; **the hologram derivation** at construction (when
  `holder == HolonRecord`). Gate = the workspace cascades to **0**, SET-diff ∅, AND the 293.R2 parity probe GREEN
  (generic record/holon accessors resolve — the break dies for free, because there is one repr).
- **R2.2 — the codegen.** With one repr, one walk mints ctor + accessor; `register_struct_methods` +
  `register_record_methods` + the `defrecord`/`defholon` macros' per-kind accessor/ctor emission collapse into one
  emission. (`register_record_methods` is already vestigial — runtime.rs:1429 skips when the macro registered the
  ctor.) The `parse_recordtype` bare-name fix (`parse_declared_name`, mirroring the struct path) lands here.
- **R2.3 — construction-form parity.** Bare `:T` for all three (drop `/new`); the `.wat` cascade (`Launched/new` → …).

## The gate
The existing RED probe `tests/types/probe_arc293_r2_aggregate_codegen_parity.{rs,wat}` (generic core-record +
holon-record accessors resolve, all three at parity; holon ⊂ core) — GREEN when the collapse lands. Plus: the whole
workspace green / SET-diff ∅ (the wire law, EDN round-trip, holon VSA, defservice State all preserved — the policies
ride the holder, unchanged in behavior).

## Out of scope (named)
- The `HolonAST` internals / VSA ops — untouched; `holon: Hologram(Arc<HolonAST>)` carries the existing type.
- `defenum` / `defnewtype` reprs — different shapes, not the three holders; leave them.

## Pairs
`REALIZATIONS.md` R2 *FRANGE UT UNUM FIAT* (this is its fulfillment — flip to PROBATUM on green) ·
`NOTE-base-struct-horizon.md` (Layer 2 — "the optional horizon" deferral was wrong; this is the root) ·
`value/value.rs` (the three variants) · `feedback_uniform_operation_or_decomplect_is_catastrophic` ·
`feedback_option_carrying_semantics_screams_enum` (why `HolonForm` is a named enum, not `Option`).
