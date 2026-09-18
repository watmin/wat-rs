# DESIGN — the `filter:test-pass` union, and three gates that cannot fail

> Drawn 2026-09-05 at HEAD `9770eeeb4`. Source: vigilia 2026-09-05 `recon/census-name-audit.md` item A
> — thirteen sections nobody had rowed. **Verified on disk by the orchestrator at THIS HEAD.**

## Why this and not A4

A4's root (`identity`'s memo vs its walk) is `src/value/value.rs` — **main's file, not rete's**. Its
rete half (`seen_ids`/`seen_rest` as two halves of one set) is real CLASS A but buys a closed door,
not a closed hazard: the partition is safe while `identity()` is pure for a given value, and it is.
**This is a live defect in rete's own instruments**, and every cost claim in this arc reads through
them.

## The defect: one key, two disjoint populations

`filter:test-pass` is incremented from three sites in `fire/mod.rs`:

| site | did a predicate actually run? |
|---|---|
| `:2068` (reuse arm) | **NO** — `exec_stashed_where` is never called; the tree proved it |
| `:2078` (eval arm, tree branch) | yes, preceded by `filter:test-evals` |
| `:2102` (eval arm, fallback) | yes, preceded by `filter:test-evals` |

So `test-pass` is **passes ∪ tree-proven-pushed-without-evaluating** — *not* a subset of
`test-evals`. Its only written description
(`tests/where_tree_branch_differential.rs:83`) calls it *"a token that reached a beta/d_beta push"*,
which is accurate — and no increment site carries a doc saying so.

## ★ And a consumer subtracts them anyway, on an axis where the answer is fixed

`tests/node_share_cost.rs` asserts its own precondition at **`:296`**:

```rust
assert!(fire_reuse > 0 && fire_evals == 0, …);
```

Then, with `evals == 0` guaranteed:

```
:867   let wasted = evals.saturating_sub(passes);     →  saturates to 0, always
:869   let waste_pct = if evals == 0 { 0.0 } …        →  0.0, always
:882   assert!(evals <= passes.saturating_mul(2))     →  0 <= 2p, always
:887   assert!(evals <= (m as u64).saturating_mul(4)) →  0 <= 4m, always
:894   assert!(worst_waste < 50.0, "peak waste …")    →  0.0 < 50.0, always
```

**Three assertions are arithmetic identities**, and the headline one reports *"0% waste"* — the
flattering reading — as a **measurement**. It is a consequence of the axis, not of the engine.

`node_share_cost.rs:882`'s own comment says *"Linear scan is N×M (10_000 at [50 200]) — that must
not pass."* On this axis a linear scan **would** pass: `evals` would stop being 0, and `wasted`
would finally mean something — but the two `evals <= …` bounds are the only things that would then
bite, and the waste gate has never been exercised at all.

## The one contract decision, pinned

**A census key names ONE quantity. Split the union; then let the gates say what they can prove.**

- **Split:** the reuse arm gets its own key (it already bumps `filter:test-reuse` at `:2067` — the
  push it performs is a *different event* from an evaluated pass). `filter:test-pass` then means
  "a predicate ran and passed" and IS a subset of `filter:test-evals`, which is what `wasted`
  needs to be arithmetic rather than fiction.
- **Then fix the gates to what they measure.** A gate asserting `0.0 < 50.0` must either be driven
  on an axis where `evals > 0`, or say plainly that it is inapplicable here and refuse — the shape
  `right_index_counter_invariant.rs`'s `assert_applicable` already uses in this tree.

**Do not "fix" this by changing the assertion's threshold.** The number is not wrong; the number is
not a number.

## Scope

**IN:** the `filter:test-pass` split, its doc at every increment site, and the three assertions in
`node_share_cost.rs` made honest. Floor GREEN.

**OUT, affirmatively cut:** the other twelve census sections (B–M) — real, rowed, and each needing
its own reading; A4; D2p; F2. **`filter:test-reuse` and `filter:test-evals` keep their meanings** —
this strike changes what `test-pass` counts, nothing else.

## ⛔ Expect the census numbers to MOVE

This is the one strike this session where a changed cost number is the **point**, not a STOP.
`test-pass` will drop on any axis that uses the reuse arm. What must not change is engine behaviour:
same facts, same rows, same fires. Say which recorded numbers moved and why, in the SCORE.
