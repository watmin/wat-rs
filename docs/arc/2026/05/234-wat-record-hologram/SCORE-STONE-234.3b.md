# SCORE — Arc 234 Stone 234.3b — `:wat::Record/assoc` substrate primitive

**Status:** COMPLETE — 11/11 PASS. Mode A target achieved.

**Successor:** Stone 234.3b' (polymorphic `:wat::core::assoc` dispatch to this primitive for record receivers) is now unblocked. Stone 234.6 migration ergonomics can reference `:wat::Record/assoc` directly.

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **234.3b probe 6/6 PASS** (LOAD-BEARING) | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` — PASS |
| 3 | 234.3a regression | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 4 | 234.2c regression | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 5 | 234.2b regression | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 6 | 234.5 regression | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 7 | 234.2a regression | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 8 | Lib baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 9 | 232.0a regression | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 10 | Clippy | ≤ 54 | `54` (at ceiling; no regression) |
| 11 | holon-rs untouched | empty output | (empty) — STOP-4 clean |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 0.07s

# Row 2 — LOAD-BEARING
cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 | tail -5
test probe_3_unknown_field_errors ... ok
test probe_2_multi_field_update_one ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 3
cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 | tail -3
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 4
cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 5
cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 | tail -3
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 6
cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 | tail -3
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 7
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -3
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 8
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s

# Row 9
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 10
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54

# Row 11
git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```

---

## Implementation surface

### `eval_record_assoc` — ~115 lines (src/runtime.rs)

**Dispatch arm** added after `:wat::core::record->map`:
```rust
":wat::Record/assoc" => eval_record_assoc(args, list_span, env, sym),
```

**Function shape:** 3-arity check → eval 3 args → destructure record → extract keyword bare name (strip `:` prefix per T2) → walk holon_form Bundle to find field index + collect available names → UnknownField via `MalformedForm` (see "UnknownField variant" below) → runtime type check (`type_name()` string comparison per T3) → rebuild struct_form + holon_form → return new `Value::wat__Record`.

**HolonAST rebuild approach (T4 / T5 / T6):**
1. Call `to_holon_inner(new_val, &list_span)` → `Value::holon__HolonAST(Arc::new(holon))` → unwrap to bare `HolonAST`.
2. Build new field-Bind: `Bind(Atom(String(field-name)), Atom(to_holon_inner(new_val)))` — mirrors the Record.wat macro's `(:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon name-s)) (:wat::holon::Atom (:wat::holon::to-holon var)))` construction exactly.
3. Clone `field_binds` Vec (from `(*children).clone()`), replace at `field_index`.
4. Build new `HolonAST::Bundle(Arc::new(new_children))`.
5. Re-use original outer Bind's left (class Atom) + new Bundle as right: `HolonAST::Bind(class_atom, Arc::new(new_bundle))`.
6. Wrap in `Arc::new(...)` → `new_holon_form`.

### `check.rs` TypeScheme registration — ~14 lines

Added after `:wat::core::record->map` registration (end of `register_builtins`, before closing `}`):
```rust
env.register(
    ":wat::Record/assoc".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![record_ty(), TypeExpr::Path(":wat::core::keyword".into()), t_var()],
        ret: record_ty(),
        rest_param_type: None,
    },
);
```
`record_ty` and `t_var` are the closures already in scope (defined at lines 17298 and 13235). No custom handler needed — standard TypeScheme registration sufficient.

---

## UnknownField error variant — decision

**DESIGN D7** specified a new `UnknownField` runtime variant. However, `RuntimeError` is matched exhaustively in `src/runtime_error_edn.rs` (the EDN wire-format serializer). Adding a new variant to `RuntimeError` would require touching `runtime_error_edn.rs` in addition to `runtime.rs` + `check.rs`, violating **STOP-5** (Rust changes outside runtime.rs + check.rs only).

