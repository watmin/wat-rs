# Arc 031 — sandbox inherits outer config — BACKLOG

**Shape:** four slices, leaves-to-root. Status markers:
- **ready** — dependencies satisfied; can be written now
- **obvious in shape** — will be ready when the prior slice lands
- **foggy** — needs design work before it's ready

---

## Slice 1 — substrate: run-sandboxed-ast inherits caller's Config

**Status: ready.**

Targets: `src/sandbox.rs`, `src/runtime.rs`.

Changes:
- The sandbox entry that today calls `startup_from_forms` with a
  fresh default Config receives the CALLER's committed Config as
  the starting baseline instead. The caller's Config is already
  reachable from the runtime dispatch site (the active
  `FrozenWorld` / `SymbolTable` holds it).
- The freeze pipeline inside the sandbox continues to process
  inner setters exactly as today — entry-position setters commit,
  last-setter-wins, non-entry setters are rejected. Only the
  starting value changes.
- Same change applied symmetrically to
  `run-sandboxed-hermetic-ast` (arc 012 fork-based sibling). The
  child process receives the parent's committed Config as part of
  its starting state alongside the `Vec<WatAST>` forms.

**Rust unit tests:**
- `sandbox_inherits_outer_dims_when_inner_unset`
- `sandbox_inherits_outer_capacity_mode_when_inner_unset`
- `sandbox_inner_setter_overrides_inherited` — inner set-dims!
  still wins
- `hermetic_sandbox_inherits_outer_config_through_fork`
- `sandbox_with_both_setters_works_like_today` — back-compat for
  explicit-setter callers

**Sub-fogs:**
- **1a — where does the caller's Config live at the dispatch site?**
  `eval_kernel_run_sandboxed_ast` lives in `src/runtime.rs`; it
  has `sym: &SymbolTable` which carries the current frozen config
  via `sym.config()`. Verify at implementation.
- **1b — fork inheritance.** Fork copies the parent's process
  memory COW. The parent's active `FrozenWorld` is in memory.
  Decide at implementation whether to pass the Config as an
  explicit parameter into the child's `startup_from_forms` or
  whether the child re-reads it from the inherited world
  pointer. Explicit parameter is honester.

## Slice 2 — deftest / deftest-hermetic drop mode + dims

**Status: obvious in shape** (once slice 1 lands).

Targets: `wat/std/test.wat`, all deftest callsites across the
workspace.

Changes:
- `:wat::test::deftest` signature shrinks from `(name mode dims
  prelude body)` to `(name prelude body)`. Template stops emitting
  `(set-capacity-mode! ,mode)` and `(set-dims! ,dims)`.
- Same for `:wat::test::deftest-hermetic`.
- Doc comments updated to show the new shape.
- All direct deftest callsites migrated: `(deftest :name :error 1024 () body)`
  → `(deftest :name () body)`. Arc 030 just swept these for the
  arg-order flip; same files touched again for the drop.

Files (from arc 030's 16-file callsite sweep):
- `wat/std/test.wat` (the definition)
- `wat-tests/holon/Circular.wat`
- `wat-tests/holon/Reject.wat`
- `wat-tests/holon/Sequential.wat`
- `wat-tests/holon/Subtract.wat`
- `wat-tests/holon/Trigram.wat`
- `wat-tests/holon/coincident.wat`
- `wat-tests/holon/eval_coincident.wat`
- `wat-tests/std/service/Console.wat`
- `wat-tests/std/stream.wat`
- `wat-tests/std/test.wat`
- `crates/wat-lru/wat-tests/CacheService.wat`
- `crates/wat-lru/wat-tests/LocalCache.wat`
- `examples/with-loader/wat-tests/test_loader.wat`
- `tests/wat_make_deftest.rs`
- `tests/wat_test_cli.rs`
- `holon-lab-trading/wat-tests/test_scaffold.wat` (lab repo)

**Sub-fogs:**
- **2a — deftest's test-file preamble commitment.** Every file
  using direct deftest (not via factory) still needs its outer
  `(:wat::config::set-capacity-mode! :error)` +
  `(:wat::config::set-dims! 1024)` at the top. Verify that
  `wat-tests/` files in the workspace already have those (they
  do — the test_runner requires entries to commit config). Lab's
  `test_scaffold.wat` likewise already has them.

## Slice 3 — make-deftest / make-deftest-hermetic drop mode + dims

**Status: obvious in shape** (once slice 2 lands).

Targets: `wat/std/test.wat` (factory macros), lab test files that
use the factory.

Changes:
- `:wat::test::make-deftest` signature shrinks from
  `(name mode dims default-prelude)` to `(name default-prelude)`.
  Factory-generated defmacro's template calls the now-simpler
  `:wat::test::deftest` with just `(name prelude body)` shape.
- Same for `:wat::test::make-deftest-hermetic`.
- Nested-quasiquote template updates: `,,mode ,,dims ,,default-prelude`
  → just `,,default-prelude`. Arc 029's `expand_form` quote-preserve
  still applies; the nested `,,` resolution pattern is unchanged
  beyond having one argument to resolve instead of three.
- Doc comments updated.

Lab migration:
- `holon-lab-trading/wat-tests/vocab/shared/time.wat` preamble:
  `(:wat::test::make-deftest :deftest :error 1024 ((...)))` →
  `(:wat::test::make-deftest :deftest ((...)))`.

**Sub-fogs:**
- **3a — arc 029 wat-level test.** `tests/wat_make_deftest.rs`
  asserts on the registered `:my-deftest` body shape. That shape
  changes (fewer args). Update the assertion to match.

## Slice 4 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slices 1-3 land).

Writing:
- `docs/arc/2026/04/031-sandbox-inherits-config/INSCRIPTION.md` —
  standard shape. What shipped slice by slice, commit refs,
  sub-fog resolutions named with the code.
- `docs/USER-GUIDE.md` — update the Testing section. Remove
  mode/dims from every test-macro example. Point at arc 031 as
  the reason for the simpler shape.
- `docs/CONVENTIONS.md` — short note on "entry commits config,
  sandboxes inherit" rule. Cross-reference arc 027's loader
  inheritance pattern.
- `wat-tests/README.md` — example deftest calls shrink by two
  arguments.
- `docs/README.md` — arc index gains row 031.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — new row naming the factory-arg shrinkage and the
  config-inheritance semantic for `run-sandboxed-ast`.

**Sub-fogs:**
- **4a — BOOK chapter?** Defer. Chapter 32 narrated the cold
  boot; arcs 027-030 closed as code+docs without a dedicated
  chapter. Arc 031 can ride the same pattern unless it surfaces
  something chapter-worthy during implementation.

---

## Working notes (updated as slices land)

- Opened 2026-04-23 following arc 030's arg-order flip close
  ("yes — B — that's the form" confirmation of Path B).
- This is the second half of the make-deftest ergonomics story;
  the first half was arc 029's nested-quasiquote substrate that
  let the factory exist at all.
