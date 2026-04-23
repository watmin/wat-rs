# Arc 027 — deftest just works — INSCRIPTION

**Shipped:** 2026-04-23. Four code slices (DESIGN scoped five but
slice 5 is the INSCRIPTION itself, this file).

**Commits:**
- `d2b070e` — DESIGN + BACKLOG + slice 1 (canonical-path dedup)
- `865513f` — slice 2 (`:None` sandbox scope inherits outer loader)
- `c1a2240` — slice 3 (`wat::test!` default loader widens to
  `CARGO_MANIFEST_DIR`)
- `ecfc160` — slice 4 (deftest gains `prelude` param + hermetic
  sibling; wat-rs callsite migration)
- `47b2bd8` (lab) — lab migration: deftest + self-loading types
  collapse per-test prelude
- (this commit) — slice 5 (INSCRIPTION + doc sweep)

---

## What shipped

### Slice 1 — canonical-path dedup on `(load!)` (d2b070e)

The originally-planned relative-path resolver sub-fog (1a)
dissolved during study: reading `load.rs:893-924`
(`resolve_within_scope`) showed file-relative resolution already
shipped — `base_canonical = Some(...)` resolves against the
importing file's directory. So `(load! "./foo.wat")` and
`(load! "foo.wat")` from inside the same parent already produce
the same target. The TypeScript-style `./` is a notation
choice, not a mechanism change.

The REAL change landing in slice 1: **flip `LoadError::DuplicateLoad`
to silent no-op.** Cycle detection stays in place (fires before
dedup). The existing `duplicate_load_halts` test migrated to
`diamond_dependency_deduplicates` asserting the new behavior —
when two different files both `(load!)` a common helper, the
helper parses once, registers once, and the second load
returns without error.

This is what makes the slice 4 ergonomic story possible: a
test's manual `(load!)` list can include a path that the file's
own `./`-relative transitive load has already pulled in, and
the substrate silently collapses rather than halting.

### Slice 2 — `:None` sandbox scope inherits outer loader (865513f)

`:wat::kernel::run-sandboxed-ast` gained loader-inheritance
semantics. Pre-027: `:None` scope meant "fresh InMemoryLoader"
— sandboxed programs with no explicit scope had no filesystem
reach at all, even when the enclosing test binary had one. Post-
027: `:None` scope clones the caller's active loader (if any)
so sandboxed `(load!)` calls reach the same roots the test
harness reached. Explicit `scope :"<path>"` still builds a
fresh `ScopedLoader` clamped to that path; the inheritance
only kicks in for the `None` case.

Scope narrowed during write: the BACKLOG listed four primitives
(`run_sandboxed`, `run_sandboxed_ast`, `run_sandboxed_hermetic`,
`run_sandboxed_hermetic_ast`). Arc 012 had already retired the
two hermetic Rust primitives — they moved to wat stdlib
(`wat/std/hermetic.wat`) as wat functions over
`:wat::kernel::fork-with-forms`. So only two active Rust
primitives needed updates. Sub-fog 2a (fork-child loader
inheritance) dissolved — COW of the parent's frozen SymbolTable
already carries the loader across fork, the same mechanism
`install_dep_sources` rides (arc 015 slice 3a).

Shipped: shared helper `resolve_sandbox_loader(scope_opt, sym, op)`
in `src/sandbox.rs`. Both primitives collapsed their inline
loader-building to a one-line call. Three unit tests in the
mod-test block — explicit-scope builds ScopedLoader; `:None`
with outer attached clones the same Arc (pointer identity via
`Arc::ptr_eq`); `:None` with no outer falls back to
`InMemoryLoader`.

### Slice 3 — `wat::test!` default loader widens (c1a2240)

The `wat::test!` macro's implicit loader widened from
`"wat-tests"` (= `CARGO_MANIFEST_DIR/wat-tests`) to
`CARGO_MANIFEST_DIR` (crate root). Explicit `loader: "<subpath>"`
override unchanged — still resolves to
`CARGO_MANIFEST_DIR/<subpath>`.

Macro restructured: the previous None/Some(loader) branching on
`effective_loader` collapsed. Every expansion now emits
`run_and_assert_with_loader` with a concrete `ScopedLoader`.

Why: the lab (and any downstream consumer) keeps its wat source
tree rooted at the crate — `wat/`, `wat-tests/`, etc. A test
that wants to `(load!)` a vocab module from its `wat/` sibling
needs to reach up from `wat-tests/` to the crate root. Widening
the default to the crate root means `(load! "wat/vocab/.../X.wat")`
works from any `wat-tests/` subtree without per-project
configuration.

### Slice 4 — deftest gains prelude + hermetic sibling (ecfc160)

The builder pressure landed here: *"we do not do deferral — we
fix the thing when we find it broken... `(:wat::test::deftest)`
should just work for consumers as it does in the wat-rs tests."*

Initial attempt tried to keep the manual `run-sandboxed-ast` +
`:wat::test::program` + explicit load lines shape because two
constraints seemed to block adding loads to `deftest`:
- `:user::main` runs at RUNTIME; `(:wat::load-file!)` is a
  startup-time form refused at eval.
- AST-entry sandboxes have no source-file context for relative
  paths.

The cleaner path was extending the macro to carry a prelude.

