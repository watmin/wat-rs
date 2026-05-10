# Arc 170 slice 2 — SCORE

Wat-level surface shipped. Substrate-as-teacher pattern in
motion: workspace 1594/545 (was 2124/0) — exactly the BRIEF row N
prediction. 545 failures are slice 3's mechanical sweep input.

Mode A clean, ~180 min opus (within 90-180 predicted band; upper).
Branch `arc-170-program-entry-points` carries slice 2 commits
ending at `09d7b04`.

## Scope as shipped

New surface:
- `wat/kernel/exit-code.wat` — `(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)`
- `src/spawn_process.rs` — `eval_kernel_spawn_process(fn)` dispatch
  arm; uses slice 1b's `extract_closure` + slice 1c's PipeFd
  Sender/Receiver substrate
- `tests/wat_arc170_program_contracts.rs` — 11 BRIEF tests
  expanded into 15 sub-cases (T1/T8/T9/T11 split into
  canonical+legacy pairs)

Updates:
- `src/freeze.rs` — `expected_user_main_signature` (4-arg + ExitCode);
  `validate_user_main_signature` (4-arg + ExitCode validator)
- `src/check.rs` — 3 walker variants (BareLegacyMainSignature,
  BareLegacyForkProgram, BareLegacySpawnProgram) + Display +
  Diagnostic + bodies + scheme registration; fired at user-source
  pre-pass (`freeze.rs:599-619`)
- `src/runtime.rs` — `:wat::kernel::spawn-process` dispatch arm
- `src/fork.rs` — argv plumbing + ExitCode return arm
  (`invoke_user_main` invocation site)
- `crates/wat-cli/src/lib.rs` — argv passthrough via
  `std::env::args()`; ExitCode → `std::process::exit(u8 as i32)`
- `src/closure_extract.rs` — `:wat::core::nil` round-trip emit
  fix (honest delta A; load-bearing for child-world re-freeze)
- `src/{compose,harness,spawn,stdlib}.rs` — passthrough updates

## Scorecard

All 19 rows from EXPECTATIONS-SLICE-2.

| Row | Verified | Pass |
|-----|----------|------|
| A — DESIGN-intent alignment | `(:wat::kernel::spawn-process fn)` end-to-end with typed-channel I/O via slice 1c substrate; `:user::main` 4-arg + ExitCode; argv pure passthrough; the fn IS the program (Layer 3 substrate primitive; no entry-keyword ceremony at wat surface) | ✓ |
| B — `:wat::kernel::ExitCode` typealias | minted at `wat/kernel/exit-code.wat`; aliases `:wat::core::u8`; placement chosen + reasoned (mirrors `wat/kernel/channel.wat` precedent) | ✓ |
| C — `expected_user_main_signature` updated | 4-arg vector (IOReader IOWriter IOWriter Vector\<String\>); ret type `:wat::kernel::ExitCode` | ✓ |
| D — `validate_user_main_signature` updated | rejects 3-arg main with diagnostic; accepts new 4-arg + ExitCode signature | ✓ |
| E — `eval_kernel_spawn_process(fn)` minted | new module `src/spawn_process.rs`; dispatch arm registered in runtime.rs; calls slice 1b's extract_closure; uses slice 1c's PipeFd Sender/Receiver substrate; child invokes via entry_form eval; returns Process<I,O> with typed-channel handles + legacy-field placeholders matching slice 1c's fork-program-ast pattern | ✓ |
| F — `invoke_program_entry` helper or inline | inlined in `spawn_process.rs` (agent decision; surfaces in honest delta 3) | ✓ |
| G — Legacy dispatch arms unchanged | `eval_kernel_fork_program{,_ast}` + `eval_kernel_spawn_program{,_ast}` arms STAY AS-IS during sweep window (slice 4 retires per bandaid-bounded discipline) | ✓ |
| H — wat-cli argv passthrough | `std::env::args()` collected into Vec\<String\>; passed as 4th arg of `invoke_user_main`; flag parsing unaffected | ✓ |
| I — wat-cli ExitCode handling | `Value::U8` return → `std::process::exit(n as i32)`; defensive arm for non-u8 | ✓ |
| J — `BareLegacyMainSignature` walker variant | new variant + Display + Diagnostic + body; fires on 3-arg main signature at user-source pre-pass; tests verify firing (T11) | ✓ |
| K — `BareLegacyForkProgram` walker variant | new variant + Display + Diagnostic + body; fires on user-source `fork-program{,_ast}` callsites; tests verify (T8 split) | ✓ |
| L — `BareLegacySpawnProgram` walker variant | new variant + Display + Diagnostic + body; fires on user-source `spawn-program{,_ast}` callsites; tests verify (T9 split) | ✓ |
| M — `tests/wat_arc170_program_contracts.rs` | 15 sub-test pass (T1-T11 with T1/T8/T9/T11 split into canonical+legacy pairs); covers contract enforcement + walker firing + spawn-process end-to-end | ✓ |
| N — Workspace ships RED | **1594 passed / 545 failed** post-slice-2 (was 2124/0). Walker firings on user-source legacy patterns produced exactly the expected sweep input. 268 deftest_* (wat/test.wat macro generates legacy 3-arg main via expanded_user) + 277 other (legacy verb callsites + 3-arg main fixtures). Stdlib paths silently survive per `freeze.rs:599-619` user-source-only walker scoping. New arc170 contract tests 15/15 pass | ✓ |
| O — Slice branch on remote | `arc-170-program-entry-points` carries slice 2 commits ending at `09d7b04` + this SCORE; main untouched | ✓ |
| P — Zero Mutex usage | no Mutex/RwLock/CondVar introduced; Arc + crossbeam + atomics only | ✓ |
| Q — SCORE-SLICE-1.md / 1B / 1C untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |
| R — Slice 1b + 1c API unchanged | `extract_closure` signature + `ClosurePackage { prologue, entry_form }` shape untouched at the API level. Honest delta A: `function_to_fn_form` / `function_to_define_form_with_body` had a `:()`-emit bug fixed inline (slice 1b correctness; surfaced when slice 2 re-freezes through user-pre-pass; load-bearing). Public API shape unchanged; emit-side behavior corrected. typed_channel module API untouched | ✓ |
| S — No spawn-thread changes | spawn-thread keeps existing behavior (positive control via T10 in test suite) | ✓ |

