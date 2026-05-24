# BRIEF — Arc 234 Stone 234.0 — `:wat::core::type` polymorphic primitive

## What we're doing

Mint ONE wat-callable substrate primitive — `:wat::core::type` — that extracts a Value's record-type FQDN as a String, polymorphically over every Value variant. This is the smallest substrate addition in arc 234 and the prerequisite for the revised Stone 232.1 polymorphic defprotocol + all subsequent arc 234.x stones.

```
(:wat::core::type <any-value>) -> :wat::core::String
```

After this stone: defprotocol's dispatcher (Stone 232.1 revised) can extract the receiver's type FQDN regardless of storage backend (HolonAST classifier-wrap, struct, primitive, etc.); arc 234's downstream stones (defrecord, record-y verbs, hash-destructure) all consume the same primitive for dispatch routing.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md`** — sub-DESIGN with 6 locked decisions (D1-D6) + dispatch table + trap-door audit. **The dispatch table (D2) is non-negotiable**:
   - `Value::holon__HolonAST(h)` → `extract_classifier(h).unwrap_or_else(|| "wat::holon::HolonAST".to_string())`
   - `Value::Struct(sv)` → `sv.type_name.trim_start_matches(':').to_string()`
   - Any other Value → `Value::type_name().to_string()`

2. **`tests/probe_diagnostic_polymorphic_type.rs`** — FM 2-bis probe (commit `529760b`). 8 contracts; currently 8/8 FAIL with `UnknownFunction(":wat::core::type")`. **The probe IS the success criterion** — flip 0/8 → 8/8.

3. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN.md`** — arc 234 umbrella; the hologram thesis context (why `:wat::core::type` matters).

4. **`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md`** — apply primitive precedent. TypeScheme + dispatch arm + eval fn shape pattern.

5. **`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md`** — extract-classifier lift precedent. Small wat-callable primitive shape.

## Implementation surface

**`src/runtime.rs`:**

1. New `fn eval_type` — accepts args + list_span + env + sym; arity check (1 arg; ArityMismatch on other); evaluates the arg (via `eval_inner(...)?.value_owned()`); matches on the resulting Value variant per D2 dispatch table; returns `Value::String(Arc::new(type_str))`. Doc comment notes arc 234.1 will add a `Value::wat_record` arm.

2. New dispatch arm in `dispatch_keyword_head_value`:
   ```rust
   ":wat::core::type" => eval_type(args, list_span, env, sym),
   ```
   Place alongside other `:wat::core::*` runtime primitives (recommend near `:wat::core::apply` per Stone 232.0 precedent).

**`src/check.rs`:**

1. New entry in `register_builtins`:
   ```rust
   env.register(
       ":wat::core::type".into(),
       TypeScheme {
           type_params: vec!["T".into()],
           params: vec![TypeExpr::Var("T".into())],
           ret: TypeExpr::Path(":wat::core::String".into()),
           rest_param_type: None,
       },
   );
   ```
   Follow apply's TypeScheme pattern (Stone 232.0) for "accepts any value, returns concrete type."

2. May need `infer_list` special-case if the generic T-var doesn't propagate correctly through inference for this primitive — verify via cargo test; if probe surfaces an inference error, mirror `infer_apply`'s pattern.

## What does NOT change

- HolonAST enum — unchanged (holon-rs frozen since 530650c per STOP-4)
- Value enum — sealed by `#[wat_value]`; no new variants in this stone
- `Value::type_name()` Rust method — consumed as-is; if a probe surfaces a non-FQDN return for any variant, that's a separate stone (likely arc 109 follow-up)
- `extract_classifier` Rust fn — consumed as-is; no modifications
- Arc 233 deliverables — unchanged; regression guards stay GREEN
- Arc 232.0a — unchanged; regression guard stays GREEN

## Out of scope (affirmative scope-bounding)

