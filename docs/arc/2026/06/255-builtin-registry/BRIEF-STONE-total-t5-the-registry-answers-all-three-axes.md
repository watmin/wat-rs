# BRIEF — STONE total-T5: derive all three axes; prune the ledger the derivation makes stale

Read `DESIGN-STONE-total-t5-the-registry-answers-all-three-axes.md` first.

## The work, one paragraph

`intrinsic_meta` decides `pure` and `deterministic` by membership in a 177-name `matches!`, while
every registered verb already declares both. **Make it read the registry for any verb the registry
knows, on all three axes.** Then delete the `KNOWN_UNREVIEWED` rows the change makes stale — the
gate will tell you exactly which.

## Read in order

```
src/rete/purity.rs  fn intrinsic_meta       the early returns, then the `pure_det` matches!,
                                            then T4b's `total` derivation (your model — it already
                                            does for one axis what this does for three)
src/intrinsic/mod.rs:1038                   matches!(entry.purity, Pure | Preserving) — the
                                            house convention for treating Preserving as satisfying
src/rete/purity.rs  KNOWN_UNREVIEWED        228 rows; most will go stale
src/rete/purity.rs  the `stale` assert      it names exactly which rows to delete. Let it.
```

## Implementation sketch

```rust
// AFTER the existing rete_op_for and the early-return special cases, BEFORE the pure_det matches!:
if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
    return Some(OpMeta {
        pure:          matches!(e.purity,      wat_doc::Purity::Pure | wat_doc::Purity::Preserving),
        deterministic: matches!(e.determinism, wat_doc::Determinism::Deterministic
                                             | wat_doc::Determinism::Preserving),
        total:         matches!(e.totality,    wat_doc::Totality::Total
                                             | wat_doc::Totality::Preserving),
    });
}
// unregistered only, below: the residual pure_det + total hand-rulings
```

★ **Order matters.** The early-return special cases stay AHEAD of this block. They are verified to
agree with their registrations, so they are redundant — but proving that and retiring them is a
follow-up, and moving them now would mix two changes.

## ⛔ THE CONTAINMENT YOU MUST RE-PROVE

The design measured, via the registry: **275 verbs newly ruled, 163 pure∧deterministic, and ZERO
also total.** Because every one carries `@Totality Unreviewed`, the four-axis fence admits none of
them. **Re-measure this yourself before and after** — it is the argument that this stone is safe.

## Blast radius

`src/rete/purity.rs` only.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **`ALSO_TOTAL` is not zero.** A newly-ruled verb that passes all of pure∧det∧total would be
   newly admissible to a `where`. STOP and name it — that is a builder ruling, not a rider's.
2. **A verb ALREADY in `intrinsic_meta` changes its verdict.** The registry and the hand-list were
   measured to agree; a disagreement is a real finding. STOP and name the verb, both verdicts, and
   its declaration.
3. **You are about to keep a name-list of "which verbs derive".** The residue is defined by
   `lookup_entry(head) == None` and nothing else. STOP.
4. **You are about to retire an early-return special case** (`uuid::v4`, `keys`/`values`,
   `stream::next`, `aggregate-new`, …). Follow-up stone. STOP.
5. **The `where`-corpus goes red.** STOP with the failure verbatim.

## Acceptance

```
 0. ★ BASELINE FIRST: for every registered verb, record intrinsic_meta's三 verdicts (or None).
      Also record NEWLY_RULED / PURE_AND_DET / ALSO_TOTAL. Report all four numbers.
 1. ★ THE DERIVATION IS IN, placed after the early returns and before the residual matches!.
 2. ★ RE-MEASURE ALSO_TOTAL = 0 after the change. State it.
 3. ★ NO VERB ALREADY IN intrinsic_meta CHANGED VERDICT. Diff the pre/post verdicts for the ~133
      registered names it already covered; report the diff is empty.
 4. ★ THE LEDGER PRUNED to what the gate demands. Report: KNOWN_UNREVIEWED before, after, and the
      count the `stale` assert named. Do not delete a row the gate did not name.
 5. ★ THE RESIDUAL HAND-RULINGS now cover ONLY unregistered verbs. Show that
      `registry().lookup_entry()` is None for every name still listed.
 6. ★ PROVE THE REGISTRY IS THE SOURCE, not merely consulted: flip one verb's `@Purity` to
      Effectful, show `intrinsic_meta(...).pure` become false, restore. Pick a verb that was
      NOT previously in the hand-list, so the old path cannot be what answered.
 7. ★ THE WHERE-CORPUS IS GREEN. Name how you ran it.
 8. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 9. cargo nextest run --release -E 'test(rete) + test(purity) + test(intrinsic) + test(where)'
```

★ **Row 6 picks an unlisted verb on purpose.** Flipping one the hand-list already covered proves
nothing about which path answered.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

The four row-0 numbers. `ALSO_TOTAL` after. The row-3 diff, explicitly empty or not. Ledger before
/ after / the count the gate named. Row 5's evidence. Row 6's flip, the verb you chose and why, and
the restore. How you ran the `where`-corpus. Then the honest deltas — above all **any verb where
the registry and the hand-list disagreed**, because every one of those is a ruling somebody made
twice and differently.
