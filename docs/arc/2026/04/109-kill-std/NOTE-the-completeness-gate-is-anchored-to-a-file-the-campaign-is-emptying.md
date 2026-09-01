# NOTE — the purity completeness gate is anchored to a FILE the campaign is emptying

> Found 2026-09-01 when `src/declare/` went RED. **No ruling, nothing drawn.** This records a gate
> whose population definition is a file path, in a campaign whose success condition is emptying that
> file.

## What happened

`every_dispatched_verb_is_classified_or_disposed` (`src/rete/purity.rs`) failed on the declare stone:

```
2 verb(s) in `KNOWN_UNREVIEWED` are no longer unreviewed — they have been ruled on (or no
longer dispatched). DELETE their lines; the ledger must shrink as the debt is paid, or it
rots into a lie.
  :wat::core::defalias
  :wat::core::extend-type
```

The gate reads **one file**, and its own `.expect` string is the claim that broke:

```rust
std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime.rs"))
    .expect("runtime.rs must be readable — it holds the verbs still dispatched literally …")
```

★ *"it holds the verbs still dispatched"* was TRUE when written. The declare stone moved those two
verbs' match arms into `src/declare/register.rs:545` and `:1955`, and the gate — correctly, by its
own definition — reported them as having left its population.

## ⛔ THE FIX I TRIED FIRST WAS WRONG, IN THE OTHER DIRECTION

`dispatch_verbs` is file-agnostic, so widening the scan to every `.rs` under `src/` looked obvious.
**Measured:**

```
                    dispatched   classified   UNREVIEWED
  runtime.rs only        543         498          32
  all of src/            693         499         170     <- WRONG
```

It sweeps in `check.rs`'s inference arms, `freeze/env.rs`'s declaration matches, `types.rs` — all
**consumers** of verb names, not dispatch. Narrow blinds the gate; wide captures the wrong
population, and no adjustment of the *file set* separates them.
`[[feedback_a_predicate_can_be_wrong_in_both_directions]]`

## What the disposition actually was, and why the gate was right

`KNOWN_UNREVIEWED` tracks **dispatched verbs whose purity is unreviewed.** `:wat::core::defalias`
and `:wat::core::extend-type` are **declaration heads**, not expressions — `check.rs`'s declaration
arm returns early for both (*"declaration forms, not value-producing expressions"*). They were on the
ledger because the text scan saw their arms in `runtime.rs`, not because the evaluator dispatches
them as verbs. **Deleting the two rows was correct**, and the gate asked for exactly that.

⚠ **But it asked for the right thing for a reason it could not distinguish.** Its message offers
*"ruled on (or no longer dispatched)"* as one disposition — and those two have opposite fixes. Had
the move genuinely dropped a dispatched verb, deleting the row would have hidden a regression behind
a green gate.

## ⬜ THE OPEN QUESTION — not ruled

Arc 109 will keep moving dispatch out of `runtime.rs`. Every such move re-runs this exact event, and
each time the reader must decide by hand whether a verb *left the dispatch* or *left the file*. The
gate cannot tell those apart, because its population is defined by a path.

> **What defines "a dispatched verb" structurally, rather than by which file its match arm sits in?**

Candidates, none measured: an attribute on the dispatch functions; a registry-derived population (the
campaign's own direction); or an explicit list of dispatch-holding modules — which is the hand-list
this arc keeps retiring, and would rot the same way the file anchor did.

★ This is the third gate in one day whose anchor the campaign falsified — after
`every_dispatch_arm_calling_eval_threads_list_span`'s `MUST_FIND` name and
`purity_mandated_examples`' "every pure verb has a runtime call site". **The pattern is not
coincidence: a campaign that moves code invalidates every gate that names WHERE rather than WHAT.**
