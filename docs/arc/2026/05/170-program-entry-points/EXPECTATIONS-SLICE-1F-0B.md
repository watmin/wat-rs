# Arc 170 slice 1f-0b — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-90 minutes opus.**

Substrate reshape with design surface. Three new Rust enums
+ ThreadIO struct field-type changes + three eval-arm
modifications + 10 test-row migrations. Pattern fully specified
by BRIEF + pass 18; opus-tier for the naming-decision calls
and the test-migration mechanical consistency check.

Comparable to:
- Slice 1f-α (60-90 predicted; actual ~50 min) — similar shape
  but FROM scratch; this slice MODIFIES the existing shape so
  has the "don't break what works" overhead
- Arc 119 step 2 (~60-90 min) — similar substrate reshape with
  enum minting

**Hard cap: 180 minutes (3 hours).** Wakeup scheduled.

## Baseline (post-slice-1f-0a — commit `cfd55fd`)

- Workspace: **1328 passed / 854 failed**

Predicted post-slice-1f-0b:

- **1328 passed / 854 failed** (±5 band) — this slice doesn't
  add or remove tests; the 10 1f-α rows migrate but stay green;
  the rot baseline stays put (slice 1f-0a-ii / iii / iv address
  that)

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Three Event enums minted in `src/thread_io.rs` | grep finds each `pub enum` definition | ✓ |
| B — `ThreadId` typealias | `pub type ThreadId = i64;` (or newtype if elected) | ✓ |
| C — ThreadIO fields use Event-typed senders | grep finds `stdout_tx: Sender<StdOutServiceEvent>` + analogues | ✓ |
| D — `eval_kernel_println` constructs `StdOutServiceEvent::Write` | grep + visual review | ✓ |
| E — `eval_kernel_eprintln` constructs `StdErrServiceEvent::Write` | same | ✓ |
| F — `eval_kernel_readln` constructs `StdInServiceEvent::Read` | same | ✓ |
| G — `src/lib.rs` re-exports the Event enums | grep finds `pub use thread_io::...{StdInServiceEvent,...}` | ✓ |
| H — All 10 1f-α test rows pass | `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 10/10 | ✓ |
| I — `cargo check --release` green | no compile errors | ✓ |
| J — Workspace within ±5 band of post-1f-0a baseline | 1323-1333 passed / 849-859 failed | ✓ |
| K — Zero new dependencies | Cargo.toml unchanged | ✓ |
| L — Zero new Mutex / RwLock / CondVar | grep returns 0 hits | ✓ |
| M — Type-check arms unchanged | TypeScheme for println / eprintln / readln matches what slice 1f-α shipped | ✓ |
| N — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Field name `stdout_tx` vs `stdout_req_tx`** — naming call
- **ThreadId representation** — typealias vs newtype
- **`#[derive(Clone)]` on Event enums** — needed or not
- **`#[derive(Debug)]` for test assertions** — should be
  standard
- **Module location of Event enums** — in `src/thread_io.rs`
  or extracted
- **`Arc<HolonAST>` ownership on stdin reply field of Add
  variant** — verify it composes
- **Any consumer of the old field names** outside the slice's
  files — surface as honest delta; don't migrate unilaterally

If any honest delta requires scope expansion, STOP and
surface.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-0b: ___ passed / ___ failed
- Fail-count delta from post-1f-0a baseline: ___ (predicted: ±5)
- Pass-count delta: ___ (predicted: ±5)
- Honest deltas surfaced: ___
- Implementation choices: ThreadId rep ___, field-name choice
  ___, derives on Event enums ___

## What's next (orchestrator-side, post-slice-1f-0b)

When 1f-0b ships:

1. Verify ship criteria locally (cargo test green; scorecard
   pass)
2. Author SCORE-SLICE-1F-0B.md
3. Atomic commit slice 1f-0b
4. Author slice 1f-β-i-redux BRIEF + EXPECTATIONS —
   `wat/kernel/services/stdin.wat` wat-side StdInService
   implementing the unified Event protocol (now with concrete
   Rust Event types from 1f-0b to mirror)

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-18)
- [x] BRIEF-SLICE-1F-0B.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-0B.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 60-90 min predicted; 180 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files +
      lines
- [x] Verified each cited primitive exists (slice 1f-α shipped
      at fcaf600; pre-grep ran 2026-05-10)
- [x] No "STOP at first red" + impossible-task constraint —
      slice modifies existing shipped code with explicit
      before/after
- [x] Baseline established: post-1f-0a 1328/854
- [x] Will spawn with `model: "opus"` explicitly (substrate
      reshape with design surface)
- [x] Will spawn with `run_in_background: true`
- [x] Wakeup scheduled at 180 min (3 hours = 10800 s) hard cap

## SCORE artifact

Slice 1f-0b is the SECOND foundation slice (after 1f-0a)
preceding slice 1f-β-i-redux. SCORE-SLICE-1F-0B.md lands
beside this when the slice ships.
