# BRIEF — Arc 234 Stone 234.1.5 — variant rename + `:wat::record` namespace promotion

## What we're doing

Rename Stone 234.1's `Value::wat_record` → `Value::wat__record` (per arc 109 `__` FQDN convention; matches `Value::wat__core__Uuid`/`wat__core__Char`/`wat__std__HashMap` family) and register `:wat::record::Record` as opaque umbrella type in check.rs — promoting the record concept-cluster to top-level `:wat::record::*` namespace (peer of `:wat::holon::*`/`:wat::kernel::*`/`:wat::config::*`/`:wat::test::*`).

NO new primitives. NO new behavior. Pure rename + type registration + cascade update.

Stone 234.1.5 is the **stepping-stone foundation** that subsequent stones (234.2a substrate primitives, 234.2b defrecord macro, 234.3 polymorphic verbs, 234.4-6) operate on. Per recovery doc § 5 + arc 157 1a-i precedent: settling foundation FIRST means six future stones operate on settled substrate with clean per-stone verification.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.5.md`** — sub-DESIGN with 10 locked decisions (D1-D10). The variant rename target (D1) + type_name return (D2) + Hash discriminant (D4) + type registration (D5) are non-negotiable.

2. **`tests/probe_arc234_stone15_namespace_promotion.rs`** — FM 2-bis probe (committed forthcoming). Currently 5/5 FAIL with `error[E0599]: no variant named 'wat__record' found for enum 'wat::Value'`. **The probe IS the success criterion** — flip 5/5 FAIL → 5/5 PASS.

3. **`tests/probe_arc234_stone1_wat_record_variant.rs`** — Stone 234.1's regression guard. β.i updates the `make_record` helper to construct `Value::wat__record { ... }` (variant rename cascades into the probe). 7 contracts stay GREEN — they verify the renamed variant's behavior is identical.

4. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN.md`** — arc 234 umbrella (has been pivoted to reflect `:wat::record::*` namespace; the sub-DESIGN cites the pivot).

5. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md`** — predecessor SCORE (variant + cascade shipped). β.i's rename preserves all semantics.

6. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md`** — Stone 234.0's polymorphic type primitive. The wat__record arm of `eval_type` preserves through rename per D3.

7. **`src/runtime.rs:651`** — `Value::wat_record { ... }` variant definition (D1 target).

8. **`src/runtime.rs:813,816-817,1026-1027,1029-1030,1177,14460,14485,18216,18218`** — all 14 substrate sites (D1, D2, D4, D6 cascade).

9. **`src/edn_shim.rs:1694,1697`** — 2 sites (D8 cascade).

10. **`src/closure_extract.rs:1716,1718-1719`** — 2 sites (D8 cascade).

11. **`src/check.rs`** — TypeDef registration mechanism. Mirror existing primitive type registration (e.g., how `:wat::core::String` or `:wat::holon::HolonAST` are registered). NEW site: register `:wat::record::Record` as opaque type (D5).

## Implementation surface

**`src/runtime.rs` (14 sites):**

1. **Line 651: variant rename.**
   ```rust
   // BEFORE:
   wat_record {
       class_fqdn: Arc<String>,
       struct_form: Arc<Vec<Value>>,
       holon_form: Arc<HolonAST>,
   }

   // AFTER:
   wat__record {
       class_fqdn: Arc<String>,
       struct_form: Arc<Vec<Value>>,
       holon_form: Arc<HolonAST>,
   }
   ```

2. **Lines 816-817: Eq arm patterns.** `Value::wat_record { ... }` → `Value::wat__record { ... }` (both halves of the pair-pattern; the arm body unchanged).

3. **Line 1029: Hash arm pattern.** `Value::wat_record { ... }` → `Value::wat__record { ... }`.

4. **Line 1030: Hash discriminant tag string.** `"wat_record"` → `"wat__record"` (D4 — honest tag matching the variant identifier).

5. **Line 1177: type_name arm.**
   ```rust
   // BEFORE:
   Value::wat_record { .. } => "wat::core::wat_record",

   // AFTER (D2):
   Value::wat__record { .. } => "wat::record",
   ```

6. **Line 14485: eval_type arm pattern (D3 — body logic preserved verbatim).**
   ```rust
   Value::wat__record { class_fqdn, .. } => class_fqdn.to_string(),
   ```

