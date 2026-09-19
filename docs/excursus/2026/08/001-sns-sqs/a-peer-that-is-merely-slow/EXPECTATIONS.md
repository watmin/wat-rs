# EXPECTATIONS — a peer that is merely slow

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"The re-recv ruling does NOT hold under a real slow ack" is the most valuable possible
outcome** — a contract decided from reasoning, refuted by the first instrument able to test it.
Report it and land only the injector. Equally: "the injector works and the desync is not
reachable by it" is a full delivery if the reason is measured.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **The knob** | `delay-bp` + `delay-ms`, plumbed like its siblings in `wat-scripts/query/faulting-store.wat`. ⛔ Fire counter **at the delay site, never at the dice roll** — the exemplar's own header states this in capitals; a counter on the roll counts intentions. |
| 2 | ⭐ **The wait is a timer channel** | `(:wat::kernel::after …)`, exemplar `wat/queue.wat:1643`. ⛔ Not a busy-wait, not a blocking sleep — and there is no sleep intrinsic, so reaching for one means inventing it. A blocking wait stalls the very loop under test. |
| 3 | ⭐ **The surplus-ack desync, attempted** | Drive an ack that is *slow, not lost*. Report whether `the-gate-methods-face-an-outcome`'s re-recv ruling holds. Either answer scores; **silence does not.** |
| 4 | **One deadline fired by LATENESS** | Name which. Show a late reply differs observably from an absent one — do not assert it. |
| 5 | ⛔ **Non-vacuity** | Fire count from the delay site, plus the wall-clock difference. **A test that passes at `delay-bp=0` and at `delay-bp=10000` has measured nothing** — run both and show they differ. |
| 6 | **Scope wall** | Bracket-peer-path injectors, the thread tier, backpressure/partition/half-close/corruption all untouched. Say what you did not do. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/`. ⛔ **Verify `git diff HEAD` is empty before the run** and say so — a floor that tested a different tree than the commit is this session's most recent failure. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **A delay that never fires.** Row 5.
- **A delay implemented as a blocking wait**, stalling the loop it is meant to slow — the fault
  would then be the instrument's, not the peer's.
- **A flaky floor bought with the injector armed by default.** The knob defaults OFF; say so.
- **Timing assertions with no margin.** A delay of N ms must be asserted as "at least N", never
  "exactly N" — this host is shared and the floor runs 5326 tests in parallel.
- ⚠ **A green that only proves the timer works.** The subject is the SYSTEM under lateness, not
  `after`.

## Deliverable

`SCORE.md`: the knob, the fire counts at both rates, the desync attempt and its answer, which
deadline fired by lateness, floor Summary + `.floor/` path + the `git diff HEAD` check.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
