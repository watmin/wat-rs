# Arc 027 — deftest just works — BACKLOG

**Shape:** five slices, leaves-to-root. Status markers: ready /
obvious in shape / foggy.

---

## Slice 1 — relative-path resolver + canonical-path dedup

**Status: ready.**

Target: `src/load.rs`. Extend `resolve_loads` (and helpers) to
support file-relative paths and canonical-path dedup.

**Shape:**
- Add `current_file_dir: Option<PathBuf>` to the load-resolution
  stack frame. Root call has `None` (scope-root base only); each
  recursive load sets it to the resolved file's directory.
- Path-resolution branch:
  - starts with `./` or `../` → resolve relative to
    `current_file_dir`, then canonicalize.
  - bare → resolve via existing `ScopedLoader::fetch_source_file`
    against scope root (unchanged).
- Add `resolved: HashSet<PathBuf>` alongside the in-progress
  stack. Canonical path lookup on every resolution; hit →
  return Ok(empty form list) so the caller sees "success, no
  new forms."
- Cycle detection stack stays in place; fires before the dedup
  set check.

**Rust unit tests:**
- `relative_path_resolves_against_current_file`
- `parent_relative_traverses_up_through_scope`
- `bare_path_resolves_against_scope_root_unchanged`
- `canonicalized_dedup_second_call_is_noop`
- `diamond_dependency_resolves_D_once`
- `cycle_still_fires_LoadError_Cycle_before_dedup`
- `escape_attempt_refused_by_scope_clamp` — `./../../etc/passwd`

**Sub-fogs:**
- **1a — canonicalize behavior on non-existent paths.** `std::fs::canonicalize`
  errors if the target doesn't exist. For NotFound errors, we
  want the path in the error message. Likely solution: canonicalize
  the **parent dir** (which must exist) then append the basename;
  or detect NotFound separately and bubble up a clean error.
- **1b — canonicalization and symlinks.** `canonicalize` resolves
  symlinks. That's probably what we want — two symlinks to the
  same file dedup correctly. But it means a symlink farm can
  reach outside the scope clamp if the link target is outside.
  The scope clamp already does its own canonical comparison; no
  regression, but flag it in slice 1 tests.
- **1c — Windows path handling.** wat-rs is Unix-primary (arc 012
  fork substrate is Unix-only); canonical-path comparison on
  Windows has case-insensitive vs case-sensitive considerations.
  Out of scope for 027 — note in slice 5 INSCRIPTION.

## Slice 2 — :None scope inherits outer loader

**Status: obvious in shape** (once slice 1's tests establish the
load-path mechanics).

Target: `src/sandbox.rs`. Four primitives: `run_sandboxed`,
`run_sandboxed_ast`, `run_sandboxed_hermetic`,
`run_sandboxed_hermetic_ast`. Shared helper
`resolve_sandbox_loader(scope_opt, sym) -> Arc<dyn SourceLoader>`.

**Rust unit tests:**
- `run_sandboxed_ast_inherits_outer_loader_on_none`
- `run_sandboxed_ast_explicit_scope_still_works`
- `run_sandboxed_ast_none_with_no_outer_loader_falls_back_inmemory`

**Sub-fogs:**
- **2a — fork-isolated siblings.** `run_sandboxed_hermetic_ast`
  forks a child process. The outer loader is an `Arc<dyn
  SourceLoader>`. `ScopedLoader` holds a canonical path string
  + no per-process state — COW inheritance via fork should work
  cleanly. Verify in test.

## Slice 3 — wat::test! default loader widens

**Status: obvious in shape** (once slices 1 + 2 land).

Target: `wat-macros/src/lib.rs`. `wat::test!` macro default
computation: when no explicit `loader:` AND no explicit `path:`,
emit a `ScopedLoader` rooted at `env!("CARGO_MANIFEST_DIR")`
instead of `"wat-tests"`.

When `path:` is explicit but `loader:` is not — still widen to
CARGO_MANIFEST_DIR (test discovery root differs from filesystem
root; tests still need to reach sibling trees).

**Sub-fogs:**
- **3a — does wat-rs's own tests/test.rs still work?** Yes —
  wat-rs stdlib is baked; tests don't rely on filesystem loader.
  Widening the scope doesn't break anything; it's just unused
  reach. Verify by running wat-rs's own `cargo test`.
- **3b — does the consumer template (examples/with-lru/,
  examples/with-loader/) still work?** These have their own
  `wat/` trees. Widening to CARGO_MANIFEST_DIR means their tests
  CAN reach `wat/` siblings via relative paths — which is the
  new capability. Existing tests don't USE it, so no regression.

## Slice 4 — migrate lab Phase 2 tests to deftest

**Status: obvious in shape** (once slices 1-3 land, lab picks up
arc 027 changes via its wat-rs Cargo dep).

Target: `wat-tests/vocab/shared/time.wat` (lab repo). Rewrite
each of the 6 tests to use `:wat::test::deftest` directly with
relative-path `(load!)` calls:

```scheme
(:wat::test::deftest :trading::test::vocab::shared::time::test-foo 1024 :error
  (:wat::load-file! "../../wat/types/candle.wat")
  (:wat::load-file! "../../wat/vocab/shared/time.wat")
  ...)
```

42 boilerplate lines → ~12 per test. If candle.wat's dep-chain
transitively loads its own deps, even fewer per test.

**Sub-fogs:**
- **4a — candle.wat's dep chain.** Does types/candle.wat already
  `(load!)` its deps (enums, newtypes, ohlcv, distances, pivot)?
  If not, each test needs to load them explicitly OR candle.wat
  needs a minor refactor to self-load. Discovery: check at
  slice 4 start.
- **4b — path depth from wat-tests/vocab/shared/*.wat to wat/.**
  Three levels up: `../../../` then into `wat/`. Verify the
  escape-clamp doesn't refuse.

## Slice 5 — INSCRIPTION + doc sweep

**Status: obvious in shape.**

- `docs/arc/2026/04/027-deftest-inherits-loader/INSCRIPTION.md`.
- `docs/USER-GUIDE.md` § 13 Testing — deftest inheritance + `./`
  relative path note.
- `docs/CONVENTIONS.md` — `(load!)` path section.
- arc 007 + arc 018 INSCRIPTION footers pointing at arc 027.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`.

---

## Working notes (updated as slices land)

- Opened 2026-04-23.
- Initial scope = :None inheritance only.
- **Expanded scope 2026-04-23** after builder's TypeScript-stance
  direction: *"i think we take a typescript stance and do
  ./wat/.... relative paths to where its being expressed... we
  can... have a resolver to full path and ignore reloading a
  file we've already loaded."* Arc now covers relative-path
  resolver, canonical-path dedup, :None inheritance, wat::test!
  scope widening, and lab test migration. Five slices total.
- **Slice 1 narrowed during study.** Reading `load.rs:893-924`
  (`resolve_within_scope`) showed that file-relative path
  resolution **already ships** — when `base_canonical = Some(...)`,
  paths resolve against the importing file's directory; when
  `None`, against the scope root. So `(load! "./foo.wat")` and
  `(load! "foo.wat")` from inside `wat/main.wat` both already
  resolve to `wat/foo.wat`. The TypeScript-style honest-prefix
  `./` is a NOTATION choice for the author (explicit about
  intent), not a mechanism change. The real change from builder's
  direction is **dedup**: today duplicate loads fire
  `LoadError::DuplicateLoad` (test `duplicate_load_halts` at
  `load.rs:1358` enforces it); builder's ask is "ignore
  reloading" — silent no-op instead of error. That's the honest
  substrate change.
- **Slice 1 shape:** flip `DuplicateLoad` error to silent no-op.
  Cycle detection stays in place (fires before dedup). Migrate
  the existing `duplicate_load_halts` test to
  `diamond_dependency_deduplicates` asserting the new no-op
  behavior. Add test for the `./` notation working identically
  to bare (documentation-worthy).
- **Slice 2 scope narrowed during write.** The BACKLOG named four
  primitives to update (`run_sandboxed`, `run_sandboxed_ast`,
  `run_sandboxed_hermetic`, `run_sandboxed_hermetic_ast`). Arc 012
  had already retired the two hermetic primitives from Rust — they
  moved to wat stdlib (`wat/std/hermetic.wat`) as wat functions
  on top of `:wat::kernel::fork-with-forms`. So only TWO active
  Rust primitives remained to update. The fork-child loader
  inheritance sub-fog (2a) dissolved — COW inheritance of the
  parent's frozen SymbolTable already carries the loader across
  fork, same mechanism `install_dep_sources` rides (arc 015 slice
  3a). Shipped: shared helper `resolve_sandbox_loader(scope_opt,
  sym, op)` in `src/sandbox.rs`; both primitives collapsed their
  inline loader-building to a one-line call. Three unit tests in
  `#[cfg(test)] mod tests`: explicit-scope builds ScopedLoader;
  `:None` with outer attached clones the same Arc (pointer
  identity via `Arc::ptr_eq`); `:None` with no outer falls back
  to `InMemoryLoader`. 566 → 569 lib tests; zero regressions
  across the workspace.
- **Slice 3 shipped clean.** `wat::test!` macro's implicit loader
  default widened from `"wat-tests"` (=CARGO_MANIFEST_DIR/wat-tests)
  to `CARGO_MANIFEST_DIR` (crate root). The explicit `loader:
  "<subpath>"` override is unchanged — still resolves to
  `CARGO_MANIFEST_DIR/<subpath>`. Macro restructured: the previous
  None/Some(loader) branching on `effective_loader` collapsed —
  every expansion now emits `run_and_assert_with_loader` with a
  concrete `ScopedLoader`. Sub-fogs 3a + 3b resolved at build —
  wat-rs's own `tests/test.rs` still green (baked stdlib has no
  filesystem reach; wider scope doesn't break anything);
  `examples/with-lru/` and `examples/with-loader/` smoke tests
  still pass (their explicit-loader sites hold). Every integration
  suite across the workspace green. The `__wat_loader_root` being
  `env!("CARGO_MANIFEST_DIR")` (not `concat!(..., "/", ...)`) is
  why — the concat form would produce a trailing-slash path that
  ScopedLoader would reject; the bare env form is the path itself.
- **Slice 4 broadened: deftest gained a `prelude` parameter.**
  Initial migration attempt kept the manual `run-sandboxed-ast` +
  `:wat::test::program` pattern because two constraints seemed to
  block `deftest`: (a) the macro puts its body inside `:user::main`
  which runs at RUNTIME, so `(:wat::load-file! ...)` — a startup
  form — couldn't live in the body; (b) AST-entry sandboxes have
  no source-file context, so `./` and `../` relative paths would
  walk out of scope.
  Builder correction: *"we do not do deferral — we fix the thing
  when we find it broken... the expression (:wat::test::deftest)
  should just work for consumers as it does in the wat-rs tests."*
  The cleaner path WAS to extend the macro.
  Shipped: `:wat::test::deftest` grew from 4-arg to 5-arg —
  `(name dims mode prelude body)`. The new `prelude` param is a
  list AST that splices via `,@prelude` BEFORE the auto-generated
  `:user::main` define. Empty `()` prelude = the minimal shape;
  list-of-load-forms = tests that compose external modules. Plus
  `:wat::test::deftest-hermetic` sibling with identical signature
  routing through `:wat::kernel::run-sandboxed-hermetic-ast` (fork
  isolation for tests that spawn driver threads — Console, Cache).
  Migration: 52 existing `deftest` callers across
  `wat-rs/wat-tests/`, 6 across `crates/wat-lru/wat-tests/` + one
  example, 1 in lab's `test_scaffold.wat` — all gained an `()`
  line as the new 4th arg. Script-driven (Python, not Perl — the
  `|` alternation crash neighborhood from Chapter 32 stayed
  avoided).
- **Slice 4 bonus: types self-load their deps.** Builder pressed
  on the 7-load prelude: *"how many of those load! are redundant...
  does candle pull in all its deps for us?"* Three loads were
  genuinely redundant (enums, newtypes, distances — time tests
  don't transitively use them). The other four were deps that
  should have been auto-loaded through the type hierarchy. Added
  `./` relative `(load!)` lines to `wat/types/candle.wat` (pulls
  ohlcv + pivot), `wat/types/distances.wat` (pulls newtypes), and
  `wat/vocab/shared/time.wat` (pulls candle). Canonical-path dedup
  (slice 1) makes the explicit loads in `wat/main.wat` no-op on
  repeat. Lab tests collapsed from a 7-load prelude to a 1-load
  prelude — just `"wat/vocab/shared/time.wat"` — everything else
  transitive. 25/25 lab tests green (including all 6 time tests
  at ~9ms each). The ergonomic win lands honestly.
- **Slice 4 scope shifted during migration.** The DESIGN expected
  `deftest` + `../../wat/...` relative-path loads. Two findings
  collapsed that target down to a smaller-but-honest shape:
  1. The `deftest` macro puts its body inside `:user::main`, which
     runs at RUNTIME. `(:wat::load-file! ...)` is a STARTUP-time
     form refused at eval (`EvalForbidsMutationForm`). So `deftest`
     bodies literally cannot carry loads without a macro refactor
     that would hoist them above the `:user::main` define — bigger
     than slice 4.
  2. AST-entry sandboxes have NO source-file context (the forms are
     a `Vec<WatAST>` constructed programmatically, not parsed from
     disk). Relative paths with `./` or `../` inside the sandboxed
     program's `(load!)` calls resolve against the loader's scope
     ROOT (arc 017's "no caller base" branch), not against the
     outer test file's location. So `../../wat/types/enums.wat`
     resolves to `CARGO_MANIFEST_DIR/../../wat/types/enums.wat` —
     which walks OUT of the scope and is refused.
  Slice 4 landed as the honest alternative: **keep the
  `run-sandboxed-ast` + `:wat::test::program` + explicit load
  lines shape; just migrate the paths from scope-relative-under-
  "wat"** (`"types/enums.wat"` with `(Some "wat")` scope) **to
  scope-relative-under-CARGO_MANIFEST_DIR** (`"wat/types/enums.wat"`
  with `:None` scope — inheriting the test binary's loader via
  slice 2). Same 7 load lines per test, but now rooted at the
  widened scope slice 3 provides. Tests all green — 25/25 lab
  tests including the 6 migrated ones at ~9ms each (~54ms total;
  dedup means 6 × 7 = 42 load-file calls parse just 7 unique
  files).
  Future work (outside arc 027): either extend `deftest` to accept
  a load-prelude separate from the runtime body, OR thread a
  synthetic source-file context through AST-entry sandboxes so
  relative-path resolution works from a caller-supplied anchor.
  Either would enable the DESIGN's minimalist shape. Neither was
  needed to prove the end-to-end loader-inheritance path.
