# Arc 170 slice 1e — EXPECTATIONS

## Independent prediction

**Predicted runtime: 60-120 minutes opus.**

Substrate edits across 4 well-defined files (`src/freeze.rs`,
`src/runtime.rs`, `src/check.rs`, `crates/wat-cli/src/lib.rs`)
plus `src/spawn_process.rs` + `wat/kernel/exit-code.wat`
deletion. The shape of each edit is settled per BUILD-PLAN +
DESIGN; the work is mechanical given the pre-grep verification
in BRIEF-SLICE-1E.md.

Comparable to:
- arc 109 slice 1d (mint :wat::core::unit; retire :() as type) —
  similar substrate-mint-and-retire pattern; ~90 min
- arc 167 slice 1 (fn-flat-signature) — similar
  validator-update + walker-update pattern; ~60 min

**Hard cap: 240 minutes** — wakeup scheduled.

## Baseline (post-foundation)

Foundation commit: `eb655d1` (phase A retirement + slice 1d
walker fix; clean working tree).

Baseline cargo test (measured 2026-05-10 against `eb655d1`):
- **1597 passed / 547 failed** across 124 suites
- The 547 failures are substrate-as-teacher input for revised
  slice 3 — tests expect the old 3-arg `:user::main` shape
  while the substrate enforces slice 2's 4-arg + ExitCode shape

Predicted post-slice-1e count: **+50 to +200 failures from
baseline** (slice 1e flips the substrate to `[] -> :nil`,
moving the signature mismatch shape; some 547 fails may
shift / new fails may appear). Range: 597-747 failed,
1397-1547 passed (rough — rev. slice 3 sweep is what collapses
the cumulative red).

This delta is EXPECTED. Substrate-as-teacher pattern (FM 15) —
fail count is the progress meter; don't panic at the number.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `:wat::runtime::argv` ambient | `(:wat::runtime::argv)` returns a `Vec<wat::core::String>` populated by `runtime::set_argv()`; type-check arm registers correctly | ✓ |
| B — `:wat::runtime::current-thread` ambient | `(:wat::runtime::current-thread)` returns the calling thread's id; works on the main thread (slice 1g extends to spawned threads) | ✓ |
| C — `expected_user_main_signature` returns `[] -> :wat::core::nil` | `src/freeze.rs:753` updated; old shape gone | ✓ |
| D — `validate_user_main_signature` enforces new shape | Diagnostic messages name `[] -> :wat::core::nil`; cite REALIZATIONS pass 10; param-count diagnostic is concise (no mention of stdio/argv params); return-type diagnostic names nil-as-success-marker | ✓ |
| E — `wat/kernel/exit-code.wat` deleted | `git status` shows D | ✓ |
| F — Zero ExitCode references in src/ + crates/ + wat/ (excluding docs/) | `grep -rn "wat::kernel::ExitCode" src/ crates/ wat/ \| grep -v ^docs/` returns nothing | ✓ |
| G — `invoke_user_main` simplified | Signature accepts FrozenWorld only OR keeps `args: Vec<Value>` with calls passing `Vec::new()`; both acceptable | ✓ |
| H — wat-cli plumbs argv into ambient | `crates/wat-cli/src/lib.rs:257`+ calls `runtime::set_argv(argv)` before `invoke_user_main`; `main_args` Vec construction retired (4 IOReader/IOWriter/Vec elements gone) | ✓ |
| I — wat-cli exit code mapping | nil-return → `ExitCode::from(0)`; existing panic propagation handles non-zero (slice 1i adds the cascade epilogue; slice 1e leaves the existing path) | ✓ |
| J — spawn-process child invocation simplified | `src/spawn_process.rs` child invocation no longer constructs stdio Values; child fn is `[] -> :wat::core::nil` | ✓ |
| K — Walker `BareLegacyMainSignature` updated | Walker fires on any `:user::main` shape that isn't `[] -> :wat::core::nil`; diagnostic names the new canonical shape | ✓ |
| L — New fixture test `tests/wat_arc170_slice_1e_user_main_nil.rs` | Three test cases: parses+freezes+invokes new shape; rejects old shape with diagnostic; `:wat::runtime::argv` accessible from main body. All green: `cargo test --release --test wat_arc170_slice_1e_user_main_nil` | ✓ |
| M — Workspace cargo test runs | `cargo test --release --workspace --no-fail-fast` produces a numeric result (pass/fail counts); fail count delta from baseline matches the +50/+200 prediction band; if outside band, surface as honest delta | ✓ |
| N — Honest deltas surfaced | per FM 5; no TODOs in source; no deferral language; substrate gaps surfaced explicitly | ✓ |
| O — Zero Mutex usage | no Mutex/RwLock/CondVar (uses OnceLock for ARGV per the static-atomic pattern in `src/runtime.rs:51-119`) | ✓ |
| P — Phase A + slice 1d work untouched | git diff shows slice 1e only edits the files BRIEF-SLICE-1E lists; phase A retirement files (`wat/std/sandbox.wat`, `wat/std/hermetic.wat` deletions) and slice 1d work (`src/closure_extract.rs` walker fix) remain at their `eb655d1` foundation state | ✓ |

