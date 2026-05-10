# Arc 170 slice 1f-i — EXPECTATIONS

## Independent prediction

**Predicted runtime: 90-150 minutes opus.**

The pattern (always-on substrate service with per-thread
registration + libc::poll select-loop) is novel for wat-rs.
The substrate has libc usage precedent (`src/fork.rs`,
`src/spawn_process.rs`) and crossbeam usage precedent
(throughout `src/runtime.rs`), but the COMBINATION
(crossbeam-typed-channel-out + libc-poll-multiplex-in) is new.
Budget reflects pattern-design + implementation + tests.

Comparable to:
- arc 089 slice 2 (Service/loop drains all clients) — similar
  service-loop with crossbeam fan-in; shipped ~120 min
- arc 103a (spawn primitive + Process struct + thread driver)
  — similar libc + crossbeam coordination; shipped ~150 min

**Hard cap: 300 minutes** — wakeup scheduled.

## Baseline (post-slice-1e)

Slice 1f-i depends on slice 1e shipping. Pre-spawn, this
EXPECTATIONS doc cites:

- Foundation baseline (post-1e UNKNOWN until 1e SCOREs):
  approximately 597-747 failed (per slice 1e EXPECTATIONS
  prediction band)
- Slice 1f-i adds NEW files; doesn't touch existing surfaces;
  expected delta from post-1e baseline is ~0 (services are
  parallel infrastructure)
- Lock the post-1e baseline number into this doc when 1e
  SCOREs

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Module structure | `src/services/mod.rs` exists; `src/services/stdin.rs` exists; `src/lib.rs` exposes `pub mod services;` | ✓ |
| B — `start_stdin_service` idempotent | First call spawns thread; second call returns same `&'static StdInServiceHandle` (no double-spawn) | ✓ |
| C — Service thread spawns + idles | Thread runs without panicking when fd 0 has no data; CPU usage stays near zero (poll blocks, not busy-wait) | ✓ |
| D — Registration roundtrip | `handle.register(thread_id)` returns crossbeam `Receiver<Option<HolonAST>>`; `handle.unregister(thread_id)` drops the channel cleanly | ✓ |
| E — Single-line EDN parsing | bytes `"42\n"` → consumer receives `Some(HolonAST::leaf_int(42))` (or equivalent) | ✓ |
| F — Multi-line ordered dispatch | bytes `"1\n2\n3\n"` → consumer receives Some(1), Some(2), Some(3) in order | ✓ |
| G — EOF propagates :None | fd 0 close → consumer receives `None`; subsequent reads return `None` | ✓ |
| H — Self-pipe trick verified | test interleaves "data write to fd, control-msg, data write" → poll wakes both; processed in order | ✓ |
| I — Zero Mutex/RwLock/CondVar | `grep -nE 'Mutex\|RwLock\|CondVar' src/services/stdin.rs` returns 0 hits | ✓ |
| J — libc::poll used directly | no `mio` / `tokio` / `async-std` added to `Cargo.toml`; `grep "use libc" src/services/stdin.rs` returns at least one hit | ✓ |
| K — Rust integration tests green | `cargo test --release --test services_stdin` passes all rows | ✓ |
| L — Workspace doesn't regress | `cargo test --release --workspace --no-fail-fast` fail-count is within 5 of post-1e baseline (slice 1f-i is parallel infrastructure; existing tests don't see the service) | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs in source; no deferral language; substrate friction surfaced | ✓ |
| N — Zero new dependencies | `Cargo.toml` unchanged | ✓ |
| O — Foundation + slice 1e files untouched | git diff shows 1f-i only adds: `src/services/mod.rs`, `src/services/stdin.rs`, `tests/services_stdin.rs` (or similar); + 1-line edit to `src/lib.rs` for the `pub mod services;` re-export | ✓ |
| P — Registration API documented for 1f-ii reuse | The pattern's public API + invariants are clearly documented in module-level rustdoc on `src/services/mod.rs` so 1f-ii applies the same shape | ✓ |

## Honest delta categories

Surface promptly; don't workaround:

- **Service-thread lifecycle precedent** — wat-rs may not have
  an "always-on background thread spawned at boot" pattern;
  if introducing one creates ordering / shutdown issues,
  surface for design discussion. The Console crossbeam thread
  in `wat/console.wat` is the conceptual ancestor but lives at
  the wat layer.
- **fd 0 ownership conflict with wat-cli** — wat-cli's
  `spawn_stdin_proxy` (`crates/wat-cli/src/lib.rs:391`) reads
  stdin for the spawn-process child. Slice 1f-i's StdInService
  also wants fd 0. Slice 1e probably reshapes wat-cli's stdio
  handling (deletes the proxy?); re-grep wat-cli at slice
  1f-i start to confirm fd 0 is uncontested.
- **EDN line-buffering edge cases** — wat-edn parses one EDN
  value per line per arc 092; if a value spans newlines (a
  list with embedded newlines), the line-delimited assumption
  breaks. Verify wat-edn's actual behavior; surface
  surprises.
- **Test-fd parameterization** — if `start_stdin_service(fd)`
  parameterization isn't ergonomic, propose alternatives in
  honest delta. Don't ship a Mutex-protected fd 0 swap-out.
- **FM 5 trap** — TODOs verboten. If a corner case surfaces
  that's out of scope, surface as honest delta; don't write
  a TODO.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ minutes (Mode A clean / B partial / C failed)
- Workspace post-1f-i: ___ passed / ___ failed
- Fail-count delta from post-1e baseline: ___
- Whether delta lands inside ±5 band: ___
- Honest deltas surfaced: ___
- Pattern API documented for 1f-ii reuse: ___ (link to rustdoc)

## What's next (orchestrator-side, post-slice-1f-i)

When 1f-i ships:
1. Verify ship criteria locally
2. Author SCORE-SLICE-1F-I.md (calibration filled; honest
   deltas captured)
3. Atomic commit slice 1f-i
4. Read the registration API rustdoc; lock its shape into
   slice 1f-ii's BRIEF
5. Author BRIEF-SLICE-1F-II.md + EXPECTATIONS-SLICE-1F-II.md
   (StdOutService — applies the pattern; faster)
6. Spawn 1f-ii (after waiting for slice 1e + 1f-i to ship)

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-13)
- [x] BRIEF-SLICE-1F-I.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-I.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 90-150 min predicted; 300 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files
- [x] Verified each cited primitive exists (libc, crossbeam,
      wat-edn, KERNEL_STOPPED pattern)
- [x] No "STOP at first red" + impossible-task constraint
- [ ] Will spawn with `model: "opus"` explicitly (substrate
      pattern-design; not mechanical)
- [ ] Will spawn with `run_in_background: true`
- [ ] Wakeup scheduled at 300 min (5 hours = 18000 s)
- [ ] Slice 1e SCOREd + atomic-committed FIRST (1f-i depends
      on 1e's foundation)

## SCORE artifact

Slice 1f-i is one of four stepping-stones in slice 1f. SCORE-SLICE-1F-I.md
lands beside this; SCORE-SLICE-1F-II.md, SCORE-SLICE-1F-III.md,
SCORE-SLICE-1F-IV.md follow as their stepping stones ship.
