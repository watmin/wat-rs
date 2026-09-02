# EXPECTATIONS — a label nothing defines is a label nothing can contradict

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,287 plus every arm you drive.**

## The scorecard, with pre-values measured at HEAD `0bb2107e4`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | vocabularies defined | `sequi` **2** mentions in CONVENTIONS.md; `perspicere` **0**; `purgare` **0** | both defined, in `sequi`'s shape |
| 2 | ★ definition precedes the gate | — | the meanings are written from the **reasons**, not derived from the categories in use |
| 3 | the census.rs six | 6 × `read-once`, every reason *"alias would be a mumble"* | re-categorised, or a finding explaining why not |
| 4 | the gate | none | refuses an unknown category, modelled on `no_unknown_sequi_rune.rs` |
| 5 | STOP-3's claim | row says `trait-contract` names a mechanism absent at all 3 sites — **UNVERIFIED** | verified or refuted, and **said which** |
| 6 | non-vacuity | — | declared, with a real floor |
| 7 | radius | — | `CONVENTIONS.md` + one gate + comment-only `src/` edits |
| 8 | lints | **173/173** (measured) | green |
| 9 | floor | **5287/5287** (measured) | ≥ 5,287, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **An invented category** (`rune:perspicere(zzz-not-a-category)`) → RED, naming file and category.
2. **★ A category in the set used where its DEFINITION does not fit** → this is the one the gate
   probably *cannot* catch, and I want it said plainly rather than implied. **State what the gate
   checks and what it does not**: spelling is machine-checkable, fit is not.
3. **Blind the vocabulary** (empty set) → the non-vacuity floor REDs, not a silent pass over zero
   categories.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

60–80 minutes, and most of it is reading 27 sites to write two honest definitions. The gate is a
copy of one that exists.

## What would make this strike a failure even if every test passes

**Deriving the vocabulary from the categories currently in use.** The gate goes green over all 27
sites, the six mislabelled ones are frozen behind it, and the strike has converted a live defect
into a permanent one with a passing test beside it. Trap 1, row 2, and the ⚠ all exist for this.

The second: **a definition written to fit the label rather than the reason.** `read-once` can be
defined to cover census.rs's six if you try — and then the vocabulary means whatever was already
written, which is not a definition, it is a transcript.
