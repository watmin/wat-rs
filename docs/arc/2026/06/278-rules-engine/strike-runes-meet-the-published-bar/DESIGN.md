# DESIGN — the vocabulary was accepted, and it raised the bar past two of our four runes

`excusare` re-fetched from the signed channel **2026-09-07**, after the builder filed
`~/work/NOTE-excusare-lacks-a-term-for-a-gate-that-cannot-be-built.md`.

## Both categories were accepted, with the proposed names

`rune:excusare` is now a closed set of **three**: `perennial`, `no-falsifier`, `below-resolution`.

The spell also blesses the placement this arc chose, in its own words:

> *"The rune goes wherever the host already gives you a reason slot … `#[ignore = "rune:excusare(below-resolution) — …"]` is the same declaration as the comment above the assertion; the `rune:` prefix is what makes it legible on sight as a weighed claim rather than one more piece of prose."*

and adds `#[ignore]` to the override surface outright — *"a test held back from the run is a silenced check, and its string is the excuse; a bare `#[ignore]` pleads nothing."*

## ★ And it set a bar two of our four runes do not clear

The published `below-resolution` requires **both numbers, both measured**:

> *"The reason MUST name **the noise floor and the margin, and show the margin is smaller** — both
> **measured**, and the margin measured from the effect, never read off the operator. **"The
> comparison is a bare `<`, so the required margin is zero" is not a margin; it is a syntax fact**
> … a reason missing **either** number, or deriving one instead of measuring it, **FAILS at birth**."*

And `no-falsifier` requires a **named attempt**:

> *"The reason MUST name **what you tried to falsify it with and why that cannot work** … **"No
> mutation available" alone FAILS**: it is indistinguishable from not having looked."*

Audited against that, at HEAD:

| site | category | verdict |
|---|---|---|
| `session.rs:1834` | `no-falsifier` | **PASSES** — names the attempt (`#[should_panic]` on the release floor) and why it cannot work (`debug_assert` compiles out; the test is `cfg(debug_assertions)`) |
| `binding_repr_bench.rs:146` | `below-resolution` | **PASSES** — margin 1.0–1.9× measured over three runs, floor 3.5–4.4× measured, margin-inside-floor stated |
| `binding_repr_bench.rs:264` | `no-falsifier` | **BORDERLINE** — argues structurally that no single ordering and no conjunction can work, but names no attempt |
| `binding_repr_bench.rs:597` | `below-resolution` | **FAILS AT BIRTH** — states the floor, cites the **excursion** (5.3×), and never states the **margin** |

`:597` is the sharp one, and it fails for a reason worth naming: it reports how far the *outlier*
moved, which is a property of the noise, when the bar asks for the size of the *effect*. **The
margin exists and is measured** — trie median 676.3 ns vs array median 2028.8 ns over six isolated
samples, a **3.0× effect inside a 3.5–4.4× floor** — it simply is not in the reason.

## THE ONE CONTRACT DECISION

**Bring the two reasons up to the published bar using numbers already measured; measure nothing new.**

- `:597` — add the margin: *"margin: the array/trie EXTEND ratio at card 64, measured 3.0× (medians 2028.8 ns / 676.3 ns) over six isolated samples 2026-09-07. Noise floor: this floor's rete-cohort contention band, measured 3.5×–4.4×. The margin sits inside the floor."* Keep the captured red as evidence of the consequence, not as the margin.
- `:264` — name the attempt: what was driven, and the outcome that shows nothing achievable reds it.

⛔ **No new measurement.** Every number this needs is already in the tree (`.config/nextest.toml`'s
band, the six-sample table in the dominance doc comment). A strike that re-measures to satisfy a
rune is the tail wagging the dog.

## Also ships

**`docs/CONVENTIONS.md`** — the `rune:excusare` table flips from *proposed upstream* to **accepted**,
with `fetched 2026-09-07` in the provenance line (the shape `purgare`'s table uses), and the
decisive-test column restated in the spell's published wording — including *"measured, never read
off the operator"*, which is the clause `:597` fell to.

`WARD_VOCABULARIES` needs **no change**: the accepted names are the proposed names.

## ★ And one new clause to check ourselves against

The spell now carves out what the rune may **not** cover:

> *"`no-falsifier` and `below-resolution` declare that **no gate is constructible** — never that a
> constructible one is inconvenient. A test that restates its implementation is `vocare`'s finding;
> a layer that leans on another layer's proof is `complectens`'s; a declared surface cell that
> answers the same to every input is `experiri`'s `inert` … **Where any of them owns the finding,
> the rune is void.**"*

Each of the four must be checked against that: is this genuinely an unbuildable gate, or another
ward's finding wearing an exemption? Report the check per site.

## Out of scope = REJECTED

- **Re-measuring** anything. The numbers exist.
- Touching `perennial`'s five `src/comms/` sites — a different category, and not this arc's.
- The `benches/` relocation (still Stone K's follow-on).

## STOP triggers

1. A site turns out to be another ward's finding under the new clause → STOP and report; the rune is
   void there and the cure is that ward's, not a better reason.
2. `:597`'s margin cannot be stated from existing measurements → STOP; do not re-measure to fill it.
3. Any floor number moves → STOP. Comments and one doc table only.
