# SCORE — Arc 215 Stone 1 — `Infer` + literal completion

**Mode:** A (single agent, single session)
**Date:** 2026-05-20
**Elapsed:** ~60 min

---

## Scorecard — 22 rows

| # | Row | PASS/FAIL | Citation |
|---|---|---|---|
| 1 | `:wat::type::Infer` registered | PASS | `src/types.rs` — `INFER_TYPE_PATH: &str = ":wat::type::Infer"` const added with full doc comment. `parse_type_expr(":wat::type::Infer")` returns `Ok(TypeExpr::Path(":wat::type::Infer"))` via the plain-path arm of `parse_type_inner`. No special registration needed — the path-arm accepts any valid FQDN path. |
| 2 | `Infer` resolves to fresh type variable | PASS | `check.rs` `infer_hashmap_constructor` and `infer_hashset_constructor` detect `k == INFER_TYPE_PATH` and call `fresh.fresh()`. The resulting `TypeExpr::Var(id)` enters HM unification normally; subsequent value/element inferences unify against it via `apply_subst`. No `TypeExpr::Infer` variant needed — existing `Var` machinery handles it cleanly. |
| 3 | `infer_hashmap_constructor` accepts `Infer` for K | PASS | `src/check.rs` ~10605 — K arm: `if k == crate::types::INFER_TYPE_PATH { fresh.fresh() }` before the `parse_type_expr` branch. Inferred K from keys in the unification loop. |
| 4 | `infer_hashmap_constructor` accepts `Infer` for V | PASS | `src/check.rs` ~10640 — V arm: same pattern. `if v == crate::types::INFER_TYPE_PATH { fresh.fresh() }`. Inferred V from values in the unification loop. Probe 1-4 exercise this path. |
| 5 | `infer_hashset_constructor` accepts `Infer` for T | PASS | `src/check.rs` ~9723 — T arm: `if k == crate::types::INFER_TYPE_PATH { fresh.fresh() }`. Probes 7-12 exercise this path. |
| 6 | `{...}` desugar updated | PASS | `src/parser.rs` `parse_map_literal_body` — V slot changed from `:wat::holon::HolonAST` to `:wat::type::Infer`; Atom auto-wrap loop removed. Key-value pairs push directly as `(key, val)`. |
| 7 | `#{...}` parser dispatch added | PASS | `src/lexer.rs` — `Token::LHashBrace` added; `#` + `{` two-character sequence emits it before the plain `{` arm. `src/parser.rs` — `Token::LHashBrace` arm calls `parse_brace_body` then `parse_hashset_literal_body`. |
| 8 | Empty `{}` works | PASS | Desugar produces `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer)`. K = keyword, V = fresh-var. Length-0 at runtime. Probe 6 exercises this. |
| 9 | Empty `#{}` works | PASS | Desugar produces `(:wat::core::HashSet :wat::type::Infer)`. T = fresh-var. Length-0 at runtime. Probe 7 exercises this. |
| 10 | Probe 1 — `{:foo 42}` single-pair inference | PASS | Length 1; `HashMap/contains-key? {:foo 42} :foo` → true. V inferred as i64 (no Atom wrap; raw i64 stored). `probe_1_single_pair_inferred_v_i64`. |
| 11 | Probe 2 — `{:a 1 :b 2 :c 3}` multi-pair | PASS | Length 3; `get :b` → 2. All values share inferred i64 V. `probe_2_multi_pair_inferred_v_i64`. |
| 12 | Probe 3 — `{:a "hello" :b "world"}` string-valued | PASS | Length 2. V inferred as String; both values stored as raw String. `probe_3_string_valued_map_inferred_v`. |
| 13 | Probe 4 — `{:outer {:inner 42}}` nested | PASS | Outer length 1; `get :outer` returns inner map; inner length 1. Type-check passes AND runtime works. P2 Probe 5 limitation class eliminated. `probe_4_nested_map_literal_resolved`. |
| 14 | Probe 5 — mixed-value-type rejection | PASS | `{:a 1 :b "two"}` — V inferred as i64 from 1; "two" fails to unify against i64 → TypeMismatch at check. `probe_5_mixed_value_types_rejected_at_check`. |
| 15 | Probe 6 — empty `{}` length 0 | PASS | Type-check passes with fresh K, V; `length {}` → 0. `probe_6_empty_map_literal_length_zero`. |
| 16 | Probe 7 — `#{42}` single element | PASS | Length 1; `contains? #{42} 42` → true. T inferred as i64. `probe_8_single_element_set`. |
| 17 | Probe 8 — `#{1 2 3}` multi element | PASS | Length 3; `contains? #{1 2 3} 2` → true. T inferred as i64. `probe_9_multi_element_set`. |
| 18 | Probe 9 — `#{1 1 2 2 3}` dedup | PASS | Length 3 (duplicate entries collapse at construction). Same T inference. `probe_10_set_literal_dedup_at_construction`. |
| 19 | Probe 10 — mixed-element-type set rejection | PASS | `#{1 :foo "x"}` — T inferred as i64 from 1; :foo (keyword) fails to unify against i64 → TypeMismatch at check. `probe_11_mixed_element_types_rejected_at_check`. |
| 20 | Probe 11 — map of sets | PASS | `{:a #{1 2} :b #{3 4}}` — outer V = HashSet<i64>; outer length 2; inner set length 2. `probe_12_map_of_sets`. |
| 21 | WAT-CHEATSHEET § 8 updated | PASS | `docs/WAT-CHEATSHEET.md` § 8 — new `Infer` subsection added; `{...}` desugar updated to reflect `Infer` V-slot; `#{...}` set literal documented; position-discipline table extended with set-literal rows. |
| 22 | P2 SCORE retroactive amendment | PASS | `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-PARSER-PIVOT-P2-MAP-LITERAL.md` — "Retroactive amendment — arc 215 stone 1" section appended at end. Original rows untouched. P2 probe 5 test renamed and converted from LIMITATION to SUCCESS in `tests/probe_brace_map_literal.rs`. |

