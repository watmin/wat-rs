# EXPECTATIONS — D7

> ⛔ **A BOUNDED NEGATIVE IS A PASS, NOT A FAILURE.** D2 closed that way and the audit was the
> deliverable. What is NOT acceptable is an unexamined "looks fine".

## The scorecard

| # | what | state AT HEAD | required after |
|---|---|---|---|
| 1 | ★ reachability answered | **undetermined** | a trigger, **or** a named list of angles tried and their results |
| 2 | ★ the invariant no longer rests on one armed test | one `#[cfg(test)]` differential, one call site | an assertion at the write site, or a cure |
| 3 | the assertion is observed firing | — | mutation-proved by an artificial collision |
| 4 | hot-path cost stated | — | measured, and `debug`-only argued if chosen |
| 5 | neither writer reaped | both live | both live — STOP-4 |
| 6 | floor | 5336/5336 | ≥ 5,336, zero FAIL |
| 7 | lints | 210/210 | green |
| 8 | clippy | rc=0 | silent |

## What would make this strike a failure even if every test passes

**A cure with no demonstrated defect.** If the drive finds no trigger and a cure lands anyway, the
strike has changed the fire path on the strength of a shape nobody showed reachable — and paid
hot-path cost for it.

**And a drive that stops at one angle.** The DESIGN names four; a report that tried one and concluded
"not reachable" has not bounded anything. The negative's whole value is the enumeration behind it —
`[[a-proof-over-a-filtered-population-is-not-a-proof]]`.
