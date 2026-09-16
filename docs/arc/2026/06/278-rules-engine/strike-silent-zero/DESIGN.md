# DESIGN-STONE — one `Option`, two facts, two different wrong answers

> **Origin (2026-08-31).** Surfaced by the rider that executed Class A2, driven and deliberately
> NOT committed as a test — because committing it would have enshrined the behaviour. It is the
> tail of A2: that strike removed nine `panic!`s, and this is the one path in the same file that
> never panicked because it was already answering wrongly and quietly.

## Why

`fold_bucket`'s `Sum` arm (`acc.rs:321-323`):

```rust
let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
    return Ok(Some(Value::i64(0)));
};
```

`operand_slot` returns `Option<usize>`, and its `None` carries **TWO FACTS**:

| where | what it means | legitimate? |
|---|---|---|
| `let &i = bucket.first()?` | the bucket is **EMPTY** | **yes** — sum's identity genuinely is 0 |
| `.position(\|(id,_)\| …)` | the var **names nothing** in the bind keys | **no** — this is A2's defect exactly |

**The empty-bucket identity is being reused to answer "the var isn't there."** And the arm's own
siblings disagree about the same `None`: `Min`/`Max`/`Mean` (`acc.rs:345-347`) return `Ok(None)`
and drop the fact. One conflated `Option`, two consumers, two different wrong answers.

This is the shape this repo has a recorded lesson about — `Option` meaning both *"legitimately
absent"* and *"I failed"* is where silence hides — and it is the shape A2's own cure was measured
against: `EXPECTATIONS.md` there says outright that **a silent wrong answer is worse than the panic
it replaced.** A2 did not create this one, but A2 is what makes it the last hole on this path.

## The measurement — driven, then re-driven

The A2 rider drove it and reported `RECON: ACCEPTED and returned i64(0)`. Re-driven here at HEAD
`2a7051c67`, with the probe banked beside this file:

```
SILENT WRONG ANSWER: import+fire ACCEPTED a :sum fold key no condition binds and returned
i64(0) instead of refusing. `operand_slot` answered `None` because the var names nothing, and
`fold_bucket`'s Sum arm read that as the EMPTY-BUCKET identity and summed to 0. Min/Max/Mean
answer the same `None` by dropping — one `Option`, two facts, two different wrong answers.
```

The fixture is the one the A2 rider built and documented: a three-var join whose `:from` binds
nothing the token does not already bind, which is the only rule shape a tampered fold key cannot
divert away from `fold_bucket`. That reasoning is written in the fixture; read it before touching
the shape.

## ⚠ WHICH ARM THE PROBE REACHES

| arm | answer to the conflated `None` | probe reaches it? |
|---|---|---|
| `acc.rs:321-323` — `Sum` | `Ok(Some(i64(0)))` — a wrong number that looks like a real sum | **YES** |
| `acc.rs:345-347` — `Min`/`Max`/`Mean` | `Ok(None)` — silently drops the derived fact | **no** |

Two arms. **One probe proves one arm.** The `Min`/`Max`/`Mean` arm needs its own fixture or its
own tamper, and the brief prescribes it rather than warning about it — this is the third strike in
a row where that distinction has mattered, and the second where it was written down in advance.

## The algorithm

Split the outcome by **type**, not by discipline. `operand_slot` returns a three-variant enum —
the bucket was empty, the slot was found, or the var is not among the bind keys — and each caller
matches all three. `Sum`'s empty-bucket arm keeps `Ok(Some(i64(0)))`; `Min`/`Max`/`Mean`'s keeps
`Ok(None)`; **both `Unbound` arms return the same refusal A2 already built** (`acc_refusal`, naming
the fold var and the door).

## ★ THE ONE CONTRACT DECISION

**The empty-bucket identity may be reached ONLY from an actually-empty bucket.** After this strike
there must be no path on which "the var names nothing" produces a number — not `0`, not a dropped
fact, not a `None` a caller reads as absence. The two facts get two names, so the conflation has
no representation and a future arm cannot re-mint it by accident. Climb to the type; a comment
saying "check the bucket first" is the rung below and it is what allowed this.

## Blast radius

`src/rete/kernel/fire/acc.rs` only, plus the probe into the existing
`tests/rete/probe_arc278_import_fold_key.rs`. No new file, no fixture change, no wire-format
change. `operand_slot` is `pub(super)`; check its callers before assuming the radius holds.

## Out of scope — AFFIRMATIVELY CUT

- **`packed_operand_field`'s `Option` (`acc.rs`, the doc above it).** It looks like the same shape
  and is not, and its own doc already says so: *"Returning None here is what routes it there; it is
  a dispatch choice about a LEGITIMATE fold, not a refusal."* Verified before cutting. **Do not
  sweep it in** — converting a correct dispatch into a refusal would break the 8b-sum and
  where-accum-where paths that rely on it.
- **`fire/mod.rs`'s `key_of` / `key_of_el`** — eight more arms of A2's class, still their own
  strike.
- **Refusing bad fold keys at the import door.** Still the other honest fix, still cut for the
  reason A2's stone gives: import has not assembled the condition set at fold-read time.
