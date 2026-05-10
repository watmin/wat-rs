# Arc 170 slice 1e — BRIEF

**Substrate; opus.** Foundation slice for the substrate-side
architectural pivot per REALIZATIONS pass 7 (ambient runtime) +
pass 10 (nil IS the exit code). After this slice, `:user::main`
is `[] -> :wat::core::nil`; `:wat::runtime::argv` is ambient;
`:wat::kernel::ExitCode` retires.

**Reference docs (read first):**
- [`DESIGN.md`](./DESIGN.md) §1 (canonical form), §2 (signature
  update), §5 (settled-design re ExitCode retirement)
- [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) passes
  7 + 10 — the user direction that locked these in
- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §3 slice 1e — scope,
  ship criteria, expected workspace impact

**Branch:** `arc-170-program-entry-points` (foundation commit
`eb655d1` is your starting point).

**Constraint:** STOP if any substrate primitive this BRIEF
references doesn't exist — DON'T workaround. Surface as honest
delta.

## Scope

### 1. Mint `:wat::runtime::argv` ambient value

The wat-cli plumbs `std::env::args()` into a process-wide
ambient value. Wat code reads it via `(:wat::runtime::argv)` →
`:wat::core::Vector<wat::core::String>`.

**Substrate work:**
- New static in `src/runtime.rs`: `ARGV: OnceLock<Arc<Vec<String>>>`
  (set-once; populated by wat-cli before main invocation)
- `pub fn set_argv(argv: Vec<String>)` — initializer; called by
  wat-cli before `invoke_user_main`
- New eval arm: `:wat::runtime::argv` (nullary; returns
  `Value::Vec(...)` wrapping the argv strings)
- New type-check arm: returns
  `:wat::core::Vector<wat::core::String>`

### 2. Mint `:wat::runtime::current-thread` ambient value

Returns the calling thread's id. Slice 1g will populate
thread-locals that this primitive reads from; for slice 1e,
implement against the main thread only (slice 1g extends to
spawned threads).

**Substrate work:**
- New eval arm: `:wat::runtime::current-thread` (nullary;
  returns thread id as `:wat::core::String` or whatever
  representation makes most sense — settle in implementation)
- New type-check arm

### 3. Update `:user::main` signature: `[] -> :wat::core::nil`

`src/freeze.rs:753` (`expected_user_main_signature`) currently
returns 4 params + `:wat::kernel::ExitCode` return. Update to:

```rust
pub fn expected_user_main_signature() -> (Vec<TypeExpr>, TypeExpr) {
    let params = vec![];  // empty — argv is ambient
    let ret = TypeExpr::Path(":wat::core::nil".into());
    (params, ret)
}
```

Then `validate_user_main_signature` (`src/freeze.rs:771`) checks
the new shape. Update the diagnostic messages:
- "param count" diagnostic: name the new `[] -> :wat::core::nil`
  shape; no mention of stdio/argv params (ambient now)
- "return type" diagnostic: `:wat::core::nil` (not ExitCode)
- The doc-comment block above `expected_user_main_signature` —
  rewrite for the new contract; cite arc 170 REALIZATIONS pass
  10 as the source

### 4. Retire `:wat::kernel::ExitCode` typealias

- Delete `wat/kernel/exit-code.wat` (the typealias definition)
- Verify no remaining substrate references to ExitCode
  (grep `src/` + `crates/` + `wat/`; expect zero hits post-edit)
- Test fixtures referencing `:wat::kernel::ExitCode` will break
  — that's expected substrate-as-teacher input for slice 3 sweep

### 5. Update `invoke_user_main` to take no args

`src/freeze.rs:716` currently accepts `args: Vec<Value>`.
Update to:

```rust
pub fn invoke_user_main(frozen: &FrozenWorld) -> Result<Value, RuntimeError> {
    let main_func = frozen.symbols().get(USER_MAIN_PATH)
        .ok_or(RuntimeError::UserMainMissing)?.clone();
    apply_function(main_func, vec![], frozen.symbols(),
                   crate::rust_caller_span!())
}
```

