# BRIEF — STONE expand-1: remove two, keep one, rename the list

Read `DESIGN-STONE-expand-1-the-allow-list-stops-lying.md` first — especially why the three
contradictions get three different dispositions.

## The work, one paragraph

`src/macros/eval.rs`'s `is_pure_total` blesses 202 verbs for use inside `defmacro` bodies. Three of
them contradict their own registration. **Remove `:wat::hashmap::keys` and `:wat::hashmap::values`;
keep `:wat::i64::/`; rename the function to `is_expand_time_legal` and rewrite its header to say
what it actually decides.**

## Read in order

```
src/macros/eval.rs:337-352   the header. Claims "ONLY the pure-total subset" and claims a
                             completeness mechanism you must TEST (row 4).
src/macros/eval.rs:353       `fn is_pure_total` and its 202-name matches!
src/rete/purity.rs           the `total` sub-list's note on keys/values' nondeterminism —
                             "iteration order is deliberately NOT part of the contract"
src/intrinsic/ast.rs         `fresh-symbol` — nondeterministic BY DESIGN, and it STAYS.
                             Read it so you can see why keys/values are different.
```

## The three dispositions, and they differ

**`hashmap::keys` · `hashmap::values` → REMOVE.** Their nondeterminism is hash iteration order,
which the substrate says is not part of the contract. A macro body folding over them emits
different code on different runs — **expansion must be reproducible.**

**`:wat::i64::/` → KEEP.** It is partial (zero divisor), and that is fine at expand time: a
compile-time failure is better than a runtime one. It was never the lie; the list's name was.

**`fresh-symbol` · `macro-call-site` → KEEP, untouched.** Nondeterministic and correct. If you find
yourself removing them for consistency with keys/values, you have the discriminator backwards —
STOP and re-read the design.

## The rename

`is_pure_total` → `is_expand_time_legal`, at its definition and every call site. **Rewrite the
header**: it must stop claiming "the pure-total subset", and should say what the audit found — the
list blesses one partial verb and two nondeterministic ones on purpose, and default-deny has held
(zero effectful entries).

★ Keep the rune (`rune:struere(invariant-coupling)`) if its reason still holds after the rename; if
the rename makes its wording false, fix the wording rather than dropping the rune.

## ⚠ ROW 4 — TEST THE HEADER'S OWN CLAIM

The header says:

> *"The suite teaches completeness: a false-refusal (a pure head missing from this list) makes a
> stdlib test RED."*

**Never tested.** After removing `keys`/`values`, run the full targeted suite. **Both outcomes are
findings and both are acceptable** — report which:

- **Something goes red** → a macro body really does call them, the mechanism works, and you have
  found a live caller whose expansion was irreproducible. Report it; do NOT re-add the verb to make
  it green without saying so.
- **Nothing goes red** → the mechanism only covers verbs the corpus happens to exercise, and the
  header overstates it. Say so, and correct the header's claim as part of the rewrite.

## Blast radius

`src/macros/eval.rs`, plus call sites of the renamed function.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to remove `fresh-symbol` or `macro-call-site`.** They are nondeterministic and
   correct. STOP.
2. **You are about to remove `:wat::i64::/`.** It is partial and legal; the name was the defect.
   STOP.
3. **Something goes red and you are about to re-add a removed verb to fix it.** That is the finding,
   not an obstacle. STOP and report.
4. **You are about to add any verb to the list**, including from the 174-verb gap. A later stone
   with a declared axis; not hand-curation. STOP.
5. **You are about to mint `@ExpandTimeLegal`.** The next stone. STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: confirm all three contradictions against the registry — keys/values
      Nondeterministic, i64::/ @Totality Partial — and that zero listed verbs are Effectful.
 1. ★ keys AND values REMOVED. i64::/, fresh-symbol, macro-call-site all still present.
 2. ★ RENAMED at the definition and every call site. Name the call sites you found.
 3. ★ THE HEADER REWRITTEN — no "pure-total subset" claim; states the two deliberate
      nondeterministic entries and the one partial entry, each with its reason.
 4. ★ THE COMPLETENESS CLAIM TESTED. Which outcome? Quote any red verbatim.
 5. ★ BREAK THE DOOR: remove one verb that a macro body DOES use (find one — a verb appearing
      inside a `defmacro` in `wat/`), show the red, restore. Proves the gate is live, whatever
      row 4 showed about keys/values specifically.
 6. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 7. cargo nextest run --release -E 'test(macro) + test(stdlib) + test(intrinsic) + test(rete)'
```

★ **Row 5 exists because row 4 might come back empty.** If nothing depends on `keys`/`values`, that
tells us about those two verbs, not about the gate. Row 5 tests the gate itself.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your pre-check. What you removed and what you deliberately kept. The call sites you renamed. The
new header text. Row 4's outcome — red or silent — with any failure verbatim. Row 5's chosen verb,
its red, and the restore. Then the honest deltas — especially **any other entry that looked wrong
to you**, because this audit found three and there may be a fourth nobody has asked about.
