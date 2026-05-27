# DESIGN — Stone S-B.1 — `:wat::core::recordtype` + `TypeDef::Record` (records become types)

**Arc:** 237, records-first-class thread (see `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`
+ `DESIGN-STONE-S-A-records-hierarchy.md`).
**Status:** READY (sub-DESIGN). The substrate half of S-B; S-B.2 (Record.wat macro
consumes it) follows.
**Builds on:** S-A SHIPPED (`d1e9cbe9`) — `is_subtype` + `typesub` registry + roots.

## Why this stone

Records are second-class: `defrecord` emits functions and registers NO type, so
`is-Circle?` is hand-emitted (narrowing `[v <- :wat::Record]` → type-errors on a
non-record instead of returning `false`), and a record class can't participate in
the type hierarchy. S-B.1 mints the **substrate type-declaration form** that makes
a record class a real `TypeDef` — so it inherits, autonomously, the type system's
uniform services: ∀T `is-X?` synthesis (via the existing `register_type_predicates`
pass) and `typesub` hierarchy membership. **The macro will emit this form (S-B.2);
it does NOT mint the predicate itself** — predicate synthesis is type-registration's
uniform job, not the macro's (the division of labor: macro emits the field-specific
constructor/accessors; type-registration mints the uniform ∀T predicate).

## Three-tier introspection doctrine (intueri-LOCKED 2026-05-25) — bounds this stone's conforms? scope

wat's type-introspection is THREE distinct questions, each its own canonical predicate
(Ruby-faithful: `instance_of?` / `is_a?` / duck). B.1 must NOT conflate them:

| tier | form | meaning | this stone |
|---|---|---|---|
| 1 exact | `is-<Name>?` (sugar) + `exact-type?` (general) | declared type == T | B.1 synthesizes `is-X?` ∀T |
| 2 lineage | `subtype-of?` (value×type) + `subtype?` (type×type, shipped) | T-or-a-subtype, via `typesub` walk | NOT B.1 — `subtype-of?` is its own stone |
| 3 conformance | `conforms?` (shipped) | union-membership / structural / alias | B.1 adds a nominal Record arm ONLY |

