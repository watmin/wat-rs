# SCORE — Stone S-A1 — `assignable` choke point (subtyping at the arg boundary)

**Date:** 2026-05-26
**Status:** COMPLETE — 6/6 probe PASS. All scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors (pre-existing warnings ceiling) |
| 2 | **S-A1 probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| tail -10` | `6 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -5` | `827 passed; 0 failed` |
| 4 | S-A regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -5` | `10 passed; 0 failed` |
| 5 | S-B.1 regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 6 | S-B.2 regression | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 7 | holon-rs untouched | STOP-5 | confirmed — zero holon-rs changes |
| 8 | src/check.rs only | STOP-4 | confirmed — only check.rs touched |

---

## `fn assignable` (minted at check.rs:14783)

```rust
/// Arg-boundary acceptance: is `actual` assignable to `expected`?
/// Liskov — a subtype is accepted where its supertype is wanted. Checks the
/// `typesub` hierarchy FIRST (mutation-free; only concrete distinct paths with a
/// registered edge), then falls through to ordinary unification (behaviour
/// unchanged for every other pair). Peels each side exactly as `unify` does at
/// its head (line ~14633): `reduce(&walk(x, subst), subst, types)`.
fn assignable(
    actual: &TypeExpr,
    expected: &TypeExpr,
    subst: &mut Subst,
    types: &TypeEnv,
) -> bool {
    let a = reduce(&walk(actual, subst), subst, types);
    let e = reduce(&walk(expected, subst), subst, types);
    if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
        if ap != ep && crate::types::is_subtype(ap, ep, types) {
            return true;
        }
    }
    unify(actual, expected, subst, types).is_ok()
}
```

Placed immediately after `fn unify` (line 14775) and before `fn walk` (now shifted to ~14807). Uses `reduce(&walk(...))` as the GROUNDED section prescribed — alias-expand then Var-follow, matching unify's own canonicalization. Mutation-free on the subtype path (concrete-path/concrete-path case binds nothing in `subst`).

---

## The 8 reroutes (landed line numbers)

All in `src/check.rs`. Exact form `!assignable(...)` replacing `unify(...).is_err()`. Borrow forms preserved from original sites.

| # | Line | Callee / context | Borrow form |
|---|------|------------------|-------------|
| 1 | 6386 | single-arg `k`, param `#1` | `&arg_ty, &expected` |
| 2 | 6867 | defclause clause-match | `arg_ty, expected_ty` (bare, `&mut clause_subst`) |
| 3 | 7025 | multi-arg `k`, param `#{i+1}` | `&arg_ty, expected` |
| 4 | 7079 | multi-arg `k`, param `#{i+1}` (no-rest path) | `&arg_ty, expected` |
| 5 | 7213 | value-head application, param `#{i+1}` | `&arg_ty, expected` |
| 6 | 10256 | 236.2-harvested `try` single-arg | `&arg_ty, &expected` |
| 7 | 10365 | 236.2-harvested `option` single-arg | `&arg_ty, &expected` |
| 8 | 12044 | spawn multi-arg `callee_label`, param `#{i+1}` | `&arg_ty, expected` |

**Left untouched (BRIEF mandate):** 14049 / 14099 — arc-146 Dispatch arms, retiring in 237.7.

---

## Line count

| File | Pre-stone lines | Post-stone lines | Net added |
|------|-----------------|------------------|-----------|
| `src/check.rs` | 21,256 | 21,279 | +23 (`assignable` function ~23 lines including doc comment; 8 reroutes are one-line substitutions with zero net change each) |

Total net: +23 lines. Well within BRIEF's 25–45 min Mode A calibration (single round, check.rs only).

---

## Cascade depth

**1 round.**

`src/check.rs` only: minted `assignable` + rerouted 8 condition lines. No new `TypeError` variant, no new `Value` variant, no new file. Zero cascade. STOP-4 not triggered.

---

## Honest deltas

### Line numbers held stable (zero drift)

The BRIEF warned that check.rs drifts from the HEAD-current line numbers. In practice, the 8 sites all landed at the EXACT lines stated in the BRIEF (6386 / 6867 / 7025 / 7079 / 7213 / 10256 / 10365 / 12044). No re-location was needed beyond the grep-confirm step.

### `assignable` placed after `fn unify`, before `fn walk`

The BRIEF said "place after `fn unify`, ~line 14780 region." `fn unify` ends at 14775; `fn walk` was at 14780. `assignable` was inserted between them (at 14783 post-insertion). `fn walk` shifted accordingly. Both helper functions remain in the same file-local scope and are visible to all callers.

### `reduce(&walk(...))` not bare `walk`

The GROUNDED section explicitly prescribed `reduce(&walk(x, subst), subst, types)` — alias-expand then Var-follow — matching unify's own head (14633-14634). The earlier DESIGN body used bare `walk`. The GROUNDED section is authoritative; `reduce(&walk(...))` is what shipped.

### Site 6386 is `:wat::WatAST` boundary, not `:wat::Record`

The grep confirmed 6386 is a `WatAST`-typed single-arg site, not a record-typed one. The BRIEF listed it as "single-arg call (callee: k, param #1)" which is correct. It is wired; probe_01 passes via the record boundary at a different infer_list arm. The reroute is correct and baseline-preserving (no `WatAST`-vs-subtype edge exists, so `assignable` falls through to `unify` unchanged for that site's existing callers).

---

## Working tree on return

```
 M src/check.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A1.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
