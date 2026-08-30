# BRIEF — STONE expand-T4a: move 143 blessings to their verbs

Read `DESIGN-STONE-expand-t4a-the-blessings-move-home.md` first — especially why an unlisted verb
stays `Unreviewed` rather than becoming `RuntimeOnly`.

## The work, one paragraph

143 registered verbs are blessed by `is_expand_time_legal`'s allow-list while declaring
`@ExpandTime Unreviewed` at their own registration site. **Change each to `@ExpandTime Legal` and
bring its reasoning with it**, as a short `/// **Expand-time ground —** …` paragraph. Everything
else is untouched.

## ⛔ DERIVE THE SET; DO NOT COPY MY NUMBER

> A verb's directive changes **iff** `is_expand_time_legal(name)` returns `true` for it.

The design says 143. **That is a prediction to check, not a list to transcribe.** Write a temporary
test that walks `registry().all_entries()`, partitions by the predicate, and prints both sets.
Work from that output. A disagreement with 143 is a finding about the design — report it.

## Read in order

```
src/macros/eval.rs  fn is_expand_time_legal   THE SOURCE. Organised by family, with a header
                                              comment per group. ⛔ DO NOT EDIT IT — T4b collapses it.
src/intrinsic/ast.rs, `fresh-symbol`          the ONE verb already reading `@ExpandTime Legal`
                                              (T2) — the shape and column alignment to copy
src/intrinsic/i64.rs                          a "Totality ground —" paragraph from total-T4a:
                                              the house shape for carrying a ruling's reasoning
```

## The shape

```rust
/// … existing prose …
///
/// **Expand-time ground —** integer arithmetic: pure, total, wrapping. Safe to evaluate
/// while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Integer arithmetic" group; the
/// verdict is that list's.
///
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Legal
/// @Category      Arithmetic
```

★ **Attribute; do not re-author.** Each paragraph carries that verb's group sentence from the list
and NAMES the group, so a reader can find the original. Where a verb has its own inline comment in
the list, that travels instead.

⛔ **A verb in the list with no prose of its own and no group header gets the honest minimum** —
say it was blessed by the list without a recorded argument. A fabricated justification is worse
than a thin true one.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to write `@ExpandTime RuntimeOnly` on anything.** No verb is ruled runtime-only
   by this stone. Absence from the list is not a verdict. STOP.
2. **You are about to change a verb NOT returned `true` by the predicate.** The 288 unlisted stay
   `Unreviewed`. STOP.
3. **You disagree with one of the blessings.** Report the verb and your argument; do not transcribe
   a verdict you believe is wrong, and do not quietly write something else. STOP.
4. **You are about to edit `is_expand_time_legal`.** T4b's job. STOP.
5. **The floor changes.** Nothing reads `entry.expand_time` yet, so this must be behaviour-neutral.
   A moved count means something reads it that we did not know about. STOP.

## Acceptance

```
 0. ★ YOUR OWN PARTITION: how many registered verbs the predicate blesses, against the design's
      143. Report both, and any names accounting for a difference.
 1. ★ EVERY BLESSED VERB READS `@ExpandTime Legal`, each with its own Expand-time-ground
      paragraph. Quote three in full — one arithmetic, one comparison, one from another family.
 2. ★ THE COUNTS MOVE EXACTLY: `@ExpandTime Legal` 1 → 144 (the 143 plus `fresh-symbol`);
      `Unreviewed` 430 → 287; `RuntimeOnly` and `Preserving` remain 0.
 3. ★ THE 288 ARE UNTOUCHED. Confirm none gained a directive change.
 4. ★ BEHAVIOUR-NEUTRAL: `git diff` touches ONLY `///` lines. State that explicitly.
 5. ★ `git diff --stat src/macros/eval.rs` is EMPTY. Say so.
 6. ★ BREAK THE DOOR: set one of the 143 to `@ExpandTime RuntimeOnly`, show the registry read it
      back as `RuntimeOnly`, restore. Proves the declarations you wrote reach the entry rather
      than sitting in prose.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo nextest run --release -E 'test(intrinsic) + test(macro) + test(reflection)'
```

★ **Row 4 is the safety argument.** Nothing consumes this field yet; if the diff touches anything
but doc comments, or the counts move, the stone did more than it claimed.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.
- A transform tool, if you build one, is a Rust Cargo binary under repo-local `tools/`, deleted
  before you finish; verify per-file non-ASCII counts unchanged.

## Report back with

Your partition against 143. Three quoted paragraphs. The three counts from row 2. Confirmation the
288 are untouched, the diff is doc-lines only, and `eval.rs` is clean. Row 6's readback and restore.
Then the honest deltas — especially **any blessing you could not find an argument for, or did not
believe**, because those are the rows the next census must revisit.