Shipped:
- `:wat::test::deftest` gained a `prelude` parameter. Signature
  went from `(name dims mode body)` to `(name dims mode prelude
  body)` (later flipped to `(name mode dims prelude body)` by
  arc 030's arg-order fix; later dropped to `(name prelude body)`
  by arc 031's config-inheritance move). The `prelude` is a list
  AST that splices via `,@prelude` BEFORE the auto-generated
  `:user::main` define. Empty `()` prelude = no startup forms.
- `:wat::test::deftest-hermetic` sibling with identical signature
  routing through `:wat::kernel::run-sandboxed-hermetic-ast`
  (fork isolation for tests that spawn driver threads — Console,
  Cache).

Callsite migration: 52 existing deftest callers across
`wat-rs/wat-tests/`, 6 across `crates/wat-lru/wat-tests/` + one
example, 1 in lab's `test_scaffold.wat`. All gained an `()` line
as the new 4th arg. Script-driven (Python, not Perl — the `|`
alternation crash neighborhood from Chapter 32 stayed avoided).

### Slice 4 bonus — types self-load their deps

Builder pressed on the 7-load prelude the lab tests initially
carried: *"how many of those load! are redundant... does candle
pull in all its deps for us?"*

Three loads were genuinely redundant (enums, newtypes, distances
— time tests don't transitively use them). The other four were
deps that should have been auto-loaded through the type
hierarchy.

Shipped: added `./`-relative `(load!)` lines to
`wat/types/candle.wat` (pulls ohlcv + pivot),
`wat/types/distances.wat` (pulls newtypes), and
`wat/vocab/shared/time.wat` (pulls candle). Canonical-path dedup
(slice 1) makes any explicit loads in `wat/main.wat` no-op on
repeat.

Lab tests collapsed from 7-load prelude → 1-load prelude —
`"wat/vocab/shared/time.wat"` pulls the entire dependency chain
transitively. 25/25 lab tests green (6 time tests at ~9ms each;
~54ms total with 7 unique files parsed across 42 load calls
thanks to dedup).

### Slice 4 scope landed pragmatically

The DESIGN expected minimalist `deftest :name body` shape with
`../../wat/...` relative-path loads inside the body. Two
findings shifted the landing:
1. `:user::main` runs at runtime; loads are startup-time.
2. AST-entry sandboxes have no source-file context; `../`
   relative paths walk out of scope.

Slice 4 landed the pragmatic shape: keep the per-test explicit
load list, but migrate paths to `"wat/..."` under the
CARGO_MANIFEST_DIR-widened scope (slice 3). The minimalist
shape returned with arc 029's `make-deftest` factory + arc 031's
config-inheritance, both of which built on slice 4's
`prelude` parameter.

### Lab migration (47b2bd8)

`holon-lab-trading/wat-tests/` tests migrated to the deftest +
self-loading-types shape. `vocab/shared/time.wat` landed as the
shape-test; `test_scaffold.wat` got its own `()` prelude.

---

## Tests

Slice 1 reshaped existing `duplicate_load_halts` into
`diamond_dependency_deduplicates`; slice 2 added three
`resolve_sandbox_loader` unit tests; slice 3 tested the widened
macro scope via existing workspace + example smoke tests;
slice 4 tested the new `prelude` parameter via every migrated
callsite (52 + 6 + 1 + 25 lab = 84 test invocations exercising
the parameter).

Workspace-wide: zero regressions across any integration suite.
Lab: 25/25 wat-tests green.

---

## Sub-fog resolutions

**1a — canonicalize on non-existent paths.** Moot; slice 1
became dedup-only. File-relative resolution ships in
`resolve_within_scope` pre-027.

**1b — canonicalization and symlinks.** Verified: existing
scope-clamp canonical comparison handles symlinks correctly.
Flag-only in this INSCRIPTION.

**1c — Windows path handling.** Out of scope for 027. wat-rs is
Unix-primary (arc 012 fork substrate). Windows canonical-path
semantics (case-insensitive) are a future-arc concern if and
when Windows becomes a target.

**2a — fork-child loader inheritance.** Resolved: COW. The fork
substrate from arc 012 copies the parent's frozen SymbolTable
into the child via `libc::fork()`; the loader Arc rides along
without any explicit passing. Same mechanism as
`install_dep_sources`.

**3a + 3b — default loader scope.** Resolved at build. `wat-rs`'s
own `tests/test.rs` green (baked stdlib has no filesystem reach;
wider scope doesn't break anything). `examples/with-lru/` +
`examples/with-loader/` smoke tests green (their explicit-loader
sites hold). The `__wat_loader_root` as `env!("CARGO_MANIFEST_DIR")`
(not `concat!(..., "/", ...)`) avoided a trailing-slash path
that ScopedLoader would have rejected.

---

## What did NOT change

- **Cycle detection.** Remains intact, fires before dedup.
  Slice 1 preserves it — a genuine cycle still surfaces as
  `LoadError::Cycle`, not a silent no-op.
- **Scope clamp.** Every `(load!)` still verifies the canonical
  resolved path starts with the scope root. Escape attempts
  (`./../../etc/passwd`) still refused.
- **Reserved-prefix gate on loaded files.** Unchanged — loaded
  files cannot define `:wat::*` or `:rust::*` paths.

---

## What comes next

Arc 029 (nested quasiquote) + arc 031 (sandbox inherits Config)
together close the ergonomic story. Arc 027's scope-inheritance
for the loader is the template arc 031 followed for Config —
same move, different environment field.

Arc 027's closing inscription was written from future knowledge
— arcs 028, 029, 030, 031 shipped between the slice-4 code and
this closing doc. The wider scope now visible (multiple
scope-inheritance arcs landing in sequence, each closing one
environment field) is part of what made this INSCRIPTION read
cleanly. Each arc adds one more shape to the substrate; the
disposable-machine pattern holds.
