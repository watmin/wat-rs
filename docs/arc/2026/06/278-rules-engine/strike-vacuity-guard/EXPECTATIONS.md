# EXPECTATIONS — the count is what the instrument reports

> **Every row's command was run against HEAD and its pre-value recorded — except the population,
> which is deliberately left to the instrument (F0).**

## ⛔ NO PINNED TEST COUNT, AND NO PINNED POPULATION

**The floor must be ≥ 5,233 plus every arm you drive.** And unlike every previous scorecard, **this
one does not tell you how many gates lack a guard.** The work-list row says one number, my audit grep
said another, and mine is demonstrably wrong. **The lint's output is the answer.** Report it.

## The scorecard, with pre-values measured at HEAD `82029517a`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ **is any gate vacuous TODAY?** | **UNKNOWN — nobody has driven it** | driven and reported per gate. A zero is a **live defect** and STOP-1 |
| 2 | the lint exists | no such gate | walks `tests/lint/`, requires a declared guard |
| 3 | two shapes accepted | `no_new_broken_doc_link.rs:236` uses the non-obvious one | it is **not** falsely flagged — or is runed, with the choice stated |
| 4 | the lint is not vacuous itself | — | its own guard, **driven**: mutate its walk to find nothing → RED |
| 5 | runes declare a mechanism | — | every rune names what it does instead and what reds first; **no "N/A"** |
| 6 | radius | — | `tests/lint/` only; **no `src/` change** |
| 7 | lints | **119/119** (measured) | green |
| 8 | floor | **5233/5233** (measured) | ≥ 5,233, zero FAIL rows |
| 9 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **Remove a guard** from a gate that has one → the new lint REDs and names it.
2. **Add a hollow rune** (`— N/A`) → REDs, if the reason rule is enforced; if reasons are not
   enforced, say so plainly rather than implying they are.
3. **Blind the lint's own walk** → its self-guard REDs. *A vacuity lint that is itself vacuous is
   the joke this strike exists to prevent.*

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

60–80 minutes, and **row 1 may end it early** — if a gate is reaching zero, STOP-1 fires and the
lint waits.

## What would make this strike a failure even if every test passes

**Writing the lint without driving row 1.** The lint would then enforce a property nobody has
confirmed is currently held, and a gate that is *already* vacuous would get a guard asserting
whatever it happens to reach today — freezing the defect instead of finding it.

The second: **a syntactic guard check.** It would flag `no_new_broken_doc_link.rs`, which is correct,
and the natural fix — adding a fake `assert!(n > 0)` to satisfy the lint — makes that gate worse.
