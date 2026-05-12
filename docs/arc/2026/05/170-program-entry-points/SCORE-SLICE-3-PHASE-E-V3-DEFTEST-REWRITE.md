# Arc 170 slice 3 Phase E V3 — SCORE (deftest macro rewrite via top-level `do` + run-hermetic)

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** BLOCKED — new substrate gap discovered; `preregister_fn_defs_in_do` does not handle `define` forms; baseline preserved at 2205/0

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::test::deftest` body uses `(:wat::core::do ~@prelude (:wat::core::define ...))` + `(:wat::test::run-hermetic ~body)` | grep — no `run-sandboxed-ast` or `:wat::core::forms` in deftest expansion | BLOCKED — rewrite attempted, 263 failures; reverted to baseline |
| B | `:wat::test::deftest-hermetic` body rewritten (Path A collapse chosen; surface rationale) | grep — no `run-sandboxed-hermetic-ast` | BLOCKED — same reason; reverted |
| C | `make-deftest` + `make-deftest-hermetic` factories still compose | workspace test passes | NOT REACHED — blocked by substrate gap |
| D | `cargo test --release --workspace --no-fail-fast`: 2205 passed / 0 failed | full test run | PRESERVED — baseline 2205/0 unchanged after revert |
| E | Documentation headers updated to reflect new expansion | manual review | NOT COMMITTED — draft header written; reverted with macro body |
| F | Phase F readiness inventory: remaining run-sandboxed-* callers documented | SCORE inventory | PASS — inventory below |

**Row D passes (baseline preserved). Rows A, B, C, E blocked. Row F passes.**

---

## PIECE 1 — Substrate readiness (verified)

```
cargo test --release --test probe_do_splice_def

running 3 tests
test probe_do_defn_via_expansion ... ok
test probe_do_def_two_vars_visible ... ok
test probe_do_def_via_macro_emission ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Gap C V2 probes pass. Substrate IS ready for `def`/`defn`-inside-`do`. The gap is one layer deeper.

---

## Root cause — preregister_fn_defs_in_do does not handle `define` forms

### What was attempted

The V3 rewrite produced the BRIEF's target expansion:

```scheme
(:wat::core::do
  ~@prelude
  (:wat::core::define (~name -> :wat::kernel::RunResult)
    (:wat::test::run-hermetic ~body)))
```

This expansion was written to `wat/test.wat` and the workspace was run. Result: 263 failures, all with the same error pattern:

```
test-runner: /home/watmin/work/holon/wat-rs/wat-tests/edn/roundtrip.wat: startup: resolve: 7 unresolved reference(s):
  - :wat-tests::edn::roundtrip-i64 (call head — not a builtin, not a registered function)
  - :wat-tests::edn::roundtrip-string (call head — not a builtin, not a registered function)
  [... all test functions in the file ...]
```

Every test in every test file failed with "not a registered function" — the test functions themselves were not visible in `sym.functions` after startup.

### The gap

`register_defines` (src/runtime.rs:1443) was extended by Gap C V2 to handle `(:wat::core::do ...)` wrapper forms at top level. When it encounters a `do` form, it calls `preregister_fn_defs_in_do` (src/runtime.rs:2246) to peek inside and register any fn-shape defs.

`preregister_fn_defs_in_do` iterates the children of the `do` form and calls `try_parse_fn_shape_def` on each child. `try_parse_fn_shape_def` recognizes `(:wat::core::def :name (:wat::core::fn ...))` shapes — this covers `def` and `defn` (which expands to `def`).

`preregister_fn_defs_in_do` does NOT call `is_define_form` + `parse_define_form`. It does not recognize `(:wat::core::define (:name -> :type) body)` shapes.

The new deftest expansion emits `(:wat::core::define (~name -> :wat::kernel::RunResult) ...)` inside the `do`. This form:
- Is NOT a `def`-of-fn shape → `try_parse_fn_shape_def` returns `None`
- IS a `define` form → `is_define_form` would return `true`, but it is never called

Result: the test function `~name` is never inserted into `sym.functions`. The `do` form itself stays in `rest` (correct). But when `resolve_references` runs at step 7, `sym.functions` is missing the test function names. Every call to those functions in the file produces `UnresolvedReference`. The test runner's `startup_from_source` returns an error before `discover_tests` is even reached.

### Why Gap C V2 probes passed but Phase E V3 failed

All three Gap C V2 probes (`tests/probe_do_splice_def.rs`) use `(:wat::core::def ...)` or `(:wat::core::defn ...)` forms inside the `do` wrapper. `defn` is a macro that expands to `(:wat::core::def :name (:wat::core::fn ...))` — a fn-shape def. `preregister_fn_defs_in_do` handles this via `try_parse_fn_shape_def`.

