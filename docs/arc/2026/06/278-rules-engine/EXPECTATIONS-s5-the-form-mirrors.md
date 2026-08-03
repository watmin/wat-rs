# EXPECTATIONS — S5 (#56): the four form mirrors

Written BEFORE the strike so the result cannot move the goalposts. Scored after the
**orchestrator's own** re-run, never the rider's report.

## Baseline, measured before the spawn

- Whole floor: **4295 run / 4295 passed / 0 failed / 262 skipped** (`5ffdfc5c`, own `--release`).
- Rete target: **218 passed / 0 failed / 9 ignored**.
- Lint target: **66 passed / 0 failed**.
- Corpus: **9 pairs / 98 rows, all agreeing**.
- Clippy: **zero** warnings, `--all-targets`.
- `wat/rete.wat:658` = `(:wat::core::and is-pure is-det)` — the fence UNARMED.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | builds clean | `cargo build --release --all-targets` | exit 0, **zero** warnings |
| 2 | clippy clean | `cargo clippy --release --all-targets` | exit 0, **zero** warnings |
| 3 | ★ `if` routes to `infer_if`, not the bool arm | a rete `if` with non-bool branches type-checks | branches unify; no bool coercion |
| 4 | ★ `if` does not evaluate the untaken branch | fixture: untaken branch raises | returns the taken value |
| 5 | ★ control for row 4 | same operand, taken | DOES raise, typed kind |
| 6 | ★ `let` actually scopes a binding | fixture: bind, then read it | the bound value |
| 7 | ★★ **TCO survives** — the `eval_tail` gate | tail-recursive fn, rete `if` tail, depth 200000 | returns; **no SIGSEGV** |
| 8 | ★★ row 7 goes RED without 1b | remove the gate, re-run row 7 | fails/segfaults; then restored |
| 9 | STOP-1 — an unrouted Form is LOUD | inspect: what does a Form row with an unrouted `core_name` do? | a located error or a build break — never `infer_boolean_shortcircuit` |
| 10 | ★ `match` arm patterns are not classified as exprs | a rete `match` whose pattern would fail as an expression | classifies clean |
| 11 | `fn` — grounded before edited | report on `check.rs:525`, `:1654`, `:1735` | a written finding, edit or STOP-3 |
| 12 | ⛔ the corpus is UNMOVED | `./wat-scripts/perf/grid/check-where-shapes.sh` | 9 pairs, 98 rows, all agreeing |
| 13 | ⛔ the fence is still UNARMED | read `wat/rete.wat:658` | `(and is-pure is-det)` — unchanged |
| 14 | rete suite green | `cargo test --release --test rete` | ≥ 218 passed, 0 failed |
| 15 | repo lints green | `cargo test --release --test lint` | 66 passed, 0 failed |
| 16 | ★ no `rune:lint` added to pass a lint | inspect the diff | zero new runes |
| 17 | ★ each op named in ONE file | inspect the diff | no rete op outside `vocabulary.rs` |
| 18 | whole floor | **orchestrator's own** `cargo nextest run --release` | ≥ 4295 passed, 0 failed |

**Rows 7 and 8 are the pair that matters and neither counts alone.** Row 7 green with row 8
un-run is the vacuous-gate class this arc has hit four times (R59; `91bbb8cd`'s 11 gates; R62's
empty rejection column; last strike's `is_err()` control). Rows 4+5 are the same discipline for
`if`, mirroring how `or`'s gate landed at `5ffdfc5c`.

Row 9 is a *design* row, not a test row: the answer may be "I made it a build break" or "I made it
a located runtime error." Either passes. "It falls through to the bool arm" fails.

## Runtime prediction

**Phase 1: 30–45 min.** Mechanical once the two mechanism edits are drawn — and they are, in the
brief, with the `:4486` gate as a copyable exemplar.

**Phase 2: 25–50 min, and this is where the band is honest rather than confident.** `match` should
mirror Phase 1. **`fn` is the unmeasured one** — three sites treat `:wat::core::fn` specially
outside the inference arm, and nobody has ground what a rete-named `fn` does at any of them.

**Upper bound 95 minutes; wakeup at 2×.**

## Trap-doors named in advance

1. **`fn` is the real unknown** (STOP-3). If it needs more than an inference route, Phase 1 +
   `match` land and `fn` reports. **That is a partial success, not a failure**, and it is the
   outcome to bet on if the band is exceeded.
2. **The `eval_tail` gate may not be a clean mirror.** `:4486` returns a `Value`; `eval_tail`'s
   arms return varied shapes (`eval_let_tail` returns a `TrackedValue` that the arm unwraps). If
   the mirror needs per-arm handling rather than one gate, that is fine — but it is bigger than
   the brief implies, and it should be reported as a delta.
3. **Row 8 may be awkward to perform.** Removing the gate to watch the test fail means a
   deliberately broken tree for one run. Do it, record both observations, restore. If it cannot be
   done safely, STOP-7 rather than asserting the gate works.
4. **The two lints.** They bit the orchestrator on the immediately prior strike in this exact
   file. A doc comment or assert message that PARSES as a wat list; a `contains()` on a rendered
   error. Both fixed at the root, never runed.

## What a Mode B looks like

Phase 1 lands green (rows 1–9, 12–17) and Phase 2 reports `match` done, `fn` STOP-3'd with a
written grounding of the three sites. **Reportable and committable** — the mechanism is in, the
head-table pair is real, and `fn` has a measurement attached to its blocker instead of a guess.

## What would make this a failure

The corpus moving (row 12), the fence arming (row 13), an op named twice (row 17), a `rune:lint`
added to pass a lint (row 16), row 7 landed without row 8, or a Form silently reaching
`infer_boolean_shortcircuit` (row 9).
