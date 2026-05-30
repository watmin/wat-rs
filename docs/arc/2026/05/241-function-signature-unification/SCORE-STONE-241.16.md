# SCORE — Stone 241.16: `:wat::core::define` EVAL-TIME RESIDUE COMPLETION (Enemy 3 of 4 — LAST scheme-style retirement)

**Mode:** A (substrate + cascade; vigilia NOT required — no new namespaced home)
**Runtime:** two sessions (compaction boundary between sessions; resumed from summary)
**Cascade size:** 8 src files modified; 7 test files migrated
**Lib tests:** 890 / 0
**Workspace test build:** clean
**Clippy:** 880 warnings (within ≤935 gate)
**Vigilia:** NOT CAST (legacy flat substrate; no new namespaced home)
**Auto-fixer:** NOT minted (deletions and targeted migrations)

---

## Phase A Scorecard (13 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (define rejection carries "Stone 241.16" marker) | PASS | `contract_01_define_rejection_carries_stone_241_16_marker` |
| 2 | Probe C02 PASS (retirement remedy preserves defn replacement) | PASS | `contract_02_retirement_remedy_preserves_defn_replacement` |
| 3 | Probe C03 PASS (define in let body still rejected) | PASS | `contract_03_define_in_let_body_still_rejected` |
| 4 | Probe C04 PASS (define in fn-body do-prefix still rejected) | PASS | `contract_04_define_in_fn_body_still_rejected` |
| 5 | FM probe whole-suite 4/4 | PASS | `probe_arc241_stone16_define_eval_residue` |
| 6 | Stone 241.11 probe preserved | PASS | `probe_arc241_stone11_define_hard_cut` |
| 7 | Stone 241.13 probe preserved | PASS | `probe_arc241_stone13_define_dispatch_hard_cut` |
| 8 | Bypass-rejection tests migrated to defstruct (S8a–S8e + eval_result/def_not_special) | PASS | all migrate to `:wat::core::defstruct`; is_mutation_head verified |
| 9 | special_forms probe updated (define absent from registry) | PASS | `lookup_form_define_is_absent_from_registry` |
| 10 | uniform_reflection probe updated (defn head in emission) | PASS | `user_function_lookup_define_emits_defn_head` + `primitive_empty_lookup_define_emits_define_head` |
| 11 | wat_arc143_lookup updated (defn not define in rendered AST) | PASS | `lookup_define_user_function_contains_defn_keyword` |
| 12 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 13 | Workspace test-build clean | PASS | `cargo build --tests --workspace` exit 0 |

---

## Structural Verification (12 rows)

| Verification | Result |
|---|---|
| `parse_define_form` + `parse_define_signature` + `ParsedDefineSignature` + `parse_param_pair` DELETED from `src/runtime.rs` (~320 lines of eval-time scaffolding) | confirmed; Stone 241.16 deletion comment replaces entire block |
| `is_define_form` fn DELETED from `src/runtime.rs` | confirmed; ~7 lines deleted; Stone 241.16 comment |
| `":wat::core::define"` eval dispatch arm DELETED from `dispatch_keyword_head_value` | confirmed; `DeclarationInExpressionPosition` arm removed; Stone 241.16 comment |
| `":wat::core::define"` arm DELETED from `is_mutation_head` (runtime.rs) | confirmed; now starts with `:wat::core::defmacro` |
| `":wat::core::define"` arm DELETED from `is_mutation_form` (freeze.rs) | confirmed; Stone 241.16 comment |
| `":wat::core::define"` arm DELETED from `is_declaration_form` (freeze.rs) | confirmed; Stone 241.16 comment |
| `":wat::core::define"` registry entry DELETED from `special_forms.rs` | confirmed; Stone 241.16 comment |
| `walk_define_form` fn DELETED from `src/closure_extract.rs` | confirmed; dispatch arm + function (~35 lines total) deleted |
| `function_to_define_ast` + `primitive_to_define_ast` head updated from `define` → `defn` | confirmed; both emitters now emit `:wat::core::defn`-headed AST |
| Stone 241.11 HARD-CUT arm at check.rs:6991 KEPT with Stone 241.16 marker update | confirmed; comment block expanded; error text includes both stones |
| check.rs S4a/S4b define-name-collection branches DELETED (sandbox-scope / legacy_user_main arms) | confirmed; Stone 241.16 comments at each deletion site |
| RETIREMENT_TABLE UNCHANGED (Stone 241.11 entry `define → defn` preserved) | confirmed; `src/remedy/retirement.rs` untouched |

---

## PRIMARY DELETIONS (runtime.rs)