## Honest deltas

### Delta A — closure-extract `:()` round-trip emit bug (slice 1b correctness gap; fixed inline)

`function_to_define_form_with_body` + `function_to_fn_form` (both
in `src/closure_extract.rs`) used `crate::check::format_type` to
render type annotations into emitted ASTs. `format_type` renders
`Tuple([])` as `:()` — the legacy unit-type form retired by arc
109 § 1d (`:wat::core::unit` was minted, then renamed to
`:wat::core::nil` per arc 153).

The child world's startup walker rejects bare `:()` (it fires
`BareLegacyUnitType`). When spawn-process's child re-freezes the
prologue + evals entry_form, any nil-returning fn fails to freeze
because its signature emit contained `:()`.

Surfaced via T4 (keyword-path-input spawn-process where fn returns
`:wat::core::nil`).

Fix: new `format_type_for_emit` / `format_type_for_emit_inner`
helpers in `closure_extract.rs` that preserve
`Tuple([]) → :wat::core::nil` round-trip. Slice 1b's API shape
unchanged; emit-side behavior corrected for the substrate's
current canonical form.

This is a real slice 1b correctness gap that slice 1b's tests
didn't exercise (slice 1b verified extracted forms re-freeze in a
fresh world but didn't path through the user-source pre-pass +
walker scoping; slice 2 does because the child uses the exact same
freeze pipeline as production user code).

Per `feedback_attack_foundation_cracks.md`: when a crack surfaces,
fix is also diagnostic. Applied + measured seal (T4 + downstream
tests now pass). SCORE-SLICE-1B stays as historical record per
`feedback_inscription_immutable.md`.

### Delta B — `walk_free_symbols` doesn't track match-arm pattern bindings (slice 1b territory; worked around)

Names introduced by `(:wat::core::Some n)` / similar patterns in
match arms surface as free symbols in arm bodies during slice 1b's
free-symbol walker. This blocks match-driven spawn-process bodies
from extraction — the walker thinks the pattern-bound names are
external dependencies that must be in `prologue`.

Worked around in T4-T7 by using nested `Result/expect` +
`Option/expect` (valid scrutinee positions per arc 110 §
CommCallOutOfPosition). These produce equivalent extraction
behavior without exercising the match-arm path.

Root-cause fix is slice 1b's territory — walker needs to extend
scope tracking to include match-arm pattern bindings. Surfaced
for future-slice disposition (could be a follow-up to slice 1b
or rolled into slice 4 substrate cleanup if the pattern surfaces
in real consumers).

Not load-blocking for arc 170; user code that needs match in
spawn-process bodies has an idiomatic workaround.

### Delta C — `spawn_process` child does NOT dup2 stdin/stdout (intentional design)

Slice 1c's typed-channel substrate gives the child PipeFd Sender +
Receiver via `OwnedFd::from_raw_fd`. The fn receives these handles
DIRECTLY rather than via dup2-redirected stdin/stdout. Only stderr
is dup2'd (for panic-marker emission via existing arc 113 cascade).

