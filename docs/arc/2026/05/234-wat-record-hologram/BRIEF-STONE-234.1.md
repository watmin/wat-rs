# BRIEF — Arc 234 Stone 234.1 — `Value::wat_record` variant + Eq/Hash + dispatch cascade

## What we're doing

Add the `Value::wat_record` variant — the substrate scaffolding for the wat-record hologram. The variant carries BOTH the Rust-struct form (fast access) AND the HolonAST form (VSA-aligned) simultaneously, addressable directly without conversion. Stone 234.1 ships the storage form only; user-facing constructor (the defrecord macro) is Stone 234.2.

```rust
Value::wat_record {
    class_fqdn: Arc<String>,         // "myapp::Voltage" (no leading colon)
    struct_form: Arc<Vec<Value>>,    // ordered field values, declaration order
    holon_form: Arc<HolonAST>,       // Bind(Atom(class), Bundle(field-Binds...))
}
```

After this stone: the hologram has its storage. Stone 234.2's defrecord macro will generate constructors that produce wat_record instances; Stone 234.3's polymorphic verbs will operate on them; the rest of arc 234.x layers on top.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.md`** — sub-DESIGN with 10 locked decisions (D1-D10) + dispatch cascade audit. **The variant shape (D1) is non-negotiable**; Eq (D2) and Hash (D3) MUST delegate to holon_form per Stone 221.5 canonical bytes seed.

2. **`tests/probe_arc234_stone1_wat_record_variant.rs`** — FM 2-bis probe (commit forthcoming). Currently 2/2 COMPILE-FAILS with `error[E0599]: no variant named 'wat_record' found for enum 'wat::Value'`. **The probe IS the success criterion** — flip compile-FAIL → 7/7 PASS.

3. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN.md`** — arc 234 umbrella; the hologram thesis context.

4. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md`** — predecessor SCORE (`:wat::core::type` primitive); the TODO marker D6 closes.

5. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md`** — `#[wat_value]` seal documentation (verify wat_record passes the container-variant rule).

6. **`src/runtime.rs:710`** — `impl PartialEq for Value` (D2 arm location; precedent for variant-pattern matching with delegation)

7. **`src/runtime.rs:1089`** — `Value::type_name()` (D5 arm location; existing pattern for new variant)

