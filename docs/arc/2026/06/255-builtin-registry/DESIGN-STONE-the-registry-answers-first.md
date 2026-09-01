# DESIGN — STONE: the registry answers FIRST — retire the three prefix guesses

> **Builder, 2026-08-31:** *"the registry must become the single sole authority for these
> properties.... we continue...."*
>
> The code already filed this work itself. `src/rete/purity.rs`, above the registry lookup:
> *"this sits AFTER `rete_op_for` and every early-return special case above (each already verified
> to agree with its own registration — **retiring them is a follow-up, not this stone**)."*
> This is that follow-up, and its first wave.

## ⛔ THE REGISTRY IS CONSULTED LAST

`intrinsic_meta` (`src/rete/purity.rs:246`) reaches the registry at **line 578**. Seventeen
hand-written verdicts return before it:

```
251  rete_op_for(head)                     the RETE_OPS vocabulary table
257  :wat::uuid::v4
283  hashmap::keys/values · map::keys/values
298  :wat::string::  ·  :wat::regex::      ⛔ A PREFIX GUESS
330  :wat::edn::                            ⛔ A PREFIX GUESS
398  aggregate-new · kwargs-construct       407  type-params-used-in     417  type-equal?
429  stream::empty · stream::cons           446  stream::next
467  rete::alpha-match ×3                   494  verify::string/http-path/s3-path
513  write-forms   523  with-children   530  macro-error   540  (a set)   552  verify::file-path
────
578  THE REGISTRY                           <- finally
```

★ Three of them are **prefix guesses** — the same mechanism the W7 NOTE disqualified for
`effectful_by_prefix`, here outranking the registry rather than backstopping it. And two carry
**hand-lists nested inside the guess**, which is `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`
by construction — the shape this arc has already retired twice elsewhere.

## ★★ THE MEASURED STATE — and the eleven stranded facts

The three prefix guesses shadow **34 registered verbs** (`string` 20 · `edn` 13 · `regex` 1). Every
one of the 34 declares, at its own registration site:

```
@Purity Pure   ·   @Determinism Deterministic   ·   @Totality Unreviewed        (34 of 34)
```

The guess **agrees** on purity and determinism for all 34 — so those two axes cannot regress. It
**disagrees** on totality for exactly eleven, and always in the same direction:

```
:wat::string::length · trim · to-lowercase · concat · contains? · starts-with? · ends-with? · empty?
:wat::edn::read-foreign · ForeignRecord/get · ForeignRecord/class
```

For these eleven the registry says `Unreviewed` and the shadow says `total: true`.

## ⛔⛔ REFUTED 2026-08-31 BY THE RIDER — THE FENCE CLAIM BELOW WAS FALSE

The section below said eleven `Unreviewed` verbs **"are admitted into `where` fences today."**
**They are not, and never were.** The rider built the probe this DESIGN asked for, ran it, and got a
refusal the DESIGN did not predict. Re-verified by the orchestrator against the pre-stone binary:

```
(where (:wat::rete::i64::> (:wat::string::length ?s) 3))
  => "where expr is not a rete primitive — ':wat::string::length' is not a rete primitive;
      a where admits only :wat::rete:: ops"
```

`wat/rete/compile.wat`'s `compile-condition` requires **`is-pure ∧ is-det ∧ is-total ∧ is-rete`**,
and `classify_expr`'s own doc says it outright: *"being pure, deterministic and total does not make
an op rete — `:wat::core::>` is all three and is still refused."* **None of the eleven are rete
vocabulary members**, so Law A refuses them unconditionally, before and after this stone. Their
totality never reached the fence at all.

