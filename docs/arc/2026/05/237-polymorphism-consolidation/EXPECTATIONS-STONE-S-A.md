# EXPECTATIONS — Stone S-A

Mode A: 10/10 on the probe + clean baseline + the hierarchy mechanism shipped
(`typesub` registry + `is_subtype` + `:wat::core::subtype?` + seeded roots), with
NO `unify`-site / `conforms?` change.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **S-A probe 10/10** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | 237.1 typeunion regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 5 | 237.5 conforms? regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | 237.6 is-predicate regression | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 7 | directional + transitive + reflexive | probe 1/2/3 | `is_subtype` walks parent chain; reverse false |
| 8 | cycle rejected | probe 5 | `register_subtype` closing a cycle → `Err` |
| 9 | built-in roots + wat surface | probe 6/7/8 | `:wat::holon::Record` is-a `:wat::Record`; `subtype?` agrees; directional |
| 10 | unknown-name error contract | probe 10 | `(subtype? :my::Nonexistent :wat::Record)` → `Err` (not false) |
| 11 | holon-rs untouched | `git -C ../holon-rs status` / scope | STOP-5 — zero holon-rs changes |
| 12 | files in scope | `git status --short` | only `src/types.rs`, `src/runtime.rs`, `src/check.rs` (+ the SCORE doc) |

**Clippy NOT a ceiling concern** per standing direction.

## Independent prediction

**Target band: 40–70 min Mode A. STOP-3: 90 min. STOP-4 (hard kill): 120 min.**

Mirror of the proven `:wat::core::conforms?` mint (237.5: in-band of 40–75, 2
cascade rounds, ~367 lines). S-A is comparable but the walker is *simpler* (flat
parent-chain BFS, no TypeExpr-grammar recursion), offset by adding a `TypeEnv`
registry field + two seeded roots + the cycle-check. Net: same tier. Cascade: 2
rounds (types.rs → runtime.rs + check.rs); 0–1 forced files (only a `TypeError`
variant cascade within the 3 files if a new `CyclicSubtype` is added).

## Risks / trap-doors

1. **`is_subtype` reaching for `collect_union_members`** — the one place precedent
   misleads. It must walk the NEW `subtype_edges` registry. (BRIEF STOP-8.)
2. **Type-keyword-infers-as-`Fn`** — the `register_builtins` TypeScheme alone won't
   carry it; the `infer_list` skip-arm validating `WatAST::Keyword` on both args is
   load-bearing. (Proven trap from 237.5.)
3. **Cycle-check direction** — `register_subtype(child, parent)` must reject when
   `parent` already `is_subtype`-of `child` (mirror `check_union_no_cycle`).
4. **`TypeError` exhaustiveness cascade** — a new `CyclicSubtype` variant forces
   Display/match arms in the same files; expected, mirror `CyclicUnion`. Not a
   scope breach (same 3 files).
5. **scope creep into `conforms?`/`unify`** — both explicitly OUT (S-B / S-A1).
   `conforms?` parent-walk is NOT in S-A (no subtype-typed value to test it on).

## SCORE

`SCORE-STONE-S-A.md` (NEW). 12-row scorecard verbatim + Final API shape
(`register_subtype` / `is_subtype` / `:wat::core::subtype?` / seeded roots) + line
counts per file + cascade depth + honest deltas (incl. `:wat::holon::is-Record?`
auto-synthesis + any `TypeError` cascade). Mirror Stone 237.5 SCORE shape.