- `Value::wat_record` variant — Stone 234.1 (variant doesn't exist yet; type primitive's dispatch table will gain one arm there)
- defrecord macro at `:wat::core::` — Stone 234.2
- Record-y polymorphic verbs (assoc, record->map, record?, record->holon, keyword-as-accessor) — Stone 234.3
- Hash-destructure — Stone 234.4
- `:wat::holon::*` auto-dispatch on wat-records — Stone 234.5
- Migration sweep + retire `:wat::holon::defrecord` user surface — Stone 234.6
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D5
- Type-checking semantics (`is?` family) — already shipped per arc 226

## Verification flow

```bash
cargo build --release -p wat 2>&1 | tail -5                                                  # 0 errors
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5                  # 8/8 PASS (LOAD-BEARING)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                               # ≥ 827 passed; 0 failed
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3         # 7/7 PASS (Stone 232.0a guard)
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3         # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3                 # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3          # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                   # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                       # empty output
```

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- **STOP-1:** unexpected compile errors NOT tracing to the new `eval_type` / dispatch arm / TypeScheme entry
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **90 min elapsed** (predicted 30-60; apply partial-state-grading per `feedback_partial_state_grading`)
- **STOP-4:** holon-rs touched (frozen since 530650c)
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — `Value::wat_record` variant, macro work, record-y verbs, destructure
- **STOP-7:** new probe `probe_diagnostic_polymorphic_type` doesn't flip 0/8 → 8/8
- **STOP-8:** any arc 233 regression guard regresses (the rank-up substrate must STAY working)
- **STOP-9:** Stone 232.0a typed-entities reflection probe regresses

## Trap-door audit (per FM 2-bis BRIEF discipline)

- **`Value::type_name()` returns generic `"wat::core::Struct"` for ALL struct instances** — confirmed via grep. D2's struct arm extracts the per-instance FQDN from `StructValue.type_name` field, NOT via `type_name()`. The leading `:` in `sv.type_name` (e.g., `":myapp::Point"`) gets stripped via `trim_start_matches(':')` for consistency with `extract_classifier` convention (FQDN without leading colon).

- **HolonAST classifier-wrap detection** — `extract_classifier(holon)` returns `Option<String>` per arc 226+227. The fallback when extract_classifier returns None (e.g., non-wrap HolonAST: bare Atom, Bundle without classifier wrap) is the variant name `"wat::holon::HolonAST"`. This is honest — non-classifier-wrapped HolonAST values DO have type "wat::holon::HolonAST" at the substrate level.

- **Polymorphic TypeScheme inference** — `:wat::core::type` accepts ANY value. The TypeScheme uses `TypeExpr::Var("T")` for the param. Apply primitive (Stone 232.0 `infer_apply`) is the precedent if `infer_list` needs special-case handling. Verify via cargo test; if generic T doesn't propagate through ordinary inference, add an `infer_list` arm that returns `:wat::core::String` regardless of arg type.

- **`Value::Vec` vs `Value::Vector`** — TWO distinct variants exist. `Value::Vec(_) => "wat::core::Vector"` (the wat-level Vector); `Value::Vector(_) => "wat::holon::Vector"` (the holon-level VSA Vector). The probe uses `[1 2 3]` literal which creates `Value::Vec` (per arc 215 collection literal inference). Both arms work via the `Value::type_name()` fall-through — no special handling needed.

- **NO new substrate beyond the one primitive** — `eval_type` should be ~30 lines. If you find yourself adding helpers in `src/runtime.rs` beyond `eval_type` itself, STOP and consider whether the extra logic is justified. The pattern is: one `match` on Value variants returning the type string.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases or parallel verb names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-234.0.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_diagnostic_polymorphic_type.rs` IS the success criterion. Flip 0/8 → 8/8.
- This is the SMALLEST stone in arc 234 — calibration band 30-60 min Mode A. The pattern is well-precedented (apply primitive Stone 232.0 ~30 min).

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md` — sub-DESIGN with 6 locked decisions + dispatch table + calibration prediction
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.0.md` — paired 11-row scorecard
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `tests/probe_diagnostic_polymorphic_type.rs` — FM 2-bis probe (commit `529760b`)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — extract-classifier lift precedent
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
