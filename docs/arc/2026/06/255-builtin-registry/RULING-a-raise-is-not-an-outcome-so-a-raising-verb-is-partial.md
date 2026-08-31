# RULING — a raise is not an OUTCOME, so a raising verb is `Partial`

> **Builder, 2026-08-30, adjudicating:**
>
> *"the premise of totality is simple, yes?... it either produces a guaranteed outcome or it
> doesn't?.. panics and raises... they are not matcheable... so.... its either a concrete value...
> or its a concrete value... the latter being an enum with error bearing arms adjacent to valid
> return value."*
>
> And the direction it serves:
>
> *"long term we will purge raises and panics from wat... the entire lang will be total.... the
> expect calls... i think they were a fatal mistake.... we will need to rip them out as we grind
> forwards."*

## The rule

**Totality asks one question: does the verb produce a guaranteed, MATCHABLE outcome?**

- A **concrete value** → total.
- An **enum with error-bearing arms beside the valid ones** (`Result`, `Option`, an outcome enum) →
  **also total.** The error is a value the caller can `match`. This is the shape the language wants.
- A **raise or a panic** → **NOT an outcome.** Nothing can `match` it. The verb is **`Partial`**.

The test is *matchability*, not whether the failure is deterministic, located, or well-diagnosed.
A raise can be all three and still not be a value.

## What this settles

⛔ **`Option/expect` and `Result/expect` are `Partial`.** `rete/purity.rs`'s `KNOWN_UNREVIEWED` doc
called them *"total but they raise"* — that is now **struck**. "But they raise" was never a
parenthetical; it is the whole answer.

★ **Every generated record accessor is therefore `Partial`**, because each wraps its receiver check
in `Option/expect`:

```clojure
(Record/field-at (Option/expect (if (= (type self) "R") (Some self) None) "…") 0)
```

That is not a defect in the accessor. It is `expect` propagating, and it is exactly the thing the
long-term purge exists to remove.

✅ **`:wat::i64::/`'s `@Total Partial` is now ADJUDICATED, not merely transcribed.**
`src/intrinsic/mod.rs` recorded it as *"not an adjudication of that dispute; it is a TRANSCRIPTION
of what the verb's own shipped doc already says"*, deferring to *"stone T4's ruling."* This is that
ruling, and it agrees with the transcription.

## ⚠ The "two contradicting lists" were already reconciled — do not re-litigate

`mod.rs`'s comment describes a pre-rename world: *"`macros/eval.rs`'s `is_pure_total` includes it …
while `rete/purity.rs`'s `total` sub-list excludes it."* Since arc 255 Stone expand-1 renamed
`is_pure_total` → `is_expand_time_legal`, that file says the opposite of what the comment reports:

> *"`:wat::i64::/` … is legal **despite being `@Total Partial`** … A partial verb can still be
> expand-time-legal … **Totality and expand-time legality are different axes.**"*

**The two lists agree.** They never disagreed about one property — they answered two questions under
one name, and the rename retired the contradiction. The only surviving dissent was the
`Option/expect` parenthetical this ruling strikes.

## The consequence nobody should mistake for a defect

A `Total` DEMAND — a gate insisting some caller-supplied callable be total — **cannot be satisfied by
anything that touches a record accessor**, until the `expect` purge lands. That is the rule working,
not a gate malfunctioning. A gate that demanded `Total` here would be correct and unusable at the
same time, which is why the demand itself is the thing to question, verb by verb, rather than the
ruling.

★ **Corollary for A-2-ii-b:** `sort$native` imposes **`Pure ∧ Deterministic`, NOT `Total`.** Each
imposed axis needs a measured defect behind it:
- **Pure** — measured: an effectful comparator fires **4 side effects on a 3-element vector**, an
  implementation detail (the two-sided `less?` call) leaking into user-observable output.
- **Deterministic** — a nondeterministic comparator makes the result unstable across runs.
- **Total** — ⛔ **no defect behind it.** Measured: `sort$native` is total *on its own merits*; a
  pathological comparator returns a scrambled well-formed vector, exit 0, no panic
  (`255-probe-can-a-user-make-sort-effectful.wat`). A comparator that *raises* simply makes the sort
  raise — ordinary propagation. Imposing `Total` would refuse every accessor key for no defect.

## The direction this serves

The long-term goal is a **total language**: no raises, no panics, every failure a matchable arm.
`expect` is named by the builder as *"a fatal mistake"* to be ripped out as the work grinds forward.
Under this ruling the registry becomes the census for that campaign — `@Total Partial` is the
worklist, exactly as `Totality`'s own `defenum` says:

> *"★ THIS VARIANT IS THE WORK LIST: the totality endgame's census is
> `all_entries().filter(|e| e.totality == Partial)`."*

⚠ **That census is only as good as the rulings behind it.** Today one verb declares `Partial` and
403 read `Unreviewed`; the purge cannot be scoped until those are measured under THIS rule.

## What this ruling does NOT do

- It does not retire `expect` — that is a campaign, not a stone, and it is not open.
- It does not rule the 403 `Unreviewed` verbs. It gives the rule they get ruled *by*.
- It does not make any currently-green thing red: nothing today imposes `Total` on a callable.
