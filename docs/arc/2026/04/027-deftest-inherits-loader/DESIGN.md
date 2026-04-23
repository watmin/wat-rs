# Arc 027 — deftest just works (relative paths + loader inheritance)

**Status:** opened 2026-04-23. Expanded scope 2026-04-23 after
builder pushed past the CompositeLoader approach: *"i think we
take a typescript stance and do ./wat/.... ?.. relative paths to
where its being expressed?.. we can... have a resolver to full
path and ignore reloading a file we've already loaded?...."*

**Motivation.** Lab arc 001's tests surfaced the cost of
deftest's current scope discipline: inner sandbox can't reach
the lab's `.wat` tree because deftest hard-codes `:None` scope,
forcing manual `run-sandboxed-ast` with explicit scope + 7
`(load!)` boilerplate lines per test. The fix has three
layers, all cohesive:

1. `(load!)` gains TypeScript-style relative paths (`./x.wat`,
   `../y.wat`) resolving against the file containing the call.
2. Sandbox primitives inherit the outer loader when scope is
   `:None` (the original arc 027 scope).
3. `wat::test!` default loader widens to `CARGO_MANIFEST_DIR`
   so tests can traverse both `wat/` and `wat-tests/` via
   relative paths.

Plus one cross-cutting: **canonical-path dedup** — second load
of a resolved canonical path is a no-op, preventing duplicate-
define errors on diamond dependencies.

---

## The current shape

**`:wat::test::deftest`** emits `run-sandboxed-ast ... :None`.
`:None` in the sandbox primitive means `InMemoryLoader::new()`
— no filesystem reach. Lab tests can't use deftest directly.

**`(load!)`** only understands scope-root-relative paths today.
Bare `"foo.wat"` resolves against `ScopedLoader`'s scope root.
No `./` or `../` support. No file-relative resolution.

**`wat::test!`** defaults `loader:` to `"wat-tests"`. A test
whose body needs to reach `../wat/types/candle.wat` can't —
the loader's scope clamps at `wat-tests/`.

---

## The fix — three layers, one arc

### Layer 1 — relative paths in `(load!)`

```scheme
;; Bare path — SCOPE-ROOT relative, unchanged from today.
(:wat::load-file! "helpers.wat")

;; Dotted path — FILE relative, resolved against the
;; directory of the file containing this (load!) call.
(:wat::load-file! "./helpers.wat")
(:wat::load-file! "../shared/common.wat")
```

Resolution:
1. If path starts with `./` or `../` — strip `./`, resolve
   against **current file's directory**.
2. Otherwise — resolve against **loader's scope root** (current
   behavior, no change).
3. Resolved path is canonicalized; final check is the loader's
   standard canonical-path clamp (ScopedLoader refuses anything
   above scope root).

**Current-file threading.** `resolve_loads` already walks a
stack of source files during load resolution (for cycle
detection). Extending the stack to carry the current file's
directory is ~30 lines of Rust. The stack frame grows one
`PathBuf` field; load-call site queries `stack.top().parent_dir`
when it sees `./` or `../`.

### Layer 2 — canonical-path dedup

`resolve_loads` tracks a `HashSet<PathBuf>` of already-resolved
canonical paths. Each `(load!)` call:

1. Resolve path (relative or scope-relative) to canonical.
2. If canonical already in set → no-op (return Ok, don't
   re-parse, don't re-emit forms).
3. Otherwise → insert, parse, recurse, re-emit.

Cycle detection fires **before** dedup (the in-progress stack is
separate from the completed-set). Diamond dependency resolves
correctly: A loads B and C; B loads D; C loads D; D parses once.

### Layer 3 — :None scope inherits outer loader

`eval_kernel_run_sandboxed_ast` (and 3 siblings) — when scope is
`:None`, clone the outer `SymbolTable`'s `source_loader` instead
of creating a new `InMemoryLoader`. The outer loader is already
there (set at freeze time). Sandbox primitives just ignore it
today.

Sandbox primitives keep `InMemoryLoader` as the fallback if the
outer `SymbolTable` has no loader attached (defensive; shouldn't
happen after a normal freeze).

### Layer 4 — `wat::test!` default scope widens

`wat::test! {}` with no explicit `loader:` expands to a
`ScopedLoader` rooted at `CARGO_MANIFEST_DIR` (crate root), not
at `path:` (which was `"wat-tests"`). That lets tests traverse
up and over into sibling trees:

```scheme
;; wat-tests/vocab/shared/time.wat:
(:wat::load-file! "../../wat/types/candle.wat")
;; Resolves to CARGO_MANIFEST_DIR/wat/types/candle.wat. Legal —
;; stays within the crate-root scope clamp.
```

`wat::main!`'s default stays at `"wat"` (production binary narrow
scope). Explicit `loader:` overrides on either macro continue to
win.

---

## The lab tests after arc 027

Current `wat-tests/vocab/shared/time.wat` per test (6 tests, 42
load lines total):

