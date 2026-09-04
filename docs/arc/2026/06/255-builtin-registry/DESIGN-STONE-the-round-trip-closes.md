# DESIGN — STONE: the round trip closes — 430/432 → 432/432, and the two gaps are SPELLING

> **Builder, 2026-09-04:** *"both sides of the migration can answer questions..... hand built lists
> can now be interrogated for completeness"* → *"brief and release the two-defect strike"*
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]` items **4** and **5** (what they take,
> what they return). This stone does not migrate signatures. It removes the last two measured
> obstacles to the claim that it COULD.

## What the enumeration made askable

`register_builtins` is the largest hand-list in the campaign — 325 names, **302 of which already
have a registry row**. It has sat on the fork list as *"needs a design stone; signature migration
hasn't started."* Measured this session, from the probe's own output:

```
registered rows WITH a checker scheme ....... 432
round-trip EXACTLY (doc -> TypeExpr == scheme) 430
failed ......................................   2
schemes carrying generics ...................  87
   of those, every var NAMED in the doc types   87
   quantifier NOT recoverable from the doc ...   0
```

**The data was already sufficient at 99.5%, and the generics question — the plausible killer — came
back 87/87 clean.** The number was buried in a `#[test]`'s `eprintln` where nobody could ask it.

## ⛔ THE TWO ARE NOT DATA DEFECTS — read this before touching anything

The probe compares `TypeExpr` values **structurally** (`PartialEq`), deliberately not through
`typeexpr_to_doc_string`. Its own comment says why, and it is the right call:

> *"Comparing through the forward projection is the defect this probe exists to avoid: a lossy
> projection makes two different TypeExprs compare equal… The first draft of this probe did exactly
> that and returned a perfect 386/386."* `[[feedback_a_green_test_can_prove_nothing]]`

Against that bar, the two failures are:

```
:wat::rete::lower    doc `:wat::core::nil` --parse--> TypeExpr::Tuple([])
                     scheme holds           TypeExpr::Path(":wat::core::nil")
                     ⭐ SAME TYPE. `:wat::core::nil` is a TYPEALIAS to `Tuple(vec![])`
                     (`src/types.rs:1069`). The parser resolves it; the scheme does not.

:wat::string::join   scheme:  TypeExpr::Path("T")     ← BARE, no leading colon
                     everywhere else: Path(":T") (what the file's own `t_var()` builds)
                     Its `type_params: vec!["T"]` is fine; the USE site disagrees with the house
                     spelling. Found incidentally during Stone 1c-f and set aside as out of scope.
```

**Neither says the registry's `@arg`/`@ret` cannot express the type.** Both say the two sides spell
the same type differently. Calling them "defects" — as the orchestrator first did — overstates them,
and the overstatement matters because it changes which side you fix.

## The real deliverable: a canonical-form ruling

The campaign's own rule is **one authority per question**. Today there are two spellings for one type
in two places, and Phase 3b (*check asks the registry*) will inherit whichever the round trip blesses.
So the stone's deliverable is not "make the number 432" — it is **decide what canonical means, then
make the number 432 as evidence.**

Two questions, each to be answered by measurement, not preference:

```
Q1  nil        Is the canonical TypeExpr for `:wat::core::nil` the RESOLVED `Tuple([])` or the
               UNRESOLVED `Path(":wat::core::nil")`?
               ⭐ The instrument may be the thing at fault: "can the doc reconstruct the scheme?"
               is a question about TYPES, not SPELLINGS, so resolving aliases on BOTH sides before
               comparing may be the honest fix rather than editing either datum.
Q2  type vars  Is it `Path(":T")` or `Path("T")`? One row disagrees with the rest. Whichever wins,
               the loser is a latent inconsistency that will bite the migration.
```

⚠ **If Q1's answer is "fix the instrument," then this stone ships ONE data change, not two, and the
count is 432/432 for a different reason than the brief assumed.** That outcome is a success, and the
report must say plainly which of the two it was.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **close the round trip** | YES | YES | YES | YES |

- **Obvious? YES** — two rows out of 432 disagree on spelling with everything around them.
- **Simple? YES** — at most one alias resolution and one character. No new mechanism.
- **Honest? YES**, and this is the one that earns the stone: a 430/432 that everyone reads as "two
  defects remain" when it is actually "two spellings differ" is a number that MISLEADS about what
  blocks Phase 3b. Closing it replaces an ambiguous number with an unambiguous one.
- **Good UX? YES** — the next stone inherits one spelling per type instead of two.

## ⛔ WHAT 432/432 WILL NOT MEAN

Stated here so no later reader over-reads the number this stone produces:

- **It does not mean DEBT falls.** The probe opens `let Some(scheme) = check_env.get(name) else
  { continue }` — it has **never looked at a single one of the 121 DEBT rows**, which have no scheme
  at all. The SEAM already carries this warning; this stone does not lift it.
- **It does not mean `register_builtins` can be deleted.** It means the DATA is sufficient for the
  432 rows that have both. `CheckEnv` still does not ask, and making it ask is Phase 3b.
- **It does not mean the schemes are RIGHT.** The round trip proves doc and scheme AGREE. Stone 1c-f
  found both agreeing on a `Vector` that `infer_foldl` had not accepted since 118.B6 — **two sides
  can agree and both be stale.** `[[feedback_two_instruments_agreeing_is_not_corroboration]]`

## Scope

**In:** Q1 and Q2 answered by measurement · whichever single change each implies · the probe re-run
to 432/432 · the canonical answers recorded where the next stone will find them.

**Out, affirmatively:** Phase 3b itself (making `CheckEnv` ask) · the 121 DEBT rows · any other
scheme's content · the three RETE_OPS `core_name` orphans and the 34 syntax-less special forms
found in the same census — named as separate targets, not folded in here.
