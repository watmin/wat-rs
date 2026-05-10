# Arc 170 slice 1f-δ — BRIEF (restore `run-sandboxed-hermetic-ast`)

**Sonnet pattern-apply.** Closes § Row K (from slice 1f-β-i V2 SCORE). The 854 baseline + 15 trio hermetic-test failures all share a single root cause: **`:wat::kernel::run-sandboxed-hermetic-ast` has zero eval arm in `src/runtime.rs`** (type-registered in `check.rs`, but the wat-side definition retired in commit `eb655d1` slice 3 without restoring a substrate impl).

Concrete failure (verified): all hermetic tests panic at `src/test_runner.rs:494` with `unknown function: :wat::kernel::run-sandboxed-hermetic-ast`. Single root cause; literal restore is the fix.

## Slice surface

> *"Restore the wat-side hermetic test scaffold."*

The TIERS.md migration to `spawn-process` is **not** this slice — it's a separate, future arc when there's a reason. This slice restores the foundation crack opened by slice 3's retirement-without-replacement.

## Scope

### Edit 1 — `src/runtime.rs` + `src/check.rs` — add Process accessor eval arms

The retired wat-side wrapper called `:wat::kernel::Process/stdin`, `:wat::kernel::Process/stdout`, `:wat::kernel::Process/stderr` to access the forked child's pipes. These eval arms don't exist today (only `Process/join-result` ships, at `runtime.rs:3636`).

Mirror the `Process/join-result` arm shape (`runtime.rs:15502+`) for three sibling accessors:

```rust
":wat::kernel::Process/stdin"  => eval_kernel_process_stdin(args, env, sym, list_span),
":wat::kernel::Process/stdout" => eval_kernel_process_stdout(args, env, sym, list_span),
":wat::kernel::Process/stderr" => eval_kernel_process_stderr(args, env, sym, list_span),
```

Each: takes a `Process<I,O>` Value::Struct, extracts the appropriate IO Value from the struct's fields, returns it. The Process struct's field layout (per `src/spawn_process.rs:220-225`):
- Field 0: `Value::wat__core__fn` body fn (or similar)
- Field 1+: includes `Value::io__IOWriter(stdin_writer)`, `Value::io__IOReader(stdout_reader)`, `Value::io__IOReader(stderr_reader)`, `Value::wat__kernel__ProgramHandle(...)`

**Surface friction if** the field order isn't directly accessible — check `src/spawn_process.rs:218-228` (the StructValue construction for `Process`) for the exact field index of each.

Type signatures in `src/check.rs`:
- `(:wat::kernel::Process/stdin proc) -> :wat::io::IOWriter`
- `(:wat::kernel::Process/stdout proc) -> :wat::io::IOReader`
- `(:wat::kernel::Process/stderr proc) -> :wat::io::IOReader`

Mirror the `Process/join-result` type registration at `src/check.rs:15500+` (or wherever it lives).

### Edit 2 — new `wat/kernel/hermetic.wat` — restore the wat-side wrapper

Restore content from git: `git show eb655d1^:wat/std/hermetic.wat`. Two adaptations:

1. **File location:** `wat/kernel/hermetic.wat` (not `wat/std/` per arc 109 K-namespace doctrine).
2. **Fold in the helpers from `wat/std/sandbox.wat`:** `failure-from-process-died` is required (the hermetic wrapper calls it). Restore via `git show eb655d1^:wat/std/sandbox.wat` and include the helper. `drain-lines` is also referenced — already present in the deleted hermetic.wat itself (lines 47-56 of that file). The `string::join` ref is current.

The full set of wat-side fns this file should define:
- `:wat::kernel::drain-lines-acc` (helper)
- `:wat::kernel::drain-lines` (helper)
- `:wat::kernel::failure-from-process-died` (helper)
- `:wat::kernel::run-sandboxed-hermetic-ast<I,O>` (the entry verb)

Naming verification: arc 109 work didn't rename any of these. The signatures use current FQDN forms (`:wat::core::Vector<T>`, `:wat::core::Option<T>`, `:wat::core::Some`/`:wat::core::None` — all current). The `<I,O>` parametric form on the entry verb is per arc 112 (still valid).

### Edit 3 — `src/stdlib.rs` — register the new file

Insert after `wat/kernel/services/stderr.wat` entry:

```rust
// Arc 170 slice 1f-δ — restore :wat::kernel::run-sandboxed-hermetic-ast
// as wat-side wrapper around fork-program-ast (closes § Row K from
// slice 1f-β-i V2 SCORE). The TIERS.md migration to spawn-process
// remains a separate future arc.
WatSource {
    path: "wat/kernel/hermetic.wat",
    source: include_str!("../wat/kernel/hermetic.wat"),
},
```

