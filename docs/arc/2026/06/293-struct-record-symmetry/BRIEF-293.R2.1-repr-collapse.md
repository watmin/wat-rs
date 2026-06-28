# BRIEF — 293.R2.1: collapse the three aggregate `Value` variants → one `Value::Aggregate`

**The work, in one paragraph.** Three `Value` variants encode one thing — `(class, positional fields)` + an optional
hologram: `Value::Struct(StructValue{type_name, fields})`, `Value::wat__Record{class_fqdn, struct_form}`,
`Value::wat__holon__Record{class_fqdn, struct_form, holon_form}` (`src/value/value.rs:196/344/360`). **Replace all
three with ONE `Value::Aggregate(Arc<AggregateValue>)`** carrying `{ class, fields, holder, holon }`, where `holder:
Holder` (the existing `{Struct,Record,HolonRecord}` enum, `src/types.rs`) is the label and `holon: HolonForm` is
`Empty` for Struct/Record and `Hologram(Arc<HolonAST>)` for HolonRecord. Then **ride the exhaustive-`match` cascade to
zero** — every site that matched the three variants either collapses to one `Value::Aggregate(a)` arm (same behavior)
or branches on `a.holder` / `a.holon` (different behavior). This is behavior-preserving: the wire law, EDN round-trip,
holon VSA, defservice State, identity — all preserved, now keyed on the holder instead of the variant. It also fixes
the generic-record accessor parity break for free (one repr → one codegen).

## The one contract decision (pinned)
```rust
// src/value/value.rs (or a new src/value/aggregate.rs — judge by what keeps value.rs clean)
pub enum HolonForm { Empty, Hologram(Arc<HolonAST>) }     // named enum, NOT Option (it gates identity behavior)
pub struct AggregateValue {
    pub class:  String,           // was StructValue.type_name / wat__Record.class_fqdn
    pub fields: Arc<Vec<Value>>,  // was StructValue.fields / struct_form
    pub holder: crate::types::Holder,   // the required label
    pub holon:  HolonForm,        // Empty unless holder == HolonRecord
}
// DELETE Value::Struct, Value::wat__Record, Value::wat__holon__Record + struct StructValue.
// ADD  Value::Aggregate(Arc<AggregateValue>).
```
Add constructors/helpers as needed (e.g. `AggregateValue::struct_(..)`, `::record(..)`, `::holon(..)`, or one
`::new(class, fields, holder)` that computes the hologram when `holder == HolonRecord`). Keep `holon` an enum.

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/value/value.rs`** — the `Value` enum (`:196` `Struct`, `:344` `wat__holon__Record`, `:360` `wat__Record`)
   + `struct StructValue` + the `impl PartialEq` (`:652/695/701`) + `impl Hash` (`:943`) + `Display`/type-name. Make
   the type change here, then let the compiler waterfall. **Eq/Hash is a JUDGMENT site:** today holon hashes/eqs on
   `holon_form` (canonical, Stone 234.1) and base on `(class_fqdn, struct_form)` — preserve EXACTLY: a
   `Value::Aggregate` with `holon = Hologram(h)` → eq/hash on `h`; else on `(class, fields)`. Cross pairs (one holon,
   one base) → not equal (today they're different variants → `_ => false`; now same variant, so branch on `holon`).
2. **EDN encode/decode (`src/edn_shim.rs`)** — JUDGMENT site. Today keys off the variant (struct ↛ wire; base record
   encodes `struct_form`; holon encodes `holon_form` as the canonical tagged literal, projects `struct_form` on
   receipt — arc 234.7b "no recompute"). Rewrite to key off `a.holder`: `Struct` → not portable (the wire gate
   rejects, unchanged); `Record` → encode `(class, fields)`; `HolonRecord` → encode the `Hologram`. Decode rebuilds a
   `Value::Aggregate` with the right holder + holon.
3. **`is_portable_type` / the wire gate (`src/check.rs:13313`)** — already keyed on `holder == Struct` via the type
   def; the VALUE-side portability (if any) now reads `a.holder != Struct`. Confirm the comms send'/recv' path.
4. **`closure_extract.rs`, `rete/kernel.rs`, `rete/matcher.rs`, `runtime.rs`, `collection/*`** — the mechanical bulk:
   each `Value::Struct(_)` / `Value::wat__Record{..}` / `Value::wat__holon__Record{..}` arm → `Value::Aggregate(a)`,
   collapsing identical arms, branching on `a.holder`/`a.holon` where they differ. `grep -rn 'Value::Struct\b\|wat__Record\|wat__holon__Record\|StructValue' src/` for the full set — ride it to zero.
5. **The constructors** — `struct-new`, `:wat::Record::of` (2-arg), `:wat::holon::Record::of` (3-arg) in `runtime.rs`
   (grep) build the old variants; point them at `Value::Aggregate` with the right holder + `holon` (Empty for
   struct/record; the holon ctor computes `Hologram` from fields). `struct-field` / `Record/field-at` accessors read
   `a.fields[idx]` — they can UNIFY (both just index `fields`) but keep both keyword entry points working.

## STOP triggers (halt + surface — do NOT improvise)
- **STOP-1 (Eq/Hash identity drift):** if preserving holon-canonical vs base-structural identity under one variant is
  not a clean `match a.holon { Hologram(h) => …, Empty => (class,fields) }` — STOP and show the cross-variant cases.
- **STOP-2 (EDN codec):** if the holon encode/decode (`holon_form`-as-tagged-literal, project struct_form on receipt,
  "no recompute" 234.7b) does not map cleanly onto `holon: Hologram` — STOP and report the exact codec seam; do NOT
  recompute the hologram on decode if today's path doesn't.
- **STOP-3 (the cascade exceeds one coherent landing):** if the number of match sites is too large to change + verify
  in one pass without losing the thread — STOP, report the `grep` count + which files are done, so the orchestrator
  can split it. (A wide repr change is normal; a half-done one that doesn't compile is not.)

## The gate (the orchestrator re-runs forced-clean)
- `cargo build --release -p wat` cascades to **clean** (ride the exhaustive-match errors to zero).
- The 293.R2 parity probe GREEN (un-ignore): `cargo nextest run --release -E 'test(aggregate_codegen_parity_generic_record_accessors)'` → `(:r2::probe)` = 60. *(If the codegen still mangles generic record accessors, that is R2.2 — note it; the repr collapse alone may not fix the accessor KEY, which lives in the macro. If the probe stays RED on `:R/v`, report it as expected-for-R2.2, not a failure of R2.1.)*
- Whole workspace: `cargo nextest run --release` → floor 0, SET-diff ∅ vs HEAD. The wire/EDN/holon/defservice tests
  are the oracle — `-E 'test(core_record_def) + test(holon) + test(defstruct) + test(counter_on) + binary(types)'`
  must stay green.

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally; let the exhaustive-match cascade waterfall (the fail-count is the progress meter). Read
every diff. Trust only forced-clean builds. STOP + report if a STOP fires or the cascade exceeds one coherent landing.