**Load-bearing consequence:** the hierarchy/lineage walk lives in the NEW `subtype-of?`
predicate (tier 2), **NOT** in `conforms?`. `conforms?` stays tier-3 forever — no
parent-walk is ever added to it. So B.1's `conforms_check` Record arm is **nominal-exact
only** (mirror Struct), and there is no "parent-walk deferred." (This corrects the earlier
draft, which planned to defer a conforms? parent-walk to B.2 — that walk does not exist;
it's `subtype-of?`.) `exact-type?` (tier-1 general) + `subtype-of?` (tier-2 value-lineage)
are their own small stones, orthogonal to B.1, built on shipped `is_subtype`.

## Name (intueri-LOCKED 2026-05-25)

**`:wat::core::recordtype`** — `(:wat::core::recordtype :my::Circle :wat::Record)`.
Mints-family sibling of `newtype` (compound lowercase, `*type` suffix on the
mint axis). Dodges the triple-collision with the `:wat::Record` umbrella type,
the `:wat::Record::def` macro, and the existing `:wat::core::record?` /
`record->map` value predicates. Shape: `(recordtype <class-fqdn> <parent-type>)`
where parent is `:wat::Record` (base) or `:wat::holon::Record` (holonic).

## What this stone delivers (substrate only — Record.wat untouched)

1. **`TypeDef::Record(RecordDef)`** variant in `src/types.rs`, `RecordDef { name:
   String, parent: String }`. Dedicated kind — NOT `TypeDef::Struct` (which would
   trip `register_struct_methods` into emitting a spurious `:my::Circle/new` +
   colliding accessors; confirmed: that pass iterates the whole TypeEnv's Structs,
   freeze.rs:853). `TypeDef::Record` is NOT fed to `register_struct_methods` → no
   constructor/accessor generation (the macro provides those in S-B.2).
2. **The `recordtype` declaration form** — parse + register:
   - decl-head dispatch recognizes `recordtype` (mirror how `typeunion` is wired:
     `classify_type_decl` ~types.rs:1803 + `parse_typeunion` ~types.rs:2145).
   - `parse_recordtype` → `RecordDef { name, parent }`.
   - registration (in the `register_with_span` path, mirroring how Union registers):
     insert the `TypeDef::Record` **AND** wire the subtype edge
     `env.register_subtype(name, parent)` (the parent edge is registered as part of
     declaring the record type — one form, both effects). Reject if the parent is
     unknown / the edge would cycle (reuse S-A's `register_subtype` cycle-check).
3. **`register_type_predicates` arm** (runtime.rs ~3208) — add `TypeDef::Record(r)
   => &r.name` so `is-<Name>?` synthesizes ∀T for record classes, identical to the
   four siblings. THIS kills the asymmetry.
4. **`conforms_check` Record arm** (runtime.rs ~16157) — `Some(TypeDef::Record(_))`
   → nominal identity (`concrete_type_name_matches`, like Struct/Enum/Newtype).
   Cascade-forced (the Path arm matches on the resolved TypeDef). Proven end-to-end
   in S-B.2 (needs a record VALUE, which the macro provides there); in B.1 it is the
   obvious-correct mirror of the Struct arm.
5. **Ride the exhaustiveness cascade** — adding a `TypeDef` variant forces a Record
   arm at every exhaustive `match` over `TypeDef` (~subset of ~135 `TypeDef::`
   refs; the Union precedent cascaded to ~4 sites — expect a similar handful:
   `types.rs`, `runtime.rs`, `check.rs`, likely `closure_extract.rs`). Substrate-as-
   teacher: `cargo build` names each site; add the Record arm (mirror the nearest
   nominal sibling — usually Struct). NOT a crisis; the fail-count is the meter.

## Out of scope (REJECTED — not deferral)

- **Any hierarchy/lineage behavior in `conforms?`** — REJECTED outright (not
  deferred). Lineage is tier 2 = the separate `subtype-of?` predicate (its own
  stone). `conforms?` stays tier-3 (union/structural/alias) forever; B.1's Record
  arm is nominal-exact, mirroring Struct.
- **`exact-type?` (tier-1 general) and `subtype-of?` (tier-2 value-lineage)** — their
  own small stones (mirror the shipped `subtype?`/`conforms?` mint shape; built on
  `is_subtype`). Orthogonal to B.1; not required for B.1's probe (which uses the
  synthesized `is-X?` + the shipped type×type `subtype?`).
- **Record.wat / `defrecord` changes** — S-B.2 (the macro emits `recordtype` + drops
  its hand-rolled predicate + constructor returns `-> :my::Circle`).
- **The arg-boundary `assignable` wiring** — S-A1 (rides on B.2's real record values).
- **Per-field type conformance / rich VSA field encodings** — arc 235.
- **`RecordDef` carrying fields** — not needed for type-identity + is-X? + subtype;
  the field shape lives in the macro's emitted accessors. `RecordDef { name, parent }`
  is the minimal honest variant.

## FM 2-bis probe (NEW — committed before the BRIEF)

`tests/probe_arc237_sB1_recordtype.rs`. Drives `startup_from_source` with the
`recordtype` form DIRECTLY (it is a real surface primitive — no defrecord macro,
no record value needed for the core contracts). Pre-stone: fails (the form is
unknown / `TypeDef::Record` doesn't exist). Post-stone: all PASS. Contracts:

1. **form registers** — `(:wat::core::recordtype :my::Circle :wat::Record)` at top
   level → startup succeeds (the class is a known type).
2. **is-X? synthesized ∀T (THE asymmetry-killer)** — `:my::is-Circle?` exists; called
   on a NON-record value → `false`, NOT a type error: `(:my::is-Circle? 42)` →
   `false`. (Pre-stone, records' hand-emitted predicate type-errored here; this is
   the death of the asymmetry.)
3. **edge wired by recordtype** — `(:wat::core::subtype? :my::Circle :wat::Record)`
   → `true` (declaring the record type registered its parent edge).
4. **directional** — `(:wat::core::subtype? :wat::Record :my::Circle)` → `false`.
5. **holon-flavor parent + transitive** — `(:wat::core::recordtype :my::Sphere
   :wat::holon::Record)`; `(:wat::core::subtype? :my::Sphere :wat::Record)` → `true`
   (Sphere → holon::Record → Record, transitive through the seeded root).
6. **unknown parent rejected** — `(:wat::core::recordtype :my::Bad :my::DoesNotExist)`
   → startup error (parent must be a known type).
7. **`:my::is-Circle?` true-path** — deferred to S-B.2 (needs a real `:my::Circle`
   value from the macro); noted, not asserted here.

Plus baseline: `cargo test --release --lib` ≥ 827/0 (Record.wat untouched → no
existing record test changes; the cascade arms are additive).

## Proven-moves template (mirror — arcs 237.1 / S-A)

- **`recordtype` parse+register mirrors `typeunion`** (237.1): `parse_typeunion`
  (types.rs:2145) + the `classify`/decl-head dispatch + `register_with_span`
  Union-validation arm are the exact shapes to copy. Record's registration ALSO
  calls `register_subtype` (S-A's method) — that's the one addition beyond the
  typeunion template.
- **`TypeDef::Record` cascade mirrors `TypeDef::Union`'s** (237.1 added Union and
  rode ~4 cascade sites). Same discipline: `cargo build` → add Record arm per
  named site → rebuild. 0 NEW files (all cascade sites are existing TypeDef
  matches).
- **`register_type_predicates` arm + `conforms_check` arm** mirror the existing
  Struct/Enum/Newtype/Union arms exactly.
- Trap-door: do NOT register the record class as `TypeDef::Struct` (spurious
  `/new` + colliding accessors via `register_struct_methods`). Dedicated
  `TypeDef::Record`, never fed to the struct-method pass.
- SCORE shape = SCORE-STONE-237.1 / SCORE-STONE-S-A.

## Files

- `src/types.rs` — `TypeDef::Record` + `RecordDef` + `recordtype` parse + register
  (+ edge wiring) + `TypeDef::name()` arm + cascade arms.
- `src/runtime.rs` — `register_type_predicates` Record arm + `conforms_check` Record
  arm + cascade arms.
- `src/check.rs` — recordtype as a recognized decl head if the checker validates
  decl forms (verify; mirror typeunion) + any cascade arm.
- Possibly `src/closure_extract.rs` (cascade only — mirror its Union arm).
- NO Record.wat. NO new `Value` variant. NO holon-rs (STOP-5).

## Calibration

New `TypeDef` variant + a decl form (parse+register, mirror typeunion) +
2 synthesis/conformance arms + a bounded exhaustiveness cascade. Comparable to
237.1 (typeunion: new TypeDef variant + parse + register + unify arms + cascade)
which shipped in-band. **Target band: 45–75 min Mode A; 100 STOP-3; 130 STOP-4.
Cascade: 2–3 rounds (variant → cascade arms → probe-green), handful of forced
sites, 0 new files.** Mirror SCORE-STONE-237.1 shape; cite 237.1 + S-A in the BRIEF.
