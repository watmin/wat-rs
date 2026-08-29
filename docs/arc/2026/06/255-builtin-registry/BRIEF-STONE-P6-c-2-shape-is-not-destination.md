# STONE P6-c-2 — SHAPE is not DESTINATION, and a ruling must outlive the rule it was made under

> Read `NOTE-p6c-is-a-campaign-not-a-stone.md`. **This stone must land before ANY homing wave** —
> the instrument that would drive those waves currently green-lights homing `:wat::core::if`.

## The defect, measured 2026-08-28 after Stone P6-c-1

P6-c-0's rider hand-ruled `quote`, `quasiquote` and `fn` **SPECIAL-FORM** by reading them, and
flagged `if`, `do`, `match` **UNKNOWN-with-reason** rather than guess. Those were correct judgements.
They lived in a report. P6-c-1 then widened the shape rule, and the instrument now says:

```
:wat::core::quote        INTRINSIC-READY      ← hand-ruled SPECIAL-FORM by P6-c-0
:wat::core::quasiquote   INTRINSIC-READY      ← hand-ruled SPECIAL-FORM
:wat::core::fn           INTRINSIC-READY      ← hand-ruled SPECIAL-FORM
:wat::core::if           INTRINSIC-READY      ← hand-flagged UNKNOWN
:wat::core::do           INTRINSIC-READY      ← hand-flagged UNKNOWN
:wat::core::match        INTRINSIC-READY      ← hand-flagged UNKNOWN
:wat::stream::lazy       SPECIAL-FORM         ← survived ONLY because its marker is comment-text
```

**A judgement was made, was right, and the instrument recomputed over it.** Anyone driving a wave
off this tool today would register `:wat::core::if` as an intrinsic.

## ★ THE CONTRACT DECISION — two axes, and one of them is never computed

`INTRINSIC-READY` conflates two independent questions:

```
SHAPE        does this signature fit `#[wat_intrinsic]`?      ← the tool measures this, correctly
DESTINATION  should this verb BE an intrinsic at all?          ← nobody measures this. It is RULED.
```

**Shape must never imply destination.** The tool reports SHAPE as it does today, and carries
DESTINATION as a **FROZEN NAMED LEDGER** it reads rather than derives — same discipline as
`FROZEN_CHECKER_DEBT_LEDGER` and `KNOWN_UNREVIEWED`: names, never a count, and it must go red when a
name in it stops matching reality. `[[feedback_a_gate_freezes_names_never_a_count]]`

A row is homeable only when **SHAPE=fits AND DESTINATION=intrinsic**. Anything else prints both and
is not a candidate.

⚠ **`:wat::stream::lazy` is the warning, not the model.** It kept its ruling by accident — its
marker is prose in a leading comment, so a text scan happened to find it. A ruling that survives
because of where somebody put a comment is not carried; it is lucky.

## The work

1. **A frozen DESTINATION ledger in the instrument**, seeded with the rulings P6-c-0 made and this
   NOTE records: `SPECIAL-FORM` = `quote` · `quasiquote` · `fn` · `stream::lazy`;
   `DECLARATION-GUARD` = `core::def` · `core::defclause`; `UNKNOWN-RULED-PENDING` = `if` · `do` ·
   `match` · and the CONTROL-FLOW-MULTI-MODE set (`let` · `and` · `or` · `ann-form`).
2. **Report both axes per row.** `SHAPE=fits DESTINATION=special-form` must be unmistakable, and the
   homeable set must be printed as its own explicit count.
3. **Stop deriving SPECIAL-FORM from comment text.** Delete that heuristic or demote it to a
   *suggestion* that can only ADD a candidate to review, never decide one. It produced exactly one
   correct answer by luck and zero others.
4. **Fix the two known false negatives** the P6-c-1 rider flagged and correctly did not patch:
   `eval_and`/`eval_or` (`src/runtime.rs:11409`) declare a legal tail, but an inline
   `// rune:lint(unused-span)` on the trailing param corrupts `find_fn_signature`'s splitter into
   reading it as by-value. Strip comments before splitting parameters.

## STOP triggers — each REJECTS.

1. **A name in the frozen ledger no longer appears in the match.** The ledger must go red, not
   silently skip it. That is the whole point of freezing names.
2. **You cannot rule a verb.** Put it in `UNKNOWN-RULED-PENDING` with its reason. An UNKNOWN that is
   *recorded* is the deliverable; a guess is not.
3. **You find yourself homing a verb.** This stone moves no arm and changes no runtime behaviour.
4. **Stripping comments changes any row other than `eval_and`/`eval_or`.** Report it — the splitter
   was wrong somewhere else too and that is a finding.

## Acceptance

```
 0. ★ REPRODUCE THE REGRESSION FIRST. Run the instrument as it stands and paste the six rows above
      verbatim. A fix whose defect was never shown is not a fix.
 1. ★ AFTER: those six read SHAPE=fits with DESTINATION=special-form / declaration-guard /
      unknown-pending, and NONE of them appears in the homeable set.
 2. ★ THE LEDGER GOES RED WHEN IT SHOULD. Temporarily rename one ledgered FQDN to a name not in the
      match; the instrument must fail loudly naming it. Paste it. Restore.
 3. ★ THE HOMEABLE COUNT IS PRINTED AS ITS OWN NUMBER, and it is 118 minus the ruled-out. Say what
      it is; that number is the campaign's real size and every later wave is drawn from it.
 4. ★ eval_and / eval_or MOVE from NEEDS-SHAPE to SHAPE=fits, and NO OTHER ROW CHANGES because of
      the comment-stripping fix. Diff the full table before and after that change alone.
 5. ★ THE COMMENT HEURISTIC IS GONE OR DEMOTED. Say which, and show `:wat::stream::lazy` still
      lands SPECIAL-FORM — now from the LEDGER, not from prose.
 6. cargo build --release --all-targets — clean. No Rust should change; say so if the diff is the
      instrument only, which is the expected shape.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

The six regression rows before and after. The red-when-stale proof. The homeable count. The
before/after table diff for the comment-stripping fix alone. Then the honest deltas — especially any
verb you could not rule, and why.
