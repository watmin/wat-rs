# Arc 170 slice 2 — substrate consumer (uses slice 1 closure extraction)

## Goal

Wire `:user::main`'s new contract (argv + ExitCode return) and
mint `:wat::kernel::spawn-process` (taking a fn directly), reaching
slice 1's `extract_closure` internally. Rename existing
`fork-program*` substrate verbs into the `spawn-process*` family.
Delete `spawn-program*` (in-thread fresh-world variant; Q1 settled
"retire"). Pass `std::env::args()` through wat-cli to `:user::main`.
Mint the substrate-as-teacher walkers that fire on legacy
3-arg-main signature + legacy fork-program / spawn-program verb
usage so slice 3 sweep can mechanically migrate consumers.

This slice ships a workspace where `cargo test` is GREEN with the
NEW contract enforced AND legacy shapes still freeze (for the
sweep window) — substrate-as-teacher discipline per arcs 167 / 168
/ 169 precedent.

## Slice 1 context (just shipped)

Slice 1 commit `787c977` minted `src/closure_extract.rs` +
`tests/wat_arc170_closure_extraction.rs`. Public API:

```rust
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,  // synthetic name :__closure::__pkg_<n> for inline lambdas
}

pub enum ExtractionError {
    NonPortableCapture { name, type_name, path },
    UnresolvedSymbol { name, span },
    Internal(String),  // gaps surfaced honestly
}

pub fn extract_closure(
    fn_value: &Value,
    parent_symbols: &SymbolTable,
    parent_types: &TypeEnv,
) -> Result<ClosurePackage, ExtractionError>;
```

Read `SCORE-SLICE-1.md` for the six honest deltas — most relevant
for slice 2:
- **Delta A**: `Value::wat__core__fn` arm in encoder returns
  Internal error. If slice 2 surfaces a real consumer needing
  captured-fn-value encoding, surface as honest delta — don't
  bridge in slice 2; slice 1 follow-up implements it.
- **Delta C**: Several Value kinds (HolonAST, WatAST, RustOpaque,
  holon::Vector, Instant, Duration) return Internal error. Same
  rule.
- **Delta D**: NonPortableCapture diagnostic uses runtime-level
  type names (e.g., `rust::crossbeam_channel::Sender`); the
  wat-surface `:wat::kernel::Sender<i64>` form is an extension
  point if slice 2 needs it.

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points` (already
  carries slice 1 commit + SCORE)
- Multiple WIP commits + pushes welcome on the branch
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships

## Read first (in order)

1. `docs/arc/2026/05/170-program-entry-points/DESIGN.md` — full
   arc scope; client/server framing; settled decisions; "What
   ships" table (substrate impact) is your shipping checklist
2. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1.md`
   — six honest deltas you might hit
3. `docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-2.md`
   — your scorecard
4. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 10,
   FM 11, FM 16) — discipline floor
5. Existing pieces this slice touches:
   - `src/freeze.rs` (lines 700-789) — `invoke_user_main`,
     `expected_user_main_signature`, `validate_user_main_signature`
   - `src/fork.rs` (lines 425-878) — `eval_kernel_fork_program_ast`,
     `eval_kernel_fork_program`, `fork_program_from_source`
   - `src/spawn.rs` — `eval_kernel_spawn_program{,_ast}` (entire
     file deletes; mini-TCP guidance in module docstring stays
     elsewhere)
   - `src/runtime.rs` lines 3530-3540 — dispatch arms for the four
     legacy verbs
   - `crates/wat-cli/src/lib.rs` — argv plumbing + exit code
     handling
   - `src/check.rs` — walker pattern (see arcs 167/168/169 for
     `BareLegacy*` walker variants)

## Substrate edits

### 1. Mint `:wat::kernel::ExitCode` typealias

`(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)`

POSIX truth (0-255). Bodies write `(:wat::core::u8 0)` for
success; non-zero values propagate to OS. Lives wherever existing
kernel typealiases live (probably wat-side stdlib;
`wat/holon/kernel/` or similar — discover by grep).

### 2. Update `:user::main` signature (4-arg + ExitCode return)

```rust
// src/freeze.rs::expected_user_main_signature
pub fn expected_user_main_signature() -> (Vec<TypeExpr>, TypeExpr) {
    let params = vec![
        TypeExpr::Path(":wat::io::IOReader".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
        TypeExpr::Parametric {
            head: ":wat::core::Vector".into(),
            args: vec![Box::new(TypeExpr::Path(":wat::core::String".into()))],
        },
    ];
    let ret = TypeExpr::Path(":wat::kernel::ExitCode".into());
    (params, ret)
}
```

