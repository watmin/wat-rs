# BRIEF — census G: delete `prod:vec-alloc` and `prod:record-alloc`

Read `DESIGN.md` first, especially why this deletes where census E kept a never-firing branch — the
line is falsehood, not unreadness. **Two `census_count_n` lines go, plus their `//` comments, plus a
recorded reason. Nothing else.**

## Read in order

1. `src/rete/eval_insert.rs:186-210` — the whole tail of the function. Both bumps are on one
   straight-line path; note the three phase marks bracketing it (`prod:shape` `:190`,
   `prod:resolve` `:203`, `prod:construct` `:209`) and that `Vec::with_capacity(value_asts.len())`
   at `:189` allocates nothing when the vec is empty. Those marks are why deleting costs no
   observability.
2. `src/rete/eval_insert.rs:40-60` — `rete_kwargs_value_asts`. The kwargs branch allocates three
   vecs, the positional branch two. This is the evidence that a hardcoded `2` is not the allocation
   count on either branch.
3. `src/rete/eval_insert.rs:154-157` — `prod:class-alloc`. **Leave it.** One real `String` per call,
   accurately named. Read it so you can say in the SCORE why it stays and these two go.
4. `src/rete/kernel/tests/accum_cost.rs:130-150` — the exact-equality NAMES list. Neither deleted
   name is in it; confirm that after the change and say so.

## What to write where the counters were

One short comment recording the decision, so the next hand does not re-add them: the numbers were a
hardcoded constant rather than a measurement, nothing read them, and a real allocation count was
rejected because nothing wants one (census F's precedent). Name the three phase marks and
`prod:derivations` as what still covers this path.

## Verification — a grep and a green floor, not a mutation

A deleted counter that nothing reads cannot red anything. Report, in this order:

1. `grep -rn 'prod:vec-alloc\|prod:record-alloc' src/ tests/` → **no hits** (including comments).
2. The full floor Summary line — **green is the evidence** that nothing was observing them.
3. Whether any exact-equality NAMES list changed → expected **none**. If one did, that is STOP-1.

Do not manufacture a red. Census D and E reported honestly where no proof was available; same here.

## Blast radius

`src/rete/eval_insert.rs` only — two `census_count_n` lines, their trailing comments, and one
replacement comment. **No behaviour. No renames. No new counter. No test change.**

## STOP triggers

1. Any NAMES list or test needs editing → STOP and report; something *was* observing them.
2. Any measured value changes → STOP.
3. You reach for a replacement counter (real allocations, or a renamed call count) → STOP; both
   rejected in DESIGN.
4. `prod:class-alloc` or `prod:derivations` changes → STOP; they are correct.

## Prior result to copy for shape

`../strike-census-D-key-alloc/SCORE.md` — it stated plainly where a mutation proof was not
available instead of inventing one. Same discipline.
