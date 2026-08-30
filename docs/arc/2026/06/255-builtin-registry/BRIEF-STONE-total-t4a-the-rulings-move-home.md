# BRIEF — STONE total-T4a: move 27 totality rulings to their verbs

Read `DESIGN-STONE-total-t4a-the-rulings-move-home.md` first.

## The work, one paragraph

Twenty-seven verbs are declared `@Total Unreviewed` at their registration sites while
`intrinsic_meta`'s `total` sub-list (`src/rete/purity.rs`) already rules them total, with per-op
reasoning. **Change each to `@Total Total` and bring its reasoning with it**, as a short
`/// **Totality ground —** …` paragraph in that verb's own doc block. Nothing else changes.

## Read in order

```
src/rete/purity.rs, the `── `total` —` block   THE SOURCE. Its header explains the sub-list is
                                               "exactly the verbs the where-corpus uses inside a
                                               `where` … each verified total by READING its own
                                               implementation". Below it, per-op reasoning.
                                               ⛔ DO NOT EDIT THIS BLOCK. T4b collapses it.
src/intrinsic/i64.rs:~171                      `:wat::i64::/`'s `@Total Partial` — the shape to
                                               copy for placement and column alignment.
src/rete/collect.rs                            an existing "**Purity ground —**" prose paragraph:
                                               the house shape for carrying a ruling's reasoning
                                               in a doc block.
```

## The 27, by home

```
src/intrinsic/i64.rs         < <= = > >= not= to-f64 to-string        8
src/intrinsic/f64.rs         < <= = > >= not= to-string               7
src/intrinsic/vector.rs      length contains? get                     3
src/intrinsic/collection.rs  last reverse range                       3
src/intrinsic/holon/atom.rs  cosine dot coincident? presence?         4
src/intrinsic/special/control_flow.rs   if                            1
src/intrinsic/special/binding.rs        let                           1
```

## The shape

```rust
/// … existing prose …
///
/// **Totality ground —** comparisons never overflow (only `+`/`-`/`*`/`/` do), so the
/// output is defined for every pair of i64 inputs. Verified against `eval_compare`.
/// Ruling relocated from `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the
/// verdict is that list's, made by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @Category      Arithmetic
```

★ **Attribute the verdict, do not re-author it.** Each paragraph carries that verb's own reasoning
from the sub-list. Where the sub-list rules a GROUP in one sentence (the comparison families, the
`to-string` trio), each verb gets that sentence — but say which group it came from, so a reader can
find the original.

⛔ **Do not invent reasoning for a verb the sub-list covers only by inclusion.** If a verb is in the
list with no prose of its own, say so in your report and write the honest minimum: that it was
verified by the sub-list without a recorded per-op argument. A fabricated justification is worse
than a thin true one.

## Blast radius

The eight files above. **`src/rete/purity.rs` is NOT edited** — it is the source you read from.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **The floor changes.** Nothing reads `IntrinsicEntry.totality` in production yet, so this stone
   must be behaviour-neutral. A moved test count or a new failure means something unexpected reads
   it. STOP.
2. **You disagree with one of the 27's totality.** Report the verb and your argument. Do NOT
   transcribe a verdict you believe is wrong, and do NOT quietly write `Partial` instead. STOP.
3. **You are about to edit `intrinsic_meta` or its `total` sub-list.** T4b's job. STOP.
4. **A verb in the list is not registered, or has no `@Total` line to change.** The design says all
   27 are registered. STOP and report which.
5. **`:wat::i64::*` gains a `@Total` answer.** It is the CONTROL in
   `totality_is_carried_from_the_doc_into_the_registry_entry` and must stay `Unreviewed`, or that
   test stops proving carriage. STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: confirm all 27 are registered and currently read `@Total Unreviewed`.
      Report the command and the count.
 1. ★ ALL 27 NOW READ `@Total Total`, each with its own Totality-ground paragraph. Quote three
      paragraphs in full — one comparison, one conversion, one from the holon four.
 2. ★ THE COUNT MOVES EXACTLY: `@Total Total` 0 → 27; `@Total Unreviewed` 431 → 404;
      `@Total Partial` unchanged at 1 real verb (`i64::/`).
 3. ★ THE CONTROL SURVIVES: `:wat::i64::*` still reads `Unreviewed`, and
      `totality_is_carried_from_the_doc_into_the_registry_entry` still passes.
 4. ★ ZERO BEHAVIOUR CHANGE: `git diff` touches ONLY doc-comment lines. No fn body, no signature,
      no test, no `.wat`. State that the diff is `///` lines only.
 5. ★ `src/rete/purity.rs` diff is EMPTY. Say so.
 6. ★ BREAK THE DOOR: set one of the 27 to `@Total Partial`, show the registry read it back as
      `Partial` (the T2b carriage test's mechanism), restore. Proves the declarations you wrote are
      actually reaching the registry rather than sitting in prose.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo nextest run --release -E 'test(intrinsic) + test(purity) + test(rete)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your pre-check. Three quoted Totality-ground paragraphs. The three counts from row 2. The control's
status. Confirmation the diff is doc-lines only and that `purity.rs` is untouched. Row 6's readback
and restore. Then the honest deltas — above all **any verb whose recorded reasoning you could not
find, or did not believe**.