```scheme
(:wat::core::define
  (:trading::test::vocab::shared::time::test-encode-time-facts-count
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-ast
    (:wat::test::program
      (:wat::config::set-capacity-mode! :error)
      (:wat::config::set-dims! 1024)
      (:wat::load-file! "types/enums.wat")
      (:wat::load-file! "types/newtypes.wat")
      (:wat::load-file! "types/ohlcv.wat")
      (:wat::load-file! "types/distances.wat")
      (:wat::load-file! "types/pivot.wat")
      (:wat::load-file! "types/candle.wat")
      (:wat::load-file! "vocab/shared/time.wat")
      (:wat::core::define (:user::main ...) -> :()) ...)
    (:wat::core::vec :String)
    (Some "wat")))
```

After arc 027:

```scheme
(:wat::test::deftest :trading::test::vocab::shared::time::test-encode-time-facts-count 1024 :error
  ;; deftest inherits the outer test binary's loader (arc 027 layer 3).
  ;; The outer loader is a ScopedLoader at CARGO_MANIFEST_DIR
  ;; (arc 027 layer 4).
  (:wat::load-file! "../../wat/types/candle.wat")
  (:wat::load-file! "../../wat/vocab/shared/time.wat")
  ;; Canonical-path dedup (arc 027 layer 2) means if any prior
  ;; test already loaded these, the second load is a no-op — just
  ;; makes the defines visible in this test's scope.
  (:wat::core::let*
    (((t :trading::types::Candle::Time)
      (:trading::types::Candle::Time/new 30.0 14.0 3.0 15.0 6.0))
     ((facts :Vec<wat::holon::HolonAST>)
      (:trading::vocab::shared::time::encode-time-facts t)))
    (:wat::test::assert-eq (:wat::core::length facts) 5)))
```

42 boilerplate lines → 12 lines (two explicit relative loads per
test + the deftest shell). Load cascade for types resolves via
candle.wat's own `(load!)` calls on its deps (assuming candle.wat
becomes self-loading; minor cleanup during slice 4).

Even tighter if candle.wat's full dep chain loads transitively
from a single entry.

---

## Slices

1. **Slice 1** — `resolve_loads` relative-path resolution +
   canonical-path dedup. `Cargo.toml` `pathdiff` or stdlib
   `canonicalize`. Rust unit tests for relative / bare / cycle /
   dedup / escape cases.
2. **Slice 2** — `eval_kernel_run_sandboxed_ast` + 3 siblings
   inherit outer loader on `:None`. Shared helper. Unit tests.
3. **Slice 3** — `wat::test!` macro default loader widens to
   `CARGO_MANIFEST_DIR`. Macro emit change only. Existing tests
   unaffected (their explicit `loader:` overrides hold;
   implicit-default tests get wider scope, which doesn't break
   anything).
4. **Slice 4** — migrate `wat-tests/vocab/shared/time.wat` to
   use deftest with relative loads. Prove end-to-end. (Phase 3
   lab tests — scale_tracker, scaled_linear, rhythm — stay on
   their current pattern; dedicated migration when their arcs
   get revisited.)
5. **Slice 5** — INSCRIPTION + doc sweep (USER-GUIDE § 13,
   CONVENTIONS.md footer on relative paths, arc 018 INSCRIPTION
   footer pointer, lab FOUNDATION-CHANGELOG row).

## Tests

### Rust unit tests (resolve_loads layer)

- `relative_path_resolves_against_current_file`
- `bare_path_resolves_against_scope_root_unchanged`
- `parent_relative_traverses_up`
- `canonicalized_dedup_no_reparse`
- `diamond_dependency_loads_once`
- `cycle_still_detected_before_dedup`
- `escape_attempt_refused_by_scope_clamp`

### Rust unit tests (sandbox layer)

- `run_sandboxed_ast_inherits_outer_loader_on_none`
- `run_sandboxed_ast_explicit_scope_still_works`
- `run_sandboxed_ast_explicit_none_with_no_outer_loader_falls_back_inmemory`

### wat-level test

`wat-tests/holon/deftest-inherits-loader.wat` — a deftest that
`(load! "./something.wat")` and asserts the loaded define is
callable.

## Doc sweep

- `docs/USER-GUIDE.md` § 13 Testing — deftest's new inheritance
  semantic + relative path note.
- `docs/CONVENTIONS.md` — section on `(load!)` paths with the
  bare-vs-dotted distinction.
- `docs/arc/2026/04/007-wat-tests-wat/INSCRIPTION.md` +
  `arc/2026/04/018-opinionated-defaults-and-test-rename/INSCRIPTION.md`
  — footers pointing at 027's refinements.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  `:wat::load-file!` row may gain a note on relative paths.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — new row for arc 027.

## Non-goals

- CompositeLoader. Relative-path resolution kills the
  multi-root problem at the source. One ScopedLoader with a
  broader scope covers both trees.
- Bare-path migration. Existing `(load! "foo.wat")` callers
  keep working. Authors migrate to `(load! "./foo.wat")` when
  they want unambiguity; no forced sweep.
- Absolute paths (`/x.wat`). TS supports these via import maps;
  wat doesn't need them yet.
- Node-style bare-specifier resolution (looking up external
  packages). `:rust::*` is how wat reaches external crates; no
  parallel mechanism needed.

## Why this is inscription-class

The substrate gains a load-path semantic (relative) that was
missing. The semantic is the one every mature module system has
adopted (TypeScript, ES modules, Python relative imports,
Rust's super/self). deftest's "just works" behavior lands as a
consequence. Same shape as arcs 004's `reduce` (absence pointed
at real substrate work), arcs 019-026 (code-led, spec-follows
refinement).
