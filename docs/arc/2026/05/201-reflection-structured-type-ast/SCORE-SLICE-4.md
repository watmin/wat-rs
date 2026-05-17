# SCORE — Arc 201 Slice 4 — `signature-of` → `signature-of-defn` rename

**Slice:** 4 of 6 (see DESIGN.md § Stepping stones).
**Date:** 2026-05-16.
**Predecessors:** slice 1 (`0706949`) — structured type-AST emission; slice 2 (`c9445a4`) — `Bundle/children` + `Bundle/first` accessors; slice 3 (`815d597`) — `signature-of-fn` primitive.

## SCORE rows

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `:wat::runtime::signature-of` is GONE from substrate registration (`check.rs` `env.register`); `:wat::runtime::signature-of-defn` REGISTERED | **YES** | `grep -n "\"\\:wat\\:\\:runtime\\:\\:signature-of\"" src/check.rs` returns zero matches. `grep "signature-of-defn" src/check.rs` returns the `env.register` entry at ~line 14192 and the `infer_list` dispatch arm at ~line 4721. The `":wat::runtime::signature-of"` string literal is gone from all of `src/`. |
| B | Internal Rust identifiers renamed (`eval_signature_of` → `eval_signature_of_defn`); slice 3 sibling preserved (`eval_signature_of_fn` unchanged) | **YES** | `fn eval_signature_of_defn` defined in `src/runtime.rs`; `const OP: &str = ":wat::runtime::signature-of-defn"` inside it. `eval_signature_of_fn` at line ~9902 unchanged — `grep eval_signature_of_fn src/runtime.rs` returns dispatch arm + fn definition + OP constant all correctly spelled. `grep "eval_signature_of[^_]" src/` returns zero matches (no bare `eval_signature_of` without a suffix). |
| C | `wat/runtime.wat` `define-alias` macro uses new name; macro expansion still works | **YES** | Both `(:wat::runtime::signature-of-defn target-name)` calls in `wat/runtime.wat` updated. `cargo test --test wat_arc143_define_alias` passes: `define_alias_foldl_to_user_fold_delegates_correctly`, `define_alias_length_to_user_size_delegates_correctly`, `define_alias_unknown_target_panics_at_expand_time` — all 3 pass. The macro invokes `signature-of-defn` at expand-time; substrate dispatch is live. |
| D | 13 test files swept; all calls updated; all pass | **YES** | All 13 test files updated. Per-file verification: `cargo test --release --workspace --no-fail-fast` — every test that was passing pre-rename still passes. Tests for the 13 files: arc143_lookup (4 renamed fns), arc143_define_alias (3), arc143_manipulation (multiple), arc136_do_form (1), arc144_lookup_form (multiple renamed fns), arc144_uniform_reflection (multiple), arc144_special_forms (multiple), arc144_hardcoded_primitives (15 renamed fns + helper), arc146_dispatch_mechanism (1 renamed fn), arc150_variadic_define (1 renamed fn), arc201_signature_of_fn (docstring only — fn calls remain `signature-of-fn`), arc201_holon_ast_accessors (all calls), arc201_structured_signature_types (all calls + 4 renamed fns). Passed 0 → failure count post-rename = same as pre-rename. |
| E | Workspace failure count ≤ baseline (EXPECTATIONS-SLICE-4 baseline: ≤ 3 documented failures) | **YES (with disclosure)** | Full `cargo test --release --workspace --no-fail-fast`: **pass count unchanged, failure count = 4**. The EXPECTATIONS-SLICE-4 documented 3 pre-existing failures but a 4th (`startup_error_bubbles_up_as_exit_3` in `wat-cli`) was already failing pre-rename — confirmed by stash-round-trip test. Slice 4 introduced 0 new failures. Net delta: 0. See § Honest deltas → Baseline disclosure. |

**Overall:** A YES / B YES / C YES / D YES / E YES. The slice ships as designed.

## Honest deltas

### Actual site count vs estimate

- **Estimated:** ~150 edits across ~18 files (orchestrator grep at commit 9105e17)
- **Actual:** 173 edits across **21 files** (git diff --stat HEAD reports 173 insertions / 173 deletions)
- **Drift explanation:** The 3 extra files are from Rust test function renames (`signature_of_*` → `signature_of_defn_*`) that were implicitly in scope per BRIEF § "Sweep Rust identifier `signature_of` → `signature_of_defn` ONLY where it refers to THIS primitive (test fixture function names, comments, doc strings)" — the orchestrator's grep found WAT literal calls but didn't count each Rust function rename as a separate edit. The 14 test functions renamed in `wat_arc144_hardcoded_primitives.rs` alone account for most of the overage.

### STOP-trigger fires

**None fired.** Zero STOP-trigger activations. Specifically:

