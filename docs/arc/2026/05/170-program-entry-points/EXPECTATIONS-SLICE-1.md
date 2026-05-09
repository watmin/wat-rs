# Arc 170 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 90-180 minutes (opus agent).**

Reasoning:
- This is a NEW substrate primitive built from scratch (no
  existing template to mirror beyond Value→AST patterns from arc
  091 slice 8 struct→form)
- 5 subsystems (walker / dep closure / Value→AST / portability
  check / ClosurePackage assembly)
- 15 integration tests covering positive + negative cases
- Existing wat-rs infrastructure (SymbolTable, TypeEnv,
  startup_from_forms, apply_function) provides the building
  blocks but the integration is non-trivial
- Comparable in scope to arc 089 slice 1-3 (Db substrate +
  Service/loop) which ran 90-180 min each
- Larger in scope than arc 167/168/169 slice 1 (which were 30-90
  min) because arc 170 slice 1 mints an entire new module with
  its own algorithms

**Time-box (2× upper-bound): 360 minutes.** If opus still
iterating at 180 min, in-flight check; hard cap at 360.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `src/closure_extract.rs` minted | new module exists with public `ClosurePackage` struct + `ExtractionError` enum + `extract_closure` entry point | ✓ |
| B — Free-symbol walker | walks fn body + extracted dep ASTs; tracks scope (params, lets, nested fns); classifies free names (substrate primitive / user dep / user type / captured value) | ✓ |
| C — Dep-closure builder | recursive extraction with fixpoint; visited-set guards against infinite recursion; topological ordering of output | ✓ |
| D — Value→AST encoder | covers all portable Value kinds (primitives + Vector + HashMap + Struct + Enum variants + Option + Result + Tuple + Bytes); leverages existing struct→form for structs | ✓ |
| E — Portability type-check | refuses non-portable types (Sender / Receiver / Channel / Thread / Process / HandlePool / IOReader / IOWriter); returns `Err(NonPortableCapture)` with named field-path for nested cases | ✓ |
| F — ClosurePackage assembly | output forms in topological order: types → captures → user defns → entry; entry name canonical for keyword-path inputs, synthetic for lambda inputs | ✓ |
| G — All 15 Rust integration tests pass | tests/wat_arc170_closure_extraction.rs 15/15 green | ✓ |
| H — Workspace stays clean | inline pipeline shows `passed: 2106 failed: 0` (was 2091/0; +15 new tests) | ✓ |
| I — No wat-level surface added | no new wat-callable verbs minted in this slice; closure_extract::extract_closure is Rust-public but not registered in wat eval dispatch | ✓ |
| J — No spawn-process / spawn-thread / fork-program changes | those are slice 2's territory; slice 1 doesn't touch invocation paths | ✓ |
| K — No `:user::main` signature changes | also slice 2's territory; slice 1 doesn't touch validate_user_main_signature or invoke_user_main | ✓ |
| L — Slice branch on remote | branch carries the slice 1 commit(s); main untouched | ✓ |
| M — Zero Mutex usage | no Mutex / RwLock / CondVar introduced (zero-mutex doctrine; per memory `feedback_zero_mutex.md`) | ✓ |
| N — Substrate-as-teacher diagnostic | `NonPortableCapture` Err message names the offending capture, the type, the path (for nested), and suggests pipes/restructure | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Macro expansion ambiguity (Q-impl-1).** If walking the fn body
  surfaces unexpanded macro forms, the assumption was wrong;
  surface as honest delta.
- **Closure-of-closure handling (Q-impl-2).** If captured Values
  include fn values themselves, recursive sub-extraction needs
  thought; flag if non-trivial.
- **Type registry surprise.** If `parent_types.get` doesn't return
  the AST shape the encoder expects (e.g., for built-in types
  vs user types), surface.
- **Existing struct→form integration.** If extending struct→form
  for the broader Value→AST encoder requires more than additive
  changes, flag.
- **FM 5 trap.** If you find yourself wanting to leave a TODO or
  unhandled case, STOP. Surface as honest delta; orchestrator
  decides whether the gap blocks slice 1 or can defer to slice 2.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 90-180 min band.

Subsystems built:
- Free-symbol walker: ___ lines / ___ tests
- Dep-closure builder: ___ lines / ___ tests
- Value→AST encoder: ___ lines / ___ tests
- Portability check: ___ lines / ___ tests
- ClosurePackage assembly: ___ lines / ___ tests

Honest deltas surfaced: ___ (count + brief).

Q-impl resolutions:
- Q-impl-1 macro expansion: ___
- Q-impl-2 closure-of-closure: ___
- Q-impl-3 snapshot timing: ___
- Q-impl-4 recursive types: ___
- Q-impl-5 span preservation: ___

## What's next (orchestrator-side, post-slice-1)

When slice 1 ships green:
- Slice 2 BRIEF + EXPECTATIONS authored
- Slice 2 wires `eval_kernel_spawn_process` to call slice 1's
  `extract_closure` internally
- Slice 2 also handles `:user::main` argv + ExitCode + wat-cli
  plumbing + walkers for legacy shapes

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
