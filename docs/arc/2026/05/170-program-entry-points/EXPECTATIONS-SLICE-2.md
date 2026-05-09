# Arc 170 slice 2 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 90-180 minutes (opus agent).**

Reasoning:
- Slice 2 is a substrate consumer of slice 1's closure extraction
  + walker minting + signature update + verb consolidation
- Comparable in scope to arc 167/168/169 slice 2 (which were
  90-180 min each)
- Multiple touch points: `freeze.rs` signature, `runtime.rs`
  dispatch, new `spawn_process.rs` (or extension to `fork.rs`),
  `check.rs` walker variants × 3, `wat-cli/lib.rs` argv plumbing,
  new integration test file
- Half the heavy lifting (closure extraction algorithm) ships in
  slice 1 already; slice 2 wires it
- Substrate-as-teacher walker pattern is a paved path from arcs
  167/168/169

**Time-box (2× upper-bound): 360 minutes.** If opus still
iterating at 180 min, in-flight check; hard cap at 360.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `:wat::kernel::ExitCode` typealias minted | new typealias defined wat-side; aliases `:wat::core::u8`; `cargo test --workspace` green with new alias resolvable | ✓ |
| B — `expected_user_main_signature` updated | 4-arg vector (IOReader IOWriter IOWriter Vector\<String\>); ret type `:wat::kernel::ExitCode` | ✓ |
| C — `validate_user_main_signature` updated | rejects 3-arg main with diagnostic naming the new contract; rejects nil-return main; new 4-arg + ExitCode passes | ✓ |
| D — `eval_kernel_spawn_process(fn)` minted | new dispatch arm `:wat::kernel::spawn-process`; takes fn arg; calls `extract_closure` internally; reaches `fork_program_from_source`-style pathway with extracted forms; child invokes synthesized entry not `:user::main`; returns `:wat::kernel::Process` struct | ✓ |
| E — `invoke_program_entry` (or equivalent) minted | helper for child-side: invokes a NAMED entry symbol post-freeze (vs `invoke_user_main` which invokes `:user::main` strictly); used by spawn-process | ✓ |
| F — `eval_kernel_fork_program*` arms wrapped to fire walker | dispatch arms route through walker-firing wrapper; fall through to legacy implementation during sweep window | ✓ |
| G — `eval_kernel_spawn_program*` arms — walker fires + path active | dispatch arms fire walker; legacy implementation can stay live during sweep OR be deleted in slice 2 if cleaner (sonnet's call) — note that DESIGN says "DELETE" but slice plan + arcs 167/168/169 precedent says "wrapper during sweep window, retire in slice 4". Pick one + document the choice. | ✓ |
| H — wat-cli argv passthrough | `crates/wat-cli/src/lib.rs::run` collects `std::env::args()` into `Vec<String>`; passes as 4th arg; wat-cli's flag short-circuits unaffected | ✓ |
| I — wat-cli ExitCode handling | converts `Value::U8` return → `std::process::exit(u8 as i32)`; defensive arm for non-u8 (shouldn't reach with type-checker enforcement) | ✓ |
| J — `BareLegacyMainSignature` walker variant | new variant in `Walker` enum + Display + Diagnostic + body firing on 3-arg main signature; tests verify firing | ✓ |
| K — `BareLegacyForkProgram` walker variant | new variant + Display + Diagnostic + body firing on `:wat::kernel::fork-program{,_ast}` callsites; tests verify firing | ✓ |
| L — `BareLegacySpawnProgram` walker variant | new variant + Display + Diagnostic + body firing on `:wat::kernel::spawn-program{,_ast}` callsites; tests verify firing | ✓ |
| M — `tests/wat_arc170_program_contracts.rs` | new integration test file; 10 tests T1-T10 from BRIEF-SLICE-2 § 8 all pass | ✓ |
| N — Workspace stays clean | post-slice-2 verified locally: `passed: 2108+10+ = ~2118+ failed: 0` (was 2108 pre-slice; +10 integration tests minimum) | ✓ |
| O — Zero Mutex usage | no Mutex / RwLock / CondVar introduced (zero-mutex doctrine; per memory `feedback_zero_mutex.md`) | ✓ |
| P — No slice 3 territory edits | wat-rs internal user code (lab, wat-tests/, internal `:user::main` defns) NOT migrated; slice 3's job; slice 2 surface stays minimal | ✓ |
| Q — Slice branch on remote | `arc-170-program-entry-points` carries slice 2 commit(s) + slice 1 commits + this scorecard; main untouched | ✓ |
| R — Substrate-as-teacher diagnostic UX | each walker's diagnostic names what changed, names migration target, cites DESIGN; consistent with arc 167/168/169 voice | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Slice 1 captured-fn-value gap exercised (Delta A reprise).**
  If real consumer fns capture closures-of-closures and slice 2's
  T6 hits `Value::wat__core__fn` arm's `Internal` error, surface
  as honest delta. orchestrator decides whether to extend slice 1
  (recursive sub-extraction) or accept the gap for slice 2.
- **Slice 1 Value-kind encoding gaps exercised (Delta C reprise).**
  Same rule for HolonAST / WatAST / RustOpaque / holon::Vector /
  Instant / Duration. Note that integration tests using these
  Value kinds in captures aren't required by BRIEF; if you find
  a NATURAL test pattern that hits one, surface — don't manufacture.
- **Diagnostic-UX runtime-vs-source type-name (Delta D reprise).**
  T7 exercises NonPortableCapture; if the runtime-level
  `rust::crossbeam_channel::Sender` reads hostile, surface as honest
  delta — don't bridge with a fake type-name lookup.
- **`fork-program*` retire-now vs sweep-window decision.** DESIGN
  says "DELETED" but slice plan + arcs 167/168/169 precedent says
  "wrapper during sweep window". Pick one + document; surface to
  orchestrator if you're uncertain.
- **`spawn-program*` same decision.** Same rule.
- **`invoke_program_entry` shape.** If `apply_function` already
  works for invoking a non-`:user::main` symbol given its name,
  no new helper needed — surface as a "didn't need this" honest
  delta. The shape exists in the BRIEF as a guess; the substrate
  may already provide it.
- **wat-side typealias placement (`:wat::kernel::ExitCode`).** If
  the wat-side stdlib doesn't have an obvious place, surface;
  don't manufacture a new file just for this typealias.
- **FM 5 trap.** Same rule as slice 1: if a TODO is tempting, STOP.
  Surface.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode
C failed). Compare to predicted 90-180 min band.

Substrate edits:
- `:wat::kernel::ExitCode` typealias: ___ lines / ___ tests
- `expected_user_main_signature` + validator update: ___ lines /
  ___ tests
- `eval_kernel_spawn_process`: ___ lines / ___ tests
- `invoke_program_entry`: ___ lines / ___ tests
- Walker variants × 3: ___ lines / ___ tests
- wat-cli argv + exit code: ___ lines / ___ tests
- Integration tests: ___ lines / ___ count

Honest deltas surfaced: ___ (count + brief).

Slice 1 deltas exercised:
- Delta A captured-fn-value: ___
- Delta C Value-kind gaps: ___
- Delta D diagnostic UX: ___

## What's next (orchestrator-side, post-slice-2)

When slice 2 ships green:
- SCORE-SLICE-2.md authored + committed
- Slice 3 BRIEF + EXPECTATIONS authored (sonnet sweep target —
  consumer migrations across wat-tests + lab if applicable)
- Slice 3 mechanical sweep spawned (sonnet, 90-180 min predicted)

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-2.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