`validate_user_main_signature` updates parameter slot labels to
add `argv` (4th slot) and updates return-type expectation. The
wat-surface error message for a 3-arg main is the *substrate-as-
teacher* surface — don't make this hostile; the walker fires the
diagnostic with migration guidance.

### 3. Mint `eval_kernel_spawn_process(fn)`

Single primitive — fn input, ClosurePackage internally, reaches
today's `fork_program_from_source` pathway. No `_ast` variant.

```rust
// src/spawn_process.rs (new module) or extend src/fork.rs
pub fn eval_kernel_spawn_process(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";
    // arity 1 (fn)
    // eval the fn arg
    // closure_extract::extract_closure(&fn_value, sym, &type_env)
    //   -> ClosurePackage { forms, entry }
    // forks OS process; child invokes the entry symbol post-freeze
    //   (forms include the fn def as :user::process or whatever the
    //    closure-extraction synthesized name is; entry is what we
    //    apply_function on the child side)
    // Returns :wat::kernel::Process struct (same as fork-program-ast)
}
```

The reach into `fork_program_from_source` (or
`fork_program_from_forms` if that's the cleaner pathway) replaces
the current "freeze from string" with "freeze from forms"; the
child invokes `entry` not `:user::main` (the extracted fn satisfies
`:user::process` contract — `(IOReader IOWriter IOWriter) -> :nil`).

This invocation difference is where slice 2 diverges from arc 104:
the kernel-spawned child no longer evaluates `:user::main`; it
evaluates the closure-extracted entry directly. `invoke_user_main`
stays for the CLI path; child-process invocation needs a sibling
`invoke_program_entry(world, entry_name, args)` helper.

### 4. Rename `eval_kernel_fork_program*` arms

Pre-arc-170:
- `:wat::kernel::fork-program` → `eval_kernel_fork_program`
- `:wat::kernel::fork-program-ast` → `eval_kernel_fork_program_ast`

Post-arc-170:
- `:wat::kernel::spawn-process` → `eval_kernel_spawn_process` (the
  new one; takes fn)

The old `fork-program*` verbs stay live during the sweep window
(slice 3) but the dispatch arms route to a thin wrapper that fires
the `BareLegacyForkProgram` walker, then falls through to the
existing implementation. Same pattern as arcs 167/168/169.

### 5. Delete `eval_kernel_spawn_program*` arms (Q1 settled)

`:wat::kernel::spawn-program` and `:wat::kernel::spawn-program-ast`
get the deletion treatment. Walker `BareLegacySpawnProgram` fires
with diagnostic naming a migration target (spawn-process for fork
semantics; spawn-thread for parent's-world semantics — guide is a
pointer, not a decision).

`src/spawn.rs` may delete entirely; the mini-TCP discipline guidance
in its module docstring relocates to wherever the surviving verbs
live (probably DESIGN-doc-level).

### 6. wat-cli argv passthrough

```rust
// crates/wat-cli/src/lib.rs::run (or wherever invoke_user_main is called)
let argv: Vec<String> = std::env::args().collect();
// flag parsing already happens (--check, --check-output);
// pass FULL argv unfiltered to :user::main (per scratch/2026/05/019:
// "no silent argv reshaping; what the binary received is what the
// program sees")
let main_args: Vec<Value> = vec![
    Value::io__IOReader(stdin),
    Value::io__IOWriter(stdout),
    Value::io__IOWriter(stderr),
    value_vector_of_strings(argv),
];
let exit_value = invoke_user_main(&world, main_args)?;
let exit_code = match exit_value {
    Value::U8(n) => n as i32,
    _ => 1,  // type-checker enforces ExitCode return; this arm defensive
};
std::process::exit(exit_code);
```

The type-checker's contract enforcement guarantees `:user::main`
returns u8; the runtime arm above is a defensive tail (if the
contract somehow lies, exit 1 with a meaningful diagnostic). No
panic path; no silent success.

### 7. Substrate-as-teacher walkers

Three new walker variants in `src/check.rs` (mirror arcs
167/168/169 patterns):

- **`BareLegacyMainSignature`** — fires when freezing a `:user::main`
  with the 3-arg signature. Diagnostic explains the new 4-arg +
  ExitCode contract; cites scratch/2026/05/019; offers migration
  template.
- **`BareLegacyForkProgram`** — fires on `:wat::kernel::fork-program`
  or `:wat::kernel::fork-program-ast` callsites. Diagnostic names
  `:wat::kernel::spawn-process` as replacement; explains fn-input
  reshape; cites DESIGN.
- **`BareLegacySpawnProgram`** — fires on `:wat::kernel::spawn-program`
  or `:wat::kernel::spawn-program-ast` callsites. Diagnostic
  surfaces both options (spawn-process for fork semantics;
  spawn-thread for parent's world); names DESIGN's two-mode
  taxonomy.

Each walker's body is the firing pattern + Display + Diagnostic.
Tests verify walkers fire on legacy shapes and don't fire on
already-migrated shapes.

### 8. New `tests/wat_arc170_program_contracts.rs`

Integration tests covering the new contracts:

- T1: `:user::main` 4-arg signature freezes; 3-arg fires walker
- T2: `:user::main` returns ExitCode (u8) — value 0 propagates;
  value 42 propagates
- T3: argv pure passthrough — wat program reads argv[0..N] verifies
  whatever wat-cli received
- T4: `(:wat::kernel::spawn-process fn)` — fn matching
  `:user::process` contract spawns an OS process; pipes work end-
  to-end; child exits cleanly
- T5: `(:wat::kernel::spawn-process inline-lambda)` — inline lambda
  works (uses slice 1's synthetic-name path)
- T6: `(:wat::kernel::spawn-process factory-fn)` — factory-pattern
  capture works (single-level; per slice 1 honest delta this is
  the well-tested case)
- T7: `(:wat::kernel::spawn-process)` with a fn that captures a
  Sender — fires NonPortableCapture diagnostic from slice 1
- T8: `(:wat::kernel::fork-program ...)` callsite — walker fires
- T9: `(:wat::kernel::spawn-program ...)` callsite — walker fires
- T10: `(:wat::kernel::spawn-thread fn)` — UNCHANGED behavior;
  contract conceptually `:user::thread` but no walker fires (this
  is a positive control; verifies slice 2 doesn't break the
  existing thread spawn)

Predicted: 10 integration tests + whatever in-module unit tests
make sense for the new walker variants + signature validator.

## Branch isolation

- Slice 2 commit(s) on `arc-170-program-entry-points`
- main untouched
- Slice 3 (consumer sweep) consumes slice 2's substrate; both walkers
  + new spawn-process work together

## What slice 2 does NOT do

- **Slice 3 (sweep)**: User-code migrations across wat-rs +
  wat-tests are SLICE 3's territory. Slice 2 ships the substrate
  + walkers; slice 3 does the mechanical sonnet sweep.
- **Slice 4 (retirement)**: Old `fork-program*` and
  `spawn-program*` substrate code stays live during the sweep
  window. Slice 4 retires them.
- **Slice 5 (paperwork)**: SCOREs + INSCRIPTION + 058 row +
  USER-GUIDE.

## Predicted runtime

90-180 minutes (opus). Time-box hard cap at 360 minutes.
Comparable to slice 1 in scope (multiple substrate edits +
walker minting + integration tests); leverages slice 1's already-
shipped closure extraction so half the heavy lifting is done.

## On hitting honest deltas

Per FM 5: do NOT bridge with TODOs; STOP and surface.

Slice 1's honest deltas are the most likely ones to hit:
- If a real consumer needs `Value::wat__core__fn` encoded
  (closure-of-closure), STOP and surface — slice 1 follow-up,
  don't bridge in slice 2
- If diagnostic UX surfaces the wat-surface-vs-runtime type-name
  mismatch (slice 1 delta D), surface as honest delta; orchestrator
  decides whether to thread type-context through or accept the
  runtime-level name
- If the spawn-process pathway needs anything closure_extract
  doesn't currently provide, surface as honest delta; don't bridge
  by writing a parallel extraction

## Branch state at slice 2 start

```
$ git log --oneline -5
bb155ed (HEAD -> arc-170-program-entry-points, origin/arc-170-program-entry-points)
   arc 170 slice 1: SCORE — 14/14 rows pass, Mode A clean
787c977  arc 170 slice 1: Rust closure extraction substrate primitive
... (DESIGN + CLOSURE-EXTRACTION + BRIEFs)
... (slice 0 — branch creation)
```

`cargo test --workspace` baseline at slice 2 start: `passed: 2108
failed: 0`.

## Slice 2 commit policy

- One commit per logical chunk OK (e.g., "ExitCode + signature
  update", "spawn-process minted", "walkers minted", "wat-cli
  argv passthrough", "tests"). Multiple commits help orchestrator
  review the diff in pieces.
- Final commit message: "arc 170 slice 2: substrate consumer +
  walkers (uses slice 1 closure extraction)" (or similar)
- Push to `arc-170-program-entry-points` after each commit so
  orchestrator can monitor progress

## SCORE artifact

After slice 2 ships green, orchestrator writes
SCORE-SLICE-2.md with the scorecard from EXPECTATIONS-SLICE-2 +
honest deltas + calibration row. You report to the chat; the
orchestrator owns the SCORE artifact + commit (closure paperwork
is orchestrator-side per memory `feedback_paperwork_orchestrator_side`).
