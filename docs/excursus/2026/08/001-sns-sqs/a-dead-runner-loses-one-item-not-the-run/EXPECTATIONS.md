# EXPECTATIONS — a dead runner loses one item, not the run

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"Only `Closed` is reachable, and re-dispatch needs a value the surface cannot carry" is a FULL
delivery** — row 1's evidence plus the blocked-on-surface statement, nothing landed. The previous
two stones in this area both over-claimed; a precise negative is worth more than a speculative fix.

⚠ But **"there are other raising arms elsewhere" is NOT a reason to stop.** `bracket.wat` holds 33
`assertion-failed!` sites and the stdlib 291. This stone owns `collect-loop`'s seven.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | ⭐ **Per-arm reachability IN A BRACKET POOL** | Seven arms, each answered with evidence: reachable, or structurally impossible and why. ⛔ Emitted-somewhere is not reachable-here — that conflation is what made the last stone chase dead code. An impossible arm keeps its `assertion-failed!` **and gains a comment saying it is a substrate violation, not a runtime failure.** |
| 2 | **Taxonomy placement** | Every reachable arm in exactly one of RETRY / REPORT-FINAL / REPORT-GONE. `Malformed` is never retried (inherited contract). Every bound wall-clock, and every report names which bound fired. |
| 3 | ⭐ **RETRY actually re-dispatches** | A killed runner's held item (`holding[idx]`) goes to a survivor and produces its `O`. All N results returned. |
| 4 | ⭐ **Control by MUTATION** | Remove the re-dispatch → the test goes RED. Show both runs. ⛔ A green-only control fails this row outright — that is exactly what `632335c55` had to delete. |
| 5 | **Non-vacuity** | Prove the runner really died in the green run (the fault fired), the way the queue control asserts `recv-drops>0`. |
| 6 | **Surface untouched** | `map` / `each` signatures byte-identical. If an honest treatment needs a richer return type, **stop and report** — do not widen it here. |
| 7 | **Scope wall** | The ten dead `RecvOutcome` arms, stone 1b, and the queue path all untouched. Say what you did not do. |
| 8 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/`. A red: do not re-run, capture whole, name the arm. Clippy counted over the WHOLE output. ⚠ Expect `no_loose_string_assert` and `no_inlined_wat_in_tests` to police new tests — fixtures, not inlined wat; runes only with a reason that earns it. |

## What would make this stone wrong

- **Re-dispatch that loops forever** when every runner dies — the bound is wall clock and it must
  name itself.
- **Re-dispatch that duplicates an item** — a runner that died *after* sending its result, whose
  `Closed` arrives later, must not have its item handed out again. ⭐ **This is the sharp edge: say
  explicitly how you distinguish "died holding it" from "died having finished it."**
- **`holding` read as authoritative when it is stale.** It is written on dispatch; prove the write
  ordering makes it correct at the moment `Closed` is observed.
- **A control that kills the runner so early it never held anything** — then re-dispatch is
  trivially unnecessary and the test proves nothing.

## Deliverable

`SCORE.md`: the seven-arm reachability table with evidence, the taxonomy placement, the mutation
evidence for row 4, the duplicate-dispatch argument, floor Summary + `.floor/` path.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
