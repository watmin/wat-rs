# Arc 213 stone δ-1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 30-45 min Mode A. Smallest stone since β. Single field addition + one signature change + 3 mechanical caller updates.
- **LOC changed:** ~15-25 (field + constructor body + 3 caller line changes + doc comment updates)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** LOW. Field is additive; behavior unchanged; libc paths preserved as fallback.

## Honest-delta watch

### Risk 1 — Pidfd lifetime + Arc<ChildHandleInner> sharing

Pidfd wraps OwnedFd with Drop closing the fd. ChildHandleInner is wrapped in `Arc<ChildHandleInner>` (interior shared). The Pidfd is owned by ChildHandleInner; cloning the Arc doesn't clone the Pidfd. When the last Arc drops, ChildHandleInner drops, Pidfd::Drop fires, fd closes. Matches lifeline_w lifetime exactly.

No risk if Arc semantics are honored. Test it by running the wait_or_cached path (which currently uses self.pid via libc::waitpid) — should still work because pid field is still set.

### Risk 2 — Pidfd Send + Sync compatibility

ChildHandleInner already implements Send + Sync via:
- pid: pid_t (Send + Sync)
- reaped: AtomicBool (Send + Sync)
- cached_exit: OnceLock<i64> (Send + Sync)
- lifeline_w: Option<OwnedFd> (Send + Sync)

Adding pidfd: Pidfd: per arc 213 α SCORE, Pidfd is explicitly Send + Sync. Compatible.

### Risk 3 — Compiler error if Pidfd not imported in spawn_process.rs

Sonnet checks the existing `use crate::fork::{...}` line in spawn_process.rs. If `Pidfd` isn't included, add it. This is the one mechanical import-fix likely.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `pidfd: Pidfd` field added to ChildHandleInner (not Option; every site has one) | YES |
| 2 | `ChildHandleInner::new` signature changed from `(pid, lifeline_w)` to `(pidfd, lifeline_w)` | YES |
| 3 | `pid` extracted internally via `pidfd.pid()` | YES |
| 4 | `pub pid: libc::pid_t` field PRESERVED (libc paths still use it; δ-3 retires) | YES |
| 5 | Site 1 (`src/fork.rs:680` γ-1): `let pid = pidfd.pid()` removed; `ChildHandleInner::new(pidfd, ...)` passes Pidfd | YES |
| 6 | Site 2 (`src/fork.rs:1085` γ-2): same migration | YES |
| 7 | Site 3 (`src/spawn_process.rs:255` γ-3): same migration | YES |
| 8 | `wait_or_cached` UNCHANGED (still uses self.pid via libc::waitpid) | YES |
| 9 | `Drop::drop` UNCHANGED (still uses self.pid via libc::kill + libc::waitpid) | YES |
| 10 | `eval_kernel_wait_child` UNCHANGED (still uses handle.pid via libc::waitpid) | YES |
| 11 | cargo build --release clean (maybe Pidfd import added in spawn_process.rs) | YES |
| 12 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 13 | All baseline test binaries: post-count == pre-count | YES |
| 14 | Zero modifications outside `src/fork.rs` + `src/spawn_process.rs` | YES |
| 15 | SCORE inscribes Pidfd lifetime/Arc-share confirmation + any compiler-error surprises | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; substrate now stores Pidfd at every forked-child construction site
- **Mode B (acceptable):**
  - Pidfd lifetime / Arc interaction has an unexpected complication; REVERT + inscribe + return
  - A test fails in a way that surfaces unexpected behavior (e.g., Pidfd's Drop closes fd before final waitpid)
- **Mode C:** STOP rule broken (touched δ-2/δ-3 territory, modified wait paths, removed pid field, retired libc calls)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (additive field; behavior unchanged; pattern proven by α/β/γ). HIGH on first-attempt Mode A (smallest stone since β; no test should change behavior).
- **Risk factors:**
  - Pidfd import in spawn_process.rs (one-line mechanical fix if missing)
  - Pidfd's Drop firing too early if Arc-share semantics are misunderstood (unlikely; established Rust pattern)
- **Why this matters:** δ-1 is the substrate mint that enables δ-2 to migrate wait/kill paths to PID-reuse-safe pidfd methods. Without δ-1's field, δ-2 has no source for the pidfd. δ-1 is the foundation; δ-2 is the migration; δ-3 is the retirement.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Post-γ-3 tiebreaker (corrected with ε's honest audit):
- δ-1: 30-45 min bounded; pattern proven; substrate-canonical foundation for δ-2/δ-3
- ε (5 probe restructures): 4-7h total; per-probe non-mechanical (grandchild observation via pidfd_open or test architecture change)
- ζ (L2 module privacy): bigger; needs α-η first
- arc 212 ζ-1: multi-hour atomic-commit; independent of γ; could go now but ζ-7 verify gates on green workspace

Verdict: δ-1 wins on cadence + risk + tractability transfer (lays foundation δ-2/δ-3 build on; ε can mirror δ's pidfd-observation pattern when restructured).

After δ-1 ships → re-run tiebreaker on δ-2 vs ε vs ζ vs arc 212 ζ-1.

## Cross-references

- Arc 213 DESIGN — full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 α SCORE — Pidfd type + spawn_lifelined primitive
- Arc 213 β SCORE — run_in_fork migration (uses Pidfd transient)
- Arc 213 γ-1/γ-2/γ-3 SCOREs — three fork sites canonicalized (currently drop the Pidfd; δ-1 stores it)
- `src/fork.rs:184-201` — ChildHandleInner struct (δ-1's field addition target)
- `src/fork.rs:204-211` — ChildHandleInner::new (signature change target)
- `src/fork.rs:680` — γ-1 construction site
- `src/fork.rs:1085` — γ-2 construction site
- `src/spawn_process.rs:255` — γ-3 construction site
- `src/fork.rs:217-233` — wait_or_cached (δ-2's migration target; UNCHANGED in δ-1)
- `src/fork.rs:236-250` — ChildHandleInner Drop (δ-2's migration target; UNCHANGED in δ-1)
- `feedback_tractability_tiebreaker` — sequencing discipline
- `feedback_substrate_owns_not_callers_match` — doctrine: ChildHandleInner OWNS the pidfd
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — architectural commitment
