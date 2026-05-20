# SCORE — Arc 215 Stone 2 — `[...]` Vector unification + `{...}` keyword-key lift

**Mode:** A (single agent, single session)
**Date:** 2026-05-20
**Elapsed:** ~55 min

---

## Scorecard — 22 rows

| # | Row | PASS/FAIL | Citation |
|---|---|---|---|
| 1 | `infer_list_constructor` accepts `:wat::type::Infer` for T | PASS | `src/check.rs` ~11111 — added `if k == crate::types::INFER_TYPE_PATH { fresh.fresh() }` arm before the `parse_type_expr` branch in the `WatAST::Keyword` match of `infer_list_constructor`. Mirrors Stone 1's HashSet pattern exactly. |
| 2 | Expression-position `[...]` routes through unified path | PASS | `src/check.rs` ~4641 — the `WatAST::Vector(items, span)` arm in `infer()` now synthesizes `WatAST::Keyword(INFER_TYPE_PATH, span)` and prepends it to items, then calls `infer_list_constructor`. Result type is `Parametric { head: "wat::core::Vector", args: [inferred_T] }`. |
| 3 | Binder-position `WatAST::Vector` unchanged | PASS | All other `WatAST::Vector` arms in check.rs (lines 2156, 2827, 3336, 4036, 4281, 7452, 7775, 8094, 8866, 9541, 11256) are distinct match sites inside binder handlers (`process_let_binding`, fn signature walkers, etc.) — none of these changed. arc 169 / arc 167 binder semantics intact. `probe_10_let_binder_vector_preserved` (13/13). |
| 4 | `{...}` keyword-key parse-time check dropped | PASS | `src/parser.rs` `parse_map_literal_body` — removed the per-key `WatAST::Keyword` check loop (lines 447-457 in pre-stone state). Alternating k/v odd-count rule preserved. Any non-symbol first child now routes to MapLiteral at the outer dispatch. |
| 5 | `{...}` desugar K changed to `:wat::type::Infer` | PASS | `src/parser.rs` `parse_map_literal_body` — K slot changed from `":wat::core::keyword".to_string()` to `":wat::type::Infer".to_string()`. Desugar shape is now `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer k v ...)`. `infer_hashmap_constructor` accepts `Infer` for K via Stone 1's extension. |
| 6 | Probe 1 — `[1 2 3]` integer Vec preserved | PASS | `probe_1_integer_vec_length_and_first_element` — length 3; Vector/get at index 0 → Some(1). Existing user-visible behavior unchanged by unification. |
| 7 | Probe 2 — `[1.5 2.5]` float Vec preserved | PASS | `probe_2_float_vec_length` — length 2; T inferred f64 from first element 1.5. |
| 8 | Probe 3 — `["a" "b"]` string Vec preserved | PASS | `probe_3_string_vec_length` — length 2; T inferred String. |
| 9 | Probe 4 — `[]` empty Vec preserved | PASS | `probe_4_empty_vec_length_zero` — length 0; T is fresh type variable; type-check passes. |
| 10 | Probe 5 — `[true false true]` bool Vec preserved | PASS | `probe_5_bool_vec_length` — length 3; T inferred bool. |
| 11 | Probe 6 — `(:wat::core::Vector :wat::type::Infer 1 2 3)` new path | PASS | `probe_6_explicit_infer_vector_form` — length 3. The explicit-infer verb form routes through `infer_list_constructor` via the existing `:wat::core::Vector` dispatch arm (check.rs ~4981). INFER_TYPE_PATH detection in the Keyword arm of `infer_list_constructor` fires; T concretized as i64. |
| 12 | Probe 7 — `(:wat::core::Vector :wat::type::Infer)` empty new path | PASS | `probe_7_explicit_infer_vector_form_empty` — length 0; T stays fresh. Runtime produces empty `Value::Vec`. |
| 13 | Probe 8 — `[1 "two"]` mixed-type rejection | PASS | `probe_8_mixed_type_vector_rejected_at_check` — T inferred as i64 from 1; "two" fails to unify against i64 → TypeMismatch at check. Startup fails with TypeMismatch diagnostic. |
| 14 | Probe 9 — `(:wat::core::Vector :wat::core::i64 1 2 3)` explicit type preserved | PASS | `probe_9_explicit_type_vector_form_preserved` — length 3; explicit-type path in `infer_list_constructor` unchanged (only the `INFER_TYPE_PATH` special case was added; other keywords go to `parse_type_expr` as before). |
| 15 | Probe 10 — let binder `[x 1 y 2]` preserved | PASS | `probe_10_let_binder_vector_preserved` — `(:wat::core::let [x 1 y 2] (:wat::core::+ x y))` → 3. Binder path unchanged. |
| 16 | Probe 11 — `{1 "v" 2 "w"}` int-keyed map | PASS | `probe_11_int_keyed_map_length_and_get` — length 2; HashMap/contains-key? on key 1 → true. K inferred as i64; `hashmap_key` already accepts `Value::i64` via `"I:{}"` format (runtime.rs line 8817). |
| 17 | Probe 12 — `{"a" 1 "b" 2}` string-keyed map | PASS | `probe_12_string_keyed_map_length_and_contains` — length 2; contains "a". K inferred as String; `hashmap_key` accepts `Value::String` via `"S:{}"` format. |
| 18 | Probe 13 — `{1 "v" "two" "w"}` mixed-K rejection at check | PASS | `probe_13_mixed_k_map_rejected_at_check` — K inferred as i64 from key 1; "two" (String) fails to unify against i64 → TypeMismatch at check. |
| 19 | P2 Probe 6 flipped | PASS | `tests/probe_brace_map_literal.rs` — `probe_6_non_keyword_key_rejected_at_parse` renamed to `probe_6_non_keyword_key_accepted_with_inferred_k`. Now asserts `{42 :v}` parses cleanly (Ok result) AND type-checks + evaluates to length 1. Historical note preserved in doc comment. 9/9 PASS. |
| 20 | WAT-CHEATSHEET § 8 updated | PASS | `docs/WAT-CHEATSHEET.md` — Three subsections updated: (a) `Infer` placeholder table extended with Vector row; (b) new "Three-literal unification" subsection documenting desugar shapes + escape hatch verb forms; (c) `{...}` syntax section updated to reflect K=Infer + non-keyword example; (d) new `[...]` vector literal subsection; (e) position-discipline table extended with `[...]` rows + updated `{...}` routing note. |
| 21 | arc 058 audit row added | PASS | `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/INDEX.md` — timestamped "2026-05-20 (stone 2)" entry added after the stone 1 entry. Lists all changed files, behavioral changes, test probes (13/13), and P2 Probe 6 flip. Used `git -C` pattern is inapplicable here (direct edit; file is in sibling repo accessed by absolute path). |
| 22 | CONVENTIONS updated | PASS | `docs/CONVENTIONS.md` — "Type-placeholders" section extended: placeholder table updated to include `Vector` in the "Appears in" column; three-literal desugar table added; two-layer enforcement model documented (literal coherence + function-signature unification). |