⚠ **I had the instrument and did not point it at the claim.** The fence probes were built earlier
the same session, for a different stone. I asserted a consequence I never tested — the SECOND time in
two consecutive stones that a bar was written from what I expected rather than derived from the rule
(`struct-field`'s "REFUSED → ADMITTED" was the first).
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## ★ WHAT SURVIVES, AND WHY THE STONE IS STILL RIGHT

The **two-authorities defect is real and unchanged**: the registry declares `Unreviewed`, a prefix
guess asserts `total: true`, and the guess wins because it answers first. What was wrong was only the
CONSEQUENCE I claimed. The real consumers are the single-axis reflection verbs —
`:wat::rete::total?` / `pure?` / `deterministic?` — which read one axis with no Law A conjunct, and
which the guess answers for today.

★★ And the re-derivation contract earned its place immediately: `:wat::string::concat` came back
**`Partial`**, not `Total`. It is variadic and `check.rs:14944` says *"the checker accepts arity 0 …
so the runtime owns the diagnostic"* — a well-typed program reaches a raise. The guess's comment
claimed *"always return for any two strings"*, quietly assuming arity 2. **The stranded fact was not
merely stranded; it was WRONG**, and transcription would have carried it in.

⛔ **That defeats the default-deny.** `wat/runtime-meta.wat` on `Unreviewed`:

> *"NOBODY HAS MEASURED THIS VERB YET. Not a pole, not a guess. **Default-deny: it does NOT satisfy
> the `where`-fence**, so an unreviewed verb is refused rather than admitted."*

Eleven verbs the registry marks unmeasured are admitted into `where` fences today.

★★★ **But this is a fact STRANDED, not a fact contradicted.** The guess's own comments *argue* each
one — *"`concat`/`contains?`/`starts-with?`/`ends-with?` always return for any two strings"* — and
arc 255 Stone F says outright it listed them there *"so the fact survives the move."* The
measurement was done. It simply lives where the registry cannot see it.

## THE ONE CONTRACT DECISION — pinned

**A fact moves IN; it is never deleted and never re-guessed.** Each of the eleven verdicts is
re-derived by reading the body, written as `@Totality` at that verb's own registration, and the
prefix guess is then deleted. A verdict that cannot be re-derived from its body is a **STOP**, not a
transcription.

## ★ THE PREDICTION — uneven, and falsifiable in both directions

```
the 11 stranded verbs   fact moves to @Totality   ->  where-fence behaviour UNCHANGED
the other 23            guess said total:false
                        Unreviewed default-denies ->  where-fence behaviour UNCHANGED
purity / determinism    guess agrees with all 34  ->  ZERO change on those axes
```

**The whole stone is behaviour-preserving.** If any `where` clause flips admitted↔refused, a fact
failed to move and that is the finding.

⚠ **This DESIGN publishes no absolute `@Totality` census.** My own parser and the seam's instrument
disagree by ~5 on the totals, and **five instruments have already been caught contaminated this
session** (prose counted as symbols; a ledger count that swept from a comment; an arity read from the
wrong guard; a doc block whose back-walk stopped early; and this). The eleven are enumerated **by
name**, verified individually — that is the claim, and it is the only one that needs to be true.

## Out of scope = REJECTED (not deferred)

- **The other fourteen early returns** (`rete_op_for`, the singletons, the three `matches!` sets).
  Same campaign, later waves. This wave takes the three that are PREFIX GUESSES, because they are the
  disqualified mechanism and they hold every stranded fact found so far.
- **`accessor_meta` / `constructor_meta`.** They derive from the frozen TypeEnv, not from a name — a
  different question, and `[[NOTE-a-nature-is-a-transport-fact-not-a-purity-verdict]]` records the
  ruling arc 293.W owns.
- **`effectful_by_prefix`.** Untouched. It is a FALLBACK below the registry, not an authority above
  it — the opposite shape from the three retired here.
- **Measuring the other 23's real totality.** They stay `Unreviewed`, honestly. Guessing them to
  empty a ledger is the failure `Unreviewed` exists to prevent.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **move the 11 facts in, then delete the 3 guesses** | YES | YES | YES | YES | ✅ **ADMITTED** |
| delete the guesses, leave all 34 `Unreviewed` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| transcribe the 11 from the comment without re-reading | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| move the registry lookup to the TOP, delete nothing | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| retire all 17 authorities in one stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |

- **leave-them-Unreviewed Honest? NO** — it discards eleven measurements the substrate paid for and
  silently narrows the fence. Deleting a fact to retire its container is not a retirement.
- **transcribe-without-reading Honest? NO** — a copied verdict is a hand-list that moved house. The
  arc's own lesson: a gate reading a copy of the truth cannot fail.
- **registry-to-the-top Simple? NO / Honest? NO** — sixteen unreachable verdicts left in place read
  as live rules; and the eleven stranded facts would vanish silently the moment the registry answered
  first, flipping the fence with no diff explaining why.
- **all-17-at-once Simple? NO** — `rete_op_for` is a whole vocabulary table with its own consumers; a
  red could not be attributed to a wave.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ the eleven carry real verdicts | their `@Totality` lines | measured per verb, each citing its body |
| ★ the single-axis reflection is UNCHANGED | `:wat::rete::total?` on each of the eleven | same answer as before the cut |
| ★ Law A is untouched | a `where` using any of the eleven | REFUSED as "not a rete primitive", before AND after |
| the three guesses are gone | `grep 'starts_with(":wat::string::' src/rete/purity.rs` | 0 hits |
| the registry moved up | `intrinsic_meta`'s lookup | three fewer verdicts above it |
| no purity/determinism drift | the 34 declarations | still `Pure ∧ Deterministic`, unedited |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
