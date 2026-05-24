# SCORE — Arc 234 Stone 234.3a — read verbs: `:wat::core::record?` + `:wat::core::record->map`

**Status:** COMPLETE — 11/11 PASS. Mode A target achieved.

**Successor:** Stone 234.3b (`:wat::core::assoc` polymorphic record arm) is now unblocked.
Field-name extraction machinery (holon_form walking) established here is available for reuse.

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **New probe FLIPS 6/6 FAIL → 6/6 PASS** (LOAD-BEARING) | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` — PASS |
| 3 | Stone 234.2c regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 4 | Stone 234.2b regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 5 | Stone 234.5 regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 6 | Stone 234.2a regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 7 | Stone 234.1 regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 8 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 9 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 10 | Clippy no new warnings | ≤ 54 | `54` (at ceiling; no regression) |
| 11 | holon-rs untouched | empty output | (empty) — STOP-4 clean |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 0.05s

# Row 2 — LOAD-BEARING
cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 | tail -5
test probe_3_record_to_map_single_field ... ok
test probe_4_record_to_map_multi_field_heterogeneous ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 3
cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 | tail -5
test probe_3_panic_message_names_both_classes ... ok
test probe_2_wrong_class_panics ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 4
cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 | tail -5
test probe_3_predicate_true_on_matching_class ... ok
test probe_1_single_field_construction ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 5
cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 | tail -5
test probe_4_bundle_accepts_records_as_children ... ok
test probe_2_cosine_accepts_records ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 6
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -5
test probe_2_type_returns_class_fqdn ... ok
test probe_7_equality_via_holon_form ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

# Row 7
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5
test probe_5_hash_eq_consistency ... ok
test probe_4_eq_different_field_values ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 8
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.20s

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

### `eval_record_q` — ~20 lines (src/runtime.rs)

Single `matches!` pattern on `Value::wat__Record { .. }`. Arity check (1 arg), eval arg, pattern match, return bool. No helpers reused — trivial variant check.

Dispatch arm added after `:wat::Record/field-at` in `dispatch_keyword_head`:
```rust
":wat::core::record?" => eval_record_q(args, list_span, env, sym),
```

### `eval_record_to_map` — ~85 lines (src/runtime.rs)

**HolonAST extraction helper reused:** No new helper was minted. Used direct pattern matching on the `HolonAST` enum variants, mirroring `extract_classifier` (line 14652) and `require_bundle` (line 12212). The `bind_left` / `bind_right` helpers (lines 14673–14690) were not called directly — the same nested pattern match was used inline since the traversal is a one-shot path through Bind → Bundle → Bind children → Atom → String.

**Traversal path (T2 verified):**
```
holon_form: Arc<HolonAST>
  = Bind(                               ← HolonAST::Bind outer
      Atom(String("myapp::Voltage")),   ← left = class classifier (ignored)
      Bundle([                          ← right = field-Bind Bundle
        Bind(                           ← each child
          Atom(String("magnitude")),    ← left = field-name (no leading colon)
          Atom(...)                     ← right = value (ignored; struct_form used)
        ),
        ...
      ])
    )
```

**HashMap construction pattern:**
```rust
let mut map: std::collections::HashMap<Value, Value> =
    std::collections::HashMap::with_capacity(field_binds.len());
for (i, field_bind) in field_binds.iter().enumerate() {
    let field_name = /* extract from Bind.left.Atom.String */;
    let key = Value::wat__core__keyword(Arc::new(format!(":{}", field_name)));
    let val = struct_form[i].clone();
    map.insert(key, val);
}
Ok(Value::wat__std__HashMap(Arc::new(map)))
```

**Key format (T6 verified):** `Value::wat__core__keyword` stores WITH leading colon per contract at line 7558 (`format!(":{}", s)`). Field names from `keyword/to-string` arrive WITHOUT colon (line 7494: `"foo"` not `":foo"`). So `format!(":{}", field_name)` correctly produces `":magnitude"` etc.

**`#[allow(clippy::mutable_key_type)]` required:** `HashMap<Value, Value>` triggers clippy. Annotated per Stone 216.5c precedent (line 9469).

Dispatch arm added after `:wat::core::record?`:
```rust
":wat::core::record->map" => eval_record_to_map(args, list_span, env, sym),
```

### check.rs TypeScheme registrations — ~28 lines

Both registrations added after the existing `":wat::Record/field-at"` registration (end of `register_builtins` function, before the closing `}`).