```rust
// Stone 241.16 — DELETED: ParsedDefineSignature, parse_define_form,
// parse_define_signature, and parse_param_pair DELETED.
// `:wat::core::define` is HARD CUT (eval-time residue completed);
// these functions processed the old Scheme-style `(:wat::core::define sig body)` form.
// ~30 error-construction sites inside these functions died with them.
// Per Stone 241.13 src/dispatch.rs 445-line deletion precedent.
```

```rust
// Stone 241.16 — is_define_form DELETED.
// The function is dead code; `:wat::core::define` is HARD CUT (total).
// the function itself is now dead code. `:wat::core::define` is HARD CUT (total).
```

---

## Bypass-Rejection Test Migrations (S8a–S8e)

All tests that programmatically constructed define-headed AST and asserted bypass-rejection at eval time were migrated to `:wat::core::defstruct` (still in `is_mutation_head`):

| File | Test | Migration |
|---|---|---|
| `src/runtime.rs` | `eval_ast_bang_refuses_mutation_form` | `define (:evil ...)` → `defstruct :evil::T [x <- :wat::core::i64]` |
| `src/runtime.rs` | `eval_edn_bang_refuses_mutation_inside_string` | same pattern |
| `src/freeze.rs` | `eval_refuses_define` | define → defstruct; assertion head updated to `:wat::core::defstruct` |
| `src/freeze.rs` | `eval_refuses_mutation_form_at_any_depth` | nested define → nested defstruct |
| `src/freeze.rs` | digest test | define → defstruct |
| `tests/wat_eval_result.rs` | `eval_ast_bang_mutation_form_surfaces_as_err` | define → defstruct |
| `tests/wat_eval_result.rs` | `try_propagates_eval_err_through_helper` | define → defstruct |
| `tests/wat_eval_result.rs` | `eval_err_exposes_both_kind_and_message` | define → defstruct; message assertion `contains(":wat::core::defstruct")` |
| `tests/probe_def_not_special.rs` | `probe_define_at_expression_position_still_emits_error` → `probe_define_rejected_at_startup_check` | startup now FAILS (HARD CUT fires before eval); asserts `Err` containing `:wat::core::define` |
| `tests/probe_def_not_special.rs` | Probe 1 fixture | define `(:user::main → ...)` → defn `(:user::main [] → ...)` |

---

## Error String Migrations (S7)

All user-facing error diagnostic strings that named `:wat::core::define` as the form-head migrated to `:wat::core::defn`:

| File | Location | Migration |
|---|---|---|
| `src/runtime.rs` | `parse_type_keyword` error head | `":wat::core::define"` → `":wat::core::defn"` |
| `src/runtime.rs` | `parse_type_slot` body (7 sites) | `MalformedForm { head: ":wat::core::define" }` → `":wat::core::defn"` |
| `src/runtime.rs` | `split_name_and_type_params` error head | define → defn |
| `src/check.rs` | sandbox-scope-leak hint (~913) | hint example updated from define to defn syntax |
| `src/runtime.rs` | sandbox-scope-leak hint (~2513) | hint example updated from define to defn syntax |
| `src/check.rs` | wrapper label (~2414) | `":wat::core::define (body)"` → `":wat::core::defn (body)"` |

---

## Pre-INSCRIPTION Grep Gate

Active substrate code references to `:wat::core::define` string literals in `src/`:

| File | Line | Category |
|---|---|---|
| `src/check.rs:6991` | `":wat::core::define" => {` | REQUIRED — the HARD-CUT arm itself (Stone 241.11, updated Stone 241.16) |
| `src/resolve.rs:226` | `if head == ":wat::core::define" { return; }` | REQUIRED — fast-path skip to prevent resolver recursing into retired form's body (Stone 241.11 comment present) |

All other occurrences in `src/` are historical/doc comments. Gate CLEAN.

Integration test files with active `:wat::core::define` references (acceptable):
- `tests/probe_arc241_stone11_define_hard_cut.rs` — regression probe; fixtures assert startup rejection
- `tests/probe_arc241_stone16_define_eval_residue.rs` — FM probe; fixtures assert startup rejection
- `tests/probe_def_not_special.rs` — Probe 4 fixture asserts startup rejection (migrated from runtime rejection)

All other test-file references are comments or historical doc text.

**Active eval-time substrate callers of `:wat::core::define` string: 0**

Gate CLEAN.

---

## Cascade Audit

### S1 — parse_define_form block DELETED (runtime.rs)

`ParsedDefineSignature` struct, `parse_define_form`, `parse_define_signature`, `parse_param_pair` — ~320 lines of Scheme-era eval-time scaffolding deleted. ~30 error-construction sites inside these functions died with them (per Stone 241.13 precedent). Stone 241.16 deletion comment block replaces the entire region.

### S2 — is_define_form DELETED (runtime.rs)

~7-line predicate function deleted. Was the structural "is this a define form?" check — now dead code since define is HARD CUT at startup. Stone 241.16 comment at deletion site.

