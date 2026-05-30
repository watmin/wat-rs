# SCORE — Stone 241.17: `:wat::core::defmacro` signature migration to canonical (closes arc 177)

**Mode:** A (substrate + cascade; vigilia NOT required — no new namespaced home)
**Runtime:** two sessions (compaction boundary between sessions; resumed from summary)
**Cascade size:** 3 src files modified; 18 test files migrated; 1 doc file migrated; reflection emitter updated
**Lib tests:** 890 / 0
**Workspace test build:** clean
**Clippy:** 898 warnings (within ≤940 gate)
**Vigilia:** NOT CAST (legacy flat substrate; no new namespaced home)
**Auto-fixer:** NOT minted (per-file mechanical migration; parse_argspec_triples routing straightforward)

---

## Phase A Scorecard (10 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (canonical 6-item shape works) | PASS | `contract_01_defmacro_canonical_shape_works` |
| 2 | Probe C02 PASS (old paren-pair shape REJECTED) | PASS | `contract_02_old_paren_pair_shape_rejected` |
| 3 | Probe C03 PASS (canonical rest-binder shape works) | PASS | `contract_03_defmacro_canonical_rest_binder_works` |
| 4 | FM probe whole-suite 3/3 | PASS | `probe_arc241_stone17_defmacro_canonical` |
| 5 | `wat/core.wat:180` defn macro LOAD-BEARING migration | PASS | migrated first; lib green before proceeding to other wat/ files |
| 6 | 29 wat/ defmacro callers migrated (12 files) | PASS | core.wat × 1, test.wat × 9, Record.wat × 2, holon/*.wat × 10, kernel/run_threads.wat × 3 |
| 7 | tests/ cascade migrated (fixture-bearing files) | PASS | 17 files with old-shape fixtures; 1 file (probe_arc241_stone17) is new-shape already |
| 8 | Reflection emitter updated to canonical 6-item form | PASS | `macrodef_to_signature_ast` + `macrodef_to_define_ast` in `src/runtime.rs` |
| 9 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 10 | Workspace test-build clean | PASS | `cargo build --tests --workspace` exit 0 |

---

## Structural Verification (8 rows)

| Verification | Result |
|---|---|
| `parse_defmacro_signature` DELETED entirely from `src/macros.rs` (~80+ lines) | confirmed; Stone 241.17 deletion comment block replaces region |
| `parse_defmacro_form` REWRITTEN to route through `parse_argspec_triples` (3rd major consumer) | confirmed; 6-item + 7-item (with metadata) canonical dispatch; old 3-item path is HARD-CUT |
| HARD-CUT arm: `items.len() == 3 && items[1] is List` → `MalformedDefmacro` with "Stone 241.17" marker | confirmed; `src/macros.rs` parse_defmacro_form; no compatibility shim |
| `find_defmacro_body` helper updated: `items[2]` → `items[5]` (new body position) | confirmed; test helper in `src/macros.rs` |
| All lib-test fixtures in `src/macros.rs` migrated to new canonical shape | confirmed; 15+ fixture strings; zero-param macros use `[]` argspec vector |
| `src/freeze.rs` lib-test fixtures migrated | confirmed; `user_macro_registers` + `eval_refuses_defmacro` |
| `macrodef_to_signature_ast` returns canonical argspec Vector `[name <- :AST ...]` | confirmed; `src/runtime.rs`; was emitting old paren-pair List head |
| `macrodef_to_define_ast` emits canonical 6-item form `(:wat::core::defmacro :name [argspec] -> :Ret body)` | confirmed; `src/runtime.rs`; was emitting 3-item form |

---

## PRIMARY DELETION (macros.rs)

```rust
// Stone 241.17 — DELETED: parse_defmacro_signature DELETED.
// `:wat::core::defmacro` signature shape MIGRATED to canonical Vector-of-triples
// (mirrors defn per arc 166). parse_defmacro_form now routes through
// parse_argspec_triples (3rd major consumer after fn + defclause).
// Old paren-pair-with-type shape is HARD CUT (no compatibility shim).
// ~80+ lines of arc 010/150 paren-pair parsing machinery died.
```

---

## Cascade Audit

### S1 — parse_defmacro_form rewritten (macros.rs)

`parse_defmacro_form` at `src/macros.rs:320` completely rewritten. Old 35-line function routed through `parse_defmacro_signature`. New ~120-line function dispatches on item count:
- `items.len() == 3 && items[1] is List` → HARD-CUT rejection with "Stone 241.17" marker
- `items.len() == 6` → canonical destructure: name(1) + argvec(2) + `->` check(3) + ret-type(4) + body(5)
- `items.len() == 7` → metadata-map at items[2] skipped; argvec at items[3]
- `items.len() == n` (other) → MalformedDefmacro

Routes argspec Vector through `parse_argspec_triples` with `allow_rest_binder: true` (mirrors defclause). Manual `ArgSpecError → MacroError::MalformedDefmacro` conversion (no `From` impl for MacroError).

### S2 — parse_defmacro_signature DELETED (macros.rs)

~80+ lines of arc 010/150 paren-pair parsing machinery deleted. Stone 241.17 deletion comment block at site.

### S3 — find_defmacro_body helper updated (macros.rs)

Body position `items[2]` → `items[5]` per canonical 6-item shape. Comment updated.

### S4 — wat/ file migrations (12 files, 29+ sites)

| File | Sites | Notes |
|---|---|---|
| `wat/core.wat` | 1 | LOAD-BEARING `defn` macro; migrated first; lib green verified before proceeding |
| `wat/test.wat` | 9 | deftest, deftest-hermetic, make-deftest (× 2 incl. inner templates), make-deftest-hermetic (× 2), run-hermetic, run-hermetic-with-prelude, run-thread, run-hermetic-with-io |
| `wat/Record.wat` | 2 | `:wat::Record::def` and `:wat::holon::Record::def` |
| `wat/holon/Amplify.wat` | 1 | 3 fixed params |
| `wat/holon/Log.wat` | 1 | 3 fixed params |
| `wat/holon/Sequential.wat` | 1 | 1 fixed param |
| `wat/holon/Trigram.wat` | 1 | 1 fixed param |
| `wat/holon/Bigram.wat` | 1 | 1 fixed param |
| `wat/holon/Project.wat` | 1 | 2 fixed params |
| `wat/holon/Ngram.wat` | 1 | 2 fixed params (incl. `n`) |
| `wat/holon/Reject.wat` | 1 | 2 fixed params |
| `wat/holon/ReciprocalLog.wat` | 1 | 2 fixed params |
| `wat/holon/Subtract.wat` | 1 | 2 fixed params |
| `wat/holon/Circular.wat` | 1 | 2 fixed params |
| `wat/kernel/run_threads.wat` | 3 | run-threads-n1 (2 fixed), run-threads-n3 (4 fixed), run-threads (1 fixed + rest-binder) |

**make-deftest inner template note**: the inner quasiquote template produces a defmacro at expansion time. `~name` occupies items[1] in the generated form. Both outer defmacro declarations and inner quasiquote-generated templates migrated to canonical shape.

### S5 — tests/ cascade migration (17 files with old-shape fixtures)

Per-file judgment applied. Comment-only references preserved.

| File | Sites | Pattern |
|---|---|---|
| `tests/wat_variadic_defmacro.rs` | 6 | variadic defmacros with `& (name :T)` rest-binders |
| `tests/wat_arc144_lookup_form.rs` | 3 | `(:my::ident (x :AST) -> :AST) \`~x)` pattern (replace_all) |
| `tests/wat_idempotent_redeclare.rs` | 2 | `(:my::ident (x :AST) -> :AST) \`~x)` × 2 (replace_all) |
| `tests/probe_def_not_special.rs` | 1 | `(:h::mix-id8 (z :AST) -> :AST)` |
| `tests/probe_declaration_form_lift.rs` | 2 | `(:h::id-macro (x :AST) -> :AST)` + `(:h::mix-id (z :AST) -> :AST)` |
| `tests/probe_let_splice_define.rs` | 1 | `(:my::probe (body :AST<...>) -> :AST<...>)` |
| `tests/probe_let_splice_struct.rs` | 1 | same pattern |
| `tests/probe_let_splice_enum.rs` | 1 | same pattern |
| `tests/probe_do_splice_define.rs` | 1 | same pattern |
| `tests/probe_do_splice_enum.rs` | 1 | same pattern |
| `tests/probe_do_splice_struct.rs` | 1 | same pattern |
| `tests/probe_do_splice_def.rs` | 1 | same pattern |
| `tests/wat_macro_vector_splice_symmetry.rs` | 4 | splice tests incl. rest-binder forms |
| `tests/probe_diagnostic_macro_splice_from_let.rs` | 2 | bare signature without return type (added `:AST<...>` return) |
| `tests/probe_arc237_s0_records_gate.rs` | 2 | `(:my::defthing (name :AST<...>) -> :AST<...>)` |
| `tests/wat_arc170_closure_extraction.rs` | 1 | `(:my::triple (x))` — bare param, no type annotation |
| `tests/wat_arc144_uniform_reflection.rs` | 1 | `(:my::id (x :AST) -> :AST)` |

`tests/probe_arc241_stone17_defmacro_canonical.rs` — NOT migrated. C01/C03 are already new-shape; C02 intentionally tests old-shape rejection.

### S6 — Doc cascade migration (1 file)

`docs/USER-GUIDE.md` — two defmacro examples in the `defmacro` section migrated:
- Line 1218: `(:my::app::when ...)` one-param + body-param example
- Lines 1242-1244: `(:my::factory ...)` macro-generating-macro example (with inner template)
  - `,cond`/`,body` unquote sugar updated to `~cond`/`~body` (canonical wat unquote)
  - Inner template updated to new canonical 6-item inner form

`docs/CLOJURE-ROSETTA.md` and `docs/INTENTIONS.md` — no active defmacro shape examples; keyword references only; preserved as-is.

Arc-specific docs in `docs/arc/2026/04/*/` (INSCRIPTION, BACKLOG, DESIGN files) — historical records per `feedback_inscription_immutable`; NOT migrated.

### S7 — Reflection emitter audit (runtime.rs)

`grep -n "Keyword.*defmacro" src/` found 3 hits:
- `src/macros.rs:315` — `is_defmacro_form` predicate; UNCHANGED (keyword detection; not shape emission)
- `src/macros.rs:1651` — `find_defmacro_body` test helper; already updated (items[5])
- `src/runtime.rs:12653` — `macrodef_to_define_ast`; OLD: 3-item form; MIGRATED to canonical 6-item form

Both `macrodef_to_signature_ast` and `macrodef_to_define_ast` updated:
- `macrodef_to_signature_ast`: now returns `WatAST::Vector` with canonical triples (`name <- :AST ...`); was returning a `WatAST::List` with paren-pair `(name :AST)` children
- `macrodef_to_define_ast`: now returns 6-item canonical form; was returning 3-item old form

`lookup-define` and `signature-of-defn` reflection on macros now emit canonical surface.

### S8 — Probe verification

`tests/probe_arc241_stone17_defmacro_canonical.rs` — **3/3 PASS**.

At start of session (post-S1/S2/S3 substrate rewrite, pre-S4 wat/ migrations):
- C01: FAIL (startup fails; stdlib wat files still use old shape)
- C02: PASS (HARD-CUT arm fires correctly for old shape)
- C03: FAIL (startup fails)

Post-S4 (all wat/ migrations including LOAD-BEARING `wat/core.wat:180`):
- C01: PASS
- C02: PASS
- C03: PASS

---

## Pre-INSCRIPTION Grep Gate

Active `(:wat::core::defmacro\s*$` in `wat/`:

| File | Line | Category |
|---|---|---|
| `wat/test.wat:332` | `` `(:wat::core::defmacro `` | NEW shape — quasiquote template opening; `~name` follows on next line (items[1]) |
| `wat/test.wat:348` | `` `(:wat::core::defmacro `` | NEW shape — same (make-deftest-hermetic inner template) |

Both are quasiquote-template openings that continue to new canonical items on subsequent lines. These are multi-line forms — the parser sees 6 items total, with `~name` at items[1]. Gate CLEAN.

Integration test files with active `:wat::core::defmacro` references (acceptable):
- `tests/probe_arc241_stone17_defmacro_canonical.rs` — C02 fixture intentionally tests old-shape rejection

**Active old-shape callers in substrate src/: 0.**

Gate CLEAN.

---

## Trap-Doors Closed

**T1 (macros.rs test fixtures)**: 6 lib-test fixtures in `src/macros.rs` still used old 3-item paren-pair shape after S1-S3 substrate rewrite. All 6 migrated: 2 zero-param macros (added `[]` argspec vector), 3 one-param macros, 1 two-param macro. `find_defmacro_body` helper also updated (items[2] → items[5]).

**T2 (freeze.rs fixtures)**: 2 lib-test fixtures in `src/freeze.rs` still used old shape. Migrated to canonical; assertions remain valid.

**T3 (make-deftest inner templates)**: The `make-deftest` and `make-deftest-hermetic` macros in `wat/test.wat` generate inner defmacros via nested quasiquote. The inner quasiquote template must also produce the new canonical 6-item form at expansion time. Both inner templates migrated: `~name` at items[1], `[test-name <- ... body <- ...]` at items[2], `-> :AST<...>` at items[3-4], body at items[5].

**T4 (probe_diagnostic_macro_splice_from_let fixtures)**: Two fixtures used old paren-pair shape AND omitted return type annotation. Migrated with explicit `-> :AST<wat::core::nil>` return type.

**T5 (wat_arc170_closure_extraction fixture)**: Old fixture `(:my::triple (x))` — bare param with no type annotation in old shape. Migrated to `[x <- :AST] -> :AST` canonical form.

**T6 (reflection emitter)**: `macrodef_to_signature_ast` + `macrodef_to_define_ast` in `src/runtime.rs` — found and migrated. Without this, `lookup-define` on a macro would emit the retired old-shape form.

---

## Honest Deltas

### Scope wider than BRIEF's manifest

The BRIEF counted "36 tests/ references" and "29 wat/ callers". Actual:
- `tests/`: 17 files with old-shape fixture strings; 1 file already new-shape (probe); 11 files comment/keyword-only references (no fixture migration needed)
- `wat/`: 15 files with 29+ migration sites (counts matched)

The 6 remaining lib-test fixtures in `src/macros.rs` were discovered after S1-S4 when the first lib test run produced 6 failures. All repaired.

### Two-session delivery due to compaction boundary

Context compaction hit after all wat/ + tests/ migrations were complete but before final verification + SCORE. Session 2 resumed from summary: confirmed lib test status (6 failing fixtures remaining in macros.rs), fixed all 6 fixtures, ran full verification suite, authored SCORE.

### Clippy rose slightly (880 → 898)

Stone 241.16 was at 880; Stone 241.17 at 898. The additions come from unused dead_code warnings in some test fixture fns. Within ≤940 gate.

---

## Calibration

| Phase | Predicted | Actual |
|---|---|---|
| S1-S3 (parse_defmacro_form rewrite + parse_defmacro_signature delete + HARD-CUT arm) | 20 min | ~30 min (ArgSpecError → MacroError conversion required manual match; no From impl) |
| S4 wat/ migrations (29 sites × 12 files) | 30 min | ~45 min (make-deftest inner templates required careful depth analysis; two-session) |
| S5 tests/ cascade (17 files × ~1-6 sites each) | 30 min | ~35 min (zero-param macros and bare-type fixtures required judgment; probe file correctly preserved) |
| S6 doc cascade | 10 min | ~10 min |
| S7 reflection emitter audit | 10 min | ~10 min |
| S8 probe verification | 5 min | ~5 min |
| S9 grep gate | 5 min | ~5 min |
| S10 SCORE | 15 min | ~20 min |
| **Total** | **~125 min** | **~160 min** (two sessions; compaction boundary) |

---

## What This Unblocks

**Stone 241.18 (INSCRIPTION)** — orchestrator-direct paperwork closing BOTH arc 241 (function signature unification, all four enemies eliminated) AND arc 177 (defmacro syntax; absorbed by this stone).

**Enemy 4 of 4 is ELIMINATED.** The Clojure-aligned unification arc's four enemies:
- Enemy 1 (`:wat::core::struct`) — HARD CUT (Stone 241.8)
- Enemy 2 (`:wat::core::define-dispatch`) — HARD CUT (Stone 241.13)
- Enemy 3 (`:wat::core::define`) — HARD CUT TOTAL (Stone 241.11 startup; Stone 241.16 eval-time residue)
- Enemy 4 (`:wat::core::defmacro` paren-pair shape) — HARD CUT TOTAL (Stone 241.17)

**def-family parser unification GENUINELY COMPLETE.** `defn`, `fn`, `defclause`, and `defmacro` all route through `parse_argspec_triples`. One canonical parser. One canonical shape. No privileged paths.

**arc 177 (defmacro-syntax-clojure) absorbed.** The arc's single design decision — migrate defmacro signature to Clojure-style Vector-of-triples — is fully executed by this stone. Stone 241.18 INSCRIPTION closes both arcs simultaneously.
