# DESIGN-STONE — two rune vocabularies with no definition and no gate

> **Origin (2026-09-01).** Class **F1**, row 3 — the last open F1 lint, after row 4 was struck as
> already satisfied by C1. Driven at HEAD `0bb2107e4`.

## Why — the sibling has both; these have neither

| ward | vocabulary defined? | gated? |
|---|---|---|
| `sequi` | ✅ `docs/CONVENTIONS.md:1055` — *"a CLOSED set of four (2026-08-25)"* | ✅ `tests/lint/no_unknown_sequi_rune.rs` |
| **`perspicere`** | ❌ **zero mentions in CONVENTIONS.md** | ❌ none |
| **`purgare`** | ❌ **zero mentions** | ❌ none |

Both are in live use — `perspicere` across 18 sites in 3 categories (`read-once`,
`intentional-structure`, `mumble-alias`), `purgare` across 9 in 4 (`trait-contract`,
`safety-margin`, `public-api`, `future-fixture`). **Nothing says what any of them mean, and nothing
checks that a rune uses one that exists.**

`sequi` got both **because of exactly this defect**. `arm.rs`'s own note records it: a rune was
*"labelled `host-idiom` beside an identical `ambient-context` neighbour… the categories had no
written definition, so nothing could notice the two disagreeing."*

## The live finding — the category and its reason disagree, six times in one file

`src/rete/kernel/census.rs` carries **six** `rune:perspicere(read-once)` runes. Every one of their
reasons ends *"…; **alias would be a mumble**."*

That is **`mumble-alias`'s** justification, not `read-once`'s — and `mumble-alias` is a real category
in this tree, used at `collection/eval.rs:1898` and `comms/process.rs:327` with reasons of a visibly
different shape (*"turbofish reads better than…"*, *"return type … is…"*). So six sites carry one
category and argue for another, and **nothing can notice, because neither is defined.** The same
shape `sequi` was cured of, in the ward that was never given the cure.

## ★ THE ONE CONTRACT DECISION

**A rune's category comes from a closed, written set, and a gate refuses one that is not in it.**
The definition lands where `sequi`'s already is, and the gate is modelled on the one that already
works — this is a *third instance*, not a new mechanism.

## ⚠ THE DEFINITION MUST BE WRITTEN BEFORE THE GATE, AND THAT ORDER IS THE STRIKE

A gate over an undefined vocabulary can only check spelling. It would pass all 27 sites today —
including the six that are wrong — because every category *in use* would be in the set derived from
use. **Deriving the vocabulary from what is written is how the defect becomes permanent.**

So: read the sites, decide what each category **means**, write that down, and only then let the gate
enforce it. Where a site's reason contradicts its category, the category is what changes — the
reason is the author's actual argument.

## Blast radius

`docs/CONVENTIONS.md` (two vocabularies, beside `sequi`'s), one gate under `tests/lint/`, and the
rune sites whose category the definitions expose as wrong. **No behaviour change** — every `src/`
edit is a comment.

## Out of scope — AFFIRMATIVELY CUT

- **Re-runing sites whose category is right but whose reason is thin.** That is a different quality
  bar and it is not this row's finding.
- **The other wards' runes** (`circumspicere`, `struere`, `solvere`, `intueri`, `lint`, …). If the
  gate is written to scale, say so — but this strike defines and enforces **two**, and a sweep of
  every rune family is its own work.
- **`sequi`'s own vocabulary.** Defined and gated already; untouched.