Gap C V2 SCORE (cross-reference) notes: "Phase E V3 (next): deftest macro emits `(:wat::core::do ~@prelude (:wat::core::defn ~name ...))`." That framing used `defn` as the expected form. The actual BRIEF target shape uses `(:wat::core::define (:name -> :type) body)` — a `define` form, not `defn`. The gap is between what the probes validated and what deftest actually emits.

### The fix — Gap E (proposed name)

`preregister_fn_defs_in_do` must be extended to also handle `define` forms. The pattern mirrors what `register_defines` does at the top level:

```rust
// Current (in preregister_fn_defs_in_do, src/runtime.rs:2254):
if let Some((path, func)) = try_parse_fn_shape_def(child) {
    ...
}

// Needed addition (after the try_parse_fn_shape_def arm):
} else if is_define_form(child) {
    let (path, func) = parse_define_form(child.clone())?;
    if check_reserved_prefix && crate::resolve::is_reserved_prefix(&path) {
        let span = child.span().clone();
        return Err(RuntimeError::ReservedPrefix(path, span));
    }
    if !sym.functions.contains_key(&path) {
        sym.functions.insert(path, func);
    }
}
```

Note: `parse_define_form` takes ownership (`form: WatAST`), requiring `.clone()` on the child since it stays in the `do` form in `rest`. Alternatively, a borrowing variant `parse_define_form_ref` could be written to avoid the clone — but the clone is correct and minimal.

`preregister_fn_defs_in_let` (src/runtime.rs:2293) has the identical gap and needs the identical addition for consistency, even though Phase E does not use `let` as the outer wrapper.

A probe for this gap:

```rust
#[test]
fn probe_do_define_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::helper -> :wat::core::i64)
            42)
          (:wat::core::define (:my::main -> :wat::core::i64)
            (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}
```

This probe fails before the fix and passes after.

The macro-emission variant (the Phase E use case directly):

```rust
#[test]
fn probe_do_define_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::define (:my::helper -> :wat::core::i64)
               42)
             ~body))

        (:my::probe (:wat::core::define (:my::main -> :wat::core::i64) (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}
```

---

## deftest-hermetic — Path A rationale (for orchestrator record)

Path A was chosen during the rewrite attempt (before the substrate gap was discovered). Rationale surfaced per BRIEF requirement:

**Path A — collapse to alias.** The new `run-hermetic` macro (arc 170 slice 3 phase C) is hermetic-by-default — it spawns a subprocess via `spawn-process`. Under the new architecture, every `deftest` call is already hermetic. The name `deftest-hermetic` is a vestigial distinction from the old `run-sandboxed-hermetic-ast` era (which required an explicit fork vs in-process choice).

Path A is the honest path:
- **Obvious**: two macros with identical expansions is confusing.
- **Simple**: one expansion path, one maintenance surface.
- **Honest**: keeping `deftest-hermetic` with different behavior would be misleading since the behavior is now the same. Keeping it as an alias preserves call-site compatibility during migration without lying about what it does.
- **Good UX**: callers that use `deftest-hermetic` because they explicitly want subprocess isolation get exactly that — they just get it from the same mechanism as `deftest`.

Path B (distinct identity) was rejected: under the new substrate, the identity distinction has no behavioral backing. The name would be documentation-only, and documentation-only distinctions without behavioral backing are honest only if documented as such. The SCORE header comment in the reverted draft made this explicit.

Phase F can retire the `deftest-hermetic` name or make it a documented synonym in a sweep arc.

---

## Phase F readiness — remaining run-sandboxed-* callers

Post-Phase-E-V3 (when it ships), the callers will be:

In `wat/test.wat`:
1. `run-ast` wrapper (line 238) — calls `:wat::kernel::run-sandboxed-ast` directly
2. `run-hermetic-ast` wrapper (line 258) — calls `:wat::kernel::run-sandboxed-hermetic-ast` directly

In `wat/kernel/hermetic.wat`:
3. Line ~100 — calls `:wat::kernel::run-sandboxed-hermetic-ast<I,O>` — stdlib implementation; not a deftest caller; Phase F audits this separately

Items 3 (deftest) and 4 (deftest-hermetic) from the V2 SCORE count will be retired when Phase E V3 ships.

**Phase F scope after Phase E V3 ships: 3 callers** (down from 5: deftest + deftest-hermetic removed from the count).

---

## Honest deltas

### Delta 1 — Gap C V2 was verified with def/defn; Phase E needs define

The three Gap C V2 probes all use `(:wat::core::def ...)` or `(:wat::core::defn ...)` (which expands to `def`). `preregister_fn_defs_in_do` was extended to handle fn-shape `def` forms via `try_parse_fn_shape_def`. The deftest target expansion uses `(:wat::core::define ...)`. `preregister_fn_defs_in_do` has no arm for `define` forms. The probes passed; Phase E V3 failed because the two form types are structurally different.

The Gap C V2 SCORE's Phase E V3 note said "deftest macro emits `(:wat::core::do ~@prelude (:wat::core::defn ~name ...))`." The BRIEF's target shape uses `define`. This mismatch between the SCORE's forward-looking note and the BRIEF's actual target was the hidden assumption that broke.

