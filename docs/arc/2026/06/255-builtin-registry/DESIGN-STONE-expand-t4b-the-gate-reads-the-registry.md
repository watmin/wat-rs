# DESIGN — STONE expand-T4b: the expand-time gate DERIVES; the hand-list becomes a backlog

> T4a (`f84f37ba0`) moved 141 blessings to their registration sites. This stone makes
> `is_expand_time_legal` read them, and collapses the duplicate T4a opened.

## The mechanism today

```rust
fn is_expand_time_legal(head: &str) -> bool {
    matches!(head, ":wat::i64::+" | ":wat::f64::<" | … )   // 202 names
}
```

141 of those now carry the answer at their own site. The `matches!` is a second copy of a fact the
registry holds — the shape 255.1c retired as *"a gate reading a copy of the truth"*
(`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).

## The derivation

```rust
if let Some(e) = registry().lookup_entry(head) {
    return matches!(e.expand_time, ExpandTime::Legal | ExpandTime::Preserving);
}
matches!(head, /* the unregistered residue */)
```

`Unreviewed` and `RuntimeOnly` both yield `false` — default-deny, and the reason `Unreviewed` was
minted as a fourth variant rather than folded into a pole.

## ⛔ A PREREQUISITE THIS STONE MUST DISCHARGE FIRST — measured

Deriving today moves exactly two verdicts:

```
VERDICT_FLIPS=2   :wat::hashmap::keys    true -> false
                  :wat::hashmap::values  true -> false
```

**Those are the two T4a's rider correctly withheld**, because the code blessed them while a stale
comment claimed expand-1 had removed them. The comment is now corrected and says they STAY — but
their directives still read `Unreviewed`, so a derivation would refuse them.

★ **And that refusal is precisely the change that took 247 tests red**, by making
`:wat::core::format` undefinable — `format` calls `keys` at expand time
(`wat/core.wat:1939`). So this stone **annotates `keys`/`values` as `@ExpandTime Legal` first**,
with the reasoning already written into `eval.rs`'s corrected comment, and only then derives.

That is not scope creep; it is the deferred half of a correction. T4a withheld a transcription
because the file contradicted itself. The contradiction is resolved, so the transcription lands.

## The one contract decision, pinned: THE VERDICTS DO NOT MOVE

**For every name the predicate is asked about, the answer must be exactly what it is today.** The
derivation changes *where* the answer comes from, never the answer. After the two annotations,
`VERDICT_FLIPS` must be **0** — measure it before and after.

`:wat::core::format` is the live consumer and the canary: if it cannot be defined, the stone is
wrong.

## What the residue becomes

The fall-back `matches!` holds the allow-list's **unregistered** names — measured at T4a as **59**.
Every one is a verb with no registration site to carry its blessing. **It stops being a hand-list
and becomes a homing backlog**, each row retiring when its verb gets a home — the same shape as
totality's 11.

★ It must carry a comment saying so. An unexplained 59-name `matches!` reads exactly like the
202-name list it replaced.

## What does NOT close here

**The 174-verb gap.** 288 registered verbs read `Unreviewed`, and ~174 of them are pure ∧
deterministic and probably belong on the list. Derivation does not rule on them — it makes the gap
**visible in the source**, at each verb, where before it was invisible by construction. Closing it
is a census with a place to write the answer, which is what the axis bought.

## Out of scope = REJECTED

- **Ruling any of the 288.** Not this stone.
- **`RuntimeOnly` on anything.** Still zero verbs ruled runtime-only; that verdict needs a maker.
- **The `expand-twice-and-compare` instrument.** Named in `eval.rs`'s header as the honest way to
  catch order-dependence; building it is its own stone.

## Calibration

Predicted 30–45 min. Two annotations, one lookup, one residue, and a verdict diff over the whole
population.
