# DESIGN — STONE expand-T4a: 143 blessings move to the registration site

> T3 (`985be9a78`) made `@ExpandTime` required; all 431 registrations answer, 430 of them
> `Unreviewed`. The answers exist — in a hand-list, 50 files away from the verbs they describe.

## The scope, measured

```
202  names in `is_expand_time_legal`'s allow-list
143  of them are REGISTERED          ← these get `@ExpandTime Legal`
 59  are not registered              ← the residue; stays in the hand-list until homed
288  registered verbs are NOT listed ← ⛔ THEY STAY `Unreviewed`
```

## ⛔ THE PINNED CONTRACT — an unlisted verb is NOT `RuntimeOnly`

This is the whole design and the easiest thing to get wrong.

**The allow-list is DEFAULT-DENY.** A verb's absence from it means *refused*, and refusal has two
indistinguishable causes: *"deliberately excluded"* and *"never added."* The list itself cannot tell
them apart — which is precisely the defect measured at expand-1:

```
174  pure ∧ deterministic verbs are absent from the list
     not because they are runtime-only, but because nobody added them
```

So transcribing "not listed ⇒ `RuntimeOnly`" would **manufacture 288 verdicts nobody made**, and
174 of them would be wrong. `Unreviewed` is the honest reading of absence, and it is default-deny —
so nothing is admitted that was not admitted before.

★ **T4a moves POSITIVE membership only.** Someone deliberately added each of the 143; that is a
ruling and it relocates. Absence is not a ruling and does not.

## What the 143 carry with them

The list is organised by family with a header per group:

```
// ── Integer arithmetic (pure, total, wrapping) ────────────────
// ── Integer comparison ────────────────────────────────────────
// ── Float arithmetic (pure, IEEE 754) ─────────────────────────
```

Each verb takes its group's sentence and names the group, so the original stays findable — the
shape that worked for totality's T4a. Where a verb has its own inline comment, that travels instead.

⚠ **Do not re-author.** The verdicts are the list's. A rider who *disagrees* with one reports it
rather than transcribing a claim it does not believe.

## What this does NOT change

**Nothing.** No consumer reads `entry.expand_time` yet — `is_expand_time_legal` still answers from
its own list, and T4b makes it derive. So the floor must come back identical, which is this stone's
strongest acceptance criterion and the reason it is separable from the derivation.

## After T4b, the residue

`is_expand_time_legal`'s hand-list will hold **59 unregistered names** — verbs with no registration
site to carry their blessing. Same shape as totality's 11: not a hand-list any more, a **homing
backlog**, each row retiring when its verb gets a home.

★ And the 174-verb gap does not close here. It closes when someone rules on those verbs — which is
now possible for the first time, because there is a place to write the answer.

## Out of scope = REJECTED

- **`is_expand_time_legal`'s derivation.** T4b.
- **Ruling any of the 288.** The gap is real and named; closing it is a census, not a sweep.
- **`RuntimeOnly` on anything.** No verb is being ruled runtime-only by this stone. That verdict
  needs someone to make it.

## Calibration

Predicted 45–70 min. 143 directives across many files, each with attributed reasoning.
