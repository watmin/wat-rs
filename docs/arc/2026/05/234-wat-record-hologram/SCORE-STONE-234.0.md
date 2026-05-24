# SCORE — Arc 234 Stone 234.0 — mint `:wat::core::type`

**Status:** COMPLETE. 11/11 PASS.
**Date:** 2026-05-24.
**Model:** sonnet (claude-sonnet-4-6)

---

## 11-Row Scorecard

| # | Row | Verification command | Expected | Actual |
|---|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors | `Finished release profile [optimized] target(s) in 26.80s` — 107 pre-existing warnings only; 0 errors |
| 2 | **New probe FLIPS 0/8 → 8/8** (LOAD-BEARING) | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `test result: ok. 8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s` |
| 4 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 5 | Stone 233.3 regression guard | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 6 | Stone 233.2.e regression guard | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 7 | Stone 233.2.l regression guard | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 8 | Stone 233.2.k regression guard | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 9 | Stone 233.1 ValueSnapshot guard | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 | `54` — at limit; 0 new warnings introduced |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output | (empty) |

---

## Per-Section Line Counts

### `src/runtime.rs` — `fn eval_type`

```rust
fn eval_type(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::type";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch { ... });
    }
    let arg_val = eval_inner(&args[0], env, sym)?.value_owned();
    let type_str = match &arg_val {
        Value::holon__HolonAST(h) => {
            extract_classifier(h).unwrap_or_else(|| "wat::holon::HolonAST".to_string())
        }
        Value::Struct(sv) => sv.type_name.trim_start_matches(':').to_string(),
        other => other.type_name().to_string(),
    };
    Ok(Value::String(Arc::new(type_str)))
}
```

**Function body:** 18 lines. **With doc comment (15 lines):** 33 lines total.

### `src/runtime.rs` — dispatch arm

```rust
":wat::core::type" => eval_type(args, list_span, env, sym),
```

**With comment block (5 lines):** 6 lines total. Dispatch arm itself: **1 line**.

### `src/check.rs` — TypeScheme registration

```rust
env.register(
    ":wat::core::type".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![t_var()],
        ret: TypeExpr::Path(":wat::core::String".into()),
        rest_param_type: None,
    },
);
```

**With comment block (8 lines):** 18 lines total. Registration itself: **10 lines**.

### `infer_list` special-case

**None added.** Standard `infer_call` path through `register_builtins` is sufficient —
`:wat::core::type` always returns `:wat::core::String` regardless of arg type, so no
annotation-driven return-type extraction is needed (unlike apply). The probe verified
the TypeScheme propagates correctly without a special case.

**Grand total new substrate:** ~57 lines across 2 files.

---

## Time Breakdown

- Document reading (BRIEF + EXPECTATIONS + DESIGN-STONE-234.0 + probe + DESIGN + SCORE-232.0 + SCORE-232.0a): ~12 min
- Reading runtime.rs (dispatch pattern, type_name(), extract_classifier, StructValue): ~5 min
- Reading check.rs (infer_list, register_builtins, TypeExpr patterns): ~5 min
- Authoring `eval_type` + dispatch arm in runtime.rs: ~3 min
- Authoring TypeScheme registration in check.rs: ~3 min
- `cargo build --release` (first compile): ~27 sec compile time
- Running load-bearing probe (8/8 PASS on first run): ~22 sec
- Running all 11 verification commands: ~8 min
- SCORE writing: ~10 min

**Actual elapsed: ~38 min**
**Predicted band: 30–60 min Mode A**
**Result: IN BAND (lower half)**

---

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Target band | 30–60 min | 38 min — in band |
| Upper bound (STOP-3) | 90 min | Not approached |
| Confidence | high | Validated |
| Lines of new substrate | ~50 | ~57 (slightly over; doc comment discipline) |
| Compile + iterate cycles | ~5 min | 1 cycle, 0 iteration (8/8 PASS on first run) |
| infer_list special-case | possibly needed | NOT needed — confirmed |

**Key calibration note:** Zero iteration cycles. The probe passed 8/8 on the first
clean compile. This is the smallest stone in arc 234 and the fastest execution in the
arc 232–234 sequence.

---

## Rank-Up Evidence — Arc 233 + Stone 232.0a Tools

**The measurable rank-up property from EXPECTATIONS:** "sonnet's iteration cycles
should be informative without diagnostic-print scaffolding."

### Concrete case 1 — TypeExpr::Var(u64) vs TypeExpr::Path(":T") disambiguation

The BRIEF's D4 specifies `TypeExpr::Var("T".into())` for the param. Reading the actual
`types.rs` definition revealed `TypeExpr::Var` takes `u64`, not `String`. The correct
idiom in `register_builtins` is `t_var()` = `TypeExpr::Path(":T".into())` — the same
convention used by the apply sentinel (Stone 232.0 precedent). The read sequence
(SCORE-232.0 → register_builtins at line 12926 → apply sentinel at line 16936) showed
the exact pattern before writing a single line of code. No compile error needed to
surface this discrepancy.

