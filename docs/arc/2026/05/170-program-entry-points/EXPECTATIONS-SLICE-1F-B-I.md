# Arc 170 slice 1f-β-i — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-90 minutes opus + wat-author.**

Pattern derivation from `wat-tests/service-template.wat` +
adaptation for dynamic-membership (HashMap routing +
control-pipe Signal handler). New territory: this slice mints
the dynamic-membership service shape for the substrate. Future
stones (1f-β-ii, 1f-β-iii) apply the pattern mechanically.

Comparable to:
- Arc 119 step 2 (LRU enum-Request reshape) — similar enum +
  channel-family declaration + driver-loop reshape; took
  ~60-90 min.
- Arc 089 slice 2-3 (Service drain + per-batch dispatch) —
  similar service-shape work; ~90-120 min for two slices.

**Hard cap: 180 minutes (3 hours).** Wakeup scheduled.

## Baseline (post-slice-1f-α — commit `fcaf600`)

- Workspace: **1327 passed / 855 failed** (verified locally
  post-1f-α commit)

Slice 1f-β-i adds tests in `wat-tests/kernel/services/stdin.wat`
(predicted ≥ 4, ideally 6 rows). Predicted post-slice-1f-β-i:

- **~1331-1333 passed / ~855 failed** (within ±5 band)

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `wat/kernel/services/stdin.wat` exists | file present | ✓ |
| B — `wat-tests/kernel/services/stdin.wat` exists | file present | ✓ |
| C — `:wat::kernel::ThreadId` typealias declared | grep finds declaration | ✓ |
| D — `:wat::kernel::services::StdInService::Signal` enum declared | enum declaration with add + remove variants | ✓ |
| E — Service typealiases declared (ThreadReq / Routing / ControlRx / ControlTx) | grep finds canonical names | ✓ |
| F — `:wat::kernel::services::StdInService::spawn` defined | callable; returns tuple (Thread, ControlTx) | ✓ |
| G — Driver loop uses HashMap routing + TCO | grep finds canonical pattern; assoc/dissoc on routing | ✓ |
| H — Driver loop integrates control-pipe handler via select | grep finds select including control-rx in selector set | ✓ |
| I — `:wat::edn::read` used to parse fd-0 line to HolonAST | grep finds call site | ✓ |
| J — `:wat::io::IOReader/read-line` used to read fd-0 line | grep finds call site | ✓ |
| K — `src/stdlib.rs` registers `wat/kernel/services/stdin.wat` | grep + entry present | ✓ |
| L — `src/stdlib.rs` registers `wat-tests/kernel/services/stdin.wat` (if wat-tests files need registration) OR cargo test discovers it via wat::test! macro | verify via cargo test --list | ✓ |
| M — Test rows ≥ 4 pass (ideally 6) | `cargo test --release --workspace` includes new stdin.wat tests; all pass | ✓ |
| N — Workspace within ±5 band | post-1f-β-i fail count is 850-860 (baseline 855) | ✓ |
| O — `cargo check --release` green | no compile errors | ✓ |
| P — Zero new dependencies | Cargo.toml unchanged | ✓ |
| Q — Zero new Mutex / RwLock / CondVar | grep returns 0 hits in modified Rust files | ✓ |
| R — Slice 1f-α substrate primitives untouched | git diff fcaf600..HEAD shows no src/thread_io.rs / src/check.rs / src/runtime.rs / src/lib.rs changes beyond stdlib registration | ✓ |
| S — No slice 1f-γ / 1f-δ work | no spawn-thread modifications; no service-spawning runtime code | ✓ |
| T — Honest deltas surfaced | per FM 5; no scope expansion | ✓ |

**20 rows.**

## Honest delta categories

Surface promptly; don't work around:

- **ThreadId representation** — typealias vs newtype decision
- **ThreadId placement** — in stdin.wat or new wat/kernel/types.wat
- **Service-spawn signature shape** — IOReader + maybe other params
- **Routing-table typealias name** — Routing / Handles / etc.
- **`:wat::kernel::select` heterogeneous-set behavior** — if it
  doesn't accept (HashMap values + control-rx) cleanly, surface
- **In-memory IOReader for tests** — canonical pattern existence
- **`:wat::edn::read` return type** — wrapped vs unwrapped
  HolonAST
- **Routing-table iteration overhead** — building the select-set
  from HashMap values on every iteration; if the wat-side
  HashMap doesn't have an efficient `values` op, surface
- **Test fixture for in-memory IOReader** — if mock IOReader
  doesn't exist in wat's substrate, surface; tests may need to
  use `:wat::io::TempFile` or similar

If any surface as substantive substrate friction (scope
expansion required), STOP and surface — don't expand the slice
unilaterally.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-β-i: ___ passed / ___ failed
- Fail-count delta from post-1f-α baseline: ___ (band: ±5)
- Pass-count delta: ___ (predicted: +4 to +6)
- Honest deltas surfaced: ___
- Implementation choices: ThreadId ___, placement ___, spawn
  signature ___, routing name ___, test fixture ___

## What's next (orchestrator-side, post-slice-1f-β-i)

When 1f-β-i ships:
1. Verify ship criteria locally (cargo test green; scorecard pass)
2. Author SCORE-SLICE-1F-B-I.md
3. Atomic commit slice 1f-β-i (the wat files + stdlib.rs)
4. Author slice 1f-β-ii BRIEF + EXPECTATIONS —
   wat/kernel/services/stdout.wat applying the pattern from
   1f-β-i + adapted for output direction (caller-pre-serialized
   String + ack-only reply)

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-17)
- [x] BRIEF-SLICE-1F-B-I.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-B-I.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 60-90 min predicted; 180 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files
- [x] Verified each cited primitive exists (pre-grep ran
      2026-05-10 — see orchestrator's pre-flight in context)
- [x] No "STOP at first red" + impossible-task constraint —
      slice scopes match available substrate
- [x] Baseline established: post-1f-α workspace 1327/855
- [x] Will spawn with `model: "opus"` explicitly (substrate +
      wat-author work; design choice surface)
- [x] Will spawn with `run_in_background: true`
- [x] Wakeup scheduled at 180 min (3 hours = 10800 s) hard cap

## SCORE artifact

Slice 1f-β-i is the FIRST of three 1f-β stepping stones (β-i /
β-ii / β-iii). SCORE-SLICE-1F-B-I.md lands beside this when the
slice ships.