### S3 — eval dispatch arm DELETED (dispatch_keyword_head_value, runtime.rs)

`":wat::core::define" => Err(RuntimeError::DeclarationInExpressionPosition(...))` deleted from the eval dispatch match. After Stone 241.16, define never reaches eval (caught at startup-check). Stone 241.16 comment.

### S4 — check.rs processing arms DELETED

**S4a**: Sandbox-scope inner-form scan: define-name-collection branch deleted (~2884). Permanently unreachable (startup-check fires before sandbox scan sees define). Stone 241.16 comment + empty `inner_names: HashSet` placeholder.

**S4b**: `check_legacy_user_main_signature` define arm deleted (~3141). Pre-Stone-241.11 processing arm; permanently unreachable. Stone 241.16 comment.

### S5 — special_forms.rs registry entry DELETED

`insert(&mut m, ":wat::core::define", &["<head>", "<body>"])` deleted. Test `registry_covers_audited_forms` updated: `:wat::core::define` removed, Stone 241.16 comment left. This is now `lookup_form_define_is_absent_from_registry` (renamed from `lookup_form_define_returns_special_form`).

### S6 — closure_extract.rs (discovered at grep gate)

`":wat::core::define" => { return walk_define_form(rest, locals, state); }` dispatch arm deleted at ~602. `walk_define_form` fn (~35 lines) deleted at ~747-781. Eval-time closure-extraction for define forms — permanently dead code post-Stone 241.11. Stone 241.16 deletion comment at both sites.

### S7 — Error string migrations (runtime.rs + check.rs)

`parse_type_keyword`, `parse_type_slot` (7 sites), `split_name_and_type_params`, sandbox-scope-leak hints (check.rs + runtime.rs), wrapper label: all migrated from `":wat::core::define"` to `":wat::core::defn"`. These utilities are called by defn parsing (and formerly by define parsing); the error diagnostic should name the current form.

### S8 — Bypass-rejection test migrations (9 tests across 5 files)

See Bypass-Rejection Test Migrations table above.

### S9 — Reflection emitter update (runtime.rs)

`function_to_define_ast` and `primitive_to_define_ast`: both changed from emitting `:wat::core::define`-headed AST to `:wat::core::defn`-headed AST. Docs updated. These are used by `lookup-define` reflection — they must emit the live form, not the retired one.

### S10 — mutation_form / declaration_form predicate cleanup (freeze.rs)

`is_mutation_form` and `is_declaration_form` in `src/freeze.rs`: `| ":wat::core::define"` arm deleted from each. Stone 241.16 comment. These are used by the freeze-time bypass-rejection mechanism.

### S11 — is_mutation_head update (runtime.rs)

`":wat::core::define"` arm deleted from `is_mutation_head`. Stone 241.16 comment. Now starts with `":wat::core::defmacro"`.

### S12 — doc/comment cascade (test files)

- `tests/probe_let_splice_define.rs` + `tests/probe_do_splice_define.rs`: header and doc comments updated from define to defn.
- `tests/probe_declaration_form_lift.rs`: `probe_is_declaration_form_covers_all_7_keywords` — `:wat::core::define` removed from `covered` array; `:wat::core::defalias` added as 7th slot (Stone 241.12).
- `tests/wat_arc143_lookup.rs`: `lookup_define_user_function_contains_define_keyword` renamed to `lookup_define_user_function_contains_defn_keyword`; assertion updated from `contains("define")` to `contains("defn")`.
- `tests/wat_arc144_uniform_reflection.rs`: two tests updated to assert `:wat::core::defn` head (not `:wat::core::define`).
- `tests/wat_arc144_special_forms.rs`: `lookup_form_define_returns_special_form` renamed/migrated to `lookup_form_define_is_absent_from_registry` — now asserts `is_none()`.

---

## Trap-Doors Closed

**T1 (parse_type_keyword/parse_type_slot error heads)**: These utilities still exist (called by defn parsing). Their error diagnostic strings previously named `:wat::core::define` — migrated to `:wat::core::defn`. Honest: the error now names the live form.

**T2 (closure_extract.rs)**: Discovered at S12 grep gate. Active dispatch arm + `walk_define_form` function survived — both deleted. Eval-time residue was more pervasive than the BRIEF's manifest listed.

**T3 (check.rs:6991 HARD-CUT arm)**: Confirmed as Stone 241.11's arm → KEPT with Stone 241.16 marker update. Per doctrine: the startup-check arm is the primary enforcement mechanism; Stone 241.16 completes the substrate-side deletion but does not remove the rejection itself.

**T4 (bypass-rejection test fixtures)**: Migrated to `:wat::core::defstruct` (mechanism preserved; specific head changes). `probe_def_not_special.rs` Probe 4 additionally migrated from eval-time assertion to startup-check assertion (startup now fails; not runtime).

