# SCORE — Stone 237.8a — Arithmetic + Comparison HARD CUT under THE DECISION

**Mode A.** All rows green. Consumer cascade = 0 sites. Runtime clean.

## FM-9 Independent scorecard

| # | Row | Verify | Result |
|---|-----|--------|--------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | **0** |
| 2 | **probe green (LOAD-BEARING)** | `cargo test --release --test probe_arc237_8a_no_implicit_coercion 2>&1 \| grep "test result"` | **9 passed; 0 failed; 0 ignored** |
| 3a | **test-build (gate part 1)** | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` | **0** |
| 3b | **lib baseline (gate part 2)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | **834 passed; 0 failed; 1 ignored** |
| 4 | define-dispatch decls gone | `grep -c "define-dispatch :wat::core::" wat/core.wat` | **0** |
| 5 | tombstone in place | `grep -c "237.8a" wat/core.wat` | **1** |
| 6 | infer_arithmetic tightened (no any_f64) | `awk '/fn infer_arithmetic/,/^}/' src/check.rs \| grep -c "any_f64"` | **0** |
| 7 | infer_comparison tightened (cross-numeric gone) | manual diff inspection | **deleted** (replaced with tombstone comment + fallthrough to unify) |
| 8 | lexer mixed-type entries gone | `grep -cE "op'f64'i64\|op'i64'f64" src/lexer.rs` | **0** |
| 9 | mixed-type leaves retired | `grep -cE "'i64'f64\|'f64'i64" src/check.rs src/runtime.rs` | **8 refs — all inside tombstone comments** |
| 10 | per-Type leaves kept | `grep -c "i64::+'2\|f64::+'2" src/check.rs src/runtime.rs` | **102 refs (registered + dispatched + referenced)** |
| 11 | per-Type variadic wat fns kept | `grep -c "wat::core::i64::+ \|wat::core::f64::+ " wat/core.wat` | **2** |
| 12 | DispatchRegistry untouched | `grep -c "DispatchRegistry" src/check.rs src/runtime.rs` | **7 (check.rs: 3, runtime.rs: 4) — unchanged from HEAD** |
| 13 | holon-pair handlers untouched | `git diff HEAD src/check.rs \| grep -c "fn infer_polymorphic_holon_pair"` | **0** |
| 14 | time-arith handler untouched | `git diff HEAD src/check.rs \| grep -c "fn infer_polymorphic_time_arith"` | **0** |
| 15 | scope | `git status --short` | **5 substrate files only: check.rs, runtime.rs, lexer.rs, core.wat, probe** |
| 16 | consumer cascade accounted for | see section below | **0 sites — cascade was empty** |

## Files touched

```
src/check.rs                                  | 135 ++/--
src/lexer.rs                                  |   7 +/-
src/runtime.rs                                | 286 +/--  (heavy delete)
tests/probe_arc237_8a_no_implicit_coercion.rs |   3 -
wat/core.wat                                  |  58 ++/--
```

5 files. 124 insertions, 365 deletions. Net: heavy cut.

## What was changed per file

### `wat/core.wat`

- DELETED 4 `define-dispatch` decls (+'2, -'2, *'2, /'2) — 24 lines of
  cross-type arm routing (i64→f64, f64→i64 mixed arms).
- REPLACED with tombstone comment (arc 237 Stone 237.8a signature) +
  updated architecture comment reflecting two-layer (not three-layer)
  post-decision shape.

### `src/check.rs`

- TIGHTENED `infer_arithmetic` docstring — removed f64-promotion language;
  added THE DECISION statement.
- TIGHTENED `infer_arithmetic` 2+-ary body (lines ~13280-13289):
  - DELETED: `any_f64` / `all_known_numeric` / widest-contagion logic.
  - ADDED: `all_i64` / `all_f64` match; mixed → TypeMismatch naming the
    first non-matching arg (finds the type mismatch via `unify`), falls
    back to f64 return type after pushing the error.
- TIGHTENED `infer_comparison` docstring — removed cross-numeric-promotion
  framing; added THE DECISION statement.
- DELETED `infer_comparison` cross-numeric path (3 lines):
  ```rust
  if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
      return ...; // accepts (i64, f64) silently
  }
  ```
  Replaced with tombstone comment. (i64, f64) now falls through to the
  same-type-or-subtype unify check, which rejects it correctly.
- DELETED 8 mixed-type leaf registrations from `register_builtins`:
  `+'i64'f64`, `-'i64'f64`, `*'i64'f64`, `/'i64'f64`,
  `+'f64'i64`, `-'f64'i64`, `*'f64'i64`, `/'f64'i64`.
  Replaced with tombstone comment.

### `src/runtime.rs`

- TIGHTENED `apply_arith_pair`: deleted the `(Value::i64, Value::f64)` and
  `(Value::f64, Value::i64)` arms (16 lines of cross-type promotion). Both
  now fall to the catch-all TypeMismatch arm with updated expected-message:
  "matching numeric pair (i64, i64) or (f64, f64)".
- DELETED 8 eval match arms for mixed-type leaves (`+'i64'f64` through
  `/'f64'i64`) — 44 lines including their closures.
- DELETED `fn eval_i64_f64_arith` — 40-line AST-level helper.
- DELETED `fn eval_f64_i64_arith` — 40-line AST-level helper.
- DELETED 8 arms from `dispatch_substrate_impl` match for the mixed-type
  Value-level leaves.
- DELETED `fn arith_i64_f64_inner` — 23-line Value-level helper.
- DELETED `fn arith_f64_i64_inner` — 23-line Value-level helper.
- DELETED 8 entries from the `step_descend_then_fire` canonical list.
- All deletions replaced with tombstone comments.
- Updated `dispatch_substrate_impl` docstring: "4 same-type i64-i64 +
  4 same-type f64-f64 = 8 leaves. Mixed-type leaves DELETED."

### `src/lexer.rs`

- DELETED 2 mixed-type test entries from `keyword_apostrophe_full_op_table`:
  `":wat::core::op'f64'i64"` and `":wat::core::op'i64'f64"`.
  Same-type `":wat::core::op'f64'f64"` retained as coverage.
  Tombstone comment explains the deletion.

### `tests/probe_arc237_8a_no_implicit_coercion.rs`

- REMOVED 3 `#[ignore = "..."]` annotations:
  - `arith_i64_f64_mixed_rejected_at_check`
  - `arith_f64_i64_mixed_rejected_at_check`
  - `comparison_i64_f64_mixed_rejected_at_check`

