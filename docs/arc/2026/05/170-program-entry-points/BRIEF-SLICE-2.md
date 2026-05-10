# Arc 170 slice 2 — substrate consumer (wat-level surface)

> **Status (2026-05-09):** REDRAFTED post-slice-1c. Prior frozen
> v1-shape content retired. This BRIEF reflects the full settled
> foundation: slice 1b's `extract_closure` API + ClosurePackage
> { prologue, entry_form }; slice 1c's typed-channel substrate
> (PipeFd Sender/Receiver via Option B internal enum) + Process<I,O>
> additive shape (legacy 4 fields retained as bandaid; slice 4
> retires per bandaid-bounded-by-arc-close discipline).

## Goal

Mint the wat-level surface that consumes the substrate foundation
shipped in slices 1 → 1b → 1c. This is the slice that turns the
substrate primitives into wat-callable verbs; the slice that
makes `:user::main` argv-aware + ExitCode-returning; the slice
that fires substrate-as-teacher walkers on legacy callers so
slice 3 can mechanically sweep them.

This slice ships RED workspace per arc 168 precedent — the
walkers fire FATAL on user-source pre-pass; user-authored
callsites of legacy verbs + 3-arg `:user::main` immediately fail
to freeze. That broken-test stream is slice 3's input. Stdlib
paths (`register_stdlib_*`) silently migrate per existing walker
scoping.

## Read first (in order)

1. `docs/arc/2026/05/170-program-entry-points/DESIGN.md` —
   full arc; tier framework; client/server framing; the bandaid-
   bounded-by-arc-close discipline at slice 4
2. `docs/arc/2026/05/170-program-entry-points/TIERS.md` — typed-
   channel uniformity across tiers; OS-boundary exception
   (`:user::main` keeps IOReader/IOWriter/argv :Vector\<String\>)
3. `docs/arc/2026/05/170-program-entry-points/REALIZATIONS-SLICE-1.md`
   — six framing passes; pass 6 (bandaid-bounded discipline);
   FM 5/9/10/11 enforcement
4. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1B.md` —
   slice 1b's API + Symbol→Keyword precedent
5. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1C.md` —
   slice 1c's typed-channel substrate + Process additive shape
6. `docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-2.md`
   — your scorecard
7. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 11,
   FM 12, FM 16) — discipline floor

## Slice 1b + 1c context (foundation)

**Slice 1b** (commits `a23acf3` + `365343f` + SCORE `84b6ca6`):

```rust
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,
    pub entry_form: WatAST,  // Keyword AST for keyword-path; fn-form AST for inline lambda
}

pub fn extract_closure(
    fn_value: &Value,
    parent_symbols: &SymbolTable,
    parent_types: &TypeEnv,
) -> Result<ClosurePackage, ExtractionError>;
```

**Slice 1c** (commits `3c737ee` + `8eda4d3` + SCORE `5d9fc34`):

```rust
// Option B substrate (transport-polymorphic with internal enum)
pub enum SenderInner {
    Crossbeam(Arc<crossbeam_channel::Sender<Value>>),
    PipeFd { writer: PipeWriter, encoder: EdnEncoder },
}
pub enum ReceiverInner {
    Crossbeam(Arc<crossbeam_channel::Receiver<Value>>),
    PipeFd { reader: PipeReader, decoder: EdnDecoder },
}
Value::wat__kernel__Sender(Arc<SenderInner>)
Value::wat__kernel__Receiver(Arc<ReceiverInner>)

// Process<I,O> ADDITIVE (bandaid; retires in slice 4)
:wat::kernel::Process<I,O> = {
  stdin :IOWriter,    // legacy bandaid; slice 4 retires
  stdout :IOReader,   // legacy bandaid; slice 4 retires
  stderr :IOReader,   // legacy bandaid; slice 4 retires
  handle :ProgramHandle,
  tx :Sender<I>,      // typed-channel transport (USE THIS in slice 2)
  rx :Receiver<O>,    // typed-channel transport (USE THIS in slice 2)
}
```

