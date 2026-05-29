# SCORE — Stone 241.11: `:wat::core::define` ⇒ `:wat::core::defn` HARD CUT

**Mode:** A (substrate + cascade; vigilia NOT required per D5 — no new namespaced home)
**Runtime:** two sessions (context boundary mid-flight); resumed directly from compacted summary
**Cascade size:** ~271 sites (auto-fixer handled the bulk; resolve.rs + runtime.rs + core.wat manual)
**Lib tests:** 890 / 0 (1 pre-existing ignored)
**Clippy:** 885 warnings (≤ 902 gate)
**Vigilia:** NOT CAST (D5 — legacy flat substrate; no new namespaced home)

---

## Phase A Scorecard (11 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (defn baseline) | PASS | `contract_01_defn_success_baseline` |
| 2 | Probe C02 PASS (legacy define HARD CUT rejected) | PASS | `contract_02_legacy_define_hard_cut_rejected` |
| 3 | Probe C03 PASS (remedy names :wat::core::defn) | PASS | `contract_03_retirement_remedy_names_defn` |
| 4 | Probe C04 PASS ([retirement replacement] annotation) | PASS | `contract_04_retirement_kind_annotation_present` |
| 5 | Probe C05 PASS (retirement table has define→defn entry) | PASS | `contract_05_retirement_table_includes_define_entry` |
| 6 | Probe whole-suite 5/5 | PASS | `probe_arc241_stone11_define_hard_cut` |
| 7 | Stone 241.10 probe preserved 8/8 | PASS | `probe_arc241_stone10_remedy` |
| 8 | Stone 241.1–241.9 probes preserved | PASS | all prior arc 241 probes green |
| 9 | arc 237 stone 2 probe preserved 12/12 | PASS | `probe_arc237_stone2_defclause_substrate` — required extra work (see below) |
| 10 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 11 | Workspace test-build clean | PASS | `cargo build --release --tests --workspace` exit 0 |

---

## Structural Verification (8 rows)

| Verification | Result |
|---|---|
| `RETIREMENT_TABLE` has ≥ 4 entries | ✓ — 4 entries (struct, struct-restricted, enum, define) |
| `:wat::core::define` entry in retirement table | ✓ — `src/remedy/retirement.rs` line 49 |
| HARD-CUT arm for `:wat::core::define` in check.rs | ✓ — 3 match sites (3236, 7044, 2979) |
| `register_defines` DELETED from freeze.rs | ✓ — 0 matches in `src/freeze.rs` |
| `register_stdlib_defines` DELETED from freeze.rs | ✓ — 0 matches (lives in `src/runtime.rs`; caller deleted) |
| `register_define_dispatches` PRESERVED (arc 146; D4) | ✓ — `src/dispatch.rs` line 247 |
| `parse_define_dispatch_form` PRESERVED (arc 146; D4) | ✓ — `src/dispatch.rs` line 301 |
| `crates/fix-defines/` DELETED (ephemeral) | ✓ — "No such file or directory" |

---

## Cascade Audit

### Auto-fixer story

Per BRIEF authorization (D2+T4), `crates/fix-defines/` was minted as an ephemeral Rust crate that walked all `.wat`, `.rs`, and test source files and rewrote:

- Pattern A: `(:wat::core::define (:ns::name -> :Ret) body)` → `(:wat::core::defn :ns::name [] -> :Ret body)`
- Pattern B: `(:wat::core::define (:ns::name (p :T) -> :Ret) body)` → `(:wat::core::defn :ns::name [p <- :T] -> :Ret body)`
- Pattern C: tests and `.wat` source using `run_compute`-style helpers

The auto-fixer was run, all ~271 sites migrated, then `crates/fix-defines/` was deleted before commit. The `Cargo.toml` workspace member line was removed atomically with the deletion.

### Trap-doors encountered

**T6 — resolve.rs dispatch-head rejection:**
Before Stone 241.11, `define` bodies were consumed by `register_defines` and never walked by the resolver (step 7). After the migration, `defn` bodies stay in the residue and ARE walked at step 7. Dispatch-registered heads (`:h::describe`, `:h::mix-count`) inside `defn` bodies were rejected by `is_resolvable_call_head` because the resolver didn't check `sym.dispatch_registry`.