**T5 (reflection emitters)**: `function_to_define_ast` + `primitive_to_define_ast` — found and migrated to emit `:wat::core::defn`-headed AST. Without this, `lookup-define` reflection would emit the retired form.

**T6 (wat_eval_result.rs discovered at grep gate)**: Three tests using `(:wat::core::define ...)` inside `(:wat::core::quote ...)` + `eval-ast!` — testing `"mutation-form-refused"`. After Stone 241.16, `is_mutation_head` no longer recognizes define, so eval-ast! would not refuse it (would hit `UnknownFunction` instead). Migrated to `:wat::core::defstruct` — same mechanism, still refused, correct `"mutation-form-refused"` kind.

---

## Honest Deltas

### Scope was wider than the BRIEF's manifest

The BRIEF listed primary targets in `src/runtime.rs`, `src/freeze.rs`, `src/check.rs`, `src/special_forms.rs`. Two additional files were found at the S12 grep gate with active eval-time residue:
- `src/closure_extract.rs`: `walk_define_form` + dispatch arm
- `tests/wat_eval_result.rs`: 3 tests using define in quoted AST

Both cleaned per `feedback_hard_cut_admits_no_bypasses`. No privileged paths.

### Two-session delivery due to compaction boundary

Context compaction hit mid-stone. Session 2 resumed from summary, completed `tests/probe_def_not_special.rs` + `tests/wat_eval_result.rs` migrations, ran full verification, authored SCORE.

### Pre-existing failures documented

`wat_arc143_lookup`: `body_of_user_define_returns_some` + `signature_of_defn_user_define_returns_some` — pre-existing TypeMismatch (`:wat::runtime::signature-of-defn` + `:wat::runtime::body-of` receiving `fn` type). Confirmed by `git stash` round-trip. Not introduced by Stone 241.16.

`wat_arc144_uniform_reflection`: `user_function_signature_and_body_return_some` — pre-existing. Same pattern.

`probe_declaration_form_lift`: 3 tests (`probe_mixed_declaration_prelude_all_lift`, `probe_newtype_in_fn_body_do_prefix_lifts_to_prologue`, `probe_typealias_in_fn_body_do_prefix_lifts_to_prologue`) — pre-existing arc 242 nil-doctrine failures. Confirmed by `git stash` round-trip.

Stone 241.16 introduces **zero new test failures**.

### Clippy down to 880 (from 889 at Stone 241.15 baseline)

~9 additional warnings removed by deleting `parse_define_form`, `is_define_form`, `walk_define_form`, and associated dead-code. Healthy downward delta.

---

## Calibration

| Phase | Predicted | Actual |
|---|---|---|
| S1 parse_define_form block deletion | 10-15 min | ~15 min (multiple edit attempts to get extent right) |
| S2 is_define_form deletion | 3 min | ~3 min |
| S3 eval dispatch arm deletion | 3 min | ~2 min |
| S4 check.rs arms | 10 min | ~10 min |
| S5 special_forms.rs | 5 min | ~5 min |
| S6 closure_extract.rs (discovered) | — | ~10 min (grep-found; 2-site deletion) |
| S7 error string migrations | 10 min | ~20 min (7 sites; parse_type_slot had multiple indentation mismatches) |
| S8 bypass-rejection test migrations | 15 min | ~20 min (9 tests across 5 files; 2 files discovered at grep gate) |
| S9 reflection emitter update | 5 min | ~5 min |
| S10+S11 predicate cleanup | 5 min | ~5 min |
| S12 grep gate + doc cascade | 10 min | ~15 min (wider than expected; multiple discovered sites) |
| S13 SCORE | 15 min | ~20 min |
| **Total** | **~91 min** | **~130 min** (two sessions; compaction boundary) |

Two-session delivery inflates wall-clock but not decision load. Under-band per session.

---

## What This Unblocks

**Stone 241.17** — INSCRIPTION closes arc 241 (orchestrator-direct paperwork).

**Enemy 3 of 4 is ELIMINATED.** The Clojure-aligned unification arc's four enemies:
- Enemy 1 (`:wat::core::struct`) — HARD CUT (Stone 241.8)
- Enemy 2 (`:wat::core::define-dispatch`) — HARD CUT (Stone 241.13)
- Enemy 3 (`:wat::core::define`) — HARD CUT TOTAL (Stone 241.11 startup; Stone 241.16 eval-time residue)
- Enemy 4 (next) — pending Stone 241.17 INSCRIPTION then 241.18+

**One-canonical-path doctrine**: `:wat::core::defn` is now THE function-binding form. No privileged paths. No defense-in-depth. The substrate is clean.
