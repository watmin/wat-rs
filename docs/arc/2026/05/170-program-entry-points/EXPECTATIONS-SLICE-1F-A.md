# Arc 170 slice 1f-α — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-90 minutes opus.**

Pattern fit is tight. The eval-arm pattern is well-established
(`eval_edn_write` template); the thread-local pattern has one
precedent already; ThreadIO struct is mechanical; tests follow
the slice 1e fixture style with a tester-thread playing service.
No novel substrate machinery — this slice composes existing
primitives.

Comparable to:

- Slice 1f-i (Rust singleton; ~30 min) — similar substrate eval-arm
  + thread-local pattern, but had more novel pieces (libc::poll,
  self-pipe). Slice 1f-α is simpler — no fd machinery; just
  crossbeam channels + a thread_local! cell.
- Slice 1e (ambient runtime; predicted 60-90 min, also opus) —
  similar slice character.

**Hard cap: 180 minutes (3 hours).** Wakeup scheduled.

## Baseline (post-commit `134749a` — slice 1f-i + 1f-ii deletion)

Pre-slice-1f-α workspace: ~unchanged from post-1f-W (1329 passed
/ 855 failed) MINUS the 12 tests in `tests/services_stdin.rs`
that the deletion removed. Approximate baseline:

- **~1317 passed / ~855 failed**
  (subtract 12 from 1329; fail count unchanged because the
  deleted tests were all in the passing set)

Slice 1f-α adds 10 new tests (rows A through J). Predicted
post-slice-1f-α:

- **~1327 passed / ~855 failed** (within ±5 band)

Verify the actual baseline at scoring time via:

```
cargo test --release --workspace --no-fail-fast 2>&1 | tail -5
```

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `:wat::kernel::println` arm exists | grep `eval_kernel_println` in `src/runtime.rs` or `src/edn_shim.rs` | ✓ |
| B — `:wat::kernel::eprintln` arm exists | grep `eval_kernel_eprintln` | ✓ |
| C — `:wat::kernel::readln` arm exists | grep `eval_kernel_readln` | ✓ |
| D — `ThreadIO` struct + `thread_local!` cell exist | grep `pub struct ThreadIO` + `THREAD_IO` thread_local | ✓ |
| E — `install_thread_io` + `uninstall_thread_io` setters exist | callable from tests | ✓ |
| F — `RuntimeError::ServiceNotRunning` variant exists | grep the enum in `src/runtime.rs` | ✓ |
| G — Type-check arms registered for all three primitives | `(:wat::kernel::println 42)` parses + type-checks; same for eprintln + readln | ✓ |
| H — Unpopulated path returns clean error (no panic, no UB) | tests A/B/C in test fixture pass | ✓ |
| I — Populated println sends serialized String + returns nil | test D passes | ✓ |
| J — Populated eprintln sends serialized String + returns nil | test E passes | ✓ |
| K — Populated readln blocks + returns received form | test F passes | ✓ |
| L — Polymorphic value types serialize correctly | test G passes (i64, String, bool, tuple, struct each EDN-render correctly) | ✓ |
| M — Type-check accepts any-T for println / eprintln | tests H + I pass | ✓ |
| N — Type-check infers HolonAST return for readln | test J passes | ✓ |
| O — `tests/wat_arc170_slice_1f_alpha_helpers.rs` cargo test green | `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 pass | ✓ |
| P — Workspace doesn't regress | post-1f-α fail count is within ±5 of post-1f-deletion baseline | ✓ |
| Q — `cargo check --release` green | no compile errors | ✓ |
| R — Zero new dependencies | `Cargo.toml` unchanged | ✓ |
| S — Zero new Mutex / RwLock / CondVar | grep returns 0 hits in modified files (only doc-mentions OK) | ✓ |
| T — Honest deltas surfaced | per FM 5; if any decision required scope expansion, surface — don't work around | ✓ |
| U — Slice 1f-i + 1f-ii artifacts untouched | git diff shows only the new files + RuntimeError extension + check.rs registrations + lib.rs (no edit needed; module already declared elsewhere) | ✓ |

**21 rows.**

## Honest delta categories

Surface promptly; don't work around:

- **eval-arm registration site convention** — `src/runtime.rs`
  uses some pattern (match arm in `eval`, registry table, macro,
  etc.). Follow whatever `:wat::edn::write` does. If the pattern
  is unclear, surface.
- **Type-check arm registration site** — same. Follow the
  `:wat::edn::write` precedent in `src/check.rs`.
- **`Value::Nil` variant name** — if the actual nil value is
  `Value::Unit` or `Value::Holon(HolonAST::Nil)` or similar, use
  what the substrate already defines.
- **`Arc<HolonAST>` vs raw `HolonAST` ownership** on the stdin
  reply channel — surface the choice + reasoning.
- **`RuntimeError::ChannelDisconnected`** — if this variant
  doesn't exist yet, add it OR reuse an existing send/recv error
  per arc 111's discipline. Don't invent a new error category if
  one fits.
- **Module placement** — ThreadIO + the eval arms can live in
  `src/runtime.rs` (alongside the existing thread_local) OR a
  new `src/thread_io.rs` if cleaner. Surface the choice.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-α: ___ passed / ___ failed
- Fail-count delta from post-1f-deletion baseline: ___ (band: ±5)
- Pass-count delta: ___ (predicted: +10)
- Honest deltas surfaced: ___
- Implementation choices: module placement ___, error variants
  ___, eval-arm registration pattern ___

## What's next (orchestrator-side, post-slice-1f-α)

When 1f-α ships:

1. Verify ship criteria locally (cargo test green;
   row-by-row scorecard pass)
2. Author SCORE-SLICE-1F-A.md
3. Atomic commit slice 1f-α
4. Author slice 1f-β BRIEF + EXPECTATIONS — wat-side service
   implementations (`wat/kernel/services/{stdin,stdout,stderr}.wat`)
   in canonical service-template + Signal::add/remove handler
   shape

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-16)
- [x] BRIEF-SLICE-1F-A.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-A.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 60-90 min predicted; 180 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files
      (value_to_edn_with at line 954, eval_edn_write at line 72,
      require_one_arg at line 108, thread_local! precedent at
      14304)
- [x] Verified each cited primitive exists (pre-grep ran
      2026-05-10 — see commit 134749a context)
- [x] No "STOP at first red" + impossible-task constraint —
      this slice composes existing substrate; no gaps
- [x] Baseline established: post-1f-deletion (~1317 passed /
      ~855 failed; verify at scoring time)
- [x] Will spawn with `model: "opus"` explicitly (substrate
      work; design choice surface)
- [x] Will spawn with `run_in_background: true`
- [x] Wakeup scheduled at 180 min (3 hours = 10800 s) hard cap

## SCORE artifact

Slice 1f-α is the FIRST of five 1f stepping stones (α / β / γ /
δ / ε). SCORE-SLICE-1F-A.md lands beside this when the slice ships.
