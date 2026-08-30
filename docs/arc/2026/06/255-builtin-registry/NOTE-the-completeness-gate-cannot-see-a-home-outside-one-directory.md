# NOTE — the completeness gate cannot see a home outside ONE directory. 25 verbs are in the hole.

> **Measured 2026-08-29 at HEAD `5725ab10d` (W6 struck). Surfaced by W6's rider, then re-measured
> independently orchestrator-side with the search pattern validated on controls first.**
>
> ⛔ **A finding with a named remedy. NOT drawn — the remedy is a stone, not a trivial fix, and it
> makes ~25 verbs scream at once.**

## What the gate does

`every_dispatched_verb_is_classified_or_disposed` (`src/rete/purity.rs:2686`) is arc 255's
completeness meter. It builds the population of "verbs the runtime dispatches" as a **UNION**:

1. **literal dispatch arms** — text-scanned out of `src/runtime.rs` between two named anchors,
   `fn dispatch_keyword_head_value(` and `fn dispatch_substrate_impl(`; and
2. **`#[wat_intrinsic]` registrations** — read from disk with
   `std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src/intrinsic"))`, files plus **one**
   subdirectory level.

Then it asserts every verb in that population is classified, disposed, or on `KNOWN_UNREVIEWED`.

## The defect, in one sentence

**Homing a verb deletes its literal arm (half 1) and its registration only counts if it landed under
`src/intrinsic/` (half 2) — so a home anywhere else removes the verb from BOTH halves and it leaves
the population entirely.** It does not become *ruled*. It becomes *invisible*.

## ★ AND THE GATE'S OWN COMMENT ALREADY NAMED THIS CLASS

Sitting directly above the `read_dir`, in the gate's own source:

> *"⚠ **ARC 255'S CARVE DRAINS THIS SCAN.** Every home carved out of `runtime.rs`'s literal dispatch
> removes verbs from the only population this scan could see — and a shrinking population makes a
> COMPLETENESS gate report better every stone while measuring less. `:wat::io::` alone moved 23
> verbs and took the count 423 → 400 … So the population is the UNION — literal arms plus every
> `#[wat_intrinsic]`-registered name."*

The diagnosis is exactly right and the remedy was **scoped to one directory**. The drain reopened
the instant a home landed anywhere else, and nothing said so, because the fix's own prose reads as
though the class were closed.
`[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`

## The measurement

```
  429   #[wat_intrinsic] registrations in the tree (anchored grep, git-independent)
  404   under src/intrinsic/          — VISIBLE to the gate
   25   everywhere else               — INVISIBLE
```

The 25, by home, and whether anything else accounts for them:

```
src/runtime.rs        15   argv · body-of · current-thread · extract-arg-names · extract-arg-types
                           field-names-of · field-types-of · lookup-define · metadata-of
                           rename-callable-name · return-type-of · signature-of-defn
                           signature-of-fn · form::matches? · program::cpu-count
src/rete/*.rs         10   arm-session · release-session · collect-rules · eval-insert · eval-test
                           export · import · lower · step-payload · axis-violation
```

**24 of the 25 have ZERO literal dispatch arms AND ZERO mentions anywhere in `purity.rs`** — not on
`KNOWN_UNREVIEWED`, not in `intrinsic_meta`, not covered by a namespace disposition. The 25th
(`:wat::rete::axis-violation`) is mentioned twice in `purity.rs` prose but is likewise not in the
population.

★ **Note what this does to the rete waves specifically.** W5b widened `effectful_by_prefix` to
include `:wat::rete::` so those verbs would be *disposed impure by namespace rule*. A namespace rule
can only dispose a verb the gate can SEE. Ten rete verbs are homed in `src/rete/`, so the rule that
was widened for them never reaches them.

**So the gate reports a population of 510 while the dispatched surface is 535.** A completeness gate
that is 25 short is not a completeness gate.

## Why it did not go red

It cannot. Losing a verb from the population removes work from every downstream count; nothing
asserts the population is COMPLETE, only that everything *in* it is accounted for. The one
non-vacuity guard is `assert!(verbs.len() > 400)` — a floor against the anchors drifting, which 510
clears comfortably while 25 verbs sit outside it.

W6's rider hit the *symptom* by homing in place first, and the message it got describes the wrong
event:

```
4 verb(s) in `KNOWN_UNREVIEWED` are no longer unreviewed — they have been ruled on
(or no longer dispatched). DELETE their lines...
```

They had not been ruled on. The population had shrunk underneath the ledger. The parenthetical
*"(or no longer dispatched)"* is the only hint, and it reads as reassurance rather than as the alarm
it actually is.

## The remedy — and it is one rung, not a sweep

The ladder (`extirpare`):

- **convention** — *"always home into `src/intrinsic/`."* ⛔ Rejected. It is a rule every future hand
  must remember, and W3 already broke it for a per-verb reason that was locally sound.
- **✅ check — make the scan's ROOT the whole tree.** Replace the `src/intrinsic` `read_dir` with a
  recursive walk of `src/` (and `crates/`). Then **where a home lands stops mattering**: the
  population is every registration plus every literal arm, and the blind spot has no location left
  to hide in. This is the cheapest rung that removes the class rather than the instance.
- **unrepresentable** — a macro that refuses to expand outside `src/intrinsic/`. Not obviously
  available (the attribute is a proc-macro; it does not reliably know its own path), and it would
  *forbid* the in-place home rather than *account for* it. The check rung is the right stop.

⚠ **Imposing it makes ~25 verbs scream at once** — they enter the population as UNREVIEWED and the
gate goes red until each is ruled or ledgered. **That is the point.** It is the same move that took
the param-spec wall 2765 → 0: impose the check, read the screams, and let the failures name the work
that four greps could not. `[[feedback_impose_the_check_and_read_the_screams]]`

Expect the stone to be: one scan-root change, then 25 rulings — 15 reflection/runtime verbs and 10
rete verbs, the latter of which should mostly fall straight to the `:wat::rete::` namespace
disposition W5b already widened for them, since the only reason it never applied was that they were
invisible.

## What this does NOT say

It does not say the 25 verbs are broken, unchecked, or unsafe. They dispatch correctly, they are
registered, they type-check, and the floor is green at 5079/5079. **It says the METER has a hole,
and the meter is what six waves of this campaign have been steering by.**

⛔ Not drawn. Builder's ruling on whether this goes before, after, or instead of W7.

---

`DERIVAMVS NE MENTIAMVR.` · `NISI FRANGAS, NIHIL PROBAS.`