- **STOP-trigger 1 (hidden consumer):** The sweep found all 21 files in scope matched the BRIEF's list. No surprise consumers discovered. The EXPECTATIONS-SLICE-4 predicted this as "likely candidate" but it did not materialize — the orchestrator's grep was complete.
- **STOP-trigger 2 (test fails after rename):** Zero test failures introduced. All previously-passing tests pass with the renamed primitive.
- **STOP-trigger 3 (substring corruption):** Zero corruptions. `grep "signature-of-defn-fn"` returns zero matches. `eval_signature_of_fn`, `":wat::runtime::signature-of-fn"`, and the filename `tests/wat_arc201_signature_of_fn.rs` are all intact. Per-file grep discipline caught the distinction.
- **STOP-trigger 4 (eval path looks different):** The rename is purely mechanical. `eval_signature_of_defn` is the same function body as `eval_signature_of` — only the name changed. OP constant updated to match.
- **STOP-trigger 5 (baseline regression):** 0 new failures. See E row.
- **STOP-trigger 6 (alias temptation):** No alias temptation surfaced. Hard-cut confirmed.

### Baseline disclosure (EXPECTATIONS-SLICE-4 discrepancy)

The EXPECTATIONS-SLICE-4 documented 3 pre-existing failures:
1. `lifeline_pipe_zero_orphans_across_100_trials` — FD-multiplex flake
2. `deftest_wat_tests_tmp_totally_bogus` — unrelated wat-test fixture
3. `t6_spawn_process_factory_with_capture_round_trips` — arc 170 slice 6 documented gap

Post-slice-4, the run shows 4 failures. The 4th (`startup_error_bubbles_up_as_exit_3` in `wat-cli`) was verified pre-existing via stash round-trip: running the test at the HEAD commit before any slice 4 edits produced the same FAILED result. The EXPECTATIONS doc's count was understated; this is not a slice 4 regression. Disclosed honestly per `feedback_inscription_immutable`.

### Comment-vs-code distinction judgments

Several comments referenced `signature-of` as the primitive's NAME (not as a concept). All were updated. The key judgment calls:

- **`src/check.rs` lines referring to "the check-side special-case mirrors `signature-of`'s arc-009 bypass"** — these refer to THIS primitive by name in a technical comment about its implementation pattern. Updated to `signature-of-defn`. The comment remains accurate: the new name IS the same primitive with the same bypass.
- **`src/freeze.rs` comments about "reflection-driven macros invoke `signature-of`"** — refers to the verb call in the define-alias macro. Updated.
- **`src/runtime.rs` docstring for `eval_signature_of_fn` referencing `signature-of` as the sibling** — updated to `signature-of-defn` to name the sibling accurately under the new spelling.
- **`wat_arc201_signature_of_fn.rs` docstring "The fn-input sibling of `signature-of`"** — updated to `signature-of-defn`. The file's OWN primitive (`signature-of-fn`) calls are unchanged.

No judgment call concluded "leave as concept reference" — every instance was naming this specific primitive.

### Active docs prose check

- **`docs/USER-GUIDE.md` line 1315:** Already contained a namespace error (`:wat::core::signature-of` should be `:wat::runtime::signature-of`). This is a pre-existing doc error unrelated to slice 4. Updated the verb suffix only (`-of` → `-of-defn`); did not "fix" the namespace, which is outside slice 4 scope.
- **All 5 USER-GUIDE hits:** Pure mechanical replace. No prose rewording needed — every sentence read accurately with the new name. Example: "Reflection (`signature-of-defn`) round-trips the variadic shape correctly" is as honest as the previous phrasing.

### Rust test function renames

14 test functions in `tests/wat_arc144_hardcoded_primitives.rs` renamed from `signature_of_*` to `signature_of_defn_*`. Additionally a private helper `assert_signature_of_some` renamed to `assert_signature_of_defn_some`. This was per BRIEF § "Sweep Rust identifier `signature_of` → `signature_of_defn` ONLY where it refers to THIS primitive." All renamed; the `eval_signature_of_fn` / `signature_of_fn_*` test functions in `wat_arc201_signature_of_fn.rs` are untouched.

### Substring preservation paranoia

The per-file grep discipline (checking `grep -v "signature-of-fn"` after each file edit, then confirming `signature-of-defn-fn` does not exist) caught no actual corruptions — but the discipline was executed for all 21 files. The tool `replace_all` was used only when the replaced string was unique to the primitive being renamed (e.g., `(:wat::runtime::signature-of :user::my-mul)` has no false-positive risk). Multi-occurrence strings (like the 3 identical `sig-opt (:wat::runtime::signature-of :user::add-two)` blocks in `wat_arc201_holon_ast_accessors.rs`) were replaced with `replace_all: true` after confirming the target string only referred to the old primitive.

## Files touched