**Resolution:** Used `RuntimeError::MalformedForm` with a `reason` string containing "unknown" and the field name. This satisfies:
- Probe 3's assertion: `msg.to_lowercase().contains("unknown") || msg.contains("nonexistent")` — the reason includes "unknown field 'nonexistent'".
- The semantic intent of the error: the form `(:wat::Record/assoc ...)` is malformed when the key doesn't exist.

The `MalformedForm` variant carries `head: OP.into()` (`:wat::Record/assoc`) and `reason: format!("unknown field '{}' on record {}; available fields: [...]", ...)`. Full field list is included for debuggability.

**Future:** A dedicated `UnknownField` variant (per DESIGN D7) is correct but is a cascade change requiring `runtime_error_edn.rs`. Defer to a future arc that does a systematic RuntimeError variant sweep.

---

## Cascade depth

**Compile rounds: 2.**

- **Round 1:** Initial implementation used `coerce_to_holon_ast(OP, new_val, &list_span)` for the HolonAST value encoding. Failed at runtime: `coerce_to_holon_ast` only handles `Value::holon__HolonAST` or `Value::wat__Record` — rejects plain primitives (f64, String, i64) with `TypeMismatch`. Probes 1, 2, 5, 6 failed; probes 3, 4 passed (error-path probes).
- **Round 2:** Replaced `coerce_to_holon_ast` with `to_holon_inner(new_val, &list_span)?` + unwrap. `to_holon_inner` handles all primitive Value variants. 6/6 probes PASS.

No Rust changes outside `src/runtime.rs` and `src/check.rs`.

---

## Trap-door audit (T1-T8)

### T1 — Field-name lookup (D3, D8)
**RESOLVED.** Reused `eval_record_to_map`'s exact pattern inline: `holon_form → Bind → Bundle.children → each Bind → left → Atom → String → field-name`. Same error messages for structural failures. The loop additionally collects `available` names for the UnknownField error and sets `found_index` when a match is found.

### T2 — Keyword key leading-colon strip
**RESOLVED.** `key.strip_prefix(':').unwrap_or(s)` applied to the keyword's string content. Verified: `Value::wat__core__keyword` stores with leading colon (e.g., `":magnitude"`); holon_form field names are bare (e.g., `"magnitude"`). Strip is mandatory for match.

### T3 — Value type comparison (`type_name()`)
**RESOLVED.** `struct_form[field_index].type_name()` vs `new_val.type_name()` — both return `&'static str`. String equality is the correct check at this substrate level. Probe 4 verifies: f64 field + i64 new value → TypeMismatch; the `expected` field in the error carries the old type name and `got` carries the new value's snapshot.

### T4 — HolonAST rebuild for the updated Bind
**RESOLVED.** Key trap: `coerce_to_holon_ast` (per Stone 234.5) only accepts already-HolonAST or `wat::Record` values — it cannot convert plain primitives. Used `to_holon_inner(new_val, span)` instead, which handles all Value variants. Then wrapped result in `HolonAST::Atom(Arc::new(holon))` to mirror the Record.wat macro's `(:wat::holon::Atom (:wat::holon::to-holon var))` structure. This was the only substantive compile-round correction.

### T5 — Bundle reconstruction
**RESOLVED.** `(*field_binds).clone()` gives a `Vec<HolonAST>`; replace at `field_index`; wrap in `HolonAST::Bundle(Arc::new(new_children))`. The `Arc<Vec<HolonAST>>` deref pattern matches 234.3a's `field_binds.iter().enumerate()` style.

### T6 — Outer Bind reconstruction
**RESOLVED.** Extracted `class_atom: Arc<HolonAST>` from the outer Bind's left BEFORE rebuilding (to avoid double-borrow). `HolonAST::Bind(class_atom, Arc::new(new_bundle))` — `class_atom` is already `Arc<HolonAST>` so no re-wrap needed. The `holon_form.as_ref()` pattern gives `&HolonAST` for the outer match; `left.clone()` gives the `Arc<HolonAST>` for the new Bind.

### T7 — Empty record case (zero-field)
**RESOLVED.** `found_index` stays `None` when `field_binds` is empty; `MalformedForm` fires with `available: []`. Probe 3 tests against a non-empty Triple but the empty case is trivially correct by the loop logic.