7. **Line 18218: render_value arm pattern.** `Value::wat_record { ... }` → `Value::wat__record { ... }` (body unchanged).

8. **Lines 813, 1026-1027, 14460, 18216: doc-comment updates.** Variant references in comments — sonnet updates the `wat_record:` references to `wat__record:`. Historical "Arc 234 Stone 234.1" prefix STAYS (history is honest — Stone 234.1 minted the variant; β.i renames it).

**`src/edn_shim.rs` (2 sites):**

9. **Line 1697: render arm pattern.** `Value::wat_record { ... }` → `Value::wat__record { ... }`.
10. **Line 1694: render comment update.**

**`src/closure_extract.rs` (2 sites):**

11. **Lines 1718-1719: cascade arm + error message.** `Value::wat_record { .. }` → `Value::wat__record { .. }`; error string updates per honesty.
12. **Line 1716: cascade comment.**

**`src/check.rs` (1 site — NEW):**

13. **Register `:wat::record::Record` as opaque primitive TypeDef.** Mirror existing primitive type registration pattern (e.g., how `:wat::core::String`, `:wat::holon::HolonAST`, `:wat::kernel::Sender` are registered as opaque types). Look at how those four are registered + apply the same shape for `:wat::record::Record`.

**`tests/probe_arc234_stone1_wat_record_variant.rs` (~7 sites — helper + 7 test fns):**

14. **`make_record` helper:** construct `Value::wat__record { ... }` (line ~78 — variant pattern updated).
15. **Test bodies (probes 1, 2, 3, 4, 5, 6, 7):** match patterns `Value::wat__record { ... }`.
16. **Doc-comment headers in probe file:** `wat_record variant` → `wat__record variant` (honesty in test docs).

## What does NOT change

- **No new substrate primitives** — `:wat::record::of`, `:wat::record::field-at`, `:wat::record::def`, `:wat::record::is?`, `:wat::record::to-map` are Stone 234.2a (β.iii)
- **defrecord macro** — Stone 234.2b
- **Per-class type registration** (`:myapp::Voltage` as `:wat::record::Record` alias) — Stone 234.2b
- **Polymorphic verb extensions** — Stone 234.3
- **`:wat::holon::to-holon` wat__record arm extension** — later stone
- **Stone 234.2a in-flight artifacts at `db39ebd` + `7113c51`** — β.ii orchestrator paperwork AFTER β.i ships
- **HolonAST enum** — unchanged (holon-rs frozen since `530650c` per STOP-4)
- **Arc 233 deliverables** — unchanged; regression guards stay GREEN
- **Stone 234.0 / Stone 234.1's observable behavior** — preserved verbatim; this stone changes NAMES only

## Out of scope (affirmative scope-bounding)

- Any new substrate primitive — Stone 234.2a
- defrecord macro — Stone 234.2b
- Per-class type registrations — Stone 234.2b
- Polymorphic verb extensions — Stone 234.3
- Hash-destructure — Stone 234.4
- `:wat::holon::to-holon` auto-dispatch on wat__records — Stone 234.5
- Migration sweep — Stone 234.6
- Stone 234.2a in-flight artifacts revision (β.ii orchestrator paperwork)
- holon-rs — STOP-4
- Parallel API / aliases — HARD CUT per D10

## Verification flow

```bash
cargo build --release -p wat 2>&1 | tail -5                                                                # 0 errors
cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 | tail -5                         # 5/5 PASS (LOAD-BEARING)
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5                           # 7/7 PASS (regression guard updated by β.i)
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5                                # 8/8 PASS (Stone 234.0)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                                             # ≥ 827 passed; 0 failed
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3                       # 7/7 PASS (Stone 232.0a)
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3                              # 5/5 PASS
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3                       # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3                               # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3                              # 5/5 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                                 # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                                     # empty
grep -rE "Value::wat_record|\\bwat_record\\b" src/ tests/ 2>/dev/null | grep -v "Arc 234 Stone 234.1\\b" | wc -l   # 0 (zero leftover references except historical Stone 234.1 prefix comments)
```

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- **STOP-1:** unexpected compile errors NOT tracing to the variant rename + cascade addressing + type registration
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **45 min elapsed** (predicted 15-30; apply partial-state-grading per `feedback_partial_state_grading`)
- **STOP-4:** holon-rs touched (frozen since `530650c`)
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — new primitives, defrecord macro, per-class types, polymorphic verbs
- **STOP-7:** FM 2-bis probe doesn't compile-clean + flip 5/5 PASS
- **STOP-8:** Stone 234.1 regression guard (probe_arc234_stone1_wat_record_variant) regresses post-rename
- **STOP-9:** Stone 234.0 polymorphic type probe regresses
- **STOP-10:** Stone 232.0a typed-entities reflection probe regresses
- **STOP-11:** any arc 233 regression guard regresses

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **Stone 234.1's probe MUST stay GREEN** through the rename. β.i updates the helper + match patterns; if any contract fails post-rename, that's a behavior change masquerading as a rename. STOP-8 fires.

