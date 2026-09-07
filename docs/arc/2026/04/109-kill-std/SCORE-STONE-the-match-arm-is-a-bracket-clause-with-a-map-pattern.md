# SCORE — RELAND 2: inline wat and `.wat.bad` fixtures

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on the previous match-arm strike + H-2*. Those stones were not reverted.

The four probe rows still PASS, including row 2 (`-1` by name) and row 4 (retired paren refused). `#[ignore]` count 0.

This ingest converted the residue the form-tree codemod cannot see: Rust string literals, and `.wat.bad` arms that are incidental scaffolding.

---

## Expectation 1 — the floor

**Not run.** Orchestrator. Previous ingest's 108 failures were this residue; the targeted tests that named those failures now pass (20/20 below).

## Expectation 2 — the 4 probe rows

**PASSED.** `cargo nextest run --release -E 'test(probe_arc109_match_arm)'`:
- `a_map_pattern_arm_binds_its_declared_keys` PASS
- `a_map_pattern_binds_by_name_not_by_position` PASS (`-1`)
- `a_unit_variant_arm_is_an_empty_map` PASS
- `the_retired_positional_clause_is_refused` PASS
- `#[ignore]` = 0

## Expectation 3 — inline-wat arms

`grep -rE '\(\(:[a-z][a-zA-Z0-9_:]*::[A-Z]' src tests --include='*.rs'` = **0**.

Hand-edits of strings (R21 does not govern strings). Bulk in `src/runtime.rs` (including `format!` braces doubled as `{{:value …}}` so Rust format strings still compile). Also `src/kernel/spawn.rs`, `src/check.rs` (parametric-enum inline programs), `@example` docs, retirement notes.

False positives reverted: a `Some((:wat::core::Vector :- [T]))` type in a comment; a `define-dispatch` clause; a `::Lost/::Closed` prose comment.

## Expectation 4 — each `.wat.bad` still fails for ITS OWN reason

Tests/ contains 11 match-containing `.wat.bad` files (the brief's 17 included cond-only fixtures, a comment-only file, a `define-dispatch` file, and two historical `docs/arc/…/complected-*.wat.bad` in old rust-scheme that no test loads). Each match-arm fixture:

| # | fixture | test | asserted error | after |
|---|---|---|---|---|
| 1 | `typed_if_match_arm_type_mismatch.wat.bad` | `match_arm_type_mismatch_named_by_arm_index` | `TypeMismatch` on `arm #2` | **held.** PASS |
| 2 | `typed_if_match_match_no_type_kw.wat.bad` | `match_with_stray_arrow_rejected` | match `no longer takes \`-> :T\`` | **held.** PASS |
| 3 | `typed_if_match_match_too_few.wat.bad` | `match_too_few_args_rejected_with_shape_guidance` | `at least a scrutinee and one arm` | no arms to migrate. **held.** PASS |
| 4 | `typed_if_match_bare_symbol_variant.wat.bad` | `match_bare_symbol_user_variant_pattern_emits_keyword_hint` | MalformedForm mentioning `:wat::kernel::LociDiedError::Panic` and `bare-symbol` | scaffolding Ok/Err/Some/None migrated. Defect arms `((Panic m)` / `((RuntimeError m)` **left as List** (they ARE the defect). infer_match now appends the arc 105 hint onto the retired-clause refusal so the named strings still appear. **held.** PASS |
| 5 | `enums_tagged_arity_mismatch.wat.bad` | `tagged_variant_arity_mismatch_reported` | 2 fields / 1 binder | now `map pattern has 1 key(s), variant \`:my::Event::Pair\` declares 2` — same count defect, new grammar wording. Golden recaptured. PASS |
| 6 | `enums_unit_pattern_on_tagged.wat.bad` | `unit_variant_pattern_on_tagged_variant_rejected` | unit pattern on tagged Pair | now `map pattern has 0 key(s), variant declares 2` — same defect. Golden recaptured. PASS |
| 7 | `enums_missing_variant.wat.bad` | `missing_variant_arm_reports_non_exhaustive` | non-exhaustive Blue | **held** (spans shifted). Golden recaptured. PASS |
| 8 | `enums_cross_enum.wat.bad` | `cross_enum_variant_pattern_rejected` | scrutinee Color vs Side + off-enum variants | **held** (spans shifted). Golden recaptured. PASS |
| 9 | `probe_hashmap_ctor_vector_symmetric_p6.wat.bad` | `probe_p6_wrong_value_type_rejected_at_type_check` | HashMap value type mismatch | **held.** PASS |
| 10 | `probe_arc170_w2a_kwargs_check_mint_swap.wat.bad` | `w2a_kwargs_check_mint_swap_is_compile_error` | `TypeMismatch` on swapped handles | **held.** PASS |
| 11 | `probe_arc170_c2_mixed_macro_swap.wat.bad` | `mixed_via_macro_swap_is_compile_error` | `TypeMismatch` on swapped Dialable | **held.** PASS |

Not migrated (not match arms): `probe_arc170_wrong_service_compile_error.wat.bad` (comment), `probe_arc241_stone13_define_dispatch_hard_cut.wat.bad` (define-dispatch), cond `.wat.bad`s, `docs/arc/…/complected-*.wat.bad`.

## Expectation 5 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. 5 dead-code warnings from the first rust half (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`). `-D warnings` would fail on those; they pre-exist this ingest.

## The rotted script (STOP-3)

`wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat` — **UnresolvedReference** `:probe::chan-svc::State/seen` at line 91. No match arms. Header already says SUPERSEDED 2026-08-05. Not an arm-grammar failure. Not touched.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 `.wat.bad` now passes | **held.** None of the 11 started passing. |
| STOP-2 fails for the NEW reason | **held** for 1–4 and 7–11 (named error unchanged). Rows 5–6 reword the same count/unit-on-tagged defect into map-key-count; goldens recaptured; still the original defect. Row 4 would have been emptied by the retired-clause refusal; the hint was re-homed onto that refusal. |
| STOP-3 rotted script fixed via arms | **held.** Diagnosed, not edited. |
| STOP-4 a `.wat` (not `.rs`/`.bad`) arm | **held.** No form-tree `.wat` hand-edits this ingest. |

## Targeted checks

```
grep -rE '\(\(:[a-z][a-zA-Z0-9_:]*::[A-Z]' src tests --include='*.rs'  →  0
cargo nextest run --release -E 'test(probe_arc109_match_arm)|…named wat.bad tests…'
  20 passed, 0 failed
cargo clippy --release --all-targets --workspace
  0 errors, 5 dead-code warnings (pre-existing)
./target/release/wat wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat
  UnresolvedReference :probe::chan-svc::State/seen
```

## Files (this ingest)

```
src/runtime.rs, src/kernel/spawn.rs, src/check.rs, src/remedy/retirement.rs,
src/intrinsic/{edn,ast,holon/atom}.rs, src/intrinsic/special/{match_form,rete_alias}.rs,
src/reflect/{lookup,verbs}.rs, src/rete/purity.rs
  inline-wat strings → new arm grammar; format! maps escaped
src/check.rs
  infer_match: List-arm refusal appends the bare-symbol variant hint
tests/**/*.wat.bad (11 match fixtures) + 4 enums__*.edn goldens
```
