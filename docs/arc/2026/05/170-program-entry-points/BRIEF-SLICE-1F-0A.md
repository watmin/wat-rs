# Arc 170 slice 1f-0a — BRIEF

**Foundation crack fix; sonnet.** Migrate the wat-side
`:wat::test::deftest` + `:wat::test::deftest-hermetic` macros in
`wat/test.wat` to emit the new `:user::main () -> :wat::core::nil`
signature (per slice 1e's lock-in). The current macros emit the
retired four-arg shape, which `validate_user_main_signature` in
`src/freeze.rs` rejects; every deftest in the workspace fails
with this rot, accounting for the 855-failure baseline.

## Mission

Two macros to edit in `wat/test.wat`:

### 1. `:wat::test::deftest` (around line 305)

**Before** (current, retired shape):

```
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define
           (:user::main
             (stdin  :wat::io::IOReader)
             (stdout :wat::io::IOWriter)
             (stderr :wat::io::IOWriter)
             -> :wat::core::nil)
           ,body))
       (:wat::core::Vector :wat::core::String)
       :wat::core::None)))
```

**After** (slice-1e-aligned):

```
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           ,body))
       (:wat::core::Vector :wat::core::String)
       :wat::core::None)))
```

The only change: replace the three stdio parameters with an
empty arg list. Slice 1e's ambient runtime exposes stdio via
`:wat::kernel::println` / `:wat::kernel::eprintln` /
`:wat::kernel::readln` (when those primitives' channels are
populated; for slice 1f-0a's scope, that's slice 1f-α's
`tests/wat_arc170_slice_1f_alpha_helpers.rs` test fixture
problem — not this slice's problem).

### 2. `:wat::test::deftest-hermetic` (around line 336)

Same change pattern. Replace the three-stdio-param shape with
empty arg list:

**After:**

```
(:wat::core::defmacro
  (:wat::test::deftest-hermetic
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-hermetic-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           ,body))
       (:wat::core::Vector :wat::core::String)
       :wat::core::None)))
```

### Alias factories — no edit needed

`:wat::test::make-deftest` (around line 388) and
`:wat::test::make-deftest-hermetic` (around line 405) emit
`:wat::test::deftest <test-name> ,,default-prelude ,body`
internally. They inherit the new shape automatically from the
two base-macro edits above.

## What to NOT do

- **No `examples/*` migration.** The examples
  (`examples/with-loader/wat/main.wat`,
  `examples/interrogate/wat/main.wat`, etc.) still carry the
  retired 4-arg `:user::main` shape. They are independently
  broken — surface this if you observe it, but DO NOT migrate
  them in this slice. Per user direction "we fix what we break
  once the idealized shape is realized" — examples migrate in
  their own slice/arc.
- **No wat-tests/* migration.** Test files (`wat-tests/...`)
  contain deftest calls; those expand through the macro and
  inherit the new shape automatically. No deftest test-file
  body should need editing.
- **No substrate (Rust) changes.** This slice is wat-source-only
  (`wat/test.wat`).
- **No new comments / explanations / docstrings.** Replace the
  emitted forms verbatim per the before/after above. The
  surrounding docstrings + comment blocks for each macro stay
  as-is.

## Substrate-grep citations (verify before committing)

- `wat/test.wat:305-329` — current `:wat::test::deftest`
  defmacro body
- `wat/test.wat:336-354` — current `:wat::test::deftest-hermetic`
  defmacro body
- `src/freeze.rs:703` — `USER_MAIN_PATH` constant
- `src/freeze.rs:717` — `invoke_user_main` (post-slice-1e:
  takes `&world`, `Vec::new()` for args; no stdio param
  passing)
- `src/freeze.rs::validate_user_main_signature` — the validator
  this slice is making the macro happy with

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — `:wat::test::deftest` emits new shape | sed/grep on `wat/test.wat` finds `(:user::main -> :wat::core::nil)` inside the deftest body; the old 3-param shape is GONE | ✓ |
| B — `:wat::test::deftest-hermetic` emits new shape | same for deftest-hermetic | ✓ |
| C — `cargo check --release` green | no compile errors | ✓ |
| D — Workspace fail count drops dramatically | `cargo test --release --workspace --no-fail-fast 2>&1 \| grep 'test result' \| awk '{...}'` shows fail count drop from 855 to near zero (≤ 50; ideally 0) | ✓ |
| E — Pass count rises correspondingly | the 855 previously-failing deftests now pass; ~+850 to pass count expected | ✓ |
| F — No tests break that were previously passing | tests in the 1327-passing baseline don't regress | ✓ |
| G — slice 1f-α tests now actually run | `tests/wat_arc170_slice_1f_alpha_helpers.rs` Rust-level tests stay green (10/10); they don't use deftest so they were already running, but verify | ✓ |
| H — Zero new dependencies | Cargo.toml unchanged | ✓ |
| I — No other wat-source changes | `git diff --stat` shows only `wat/test.wat` modified | ✓ |
| J — Honest deltas surfaced | per FM 5; if any decision required scope expansion, surface — don't work around | ✓ |

## Honest delta categories — surface, don't work-around

- **If `wat/test.wat`'s deftest macros invoke OTHER macros or
  helpers that themselves depend on the old `:user::main` arity**
  (e.g., a wrapper that constructs the IOReader/IOWriter args
  somewhere outside the macro body), surface — there may be
  more than the two macro-body edits.
- **If `validate_user_main_signature` actually accepts BOTH
  the old and new shape** (slice 1e was permissive at the call
  site), the 855-failure baseline might be from something else.
  Verify post-edit that fail count actually drops.
- **If migrating reveals that `wat-tests/*` files have manual
  `(:user::main (stdin) (stdout) (stderr) ...)` definitions**
  (NOT going through the macro), those would need migration too.
  Surface as honest delta — likely small (few files) or zero
  (most tests go through the macro).
- **`:wat::test::TestResult` type** — if this type's definition
  is also affected by the slice-1e signature change (unlikely,
  but possible), surface.

## Predicted runtime

15-30 min sonnet. Two textual edits + cargo test re-run +
honest-delta sweep.

**Hard cap:** 60 min (1 hour). Wakeup scheduled.

## Reference

- DESIGN.md (passes 1-18)
- REALIZATIONS-SLICE-1.md § Pass 10 (`:wat::core::nil` IS the
  exit code; the canonical `:user::main` signature) + § Pass
  18 (this slice's diagnostic comes from)
- BUILD-PLAN.md § Slice 1f-0a (the spec this slice fulfills)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1E.md`
  — slice 1e's calibration record (slice 1e shipped the
  validator; this slice fixes its remaining unsweeped consumer)
- `wat/test.wat` — the file to edit
- `src/freeze.rs:703-740` — the substrate's canonical
  user::main shape + validator