### Concrete case 2 — #[wat_value] seal prevented Value variant consideration

The `#[wat_value]` proc-macro (Stone 233.2.l) sealed the `Value` enum. When writing
`eval_type`, the natural question was "do I need a new Value variant to handle the
match return?" The seal provides structural confidence: `Value::String(Arc::new(...))` 
already exists; no new variant is needed. The implementation was written with
zero hesitation about the match arm return type.

### Concrete case 3 — No infer_list special-case needed (honest empirical finding)

The BRIEF flagged polymorphic TypeScheme inference as a risk. The standard
`infer_call` path through `register_builtins` (`:T` param, `:wat::core::String` ret)
was sufficient — the probe passed probe_2 (String → String), probe_1 (i64 → String),
and all others without a special-case. This was confirmed empirically rather than
assumed. The absence of a special-case is documented explicitly so future readers know
the decision was tested, not just omitted.

### Concrete case 4 — 8/8 PASS on first compile without print scaffolding

All 8 probe contracts passed on the first clean compile. The substrate's existing
pattern — eval_inner + value_owned + match on Value + return Value::String — is
well-grooved after arcs 232 and 233. The dispatch table from D2 translated directly
to Rust without gap. No scaffolding was added; no debug prints were inserted.

---

## Honest Deltas

### Delta 1 — TypeExpr::Var vs TypeExpr::Path in register_builtins

BRIEF's D4 says `TypeExpr::Var("T".into())` but `TypeExpr::Var` takes `u64` in the
actual codebase. Correct idiom is `t_var()` = `TypeExpr::Path(":T".into())` per the
existing `register_builtins` convention. No impact — reading the SCORE-232.0 precedent
and the actual check.rs code before writing surfaced this immediately.

### Delta 2 — No infer_list special-case added

BRIEF predicted "possibly needed." Empirically confirmed NOT needed. The TypeScheme
`params: vec![t_var()]` (`:T`), `ret: TypeExpr::Path(":wat::core::String"...)` is
sufficient for the standard `infer_call` path — the type-checker sees "accepts any
value, returns String" without additional special-case logic. Probe_1 through Probe_8
all pass, including the defrecord and struct cases where the arg type is exotic.

### Delta 3 — Clippy at 54 (at limit; no new warnings)

Pre-stone count was 54. Post-stone count is 54. The new `eval_type` function follows
the existing arity-check + eval-inner + match + return pattern without any
`let _ = ...` or unused-variable patterns that would add new clippy warnings.

---

## STOP Triggers

None fired.

- **STOP-1:** 0 compile errors. ✓
- **STOP-2:** 827 passed; 0 failed. ✓
- **STOP-3:** ~38 min elapsed; well within 90 min. ✓
- **STOP-4:** holon-rs untouched (empty git status). ✓
- **STOP-5:** Clippy count 54 (≤ 54 limit). ✓
- **STOP-6:** No Value variant, no macro, no record-y verbs, no destructure — eval_type + dispatch arm + TypeScheme only. ✓
- **STOP-7:** New probe flips 0/8 → 8/8. ✓
- **STOP-8:** All arc 233 regression guards GREEN (5/5, 5/5, 3/3, 5/5, 8/8). ✓
- **STOP-9:** Stone 232.0a typed-entities reflection probe 7/7 GREEN. ✓

---

## What This Unblocks

- **Revised Stone 232.1** — `:wat::core::defprotocol` + `:wat::core::extend-type`
  polymorphic via `:wat::core::type`. The dispatcher can now extract the receiver's
  type FQDN regardless of storage backend (HolonAST classifier-wrap, struct, primitive).
- **Stone 234.1** — `Value::wat_record` variant. The `eval_type` doc comment already
  marks the TODO arm (`// TODO: arc 234.1 adds wat_record arm here returning class_fqdn`);
  the addition is a single match arm.
- **Stone 234.3** — polymorphic record-y verbs (assoc, record->map, record?, record->holon,
  keyword-as-accessor). All consume `:wat::core::type` for dispatch routing.
- **All subsequent arc 234.x stones** — the polymorphic type primitive is the dispatch
  foundation for the wat-record hologram dungeon.

---

## Cross-References

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.0.md` — authoritative plan
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.0.md` — 11-row scorecard target
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md` — sub-DESIGN (6 locked decisions)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `tests/probe_diagnostic_polymorphic_type.rs` — FM 2-bis probe (8 contracts; now 8/8 PASS)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — extract-classifier precedent
- `src/runtime.rs` — `fn eval_type` (near line 14401); dispatch arm in `dispatch_keyword_head_value`
- `src/check.rs` — `register_builtins` entry for `:wat::core::type` (at end of function)
