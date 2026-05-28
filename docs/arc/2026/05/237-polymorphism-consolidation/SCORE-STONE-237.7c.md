# SCORE — Stone 237.7c — `:wat::core::assoc` polymorphic HashMap+Record intrinsic

**Shadowdancer independent verify — 2026-05-27**

## Scorecard

| # | Row | Result | Value |
|---|-----|--------|-------|
| 1 | compile clean | PASS | `cargo build --release -p wat 2>&1 \| grep -c "^error"` → 0 |
| 2 | **probe green (LOAD-BEARING)** | PASS | `6 passed; 0 failed; 0 ignored` |
| 3a | **test-build gate (LOAD-BEARING)** | PASS | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` → 0 |
| 3b | **lib baseline (LOAD-BEARING)** | PASS | `834 passed; 0 failed; 1 ignored` |
| 4 | **MECHANISM — alias gone** | PASS | `grep -c "define-alias :wat::core::assoc" wat/core.wat` → 0 |
| 5 | **MECHANISM — tombstone in place** | PASS | `grep -c "237.7c" wat/core.wat` → 1 |
| 6 | **MECHANISM — infer_assoc helper** | PASS | `grep -c "fn infer_assoc" src/check.rs` → 1 |
| 7 | **MECHANISM — custom arm + eval arm wired** | PASS | `grep -c '":wat::core::assoc"' src/check.rs src/runtime.rs` → check.rs:3, runtime.rs:4 (≥ 2) |
| 8 | **TRAP — HashMap arm uses V for arg2** | PASS | Probe row `assoc_hashmap_wrong_value_type_rejected_at_check` green; arg2 unifies with `val_ty` (V), not `key_ty` (K) |
| 9 | **TRAP — Record arm uses :keyword for arg1** | PASS | Both base + holonic Record probe rows green; arg1 unifies with `:wat::core::keyword` |
| 10 | **TRAP — Record arm does NOT unify arg2** | PASS | Record arm has `// arg2 is free ∀T — no unification` comment; no unify call within the arm for arg2 |
| 11 | **PARITY — holonic flavor preserved** | PASS | `assoc_holonic_record_returns_holonic_record_parity_preserved` green; routes through `eval_record_assoc` which rebuilds both struct_form + holon_form |
| 12 | NO touch of per-Type leaves | PASS | `eval_record_assoc` and `hashmap_assoc_inner` bodies unchanged; only routed INTO |
| 13 | scope | PASS | 5 files touched: `src/check.rs`, `src/runtime.rs`, `wat/core.wat`, `tests/probe_arc237_7c_assoc_polymorphic.rs`, `SCORE-STONE-237.7c.md`; NO holon-rs; NO per-Type leaf modifications |

## Mode classification

**Mode A.** All rows green. No STOP triggers fired.

## One behavioral lib-test update

`runtime::tests::assoc_on_vec_rejects_post_slice4` expected `op == ":wat::core::HashMap/assoc"` (the old alias's delegated op name). The new `eval_assoc` else-arm returns `op == ":wat::core::assoc"`. This is a correct mechanism-swap identical to what 7b-iv did for `hashmap_get_requires_hashmap_arg`. The behavior is preserved (Vec still rejected); only the op name in the error changes. Comment and assertion updated. Lib count stays 834/0.

## One twist not in BRIEF (surfaced)

The BRIEF specified `TypeExpr::Path(p) if p == ":wat::Record"` for the Record arm. At runtime, `(:wat::holon::Record::def ...)` creates a type that reduces to `TypeExpr::Path(":wat::holon::Record")`, not `:wat::Record`. The base record constructor returns `:wat::Record`; the holonic constructor returns `:wat::holon::Record`. This mirrors the pattern already in `check.rs` at line 6398 where BOTH paths are handled as record umbrellas. The arm was extended to match both:

```rust
TypeExpr::Path(p) if p == ":wat::Record" || p == ":wat::holon::Record" => { ... }
```

This is consistent with the Liskov intent — both flavors satisfy the umbrella; flavor is a runtime property. The return type is always `TypeExpr::Path(":wat::Record")` (umbrella). No STOP trigger — the BRIEF's description "the umbrella IS the Path" is honored; there are two umbrella paths (base + holonic), both handled identically by the Record arm. The probe's holonic row is the load-bearing proof.

## STOP triggers

None fired.

## Definition of done — all criteria met

- [x] All 6 probe tests green; 0 ignored
- [x] test-build 0 errors
- [x] lib 834/0
- [x] `wat/core.wat` no longer has the `:wat::core::assoc` alias line; tombstone comment in place
- [x] `src/check.rs` has `fn infer_assoc` + dispatch arm + `:wat::core::assoc` fallback TypeScheme
- [x] `src/runtime.rs` has `fn eval_assoc` + dispatch arm
- [x] probe's two `#[ignore]` annotations removed
- [x] Only scoped files touched; no holon-rs; no per-Type leaf body modifications; no other alias retirement; no registry deletion

## Shadowdancer runtime

~20 min (within the 15-25 min Mode-A band; well under the 50-min wakeup).
One remediation loop: the holonic Record type path (`:wat::holon::Record` vs `:wat::Record` arm mismatch, caught and fixed cleanly before submitting).