## Consumer-sweep cascade

**0 sites migrated.** The `cargo build --release --tests --workspace` found
zero cross-type arithmetic or comparison call sites in the workspace. All
existing callers were already type-homogeneous. The cascade the BRIEF
predicted as "likely small but unbounded" turned out to be empty.

Spot-check rationale (pre-stone from DESIGN-STONE-237.8.md): "~20 files use
bare `:wat::core::+`/`-`/`*`/`/`. Spot-checks (`:wat::core::- 0.0 ratio` =
f64+f64, `:wat::core::+ (...) 1` = i64+i64, `:wat::core::* 2.0 pi` = f64+f64)
suggest most callers are already type-homogeneous." Confirmed — zero migration
required.

No `.wat` consumer files touched. No explicit `(:wat::core::i64::to-f64 ...)`
homogenization calls added (none needed). No lab files touched. No holon-rs
touched.

## Runtime count delta

- **Removed**: 8 mixed-type leaf registrations from check.rs
- **Removed**: 8 mixed-type eval arms + 2 AST-level helpers + 2 Value-level
  helpers + 8 dispatch_substrate_impl arms + 8 step_descend_then_fire entries
  from runtime.rs
- **Removed**: 4 define-dispatch decls (12 mixed-type arms total) from core.wat
- **Kept**: 8 per-Type same-type leaves (i64::+'2 through f64::/'2) in both
  check.rs and runtime.rs
- **Kept**: 8 per-Type variadic wat fns (i64::+, i64::-, ...) in core.wat

Net: ~26 runtime-registered callables retired. DispatchRegistry still present
(8b's job to delete once it becomes 0-tenant).

## STOP-trigger report

**No STOP triggers fired.**

- Did not add implicit coercion back anywhere.
- Did not touch `infer_polymorphic_holon_pair_*` or `infer_polymorphic_time_arith`.
- Did not delete `src/dispatch.rs` or any `DispatchRegistry` use site.
- Did not delete per-Type leaves.
- Consumer cascade required zero lab/holon-rs touches.
- No `#[ignore]` re-added.
- `infer_polymorphic_holon_pair_to_f64` / `..._to_bool` / `..._to_path`
  and `infer_polymorphic_time_arith` were inspected (git diff confirms zero
  body changes) — they are legitimately different polymorphism, no cross-type
  falsehood, not in scope.

## Mode classification

**Mode A.** All rows green. Consumer cascade = 0 sites (well within ≤ 10).
Both handler tightenings are structurally clean (one match-arm replacement
each: `infer_arithmetic` 2+-ary block, `infer_comparison` 3-line delete).
STOP-2 delta: 0.

## Shadowdancer runtime

Substrate edits: ~20 min. All four verification commands: pass on first
attempt. Consumer cascade: instantaneous (0 errors). Total: ~25 min.
Within predicted 30-50 min Mode-A band.

## On green — advance

237.8a shipped. THE DECISION applied to arithmetic + comparison. The
cross-numeric falsehood deleted across the substrate. `define-dispatch` decls
evacuated. Mixed-type leaves retired. Consumer cascade confirmed empty.

Remaining in arc 237:
- **237.8b** — DispatchRegistry HARD CUT (0-tenant registry deletion). The
  4 `define-dispatch` decls were the last tenants; they are gone. The
  registry itself (`src/dispatch.rs` + `dispatch_registry` field +
  set/get methods + guard at check.rs:5460) is now a 0-tenant structure
  awaiting cleanup.
- **237.9** — INSCRIPTION (folds arc 146 + arc 148 + arc 237; USER-GUIDE
  records-doctrine sentence + THE DECISION as canonical reference).
