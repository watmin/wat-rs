# EXPECTATIONS — every worker races its own timer

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"Route (a) costs the crash channel and route (b) is bigger than this stone"** is a full
delivery — the up-wire enum landed, the reach decision reported, the timer not yet wired. Say which
and why. ⚠ **Do not buy a stall bound by breaking death detection**, which is the one fault we can
already induce.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **The up wire gains variants** | An enum symmetric with `PoolMsg`: today's payload plus a tick a timer can deliver. Named for the wire, not the timer. ⚠ 32 sites — occurrences not lines; read `CLAUDE.md` on the codemod before hand-editing `.wat`. |
| 2 | ⭐ **Timer per runner, in the set** | Armed when the runner is handed work, carrying its index. A tick for *k* re-queues `holding[k]` via the existing `collect-requeue`. |
| 3 | ⭐ **The reach decision, with evidence** | (a) unified-`Peer` runners or (b) spawn-tier select accepts a timer. ⛔ **Read the crash-channel dependency first** and report what (a) would cost. A choice without that reading does not score. |
| 4 | ⭐ **Control by MUTATION** | N runners all stalling produces a bound, not a hang. Remove the timers → the test must **fail** (a timeout is a fail). Show both runs. |
| 5 | **Non-vacuity + margins** | The stall fired; the bound asserted as "at least N". Shared host, 5334 tests. |
| 6 | **Scope wall** | Hardening, suppression, thread tier, lineage-Admin, queue knobs, and the queue path's own stall test — all untouched. Say what you did not do. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/`, and **state the tree it ran against**. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **A stall bound that costs death detection.** Route (a)'s hazard; row 3 exists for it.
- **A tick that cannot be distinguished from a real reply** — then a timer looks like completed work
  and the item is lost silently, which is worse than the hang.
- **A timer armed at dispatch but never disarmed on a normal reply** — the set fills with stale
  ticks and every item gets re-queued once spuriously. ⭐ **Say explicitly how a tick that loses its
  race is discarded.**
- **`collect-requeue`'s existing guards bypassed.** It already refuses an idle runner, an item
  already in `pairs-acc`, and a double-queue. A tick path that re-queues directly re-opens the
  duplicate-dispatch hole.
- **A busy-wait instead of `after`.** There is no sleep intrinsic; do not add one.

## Deliverable

`SCORE.md`: the up-wire enum and its site count, the reach decision with the crash-channel reading,
the mutation evidence for row 4, the tick-loses-its-race argument, floor Summary + `.floor/` path +
the tree statement.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