Slice 2 USES the typed-channel fields (`tx` + `rx`); leaves the
legacy 3 fields populated with placeholder values (slice 1c's
construction sites already do this; agent inspects + matches).

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points`
- Multiple commits + pushes welcome
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships
- DO NOT edit SCORE-SLICE-1.md / SCORE-SLICE-1B.md / SCORE-SLICE-1C.md
  (immutable per `feedback_inscription_immutable.md`)

## Substrate edits

### 1. `:wat::kernel::ExitCode` typealias

```scheme
(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)
```

POSIX truth (0-255). Place in a wat-side kernel file; `wat/kernel/`
already has `channel.wat`. Either extend an existing kernel file
or mint a new `wat/kernel/exit-code.wat`. Agent picks; surfaces
the choice + reasoning.

### 2. `:user::main` 4-arg signature update + validator

`src/freeze.rs`:

```rust
// expected_user_main_signature — UPDATE to:
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

// validate_user_main_signature — UPDATE to:
// - 4 params required (was 3)
// - 4th param is :wat::core::Vector<wat::core::String> (argv)
// - return type is :wat::kernel::ExitCode (was :wat::core::nil)
// - parameter slot labels in error messages: stdin, stdout, stderr, argv
// - Reject 3-arg main with diagnostic naming the new contract
```

### 3. `eval_kernel_spawn_process(fn)` dispatch arm

New module `src/spawn_process.rs` (or extend `src/fork.rs` —
agent picks the cleaner organization). The dispatch arm:

```rust
pub fn eval_kernel_spawn_process(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";
    // arity 1 (fn)
    // eval the fn arg → fn Value
    // Use slice 1b's extract_closure(fn_value, sym, types)
    //   → ClosurePackage { prologue, entry_form }
    //
    // Fork OS process:
    // - Parent side: build PipeFd Sender + PipeFd Receiver pair
    //   using slice 1c's substrate (typed_channel module)
    // - Child side: freeze prologue → eval entry_form in fresh
    //   world → fn Value; apply fn Value with child-side typed
    //   channel handles
    //
    // Returns :wat::kernel::Process<I,O> Struct Value with the
    // parent-side typed channel handles (tx + rx) populated; the
    // legacy stdin/stdout/stderr fields populated with EITHER:
    //   (a) raw byte-pipe handles (matches slice 1c's
    //       fork-program-ast pathway construction)
    //   (b) bridges to the typed channels (each typed Sender wraps
    //       byte-pipe-fd; expose the underlying fd)
    //
    // Per slice 1c's additive shape: choice (a) is what slice 1c
    // already does in fork-program-ast. spawn-process should
    // mirror; slice 4 retires both at the legacy-field-removal
    // pass.
}
```

Wire the dispatch arm in `src/runtime.rs:3535-3537`:

```rust
":wat::kernel::spawn-process" => crate::spawn_process::eval_kernel_spawn_process(args, env, sym),
```

Legacy dispatch arms (`fork-program`, `fork-program-ast`,
`spawn-program`, `spawn-program-ast`) STAY UNCHANGED during the
sweep window — they keep working for stdlib callers; user-source
callers fail at the walker pre-pass before reaching them.

### 4. `invoke_program_entry` helper (or equivalent)

Slice 1's `invoke_user_main` invokes the `:user::main` symbol
specifically. Slice 2's spawn-process invokes the closure-extracted
entry — a Keyword (for keyword-path input) or fn-form (for
inline-lambda input) that evaluates to a fn Value.

Add a sibling helper that:
1. Evaluates `entry_form` in the frozen world → fn Value
2. Calls `apply_function` on that fn Value with channel-handle args

Or: the spawn-process implementation does this inline (probably
cleaner).

### 5. wat-cli argv passthrough + ExitCode handling

`crates/wat-cli/src/lib.rs::run` (or wherever `invoke_user_main`
is called):

```rust
let argv: Vec<String> = std::env::args().collect();
let main_args: Vec<Value> = vec![
    Value::io__IOReader(stdin),
    Value::io__IOWriter(stdout),
    Value::io__IOWriter(stderr),
    value_vector_of_strings(argv),  // NEW
];
let exit_value = invoke_user_main(&world, main_args)?;
let exit_code = match exit_value {
    Value::U8(n) => n as i32,
    other => {
        // type-checker should prevent this; defensive arm
        eprintln!("error: :user::main returned non-ExitCode value: {:?}", other);
        1
    }
};
std::process::exit(exit_code);
```

### 6. Substrate-as-teacher walker variants

Three new variants in `src/check.rs`:

- **`BareLegacyMainSignature { span: Span }`**:
  - Fires when freezing `:user::main` with 3-arg signature
  - Diagnostic explains the new 4-arg + ExitCode contract
  - Migration template included
  - Wired into `freeze.rs:599-607` user-source pre-pass

- **`BareLegacyForkProgram { span: Span }`**:
  - Fires on `:wat::kernel::fork-program{,_ast}` callsites
  - Diagnostic names `:wat::kernel::spawn-process` as replacement
  - Explains fn-input reshape; cites DESIGN

- **`BareLegacySpawnProgram { span: Span }`**:
  - Fires on `:wat::kernel::spawn-program{,_ast}` callsites
  - Diagnostic surfaces both options:
    - `spawn-process(fn)` for fork semantics
    - `spawn-thread(fn)` for parent's-world semantics
  - Cites DESIGN's two-mode taxonomy

Each variant follows arc 167/168/169 walker pattern (Display +
Diagnostic + walker body + tests).

### 7. New `tests/wat_arc170_program_contracts.rs`

Integration tests for the new contracts:

- **T1**: `:user::main` 4-arg signature freezes; 3-arg fires walker
- **T2**: `:user::main` returns ExitCode (u8) — value 0 propagates;
  value 42 propagates
- **T3**: argv pure passthrough — wat program reads argv[0..N]
  matching what wat-cli received
- **T4**: `(:wat::kernel::spawn-process fn)` — fn matching
  `:user::process` contract `[rx <- :Receiver<I> tx <- :Sender<O>] -> :wat::core::nil`
  spawns OS process; typed-channel send/recv works end-to-end
  through EDN-encoded pipes (slice 1c substrate); child exits cleanly
- **T5**: `(:wat::kernel::spawn-process inline-lambda)` — inline
  lambda works (uses slice 1b's fn-form entry_form path)
- **T6**: `(:wat::kernel::spawn-process factory-fn)` — factory-pattern
  capture works (single-level capture via slice 1b's prologue)
- **T7**: `(:wat::kernel::spawn-process)` with a fn capturing a
  `Sender<i64>` from let-scope — fires `NonPortableCapture`
  diagnostic from slice 1
- **T8**: `(:wat::kernel::fork-program ...)` callsite — walker fires
- **T9**: `(:wat::kernel::spawn-program ...)` callsite — walker fires
- **T10**: `(:wat::kernel::spawn-thread fn)` — UNCHANGED behavior;
  positive control verifying no regression
- **T11**: 3-arg `:user::main` — walker fires with the
  BareLegacyMainSignature diagnostic

## Critical syntax shapes

Per arc 167 + arc 109 + arc 153 doctrines:

- fn-form: `(:wat::core::fn [name <- :T ...] -> :Ret body)` —
  flat-vector binders with `<-` arrows; FQDN keyword
- defn: `(:wat::core::defn :name [params] -> :Ret body)`
- Type names: `:wat::core::nil` (NOT bare `:nil`),
  `:wat::kernel::Receiver<I>`, `:wat::kernel::Sender<O>`,
  `:wat::kernel::ExitCode`
- Wat type expressions: no inner colon before generic
  (per `feedback_wat_colon_quote.md`); no whitespace inside `<>`,
  `:(...)`, `:fn(...)`, `:[...]`

## Honest delta categories (if surfaced, report; don't bridge)

- **`invoke_program_entry` vs inline approach** — agent picks.
  If `apply_function` already handles invoking a non-`:user::main`
  symbol given its name, no helper needed; surface as "didn't
  need this" delta.
- **ExitCode typealias placement** — agent picks file. Surface
  if pattern not obvious.
- **Slice 1b honest delta A reprise (Symbol→Keyword)** — already
  baked into slice 1b's shipped behavior; spawn-process consumes
  whatever entry_form shape extract_closure returned. No action
  for slice 2 unless slice 1b's API changes.
- **Slice 1c honest delta C reprise (select rejects PipeFd
  Receivers)** — if integration tests need select-over-process-
  pipes, surface; that's substrate work outside slice 2 scope.
- **Slice 1c honest delta D reprise (try-recv on PipeFd returns
  Disconnected)** — if integration tests need real non-blocking
  recv on process pipes, surface; same scope rule.
- **Slice 1c honest delta E reprise (EDN round-trip semantics)** —
  Tuple→Vec, Some(x)→x. Tests should match the documented
  semantics per slice 1c's tests; surface if tripped.
- **fn Value → fn-form AST already in slice 1b** — slice 1b's
  `function_to_fn_form` exists; spawn-process's child-side
  invocation just evals entry_form (which IS the fn-form for
  inline-lambda input).
- **wat-side `:user::main` 4-arg consumers — slice 3 sweeps**.
  Slice 2's walker fires; slice 3 mass-fixes the existing 3-arg
  callsites.
- **stdlib `wat/std/sandbox.wat` + `wat/std/hermetic.wat`** still
  use legacy fork-program/spawn-program verbs internally (slice
  1c kept Process additive so stdlib still works through legacy
  dispatch arms). Slice 2 walkers do NOT fire on stdlib (per
  freeze.rs:599-607 user-source-only scoping). Stdlib continues
  working through legacy dispatch arms during sweep window;
  slice 3 rebuilds sandbox.wat + hermetic.wat on the new
  spawn-process; slice 4 destructively retires legacy verbs +
  Process legacy fields.
- **FM 5 trap** — TODOs verboten. STOP + surface.

## Predicted runtime

90-180 minutes (opus). Time-box hard cap at 360 minutes.

Comparable to slice 1's prediction (90-180; actual ~150). Slice
2 work:
- ExitCode typealias (small)
- :user::main signature update + validator (small)
- spawn-process verb dispatch + child-invocation helper (medium)
- wat-cli argv + ExitCode (small)
- 3 walker variants + Display + Diagnostic + bodies (medium)
- 11 integration tests (medium)

Smaller than slice 1's "from scratch" because the substrate
foundation (slices 1, 1b, 1c) is settled; slice 2 wires existing
substrate into wat-level surface.

## Branch state at slice 2 start

```
$ git log --oneline -5
26e8052 arc 170: bandaid-bounded-by-arc-close discipline
5d9fc34 arc 170 slice 1c: SCORE — 18/18 rows pass, Mode A clean, ~90 min
8eda4d3 arc 170 slice 1c: Process<I,O> additive reshape + integration tests
3c737ee arc 170 slice 1c: typed-channel substrate (Option B; transport-polymorphic)
4ea35bc arc 170 slice 1c: BRIEF + EXPECTATIONS authored
```

`cargo test --workspace` baseline at slice 2 start: `passed: 2124
failed: 0`.

Post-slice-2 expected: workspace ships RED (walker fires fatal
on legacy user-source callsites). Capture actual fail count;
slice 3 sweeps to green.

## SCORE artifact

After slice 2 ships, orchestrator writes SCORE-SLICE-2.md
(scorecard from EXPECTATIONS-SLICE-2 + honest deltas + calibration
row). You report to chat; orchestrator owns the SCORE artifact +
commit per `feedback_paperwork_orchestrator_side.md`.
