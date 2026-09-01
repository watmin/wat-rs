# SCORE — E1+E2, weighed against the orchestrator's own re-run

> Re-run here at `1efb42fc7`.

## The scorecard, re-run

| # | pre-value at HEAD | after |
|---|---|---|
| 1 | caret **cols 31–76** (46 chars) | ✅ **cols 65–75** — the keyword's exact extent, verified in the golden diff |
| 2 | nested-constructor producer | ⛔ **NOT REACHABLE AT ALL** — see the finding below. My row was wrong |
| 3 | `fact_span` on the kwargs path | ✅ `:nope` at line 14, col 21–26 |
| 4 | ★ `span: Span` accepted anything | ✅ **one producer**, `check_field_kw(field_kw: &WatAST, …)`; `UnknownField` is constructed at exactly one site in `validate/` |
| 5 | the dead arm | ✅ **driven** before removal, not deleted on my reading |
| 6 | `UnknownEnumVariant` unmoved | ✅ all five arms green, spans unchanged |
| 7 | radius | ⚠ **+4 files** — the bind path needed a node the classifier discarded |
| 8 | lint 116/116 | ✅ 116/116 |
| 9 | floor 5220/5220 | ✅ `Summary [ 369.671s] 5225 tests run: 5225 passed, 21 skipped`, zero FAIL rows |
| 10 | clippy rc=0 | ✅ rc=0 |

## ⛔⛔ THE FINDING CONTRADICTS MY OWN STONE: TWO OF FOUR PRODUCERS WERE DEAD

I listed the nested-constructor producer as live and gave it a scorecard row and a mutation.
**It cannot run.** `defrecord` lowers **every** record-constructor call before freeze, so
`(:fsn::Inner :nope ?k)` reaches the wall as `(:wat::core::kwargs-construct :fsn::Inner …)`;
`walk_nested_constructors` matches the record type as **HEAD**, `types.get` returns `None`, and
**four** error kinds are unreachable there — `UnknownField`, `RhsMissingFields`,
`RhsArityMismatch`, `RhsPositionalConstructionRetired`.

**Re-driven here**, not taken on report:

```
(:fsn::Outer :k ?k :inner (:fsn::Inner :nope ?k))     ← undeclared field, and `x` unsupplied
→ "ACCEPTED-UNVALIDATED"
```

**Why it looked alive:** the walker's sibling enum-variant branch *is* live — an enum variant is not
lowered — so the walk is exercised from outside and only the lowered arms are gone. `purity.rs` hit
this identical class and **was** taught the post-lowering shape; this walker never was.

Not fixed here, and rightly: that is a wall-reachability strike across four error kinds, not a span
strike. **Pinned as a test with an anti-vacuity guard** — it asserts the program is accepted AND that
it reached its sentinel, so it cannot pass by failing some other way, and it names what to assert
when someone wires the branch. A finding as a live gate rather than a paragraph in a stone nobody
re-derives. Promoted to memory.

## ⛔ Where MY brief was thin

- **A. ★ Row 2 pointed at a dead producer** — above. My mutation 2 for it predicted "nothing reddens",
  and the rider observed exactly that; I had read a *correct* observation as insensitivity rather
  than as the finding it was. It made the mutant `unreachable!` so silence proved non-execution.
- **B. My sketch's `else { return }` would have shipped a silent vanish.** Every span this strike
  removes is an enclosing form — a `List`. Under the sketch, a future mis-call turns "wrong caret"
  into **no error at all**, worse in kind than the defect. The rider used `unreachable!`; all four
  callers are keyword-guaranteed, so it cannot fire on user input, and mutation 1 shows it firing
  loudly on exactly that mistake.
- **C. Blast radius understated by four files.** The bind path needs a node that
  `ReteClauseShape::Bind` discarded, so the shape grew `field_kw` and three consumers needed `..`.
- **D. My mutations presume the defect's own type.** "Pass `fact_span` at the kwargs producer" is
  *unwritable* after the cure — which is row 4 succeeding. Post-cure, isolating a span requires
  fabricating a keyword node, and the rider's first attempt (`&fact_items[0]`) changed the field
  **name** too and reddened nine tests. It re-ran as a synthesised keyword with the right name and
  got exactly one red. **A mutation that changes two things proves neither.**
- **E. HEAD drift unremarked.** EXPECTATIONS measured at `9c4748b4d`; the rider ran at `80dd6673f`.
  Row 1's pre-value still held, but nobody had said it should.

## Friction worth recording

- **`git stash` is denied to the rider tier by the permission classifier**, so it could not build a
  HEAD binary for a before/after. The checked-in golden served as the pre-value instead — which
  worked, and is an argument for goldens over ad-hoc measurement.
- **A deliberately-failing scratch `.wat` cannot live in `wat-scripts/scratch-pad/`** without
  reddening `every_wat_scripts_file_loads`. I hit this myself during D1's residual. Throwaway failing
  fixtures went to `/tmp`; a durable one belongs beside the probes in `tests/rete/`.

## Arms not driven, named

Row 2's producer — **not reachable, and why** (above), pinned rather than fixed. Everything else
proven.