### Delta 2 — The fix is equally small and well-bounded

The gap is one additional arm in `preregister_fn_defs_in_do` (and its `let` mirror). `is_define_form` and `parse_define_form` already exist and are already called from `register_defines` at the top level. The fix is adding the same call into the helper. ~8-10 lines per helper, no new functions needed.

### Delta 3 — Baseline preserved; no regression introduced

The rewrite attempt was fully reverted. `wat/test.wat` is at the pre-Phase-E-V3 state. Workspace is at 2205/0. The probe files (`tests/probe_do_splice_def.rs`) are unchanged.

### Delta 4 — preregister_fn_defs_in_let has the same gap

`preregister_fn_defs_in_let` (src/runtime.rs:2293) also only calls `try_parse_fn_shape_def`. If a `define` form appears in a `let` body at top level, it would have the same resolve-time failure. This gap exists regardless of Phase E. It should be fixed in the same substrate arc that fixes `preregister_fn_defs_in_do`.

### Delta 5 — Workspace drop was universal, not prelude-specific

The 263 failures were not a prelude semantic shift affecting a subset of tests. ALL deftest-expanded tests in every test file failed because the test function name itself was not registered. The failure is at the registration layer, before any prelude content is even considered.

---

## Substrate arc needed (Gap E)

**One substrate change unblocks Phase E V4:**

Extend `preregister_fn_defs_in_do` (src/runtime.rs:2246) to also handle `(:wat::core::define ...)` forms using `is_define_form` + `parse_define_form`. Extend `preregister_fn_defs_in_let` (src/runtime.rs:2293) with the identical arm for consistency.

Add two regression probes in a new file `tests/probe_do_splice_define.rs`:
1. Two `define` forms inside a top-level `do` — both must register
2. Macro-emitted `do` wrapping a `define` — must register (the Phase E use case directly)

Once Gap E ships, Phase E V4 can attempt the macro rewrite with the expectation that `(:wat::core::define ...)` inside `do` is pre-registered.

---

## TestResult typealias verification

`:wat::test::TestResult` is defined as a typealias for `:wat::kernel::RunResult` in `wat/test.wat`. The reverted deftest expansion uses `:wat::test::TestResult` as the return type annotation. The target expansion (Phase E V4) should use `:wat::kernel::RunResult` directly (since the test function return type is now explicitly `RunResult` via `run-hermetic`). Both names are accepted by `is_test_function` in `src/test_runner.rs:617-618`. The typealias remains valid and callers using either name continue to work.

---

## Documentation header draft (for orchestrator review)

The following header was drafted for `wat/test.wat` lines 260+ and is reproduced here for record. It was reverted with the macro body:

```
;; ─── deftest — Clojure-style ergonomic shell (arc 007 slice 3b; arc 027 slice 4; arc 031; arc 170 slice 3 phase E) ───
;;
;; Registers a named zero-arg test function that returns RunResult.
;; The body runs in a hermetic subprocess via :wat::test::run-hermetic
;; (arc 170 slice 3 phase C). Subprocess isolation is the default —
;; every deftest is hermetic. The `prelude` list splices top-level
;; forms (loads, type declarations, defmacros) at the deftest's
;; EXPANSION SITE under (:wat::core::do ...), registering them in
;; the outer symbol table at freeze time. Empty `()` prelude = no
;; startup forms, the minimal shape.
;;
;; Expansion:
;;
;;   (:wat::core::do
;;     <prelude spliced here — top-level forms at expansion site>
;;     (:wat::core::define (:my::test::two-plus-two -> :wat::kernel::RunResult)
;;       (:wat::test::run-hermetic <body>)))
;;
;; The `do` wrapper is handled by Gap C V2 + Gap E extended register_defines
;; (arc 170 slice 3), which recurses into top-level `do` forms and
;; registers nested define forms in sym.functions at freeze time.
```

---

## Files modified

| File | Change |
|------|--------|
| `wat/test.wat` | Rewrite attempted; 263 failures; reverted. Net zero change. |
| `docs/arc/.../SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` | This file. |

**Baseline: 2205 passed / 0 failed. Unchanged.**

---

## Orchestrator direction needed

**One substrate change (Gap E) unblocks Phase E V4:**

Extend `preregister_fn_defs_in_do` and `preregister_fn_defs_in_let` in `src/runtime.rs` to also handle `(:wat::core::define ...)` forms alongside fn-shape `def` forms. The fix calls `is_define_form` + `parse_define_form` in the same helper, mirroring what `register_defines` already does at the top level.

Gap E probe file: `tests/probe_do_splice_define.rs` (new file; two probes).

Once Gap E ships with green probes, Phase E V4 re-attempts the macro rewrite — the target expansion is unchanged from Phase E V3's BRIEF.
