# EXPECTATIONS — the bracket peer path faces chaos

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"A chaos work-fn already reaches slow and oversize; only suppression is missing"** is a full
delivery — three probes and nothing built beyond the deadline knob. ⭐ **And "`Malformed` is
unreachable on this path" is the most valuable single sentence available here**, because it grades
a fix already committed (`c4026f99e`) as correct-but-latent. Say it plainly if it is true.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | ⭐ **The measurement, before any build** | One probe each for slow / oversize / die. For each, THREE facts: **which `ServiceEvent` reaches `collect-loop`**, **which arm runs**, and — per the DoS amendment — **did the bracket survive, and was the cost bounded (by what, to what)**. ⛔ Naming the arm alone no longer scores: an unbounded wait is a DoS even when nothing crashes, and a death is a worker taking down its coordinator. |
| 2 | **Oversize is not Malformed** | An oversized reply is `RecvError::FrameTooLarge` on the process-spawn branch, not a decode failure. Report the actual variant. ⛔ Conflating them fails this row. |
| 3 | ⭐ **`Malformed` reachability, answered** | Reachable or not, with the reason. If not: my `Malformed` re-dispatch stays source-pinned and the SCORE says so. ⛔ Do **not** invent a corruption injector to make a committed fix look exercised. |
| 4 | ⭐ **The bound fires for the first time** | A runner stalls past `collect-deadline-ms`; `collect-gave-up!` produces its wall-clock report. Injectable deadline via the `the-handshake-deadline-is-injectable` shape — **one home for the value**, production default unchanged. |
| 5 | **Non-vacuity** | Each probe proves its fault fired (the stall happened, the frame was oversized), the way the slow-peer stone reports `delays-fired`. A green that never faulted is this stone's failure mode. |
| 6 | **Scope wall** | Thread tier, lineage-Admin delay, backpressure/partition/half-close/corruption, queue knobs — untouched. Say what you did not do. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/`, and **state the tree it ran against** (empty `git diff HEAD`, or quote the dirty tree as the slow-peer strike correctly did). A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **A knob built for something the work-fn already does.** Row 1 exists for this; the excursus has
  twice spent a strike on unreachable or already-covered code.
- **A timing assertion without margin** — "at least N", never "exactly N". Shared host, 5329 tests.
- **Lowering the production `collect-deadline-ms`** to make a test affordable. Inject, don't lower.
- **A stall implemented as a busy-wait**, burning a core in the floor. `after` is the mechanism and
  there is no sleep intrinsic.
- ⚠ **Claiming the re-dispatch is fault-tested when only the arm was reached.** Row 4 is about the
  *bound*, which is a different code path from the re-queue.

## Deliverable

`SCORE.md`: the three-row measurement with arms named, the `Malformed` verdict, the first
`collect-gave-up!` produced by a real stall, fire evidence per probe, floor Summary + `.floor/`
path + the tree statement.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
