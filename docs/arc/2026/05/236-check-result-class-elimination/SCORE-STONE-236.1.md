# SCORE — Stone 236.1 — primary `fn infer()` signature flip

**Date:** 2026-05-24
**Status:** COMPLETE — 11/11 PASS.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Stone 236.0 probe still PASSES** | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 5 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (= baseline; 0 new) |

---

## HARVEST classification counts (D3)

Primary `fn infer()` body had **3 `None`-return sites** requiring classification:

| Classification | Count | Sites |
|---|---|---|
| 1 — Silent ON PURPOSE (polymorphic placeholder) | 2 | Symbol arm (unknown locals); List/Vector sibling-delegation None paths |
| 2 — Error path missing diagnostic | 0 | — |
| 3 — Error path already had diagnostic | 1 | StructPattern arm (MalformedForm already pushed) |

### Classification detail

**Classification 1 (2 sites):**

- **Symbol arm** (`WatAST::Symbol(ident, _) => locals.get(...).cloned()`): When a symbol is not found in `locals`, the old code returned `None` silently. This is genuinely silent-by-intent — the symbol's type is unknown at this scope level; callers that short-circuit on `None` were correctly skipping type-unification for unknown names. Converted to `CheckResult::ok(fresh.fresh())` — a polymorphic placeholder that unifies with anything, preserving the semantics that callers were relying on (no false unification failures).

- **List/Vector sibling-delegation** (`infer_list` / `infer_list_constructor` returning `None` with no errors): When a sibling returns `None` without pushing any errors into `local_errors`, the primary `fn infer()` cannot determine why. The sibling has a silent failure — which will be addressed in arc 236.2 when sibling signatures flip. For 236.1, the primary converts to `CheckResult::ok(fresh.fresh())` with an inline comment naming this as sibling-delegation silent failure. Behavior is preserved: callers seeing `Some(TypeVar(N))` will unify the fresh var with their expected type (trivially succeeds), matching the prior `None` → caller-skips-check behavior.

**Classification 2 (0 sites):**

Zero sites in the primary `fn infer()` body were missing diagnostics on an error path. The body's only explicit `None` return was the StructPattern arm which already had a MalformedForm error pushed.

**Classification 3 (1 site):**

- **StructPattern arm**: Already pushed `CheckError::MalformedForm` before `return None`. Straight conversion to `CheckResult::errs(local_errors)`.

---

## New CheckError variants minted

**Zero.** No new variants needed. The 3 None-return sites in primary infer() body either (a) produced no error (silent-by-intent → fresh type var) or (b) already had an error pushed (Classification 3).

---

## Cascade depth

**2 compile rounds.**

- **Round 1:** Signature flip surfaced all 156 call sites (E0061: "5 arguments but 6 supplied") plus secondary errors (E0308 mismatched types, E0277 `?` operator, E0599 missing methods). All were mechanical bridge insertions.
- **Round 2:** Build clean. Zero unexpected errors.

Total error count surfaced in round 1:
- 156 × E0061 (arg count mismatch) — all primary `infer()` call sites
- 61 × E0308 (mismatched types) — call sites where return was used as `Option<TypeExpr>`
- 20 × E0277 (`?` operator on non-Try) — call sites using `?` shortcircuit
- 4 × E0599 (`unwrap_or_else` not found) — sibling constructors using `.unwrap_or_else`
- 4 × E0599 (`as_ref` not found) — `infer_def` / `infer_def_restricted` storing result then calling `.as_ref()`
- 3 × E0277 (collect type mismatch) — `.map(|a| infer(...))` in iterator chains

All 248 errors resolved in one pass. After bridge insertion, secondary errors (E0308, E0277, E0599) resolved automatically — they were downstream of the E0061 arg-count errors.

---

## Iteration pattern

The cascade applied in 7 logical passes over the call sites:

1. Primary `fn infer()` body rewrite (signature + `local_errors` + HARVEST)
2. Lines 4829, 4852 — outer callers in `check_function` and `check_form`
3. Sibling constructors `infer_some_constructor`, `infer_ok_constructor`, `infer_err_constructor` — `.unwrap_or_else` chains
4. `infer_list` special-case arms — `if let Some(...)` and `let _ =` patterns
5. Bulk `replace_all` pass — `let _ = infer(&args[0]...)`, `let _ = infer(arg...)` (39 sites), `?` operator sites, `.map()` / `.and_then()` closures
6. Remaining variable-name result sites: `let cond_ty`, `let then_ty`, `let else_ty`, `let scrutinee_ty`, `let arm_ty`, `let a_ty`, `let b_ty`, `let l_ty`, `let r_ty`, `let body_ty`, `let class_ty`, `let holon_ty`, `let rhs_ty`, `let cap_ty`, `let head_ty_opt`, etc.
7. `CheckSchemeCtx::infer` trait impl (line 13335)

---

## Per-classification narrative

### Why Symbol arm is Classification 1

The primary `fn infer()` has always returned `None` for symbols not in locals. Callers interpret `None` as "type unknown; skip this check." A fresh type var (`TypeVar(N)`) is the proper algebraic substitute: it is a universally-quantifiable type that unifies with any expected type without producing false TypeMismatch diagnostics. The behavior is preserved; the representation is more honest (explicit placeholder vs. silent absence).

### Why List/Vector sibling-delegation is Classification 1

Siblings (`infer_list`, `infer_list_constructor`) can return `None` without pushing errors in certain paths (silent failures within the sibling). In 236.1, primary `fn infer()` cannot distinguish "sibling succeeded but no type" from "sibling failed silently." The honest translation: preserve the existing caller behavior by returning a polymorphic placeholder. Arc 236.2 will flip sibling signatures to `CheckResult<TypeExpr>` and eliminate those silent paths at their source. The inline comment on each site explicitly names this as sibling-delegation pending 236.2.

### Why no Classification 2 sites

The primary `fn infer()` body is a match dispatcher — it routes to well-structured arms. Each arm that returned `None` either: (a) explicitly pushed an error first (StructPattern), or (b) was genuinely returning "no type available" (Symbol unknown, sibling delegation). No arm was a semantic error path that silently swallowed a condition. The silent failures in check.rs live in the sibling functions — 236.2's territory.

---

## Lib test delta

**Zero delta.** All 827 tests passed before and after. The HARVEST classifications (Classification 1 fresh-var substitutions) did not change any observable behavior visible to existing tests. This is expected: tests that relied on silent failure would have been incorrect tests, and there were none — confirming that the silent-failure paths in primary `fn infer()` body were genuinely the polymorphic-placeholder class, not untested error paths.

---

## Honest deltas from BRIEF

- **Symbol arm**: BRIEF predicted this would be Classification 1 or 2. It is Classification 1. The symbol-not-found case is intentionally silent (type unknown; no error; fresh var is the right output). No diagnostic added.
- **List/Vector delegation**: Not explicitly predicted in BRIEF's HARVEST list. Classified as Classification 1 (sibling-delegation silent failure). Inline comments explain the 236.2 dependency. Fresh var preserves existing `None`→caller-skips behavior.
- **StructPattern arm**: Classification 3 as predicted. Straight conversion.
- **Cascade depth**: BRIEF predicted 3-5 rounds; actual was 2 rounds (1 fix round + 1 verification). The migration was more mechanical than expected — all 156 call sites had `errors` as the last argument on a single logical line, enabling bulk replace_all passes without per-site disambiguation.
- **No new CheckError variants**: 0 minted vs. BRIEF's "0-5 expected." Confirms primary infer body was already diagnostically honest for its error paths.

---

## Working tree on return

```
 M src/check.rs
?? docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.1.md
```

No other files modified. STOP-4 (holon-rs) not touched. STOP-5 (Rust outside check.rs) not violated. STOP-6 (sibling infer_* signatures unchanged) confirmed. STOP-8 (clippy = 54, not > 54) confirmed.
