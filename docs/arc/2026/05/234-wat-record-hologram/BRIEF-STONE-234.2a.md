# BRIEF — Arc 234 Stone 234.2a — `:wat::Record::of` + `/field-at` substrate primitives

## What we're doing

Mint two substrate primitives that the Stone 234.2b defrecord macro will consume:

- **`:wat::Record::of`** — constructor: `(class-fqdn struct-form holon-form) -> record`
- **`:wat::Record/field-at`** — positional accessor: `(record index) -> field-value`

Plus register `:wat::Record` as a check.rs-known opaque type so signatures can declare `[v <- :wat::Record]`.

Stone 234.0 minted `:wat::core::type` (substrate primitive). Stone 234.1 minted `Value::wat__Record` variant (storage form). Stone 234.2a mints the **substrate verbs that construct and access** wat_record instances. Stone 234.2b (next) will build the defrecord macro that consumes these verbs.

This is the substrate-then-macro pattern — same shape as arc 232 (Stone 232.0 minted `:wat::core::apply` primitive; Stone 232.1 builds defprotocol macro on top).

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/234-record-hologram/DESIGN-STONE-234.2a.md`** — sub-DESIGN with 10 locked decisions (D1-D10). **The signatures (D2, D3) are non-negotiable**; the dispatch keyword names are canonical (D6, D10).

2. **`tests/probe_arc234_stone2a_wat_record_primitives.rs`** — FM 2-bis probe (already committed at `db39ebd`). Currently 7/7 FAIL with `UnknownFunction(":wat::Record::of")`. **The probe IS the success criterion** — flip 7/7 FAIL → 7/7 PASS.

3. **`docs/arc/2026/05/234-record-hologram/DESIGN.md`** — arc 234 umbrella; the hologram thesis context.

4. **`docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.0.md`** — predecessor SCORE (`:wat::core::type` primitive); same substrate-primitive shape.

5. **`docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.1.md`** — variant-minting predecessor SCORE; shows the variant fields record/of populates.

6. **`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md`** — apply primitive precedent; same substrate-then-macro pattern (apply ships substrate verb; defprotocol macro consumes it).

7. **`src/runtime.rs:14421`** — `eval_type` fn (predecessor pattern; how Stone 234.0 added a new substrate primitive: eval fn + dispatch arm + TypeScheme).

8. **`src/runtime.rs` (Stone 234.1 commit `5abf714`)** — `Value::wat__Record` variant; the storage form record/of populates + field-at reads.

9. **`src/check.rs`** — TypeScheme registrations + TypeDef registration patterns. The `:wat::core::String` and `:wat::holon::HolonAST` primitive type registrations are precedents for opaque-umbrella `:wat::Record` registration.

## Implementation surface

**`src/runtime.rs`:**

1. **New `fn eval_wat_record_of`** (~30-40 lines) — arity check (3 args); evaluate args; extract:
   - `class_fqdn: String` from `Value::String(s)` arg 0 — strip leading `:` if present (per D7)
   - `struct_form: Arc<Vec<Value>>` from `Value::Vec(arc_vec)` arg 1 — clone the Arc (do NOT re-wrap)
   - `holon_form: Arc<HolonAST>` from `Value::holon__HolonAST(arc_h)` arg 2 — clone the Arc

   Construct + return:
   ```rust
   Value::wat__Record {
       class_fqdn: Arc::new(class_fqdn_clean),
       struct_form: arc_vec,
       holon_form: arc_h,
   }
   ```

2. **New `fn eval_wat_record_field_at`** (~25-35 lines) — arity check (2 args); evaluate args; extract:
   - `record` from `Value::wat__Record { struct_form, .. }` arg 0
   - `index: i64` from `Value::i64(n)` arg 1
   - Bounds-check; if out of bounds, return `RuntimeError::IndexOutOfBounds` per existing Vector/get pattern
   - Return `struct_form[index as usize].clone()`

3. **Two new dispatch arms in `dispatch_keyword_head_value`** (or equivalent dispatcher):
   ```rust
   ":wat::Record::of"        => eval_wat_record_of(args, list_span, env, sym),
   ":wat::Record/field-at"  => eval_wat_record_field_at(args, list_span, env, sym),
   ```

**`src/check.rs`:**

1. **TypeDef registration for `:wat::Record`** (~5-10 lines) — mirror existing primitive type registration (e.g., how `:wat::core::String`, `:wat::holon::HolonAST` are registered). This is an OPAQUE type at check.rs level — per-class typing (`:myapp::Voltage` etc.) is Stone 234.2b's work.

2. **Two TypeScheme registrations in `register_builtins`** (~20 lines):

   ```rust
   // :wat::Record::of
   register(":wat::Record::of", TypeScheme {
       type_params: vec!["T".into()],
       params: vec![
           TypeExpr::Path(":wat::core::String".into()),
           TypeExpr::Parametric { head: "wat::core::Vector".into(), args: vec![t_var()] },
           TypeExpr::Path(":wat::holon::HolonAST".into()),
       ],
       ret: TypeExpr::Path(":wat::Record".into()),
       rest_param_type: None,
   });

   // :wat::Record/field-at
   register(":wat::Record/field-at", TypeScheme {
       type_params: vec!["T".into()],
       params: vec![
           TypeExpr::Path(":wat::Record".into()),
           TypeExpr::Path(":wat::core::i64".into()),
       ],
       ret: t_var(),
       rest_param_type: None,
   });
   ```

   Use existing helpers (`t_var()`, `TypeExpr::Path`, etc.) per the actual check.rs surface — sonnet should read existing registrations to mirror exact patterns rather than guess.

**Tests:**

The FM 2-bis probe at `tests/probe_arc234_stone2a_wat_record_primitives.rs` (committed at `db39ebd`) — 7 contracts; must flip 7/7 FAIL → 7/7 PASS.

## What does NOT change

- **No defrecord macro** (Stone 234.2b is the next stone)
- **No per-class type registration** — `:myapp::Voltage` etc. are 234.2b's work
- **No user-facing constructor verbs** (`:myapp::Voltage` shorthand) — 234.2b
- **No accessor verbs** (`:myapp::Voltage/magnitude`) — 234.2b emits these via macro
- **No polymorphic record-y verbs** (`assoc`, `record->map`, `record?`, keyword-as-accessor) — 234.3
- **HolonAST enum** — unchanged (holon-rs frozen since 530650c per STOP-4)
- **`Value::wat__Record` variant** — unchanged from Stone 234.1; this stone CONSUMES it, doesn't modify it
- **Arc 233 deliverables** — unchanged; regression guards stay GREEN
- **Arc 232.0a / Stone 234.0 / Stone 234.1** — unchanged; regression guards stay GREEN

## Out of scope (affirmative scope-bounding)

- `:wat::core::defrecord` macro (Stone 234.2b)
- Per-class type registration (`:myapp::Voltage` as `:wat::Record` alias with class_fqdn invariant)
- User-facing constructor verbs (`:myapp::Voltage`)
- Predicates (`:myapp::is-Voltage?`)
- Named per-field accessors (`:myapp::Voltage/magnitude`)
- Record-y polymorphic verbs (Stone 234.3)
- Hash-destructure (Stone 234.4)
- `:wat::holon::*` auto-dispatch on records (Stone 234.5)
- Migration sweep + retire `:wat::holon::defrecord` user surface (Stone 234.6)
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D10

## Verification flow

```bash
cargo build --release -p wat 2>&1 | tail -5                                                       # 0 errors
cargo test --release --test probe_arc234_stone2a_wat_record_primitives 2>&1 | tail -5              # 7/7 PASS (LOAD-BEARING)
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5                  # 7/7 PASS (Stone 234.1 guard)
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5                       # 8/8 PASS (Stone 234.0 guard)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                                    # ≥ 827 passed; 0 failed
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3              # 7/7 PASS (Stone 232.0a guard)
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3                     # 5/5 PASS
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3              # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3                      # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3                     # 5/5 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                        # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                            # empty
```

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- **STOP-1:** unexpected compile errors NOT tracing to the new primitives + type registration
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **90 min elapsed** (predicted 30-60; apply partial-state-grading per `feedback_partial_state_grading`)
- **STOP-4:** holon-rs touched (frozen since 530650c)
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — defrecord macro, per-class type registration, user-facing constructor verbs, predicates, named accessors
- **STOP-7:** new probe doesn't compile-clean + flip 7/7 PASS
- **STOP-8:** Stone 234.1 wat_record variant probe regresses
- **STOP-9:** Stone 234.0 polymorphic type probe regresses
- **STOP-10:** Stone 232.0a typed-entities reflection probe regresses
- **STOP-11:** any arc 233 regression guard regresses

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **`Value::Vec` Arc-ownership** — when extracting `struct_form` arg from `Value::Vec(arc_vec)`, CLONE the existing Arc (cheap refcount bump); do NOT re-wrap into a fresh `Arc::new(arc_vec.to_vec())`. Pattern precedent: existing primitives that accept Vector args.

2. **HolonAST extraction** — `holon_form` arg arrives as `Value::holon__HolonAST(Arc<HolonAST>)`. Standard extraction pattern from arc 232.0a + holon-side primitives.

3. **String class_fqdn extraction** — `Value::String(Arc<String>)` — standard. Leading-colon strip per D7: `class_fqdn_input.trim_start_matches(':').to_string()`.

4. **Polymorphic generic-T return on `record/field-at`** — TypeScheme uses `ret: t_var()`. The probe's wat-source uses recipient inference (let-binding or defn-return) to drive T's unification. If the type-checker can't infer T in the probe contexts, address by reading existing polymorphic primitives' check.rs handling (e.g., how `Vec/get` returns T) — DO NOT add custom apply-style annotation syntax. Just propagate inference correctly.

5. **TypeDef registration for `:wat::Record`** — verify the registration approach matches existing primitive types. Likely a simple `register_type_alias` or similar; investigate the existing pattern in check.rs before authoring.

6. **`#[wat_value]` seal on Value variant** — Stone 234.1 already passed the seal for `Value::wat__Record`. Stone 234.2a does NOT modify Value; just adds eval fns that construct an existing variant. No seal concerns.

