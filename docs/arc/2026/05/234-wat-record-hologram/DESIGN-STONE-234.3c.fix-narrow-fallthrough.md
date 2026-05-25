# DESIGN — Stone 234.3c.fix-narrow-fallthrough

**Status:** ACTIVE (2026-05-24).

**Origin:** Stone 234.3c added a fall-through in `dispatch_keyword_head_value`'s check.rs counterpart at line 5906-5908: when head is unknown verb + args.len() == 1, return polymorphic T (`fresh.fresh()`) instead of None. This was too broad — it accepts ANY 1-arg keyword call regardless of receiver type. Stone 234.4 probe 6 surfaced the consequence (`:i64/to-f64 c` where c is i64 passed check; cascaded type confusion at runtime).

Orchestrator framed it as "design trade-off (loose-check, strict-runtime)" in 234.4 commit `dab1a5cb`. User correctly identified that framing as deferral-violation dressing. This stone fixes the over-permissiveness.

---

## Scope

ONE file changed: `src/check.rs`. Narrow the line 5906-5908 fall-through to discriminate by receiver type.

---

## Locked decisions

### D1 — Discrimination by receiver type

After the existing inference recurse at line 5893-5895 (which infers each arg's type), CAPTURE args[0]'s inferred type. Discriminate:

| Receiver type at check time | Action |
|---|---|
| `TypeExpr::Path(":wat::Record")` | Return `fresh.fresh()` (record receiver; runtime dispatches) |
| `TypeExpr::Path(name)` where name is a registered struct type | Return `fresh.fresh()` (struct receiver) |
| `TypeExpr::Parametric { head: "wat::core::HashMap", .. }` | Return `fresh.fresh()` (HashMap receiver) |
| `TypeExpr::Var(...)` (unresolved/polymorphic) | Return `fresh.fresh()` (can't narrow; permissive but unavoidable) |
| Any other concrete type (`:i64`, `:String`, `:bool`, etc.) | Return `None` (fall through to existing UnknownFunction error) |

Sonnet investigates the exact TypeExpr matching mechanics.

### D2 — Don't re-infer args[0]

Line 5893-5895 already infers each arg via `infer(arg, env, locals, fresh, subst, errors)`. Capture the first call's result instead of discarding with `let _`.

```rust
let arg_types: Vec<Option<TypeExpr>> = args.iter()
    .map(|arg| infer(arg, env, locals, fresh, subst, errors))
    .collect();
// args[0]'s type is arg_types[0]; use for discrimination
```

Or just capture args[0] explicitly:
```rust
let receiver_ty = if args.len() == 1 {
    infer(&args[0], env, locals, fresh, subst, errors)
} else {
    None
};
```

Sonnet picks the cleaner form.

### D3 — Apply substitution before matching

The inferred type may carry unresolved type vars. Apply `apply_subst(&ty, subst)` (or equivalent) before pattern matching to get the most-resolved form. Standard wat type-checker pattern.

### D4 — Struct receiver: look up registered types

Struct types are registered in the type environment. `env` has `.types` (the TypeEnv) or similar. To check if `TypeExpr::Path(name)` is a struct, look up `name` in the type env + verify it's a Struct TypeDef.

Sonnet finds the exact predicate (likely `env.types.is_struct(name)` or similar; mirrors patterns in existing struct-dispatch code).

### D5 — HashMap parametric pattern

`TypeExpr::Parametric { head: "wat::core::HashMap", args: [K, V] }` — match the head string + 2 args. The K and V can be anything (including fresh vars).

### D6 — Unresolved-arg fallback is intentional

When args[0]'s inferred type is `TypeExpr::Var(...)` (unresolved at this point of inference), we CAN'T narrow — return polymorphic T. This is the same "permissive when unresolved" that 234.3c relied on.

Loss: an expression like `(let [x ?]  (:field x))` where x's type is unbound still type-checks. Runtime catches.

Gain: the COMMON case of concrete typed receivers (e.g., `c: i64`) now fails at check time with UnknownFunction, not at runtime with cascaded confusion.

### D7 — HARD CUT on permissiveness retention

After this fix, `(:bogus 42)` and `(:i64/to-f64 c)` (where c is i64) fail at CHECK TIME with UnknownFunction. No "permissive mode" preserved.

### D8 — All prior arc 234 + 234.4 probes stay green

234.3c probe (record/HashMap/struct keyword accessor) stays GREEN — receivers ARE record/HashMap/struct.
234.4 probe (hash-destructure) stays GREEN — destructure RHS is record/HashMap.
Lib baseline 827 unchanged.

The narrowing affects ONLY expressions that today silently type-check then fail at runtime. None of those are in current probes.

---

## Trap-door audit

### T1 — apply_subst location
The inferred type from `infer(...)` may need `apply_subst(&ty, subst)` applied to get the most-resolved form. Sonnet locates the existing helper.

### T2 — Struct type registry access
Need to query `env.types` (or whatever TypeEnv lives at) to check if a path name refers to a registered struct. Look for existing query patterns (likely in struct-dispatch code).

### T3 — Path string for :wat::Record
The TypeExpr for records is `TypeExpr::Path(":wat::Record".into())` per Stone 234.1.5. Match the exact string.

### T4 — Existing 234.3c probe must stay green
The 234.3c probe constructs records, HashMaps, structs — receivers ALL match the narrowed-allow list. Should stay 6/6 PASS.

### T5 — Stone 234.4 probe 6 (the trigger case)
Today passes (sonnet added the `i64/to-f64` slash alias as workaround). After narrowing, the alias still resolves as a registered verb — no fall-through fired. So probe 6 still passes via the alias path. Stays 6/6 PASS.

### T6 — Concrete-receiver fail case
The NEW behavior: `(:bogus 42)` where 42 is :i64 → check-time UnknownFunction. New probe needed to verify.

### T7 — Lib tests using keyword-as-accessor pattern
Some lib tests may rely on the over-permissive behavior. Substrate-as-teacher will surface them; address one of:
- Test was using doctrine-correct syntax with no substrate verb → fix test to use existing verb (mirror my own probe-author lesson)
- Test was genuinely testing the over-permissive behavior → discuss with orchestrator (likely test rewrite)

### T8 — Polymorphic-arg unresolved case
Some legitimate expressions have args whose type isn't resolved at the fall-through site (e.g., generic functions, polymorphic let-bindings). Per D6, these still get polymorphic T. Acceptable.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone3c_fix_narrow_fallthrough.rs` — contracts (4):

1. **Concrete non-record receiver fails at CHECK time** — `(let [x 42] (:bogus x))` → CheckError UnknownFunction (today: passes check, runtime UnknownFunction).
2. **Record receiver still works** — `(let [v (:Voltage 5.0)] (:magnitude v))` → 5.0 (regression check).
3. **HashMap receiver still works** — `(:port {:port 8080})` → Some(8080) (regression check).
4. **Polymorphic/unresolved receiver still accepted** — within a generic context where receiver type is unresolved, the call still type-checks. Specifically: a defn arg of generic type T should still allow `(:field x)` calls (deferred to runtime). (NOTE: if hard to construct such a case in wat without per-class TypeDef, defer this contract with NAMED follow-up; don't fake it.)

Initial state: probes 2 + 3 PASS (current behavior); probe 1 FAILS the right way (today it passes wrong-way at check time, runtime errors with UnknownFunction; new probe verifies CHECK-time UnknownFunction).

Post-stone: 4/4 PASS (or 3/4 with named-followup for probe 4 if construction is gnarly).

---

## STOP triggers

- STOP-1 unexpected compile errors
- STOP-2 lib baseline < 827
- STOP-3 45 min elapsed (small focused fix)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside check.rs
- STOP-6 scope creep
- STOP-7 234.3c regression
- STOP-8 234.4 regression
- STOP-9 any other arc 234 regression
- STOP-10 clippy > 54

Each STOP is REJECTION. If lib tests reveal usages relying on over-permissiveness: report; surface to orchestrator (don't auto-fix the consumer tests).

---

## Calibration

**Target:** 20-40 min Mode A. **Upper:** 60 min (STOP-3 with safety margin).

Surface: ~20-40 lines check.rs (the fall-through narrowing + receiver type matching).

Confidence: MEDIUM-HIGH. Focused single-file change; well-defined receiver-type discrimination; risk = uncovering consumer-test reliance on permissiveness (surface honestly if so).

---

## Cross-references

- `src/check.rs` line 5906-5908 — the over-permissive fall-through site
- `src/check.rs` line 5893-5895 — existing arg-inference recurse
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3c.md` — where the over-permissiveness was shipped
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.md` — where the consequence surfaced (probe 6)
- `feedback_no_known_defect_left_unfixed` — discipline driving this stone