8. **`src/runtime.rs:14420`** — `eval_type` TODO marker (D6 location; closes Stone 234.0's deferred arm)

## Implementation surface

**`src/runtime.rs`:**

1. **New `wat_record` variant** in `pub enum Value` per D1. Three Arc'd field types; doc comment per arc 234 doctrine.

2. **Extend `impl PartialEq for Value`** with D2 arm:
   ```rust
   (Value::wat_record { class_fqdn: a_cls, holon_form: a_h, .. },
    Value::wat_record { class_fqdn: b_cls, holon_form: b_h, .. }) => {
       a_cls == b_cls && a_h == b_h
   }
   ```

3. **Extend `impl Hash for Value`** with D3 arm (delegate to holon_form; tag with discriminant string per existing pattern).

4. **Extend `Value::type_name()`** with D5 arm: `Value::wat_record { .. } => "wat::core::wat_record"`.

5. **Extend `eval_type` at line 14420** with D6 arm: `Value::wat_record { class_fqdn, .. } => class_fqdn.to_string()` — closes Stone 234.0's TODO marker.

6. **Address any OTHER exhaustive match sites cargo surfaces** — the substrate-as-teacher cascade (FM 15). Each surfaced error names a site needing the new arm. Apply per the appropriate per-trait pattern (Eq false for cross-variant pairs; Hash uses discriminant tag; etc.).

**Tests:**

The FM 2-bis probe at `tests/probe_arc234_stone1_wat_record_variant.rs` (committed alongside this BRIEF) — 7 contracts; must flip compile-FAIL → 7/7 PASS.

## What does NOT change

- **No user-facing constructor verb** (defrecord macro is Stone 234.2)
- **No Display impl** — Stone 234.1 uses Rust's auto-derived Debug only; Display for Value is a separate scope (D4)
- **No check.rs registration for `:myapp::*` types** — Stone 234.2 handles defrecord type-system integration
- **HolonAST enum** — unchanged (holon-rs frozen since 530650c per STOP-4)
- **Arc 233 deliverables** — unchanged; regression guards stay GREEN
- **Arc 232.0a / Stone 234.0** — unchanged; regression guards stay GREEN

## Out of scope (affirmative scope-bounding)

- `:wat::core::defrecord` macro (Stone 234.2)
- Record-y polymorphic verbs (`assoc`, `record->map`, `record?`, `record->holon`, keyword-as-accessor) — Stone 234.3
- Hash-destructure — Stone 234.4
- `:wat::holon::*` auto-dispatch on wat-records — Stone 234.5
- Migration sweep + retire `:wat::holon::defrecord` user surface — Stone 234.6
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D10
- Display impl for Value enum (separate arc/stone if pursued; out of D4)
- HolonRepresentable trait impl on Value (per D9, not the right shape)

## Verification flow

```bash
cargo build --release -p wat 2>&1 | tail -5                                                  # 0 errors (cascade addressed)
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5             # 7/7 PASS (LOAD-BEARING)
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5                  # 8/8 PASS (Stone 234.0 guard)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                               # ≥ 827 passed; 0 failed
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3         # 7/7 PASS (Stone 232.0a guard)
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3         # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3                 # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3          # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                   # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                       # empty
```

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- **STOP-1:** unexpected compile errors NOT tracing to the variant addition + impl extensions + dispatch cascade addressing
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (predicted 60-120; apply partial-state-grading per `feedback_partial_state_grading`)
- **STOP-4:** holon-rs touched (frozen since 530650c)
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — defrecord macro, user-facing constructor verb, record-y polymorphic verbs, destructure, Display impl for Value
- **STOP-7:** new probe doesn't compile-clean + flip 7/7 PASS
- **STOP-8:** Stone 234.0 polymorphic type probe regresses (the immediate prior stone)
- **STOP-9:** any arc 233 regression guard regresses
- **STOP-10:** Stone 232.0a typed-entities reflection probe regresses

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **`#[wat_value]` seal acceptance** — variant has three Arc'd field types (Arc<String>, Arc<Vec<Value>>, Arc<HolonAST>). Per arc 233 Stone 233.2.l: forbidden is `Box<Self>/Arc<Self>/Rc<Self>/Self` SINGLE-FIELD wrapping variants. Container variants (Vec<Self> wrapped in Arc) are ALLOWED. wat_record should pass naturally. If proc-macro rejects, use escape hatch `#[wat_value(allow_wrapping = "wat-record carries dual form; struct_form is a container Vec<Value> wrapped in Arc, not a wrapping reference")]` with non-empty reason string. Verify empirically.

2. **Value does NOT currently have Display impl** (verified via grep: only `impl Display for ValueSnapshot` at line 1821). Stone 234.1 must NOT add Display for wat_record alone (asymmetric); use auto-derived Debug. The probe asserts via `format!("{:?}", r)`.

3. **PartialEq + Hash consistency** — Per arc 216 Stone 216.5a: if `a == b` then `hash(a) == hash(b)`. D2's Eq delegates to `class_fqdn == class_fqdn && holon_form == holon_form`; D3's Hash hashes `"wat_record" discriminant + holon_form`. Consistent BY CONSTRUCTION — both delegate to holon_form's value. Verify via probe 5.

4. **Cascade scope creep** — sonnet should ONLY add arms for the new variant. If cargo surfaces a site needing OTHER changes (e.g., refactoring an existing arm), STOP and surface as honest delta. The cascade is "add arm per existing pattern"; not "refactor".

5. **No defrecord macro / no constructor verb** — explicit out-of-scope. If sonnet finds itself reaching toward exposing a wat-level constructor, STOP. The macro is Stone 234.2.

6. **HolonAST::Bundle takes Arc<Vec<HolonAST>>** (per recent fix to the probe; line 64 fix). When the wat_record's holon_form field is set during construction, callers (Stone 234.2's macro, test helpers) must wrap the Vec in Arc::new. Not the variant's concern at Stone 234.1; just noting for future stone authoring.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases or parallel variant names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-234.1.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_arc234_stone1_wat_record_variant.rs` IS the success criterion. Flip compile-FAIL → 7/7 PASS.
- This is the BIGGEST stone in arc 234 so far (variant + cascade). Calibration band 60-120 min Mode A.
- **The substrate-as-teacher cascade IS the work** — cargo enumerates each site; address one at a time; trust the loop.

## Rank-up evidence — CAPTURE IN SCORE

Per the SCORE methodology in EXPECTATIONS, include a Rank-Up Evidence section. The Streetfighter/Helwalker class build means sonnet performs WELL when bloodied (cascade errors stacking up); the substrate-as-teacher pattern IS the empowering condition. Capture cases where:
- Arc 233 + 232.0a + 234.0 tools (ValueSnapshot, Provenance, EDN, type primitive) shortened iteration
- `#[wat_value]` seal provided structural confidence
- Cargo errors surfaced sites efficiently (substrate-as-teacher)
- The cascade depth was navigable per the per-trait pattern

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.1.md` — paired 11-row scorecard
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — predecessor SCORE
- `tests/probe_arc234_stone1_wat_record_variant.rs` — FM 2-bis probe (compile-FAIL initial state verified)
- `src/runtime.rs:710` — `impl PartialEq for Value` (D2 arm location)
- `src/runtime.rs:1089` — `Value::type_name()` (D5 arm location)
- `src/runtime.rs:14420` — `eval_type` TODO marker (D6 location)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — `#[wat_value]` seal documentation
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
