# EXPECTATIONS — P2: mint `:wat::kernel::signal`, `Signal`, `SignalOutcome`

Written **before** the strike, so the result cannot move the goalposts.
Baseline floor, from the seam and to be re-confirmed by my own `--release` re-run at weigh time:
**4340 passed / 0 failed / 262 skipped.**

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **the RED probe turns GREEN** | `./target/release/wat docs/arc/2026/06/278-rules-engine/probes/red-owner-signals-child.wat` | runs to completion — **no** `UnknownFunction: :wat::kernel::signal`. ⚠ see row 2: it may legitimately fail a *different* way |
| 2 | **the must-use gate bites** | the probe's `_sig` binding is a `let`-`_` discard of a `SignalOutcome` | a **compile error** naming the dropped outcome. Rows 1 and 2 together mean: the unknown-function death is gone and a must-use death replaces it. **A silent pass on row 2 is a failed strike** |
| 3 | **both discard doors closed** | one `do`-non-final and one `let`-`_` case | both refused; neither compiles |
| 4 | **a faced call compiles and runs** | a `match` over `Delivered`/`Gone`/`Failed` | green, and the child actually observes the signal |
| 5 | **`Kill` reaches the owner-side observable** | send `Kill`, then read the child's close | `CloseOutcome::Signaled` — **the gate no handler can fake, because there is no handler** |
| 6 | **`SignalOutcome` is in `MUST_USE_TYPES`** | read `src/check.rs:7020-7024` | present, beside `CloseOutcome`; **not** in `MUST_USE_PARAMETRIC_HEADS` (it is non-parametric) |
| 7 | **no `kill(pid)` anywhere** | `grep -rn 'libc::kill' src/` | zero hits; delivery routes through `Pidfd::send_signal` |
| 8 | **the flags are untouched** | `git diff` on `KERNEL_SIGUSR1`/`SIGUSR2`/`SIGHUP` and their setters | no change. This strike adds a send path and changes nothing about observation |
| 9 | **`Drop` still owns the only reap** | `git diff src/process/handle.rs` | `Kill` does not reap; Drop's unconditional SIGKILL+reap path is unmodified |
| 10 | **the tier table shipped** | read the `Signal` enum's doc comment | the six-row who-observes-and-how table is present — it is the only home for the `Interrupt`/`Terminate` shared landing and `Kill`'s absent child-side observable |
| 11 | **the asymmetry is bridged** | read `CloseOutcome::Signaled` or `Signal` | a WHY line: closed enum on send, bare `i64` on receive, and why that is correct rather than sloppy |
| 12 | **floor** | `cargo nextest run --release` | **Summary line**, zero new failures vs 4340/0/262. Never a piped exit code |
| 13 | **clippy** | `cargo clippy` | clean |
| 14 | **STOP-2 reported** | the rider's own words | an explicit finding: is ESRCH reachable through a pidfd, and does a `cause` field earn its place? Either answer is acceptable; **silence is not** |

## Independent prediction

**Runtime: 25–45 minutes.** It is a contained mint — two enum registrations, one eval arm, one
dispatch entry, one `MUST_USE_TYPES` string — over a kernel primitive that already exists and is
already generic over the signal. The RED probe removes all discovery cost.

**Time-box: 90 minutes** (2× the upper bound).

## Trap doors — named in advance

- **Row 1 is not sufficient on its own, and this is the row most likely to produce a false green.**
  "The probe no longer dies on UnknownFunction" can be satisfied by a verb that exists and does
  nothing. Rows 4 and 5 are the ones that prove delivery: the child must actually *observe* the flag,
  and `Kill` must actually produce `Signaled` on the owner side.
- **`Gone` may not be mintable.** If ESRCH is not reachable through a pidfd, the honest shape is two
  arms plus a raise. That is a **pass**, not a shortfall — a mint that ships an unreachable arm has
  failed a STOP, not satisfied a row.
- **The staleness warning.** `target/release/wat` reported itself older than source during the probe
  run. Rebuild before trusting any binary-mediated row; a stale arbiter is the instrument supplying
  the result.
- **`--check` is not the arbiter for a missing head.** Measured with a positive control this session:
  it returns exit 0 on a verb that certainly does not exist. Any row about a head existing or not
  must go through the runtime.
- **The child's signal handlers must actually be installed** in whatever the spawn tooling produces.
  Grounded for the CLI path (`distribution/mod.rs:347`) and for spawned children
  (`spawned_runtime.rs:51`); if the `spawn-peer` test path differs, that is a STOP, not a workaround.

## How this will be scored

By my own re-run of every row, not the rider's report. Rows 1–5 are load-bearing; rows 12–13 are the
floor. A green on 12 with a silent 2 is the R59 failure repeating under a new mechanism — the whole
reason this stone exists is a signal test that passed while no signal was ever delivered.