Loading order: AFTER `wat/kernel/services/*.wat` (which the file doesn't depend on but logically follows), AFTER `wat/kernel/channel.wat` (provides Sender/Receiver typealiases the underlying Process uses).

## Pre-flight — verify before drafting wat content

Before writing `wat/kernel/hermetic.wat`, verify the substrate has:

```
:wat::kernel::fork-program-ast    — src/fork.rs:431 (eval_kernel_fork_program_ast) ✓
:wat::kernel::Process/join-result — src/runtime.rs:15502+ ✓
:wat::kernel::ProcessDiedError/to-failure — src/runtime.rs:3630 ✓
:wat::kernel::extract-panics      — src/runtime.rs:3633 ✓
:wat::kernel::failure-from-process-died — MUST RESTORE (this slice)
:wat::kernel::Process/stdin       — MUST ADD eval arm (this slice)
:wat::kernel::Process/stdout      — MUST ADD eval arm (this slice)
:wat::kernel::Process/stderr      — MUST ADD eval arm (this slice)
:wat::core::string::join          — src/runtime.rs:3211 ✓
:wat::kernel::RunResult           — type registered ✓
:wat::kernel::Failure             — type registered ✓
:wat::kernel::Frame               — type registered ✓
:wat::io::IOReader/read-line      — src/io.rs:845 ✓
:wat::io::IOWriter/write-string   — src/io.rs:1025 ✓
```

If any of these (besides the four "MUST" items) are missing, STOP and surface as honest-delta.

## What to NOT do

- No migration to `spawn-process` — that's a future arc
- No changes to `deftest-hermetic` macro in `wat/test.wat` — it already targets the right verb path
- No changes to wat-tests (they're already authored to use deftest-hermetic correctly)
- No new dependencies; no new Mutex/RwLock/CondVar
- No changes to slice 1f-γ orchestrator — it's done

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `Process/stdin` eval arm + type-check arm | grep + green check |
| B | `Process/stdout` eval arm + type-check arm | grep + green check |
| C | `Process/stderr` eval arm + type-check arm | grep + green check |
| D | `wat/kernel/hermetic.wat` exists, parses, type-checks | cargo check green |
| E | File defines all four fns (drain-lines-acc, drain-lines, failure-from-process-died, run-sandboxed-hermetic-ast) | grep |
| F | `src/stdlib.rs` registration entry | grep |
| G | `cargo check --release` green | clean |
| H | Hermetic test sample passes — pick one from `wat-tests/kernel/services/stdin.wat` (e.g., `stdin-test::spawn-shape`) | cargo test passes |
| I | Workspace failure count drops dramatically — expected delta ≥ -800 (from 869 → < 100 or close) | cargo test count |
| J | Workspace pass count rises by approximately the same amount | cargo test count |
| K | No new regression of pre-existing passing tests | re-baseline |
| L | Only 3 files modified: `src/runtime.rs`, `src/check.rs`, `src/stdlib.rs`, and 1 new wat file | git status |
| M | Zero new deps; zero Mutex/RwLock/CondVar | grep + Cargo.toml |
| N | Honest deltas surfaced | per FM 5 |

**14 rows.**

## Predicted runtime

**45-90 min sonnet.** Three small Rust accessor arms + literal restore of wat file from git + stdlib registration. The unknowns:
- Process struct field order (verify at `src/spawn_process.rs:218-228`)
- Any current-substrate gotchas the old wat file hits (arc 109 naming changes; arc 159 let-shape changes; etc.)

**Hard cap:** 180 min.

## Honest-delta categories (anticipated)

1. **Process struct field index ordering** — verify the indices of stdin/stdout/stderr fields in `src/spawn_process.rs:220-228`'s StructValue construction. Adjust accessor arms to extract the correct index.
2. **Old wat file uses pre-current syntax** — if arc 109/159/etc. renamed any verbs or changed let-shape, adapt. Most likely current. If issues, surface and adapt.
3. **drain-lines on stderr blocks** — old hermetic.wat warned about "no concurrent drain of stdout vs stderr." For the 854 baseline tests, stderr is small (assertion failure messages) and stdout is small. Concurrent drain is genuinely future work; document the inherited limitation inline.
4. **The 854 expected-drop** — sonnet should verify the workspace count drops dramatically. If it doesn't (say only 100 tests recover), that signals a deeper issue: surface immediately.

## Reference

- Slice 1f-β-i V2 SCORE § Row K — names this slice as the closer
- TIERS.md — the future spawn-process migration remains separate
- `git show eb655d1^:wat/std/hermetic.wat` — content to restore
- `git show eb655d1^:wat/std/sandbox.wat` — `failure-from-process-died` helper to fold in
- `src/runtime.rs:15502-15521` — Process/join-result arm shape (mirror for stdin/stdout/stderr)
- `src/spawn_process.rs:218-228` — Process struct field order (verify accessor arm indices)
- `src/fork.rs:431` — fork-program-ast eval arm (the substrate primitive the wat wrapper builds atop)

## Path forward post-slice-1f-δ

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. Workspace baseline returns to clean (or near-clean) state
3. **Slice 1f-ε** — Console retirement + consumer sweep
4. Arc 170 INSCRIPTION
