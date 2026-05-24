# Sub-DESIGN — Arc 234 Stone 234.1 — `Value::wat_record` variant + Eq/Hash/Display + dispatch cascade

**Status:** ACTIVE (2026-05-24). Sub-DESIGN authored; FM 2-bis probe + BRIEF + EXPECTATIONS in flight.

**Builds on:**
- Stone 234.0 — `:wat::core::type` polymorphic primitive (commit `8b88ef8`); the TODO marker at `eval_type` line 14420 is where the wat_record arm lands
- Arc 216 Stone 216.5a — `impl PartialEq + Hash + Eq for Value` (the precedent we extend; src/runtime.rs:710 + 799+)
- Arc 233 Stone 233.2.l — `#[wat_value]` proc-macro seal (container-variant rule allows Vec<Self>; wat_record's three Arc'd field types pass naturally)
- Arc 234 DESIGN.md — the hologram thesis (this stone mints the storage form)

**Unblocks:**
- Stone 234.2 — `:wat::core::defrecord` macro (constructor generates `Value::wat_record` instances)
- All subsequent arc 234.x stones (record-y verbs, hash-destructure, VSA auto-dispatch)
- Revised Stone 232.1 — `:wat::core::defprotocol` now dispatches over wat_record AND other backends uniformly

---

## Doctrine

The substrate scaffolding for the hologram. Value::wat_record is a new Value variant carrying BOTH the Rust-struct form (fast access) and the HolonAST form (VSA-aligned) simultaneously. Neither derived from the other; both canonical. Field-type constraints (enforced at Stone 234.2's macro layer) guarantee isomorphism.

Stone 234.1 ships the STORAGE FORM ONLY. User-facing constructor + accessors are Stone 234.2's defrecord macro. The FM 2-bis probe constructs instances directly via Rust API (test helper) since no wat-level constructor exists yet.

---

## Locked decisions

### D1 — Variant shape

```rust
pub enum Value {
    // ... existing variants ...
    
    /// Arc 234 Stone 234.1 — the holographic dual-form record.
    /// 
    /// Carries both projections of an immutable record simultaneously:
    /// - `struct_form` for Rust-fast field access (positional Vec)
    /// - `holon_form` for VSA-aligned operations (HolonAST classifier-wrap)
    ///
    /// Field-type constraints (enforced at macro-expand time by defrecord)
    /// guarantee the two forms are isomorphic. The wat-record IS the hologram.
    wat_record {
        class_fqdn: Arc<String>,         // "myapp::Voltage" (no leading colon)
        struct_form: Arc<Vec<Value>>,    // ordered field values, declaration order
        holon_form: Arc<HolonAST>,       // Bind(Atom(class), Bundle(field-Binds...))
    },
}
```

Three Arc'd field types; none are Self (Vec<Value> is a container per arc 233 Stone 233.2.l's allow rule). `#[wat_value]` seal accepts naturally.

### D2 — Eq impl: delegate to holon_form

