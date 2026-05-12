# Arc 170 slice 3 Gap G SCORE — Path E strict-isolation shape for `deftest-hermetic`

**Status: BLOCKED (rows A/B/C) / PASS (rows D/E/F)**

Path E macro migration cannot land. The substrate's runtime evaluator returns
`DefineInExpressionPosition` for any `(:wat::core::define ...)` form reached
via `eval()` — which is the path taken when define appears inside a
`(:wat::core::do ...)` in a fn body. Existing callers of `make-deftest-hermetic`
(specifically `wat-tests/kernel/services/ambient-stdio.wat`) use `define` forms
in their preludes. Under Path E those defines land in the child's fn body `do`
and fail at child runtime, exiting 1.

The existing `run-sandboxed-hermetic-ast` mechanism is retained. Strict parent
isolation already holds under it — prelude forms inside `(:wat::core::forms ...)`
are quoted AST data invisible to the parent's freeze pipeline. The 4 enforcement
probes verify this contract holds (rows D/E/F all PASS).

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `:wat::test::deftest-hermetic` body uses Path E shape | BLOCKED | See analysis below |
| B | `:wat::test::make-deftest-hermetic` factory follows Path E | BLOCKED | Blocked by same gap |
| C | Documentation header explicitly names "strict isolation" contract | BLOCKED | Blocked; blocking comment added instead |
| D | 4+ enforcement probes pass | PASS | 4/4, `2227 passed; 0 failed` |
| E | Workspace at baseline + new probes / 0 failed | PASS | 2223 → 2227 / 0 failed |
| F | Existing deftest-hermetic users work correctly | PASS | 2227/0; ambient-stdio passes |

## Why Path E is blocked

### The substrate gap

`deftest-hermetic`'s Path E target expansion:

```scheme
`(:wat::core::define (~name -> :wat::kernel::RunResult)
   (:wat::test::run-hermetic
     (:wat::core::do
       ~@prelude
       ~body)))
```

`run-hermetic` is a Phase C macro that wraps its body argument in a fn:

```scheme
(:wat::core::fn
  [_rx <- :wat::kernel::Receiver<wat::core::nil>
   _tx <- :wat::kernel::Sender<wat::core::nil>]
  -> :wat::core::nil
  <body>)
