# EXPECTATIONS — no stdlib body names a user-declarable name

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## ⛔ The null is a complete outcome

If the evaluated-position census is **non-zero**, the correct delivery is: **the names, their
sites, the channel each arrived by, and nothing landed.** That refutes step C a third time and is
worth more than a gate written around a number you wished for. **Do not repair the corpus to make
a gate pass** — that is a different stone with a different floor, and `.wat` is never hand-edited.

Equally: if the fresh sweep control (row 3) turns out to be underivable without changing `src/`,
say so and say why. "I could not build this without crossing the scope wall" is a finding.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **The census, by channel** | Four counts — own path / signature / evaluated body leaf / quoted body leaf — each with its names. Taken **from the AST**, never from a grep over `wat/*.wat`. ⛔ A number without the names behind it does not score. |
| 2 | **The gate, on the property** | Message names the violating **file, line, name, enclosing fn**, and what a violation *means*. Gates the invariant, not a path list. Reuses `quote_boundary` — **a fifth hand-rolled copy of the quote boundary fails this row outright.** |
| 3 | ⭐ **The fresh 8c control** | Must distinguish *"the sweep ran and found nothing"* from *"the sweep did not run."* State plainly how it tells those apart. A control that would stay green if `src/check.rs:750` were deleted has **not** discharged the demand. |
| 4 | **Non-vacuity, per test** | Every new assertion proves its premise held (the world froze, the stdlib loaded, the walk reached bodies). The existing file's `fns > 1000` check is the exemplar — cite it, mirror it. |
| 5 | **The stale justification** | `tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs:215` says what is now true, and the two-name assertion still stands. ⛔ Deleting it fails this row. |
| 6 | **Scope wall** | No `src/` behaviour change, nothing elided. A visibility widening to reach `quote_boundary` from a test is permitted **and must be named** in the SCORE with its justification. |
| 7 | **Floor** | `scripts/floor.sh`, release. Paste the **Summary line** verbatim and the `.floor/<stamp>/` path. ⛔ A red: do not re-run, capture whole, name the exact arm, surface it. There is no known flake. |

## What would make this stone wrong

State, in the SCORE, which of these you checked and what you found:

- **The evaluated surface is zero for the wrong reason** — e.g. the walk never descends into
  bodies at all, or `FunctionBody::Wat` matched nothing. Row 4 exists for this.
- **`quote_boundary` exempts more than the walker does.** The walker treats only `AllData` and
  `Quasiquote` as data regions and keeps arc-198 width for `Match` / `MatchesSubject` / `MakeRule`
  / `Ordinary` (`src/check.rs:1701`). A census that exempts all six would under-count and
  manufacture a zero. ⛔ **Mirror the walker's four-way split exactly, and say that you did.**
- **The unquote escape.** `~x` / `~@x` inside a quasiquote template IS evaluated. A census that
  exempts whole templates reopens the hole the previous stone closed.
- **Symbols vs Keywords.** The existing walk notes both (`WatAST::Symbol` too). Say which the
  position-aware census counts and why.

## Deliverable

`SCORE.md` beside this file: the four-channel census with names, what landed, the floor's Summary
line and `.floor/` path, and a plain statement of whether row 3's control would survive the
deletion of the sweep it controls.

⚠ **Paths in any pulsare message are repo-root-relative** (`wat-rs/docs/…`), not relative to
`wat-rs/`.
