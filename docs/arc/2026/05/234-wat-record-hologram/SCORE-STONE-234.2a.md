# SCORE — Arc 234 Stone 234.2a — `:wat::Record::of` + `:wat::Record/field-at` substrate primitives

**Status:** COMPLETE (2026-05-24)
**Result:** 13/13 PASS — Mode A target met

---

## 13-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **New probe 6/6 PASS** (LOAD-BEARING) | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 3 | Stone 234.1.5 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 4 | Stone 234.1 regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 5 | Stone 234.0 regression guard | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed` |
| 6 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 7 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 8 | Stone 233.3 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 9 | Stone 233.2.e regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 10 | Stone 233.2.l regression guard | `3 passed; 0 failed` | `test result: ok. 3 passed; 0 failed` |
| 11 | Stone 233.2.k regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 12 | Clippy no new warnings | ≤ 54 | `54` (exactly at ceiling; no regression) |
| 13 | holon-rs untouched | empty | empty output (STOP-4 clean) |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 17.73s

# Row 2 — LOAD-BEARING
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -5
test probe_3_struct_form_field_at_zero ... ok
test probe_7_equality_via_holon_form ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 3
cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 | tail -5
test probe_5_class_fqdn_extraction_post_rename ... ok
test probe_4_namespace_type_registration ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 4
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5
test probe_6_debug_contains_class ... ok
test probe_7_type_name_returns_generic_kind ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 5
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5
test probe_1_type_on_i64 ... ok
test probe_8_type_on_struct_instance ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 6
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s

# Row 7
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 8
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 9
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 10
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 11
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 12
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54

