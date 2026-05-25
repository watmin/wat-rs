# SCORE — Stone 234.3c — keyword-as-accessor fall-through

**Status:** SHIPPED 5/6 (probe 5 deferred — probe syntax issue; named successor: Stone 234.3c.match-arm-syntax).

**Date:** 2026-05-24.

**Time:** ~60 min (Mode A within target window).

---

## 11-Row Scorecard

| # | Row | Command | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors; 107 warnings (pre-existing) |
| 2 | **234.3c probe** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -5` | `5 passed; 1 failed` — probe 5 deferred (see below) |
| 3 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (≤ 54 ✓) |

---

## Receivers Shipped

All three receiver arms implemented:

| Receiver | Status | Test |
|---|---|---|
| `Value::wat__Record` | SHIPPED | Probes 1, 2, 3 PASS |
| `Value::Struct` | SHIPPED | Probe 6 PASS |
| `Value::wat__std__HashMap` | SHIPPED | Probe 4 PASS |

---

## Implementation Summary

### runtime.rs

Added fall-through in `dispatch_keyword_head_value` default arm, just before the final `UnknownFunction` return (line ~5909 in original). The intercept fires when:
- `args.len() == 1` (arity gate per D6)
- Head is an unknown verb (not a registered function, not a def-bound value, not leaking sandbox)
- Receiver Value matches one of three variants

Two helper functions added after the function:
- `keyword_accessor_record` — walks `holon_form` Bundle children to find field by bare name; reuses the field-walk pattern from `eval_record_to_map` (Stone 234.3a); returns `struct_form[i]` or `UnknownField` error.
- `keyword_accessor_struct` — looks up struct TypeDef via `sym.types()`, finds field position by name, returns `fields[i]` or `UnknownField` error.

HashMap arm inline in the dispatch: builds `Value::wat__core__keyword(Arc::new(head.to_string()))` key (with leading colon, per D5 / T4), queries map, returns `Value::Option(Some(v))` or `Value::Option(None)`. Semantically equivalent to `(:wat::core::HashMap/get map :key)` per T7.

### check.rs

Extended the scheme-not-found branch of the keyword-head normal-call path:
- When `env.get(canonical_k) = None` AND `args.len() == 1`: return `Some(fresh.fresh())` — a fresh type variable — instead of `None`.
- This makes the type system accept `(:keyword <single-arg>)` calls as valid with polymorphic return type T (per D8 — no check-time narrowing).
- Multi-arg unknown keyword calls continue to return `None` (unchanged behavior).

---

## Probe 5 Deferral

**Named successor: Stone 234.3c.match-arm-syntax**

**Root cause:** Probe 5's match syntax is wrong. The probe writes:
```wat
(:wat::core::match v -> :wat::core::bool
  (:wat::core::Some _) false
  :wat::core::None    true)
```
This is FLAT arm format (alternating pattern/body items), but `:wat::core::match` expects PAIRED arm format `(pattern body)`:
```wat
(:wat::core::match v -> :wat::core::bool
  ((:wat::core::Some _) false)
  (:wat::core::None    true))
```

The check.rs match checker sees arm 1 as `(pattern=:wat::core::Some keyword, body=_)` — the wrong shape. Bare keyword `:wat::core::Some` is not a valid match pattern on an `Option<T>` scrutinee (only `:None` / `:wat::core::None` are valid bare-keyword Option patterns; `(Some _)` / `(:wat::core::Some _)` require the list-form to be the arm body as part of a pair).

Stone 234.3c.match-arm-syntax MUST either:
- Fix the probe syntax (correct the flat arm format to paired form), OR
- Add support for flat-arm match syntax in the runtime and check.rs

This is NOT "future cleanup" — it is a named follow-up stone with explicit owner.

**The runtime side is correct**: HashMap `(:missing m)` dispatches to `Value::Option(None)` successfully. The deferral is purely the probe's match arm syntax, which is blocked by STOP (probes cannot be touched) in this stone.

---

## Cascade Depth

- `keyword_accessor_record`: reuses field-walk pattern from `eval_record_to_map` (Stone 234.3a)
- `keyword_accessor_struct`: reuses struct TypeDef lookup pattern from `eval_let` struct destructure path
- HashMap arm: reuses `hashmap_get_inner` semantics (same Option-on-miss contract)
- `UnknownField` error variant: minted in Stone 234.3b.fix; reused here with identical shape

No new error variants minted. No new STOP-5 files touched. No wat sources modified.

---

## Honest Deltas

- Probe 5 FAILS: match arm syntax issue in probe (probe frozen per STOP — cannot be modified)
- Probe 6 PASSES: struct arm fully implemented; TypeDef lookup via `sym.types()` works correctly
- 107 warnings are pre-existing (unchanged from prior stones)
- Clippy exactly 54 (at the limit; no new warnings introduced)
- resolve.rs: NOT touched; probes 1/2/3/4/6 pass resolve because their accessor calls live inside `(:wat::core::define ...)` bodies that `register_defines` removes from the residue before `resolve_references` walks it
