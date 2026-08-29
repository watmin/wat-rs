# STONE P6-c-3 — DEFAULT-DENY. An unruled verb is not homeable.

> **Builder's ruling, 2026-08-28:** *"default-deny - the heretics are set ablaze by their words -
> they self identify"*
>
> Read `BRIEF-STONE-P6-c-2-shape-is-not-destination.md` first — it split SHAPE from DESTINATION and
> built the frozen ledger. This stone flips that ledger's default and makes a ruling cost something.

## The defect this closes

P6-c-2 shipped `DESTINATION_DEFAULT = "INTRINSIC"`: a verb nobody has ruled on reads as **homeable**.
That is a blanket-accept — **the exact shape of `src/resolve/walk.rs:268`, the one line this entire
arc exists to kill:**

```rust
if is_reserved_prefix(head) { return true }     // arc 255's thesis, in one line
```

The campaign inherited its parent arc's defect. 111 verbs are currently green-lit by *silence*.

## ★ THE CONTRACT DECISION — deny by default, and a ruling must be EARNED

1. **`DESTINATION_DEFAULT` becomes `UNRULED`, and `UNRULED` is NOT homeable.** The homeable set on
   the first run after this stone is **ZERO**. Every one of the 111 must be ruled back in, by
   reading it. That number climbing is the campaign's progress meter.

2. **A ruling is a `(destination, reason)` PAIR, and the reason is load-bearing.** A name alone is
   not a ruling — it is a name on a list, which is what this stone exists to stop. The instrument
   **refuses** a ledger row whose reason is missing, empty, or boilerplate, and says which row.

3. ⛔ **THE LEDGER IS NEVER BULK-FILLED.** Writing 111 rulings in one pass produces a hand-list
   nobody earned and re-creates the blanket-accept with more typing.
   `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]` **This stone rules NOTHING new.** It
   flips the default, adds the reason requirement, and carries forward exactly the 13 rulings P6-c-2
   already earned by reading. Each later wave rules its own verbs, and that ruling is the wave's
   work — not a formality it completes on the way to the code.

★ **"The heretics self-identify."** The builder's phrasing is the mechanism, not decoration: a verb
that cannot be ruled INTRINSIC reveals itself the moment somebody tries to write the reason. That is
what happened in O-iv, where riders refused nineteen verbs across three stones and were right every
time — each refusal arrived as a *sentence that could not be written*.

## The work

- Flip the default to `UNRULED`; `UNRULED` never enters the homeable set.
- `DESTINATION_LEDGER` rows become `(fqdn, destination, reason)`. Validate at load: a row missing a
  reason, or carrying an empty/placeholder one, is a FATAL like the stale-name check P6-c-2 built.
- Backfill reasons for the 13 existing rulings **from what P6-c-0 and the NOTE actually recorded** —
  not invented. If a ruling's reason is not on disk, mark it `UNKNOWN-RULED-PENDING` rather than
  writing prose to fill the slot.
- Report three counts, unmistakably: **HOMEABLE** (ruled INTRINSIC ∧ shape fits) · **AWAITING A
  RULING** (shape fits ∧ unruled — the worklist) · **RULED OUT** (ruled non-intrinsic).
- Keep P6-c-2's stale-name FATAL exactly as it is.

## STOP triggers — each REJECTS.

1. **You are about to rule a verb.** You are not. Thirteen rulings exist; carry them. If you believe
   a fourteenth is obvious, say which and why in the report and leave it UNRULED.
2. **A reason you would have to invent.** Leave it `UNKNOWN-RULED-PENDING` and name it. An unruled
   verb costs a wave ten minutes; a fabricated reason costs the ledger its meaning.
3. **The homeable count is not ZERO after the flip.** Then something is still defaulting to accept —
   find it before going further.
4. **You find yourself editing Rust.** This stone touches the instrument only.

## Acceptance

```
 0. ★ BEFORE: paste the current three-line summary (HOMEABLE 111 of 146).
 1. ★ AFTER: HOMEABLE = 0, AWAITING A RULING = 111, RULED OUT = 10. If those do not sum the way
      P6-c-2's 121/10/111 did, reconcile it in the report — do not adjust a number to fit.
 2. ★ THE REASON REQUIREMENT IS PROVEN BY BREAKING IT. Blank one existing ruling's reason; the
      instrument must go FATAL naming that row. Paste it. Restore; output byte-identical.
 3. ★ THE STALE-NAME FATAL STILL FIRES — P6-c-2's guarantee, re-proven on a name of your choosing.
 4. ★ EVERY ONE OF THE 13 REASONS TRACES TO DISK. For each, cite the file/line or the artifact it
      came from. Any you could not source is `UNKNOWN-RULED-PENDING` and listed as such.
 5. ★ A RULED-INTRINSIC VERB IS STILL HOMEABLE — add ONE temporary ruling for a verb of your
      choosing with a real sourced reason, confirm it enters the homeable set, then REMOVE it and
      confirm the count returns to 0. This proves the path works without spending a ruling.
 6. cargo build --release --all-targets — clean. No Rust should change.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

The before and after summaries. Both FATAL proofs, verbatim. The 13 reasons with their disk
citations, and any you had to leave pending. The add-then-remove proof from row 5. Then the honest
deltas — especially any verb you believe is obviously rulable and deliberately left alone.
