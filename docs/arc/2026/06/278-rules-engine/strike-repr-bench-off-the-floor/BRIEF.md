# BRIEF — rune the three timing diagnostics, and mint the vocabulary that names them

Read `DESIGN.md` first. Three sites, one category, three ad-hoc spellings today. **The four
timing-ordering assertions are deleted; nothing else about what these tests measure changes.**

## Read in order

1. `src/rete/kernel/tests/binding_repr_bench.rs:740-775` — the four ordering assertions to delete
   (`:749`, `:763`, `:770`, and their small-end sibling above). Keep the `println!` table and the
   `DOMINANCE:` verdict line — a human reads them on demand.
2. `:616-635` — the faithfulness gate. **Keep it.** It is the benchmark's precondition, and its own
   comment says so.
3. `:146` and `:264` — the two existing `#[ignore]` reasons. These get the runed form.
4. `benches/perf_arc278_fire_baseline.rs:1-20` — the house phrasing for a kept measurement that is
   not a gate. Your doc comments should read like this.
5. `tests/lint/no_unknown_ward_rune.rs:52-66` — `WARD_VOCABULARIES`. The `excusare` row goes here.
   Read `categories_on` (`:117`) too: it searches `rune:{ward}(` only for registered wards, which is
   why an unregistered rune is invisible rather than red.
6. `docs/CONVENTIONS.md:1123-1150` — the `purgare` vocabulary table. **Copy its shape exactly**:
   heading with the closed-set count and date, what the ward flags, the provenance sentence, the
   four-column table, and a paragraph naming the category most likely to drift.

## What ships

**The three runed reasons** (fold into the `#[ignore]` string — the host allows a reason slot):

```rust
#[ignore = "rune:excusare(below-resolution) — <noise floor, margin, and why the margin is smaller>"]
```

- `:146` → `below-resolution`. The reason must carry the arithmetic: a 1.0–1.9× effect against this
  floor's measured 3.5×–4.4× contention band (`.config/nextest.toml`, the rete cohort).
- `:264` → `no-falsifier`. The reason must name what could have been asserted and why nothing can be.
- `token_bindings_representation_dominance` → `below-resolution`, citing the observed red
  (trie 5860.1 vs array 3995.9 at card 64, ≈5.3× on a curve that reads 871.5 at card 32).

**The dated verdict**, in `token_bindings_representation_dominance`'s doc comment: take **six
samples**, record median and range for the largest cardinality, both columns, then the verdict —
*"measured 2026-09-07, 6 samples, median trie X (range …) vs array Y (range …); DOMINANCE: NO, so
R60's cut stands."* That is what replaces the assertion.

**The registry row** in `WARD_VOCABULARIES`: `excusare` with `perennial`, `below-resolution`,
`no-falsifier`.

**The vocabulary table** in `docs/CONVENTIONS.md`. `perennial` is the ward's own (from the
`excusare` spell). The other two are **proposed upstream and in use here pending acceptance** — say
that, and cite `~/work/NOTE-excusare-lacks-a-term-for-a-gate-that-cannot-be-built.md`. A reader must
be able to tell which categories the grimoire blesses.

## Mutation proof

The registry row is a new gate rule: **invent a fourth `excusare` category at one of the three sites
and confirm `no_unknown_ward_rune` REDs**, naming the file and the category. Quote it, restore.
That is what makes the rune gated rather than decorative.

## Floor arithmetic

`token_bindings_representation_dominance` becomes ignored: **5471 → 5470 run, 21 → 22 skipped.**
Report both numbers. Anything else is STOP-3.

## STOP triggers

1. Six samples show the trie losing at the largest cardinality in a majority → STOP and report; that
   is a representation finding, not a harness one.
2. `no_unknown_ward_rune` reds after the row lands → STOP; the row's shape is wrong.
3. Any floor movement beyond −1 run / +1 skipped → STOP.
4. You reach for a tolerance, min-of-k, or the `benches/` move → STOP; all three rejected in DESIGN.

## Prior result to copy for shape

`../strike-census-KL-stale-doc-magnitudes/SCORE.md` — a doc-and-measurement strike that dated what
it kept and said plainly where no mutation was available.
