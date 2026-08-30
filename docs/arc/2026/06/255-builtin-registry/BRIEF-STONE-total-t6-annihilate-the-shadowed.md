# BRIEF — STONE total-T6: delete every shadowed name, and the one dead ruling

Read `DESIGN-STONE-total-t6-annihilate-the-shadowed.md` first.

## The work, one paragraph

`intrinsic_meta` (`src/rete/purity.rs`) consults the registry first, then falls through to literal
name lists. **Most of those names can no longer be reached.** Compute which — the registry knows —
delete exactly those, and separately delete `:wat::core::when`, which is a ruling for a verb the
language does not have.

## ⛔ DERIVE THE SET. DO NOT TRANSCRIBE ONE.

> Delete a name **iff** `registry().lookup_entry(name)` returns `Some`.

The design predicts 133. **That is a prediction to check, not a list to copy.** Write a temporary
test that walks `intrinsic_meta`'s literal names, partitions them by `lookup_entry`, and prints
both sets. Delete from the source using that output. Report your number against the design's — a
disagreement is a finding about the design, not about you.

## Read in order

```
src/rete/purity.rs  fn intrinsic_meta   the registry-consult block (total-T5) at the top, then
                                        the early-return special cases, then the literal lists:
                                        `pure_det`'s matches!, and the residual `total` matches!
docs/.../WORKLIST-the-44-unhomed.md     what survives, and why each survivor survives
```

## The two deletions are different acts — keep them separate in the diff

**1. The shadowed names** — derived, mechanical, safety proven by verdict-invariance.

**2. `:wat::core::when`** — deleted by NAME, because `lookup_entry` returns `None` for it exactly
as for every genuine survivor. It goes because it resolves to nothing at all. ★ **Re-confirm that
yourself** before deleting: run it and quote the error. The design says it is
`unknown function: :wat::core::when`; do not take that on faith.

## ★ THE INVARIANT — and it is also the proof

**Every verb's `intrinsic_meta` verdict, all three axes, identical before and after.**

A genuinely shadowed name cannot move a verdict when deleted, because the registry answers first.
So if any verdict moves, you deleted something reachable — the invariant catches exactly the error
this stone could make. Capture verdicts for **every registered entry AND every dispatched verb**
before touching anything.

## Blast radius

`src/rete/purity.rs` only.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **Any verdict changes.** A deleted name was reachable. STOP, name the verb and both verdicts.
2. **Your derived count differs from the design's 133.** Not necessarily wrong — but STOP and
   report the difference and which names account for it before deleting.
3. **`:wat::core::when` does NOT error as `UnknownFunction`.** Then it is live and the design is
   wrong about it. STOP.
4. **You are about to delete an early-return special case** (`uuid::v4`, `keys`/`values`,
   `stream::next`, `aggregate-new`, …). They are shadowed too, but they carry reasoning and retire
   per-case in a later stone. STOP.
5. **You are about to touch a surviving name to "tidy" it.** The survivors are a worklist. STOP.

## Acceptance

```
 0. ★ BASELINE: every registered entry's and every dispatched verb's three verdicts, captured
      BEFORE any edit. Say how many verbs you captured.
 1. ★ THE DERIVED PARTITION: your counts for shadowed vs live, against the design's 133 / 44.
 2. ★ THE SHADOWED NAMES DELETED — exactly the derived set, no more.
 3. ★ `:wat::core::when` DELETED, with your own re-confirmation of its error quoted verbatim.
 4. ★ ALL VERDICTS IDENTICAL to row 0. Diff and say so explicitly. This is the stone.
 5. ★ THE SURVIVORS ARE THE WORKLIST: list every name still present, and confirm
      `lookup_entry` is `None` for each. Any survivor that returns `Some` was missed — report it.
 6. ★ BREAK THE DOOR: delete one LIVE name (say `:wat::core::foldl`), show a verdict move,
      restore. Proves the invariant in row 4 can actually fail and is not vacuous.
 7. ★ LINE ACCOUNTING: purity.rs before/after.
 8. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 9. cargo nextest run --release -E 'test(rete) + test(purity) + test(intrinsic) + test(where)'
```

★ **Row 6 is what makes row 4 mean something.** An invariant that holds because nothing could ever
have broken it proves nothing. Show it breaking on a name that genuinely still answers.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your baseline size. The derived partition against 133/44. The `when` error verbatim. The row-4 diff,
explicitly. Every surviving name with its `lookup_entry` result. Row 6's move and restore. Line
accounting. Then the honest deltas — especially **any name whose classification surprised you**,
because a name you expected shadowed and found live is a verb we think is homed and is not.