**Final PASS count: 22 / 22**

---

## Verification

```
cargo build --release
  → CLEAN (5 pre-existing dead_code warnings; 0 new warnings from arc 215 stone 2 code)

cargo test --release --test probe_arc215_stone2 -p wat
  → 13/13 PASS

cargo test --release --test probe_arc215_collection_literal_inference -p wat
  → 12/12 PASS (Stone 1 preserved)

cargo test --release --test probe_brace_map_literal -p wat
  → 9/9 PASS (P2 preserved; Probe 6 flipped)

cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
  → 9/9 PASS (P1 preserved)

cargo test --release --test wat_arc169_struct_destructure -p wat
  → 11/11 PASS (arc 169 binder paths intact)

cargo test --release --test wat_arc167_vector_ast -p wat
  → 5/5 PASS (tests 4+5 updated to assert success; tests 1-3 unchanged)

cargo clippy --release -- -D warnings
  → Pre-existing errors only (110 errors; 0 new errors from arc 215 stone 2 code)
  → My files: check.rs ~11111 — 0 new errors; parser.rs — 0 new errors;
    runtime.rs ~3943 — 0 new errors; test files — 0 errors
```

---

## Honest deltas

### D1 — Runtime path also needed updating (unexpected)

The BRIEF focused on check.rs changes. A hidden blocker: `src/runtime.rs` also had a
`WatAST::Vector(_, span)` arm at eval-position (line ~3943) that returned
`RuntimeError::MalformedForm` ("vector literals at value position are not supported").
Stone 2 had to update this arm to evaluate items and return `Value::Vec`. The arc 167
tests 4+5 that asserted the old runtime error also needed updating.

This is consistent with arc 215's stated goal (Probe 9 in arc 167 tests confirmed the
change) — the BRIEF's reference to "runtime considerations" for Change A was correct but
understated the scope. The fix is clean: evaluate each element via `eval`, collect to
`Value::Vec`. The type-checker already validated element-type uniformity, so runtime
just evaluates.

### D2 — Design choice: parser keeps `WatAST::Vector`; check.rs synthesizes `Infer` keyword

The BRIEF offered two paths for Change A:
- (a) parser emits verb-call form at expression position
- (b) check.rs internally routes through `infer_list_constructor`

Path (b) was chosen. The parser continues to emit `WatAST::Vector(items, span)` for
`[...]` at all positions (expression and binder). The `infer` function's
`WatAST::Vector` arm synthesizes the `:wat::type::Infer` keyword node and calls
`infer_list_constructor`. This keeps the parser change-free for `[...]` and avoids
the STOP-2 risk of breaking downstream walkers that pattern-match on `WatAST::Vector`.

Path (a) would have required changing the parser to distinguish expression-position
from binder-position at parse time — a subtler distinction than path (b). Path (b)
is cleaner.

### D3 — Outer brace dispatch: `Malformed` variant removed

The BRIEF anticipated dropping only the per-key keyword check inside
`parse_map_literal_body`. The outer `{...}` dispatch in `parse_primary` also had a
`BraceKind::Malformed(String)` variant for "anything else" first children. Stone 2
removes this: the enum collapses to just `MapLiteral` and `StructDestructure`. Any
non-symbol first child now routes to `MapLiteral`. The `BraceKind::Malformed` variant
is gone entirely — the restriction it enforced was exactly the arbitrary parser-layer
keyword-key constraint that Stone 2 lifts.

### D4 — `infer_list_constructor` naming (intueri Level-1 finding — logged, not fixed)

The function `infer_list_constructor` is named "list" but works on Vector. This is the
arc 109 slice 1g retirement leftover documented in the BRIEF. Logged here as honest
delta for a future arc. The function was not renamed in Stone 2 (renaming triggers
call-site sweep across multiple sites including lines 4979, 4981, 4996 — out of scope).

### D5 — arc 167 tests updated (discovery, handled)

`tests/wat_arc167_vector_ast.rs` tests 4 and 5 asserted the old "vector literals at
value position are not supported" error. Stone 2 retires that restriction, so these
tests needed updating. The update documents the historical behavior via doc comments
and asserts the new correct behavior (length 3 from `[1 2 3]`). Tests 1-3 (parse-only
checks, binder-position) were unchanged and continued passing.

---

*Three literals. One mental model. The substrate already had the inference.*
