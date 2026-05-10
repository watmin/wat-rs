# Arc 170 slice 1f-0a — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 15-30 minutes sonnet.**

Mechanical macro edit. Two `defmacro` bodies (≈ 25 lines each)
in one file (`wat/test.wat`). Before/after shape is explicit
in the BRIEF. The work IS the edit + re-run cargo test.

Comparable to:
- Arc 162 (lambda → fn internal rename) — pure mechanical sweep;
  similar low-judgment shape
- Arc 144 slice 2 (special form registrations) — ~30 min;
  smaller scope of edit

**Hard cap: 60 minutes (1 hour).** Wakeup scheduled.

## Baseline (post-pass-18 — commit `7709d0f`)

- Workspace: **1327 passed / 855 failed** (slice 1f-α shipped
  state; pre-slice-1f-0a)

Predicted post-slice-1f-0a:

- **~2100-2200 passed / ~0-50 failed** (the 855 deftests
  unblock; ~+800 to pass count; fail count drops to near zero)

Verify the actual outcome at scoring time. If the fail-count
delta is small (e.g., -50 instead of -855), surface as honest
delta — the rot may be deeper than just the macro signature.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `:wat::test::deftest` emits `(:user::main -> :wat::core::nil)` | grep finds new shape; old 3-param shape gone | ✓ |
| B — `:wat::test::deftest-hermetic` emits `(:user::main -> :wat::core::nil)` | same | ✓ |
| C — `cargo check --release` green | no compile errors | ✓ |
| D — Workspace fail count drops dramatically | fail count ≤ 50 (predicted near 0; ideal 0) | ✓ |
| E — Pass count rises | ~+800 from 1327 baseline | ✓ |
| F — No previously-passing tests regress | the 1327 baseline holds; any regression is a hard fail | ✓ |
| G — slice 1f-α Rust tests still green | `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 | ✓ |
| H — Zero new dependencies | Cargo.toml unchanged | ✓ |
| I — Only `wat/test.wat` modified | `git diff --stat` shows one file | ✓ |
| J — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Fail-count drop smaller than predicted** — if the
  855-failure baseline doesn't fully clear, the macro fix
  uncovered other rot. Surface what's remaining.
- **New test failures introduced** — if the macro change
  causes previously-passing tests to fail (unlikely; only
  user::main signature changes), surface.
- **`wat-tests/*` files with manual `:user::main` defs not
  going through the macro** — surface if discovered.
- **`:wat::test::TestResult` or related types affected** —
  surface if relevant.

If any honest delta requires scope expansion, STOP and
surface — don't expand the slice unilaterally.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-0a: ___ passed / ___ failed
- Fail-count delta from post-pass-18 baseline: ___ (predicted: -800 to -855)
- Pass-count delta: ___ (predicted: +800)
- Honest deltas surfaced: ___

## What's next (orchestrator-side, post-slice-1f-0a)

When 1f-0a ships:

1. Verify ship criteria locally (workspace fail count near 0;
   scorecard pass)
2. Author SCORE-SLICE-1F-0A.md
3. Atomic commit slice 1f-0a (the one `wat/test.wat` change)
4. Author slice 1f-0b BRIEF + EXPECTATIONS —
   `src/thread_io.rs` ThreadIO + eval arms reshape to per-service
   Event enum (per pass 18). Opus-tier (substrate edit with
   design surface for the Rust-side Event enums).

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-18)
- [x] BRIEF-SLICE-1F-0A.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-0A.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 15-30 min predicted; 60 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files
      (wat/test.wat:305 + 336; src/freeze.rs:703-740)
- [x] Verified each cited primitive exists (pre-grep ran
      2026-05-10)
- [x] No "STOP at first red" + impossible-task constraint —
      the edit is mechanical; substrate accepts the target shape
- [x] Baseline established: 1327/855 post-pass-18
- [x] Will spawn with `model: "sonnet"` explicitly (mechanical
      macro edit; sonnet's wheelhouse)
- [x] Will spawn with `run_in_background: true`
- [x] Wakeup scheduled at 60 min (1 hour = 3600 s) hard cap

## SCORE artifact

Slice 1f-0a is the FIRST of the foundation slices that precede
slice 1f-β-i-redux. SCORE-SLICE-1F-0A.md lands beside this
when the slice ships.
