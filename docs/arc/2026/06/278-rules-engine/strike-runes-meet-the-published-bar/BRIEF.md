# BRIEF — bring the runes up to the accepted bar

Read `DESIGN.md` first. `excusare` was updated **2026-09-07**; both categories were accepted with
the proposed names, and the published bar is stricter than what two of our four reasons carry.
**Comments and one doc table. Measure nothing new.**

## Read in order

1. `src/rete/kernel/tests/binding_repr_bench.rs:597` — the failing rune. It states the floor and the
   **excursion**; the bar wants the floor and the **margin**.
2. The same function's doc comment — the six-sample table landed there last strike: card 64, trie
   median **676.3 ns** (642.3–716.7), array median **2028.8 ns** (1989.4–2569.8). **That is the
   margin: 3.0×.** Use it; do not re-run.
3. `.config/nextest.toml`, the rete-cohort block — the measured contention band **3.5×–4.4×**
   (8.13→35.39, 7.98→29.42, 13.77→48.72). That is the floor.
4. `binding_repr_bench.rs:264` — the borderline `no-falsifier`. It argues why no ordering can work;
   the bar wants **what you tried**.
5. `binding_repr_bench.rs:146` and `src/rete/kernel/session.rs:1834` — the two that pass. Read them
   so the SCORE can say why, rather than asserting it.
6. `docs/CONVENTIONS.md`'s `rune:excusare` table — flip *proposed upstream* → **accepted**, add
   `fetched 2026-09-07`, and restate the decisive tests in the spell's published wording.

## What ships

**`:597`** — the margin, from the numbers above:

> margin: the array/trie EXTEND ratio at card 64 — medians 2028.8 ns / 676.3 ns = **3.0×**, six
> isolated samples 2026-09-07. Noise floor: this floor's rete-cohort contention band, measured
> **3.5×–4.4×**. The margin sits inside the floor.

Keep the captured red — it is evidence of the consequence, not the margin.

**`:264`** — the attempt: what was driven, and why nothing achievable reds it.

**`docs/CONVENTIONS.md`** — accepted, dated, with the decisive tests as published. Include the
clause `:597` fell to: *measured, and the margin measured from the effect, never read off the
operator.*

## The new void-clause check — one line per site

The spell now says the rune is **void** where another ward owns the finding (`vocare` for a test
restating its implementation, `complectens` for a layer leaning on another's proof, `experiri`'s
`inert` for a surface cell answering the same to every input). **Check all four and report per
site** — a rune covering another ward's finding is not a weaker reason, it is the wrong instrument.

## Verification

No mutation — these are reasons. Report instead:

1. all four reasons after the change, verbatim;
2. per site, which bar clause it satisfies, and the void-clause check;
3. the floor Summary line (expected unchanged).

## Blast radius

`src/rete/kernel/tests/binding_repr_bench.rs` (two `#[ignore]` strings) · `docs/CONVENTIONS.md`
(one table). **No code. No new measurement. `WARD_VOCABULARIES` unchanged — the names match.**

## STOP triggers

1. A site is another ward's finding → STOP and report; the rune is void there.
2. `:597`'s margin is not derivable from the existing six-sample table → STOP; do not re-measure.
3. Any floor number moves → STOP.

## Prior result to copy for shape

`../strike-census-KL-stale-doc-magnitudes/SCORE.md` — a doc strike that dated what it kept and said
plainly where no mutation was available.