Fix: added a dispatch-registry check to `is_resolvable_call_head` in `src/resolve.rs` — before the macro call check, check if the canonical head is in `sym.dispatch_registry`. If yes → valid call head. Two probe_declaration_form_lift tests recovered.

**T-argspec — stdlib variadic forms broke after auto-fixer migration:**
The variadic stdlib functions (`i64::+`, `i64::*`, `i64::-`, `i64::/`, `f64::+`, `f64::*`, `f64::-`, `f64::/`) in `wat/core.wat` were migrated from old-style `(:wat::core::define (...  & (xs :T) -> :R) body)` to `defn` form. The auto-fixer generated broken argspec syntax:

- Broken (0-fixed + rest): `[_a <- & xs <- :T]` — `_a` has no type annotation before `&`
- Broken (1-fixed + rest): `[first <- :T _b <- & xs <- :T]` — `_b` has no type annotation before `&`

Two fixes:

1. **`core.wat` argspec correction:** Removed the spurious `_a`/`_b` placeholders. Correct forms:
   - 0-fixed + rest: `[& xs <- :wat::core::Vector<wat::core::i64>]`
   - 1-fixed + rest: `[first <- :wat::core::i64 & xs <- :wat::core::Vector<wat::core::i64>]`

2. **`try_parse_variadic_def_fn_form` in `src/runtime.rs`:** `try_parse_fn_shape_def` uses `parse_fn_signature` with `allow_rest_binder: false` — returns `None` for any `def/fn` form with a rest binder. A new sister function handles these: detects `(:wat::core::def :name (:wat::core::fn [... & xs <- :T] -> :R body))`, calls `parse_argspec_triples` with `allow_rest_binder: true`, and only returns `Some` when `spec.rest_param.is_some()`. Non-variadic forms are left to `try_parse_fn_shape_def` (which runs first).

Result: `probe_arc237_stone2_defclause_substrate` 12/12 PASS (was 5 FAIL before these fixes).

---

## Honest Deltas

### Context boundary mid-flight

Stone 241.11 crossed a context boundary mid-execution (during the resolve.rs fix, before `try_parse_variadic_def_fn_form` was implemented). The continuation session received a compacted summary with the call site already inserted but the function body absent. The function was implemented in the continuation session and both fixes completed cleanly.

### Argspec correctness surprise

The auto-fixer generated syntactically broken argspec for variadic functions. `_a <- &` is not a valid triple (type slot is Symbol `&`, not a Keyword). The substrate accepted it silently in the old path because the forms fell to `is_define_form → parse_define_form` which used the OLD variadic syntax. After the migration, they fell to the `try_parse_fn_shape_def` → `None` path (correct detection), then to stdlib residue (silently discarded), leaving `i64::+` unregistered. The fix is two-part: correct the source + add the variadic handler.

### probe_arc237_stone2 not in EXPECTATIONS

The EXPECTATIONS doc listed Stone 241.1–241.9 probes + Stone 241.10 probe as preserved. probe_arc237_stone2 was not explicitly listed but is in scope ("arc 237/238 probes preserved" per BRIEF). The 5 failures surfaced and were resolved. Net: 12/12 PASS.

---

## What This Unblocks

**Stone 241.12** — INSCRIPTION closes arc 241. Pre-INSCRIPTION grep enforced (per FM 11 + Stone S11 of recovery doc): `grep -rn ":wat::core::define" --include="*.rs" --include="*.wat" src/ tests/ wat/` must return 0 non-comment, non-retired-path matches.

**Arc 237.8b** — reopens after Stone 241.12 per `feedback_no_regression_until_arc_done`.

**Future HARD CUTs** — Stone 241.11 is the first demonstrated consumer of the `RETIREMENT_TABLE` + `remedies_for` infrastructure minted in Stone 241.10. The bandaid-rip-with-receipts pattern is now THE pattern: append one line to `RETIREMENT_TABLE`, add one HARD-CUT arm to `check.rs`, run auto-fixer, delete ephemeral tool.