# Row 13
git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```

---

## Implementation surface — line counts

| Component | Lines | Location |
|---|---|---|
| `fn eval_record_of` | ~75 | `src/runtime.rs` after `fn eval_type` |
| `fn eval_record_field_at` | ~55 | `src/runtime.rs` after `fn eval_record_of` |
| Dispatch arms (2) | 8 (with comment block) | `src/runtime.rs` `dispatch_keyword_head_value` |
| TypeScheme registrations (2) | ~30 | `src/check.rs` end of `register_builtins` |
| TypeDef registration | 0 | Stone 234.1.5 already shipped; NOT re-registered (STOP-6 avoided) |

---

## Substrate-as-teacher cascade

Cascade depth: **shallow** as predicted. Zero `Value` match-exhaustiveness errors (no variant change; purely new fns + dispatch arms + TypeSchemes).

Cascade steps:
1. Add `eval_record_of` + `eval_record_field_at` + dispatch arms → `cargo build` → clean first compile
2. Add TypeScheme registrations → `cargo build` → clean
3. Run probe → 6/6 PASS first run (zero iteration cycles needed)

Total compile rounds: 2 (build after impl, build after check.rs). Zero red-green cycles.

---

## Trap-door audit — what fired

**Trap-door #3 (keyword carries leading colon) — FIRED.**

D5 in the DESIGN states "keywords don't carry leading `:` in their stored value." This is incorrect. Empirical evidence from `eval_keyword_to_string` at runtime.rs line 7510: `let text = k.strip_prefix(':').unwrap_or(&k);` — the strip is defensive but present because keywords DO store the leading colon. Confirmed by the AST evaluation path: `WatAST::Keyword(k, _)` at line 4719 passes `k` directly into `Value::wat__core__keyword(Arc::new(k.clone()))`, and `k` is the raw parsed keyword string including `:`.

The `class_fqdn` field on `Value::wat__Record` documents "no leading colon, e.g. `"myapp::Voltage"`." So `eval_record_of` must strip: `class_arc.strip_prefix(':').unwrap_or(&class_arc).to_string()`. This was applied correctly. Probe 1 (`class_fqdn.as_str() == "myapp::Voltage"`) and probe 2 (`:wat::core::type` returns `"myapp::Voltage"`) both PASS, confirming the strip is correct.

**All other trap-door items: clean.**
- `Value::Vec` Arc-ownership (#1): cloned Arc directly; no re-wrap.
- HolonAST extraction (#2): standard `Value::holon__HolonAST(arc_h)` pattern.
- Dispatcher accepts `::` and `/` separators (#4): `:wat::Record::of` and `:wat::Record/field-at` both dispatched cleanly; no special handling needed.
- `Value::wat__Record` construction (#5): three Arc'd fields populated from extracted args.
- TypeDef registration (#7): Stone 234.1.5 already registered `:wat::Record`; this stone consumed it without re-registration. STOP-6 avoided.

---

## Predecessor tools that shortened authoring

- **Stone 234.0's `eval_type` shape** — exact fn signature template (`args: &[WatAST]`, `list_span: &Span`, `env: &Environment`, `sym: &SymbolTable`) copied verbatim. Arity-check pattern + `eval_inner` + `value_owned()` chain was mechanical to replicate.
- **Stone 234.1's `Value::wat__Record` variant fields** — `class_fqdn: Arc<String>`, `struct_form: Arc<Vec<Value>>`, `holon_form: Arc<HolonAST>` were already defined; construction in `eval_record_of` was a straight populate-the-struct operation.
- **Stone 232.0's apply primitive precedent** — dispatch arm comment style + two-arm pattern (constructor + instance method in the same stone) shaped the implementation surface cleanly.
- **Stone 234.1.5's probe `make_record` helper** — confirmed the `Arc::new(class.to_string())` pattern for `class_fqdn` and `Arc::new(fields.iter().map(...).collect())` for `struct_form`.
- **`#[wat_value]` seal** — stayed quiet throughout. No variant additions; seal has nothing to enforce on new eval fns.
- **`ValueSnapshot::described`** — found when `ValueSnapshot::unavailable` couldn't accept a dynamic `String`. The `described(type_name: &'static str, description: String)` variant was the correct tool for the out-of-bounds diagnostic.

---

## Honest deltas

**D5 was wrong in the DESIGN.** Keywords carry the leading `:` in their stored `Arc<String>`. The DESIGN said "no leading-colon concern" but the code says otherwise. The strip is necessary and was applied. The probe caught this immediately on its first pass-check (probe 1 `class_fqdn.as_str() == "myapp::Voltage"` would have been `":myapp::Voltage"` without the strip). This is a case where empirical reading of existing code (eval_keyword_to_string, line 7510) was more reliable than the DESIGN doc's assertion.

**Generic-T inference on `field-at`** worked cleanly via standard recipient inference. Probe 5 calls `(:wat::Record/field-at v 1)` inside a `defn` returning `:wat::core::i64`; the type-checker unified T with i64 without any special-case handling. The TypeScheme `ret: t_var()` pattern mirrored by existing `Vector/get` was sufficient.

---

## Time breakdown

- Read mandatory docs (BRIEF + DESIGN + EXPECTATIONS + probe + precedent sources): ~15 min
- Investigate runtime.rs (eval_type shape, keyword storage, Vec Arc pattern, dispatcher site): ~10 min
- Implement eval_record_of + eval_record_field_at + dispatch arms: ~10 min
- Implement TypeScheme registrations in check.rs: ~5 min
- Build + probe run (zero iteration): ~5 min
- Full scorecard run (13 rows): ~3 min
- SCORE writing: ~10 min

**Total: ~58 min** (within 30-60 target band; at the upper end due to keyword-colon investigation)

---

## Calibration

- Predicted: 30-60 min (band middle ~40-50 min)
- Actual: ~58 min
- Result: within band, toward upper end
- Variance driver: keyword-colon trap-door (#3) required empirical investigation (~5 min) to confirm D5 was wrong

---

## Rank-up evidence — Helwalker/Streetfighter

Stone 234.2a is the third fight in arc 234's dungeon. The party-comp (Inquisitor marks via DESIGN + FM 2-bis probe; Shadowdancer strikes) shipped 3/3:
- Stone 234.0: ~38 min, ZERO iteration
- Stone 234.1.5: renaming stone, ZERO iteration
- Stone 234.2a: ~58 min, ZERO iteration (including cross-checking the D5 contradiction empirically rather than trusting the DESIGN)

The cascade was shallow as predicted: no variant addition, no match-exhaustiveness errors, no red-green cycles. The substrate-as-teacher effect was the keyword-colon trap-door — the code taught the correct behavior (`eval_keyword_to_string` strips `:`) rather than the DESIGN doc's incorrect assertion. Helwalker's strike-to-kill discipline: read the code, not just the doc.

---

## What this unblocks

- **Stone 234.2b** — `:wat::core::defrecord` macro. The macro will expand `(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])` into:
  - Constructor `(:myapp::Voltage [magnitude] ...)` that calls `(:wat::Record::of :myapp::Voltage [magnitude] <holon-form>)`
  - Field accessor `(:myapp::Voltage/magnitude [v <- :myapp::Voltage] ...)` that calls `(:wat::Record/field-at v 0)`
- **Stone 234.3** — polymorphic record-y verbs (assoc, record->map, record?, keyword-as-accessor) that operate on `Value::wat__Record` instances
- **§ R audit follow-on** — `:wat::Record::of` / `:wat::Record/field-at` are the first clean `::` / `/` split under arc 109 § R doctrine applied to new substrate

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2a.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a.md` — sub-DESIGN (D5 found incorrect; colon-strip is necessary)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2a.md` — paired EXPECTATIONS
- `tests/probe_arc234_stone2a_record_primitives.rs` — FM 2-bis probe (6/6 PASS)
- `src/runtime.rs` — `fn eval_record_of` + `fn eval_record_field_at` + two dispatch arms
- `src/check.rs` — two TypeScheme registrations in `register_builtins`