Or keep the `args` parameter as `Vec::new()` per call site;
either is fine. Update internal test cases at `src/freeze.rs:1183+`
that pass test args to `invoke_user_main` — those tests need
revisiting OR retirement (depends on what they're checking).

### 6. Update wat-cli (`crates/wat-cli/src/lib.rs`)

- Read `std::env::args()` (already at line 257); call
  `runtime::set_argv(argv)` BEFORE `invoke_user_main`
- Don't construct `main_args` (4-element Vec<Value> with
  IOReader/IOWriter/IOWriter/Vec<String>); call
  `invoke_user_main(&world)` with no args
- Map nil-return → `ExitCode::from(0)`; panic propagation
  already handles non-zero (StdErrService cascade lands in
  slice 1i; for slice 1e, panic still goes through whatever
  arc-113 cascade exists today)

The `args.get(0)` / `args.get(1)` / `args.get(2)` IOReader /
IOWriter / IOWriter constructions retire too — wat-cli no
longer hands stdio to user code via params. Stdio access in
slice 1e is via current substrate IO primitives;
StdInService / StdOutService / StdErrService land in slice 1f.

### 7. Update spawn-process child invocation (`src/spawn_process.rs`)

The child fn is now `[] -> :wat::core::nil` (no stdio params).
Update the child invocation path to:
- Not pass stdio Values to the child fn
- Expect nil-return from child
- Slice 1f's services boot replaces today's stdio-passing path
  entirely; slice 1e is the transitional state

### 8. Walker `BareLegacyMainSignature` updates

Today's walker fires on the legacy 3-arg shape (pre-slice-2) +
the 4-arg ExitCode shape (post-slice-2). After slice 1e, the
NEW legacy is anything that's not `[] -> :wat::core::nil`. The
walker's diagnostic should name the new shape and cite
REALIZATIONS pass 10 as the rationale.

`src/check.rs` walker variants — find `BareLegacyMainSignature`,
update its body + Display + Diagnostic to fire on
not-`[] -> :wat::core::nil` shapes; the diagnostic names the
new canonical form.

### 9. Test fixture for the new shape

Add a load-bearing test at
`tests/wat_arc170_slice_1e_user_main_nil.rs`:

```rust
//! Verifies the post-slice-1e :user::main signature:
//!   :user::main [] -> :wat::core::nil
//!
//! - parses + freezes
//! - invoke_user_main returns Value::Unit (nil)
//! - :wat::runtime::argv is accessible from main's body
```

Three test cases minimum:
- `:user::main` with `[] -> :wat::core::nil` parses + freezes
  + invokes; returns Value::Unit
- `:user::main` with `[] -> :wat::core::i64` (wrong return)
  fails freeze with diagnostic naming the new shape
- `:user::main` body calls `(:wat::runtime::argv)`; returns the
  Vec<String> set via `runtime::set_argv` before the test
  invokes main

## Constraints

- **Don't write a workaround.** If
  `:wat::runtime::argv` can't be wired in via the existing eval
  framework (some substrate gap I missed), STOP and report —
  don't ship a partial fix.
- **Don't sweep tests.** This slice is substrate-only.
  Substrate-as-teacher walker firings will surface across the
  workspace; that's expected input for revised slice 3. Don't
  touch tests outside the new `tests/wat_arc170_slice_1e_*.rs`
  fixture file.
- **Don't mint StdInService / StdOutService / StdErrService.**
  Those land in slice 1f. Slice 1e leaves the existing stdio
  path operational so the workspace stays partially testable.
- **Don't update USER-GUIDE / INSCRIPTION.** Those are slice 5
  paperwork.
- **No cross-arc work.** Stay inside arc 170's directory + the
  files this BRIEF lists. Don't touch unrelated arcs.

## Substrate-grep citations

Every primitive this BRIEF references, verified to exist:

- `expected_user_main_signature` — `src/freeze.rs:753`
- `validate_user_main_signature` — `src/freeze.rs:771`
- `invoke_user_main` — `src/freeze.rs:716`
- `wat/kernel/exit-code.wat` — exists (file currently defines
  `(typealias :wat::kernel::ExitCode :wat::core::u8)`)
