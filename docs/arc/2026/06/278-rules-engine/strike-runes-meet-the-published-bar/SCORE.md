# SCORE — the four runes meet the published bar

Comments and one doc table. Nothing re-measured. `WARD_VOCABULARIES` unchanged. Floor unchanged.
No mutation — these are reasons.

## Scorecard

| # | result |
|---|---|
| 1 ★ `:597` both numbers | **HOLD.** Margin **3.0×** (medians 2028.8/676.3 ns, six samples 2026-09-07) and floor **3.5×–4.4×**, margin-inside-floor stated. Captured red kept as consequence, not as the margin. |
| 2 ★ `:264` names an attempt | **HOLD.** Tried array-wins-extend-everywhere (sibling already prints DOMINANCE: NO); tried a single-cell ordering (one cell of a 5×2×4 grid). |
| 3 ★ void-clause check | **HOLD.** One line per site below. None is `vocare` / `complectens` / `experiri`'s `inert`. |
| 4 ★ nothing re-measured | **HOLD.** 3.0× from the six-sample table in the dominance doc comment. 3.5×–4.4× from `.config/nextest.toml` (8.13→35.39, 7.98→29.42, 13.77→48.72). 1.0–1.9× from the three runs 2026-08-30 already in `:146`. |
| 5 the two that pass | **HOLD.** `:146` and `session.rs:1834` — clauses below. Untouched. |
| 6 CONVENTIONS accepted | **HOLD.** *proposed upstream* gone. `fetched 2026-09-07`. Decisive tests in the spell's published wording, including *measured from the effect, never read off the operator*. |
| 7 floor | **HOLD.** `.floor/2026-09-07T06-08-44Z/`: `Summary [ 463.861s] 5470 tests run: 5470 passed (2 slow), 22 skipped`. Unchanged. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the failing rune.** Row 4 is what stops this strike inventing evidence.

## The four reasons, verbatim

**`session.rs:1834` `no-falsifier` — PASSES (untouched).** Clause: names the attempt (`#[should_panic]` on the release floor) and why it cannot work (`debug_assert` compiles out; the test is `cfg(debug_assertions)`).

```
// rune:excusare(no-falsifier) — a #[should_panic] on the release floor cannot fail
// this guard: debug_assert compiles out under --release, and this test is
// cfg(debug_assertions) so it does not exist in the floor binary. Nothing
// achievable on cargo nextest run --release makes the check fail. An assert!
// to buy a floor-provable mutation was rejected: that is release cost for an
// unreachable branch.
```

Void-clause: unbuildable gate. Not `vocare` (the test does not restate its implementation as a passing check of itself). Not `complectens`. Not `experiri`'s `inert`.

**`binding_repr_bench.rs:146` `below-resolution` — PASSES (untouched).** Clause: both numbers measured (margin 1.0–1.9× over three runs 2026-08-30; floor 3.5×–4.4×), margin-inside-floor stated.

```
rune:excusare(below-resolution) — lookup 1.0–1.1× / build 1.1–1.9× (three runs 2026-08-30); this floor's rete-cohort contention band is 3.5×–4.4× (.config/nextest.toml: 8.13s→35.39s, 7.98s→29.42s, 13.77s→48.72s). A 1.9× ceiling sits inside that band, so any floor tight enough to catch a regression is a threshold inside the noise.
```

Void-clause: unbuildable timing gate. Not `vocare`. Not `complectens`. Not `experiri`'s `inert`.

**`binding_repr_bench.rs:264` `no-falsifier` — was BORDERLINE; now names the attempt.**

```
rune:excusare(no-falsifier) — tried asserting array-wins-extend at every cardinality (the dominance question): the sibling token_bindings_representation_dominance already prints DOMINANCE: NO on that exact question, so the assert would be a known-false gate. Tried a single-cell ordering: one cell of a 5×2×4 grid, and a green is not evidence about the table. The test sat on the floor asserting nothing, counted as a passing test that cannot fail. Nothing achievable reds the rest of the grid without inventing a crossover N, which R60 refuses.
```

Void-clause: unbuildable gate. The table's cells answer differently across operations and cardinalities, so not `experiri`'s `inert`. Not `vocare` (no assertion restates the implementation). Not `complectens`.

**`binding_repr_bench.rs:597` `below-resolution` — was FAILS AT BIRTH (cited the 5.3× excursion); now states the margin.**

```
rune:excusare(below-resolution) — margin: the array/trie EXTEND ratio at card 64 — medians 2028.8 ns / 676.3 ns = 3.0×, six isolated samples 2026-09-07. Noise floor: this floor's rete-cohort contention band, measured 3.5×–4.4× (.config/nextest.toml: 8.13s→35.39s, 7.98s→29.42s, 13.77s→48.72s). The margin sits inside the floor. Captured red .floor/2026-09-07T03-20-25Z (trie 5860.1 ns vs array 3995.9 ns at card 64) is the consequence, not the margin.
```

Void-clause: unbuildable timing gate. Not `vocare`. Not `complectens`. Not `experiri`'s `inert`.

## Mutation

None. A reason change cannot red an instrument. Saying so is the report.

## What this did not do

Did not re-run the bench. Did not touch `perennial`'s five `src/comms/` sites. Did not relocate to `benches/`. Did not change `WARD_VOCABULARIES`.

## Final floor

`.floor/2026-09-07T06-08-44Z/`: `Summary [ 463.861s] 5470 tests run: 5470 passed (2 slow), 22 skipped`. Count unchanged. No new tests.
