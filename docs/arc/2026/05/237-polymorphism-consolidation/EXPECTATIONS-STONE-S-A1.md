# EXPECTATIONS — Stone S-A1

Mode A: 6/6 on the probe + lib baseline held (827/0) + `src/check.rs` is the ONLY
file changed.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile/build clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **S-A1 probe 6/6** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` (1 ignored) |
| 4 | S-A hierarchy regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 5 | S-B.1 recordtype regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | S-B.2 defrecord regression | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | files in scope | `git status --short` | `src/check.rs` + SCORE doc ONLY; NO Record.wat / types.rs / runtime.rs / holon-rs |

**Clippy NOT a ceiling concern** per standing direction.

## Independent prediction

**Target band: 25–45 min Mode A. STOP-3: 70 min. STOP-4 (hard kill): 95 min.**

`assignable` is ~12 lines; the 8 reroutes are one-line condition swaps. The only
variable is re-locating the 8 sites (check.rs drifts) and matching each borrow form.
Baseline-preserving by construction → no test-EXPECTATION ripple expected. If the lib
baseline moves off 827, something fired that shouldn't have (assignable is supposed to
diverge from unify ONLY on record-edge pairs) → STOP and report.

## Risks / trap-doors

1. **Directional arm inside `unify`** — the WRONG shape (sprays subtyping into
   return-position + symmetric uses = symmetric-leak class). `assignable` MUST be a
   wrapper at the arg sites, subtype-FIRST + mutation-free. (BRIEF Discipline.)
2. **`walk` vs `reduce(walk())`** — use `reduce(&walk(x, subst), subst, types)` (matches
   unify's head peel; resolves alias-of-record). Bare `walk` would miss an aliased record.
3. **Touching 14049 / 14099** — arc-146 Dispatch, retiring in 237.7. Leave them.
4. **Borrow-form mismatch** — sites vary (`&arg_ty` vs `arg_ty`, `&expected` vs
   `expected`). `assignable` takes `&TypeExpr, &TypeExpr` — match each site's existing form.
5. **6867 is clause-MATCH, not error** — reroute the condition only; keep
   `all_match = false; continue 'outer`.
6. **Constructor flip temptation** — NOT in scope. A `[c <- :my::Circle]` annotation
   already yields a subtype-typed value; the probe proves the capability without it.

## SCORE

`SCORE-STONE-S-A1.md` (NEW). 7-row scorecard + the `assignable` fn + the 8 reroute
line numbers + honest deltas + working tree. Mirror SCORE-STONE-S-A shape.