- `KERNEL_STOPPED` / `KERNEL_SIGUSR1` etc. — `src/runtime.rs:51-119`
  (model of static atomics + setters; mirror for argv)
- wat-cli main invocation site — `crates/wat-cli/src/lib.rs:257`+
  (argv collection + main_args construction)
- spawn-process child invocation — `src/spawn_process.rs`
  (slice 2 added the fn-input shape; slice 1e simplifies the
  args)
- `BareLegacyMainSignature` walker — `src/check.rs`
  (find via grep; slice 2 added it)
- TypeExpr::Path / TypeExpr::Parametric — used throughout
  `src/freeze.rs:753-816`

Any deviation from these locations: STOP, report, don't
guess.

## Ship criteria

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `:wat::runtime::argv` mints | (:wat::runtime::argv) returns the Vec set via runtime::set_argv | ✓ |
| B — `:wat::runtime::current-thread` mints | (:wat::runtime::current-thread) returns the thread id | ✓ |
| C — `expected_user_main_signature` returns `[] -> :wat::core::nil` | Updated; old shape gone | ✓ |
| D — `validate_user_main_signature` enforces new shape | Diagnostics name the new shape; fail tests confirm | ✓ |
| E — `wat/kernel/exit-code.wat` deleted | `git status` shows D | ✓ |
| F — Zero ExitCode references in src/ + crates/ + wat/ | `grep -rn "wat::kernel::ExitCode" src/ crates/ wat/` returns nothing | ✓ |
| G — `invoke_user_main` takes no args | Signature simplified | ✓ |
| H — wat-cli plumbs argv into ambient + drops main_args construction | crates/wat-cli/src/lib.rs simplified | ✓ |
| I — spawn-process child invokes fn `[] -> :nil` | src/spawn_process.rs simplified | ✓ |
| J — Walker BareLegacyMainSignature fires on not-new-shape | Diagnostic names new shape | ✓ |
| K — New fixture tests pass | `cargo test --release --test wat_arc170_slice_1e_*` green | ✓ |
| L — Workspace cargo test runs (red is fine) | `cargo test --release --workspace --no-fail-fast` produces a number; we don't expect 0 fail | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs in source; no deferral language | ✓ |
| N — Zero Mutex usage | no Mutex/RwLock/CondVar (uses OnceLock + AtomicBool patterns from existing src/runtime.rs:51) | ✓ |
| O — Phase A + slice 1d work untouched | git diff shows slice 1e only edits the files this BRIEF lists | ✓ |

## What's expected to break

Substrate-as-teacher input for revised slice 3:

- All test fixtures with `:user::main` 4-arg signature + ExitCode
  return — walker fires; tests fail with the new diagnostic
- All test fixtures referencing `:wat::kernel::ExitCode` —
  unresolved symbol
- Tests that pass `args: Vec<Value>` to `invoke_user_main`
  (internal `src/freeze.rs:1183+`) — signature mismatch; either
  retire those tests or pass empty Vec
- Anything referencing `wat/kernel/exit-code.wat` from a load!
  expression — file not found

Predicted workspace fail-count delta: +50 to +200 from the
foundation baseline. Surface as input for slice 3, not a
problem to solve in slice 1e.

## Honest delta categories

- Substrate eval-arm wiring detail surprises (especially for
  the new ambient values) — surface, don't workaround
- Walker variant integration — if the walker's tracking
  vocabulary doesn't have a clean way to express
  "anything-but-new-shape," surface for design discussion
- spawn-process child path complexity — if simplifying the
  child invocation reveals deeper coupling we missed, surface
  before continuing
- FM 5 trap — TODOs verboten

## What's next (orchestrator-side, post-slice-1e)

1. Score per EXPECTATIONS-SLICE-1E.md
2. Author SCORE-SLICE-1E.md
3. Commit slice 1e atomically
4. Slice 1f BRIEF + EXPECTATIONS authored — three substrate
   services
5. Spawn slice 1f
