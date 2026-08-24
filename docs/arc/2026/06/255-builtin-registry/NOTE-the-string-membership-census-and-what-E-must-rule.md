# NOTE — the `:wat::string::*` membership census, and the four rulings E needs first

> Measured 2026-08-23 at `78bed2e3f` (stone D shipped). **Recorded because the counts in
> `CHAIN-rendering-before-the-string-home.md` are stale and under, and because three of E's
> decisions are NAMING RULINGS that no measurement can settle.**
>
> ⛔ **SEQUENCING, builder 2026-08-23:** *"i say we update rete along with it... i'll get the rete
> work on the side ready to merge in here... then we'll move string and let rete depart again."*
> **E is BLOCKED on that merge, deliberately.** Rete work landing AFTER the rename would arrive
> carrying the old names, so the merge comes first and the move covers everything in one pass.
> **Every site count below must be RE-TAKEN after the merge** — do not migrate on these numbers.

## The membership — 24, not the chain's 22

The chain's 22 came from an instrument blind to wat-defined verbs. Three of these live in
`wat/string.wat` and a `src/` grep cannot see them (marked ᵛ).

| group | members |
|---|---|
| structural (8) | `concat` · `join` · `split` · `subs` · `length` · `trim` · `interpolate` · `capitalize`ᵛ |
| predicates (3) | `contains?` · `starts-with?` · `ends-with?` |
| case (2) | `to-lowercase` · `to-uppercase` |
| comparison (2) | `=` · `not=` |
| coercions (3) | `to-i64` · `to-f64` · `to-bool` |
| naming conversion (5) | `kebab->pascal`ᵛ · `kebab->pascal-in` · `pascal->kebab` · `pascal->kebab-in` · `declare-acronyms` |
| codemod lineage (1) | `strip-leading-colon`ᵛ |

## The four rulings — builder's, not a rider's

**1. `=` / `not=` — a generic already exists.** `:wat::core::=` and `:wat::core::not=` are both
registered. These two are typed SPECIALIZATIONS beside a live generic. Renaming them to
`:wat::string::=` puts the specialization in one namespace while its generic stays in another. Do
they survive at all, or does the generic subsume them?

**2. `declare-acronyms` is MISFILED, and E would cement it.** It is pre-registered into `macro_sym`
BEFORE freeze (`src/freeze/env.rs:144`, `:278`) and carries its own type-check arm
(`src/check.rs:2672`). It does not transform a string — it **configures the acronym registry** that
`pascal->kebab` reads. A declaration form, arc 265's territory. Rehoming it as a string verb makes
the misfiling permanent.

**3. The coercion fork is LIVE — this is the seam's own item 8.** `to-i64`/`to-f64`/`to-bool` name
the **target**, from the string side. The tree simultaneously carries `:wat::core::keyword/from-string`
and `:wat::core::char/of` — naming the **source**, from the type side. Two conventions, unresolved.
Move these three into `:wat::string::` and they migrate TWICE, which is the exact thing E exists to
prevent ("final names with final signatures, once instead of twice").

**4. `concat` is already cut; `strip-leading-colon` should be questioned.** The chain ruled `concat`
out — String→String vs Vector→Vector is genuine receiver dispatch. And `strip-leading-colon`'s own
header says *"Promoted from `:wat::fix::rename-strip-colon` (Arc 260.1b Part A dedup)"*: codemod
lineage. User-facing verb, or a `fix` helper that got over-promoted?

**Orchestrator's read, for what it is worth:** cut the 3 coercions and `declare-acronyms` from E's
scope, and `concat` stays cut — move the remaining **19**. The fork and the misfiling then get
decided on their merits rather than swept along by a rename.

## ⛔ AND THE CHAIN'S PREMISE FOR THE RETE QUESTION IS FALSE

The chain states, as the basis for its one OPEN question: *"Every string op has a paired
`:wat::rete::core::string::*` row in `rete/vocabulary.rs`."*

**Measured: 6 of 24.** Only `=`, `length`, `not=`, `subs`, `to-lowercase`, `trim` have mirror rows.
The mirror was never complete. So the question is not *"does the mirror follow"* but **"does the
PARTIAL mirror follow, and why are those six the ones that exist"** — and the builder has already
ruled the first half: rete moves with it.

## Site counts — bracketed, and STALE ON PURPOSE

The chain says 1,617 code sites. Measured here with string literals neutralised and comments split:

```
.wat   CODE 1679   COMMENT 19
.rs    ~119 code/literal    44 comment
                            ≈ 1798 code · 63 prose
```

Under by ~180, and the per-directory split is stale too (`wat-scripts` is 874 occurrences against
the chain's 622). **These numbers are pre-merge. Re-take them.** A codemod does not care about the
count, but nothing should quote these forward — that is how the chain's own figures rotted.