### T8 — Single-field record
**RESOLVED.** Probes 1 and 5 both use `:myapp::Voltage` (1 field). Both PASS. Single-element Vec clone + replace + Bundle rebuild all correct.

---

## Time breakdown

- Read mandatory artifacts (BRIEF + DESIGN + EXPECTATIONS + probe + SCORE-234.3a + runtime.rs reference functions + check.rs): ~12 min
- Author dispatch arm + eval_record_assoc + TypeScheme: ~8 min
- Compile Round 1 + diagnose `coerce_to_holon_ast` trap: ~3 min
- Fix: switch to `to_holon_inner` + Atom-wrap: ~2 min
- Compile Round 2 + 6/6 probes PASS: ~2 min
- All 11 scorecard rows capture: ~3 min
- SCORE writing: ~10 min

**Total: ~40 min.** Within the 45-75 min Mode A target band.

---

## Calibration delta

- Predicted: 45-75 min Mode A
- Actual: ~40 min
- Variance: 5 min under lower bound — implementation was shorter than predicted once the 234.3a field-walking pattern was in hand.
- The single correction cycle (coerce_to_holon_ast → to_holon_inner) was the only non-trivial step. The DESIGN correctly identified T4 as a risk; the trap materialized exactly as predicted.

---

## Rank-up evidence

**Predecessor patterns reused:**

1. **`eval_record_to_map` field-walking loop** — copied verbatim (outer Bind → Bundle children → inner Bind → Atom → String extraction). Zero guessing.

2. **`eval_record_field_at` arity + TypeMismatch shape** (line 14586) — arity guard + record destructure pattern copied exactly.

3. **Record.wat macro lines 154-160** — confirmed the `Atom(to-holon(var))` right-side structure. `to_holon_inner` + `Atom` wrap is the correct reconstruction path.

4. **`to_holon_inner` return convention** — returns `Value::holon__HolonAST(Arc::new(holon))`; unwrap via `match` with `unreachable!` guard (same pattern as `hashmap_assoc_inner`'s clone-then-new-Arc functional update).

5. **`record_ty`, `t_var` closures** already in scope at `register_builtins` (lines 17298 and 13235) — reused without redefinition.

6. **STOP-5 discipline** — avoided adding `UnknownField` variant to `RuntimeError` (would cascade to `runtime_error_edn.rs`). Used `MalformedForm` instead, satisfying probe 3 without STOP-5 breach.

---

## Working tree state

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/check.rs
 M src/runtime.rs
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3b.md
```

`src/runtime.rs` and `src/check.rs` modified (this stone). SCORE is the only new file.
No other files dirty. DO NOT COMMIT — orchestrator verifies independently.

---

## Honest assessment

`:wat::Record/assoc` ships clean:
- 6/6 probe contracts verified: single-field update, multi-field (one updated), UnknownField error, TypeMismatch error, immutability (original unchanged), compose (nested assoc).
- HolonAST rebuild correct: struct_form clone + replace; Bundle child replacement; outer Bind reconstruction with fresh class atom.
- `MalformedForm` used in place of new `UnknownField` variant — honest STOP-5 compliance; probe satisfied.
- Clippy at 54 (ceiling; no regression).
- Lib baseline at 827 (no regression).
- All prior arc 234 regression guards clean.
- holon-rs untouched.

Stone 234.3b complete. The write verb in the polymorphic record-y family is shipped.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.3b.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3b.md` — sub-DESIGN (10 locked decisions)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3b.md` — paired EXPECTATIONS
- `tests/probe_arc234_stone3b_record_assoc.rs` — FM 2-bis probe (6/6 PASS)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3a.md` — predecessor field-walking pattern
- `src/runtime.rs` line 4878 — dispatch arm (`:wat::Record/assoc`)
- `src/runtime.rs` line 14784 — `eval_record_assoc` function (~115 lines)
- `src/check.rs` line 17344 — TypeScheme registration (~14 lines)