Process Value's legacy stdin/stdout/stderr fields are populated
with byte-pipe handles wrapping the SAME underlying fds the
typed-channel handles use. This mirrors slice 1c's
`fork-program-ast` pathway for the legacy-field placeholders —
matches BRIEF row F option (a) ("byte-pipe handles match slice 1c's
construction pattern").

Slice 4 retires the legacy fields; the typed-channel handles
become the canonical surface.

### Delta D — `spawn_process` panic-payload chain emit simpler than fork.rs

`spawn_process.rs` uses a "panic: spawn-process body panicked"
stderr marker for child panics. Full arc 113 cascade-chain emit
(reading the AssertionPayload + structured cause-chain) is
fork.rs's existing infrastructure; spawn_process didn't replicate
it because the slice 2 contract is "spawn-process works
end-to-end + walkers fire," not "spawn-process matches fork.rs's
arc 113 cascade fidelity."

Acceptable for slice 2's contract. If future consumers need
deeper panic cascade through spawn-process, that's an extension
arc — not arc 170 territory.

### Delta E — `BareLegacyMainSignature` walker scoped to legacy SIO triple specifically

The walker fires only on 3-arg main with type signature
`(IOReader, IOWriter, IOWriter)` — the well-known pre-arc-170
contract shape. Does NOT fire on:
- 0-arg `:user::main -> :wat::core::nil` (closure_extract's tests
  use this; they predate the contract)
- 1-arg / 2-arg `:user::main` variants
- 3-arg `:user::main` with non-SIO type signatures

This narrows the diagnostic to the well-defined migration class
without false-positiving arbitrary signature mismatches. Future
walker variants (if wat ever ships another `:user::main` contract
shape) can layer on top.

## Workspace fail breakdown (slice 3's sweep input)

Total: **545 failures**. All trace to walker firings on user-source
legacy patterns.

**268 deftest_* failures** — `wat/test.wat`'s `:wat::test::deftest`
macro generates legacy 3-arg `:user::main` shape inside
`(:wat::core::forms ...)`. Macro expansion lands those forms in
`expanded_user`, so `BareLegacyMainSignature` walker fires per
`freeze.rs:599-619` user-source-only scoping (which treats
`expanded_user` as user source). Slice 3 rebuilds the deftest
macro to emit 4-arg ExitCode-returning main (or migrates to a
different test-harness shape entirely per the testing-lib
three-layer rebuild).

**277 other failures**:
- Tests embedding legacy 3-arg `:user::main` source fixtures
  (`BareLegacyMainSignature` fires)
- Tests calling `:wat::kernel::fork-program{,_ast}` from user
  source (`BareLegacyForkProgram` fires)
- Tests calling `:wat::kernel::spawn-program{,_ast}` from user
  source (`BareLegacySpawnProgram` fires)
- 1 lib test (`runtime::tests::assert_eq_failure_renders_actual_and_expected`)
  with embedded 3-arg main fixture
- 2 `arc112_*` typed-channel scheme probes that depended on
  legacy main shape

Stdlib paths silently survive per walker scoping:
`wat/std/sandbox.wat` + `wat/std/hermetic.wat` use legacy verbs
internally; both freeze through `register_stdlib_*` which doesn't
trigger user-source walker pre-pass. They continue working
through legacy dispatch arms during sweep window. Slice 3
rebuilds them on typed-channel API; slice 4 destructively retires
legacy verbs + Process legacy fields.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 90-180 min opus | ~180 min | A clean (upper end of band) |

Within band. Calibration data: substrate-consumer-with-3-walker-
variants-and-fn-Value-to-typed-channels = upper end of 90-180 min
opus, comparable to slice 1's 150 min pattern.

Subsystems built:
- ExitCode typealias: 1 small wat file
- :user::main signature update + validator: ~20 lines src/freeze.rs
- spawn-process verb dispatch + child invocation: NEW src/spawn_process.rs
- 3 walker variants + Display + Diagnostic + bodies + scheme: ~300 lines src/check.rs
- wat-cli argv + ExitCode plumbing: ~30 lines crates/wat-cli/src/lib.rs + src/fork.rs
- closure_extract :wat::core::nil emit fix (delta A): ~50 lines src/closure_extract.rs
- 11 integration tests → 15 sub-cases: ~600 lines tests/wat_arc170_program_contracts.rs

Honest deltas surfaced: 5 (A through E above).

## Discipline check

- ✓ FM 5 held — 5 honest deltas surfaced cleanly; delta A fixed
  inline (load-bearing for slice 2 correctness; slice 1b
  correctness gap); delta B worked around in tests + surfaced
  for future disposition; deltas C/D/E surfaced as design
  decisions with reasoning
- ✓ FM 9 honored — local cargo test verified post-spawn:
  15/15 program_contracts, 17/17 typed_channel_pipes (slice 1c
  regression), 15/15 closure_extraction (slice 1b regression),
  1594/545 workspace (matches expected RED stream)
- ✓ FM 10 — no type-system reach; walker variants + spawn-process
  arm + closure_extract :nil fix all use existing entity kinds
- ✓ FM 11 — pre-INSCRIPTION grep deferred to slice 5 closure
- ✓ FM 12 — Agent spawn included `model: "opus"` explicitly
- ✓ FM 16 honored — BRIEF didn't mention Bash/cargo availability
- ✓ Branch isolation held — main untouched
- ✓ SCORE-SLICE-1/1B/1C untouched per `feedback_inscription_immutable.md`
- ✓ Bandaid-bounded-by-arc-close discipline honored — Process
  legacy fields populated as bandaid (matches slice 1c additive
  shape); slice 4 retires per DESIGN.md slice plan

## What's next

Foundation + wat-level surface complete. Remaining slices:

- **Slice 3 — consumer sweep + testing-lib three-layer rebuild**
  Mechanical sweep of 545 failures back to green:
  - `wat/test.wat` deftest macro rebuild (268 deftest_* failures)
  - Test fixtures with 3-arg `:user::main` → 4-arg
  - Test fixtures using `fork-program*` / `spawn-program*` →
    `spawn-process(fn)` / `spawn-thread(fn)`
  - `wat/std/sandbox.wat` + `wat/std/hermetic.wat` rebuild on
    typed-channel API (testing-lib three-layer Layer 1/2/3 polish)
  - Sonnet for mechanical sweep; orchestrator for testing-lib
    rebuild design choices
- **Slice 4 — substrate retirement (opus + sonnet pair)**
  Bandaid retirement before INSCRIPTION:
  - Process<I,O> legacy 3 fields (stdin, stdout, stderr) drop
  - Walker bodies retire (their work is done — no remaining
    legacy callers in user-source post-slice-3)
  - Legacy dispatch arms retire (eval_kernel_fork_program*,
    eval_kernel_spawn_program*)
  - Atomic-commit pattern: opus destructive (don't commit) →
    sonnet sweep (don't commit) → orchestrator commits both as
    ONE atomic commit when workspace = 0-failed
- **Slice 5 — closure paperwork**
  - SCORE-SLICE-1, 1B, 1C, 2, 3, 4 already exist; slice 5 might
    coalesce into a single INSCRIPTION
  - INSCRIPTION reflects final clean shape (no bandaids; no
    deferral language per FM 11)
  - 058 changelog row (lab repo)
  - USER-GUIDE update
  - ZERO-MUTEX cross-ref
  - CONVENTIONS doc update
  - Atomic squash-merge to main → arc 109 v1 milestone closure
    unblocks (arc 170 was tracked in INVENTORY § J's territory
    for the spawn family work)

## What this slice proved

Substrate-as-teacher pattern works end-to-end at scale:
- Substrate ships → workspace breaks deterministically (545
  failures, all traced to walker firings on user-source legacy
  patterns)
- The breaks ARE the migration brief — slice 3 sweeps category
  by category until 0 failed
- Stdlib silently survives through walker scoping (sandbox.wat,
  hermetic.wat untouched)
- The discipline scales across slices: 1 → 1b → 1c → 2 each
  shipped Mode A clean via fresh-agent execution against
  accumulated artifacts; 5 substantial substrate-fit decisions
  made by agents through investigation + reasoning + FM 5
  discipline

The pattern: opus lands platform changes; substrate-as-teacher
emits failures; slice N+1 sweeps. Arc-discipline pipeline
delivers IMPECCABLE foundation per recovery doc § 12.

Slice 3 next — the polish slice; testing-lib three-layer rebuild
is where the user-visible UX collapse happens (Layer 1: write
just the body; macro hides ceremony).

## Companion docs

- BRIEF-SLICE-2.md + EXPECTATIONS-SLICE-2.md — REDRAFTED post-1c;
  this SCORE matches the redrafted scorecard
- TIERS.md — tier framework + three-layer testing API the
  testing-lib rebuild realizes in slice 3
- DESIGN.md slice 4 — Process legacy field retirement explicit
  per bandaid-bounded discipline
- REALIZATIONS-SLICE-1.md pass 6 — bandaid-bounded discipline
- SCORE-SLICE-1.md / 1B / 1C — immutable historical records
