# Arc 170 slice 2 — EXPECTATIONS

> **Status (2026-05-09):** REDRAFTED post-slice-1c. Reflects full
> settled foundation (slice 1b API + slice 1c typed-channel
> substrate + Process additive shape).

## Independent prediction

**Predicted runtime band: 90-180 minutes (opus agent).**

Reasoning:
- ExitCode typealias + :user::main signature update (~small;
  parallel to arc 167's signature work)
- spawn-process verb dispatch + child-invocation helper (~medium;
  consumes slice 1b's extract_closure + slice 1c's PipeFd
  Sender/Receiver)
- wat-cli argv + ExitCode plumbing (~small; existing
  `invoke_user_main` call site updated)
- 3 walker variants (~medium; arc 167/168/169 walker pattern
  precedent)
- 11 integration tests (~medium)

Comparable to slice 1c (90-180 min; actual ~90). Foundation is
settled; this slice wires it.

**Time-box (2× upper-bound): 360 minutes.** Hard cap at 360.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — DESIGN-intent alignment | wat-level `(:wat::kernel::spawn-process fn)` works end-to-end with typed-channel I/O via slice 1c substrate; `:user::main` 4-arg + ExitCode; argv pure passthrough; the fn IS the program (no entry-keyword ceremony at wat surface) | ✓ |
| B — `:wat::kernel::ExitCode` typealias | minted; aliases `:wat::core::u8`; placed in wat/kernel/ (location chosen + reasoned) | ✓ |
| C — `expected_user_main_signature` updated | 4-arg vector (IOReader IOWriter IOWriter Vector\<String\>); ret type :wat::kernel::ExitCode | ✓ |
| D — `validate_user_main_signature` updated | rejects 3-arg main with diagnostic naming new contract; rejects nil-return main; new 4-arg + ExitCode passes | ✓ |
| E — `eval_kernel_spawn_process(fn)` minted | new dispatch arm `:wat::kernel::spawn-process` registered in runtime.rs; takes fn arg; calls slice 1b's extract_closure; uses slice 1c's PipeFd Sender/Receiver substrate; child invokes via entry_form eval; returns Process<I,O> Struct value with typed-channel handles | ✓ |
| F — `invoke_program_entry` helper (or inline) | child-side invocation evaluates entry_form → fn Value; applies fn Value with channel-handle args. Helper minted OR inlined; agent picks; surfaces decision | ✓ |
| G — Legacy dispatch arms unchanged | `eval_kernel_fork_program{,_ast}` + `eval_kernel_spawn_program{,_ast}` arms STAY AS-IS during sweep window (per arc 168 precedent + bandaid-bounded discipline; slice 4 retires) | ✓ |
| H — wat-cli argv passthrough | `crates/wat-cli/src/lib.rs` collects `std::env::args()` into Vec\<String\>; passes as 4th arg of invoke_user_main; wat-cli flag parsing unaffected | ✓ |
| I — wat-cli ExitCode handling | Value::U8 return → `std::process::exit(u8 as i32)`; defensive arm for non-u8 | ✓ |
| J — `BareLegacyMainSignature` walker variant | new variant in `Walker` enum + Display + Diagnostic + body; fires on 3-arg main signature at user-source pre-pass; tests verify firing | ✓ |
| K — `BareLegacyForkProgram` walker variant | new variant + Display + Diagnostic + body; fires on `:wat::kernel::fork-program{,_ast}` user-source callsites; tests verify firing | ✓ |
| L — `BareLegacySpawnProgram` walker variant | new variant + Display + Diagnostic + body; fires on `:wat::kernel::spawn-program{,_ast}` user-source callsites; tests verify firing | ✓ |
| M — `tests/wat_arc170_program_contracts.rs` | new integration test file; T1-T11 from BRIEF § 7 all pass | ✓ |
| N — Workspace ships RED | walker fires fatal on user-source legacy callsites; expected fail count > 0; agent captures actual count; documents in chat report. Stdlib paths silently migrate (per freeze.rs user-source-only walker scoping). New arc170 contract tests T1-T11 should ALL pass | ✓ |
| O — Slice branch on remote | `arc-170-program-entry-points` carries slice 2 commit(s) + this scorecard; main untouched | ✓ |
| P — Zero Mutex usage | no Mutex/RwLock/CondVar introduced (zero-mutex doctrine; per `feedback_zero_mutex.md`) | ✓ |
| Q — SCORE-SLICE-1.md / 1B / 1C untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |
| R — Slice 1b + 1c API unchanged | extract_closure signature + ClosurePackage shape untouched; typed_channel module API untouched | ✓ |
| S — No spawn-thread changes | spawn-thread keeps existing behavior (positive control via T10) | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **`invoke_program_entry` shape** — helper or inline. Agent picks
  + surfaces. If apply_function suffices, no helper needed.
- **ExitCode typealias placement** — wat/kernel/ file choice.
- **Slice 1b honest delta A (Symbol→Keyword) consequence** — child-
  side eval of entry_form for keyword-path input goes through
  Keyword resolution (slice 1b's substrate-fit choice). Verify
  spawn-process's child-side correctly evaluates Keyword
  entry_forms; surface if it doesn't unify with fn-form
  entry_forms.
- **Slice 1c honest delta C (select on PipeFd)** — if integration
  tests need select over process-pipes, surface as substrate gap.
- **Slice 1c honest delta D (try-recv on PipeFd)** — if integration
  tests need real non-blocking recv, surface as substrate gap.
- **Slice 1c honest delta E (EDN round-trip semantics)** — Tuple→Vec,
  Some(x)→x. Tests should match documented semantics; surface
  if tripped.
- **stdlib breakage scope** — slice 2's walker fires on user-source
  only (freeze.rs:599-607 scoping). Verify stdlib (wat/std/) is
  untouched by walker; surface if walker scoping is wrong.
- **Process legacy field placeholder shape** — when constructing
  Process Value in spawn-process, the legacy stdin/stdout/stderr
  fields need values (per slice 1c's additive shape). Match how
  slice 1c's fork-program-ast pathway constructs these (raw byte-
  pipe handles). Surface if pattern unclear.
- **FM 5 trap** — TODOs verboten. STOP + surface.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 90-180 min band.

Substrate edits:
- ExitCode typealias: ___ lines / ___ tests
- :user::main signature update + validator: ___ lines / ___ tests
- eval_kernel_spawn_process: ___ lines / ___ tests
- invoke_program_entry: ___ lines or N/A (inline)
- wat-cli argv + ExitCode: ___ lines / ___ tests
- 3 walker variants: ___ lines / ___ tests
- Integration tests: ___ count

Workspace fail count post-slice-2: ___ (slice 3's sweep input).

Honest deltas surfaced: ___ (count + brief).

## What's next (orchestrator-side, post-slice-2)

When slice 2 ships:
- SCORE-SLICE-2.md authored + committed
- Slice 3 BRIEF + EXPECTATIONS authored — consumer sweep +
  testing-lib three-layer rebuild (the polish slice; covers
  hermetic.wat, sandbox.wat, wat/test.wat rebuild on typed-
  channel API + Layer 1/2/3 testing surface)
- Slice 3 spawn proceeds (sonnet for mechanical sweep; possibly
  opus for the testing-lib rebuild's design choices)

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-2.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
