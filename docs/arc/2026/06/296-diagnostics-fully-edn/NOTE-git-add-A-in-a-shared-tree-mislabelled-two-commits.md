# NOTE — `git add -A` in a peer-shared tree mislabelled two commits, and the second was the fix for the first

**2026-09-08.** Recorded because it happened TWICE in fifteen minutes, and the second time was
inside the commit written to correct the first.

## What happened

```
8ac76586c   message: "Docs only. Still RED, still unpushed."
            actual:  15 files. THIRTEEN are the peer's in-flight RELAND-1 work —
                     the four-store wiring, the three phantom rewrites.
                     Caught by the RIDER, in honest delta 4 of its own SCORE.

<the fix>   message: "Docs only — and this time that is measured, not asserted."
            actual:  9 files. SEVEN are the peer's RELAND-1 remainder.
                     Caught by me, reading `git show --stat` AFTER the commit.
```

The second message asserts the exact discipline it violates, in the same breath, about itself.

## The root is not carelessness — it is `git add -A`

Every prior firing of `[[feedback_i_committed_on_a_non_quiescent_tree]]` was read as an attention
failure, and the remedy was attention. **Four firings say the remedy does not work.** The affordance
is the defect: `git add -A` stages a WHOLE TREE, and in a peer-shared checkout the tree is not
mine to describe. A commit message is a claim about content, and `-A` makes that claim unfalsifiable
at the moment it is written — the stat that would refute it only exists after the commit.

Same shape as FM 19's amendment: a capable hand told to be careful with a dangerous affordance
keeps taking the obvious move. One rung up:

> ⛔ **Never `git add -A` (or `git add .`) in this repo. Name every path.**
> `git add <path> [<path>…]` — then the message and the content are written from the same list, and
> a wrong claim is impossible rather than merely discouraged.

⚠ **Read `git show --stat` BEFORE writing the message, not after.** Staging first and reading the
staged set (`git diff --cached --stat`) makes the message a REPORT of a measurement instead of a
prediction about one.

## What was NOT lost

Nothing. Every file is committed and every change is the peer's real RELAND-1 work. The defect is
purely in the LABELS — two commits assert "docs only" over substrate changes, which is a lie a
future bisect or archaeology will read as fact.

⛔ **Not amended.** Rewriting commits that hold a peer's work is precisely the *"if they accidentally
revert something they shouldn't have, we've lost work we need to recover"* risk the builder named
when ruling that local commits are save-points. The record is corrected forward, here, and the two
messages stay as they shipped. `[[feedback_a_false_confession_is_also_a_false_claim]]` cuts the other
way too: the honest act is to say what the commits contain, not to make them look right.