Two wat_records equal iff their holon_forms equal. Class_fqdn match is implicit (the holon_form's classifier-wrap encodes the class), but we check explicitly for short-circuit performance + structural honesty:

```rust
(Value::wat_record { class_fqdn: a_cls, holon_form: a_h, .. },
 Value::wat_record { class_fqdn: b_cls, holon_form: b_h, .. }) => {
    a_cls == b_cls && a_h == b_h
}
```

struct_form is access optimization; identity lives in holon_form (canonical per Stone 221.5 canonical bytes seed). Per arc 234 DESIGN's equality section.

### D3 — Hash impl: delegate to holon_form

Hash of a wat_record = hash of its holon_form. Per the canonical-form rule above. Compatible with arc 216 Stone 216.5a's value_hash infrastructure (holon_form is already HolonAST which has a Hash impl per arc 216).

```rust
Value::wat_record { holon_form, .. } => {
    "wat_record".hash(state);
    holon_form.hash(state);
}
```

(Mirror the existing pattern in `impl Hash for Value` where each variant tags itself with a discriminant string + hashes the canonical payload.)

### D4 — Debug auto-derive only; no Display impl in this stone

**Empirical finding 2026-05-24:** `Value` does NOT have a Display impl (only `ValueSnapshot` does; `src/runtime.rs:1821`). Adding Display for wat_record alone would be asymmetric with all other Value variants. Adding Display for the entire Value enum is scope creep (touches every variant; separate arc).

Stone 234.1 ships wat_record with Rust's auto-derived Debug impl ONLY. Errors + debug-print use `{:?}` formatting which renders the variant + fields automatically (including class_fqdn).

If Display becomes valuable later (e.g., for arc 234.6 migration sweep's user-facing error rendering), a separate arc/stone adds Display for the entire Value enum. Out of Stone 234.1's scope.

Probe asserts Debug output contains the class_fqdn (sufficient for the rendering-visibility contract).

### D5 — type_name() returns "wat::core::wat_record" (generic; per-instance lookup via eval_type)

```rust
Value::wat_record { .. } => "wat::core::wat_record",
```

Generic kind-string at the Rust-method layer (consistent with `Value::Struct(_) => "wat::core::Struct"`). The PER-INSTANCE FQDN is reachable via `:wat::core::type` which gets the wat_record arm in this stone (per D6).

### D6 — Extend `:wat::core::type` dispatch table (closes Stone 234.0's TODO marker)

```rust
let type_str = match &arg_val {
    Value::holon__HolonAST(h) => { ... }
    Value::Struct(sv) => { ... }
    Value::wat_record { class_fqdn, .. } => class_fqdn.to_string(),  // NEW
    other => other.type_name().to_string(),
};
```

Place at the location of Stone 234.0's TODO marker (src/runtime.rs:14420). Returns the class_fqdn directly (already FQDN without leading colon per convention).

### D7 — No user-facing constructor in this stone

`Value::wat_record` is constructable ONLY via Rust API in this stone. No wat-level constructor verb. Stone 234.2's `:wat::core::defrecord` macro will generate user-facing constructors that produce wat_record instances.

The FM 2-bis probe uses a test helper (or direct variant construction in Rust test code) to instantiate wat_record for property verification.

### D8 — Cascade: extend all exhaustive matches on Value

Adding a new Value variant triggers cargo to enumerate every exhaustive match-on-Value site. Sonnet's job is to address each surfaced site:
- `impl PartialEq for Value` (src/runtime.rs:710) — D2 arm
- `impl Hash for Value` (src/runtime.rs:799+) — D3 arm  
- `impl Display for Value` (find via grep) — D4 arm
- `Value::type_name()` (src/runtime.rs:1089) — D5 arm
- `eval_type` (src/runtime.rs:14421) — D6 arm (extends dispatch table)
- Any other exhaustive match-on-Value sites cargo surfaces

This IS the substrate-as-teacher cascade. Sonnet rides it; cargo enumerates the floors; each error names a site that needs the new arm.

### D9 — HolonRepresentable: NOT in this stone

Arc 234 DESIGN.md mentioned "HolonRepresentable impl returns holon_form directly" but `HolonRepresentable` is a PER-TYPE trait (src/comms/mod.rs:90), not impl'd on Value itself. For wat_record's holon-form access, the right path is:
- Stone 234.3 mints `:wat::core::record->holon` polymorphic primitive — returns wat_record's holon_form directly (no recomputation)
- `:wat::holon::*` verbs auto-dispatch on wat_record receivers (Stone 234.5) — use holon_form internally

Stone 234.1 just ensures the holon_form field is reachable via pattern destructure. The accessor verbs land in later stones.

### D10 — Per-variant single canonical name (HARD CUT)

`Value::wat_record` is the canonical variant name. No aliases, no parallel variant. Per `feedback_wat_llm_first_design`.

---

## Implementation surface

**`src/runtime.rs`:**

1. Add `wat_record` variant to `pub enum Value` (per D1; with doc comment per arc 233 + 234 doctrine)
2. Extend `impl PartialEq for Value` with D2 arm
3. Extend `impl Hash for Value` with D3 arm
4. Extend `impl Display for Value` (find via grep) with D4 arm
5. Extend `Value::type_name()` at line 1089 with D5 arm
6. Extend `eval_type` at line 14420 with D6 arm (closing the TODO marker)
7. Address any OTHER exhaustive match sites cargo surfaces (substrate-as-teacher cascade per D8)

**`src/check.rs`:**

Possibly no changes for this stone. Stone 234.1 doesn't add user-facing type-system primitives; the check.rs registration for wat_record TYPE ships when defrecord macro lands (Stone 234.2). However, if cargo surfaces a check.rs match on Value (exhaustive), address with a passthrough arm.

**Tests:**

1. `tests/probe_arc234_stone1_wat_record_variant.rs` — FM 2-bis probe (Rust-only; constructs Value::wat_record directly via test helper; asserts Eq/Hash/Display/type properties; LOAD-BEARING regression guard).

---

## FM 2-bis probe plan

Probe authored + committed BEFORE BRIEF. Rust-only (no wat-level construction available yet). Initial state: compile-fails (no `Value::wat_record` variant exists). Post-stone: compiles + passes all contracts.

Contracts:

1. **Construct** — `Value::wat_record { class_fqdn, struct_form, holon_form }` literal construction compiles (variant exists)
2. **Eq same** — two wat_records with same class + same holon_form return PartialEq::eq → true
3. **Eq different class** — two wat_records with DIFFERENT class + same holon_form structure return false
4. **Eq different fields** — two wat_records with same class + different field values (different holon_form) return false
5. **Hash same** — two equal wat_records produce equal hashes (`hash` of each via DefaultHasher)
6. **Display contains class** — `format!("{}", wat_record_instance)` contains the class_fqdn
7. **type_name() returns generic** — `Value::wat_record {..}.type_name()` returns `"wat::core::wat_record"` (the generic kind)
8. **:wat::core::type returns per-instance FQDN** — eval_type on a wat_record value returns the class_fqdn (the dispatch table extension from D6)

Probe is Rust-only — uses helpers from wat::runtime to construct + inspect. Test 8 may need to invoke eval_type via a Rust helper (similar to how Stone 234.0 probe goes through the eval pipeline but constructs the value as a test fixture).

---

## Substrate-as-teacher cascade (FM 15)

The substrate teaches via cargo errors. Sonnet's iteration:
1. Add Value::wat_record variant
2. `cargo build --release` — exhaustive-match errors enumerate every site needing the new arm
3. Address each site per D8's checklist; for sites NOT in checklist, apply the appropriate arm (Eq false, Hash discriminant, Display delegation, type_name generic, etc.)
4. Compile clean
5. `cargo test --release --test probe_arc234_stone1_wat_record_variant` — probe passes
6. Run regression guards (Stone 234.0 probe + Stone 232.0a + arc 233 family)

Expected cascade depth: 5-20 cargo errors as substrate enumerates the variant-match sites. Sonnet rides; each is direct application of the per-trait pattern.

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **`#[wat_value]` seal acceptance** — variant has three Arc'd field types (Arc<String>, Arc<Vec<Value>>, Arc<HolonAST>). Per arc 233 Stone 233.2.l: forbidden is `Box<Self>/Arc<Self>/Rc<Self>/Self` SINGLE-FIELD wrapping variants. Container variants (Vec<Self>, Option<Self>) are ALLOWED. wat_record's `Arc<Vec<Value>>` is a container (Vec<Self> wrapped in Arc) — should pass. If the proc-macro is strict about the wrapping rule, may need `#[wat_value(allow_wrapping = "wat-record carries dual form; struct_form is a container Vec<Value>, not a wrapping reference")]` escape hatch with non-empty reason string. Verify empirically.

2. **PartialEq + Hash consistency** — Per arc 216 Stone 216.5a: if `a == b` then `hash(a) == hash(b)`. D2's Eq delegates to holon_form; D3's Hash also delegates to holon_form. Consistent BY CONSTRUCTION. Verify via probe 5 contract.

3. **Display field rendering** — D4 renders `<class>(<field_1>, ...)`. For each field, use `format!("{}", field_value)` — Value's existing Display impl handles each value type. If existing Display doesn't handle some types cleanly, fall back to debug-form. Verify the rendering looks clean for primitive fields (i64, f64, String).

4. **Cascade scope creep** — sonnet should ONLY add arms for the new variant. If cargo surfaces a site needing OTHER changes (e.g., refactoring an existing arm), STOP and surface as honest delta. The cascade is "add arm per existing pattern"; not "refactor".

5. **No defrecord macro / no constructor verb** — D7 explicitly out-of-scope. If sonnet finds itself reaching toward exposing a wat-level constructor, STOP. The macro is Stone 234.2.

---

## Risks

- **#[wat_value] seal might reject Arc<Vec<Value>>** — mitigation: probe will surface at compile time if proc-macro is stricter than expected; escape hatch `#[wat_value(allow_wrapping = "...")]` available
- **Cascade larger than expected (>30 sites)** — mitigation: each site is mechanical per-pattern application; no design decisions; calibration absorbs cascade depth
- **Display impl edge cases on field rendering** — mitigation: each field uses Value's existing Display; well-precedented; if surfaces, handle case-by-case

---

## Out-of-scope (explicit)

- `:wat::core::defrecord` macro (Stone 234.2)
- User-facing constructor verbs
- Record-y polymorphic verbs (`assoc`, `record->map`, `record?`, `record->holon`, keyword-as-accessor) — Stone 234.3
- Hash-destructure — Stone 234.4
- `:wat::holon::*` auto-dispatch on wat-records — Stone 234.5
- Migration sweep — Stone 234.6
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D10
- HolonRepresentable trait impl — per D9, not the right shape for this stone

---

## Calibration prediction

**Target band:** 60–120 min Mode A
**Upper bound (STOP-3):** 180 min
**Confidence:** medium-high — variant addition is precedented; cascade depth is the main calibration variable.

**Rationale:**
- New variant: ~10 lines (with doc comment ~25 lines)
- 4 trait impl arms (Eq, Hash, Display, type_name): ~20 lines total
- eval_type extension (1 arm): 1 line + minor comment update
- Cascade addressed sites (estimated 5-20 cargo errors): ~30-100 lines mechanical fixes
- Probe authoring (8 contracts; Rust-only): ~200 lines
- Compile + iterate cycles: ~3-5 (variant addition + cascade addressing + final clean compile)
- SCORE writing: ~10 min

Stone 232.0a (~52 min in band) and Stone 234.0 (~38 min in band) are precedents for "well-precedented substrate addition." Stone 234.1 is bigger (variant + 4 impls vs Stone 234.0's single eval fn); 60-120 band reflects the cascade depth uncertainty.

---

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- STOP-1: unexpected compile errors not tracing to Value variant addition + impl extensions + dispatch cascade
- STOP-2: baseline lib tests regress below 827
- STOP-3: 180 min elapsed (apply partial-state-grading per `feedback_partial_state_grading`)
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — defrecord macro, constructor verb, record-y polymorphic verbs, destructure
- STOP-7: FM 2-bis probe doesn't flip compile-FAIL → all-contracts-PASS
- STOP-8: any arc 233 regression guard regresses
- STOP-9: Stone 232.0a typed-entities reflection probe regresses
- STOP-10: Stone 234.0 polymorphic type probe regresses (the immediate prior stone)

---

## What this unblocks

- **Stone 234.2** — `:wat::core::defrecord` macro generates `Value::wat_record` instances via Rust-level constructor (the variant must exist before the macro generates code referencing it)
- **Stone 234.3** — polymorphic record-y verbs all destructure wat_record via field access (struct_form for fast path; holon_form for VSA path)
- **Stone 234.4** — hash-destructure patterns match wat_record receivers
- **Stone 234.5** — `:wat::holon::*` auto-dispatch on wat_record uses holon_form
- **Revised Stone 232.1** — defprotocol's dispatcher now operates over wat_record (via :wat::core::type's extended dispatch table) AND other backends uniformly

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella; the hologram thesis
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md` — predecessor (`:wat::core::type` primitive); the TODO marker D6 closes
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — what 234.0 shipped (~38 min; clean traverse)
- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.1.md` — forthcoming
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.1.md` — forthcoming
- `tests/probe_arc234_stone1_wat_record_variant.rs` — FM 2-bis probe (forthcoming; commits before BRIEF)
- `src/runtime.rs:710` — `impl PartialEq for Value` (D2 arm location)
- `src/runtime.rs:799+` — `impl Hash for Value` (D3 arm location)
- `src/runtime.rs:1089` — `Value::type_name()` (D5 arm location)
- `src/runtime.rs:14420` — `eval_type` TODO marker (D6 location)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — `#[wat_value]` seal documentation
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