## Honest delta categories

Surface promptly if encountered; don't workaround:

- **Eval-arm wiring detail** — if minting `:wat::runtime::argv`
  / `:wat::runtime::current-thread` reveals substrate gaps in
  the existing eval-dispatch framework (e.g., the two arms
  need plumbing the framework doesn't currently support), STOP
  and surface for design discussion. Don't ship a partial.
- **Walker tracking vocabulary** — if `BareLegacyMainSignature`
  walker doesn't have a clean way to express "anything-but-
  `[] -> :wat::core::nil`," surface for design discussion.
  Probably trivial (the existing 3-arg/4-arg detection extends),
  but flag if not.
- **spawn-process child path complexity** — if simplifying the
  child invocation reveals deeper coupling we missed (e.g.,
  child currently expects stdio Values via some indirection
  the BRIEF didn't name), surface before continuing.
- **wat-cli argv collection timing** — `std::env::args()` runs
  at process start; `runtime::set_argv` must run BEFORE
  `invoke_user_main` AND before any wat code can read
  `:wat::runtime::argv`. If the existing wat-cli structure
  makes this hard, surface the ordering constraint.
- **FM 5 trap** — TODOs verboten. If a corner case surfaces
  that's out-of-scope, surface as honest delta; don't write
  a TODO.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ minutes (Mode A clean / B partial / C failed)
- Workspace post-1e: ___ passed / ___ failed
- Fail-count delta from baseline: ___
- Whether delta lands inside +50/+200 band: ___
- Honest deltas surfaced: ___

## What's next (orchestrator-side, post-slice-1e)

When slice 1e ships:
1. Verify ship criteria locally (re-run scorecard rows that
   need orchestrator confirmation)
2. Author SCORE-SLICE-1E.md (calibration filled; row-by-row
   pass/fail; honest deltas captured)
3. Atomically commit slice 1e
4. Slice 1f BRIEF + EXPECTATIONS authored — three substrate
   services (StdIn/StdOut/StdErr); per BUILD-PLAN §3 slice 1f

## SCORE artifact

Slice 1e is an ATOMIC slice (no atomic-commit pair pattern
required — pure substrate edits, no consumer sweep dependency).
SCORE-SLICE-1E.md lands as a sibling of BRIEF/EXPECTATIONS.

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-13 captured)
- [x] BRIEF-SLICE-1E.md (this slice's BRIEF) authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1E.md authored + will-be-committed
- [x] Runtime band: 60-120 min predicted; 240 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact file:line
- [x] Verified each cited primitive exists
- [x] No "STOP at first red" + impossible-task constraint —
      this slice is achievable as scoped
- [x] Baseline test re-run executing in background
- [ ] Will spawn with `model: "opus"` explicitly (substrate
      work; not mechanical sweep)
- [ ] Will spawn with `run_in_background: true`
- [ ] Wakeup scheduled at 240 min (4 hours = 14400 s)
- [ ] Non-overlapping work queued (slice 1f BRIEF authoring,
      etc.) for the spawn window