**`record?` TypeScheme:**
```rust
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![t_var()],    // ∀T — accepts any value
    ret: bool_ty(),
    rest_param_type: None,
}
```
Mirrors `:wat::holon::to-holon` (line 14080) which also uses `type_params: vec!["T".into()], params: vec![t_var()]`. The `bool_ty` and `t_var` closures are defined at lines 13233 and 13235 respectively — both in scope throughout `register_builtins`.

**`record->map` TypeScheme:**
```rust
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![record_ty()],    // :wat::Record input
    ret: TypeExpr::Parametric {
        head: "wat::core::HashMap".into(),
        args: vec![
            TypeExpr::Path(":wat::core::keyword".into()),
            t_var(),
        ],
    },
    rest_param_type: None,
}
```
`record_ty` closure defined at line 17298 (already in scope from the field-at registration above). The `TypeExpr::Parametric` pattern mirrors `hashmap_of(k, v)` at line 16900.

**Did record->map require a custom handler?** No. Standard TypeScheme registration was sufficient. The type-checker propagates `:T` via recipient inference from the `let`-binding or the defn return annotation — identical to how `:wat::Record/field-at` works. Probes 3, 4, 5, 6 all involve explicit return-type annotations (`-> :wat::core::f64`, `-> :wat::core::String`, `-> :wat::core::bool`) that drive T unification. No infer_list dispatch arm was needed.

---

## Cascade depth

**Compile rounds: 2.**

- **Round 1:** Added dispatch arms + eval functions + TypeScheme registrations. Failed with `E0716: temporary value dropped while borrowed` — the error-path code used `&format!(...)` as argument to `ValueSnapshot::unavailable(&'static str)`. The dynamic format string can't be `'static`.
- **Round 2:** Replaced all `other =>` arms in error paths with `_ =>` and static string literals. Clean compile. All 6 probes PASS.

No Rust changes outside `src/runtime.rs` and `src/check.rs`.

---

## Trap-door audit (T1-T8)

### T1 — `Value::wat__Record` field access
**RESOLVED.** Pattern `Value::wat__Record { struct_form, holon_form, .. }` matches cleanly. Both `struct_form` (Arc<Vec<Value>>) and `holon_form` (Arc<HolonAST>) destructure as documented. Exact same pattern as `eval_record_field_at` (line 14598) and `to_holon_inner` (line 15371).

### T2 — HolonAST traversal helpers
**RESOLVED.** Field names are stored as `HolonAST::Atom(Arc<HolonAST::String(s)>)` in the left position of each field-Bind. Confirmed by reading the Record.wat macro expansion (lines 154-160): `(:wat::holon::to-holon name-s)` where `name-s` comes from `keyword/to-string` (strips leading colon). `to_holon_inner` converts `Value::String(s)` → `HolonAST::string(s)` (line 15215). Then `HolonAST::Atom` wraps it via `(:wat::holon::Atom ...)` call.

Traversal: `holon_form → Bind → right (Bundle) → children → each Bind → left (Atom) → inner (String) → field-name`. Implemented inline via nested pattern match. `require_bundle` helper not called directly (its error message assumes "signature head" context; cleaner to pattern-match inline with OP-specific messages).

### T3 — Pairing holon_form children with struct_form indices
**RESOLVED.** `for (i, field_bind) in field_binds.iter().enumerate()` pairs index i with `struct_form[i]`. Construction-time invariant (Stone 234.2b macro) guarantees the lengths match. No guard needed for length mismatch — the invariant holds at construction time.

### T4 — Empty Bundle case (zero-field record)
**RESOLVED.** Probe 5 confirms: `(:wat::Record::def :myapp::Tag [])` → `record->map` → empty HashMap → `empty?` returns true. The `for` loop iterates zero times; `HashMap::with_capacity(0)` works; `Ok(Value::wat__std__HashMap(Arc::new(map)))` returns an empty map.

### T5 — class_fqdn unused for record->map
**RESOLVED.** Pattern match uses `..` to ignore `class_fqdn`. Only `struct_form` and `holon_form` are bound.

### T6 — HashMap<keyword, T> TypeScheme
**RESOLVED.** Standard `TypeExpr::Parametric` with `head: "wat::core::HashMap"` mirrors existing HashMap return types (e.g., line 16900's `hashmap_of`). No custom handler needed — T propagates cleanly via recipient inference in all 4 probes that use `record->map`.

### T7 — `record?` polymorphic-input TypeScheme
**RESOLVED.** `type_params: vec!["T".into()], params: vec![t_var()]` accepts any input type. Probe 1 (record → true), Probe 2 (i64 → false), Probe 6 (composition with if) all type-check and pass. Mirrors `:wat::holon::to-holon` shape exactly.

### T8 — Stone 234.2b regression: macro-generated predicates work alongside `record?`
**RESOLVED.** Stone 234.2b probe passes 6/6 (Row 4). `:myapp::is-Voltage?` (class-specific, generated by macro) and `:wat::core::record?` (polymorphic, substrate-minted) coexist without collision — different namespaces, different semantics.

---

## Time breakdown

- Read mandatory artifacts (BRIEF + DESIGN + EXPECTATIONS + probe + runtime.rs reference functions + check.rs patterns): ~15 min
- Author dispatch arms + eval_record_q + eval_record_to_map + TypeScheme registrations: ~10 min
- Compile Round 1 + fix E0716 (static str issue in error paths): ~5 min
- Compile Round 2 + all 6 probes PASS: ~2 min
- All 11 scorecard rows verification + capture: ~5 min
- SCORE writing: ~10 min

**Total: ~47 min.** Within the 30-60 min target band.

---

## Calibration delta

- Predicted: 30-60 min Mode A (target band center ~45 min)
- Actual: ~47 min
- Variance: tight to prediction. The single compile-round correction (E0716 on `&format!()` to `&'static str`) was the only iteration cycle.
- The holon_form traversal was the novel piece — took ~4 min to trace Record.wat macro lines 154-160 → confirm Atom(String) structure → implement inline. The `require_bundle` helper's `&'static str` constraint was the only trap.

---

## Rank-up evidence

**Predecessor patterns reused:**

1. **`eval_record_field_at` arity + TypeMismatch shape** (line 14586) — copied verbatim for both new functions. Zero guessing about RuntimeError variant fields.

2. **`to_holon_inner` / Stone 234.5's `wat__Record` arm** (line 15371) — confirmed the `holon_form.as_ref()` pattern for Arc<HolonAST> unwrap.

3. **`hashmap_assoc_inner` / `HashMap::with_capacity` construction** (line 9858) — mirrored functional map-building pattern.

4. **`#[allow(clippy::mutable_key_type)]`** (line 9469) — applied consistently with Stone 216.5c precedent.

