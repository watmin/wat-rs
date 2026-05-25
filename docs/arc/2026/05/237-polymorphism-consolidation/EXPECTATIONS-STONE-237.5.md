# EXPECTATIONS — Stone 237.5

Mode A: 12/12 PASS on the probe + clean baseline.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **conforms? probe 12/12** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | 237.1 typeunion regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 5 | 237.2 defclause regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 7 | 237.4 regression | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 8 | arc 234.0 `type` regression (sibling primitive) | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 9 | unknown-type errors, not false | (probe 12) | `is_err`, message names the type expr |
| 10 | u8 ≠ i64 end-to-end | (probe 4) | u8→`:u8` true, u8→`:i64` false |

**Clippy NOT a ceiling concern** per user direction (arc 109 closure sweeps).

## Prediction

**Target: 40–75 min Mode A. STOP-3: 100 min. STOP-4 (hard kill): 150 min.**

Surface estimate:
- `src/runtime.rs`: ~80–160 lines (dispatch arm + `eval_conforms` + recursive `conforms` walker + `concrete_type_name` helper).
- `src/check.rs`: ~20–50 lines (inference scheme; type-position arg handling).
- **Total: ~100–210 lines** across 2 files.

Confidence: HIGH. Every composed mechanism verified present (`sym.types()` access pattern at runtime.rs:7530; `collect_union_members` at types.rs:3031; `class_fqdn`/`type_name` extraction; TypeExpr grammar). The recursive walker is the only new logic; it's a clean match over 5 variants.

## Risks

1. **`Path(name)` string form** — does the parsed `TypeExpr::Path` carry a leading `:` or not? `concrete_type_name` must match the same form (`class_fqdn` is colon-free; `type_name()` returns `"wat::core::i64"` colon-free per runtime.rs:1224). Normalize both sides. Trap-door 1.
2. **Type-position arg in check** — the 2nd arg is a type keyword, not a value. If the checker value-infers it, it'll error. Mirror `:wat::core::type` / `-> :T` handling. Trap-door 2.
3. **Parametric classifier match** — confirm how a Vector value's classifier is read at runtime (arc 228/230 classifier-wrap). Element iteration over the collection's contents. Trap-door 3.
4. **Builtin-primitive Path resolution** — `:wat::core::i64` etc. are NOT in the user `TypeEnv` as Struct/Enum/Newtype; they resolve via `type_name` identity directly. Don't error on them as "unknown." Distinguish "builtin primitive path" from "genuinely unknown name." Trap-door 4.
5. **Alias target field** — confirm `AliasDef`'s target field name (`.target` assumed; verify) for the recurse.

## Out-of-scope (REJECTED)

- `is-<Name>?` auto-mint (Stone 237.6).
- Fn-type structural conformance (errors "unsupported").
- Deep parametric user-type introspection (nominal head match only).
- Migration / arithmetic deletion (237.7/237.8).
- holon-rs (STOP-5).

## SCORE

`SCORE-STONE-237.5.md` (NEW). 12-row scorecard verbatim + final signature + recursive-arm table + line counts + cascade depth + honest deltas. Mirror Stone 237.2 SCORE shape.