7. **No defrecord macro / no constructor verb / no per-class typing** — explicit out-of-scope. If sonnet finds itself reaching toward exposing a wat-level constructor `:myapp::Voltage` OR registering per-class types, STOP. The macro is Stone 234.2b.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases, no parallel primitive names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-234.2a.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_arc234_stone2a_wat_record_primitives.rs` IS the success criterion. Flip 7/7 FAIL → 7/7 PASS.
- Calibration band 30-60 min Mode A; 90 STOP-3.
- **Substrate-as-teacher cascade should be shallow** — new fns + new dispatch arms + new TypeSchemes + 1 type registration. NOT a variant addition. If cascade depth surprises (>5 sites), surface as honest delta in SCORE.

## Rank-up evidence — CAPTURE IN SCORE

Per the SCORE methodology in EXPECTATIONS, include a Rank-Up Evidence section. The substrate-as-teacher cascade IS the empowering condition for the Helwalker/Streetfighter build. For Stone 234.2a specifically, the cascade should be shallow (no variant addition; just new fns + new dispatch arms). Capture cases where:

- Stone 234.0's eval_type precedent shortened authoring time
- Stone 234.1's Value::wat__Record variant fields were straight-forward to populate
- Stone 232.0's apply primitive precedent shaped the dispatch arm signature
- `#[wat_value]` seal stayed quiet (no variant changes; no seal concerns)
- Existing primitive registration patterns (`:wat::core::String`, etc.) made TypeDef registration mechanical

## Cross-references

- `docs/arc/2026/05/234-record-hologram/DESIGN-STONE-234.2a.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-record-hologram/EXPECTATIONS-STONE-234.2a.md` — paired scorecard
- `docs/arc/2026/05/234-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.1.md` — variant-minting predecessor SCORE
- `docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.0.md` — type-primitive predecessor SCORE
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent (substrate-then-macro)
- `tests/probe_arc234_stone2a_wat_record_primitives.rs` — FM 2-bis probe (7 contracts; 7/7 FAIL initial verified)
- `src/runtime.rs:14421` — `eval_type` fn (substrate primitive precedent)
- `src/check.rs` — TypeScheme + TypeDef registration patterns
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
