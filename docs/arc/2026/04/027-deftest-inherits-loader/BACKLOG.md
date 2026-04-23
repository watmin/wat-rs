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
  (:wat::core::load! :wat::load::file-path "../../wat/types/candle.wat")
  (:wat::core::load! :wat::load::file-path "../../wat/vocab/shared/time.wat")
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
