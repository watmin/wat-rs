# Arc 146 Slice 1 — Pre-handoff expectations

**Drafted 2026-05-03.** Substantial substrate slice — multimethod
entity + registry + parse + dispatch (check + runtime) + reflection
extension + tests. Predicted MEDIUM-LARGE slice (Mode A ~50%; Mode
B-freeze-order ~15%; Mode B-arity-check-shape ~15%; Mode B-unify-
arm-pattern ~10%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-1.md`
**Output:** 1 NEW Rust file (`src/multimethod.rs`) + 1 NEW test
file + edits to `src/lib.rs` + `src/runtime.rs` + `src/check.rs` +
`src/freeze.rs` + `src/special_forms.rs`. ~600-1000 LOC + report.

## Setup — workspace state pre-spawn

- Arc 144 closed through slice 3; slice 4 (verification) pending.
  Reflection foundation in place: Binding enum, lookup_form, 3
  reflection primitives, special-form registry.
- Arc 144 slice 3b CANCELLED (per arc 144 REALIZATION 4); arc 146
  is the architectural alternative.
- 1 in-flight uncommitted file (CacheService.wat — arc 130;
  ignore).
- Workspace baseline (per FM 9): all green except slice 6 length
  canary + the wat-lru noise from CacheService.wat.

## Hard scorecard (11 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | NEW `src/multimethod.rs` + NEW `tests/wat_arc146_multimethod_mechanism.rs`; MODIFIED `src/lib.rs` (1 line) + `src/runtime.rs` (SymbolTable field + setter + Binding variant + lookup_form branch + 3 reflection-primitive arms + runtime dispatch insertion + helper) + `src/check.rs` (CheckEnv accessor + infer_list dispatch insertion + `infer_multimethod_call` helper) + `src/freeze.rs` (is_mutation_form + dispatch arm) + `src/special_forms.rs` (defmultimethod registration). NO wat files. |
| 2 | `Multimethod` + `MultimethodArm` + `MultimethodRegistry` | All present in src/multimethod.rs with the brief's shape. `pub` visibility throughout (test code pattern-matches). |
| 3 | `is_defmultimethod_form` + `parse_defmultimethod_form` | Parse function returns `Result<Multimethod, MultimethodError>`. Parses the `(:wat::core::defmultimethod :name ((pattern...) impl-kw)+)` shape. Each arm's pattern is `Vec<TypeExpr>`; impl-kw is the impl's keyword path. |
| 4 | `SymbolTable.multimethod_registry` field + setter | Mirrors `macro_registry` shape exactly. `Option<Arc<MultimethodRegistry>>`. |
| 5 | Freeze-time recognition | `is_mutation_form` includes `:wat::core::defmultimethod`. Freeze processes defmultimethod forms via the new parse path; registers into SymbolTable's MultimethodRegistry. |
| 6 | Check-time dispatch | `infer_list` head-keyword switch checks multimethod registry BEFORE the existing arms; routes to `infer_multimethod_call` if matched. The helper uses existing `unify` for arm-pattern matching against arg types. Returns matched arm's impl scheme's instantiated return type. |
| 7 | Runtime dispatch | `eval_list_call` (or equivalent dispatch site) checks multimethod registry similarly; routes to `eval_multimethod_call` if matched. Helper matches value tags against arm patterns; calls matched impl. |
| 8 | Arc 144 extensions | `Binding::Multimethod` variant added (with `'a` lifetime tying to the registry's borrowed data). `lookup_form` 6th branch. Each of the 3 reflection primitive (lookup_define / signature_of / body_of) match arms handle the Multimethod variant: lookup_define → declaration form; signature_of → declaration form; body_of → :None. |
| 9 | New test file | `tests/wat_arc146_multimethod_mechanism.rs` with 6+ tests covering: dispatch i64/f64 arms, no-arm-match check-time, reflection (lookup_form / signature_of / body_of), arity mismatch error. ALL pass. |
| 10 | **Baseline tests still pass** | `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9 (or 10/10 if defmultimethod adds); `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8; `wat_arc143_define_alias` 2/3 (length canary unchanged — slice 2 closes). |
| 11 | Honest report | ~300-word report covers all required sections from the brief; honest deltas surfaced; decisions on open questions named. |