```

The fn is passed to `spawn-process`, which forks a child. In the child,
`spawn_process_child_branch` calls:

1. `startup_from_forms(prologue)` — processes the closure's captured deps/types
2. `eval(entry_form)` → `apply_function(fn, args)` → `eval_tail(fn_body)`

`fn_body` is `(:wat::core::do ~@prelude ~body)`. `eval_do_tail` evaluates each
non-final sub-form via `eval()`. When a prelude sub-form is a `define`:

```rust
// src/runtime.rs
":wat::core::define" => Err(RuntimeError::DefineInExpressionPosition(list_span.clone())),
```

The child exits with `EXIT_RUNTIME_ERROR` (code 1). The parent sees
`RunResult { failure: Some("DefineInExpressionPosition ...") }`.

### Why Gap F-1/F-2/F-3 did not help

All three gaps fixed PARENT-SIDE freeze pipeline issues:
- **F-1**: struct/enum pre-registration in `preregister_fn_defs_in_do` — parent resolver
- **F-2**: resolver quote-awareness for `forms`/`quote`/`quasiquote` — parent resolver
- **F-3**: parent type registry inheritance to spawn-process child — `extract_closure`

None address child RUNTIME evaluation of `define` forms. The gap is in
`eval()` / `eval_do_tail()` in `src/runtime.rs`.

### Affected callers

`wat-tests/kernel/services/ambient-stdio.wat` uses `make-deftest-hermetic` with a
default-prelude containing 5 `define` forms (Layer 0–4 helpers). Under Path E:
all 5 ambient-stdio tests exit 1. This is the primary blocking caller.

### What would unblock Path E

**(a)** A substrate capability allowing `define` at expression position inside a fn
body's `do` — runtime fn registration. This is a non-trivial evaluator extension
(defines accumulate in a local env frame rather than the frozen symbol table).

**(b)** A caller sweep migrating all prelude `define` forms to a form that works at
runtime (e.g., `let`-bound fn values or top-level module definitions loaded via
`load-file!`). This requires modifying `ambient-stdio.wat` and any other callers.

Both paths are forward work. This arc cannot proceed to Path E without one of them.

## What was shipped

### Blocking comment in `wat/test.wat` (lines 332–343)

Added to the `deftest-hermetic` header to name the gap explicitly:

```
;; Arc 170 slice 3 Gap G — Path E migration BLOCKED by substrate gap:
;; (:wat::core::define ...) forms inside a (:wat::core::fn ...) body's
;; (:wat::core::do ...) cannot be evaluated at child runtime — the
;; evaluator returns DefineInExpressionPosition. Existing callers
;; (ambient-stdio.wat via make-deftest-hermetic) use define forms in
;; their preludes. Path E requires either:
;;   (a) a substrate capability to evaluate define at expression
;;       position inside a do (runtime registration of fns), OR
;;   (b) caller sweep: move all define-form preludes out of deftest-
;;       hermetic's prelude and into the test body or separate files.
;; See SCORE-SLICE-3-GAP-G-PATH-E-ISOLATION.md for full analysis.
;; Staying on run-sandboxed-hermetic-ast until the substrate gap closes.
```

### Enforcement probes: `tests/probe_deftest_hermetic_isolation.rs`

4 probes verifying the isolation contract of the current forms-based mechanism.
All pass under `cargo test --release --test probe_deftest_hermetic_isolation`.

## Probe design rationale

The isolation contract being proven is: **the parent's frozen symbol table and
type registry are UNTOUCHED by any content declared in a `deftest-hermetic`
prelude.** Only the test fn registration (`:name`) appears at the parent's top
level. Everything else stays in the child's sandboxed world.

This contract already holds under `run-sandboxed-hermetic-ast` because prelude
forms inside `(:wat::core::forms ...)` are quoted AST data — the outer freeze
pipeline's `register_types` (step 5) and `register_defines` (step 6) never see
forms inside a `forms` call. Gap F-2 (resolver quote-awareness) additionally
prevents the resolver from walking into `forms` arguments.

### Probe 1 — `probe_parent_has_no_prelude_struct_accessors`

Declares a struct (`:test::g::IsolatedType`) in the prelude. Asserts:
- `world.symbols().get(":test::g::my-hermetic-test")` — IS present (test runner finds it)
- `world.symbols().get(":test::g::IsolatedType/new")` — NOT present (isolation holds)
- `world.symbols().get(":test::g::IsolatedType/field")` — NOT present
- `world.types().get(":test::g::IsolatedType")` — NOT present in TypeEnv

Demonstrates: parent's frozen world is untouched by prelude struct declarations.

### Probe 2 — `probe_cross_test_prelude_isolation_same_fqdn_no_collision`

Two `deftest-hermetic` calls in the same file each declare a struct with the
same FQDN (`:test::g::SharedName`) in their prelude — one with `(value i64)`,
one with `(label String)`. Asserts:
- Both test fns ARE in parent (`:test::g::first-hermetic-test`, `second-hermetic-test`)
- `:test::g::SharedName` NOT in parent TypeEnv (from either prelude)
- `:test::g::SharedName/new` NOT in parent sym

Demonstrates: prelude isolation is per-test — preludes are independent, no
shared parent-side type registry entry, no cross-test contamination or collision.

### Probe 3 — `probe_test_fn_visible_prelude_content_invisible`

The test fn entry point (`:test::g::visible-test`) IS at the parent's top level.
The prelude declares `:test::g::HiddenStruct` (two fields) and a helper define
`:test::g::hidden-helper`. Asserts:
- `:test::g::visible-test` IS in parent (test runner boundary)
- `:test::g::HiddenStruct` NOT in parent TypeEnv
- `:test::g::HiddenStruct/new`, `/x` NOT in parent sym
- `:test::g::hidden-helper` NOT in parent sym

Demonstrates: exactly ONE thing crosses the parent/child boundary at the parent
level — the test fn's registration. The helper define form also stays in child.

### Probe 4 — `probe_make_deftest_hermetic_define_prelude_parent_isolated`

Mirrors the structure of `wat-tests/kernel/services/ambient-stdio.wat` —
`make-deftest-hermetic` with a default-prelude containing a `define` form.
The define (`:test::g::run-inner`) calls `run-hermetic-ast` with a nested
`program` form. Asserts:
- Generated test fn IS in parent (`:test::g::using-make-deftest-hermetic`)
- `:test::g::run-inner` NOT in parent sym (define was in forms, never registered)

Demonstrates: `make-deftest-hermetic` with `define` preludes compiles cleanly
and achieves parent isolation. The `~~default-prelude` double-unquote composition
still works — the prelude forms land inside `forms` in the child.

## Documentation header wording (for orchestrator review)

The current `deftest-hermetic` header describes the mechanism (fork, thread-safe
stdio) but does not explicitly name the strict-isolation contract. The blocked
comment added in this arc is diagnostic, not user-facing. When Path E lands, the
header should read:

```
;; ─── deftest-hermetic — strict-isolation fork-isolated test ────────────
;;
;; Prelude forms run INSIDE the test's child subprocess. The parent's
;; frozen symbol table does NOT receive the prelude's types, structs, or
;; helper defines. Use when test setup must not pollute the parent's world.
;;
;; Compare to `deftest`: deftest splices prelude at outer top-level
;; (parent has the prelude content). Use deftest when prelude is
;; file-level shared setup; use deftest-hermetic when prelude is
;; sandbox-internal.
;;
;; The test fn entry point (`:name`) IS registered at the parent's top
;; level so the test runner can discover and invoke it. Everything else
;; stays in the child's world.
```

This header is NOT shipped in this arc — blocked until Path E implementation
can land.

## Honest deltas

### Delta 1 — Path E blocked by `DefineInExpressionPosition` (primary finding)

The substrate's runtime evaluator hard-rejects `define` at expression position.
This is not a configuration issue or a missing flag — it is a categorical
rejection in `src/runtime.rs`. Existing callers have `define` forms in their
preludes. Path E cannot proceed without either a runtime extension or a caller
sweep.

The BRIEF expected this arc to land all 6 rows PASS. Only rows D/E/F pass.

### Delta 2 — Isolation contract already holds under old mechanism

The `run-sandboxed-hermetic-ast` mechanism already achieves strict parent
isolation. The probes confirm this. Path E would preserve the isolation
contract while also allowing the child to access prelude content at runtime
via `startup_from_forms` — but that benefit is available only once the
`define`-at-expression-position gap closes.

The forms-based mechanism achieves parent isolation but routes prelude through
`startup_from_forms` (step 6 `register_defines`) rather than `eval()` — which
is precisely what makes it work for `define` preludes and what Path E loses.

### Delta 3 — Probe 3 scope: body's view of parent config

The BRIEF proposed a probe demonstrating that the child cannot reach into the
parent's runtime (immutable inherited config). This was not implemented.
The parent-isolation contract (parent symbol table untouched by child prelude)
is the stronger and more actionable assertion — it is directly tested by the
freeze pipeline assertions. The child-side runtime isolation is enforced by the
subprocess boundary (the child runs in a forked process with COW symbol table),
not by the macro shape, so it does not need a new probe to demonstrate. Probes
3 and 4 cover the test-fn-visible / prelude-invisible boundary instead.

### Delta 4 — make-deftest-hermetic `~~default-prelude` composition

Double-unquote (`~~default-prelude`) in `make-deftest-hermetic` works correctly
through the forms-based path. Probe 4 confirms this — the generated test fn
appears in the parent's symbol table and the prelude helper does not. No change
required in the factory's composition logic.

## Verification commands run

```
cargo test --release --test probe_deftest_hermetic_isolation
# → 4 passed; 0 failed

cargo test --release 2>&1 | grep "^test result:" | awk '{passed+=$4; failed+=$6} END {print "passed:"passed" failed:"failed}'
# → passed:2227 failed:0
```

Baseline (post F-1+F-3+F-2): 2223/0. Post-G: 2227/0 (+4 probes).

## Files modified / created

| File | Action |
|------|--------|
| `wat/test.wat` lines 332–343 | Blocking comment added to deftest-hermetic header |
| `tests/probe_deftest_hermetic_isolation.rs` | CREATED — 4 isolation probes |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-G-PATH-E-ISOLATION.md` | CREATED — this file |

`wat/test.wat` macro bodies (deftest-hermetic, make-deftest-hermetic) are
**unchanged** from the pre-arc state — Path E rewrite reverted after 5 test
failures confirmed the substrate gap.
