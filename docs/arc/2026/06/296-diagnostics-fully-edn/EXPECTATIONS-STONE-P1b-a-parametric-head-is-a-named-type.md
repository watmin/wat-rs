# EXPECTATIONS — STONE P-1b

| # | what | command | expected |
|---|---|---|---|
| 1 | the subject refuses | `--check …__parametric_head_phantom.wat` | EXIT 1, names `:usr::TotallyMadeUp` |
| 2 | ⛔ the widest control | `--check …__parametric_head_real.wat` | EXIT 0 |
| 3 | the bare form still refuses | `--check …__bare_head_phantom.wat` | EXIT 1 |
| 4 | args still refused | `--check …__parametric_arg_phantom.wat` | EXIT 1 |
| 5 | the probe's gate | `-E 'test(p1b_a_parametric)'` | `4 passed, 0 skipped` |
| 6–9 | P-1 / P-2prereq / P-3 / A-1 unmoved | their four filters | 10 / 4 / 5 / 4, all 0 skipped |
| 10 | ONE walker | `grep -c "fn walk_" src/declare/typevar.rs` | unchanged |
| 11 | free-vars unchanged | rider states that `collect_free_type_vars` ignores the head, and which test proves it | heads are not auto-generalized |

★ **Row 2 is the row that catches the catastrophic version.** Rows 1, 3, 4 all pass if the stone
refuses parametric heads *indiscriminately* — and so does row 5. Only row 2 separates "the phantom
head is refused" from "every generic in the corpus is refused."

★ **Row 11 is the row a silent regression satisfies.** If heads start being collected as free type
variables, every parametric annotation auto-generalizes. The corpus would very likely still compile
— it would just mean something different, and no existing test asks. The rider must NAME the test
that pins it, not assert the behaviour.
`[[feedback_a_predicate_can_be_wrong_in_both_directions]]`

## Independent prediction

- **Runtime:** 15–30 min. The change is small; the corpus response is the unknown.
- **Diff:** ~20–50 lines in `src/declare/typevar.rs`.
- **Floor:** possibly non-zero. Every generic annotation in 845 files gets its head checked for the
  first time. A red naming a REAL type is a finding about the four-store union — the same shape
  RELAND-1 surfaced — not a site to patch.

## Trap-doors named in advance

1. **A head that is itself a bound type parameter.** If a parametric head can be a variable in this
   grammar, the head visit must respect `bound` exactly as arg paths do.
2. **`format_type` round-trips.** Edge keys and `contains` lookups are strings; a head arriving with
   or without its leading colon is the recurring one-side-normalized class. Check both spellings
   before concluding a real type is unknown.
3. **The free-var caller.** It shares the recursion and must not see the head. A callback the caller
   ignores is the shape; a boolean flag threaded through the walk is how this gets subtly wrong.

## What I re-run myself

Rows 1–11, then `scripts/floor.sh` unpiped with `$?` read directly, then
`cargo clippy --release --all-targets -- -D warnings`, expecting the same 5 pre-existing items.