2. **String literal updates carry semantic weight.** Line 1030 (`"wat_record"` Hash discriminant) and line 1177 (`"wat::core::wat_record"` type_name return) are both visible — Hash discriminant prevents cross-variant collisions; type_name is what consumers observe. Both update for honesty (D2 + D4).

3. **type_name() string change is observable** — any wat-level test asserting on `"wat::core::wat_record"` would break. Empirically check with grep: `grep -rE "wat::core::wat_record|wat_record\"" src/ tests/`. If any non-source-code reference exists (e.g., in a test fixture string), surface as honest delta.

4. **check.rs TypeDef registration mechanism** — investigate the existing pattern (e.g., how `:wat::holon::HolonAST` is registered as an opaque primitive type) BEFORE authoring. The grep pattern: `grep -nE "register_type|register_alias|TypeDef::" src/check.rs | head -30` should surface the canonical registration call shape. Mirror it.

5. **`#[wat_value]` seal verification** — Stone 234.1's variant passed the seal (container with three Arc'd fields). The rename doesn't change the shape; seal stays passive. No new escape hatch needed.

6. **Cascade scope creep** — sonnet should ONLY update the 18 known src/ sites + register the umbrella type + update the Stone 234.1 probe. If cargo surfaces a site needing OTHER changes (e.g., a wat-level test fixture asserting on `"wat::core::wat_record"`), surface as honest delta. Do NOT silently expand scope. STOP-6 fires.

7. **Stone 234.2a in-flight artifacts at `db39ebd` + `7113c51`** — IGNORE THESE. They reference `wat-record` and `:wat::core::wat-record/of`. They are pre-pivot working artifacts; orchestrator (β.ii) revises after β.i ships. Do NOT touch them. If sonnet sees them and is confused, IGNORE — they're not active substrate.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases or parallel variant names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-234.1.5.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_arc234_stone15_namespace_promotion.rs` IS the success criterion. Flip 5/5 FAIL → 5/5 PASS.
- Stone 234.1's probe at `tests/probe_arc234_stone1_wat_record_variant.rs` MUST stay GREEN through the rename (helper + match patterns updated; semantics preserved).
- Calibration band 15-30 min Mode A; 45 STOP-3. **This is the smallest stone of arc 234 so far** — pure rename.
- The substrate-as-teacher cascade IS shallow (18 sites enumerated empirically); cargo names every leftover.

## Rank-up evidence — CAPTURE IN SCORE

Per the SCORE methodology in EXPECTATIONS, include a Rank-Up Evidence section. For Stone 234.1.5 specifically, the predecessor stones' tools should make this clean:

- Stone 234.1's probe pattern (make_record helper) is the template
- The `__` FQDN convention from arc 109 is the canonical example pattern
- The substrate-as-teacher cascade should be SHALLOW (18 sites, mechanical)
- `#[wat_value]` seal stays quiet (no variant shape change)
- check.rs TypeDef registration mirrors existing primitive types

Capture concrete cases where these tools fired during iteration.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.5.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.1.5.md` — paired scorecard
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (post-pivot)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — variant-minting predecessor SCORE
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — type-primitive predecessor SCORE
- `tests/probe_arc234_stone15_namespace_promotion.rs` — FM 2-bis probe (5 contracts; 5/5 FAIL initial verified)
- `tests/probe_arc234_stone1_wat_record_variant.rs` — Stone 234.1's regression guard (β.i updates variant pattern; stays GREEN)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
- `feedback_no_broken_commits.md` — green tree on disk discipline (β.i ships green; unblocks β.ii + β.iii)
