# DESIGN — `production_delta` pays a map lookup per derived fact, and the fix is 100 lines away

> Drawn 2026-09-06 at HEAD `efa754a3d`. Source: vigilia 2026-09-05 `temperare` §3.
> **Every citation verified on disk at THIS HEAD.**

## ⛔ What I am NOT claiming, and why

The previous strike's DESIGN said `join_extend` did **three** lookups per emitted pair. Driven, the
live count was **one** — `span_from_row` is skipped whenever the join index already carries a
BindSpan, and I had read that function's prologue without checking whether the call site reaches it.
The instrument built for the strike immediately falsified the brief that asked for it.

**So this DESIGN asserts no lookup count.** The shape is verified; the quantity is the strike's first
job. Instrument, read, then hoist.

## The site

`production_delta` (`fire/pass/production.rs`), loop nest:

```
for node_id in &arm.kind_ids.prod {        ←  *node_id fixed for everything below
    for pid in pids {
        for tok in ts {
            for (compiled, slots) in … {
                …
                wm.production.entry(*node_id).or_default().push(derived.clone());
```

The `entry` is keyed on the **outermost** loop variable, executed in the **innermost** body.
`ProductionMemory = HashMap<i64, Vec<Value>>` (`session.rs:156`) — std SipHash on an `i64`.

**Not the same defect, do not touch it:** the neighbouring `idx.entry(derived.clone())` is the
explain index, keyed on a value that varies per fact. That lookup is load-bearing.

## ★ The law is written down, 100 lines away, with this exact arithmetic

`fire/pass/hash_join.rs:288-290`:

> *"`entry()` **HOISTED** out of the per-token loop: the key is constant, so the old form paid two
> map lookups per token (80,000 on the fanout cell) where two total will do. Correct regardless of
> the guard below."*

Same subsystem, same reasoning, same cell. Production was not swept. **This is the sixth time in this
arc that the tree stated the rule and applied it in exactly one place** — after `JoinRightIndex`'s
un-cured left twin, `explain.wat`'s missing sort, `record_token`'s four bypasses, `alpha.rs`'s
hand-maintained bool, and `from_parts` deriving the scalar but not the pair.

## The upper bound is already pinned by a gate

`census_count("prod:derivations")` (`production.rs:112`) sits nine lines above the `entry`, and
`fanout_cost.rs:236` asserts it is **exactly 40,000** on the fanout cell (`keys=100 × fanout=20`,
non-vacuity guard). So the `entry` runs **at most** once per new derived fact, and the axis to
measure on already exists.

## The one contract decision, pinned

**Buffer per node, extend once — and gate the PREDICTED lookup count.**

`temperare` named the borrow shape and it is the reason not to hold the `&mut` open: `wm.production`
and `wm.derived_facts` are disjoint fields, but the opt-in explain arm takes `&wm` whole. So the
hoist is a local `Vec<Value>` per node, `extend`ed into `wm.production` once — **not** an
`entry()` held across the inner loops.

Prediction after: **one `entry` per production node that derives anything**, independent of the
derived-fact count. Assert that as a formula in the axis parameters, as
`join_alpha_lookups_match_the_per_node_prediction` does.

⛔ **NO WALL-CLOCK CLAIM.** Counts only. Six samples or no number, and none are asked for.

## Scope

**IN:** the instrument, the before reading, the hoist, the predicted-count gate, behaviour identity.
Floor GREEN.

**OUT, affirmatively cut:** the explain index's `entry`; `temperare` §2, §4, §5; the
FxHashMap-vs-SipHash question (a hasher changes iteration order — F1 is why that rides alone);
census B, D–M; A4, D2p, F2.