21 files, 173 mechanical edits (each is a rename from `signature-of` → `signature-of-defn` or `signature_of` → `signature_of_defn` where applicable):

**Rust substrate (4 files):**
- `src/runtime.rs` — dispatch arm, fn name, OP constant, 9 comments in docstrings/inline comments
- `src/check.rs` — infer_list match arm, env.register string literal, 8 comments
- `src/freeze.rs` — 2 comments
- `src/stdlib.rs` — 1 comment

**Wat consumer (1 file):**
- `wat/runtime.wat` — 1 comment header + 2 literal calls in define-alias macro body

**Test files (13 files):**
- `tests/wat_arc143_lookup.rs` — 4 renamed test fns + 4 inline WAT calls + 2 comments
- `tests/wat_arc143_define_alias.rs` — 4 comments
- `tests/wat_arc143_manipulation.rs` — 2 doc comments + 5 inline WAT calls + 1 section header + 1 inline comment
- `tests/wat_arc136_do_form.rs` — 1 section header + 1 comment + 1 inline WAT call
- `tests/wat_arc144_lookup_form.rs` — 2 doc comments + 2 renamed test fns + 4 inline WAT calls + 2 assert messages
- `tests/wat_arc144_uniform_reflection.rs` — 1 table entry + 3 comments + 1 renamed test fn + 2 inline WAT calls + 2 assert messages
- `tests/wat_arc144_special_forms.rs` — 1 doc comment + 1 inline WAT call + 2 comments + 1 assert message format string
- `tests/wat_arc144_hardcoded_primitives.rs` — 2 doc comments + renamed helper fn + 14 renamed test fns + 1 inline comment
- `tests/wat_arc146_dispatch_mechanism.rs` — 1 doc comment + 1 renamed test fn + 1 inline WAT call
- `tests/wat_arc150_variadic_define.rs` — 1 mod-level doc + 1 section header + 1 renamed test fn + 2 comments + 1 inline WAT call
- `tests/wat_arc201_signature_of_fn.rs` — 2 docstring comments (sibling name updated; all fn-value calls preserved as `signature-of-fn`)
- `tests/wat_arc201_holon_ast_accessors.rs` — 1 doc comment + 3 inline WAT calls + 4 abort message strings + 2 comments
- `tests/wat_arc201_structured_signature_types.rs` — 1 doc comment + 1 helper docstring + 1 inline WAT call (in helper) + 2 inline WAT calls + 4 renamed test fns + 1 comment

**Active docs (3 files):**
- `docs/USER-GUIDE.md` — 5 hits (prose + code example)
- `docs/ZERO-MUTEX.md` — 1 hit
- `docs/MODULARIZATION-NOTES.md` — 1 identifier reference (`eval_signature_of` → `eval_signature_of_defn`)

**No new types. No new structs. No new special-forms. No new verbs. No back-compat alias.** Pure mechanical rename per BRIEF § Required path.

## Predicted vs actual time

Predicted 60-90 min. Actual: ~70 min including BRIEF + SCORE-SLICE-3 + DESIGN reads, per-file grep discipline, 21-file sweep with substring paranoia, cargo build (~60s), full workspace test run (~10 min), SCORE write. On target.

## Knock-on / next slice

**Unblocks:**
- Slice 5 (`extract-arg-types` wat-side convenience) — can now compose `signature-of-defn` + `signature-of-fn` + `Bundle/children` + slot-filtering uniformly; both primitives have explicit names that distinguish input-shape.
- Slice 6 (final integration + arc 170 Stone D2 `run-threads` macro) — the asymmetric pair `signature-of-defn` / `signature-of-fn` is now the canonical surface for reflection in type-driven macros.

**No interference with:**
- Slice 1 (5/5 pass after slice 4) — structured-type emission rules unchanged.
- Slice 2 (7/7 pass after slice 4) — Bundle/children + Bundle/first unchanged.
- Slice 3 (8/8 pass after slice 4) — `signature-of-fn` + `eval_signature_of_fn` both intact and passing.

## Discipline anchors honored

- `feedback_inscription_immutable` — historical BRIEFs, SCOREs, INSCRIPTIONs for arcs 143/144/146/148 + slice 1/2/3 of arc 201 left untouched. Only forward artifacts updated.
- `feedback_no_new_types` — zero new types, structs, verbs, or special forms. Pure rename.
- `project_wat_llm_first_design` — one canonical path per task. The alias option was rejected (STOP-trigger 6 held). The rename is the honest cost.
- `feedback_refuse_easy_solutions` — no back-compat alias. Short-term churn accepted as the price of naming clarity.
- `feedback_simple_is_uniform_composition` — 173 identical mechanical edits IS simple. No collapse of change-count with complexity.
- `feedback_assertion_demands_evidence` — pre-existing failure count verified by stash round-trip before asserting "no new failures."