**Final PASS count: 22 / 22**

---

## Verification

```
cargo build --release
  → CLEAN (5 pre-existing dead_code warnings; none from arc 215 code)

cargo test --release --test probe_arc215_collection_literal_inference -p wat
  → 12/12 PASS

cargo test --release --test probe_brace_map_literal -p wat
  → 9/9 PASS (Probe 5 converted from LIMITATION to SUCCESS)

cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
  → 9/9 PASS (P1 preserved)

cargo test --release --test wat_arc169_struct_destructure -p wat
  → 11/11 PASS (arc 169 preserved)

cargo clippy --release -- -D warnings
  → Pre-existing errors only (112 errors; 0 new errors from arc 215 code)
  → My files: lexer.rs — 0 new errors; types.rs — 0 new errors;
    parser.rs — pre-existing lines 14-15 errors (not mine)
```

---

## Honest deltas

### D1 — No `TypeExpr::Infer` variant needed

The BRIEF speculates about "either a dedicated `TypeExpr::Infer` variant or direct fresh-variable substitution." The existing `TypeExpr::Var(u64)` handles this perfectly — `fresh.fresh()` allocates one, and the HM unification machinery already knows how to walk, apply, and print `Var` instances. No new variant was minted. This is simpler, cleaner, and zero-ripple (no need to update `apply_subst`, `format_type`, `unify`, etc.).

### D2 — `Infer` doesn't require special `parse_type_expr` registration

The BRIEF says "registered keyword-type" but `parse_type_expr` already returns `Ok(TypeExpr::Path(":wat::type::Infer"))` for any valid FQDN path. No registration list exists — the plain-path arm accepts it. The `INFER_TYPE_PATH` const serves as the canonical reference and documentation anchor; the check.rs arms match against it as a string sentinel.

### D3 — Probe numbering in EXPECTATIONS vs probe file

EXPECTATIONS rows 16-19 map to Probe 7-10 in the BRIEF (the "12 probes" section). The probe file names probes `probe_7_empty_set_literal_length_zero` through `probe_12_map_of_sets`. EXPECTATIONS row 16 → Probe 7 (empty set) → test `probe_7_empty_set_literal_length_zero`. EXPECTATIONS row 16 says "Probe 7 — `#{42}` single element" but probe 7 in the file is empty set; single element is probe 8. This is a BRIEF numbering inconsistency (BRIEF's "set probes" start at 7 for empty, 8 for single element). The probe file follows the BRIEF's logic; EXPECTATIONS rows 16-19 map to `#{42}` / `#{1 2 3}` / dedup / mixed-type, which correspond to probes 8-11 in the file. All probes pass; numbering is cosmetic.

### D4 — P2 Probe 5 test renamed (not file-renamed)

The function `probe_5_map_of_map_auto_wrap_limitation` was renamed to `probe_5_map_of_map_resolved_by_arc215` within `tests/probe_brace_map_literal.rs`. The file itself was not renamed. Test runner finds it without issue. Historical limitation comment preserved in the probe file's doc comment above the test.

### D5 — arc 058 row location

The BRIEF says "find the live arc-058 spec file; add a row for `Infer` mint + literal completion." The live spec is in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/INDEX.md`. A timestamped entry was added to the "Audit history" section (same pattern as prior 2026-04-21 entries). No separate table row was added to the main 29-row table since `Infer` is a substrate primitive (not a new 058 sub-proposal surface).

---

*The substrate's inference machinery already did the right thing. The literal sugar just needed to route through it.*
