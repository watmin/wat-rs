# EXPECTATIONS — Arc 215 Stone 1 — `Infer` + literal completion

Mode A target: 22/22 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `:wat::type::Infer` registered | New type-keyword in registered list; `parse_type_expr(":wat::type::Infer")` returns Ok |
| 2 | `Infer` resolves to fresh type variable | TypeExpr-level representation that integrates with HM unification (either a dedicated `TypeExpr::Infer` variant or direct fresh-variable substitution) |
| 3 | `infer_hashmap_constructor` accepts `Infer` for K | Detects `:wat::type::Infer` in `args[0]`; sets `k_ty = fresh.fresh()`; doesn't error |
| 4 | `infer_hashmap_constructor` accepts `Infer` for V | Detects `:wat::type::Infer` in `args[1]`; sets `v_ty = fresh.fresh()`; doesn't error |
| 5 | `infer_hashset_constructor` accepts `Infer` for T | Detects `:wat::type::Infer` in `args[0]`; sets `t_ty = fresh.fresh()`; doesn't error |
| 6 | `{...}` desugar updated | Parser emits `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer :k v)` — no `:wat::holon::Atom` wrap on values |
| 7 | `#{...}` parser dispatch added | New rule produces `(:wat::core::HashSet :wat::type::Infer x y z ...)`; `#{` token discriminator |
| 8 | Empty `{}` works | Desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer)`; type-check accepts; length-0 at runtime |
| 9 | Empty `#{}` works | Desugars to `(:wat::core::HashSet :wat::type::Infer)`; type-check accepts; length-0 at runtime |
| 10 | Probe 1 — `{:foo 42}` single-pair inference | length 1; get :foo → Some(42) as i64 (NOT HolonAST-wrapped) |
| 11 | Probe 2 — `{:a 1 :b 2 :c 3}` multi-pair | length 3; get :b → Some(2); all values share inferred i64 V |
| 12 | Probe 3 — `{:a "hello" :b "world"}` string-valued | length 2; get :a → Some("hello"); V inferred as String |
| 13 | Probe 4 — `{:outer {:inner 42}}` nested | Outer V inferred as HashMap<keyword,i64>; outer length 1; type-checks AND runtime-works — Probe 5 resolution |
| 14 | Probe 5 — mixed-value-type rejection | `{:a 1 :b "two"}` fails at check with TypeMismatch; diagnostic names the offending value's span |
| 15 | Probe 6 — empty `{}` length 0 | Type-check passes with fresh K, V; length-0 confirmed at runtime |
| 16 | Probe 7 — `#{42}` single element | length 1; contains 42 returns true |
| 17 | Probe 8 — `#{1 2 3}` multi element | length 3; contains 2 returns true; T inferred i64 |
| 18 | Probe 9 — `#{1 1 2 2 3}` dedup | length 3 (construction dedups); same T inference |
| 19 | Probe 10 — mixed-element-type set rejection | `#{1 :foo "x"}` fails at check with TypeMismatch |
| 20 | Probe 11 — map of sets | `{:a #{1 2} :b #{3 4}}` — outer V = HashSet<i64>; both inner length 2 |
| 21 | WAT-CHEATSHEET § 8 updated | New `{...}` / `#{...}` desugar rows; `Infer` row added; explicit verb-form continues |
| 22 | P2 SCORE retroactive amendment | Probe 5 LIMITATION resolution section appended (historical record preserved) |

## Independent prediction (calibration record)

Recorded before sonnet completes; pre-spawn.

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**2× upper-bound cap:** 240 min (would clamp to 60 min via ScheduleWakeup runtime ceiling; check at 60 min)
**Confidence:** medium

**Rationale:**
- P1 calibration: predicted 30-50 min, actual ~20 min (under)
- P2 calibration: predicted 20-35 min, actual ~45 min (over due to D2 cross-cut)
- This stone is BIGGER than both — substrate change (new type-placeholder) + two inference-function extensions + parser desugar adjustment + new `#{...}` dispatch + 12-probe matrix + retroactive amendment + 3 doc updates
- The `Infer` integration with HM unification could surface subtleties (STOP-1 territory)
- Risk factors widening the band: TypeExpr integration with apply_subst / format_type / parse_type_expr / type registry; cross-cutting ripples in walkers

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]
- If overrun: where? [TBD]

## Out-of-scope rows (deliberately absent)

- `[...]` retarget (existing WatAST::Vector path unchanged)
- `'(...)` list literal
- Match-arm patterns
- WARD-PASS (out-of-zone)
- INTERSTITIAL (orchestrator-direct)
- Atom signature changes
- Backporting Vector to `Infer` form

## Honesty deltas accepted

Sonnet may surface deltas if encountered:
- `Infer` integration requires more substrate changes than anticipated (e.g., TypeExpr variant addition rippling across N walkers). Surface honestly; deliver what's feasible; flag the rest.
- HM unification interaction with `Infer` surprises. Honest deltas in SCORE.
- Probe 5's update (success vs LIMITATION) may need probe-file rename or test-name update.
- Cross-cuts to other check.rs paths that depend on the old desugar shape. Document; fix if scoped; defer if not.
