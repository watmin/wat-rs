# EXPECTATIONS — STONE 118.B8 · the arc's tail

Written before the strike, so the result cannot move the goalposts. Rows 7–9 are the ORCHESTRATOR's.
Row 6 is the orchestrator's too — Part 3 is not briefed to the rider.

| # | what | who | expected |
|---|---|---|---|
| 1 | ★ `dorun`'s peak RSS is **FLAT** in n | rider | four numbers at 100k/200k/400k/800k, slope ≈ 0. **Before** must be LINEAR — a before that is already flat means the probe is not measuring `dorun` |
| 2 | `dorun` still forces every element | rider | n elements in → **exactly n** forces recorded |
| 3 | the recursion is in TAIL position | rider, read the diff | the `Item` arm's call is the arm's whole body, not nested in an argument |
| 4 | `doall` is UNCHANGED | rider + orchestrator, read the diff | still `(into [] coll)` — it returns the Vector |
| 5 | `extract_lazyable_elem` SURVIVES | orchestrator, read the diff | function intact, all 6 call sites intact, only the doc block changed, "would delete" gone |
| 6 | the class census reports an INVENTORY from a FORM TREE | **orchestrator** | a committed `.wat` census + its output; every sibling dispositioned |
| 7 | floor | **orchestrator** | ≥4772 run, 0 FAIL, 19 skipped |
| 8 | clippy | **orchestrator** | 0 |
| 9 | ignores | **orchestrator** | 13 (line-anchored `#[ignore` grep, not the loose one) |

**Row 1 is the stone.** A `dorun` that is merely *faster* is B5's result, already banked. Flat is
this stone's, and flat is a complexity claim — so the BEFORE column is load-bearing: without a
linear before, a flat after proves nothing about `dorun`.

**Row 5 is the row a careless strike fails**, because the arc's own record tells the rider to delete
that function. The brief says otherwise and explains why; if the diff shows a deletion, the rider
followed the stale order over the live brief.

## Independent prediction

**35–55 minutes.** Part 1 is a four-line body plus two probe adaptations — the probe work is most of
it, and the retention probe's header must be read before it is adapted (two of its design choices
exist because an earlier draft lied). Part 2 is one doc block.

## Trap-doors named in advance

- **The expand-time trap is the likeliest STOP.** `dorun` is in `is_pure_total` for
  macro-expansion legality and its body becomes self-recursive; task #107 records that a macro
  body's reach into wat-defined functions is restricted. If this fires it is a **finding worth more
  than the stone** — do not route around it.
- **`_value` in the `Item` arm.** Task #67 records that a `_`-prefix slips must-use gates that a bare
  `_` catches. Here the binding is genuinely unused, which is the honest spelling — but if a gate
  fires on it, that is data about the gate, not a reason to fake a use.
- **The goldens carve-out.** 8 fixtures pin a real Rust line (`runtime.rs` ×5, `check.rs` ×2,
  `freeze.rs` ×1). Part 2 touches only a doc comment, so nothing should move — but if a number
  shifts, ratify it the way B6b did rather than bumping it on sight.
- **Do not argue from `dorun`'s zero callers.** It has none today. That is not evidence about its
  value and not a reason to inflate or deflate this stone.
  `[[feedback_no_consumers_does_not_mean_dead]]`
