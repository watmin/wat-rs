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

## ★ ADDENDUM 2026-08-29 — THE SCAN ROOT IS THE WRONG RUNG. THE GATE SHOULD NOT HAVE A LIST AT ALL.

The remedy above (widen the scan root) closes the hole. It is still the **check** rung, and after
walking one level further I do not think it is where this should stop — because the gate's real
defect is not *where it looks for names*. It is **that it keeps its own copy of an answer the
registry already holds.**

**Measured:** all **25** invisible verbs already carry `@Purity` AND `@Determinism` in the registry.
Zero exceptions — the macro *refuses to compile* a `#[wat_intrinsic]` without them
(`wat_intrinsic.rs:562`, `MissingPurity`). **The registry answers for every one of the 25 today.
The gate simply never asks it.**

What the gate asks instead is `intrinsic_meta` (`purity.rs:244`) — **758 lines, 177 verbs, hand
written, and it never touches `registry()` even once** (verified: zero `registry`/`all_entries`/
`entry.` references in its whole body). Of its three axes, **two are verbatim restatements of
mandatory registry fields.** So the population hole and the duplicate table are one defect seen
twice: a verb is "unaccounted for" only because the accounting is kept somewhere other than where
the truth is declared.

★ **Arc 255.1c already ruled exactly this class, one gate over.** `intrinsic/mod.rs:988` retired a
biconditional because, once `is_effectful_op` consulted the registry, comparing them was *"a gate
reading a copy of the truth, unable to fail for a registered row ever again"* —
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`. **`intrinsic_meta` is that same shape and
did not get the ruling.**

### And the blocker is ONE FIELD, already diagnosed, on a LOCKED model

`NOTE-the-registry-asserts-properties-nothing-verifies.md` (2026-08-26) reached this from the other
direction and named the obstruction exactly — its INSTANCE 4-bis. Arc 278's fence is **four** axes:

```
(and (pure? f) (deterministic? f) (total? f) (primitive? f))
```

The Layer-1 BASELINE of `DESIGN.md`'s **LOCKED RECORD MODEL (2026-06-21)** is:

```
name · arity · kind · pure · deterministic · expand_time_legal · defined_in · layer
```

`pure` ✓ · `deterministic` ✓ · **`total` ✗** — and `@Totality` does not exist in `wat-doc` (measured:
0 occurrences). Totality is the one axis with **nowhere else to live**, which is precisely why it
lives in a hand-curated list carrying per-op prose (*`f64::*` is not total — it overflows to ±Inf;
`f64::>` is — its output is a bool*). The two designs never met: 255's model was locked
2026-06-21; 278 invented the fourth axis afterwards, in the only place available.

**So `intrinsic_meta` cannot derive until the registry can hold `total`, and the registry cannot
hold `total` until one field is added to a model explicitly marked LOCKED.** That prior NOTE says
plainly whose call that is: *"a deliberate act on a locked model and the builder's ruling, not a
rider's."*

★ **And the locked model makes the addition self-enforcing.** Layer 1 is specified as *required
fields, enum-typed not bool, **no `Default`** → struct-literal completeness = compile error if any
is unanswered.* Adding `total` therefore **breaks every construction site until every one answers
it** — impose the check, read the screams, built into the model's own design.
`[[feedback_impose_the_check_and_read_the_screams]]`

### What this reorders

The scan-root widening is still correct and still cheap, and it can ship first — but it should be
understood as **a bandage on a gate that ought to be deleted**, not as the fix. If `total` lands in
Layer 1, then:

```
  @Totality mintable at the registration site (the shape is proven: @Purity 290 uses, @Determinism 282)
  → intrinsic_meta DERIVES all four axes instead of restating two and hoarding one
  → the 25-verb hole closes BY CONSTRUCTION (they already declare what the gate wants)
  → macros::is_pure_total deletes; 255.3 can finally land
  → the registry stops being one of six tables and becomes THE table
```

That last line is the arc's whole thesis, and it is one field away.

⛔ **Still not drawn.** The ruling required is narrow and specific: **add `total` to the Layer-1
baseline of the LOCKED RECORD MODEL.** Everything downstream is mechanical.

---

`DERIVAMVS NE MENTIAMVR.` · `NISI FRANGAS, NIHIL PROBAS.`