**Hard verdict:** all 11 must pass. Rows 6 + 7 + 8 are the
load-bearing rows (mechanism + reflection working end-to-end).

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 12 | LOC budget | Total slice diff: 600-1000 LOC. >1300 LOC = re-evaluate scope. |
| 13 | Style consistency | New code mirrors existing patterns: macros.rs for entity+registry shape; macro_registry for capability-carrier; arc 144's slice 1 reflection-helper style; existing infer_*/eval_* helpers for new dispatch helpers. |
| 14 | clippy clean | No new warnings. |
| 15 | Open-question decisions named | Sonnet's report names what was chosen for Q1 (arity check timing), Q2 (helper placement), Q3 (precedence), Q4 (freeze ordering). |
| 16 | Audit-first discipline | If sonnet finds the brief's substrate assumptions don't match reality (e.g., `eval_list_call` doesn't exist by that name; the dispatch site has a different shape), surface as honest delta with the actual file:line + adapt. |

## Independent prediction

- **Most likely (~50%) — Mode A clean ship.** Brief is detailed +
  pre-flighted; the patterns sonnet mirrors are well-established.
  ~30-50 min wall-clock (largest slice in arc 146; subsequent
  migration slices are smaller).
- **Surprise on freeze ordering (~15%) — Mode B-freeze-order
  (Q4).** Sonnet finds defmultimethod can't see the impls because
  freeze processes mutation forms in source order; the multimethod
  declaration is parsed before its impls register. If hit:
  surface clean; orchestrator decides scope (move declaration
  registration to a later pass; or require declaration to come
  AFTER impls in source).
- **Surprise on arity check (~15%) — Mode B-arity (Q1).** Sonnet
  finds parse-time arity check needs CheckEnv access that's not
  available at parse time; defers to first-call (the brief
  recommendation). Or finds parse-time IS available; ships there.
  Either is fine; report the choice.
- **Surprise on unify for arm patterns (~10%) — Mode B-unify.**
  The existing `unify` may not handle the arm-pattern shape
  cleanly (e.g., the arm pattern's type-vars need fresh
  instantiation per call site). Sonnet adapts; reports.
- **Borrow-check / lifetime friction (~10%) — Mode C.** The
  Binding<'a>::Multimethod variant ties to the multimethod
  registry's borrowed data; lifetime propagation through
  lookup_form may surface friction. Adapts.

**Time-box: 100 min cap (2× upper-bound 50 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 6 file changes expected (5 modified, 2 new).
4. Read `src/multimethod.rs` end-to-end — verify struct shapes + parse function.
5. Read the dispatch insertions in check.rs + runtime.rs.
6. Read the Binding extension + lookup_form branch.
7. Run the new test file + baseline tests.
8. Run `cargo test --release --workspace` — confirm baseline failure profile.
9. Run `cargo clippy --release --all-targets`.
10. Score; commit `SCORE-SLICE-1.md`.

## What this slice unblocks

- **Slice 2** — migrate `length` as the canonical first migration.
  The mechanism does the heavy lifting; slice 2 just declares
  the multimethod + retires the handler.
- **Slices 3-6** — migrate empty?, contains?, get, conj families.
  Same shape as slice 2.
- **Slice 7** — pure rename family (no multimethod needed).
- **Slice 8** — closure.
- **Arc 144 slice 4** — verification simpler post-arc-146-slice-2
  (length canary turns green via the migration, not via a wrapper).

The substrate gains an honest entity for polymorphic dispatch.
Foundation strengthens by one entity kind. Per § 12: each slice
compounds.
