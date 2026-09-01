# DESIGN — STONE: the registry answers FIRST, wave 2 — the named guards

> **Builder, 2026-08-31:** *"the registry must become the single sole authority for these
> properties"* → *"next wave"*.
>
> Wave 1 retired the three PREFIX guesses. This wave takes the NAMED guards that still outrank the
> registry inside the same function.

## The measured state — 19 shadowed, 17 stranded, and the shape never varies

`intrinsic_meta` (`src/rete/purity.rs:246`) now reaches the registry at **line 540**. Fourteen
hand-written verdicts still return before it. Compared against each verb's own registration:

| line | verbs | hand (P,D,T) | registry | verdict |
|---|---|---|---|---|
| 287 | `hashmap::keys` `hashmap::values` `map::keys` `map::values` | T,F,**T** | Pure/Nondet/**Unreviewed** | ⛔ stranded ×4 |
| 370 | `type-params-used-in` | T,T,**T** | Pure/Det/**Unreviewed** | ⛔ stranded |
| 380 | `type-equal?` | T,T,**T** | Pure/Det/**Unreviewed** | ⛔ stranded |
| 392 | `stream::empty` `stream::cons` | T,T,**T** | Pure/Det/**Unreviewed** | ⛔ stranded ×2 |
| 438 | `rete::pure?` `deterministic?` `total?` `primitive?` `vocabulary-admitted?` `cond-has-deferred-constraint?` | T,T,**T** | Pure/Det/**Unreviewed** | ⛔ stranded ×6 |
| 460 | `rete::alpha-match` `-local` `-under` | T,T,**T** | Pure/Det/**Unreviewed** | ⛔ stranded ×3 |
| 258 | `uuid::v4` | T,F,F | Pure/Nondet/Unreviewed | ✅ AGREES |
| 409 | `stream::next` | F,F,F | Effectful/Nondet/Unreviewed | ✅ AGREES |

★ **Seventeen stranded facts, and every one differs on TOTALITY ONLY.** Purity and determinism agree
at all nineteen sites — so those two axes cannot regress, exactly as in wave 1.

★★ **Two guards are pure duplication.** `uuid::v4` and `stream::next` already say what their
registrations say. Deleting them is free and changes nothing — and each is a small proof that the
registry is now capable of carrying the answer.

★★★ **And the self-referential one, which is the reason this wave is worth doing carefully:**
`:wat::rete::total?` / `pure?` / `deterministic?` are THEMSELVES on the shadowed list (line 438).
The verbs that REPORT the axes are hand-ruled rather than registry-ruled — they were the instrument
wave 1's probe used to observe the registry. The reporter must not be the last thing still lying.

## THE ONE CONTRACT DECISION — pinned, and wave 1 proved why

**A fact moves IN by RE-DERIVATION from the body; it is never transcribed and never re-guessed.**

⚠ Wave 1 is the evidence, not a slogan: `:wat::string::concat` was asserted `Total` by its guess's
own reasoned comment, and came back **`Partial`** — variadic, and `check.rs:14944` says *"the checker
accepts arity 0 … so the runtime owns the diagnostic."* **The stranded fact was WRONG**, and only
re-derivation caught it. Expect this wave to overturn some of its seventeen too.

## ★ THE PREDICTION — uneven, and falsifiable in both directions

```
2 guards (uuid::v4, stream::next)   agree already   ->  delete, ZERO change
17 stranded facts                   re-derived      ->  most confirm, SOME WILL NOT
purity / determinism at all 19      already agree   ->  ZERO change on those axes
```

⚠ **A wave in which all seventeen confirm is a RESULT TO DISTRUST**, given wave 1's 10-of-11. If
every one matches, say so and cite the bodies — but a uniform confirmation after a non-uniform
precedent deserves a second look before it ships.

## ⛔ WHAT THIS DESIGN DOES NOT CLAIM

Wave 1's DESIGN asserted a consequence — *"these are admitted into `where` fences"* — that was
**false**, because `compile-condition` also requires `is-rete` (Law A) and refuses core-spelled
computation unconditionally. I had the probe in hand and did not point it at the claim.
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

**So this DESIGN claims only what was measured:** the registry declares `Unreviewed`; a hand verdict
declares `total: true`; the hand wins because it answers first. **The observable consumer is
`:wat::rete::total?`** — measured in wave 1, not assumed here. ⚠ Some of these verbs ARE
`:wat::rete::`-namespaced and so may additionally clear Law A; **whether any fence behaviour changes
is for the rider to MEASURE, not for this DESIGN to predict.**

## Out of scope = REJECTED (not deferred)

- **The nine UNREGISTERED heads** — `aggregate-new` · `kwargs-construct` · `write-forms` ·
  `with-children` · `macro-error` · `verify::{string,http-path,s3-path,file-path}`. They carry no
  registration, so there is nowhere for their fact to move. **Homing them is a different kind of
  work** — a registration stone, not a retirement stone — and mixing the two would make a red
  un-attributable. Their guards stay.
- **`rete_op_for` (line 251)** — a whole vocabulary table with its own consumers and its own
  `OpMeta` per row. It is the largest remaining authority and it needs its own design.
- **`accessor_meta` / `constructor_meta`** — derive from the frozen TypeEnv, not from a name. Arc
  293.W owns that ruling; `[[NOTE-a-nature-is-a-transport-fact-not-a-purity-verdict]]`.
- **Measuring any verb not on the nineteen.** They stay `Unreviewed`, honestly.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **move the 17, delete the 2 duplicates, leave the 9 unregistered** | YES | YES | YES | YES | ✅ **ADMITTED** |
| also home the 9 unregistered in this stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| delete all 14 guards, leave the 17 `Unreviewed` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| transcribe the 17 from the guards | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| take `rete_op_for` too | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |

- **home-the-9 Simple? NO** — nine new registrations plus seventeen fact-moves plus fourteen
  deletions in one stone; a red could not be attributed to a cause.
- **leave-them-Unreviewed Honest? NO** — it discards seventeen measurements and silently narrows
  every consumer. Deleting a fact to retire its container is not a retirement.
- **transcribe Honest? NO** — wave 1 caught a WRONG fact by re-deriving. A copied verdict is a
  hand-list that changed address. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`
- **take-`rete_op_for` Simple? NO** — a table with per-row `OpMeta` and its own consumers; it is a
  design, not a wave.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ each of the 17 carries a re-derived verdict | their `@Totality` lines | measured, each citing its body |
| ★ the two duplicates change nothing | `:wat::rete::total?` on `uuid::v4` / `stream::next` | identical before and after |
| ★ the reporters still report | `:wat::rete::total?` on a known-`Total` and a known-`Unreviewed` verb | still discriminates |
| the 14 guards are gone | `intrinsic_meta` between `rete_op_for` and the registry lookup | no `OpMeta` literal except the 9 unregistered |
| no purity/determinism drift | the 19 declarations | `@Purity`/`@Determinism` unedited |
| the 9 unregistered still answered | their guards | present and unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