5. **`:wat::holon::to-holon` TypeScheme** (line 14078) — ∀T input pattern copied for `record?` registration.

6. **`record_ty` closure already defined** at line 17298 — reused for `record->map` input param without redefinition.

The `infer_record_of` custom-handler precedent (Stone 234.2a-CORRECTION) was NOT needed — standard TypeScheme inference was sufficient for both new verbs. The DESIGN correctly predicted this as the likely outcome (D6: "per 234.2a-CORRECTION + Stone 234.2b precedent, polymorphic-T return drives via recipient inference").

---

## Working tree state

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/check.rs
 M src/runtime.rs
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3a.md
```

`src/runtime.rs` and `src/check.rs` modified (this stone). SCORE is the only new file.
No other files dirty. DO NOT COMMIT — orchestrator verifies independently.

---

## Honest assessment

Both primitives ship clean:
- `record?` trivial predicate — 11/11 PASS
- `record->map` field-name extraction — 11/11 PASS
- No custom type-check handler needed
- Standard TypeScheme registration sufficient for both
- Clippy at 54 (ceiling, no regression)
- lib baseline at 827 (no regression)
- All prior arc 234 regression guards clean

Stone 234.3b is unblocked. The holon_form walking logic (Bind → Bundle → field-Bind-children → Atom → String extraction + struct_form positional pairing) is established and tested.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.3a.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3a.md` — sub-DESIGN (12 locked decisions)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3a.md` — paired EXPECTATIONS
- `tests/probe_arc234_stone3a_record_read_verbs.rs` — FM 2-bis probe (6/6 PASS)
- `src/runtime.rs` line 4869 — dispatch arms (`:wat::core::record?` + `:wat::core::record->map`)
- `src/runtime.rs` line 14645 — `eval_record_q` function (~20 lines)
- `src/runtime.rs` line 14667 — `eval_record_to_map` function (~85 lines)
- `src/check.rs` line 17308 — `record?` + `record->map` TypeScheme registrations (~28 lines)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent (NOT needed here)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — holon_form access precedent
