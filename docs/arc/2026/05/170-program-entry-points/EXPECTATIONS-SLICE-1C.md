# Arc 170 slice 1c — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 90-180 minutes (opus agent).**

Reasoning:
- New substrate primitive (typed-channel-over-EDN-pipes
  transport); leverages existing crossbeam Sender/Receiver +
  arc 092 wat-edn + arc 113 cascade pattern + existing pipe
  machinery in fork.rs / spawn.rs
- Process<I,O> struct reshape: Rust-level field updates +
  caller updates (probably 5-15 sites)
- Implementation choice for transport polymorphism (Option A/B/C
  in BRIEF) requires investigation; agent picks substrate-fit
- Rust integration tests for typed-channel-over-pipes round-trip
- Comparable to slice 1's prediction (90-180 min); slice 1
  shipped at ~150 min Mode A clean

**Time-box (2× upper-bound): 360 minutes.** Hard cap at 360.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — DESIGN-intent alignment | user-visible `Sender<T>` / `Receiver<T>` abstraction unchanged across tier 1 + tier 2; transport is substrate-internal; user works in typed Values | ✓ |
| B — Transport-polymorphic Sender/Receiver | substrate supports both crossbeam (tier 1) and EDN-over-pipes (tier 2) backends; implementation choice (Option A/B/C from BRIEF) picked + reasoned | ✓ |
| C — EDN encoding at pipe boundary | Sender/send encodes typed Value to EDN bytes via wat-edn; writes to pipe fd; Receiver/recv reads bytes from pipe fd, parses EDN, returns typed Value; line-delimited per arc 113 cascade convention | ✓ |
| D — `:wat::kernel::Process<I,O>` reshape | struct fields change from `(stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle)` to `(tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle)`; stderr drops as separate field | ✓ |
| E — Wat-side struct definition updated | the wat type def for `:wat::kernel::Process<I,O>` (in stdlib) reflects new fields | ✓ |
| F — Caller updates (Rust) | `src/fork.rs` Process construction site (~line 867) updated; `src/spawn.rs` Process construction site (~line 220) updated; `src/runtime.rs` Process accessor sites (~line 15254, 15535, 15609) updated or retired | ✓ |
| G — Caller decision (wat) | `wat/std/hermetic.wat` Process accessor sites (lines 91, 102-103) — investigate + pick: break-as-substrate-as-teacher OR shim-with-warn. Decision documented in commit message; consistent with arc 168 precedent for slice-internal breakage | ✓ |
| H — Errors propagate via join-result | `Process/join-result` returns `Result<(), ProcessDiedError>`; pipe-close on Sender side maps to Result.Err on next send; no separate stderr-channel needed | ✓ |
| I — Rust integration tests | new test file (or extension) verifies tier-2 round-trip, multi-Value stream, type fidelity (nested structs/vectors), error propagation, Process<I,O> field accessors. ≥5 tests | ✓ |
| J — Workspace state | depends on caller-decision row G. If break-as-teacher: workspace ships RED with hermetic-using tests failing (decided as input for slice 2); if shimmed: workspace stays at 2107/0. Either is acceptable; document the choice + actual fail count | ✓ |
| K — No spawn-process verb wiring | slice 2's territory; this slice does NOT mint `:wat::kernel::spawn-process` dispatch arm | ✓ |
| L — No `:user::main` signature changes | also slice 2's territory | ✓ |
| M — No walker variants | also slice 2's territory | ✓ |
| N — No new wat-level surface | beyond the wat-side Process struct definition update for new fields, no new wat-callable verbs | ✓ |
| O — Slice branch on remote | `arc-170-program-entry-points` carries slice 1c commit(s) + this scorecard; main untouched | ✓ |
| P — Zero Mutex usage | no Mutex/RwLock/CondVar introduced (zero-mutex doctrine; `feedback_zero_mutex.md`). Atomics + Arc + OnceLock permitted | ✓ |
| Q — SCORE-SLICE-1.md + SCORE-SLICE-1B.md untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |
| R — Slice 1b API unchanged | `closure_extract::extract_closure` + `ClosurePackage { prologue, entry_form }` shape unchanged; slice 1c does NOT touch closure_extract.rs | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Implementation choice for Sender/Receiver transport
  polymorphism** — Option A (separate Value variants), B
  (transport-polymorphic with internal enum), or C (multimethod
  dispatch via arc 146). Agent picks; surfaces reasoning. If the
  substrate's existing shape strongly suggests one, go with it.
- **EDN line-delimited edge cases** — Values containing
  characters that conflict with line delimiter (escaped
  newlines, etc.). wat-edn likely handles via standard EDN
  escaping; verify; surface if not.
- **Process caller breakage scope** — investigate how many
  sites touch Process/stdin, Process/stdout, Process/stderr.
  If small: update inline. If large: surface; may need a
  follow-up sweep slice.
- **`hermetic.wat` decision** — break-as-teacher vs shim-with-warn.
  Document the choice + reason. Either is acceptable per arc 168
  precedent (let it break) vs arc-already-progress (shim it).
- **Error semantics on pipe close** — EPIPE → wat-level
  Result.Err. Per arc 111 (Result-typed send/recv). Verify
  symmetry between crossbeam-disconnect and pipe-close error
  paths. Surface if they don't unify cleanly.
- **Slice 1 honest delta D reprise (diagnostic type-name)** —
  if your work touches NonPortableCapture diagnostic, the
  runtime-vs-source-spelling gap from slice 1 still applies.
  Don't fix in 1c; surface if tripped.
- **FM 5 trap** — TODOs verboten. STOP + surface.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 90-180 min band.

Subsystems built:
- Transport-polymorphic Sender/Receiver: ___ lines / ___ tests
- EDN encoding at pipe boundary: ___ lines / ___ tests
- Process<I,O> struct reshape: ___ lines changed across
  ___ files / ___ tests
- Caller updates (Rust): ___ sites
- Caller updates (wat) decision + impl: ___ sites
- Integration tests: ___ count

Honest deltas surfaced: ___ (count + brief).

Implementation choice (Sender/Receiver transport): Option ___;
reasoning: ___.

`hermetic.wat` decision: break-as-teacher / shim-with-warn;
fail-count impact: ___.

## What's next (orchestrator-side, post-slice-1c)

When slice 1c ships:
- SCORE-SLICE-1C.md authored + committed
- BRIEF-SLICE-2.md + EXPECTATIONS-SLICE-2.md REDRAFTED against
  slice 1b + slice 1c shipped foundations:
  - `closure_extract::extract_closure` API
  - `ClosurePackage { prologue, entry_form }` shape
  - typed-channel `:user::process` contract
  - `:wat::kernel::Process<I,O>` typed-channel handles
  - EDN-over-pipes substrate (slice 1c's deliverable)
- Slice 2 spawn proceeds against the full settled foundation

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1C.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
