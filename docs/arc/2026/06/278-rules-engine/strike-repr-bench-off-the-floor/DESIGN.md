# DESIGN — a timing gate is not a gate: rune the three, and mint the vocabulary

Raised by a real floor red on `.floor/2026-09-07T03-20-25Z` (captured in that run's `ARM.txt`),
found while landing A4. **Not caused by A4.**

## The red

```
panicked at src/rete/kernel/tests/binding_repr_bench.rs:762:5:
at the largest cardinality (64) the array EXTENDED faster than the trie
(3995.9ns vs 5860.1ns) — copying 64 pairs beat structural sharing.
If that reproduces, the array DOMINATES and the verdict printed above is wrong
```

## Why it is not a flake, and not dismissed as "timing"

`assert!(hi_ext_trie < hi_ext_arr)` — **a bare `<` between two floats, no margin.** Each is a mean
over **one contiguous wall-clock window** (`REPS 400 × FANOUT 20`), and the trie and array windows
are timed **sequentially** — so the assertion compares *which one ran during a quieter moment.*

The floor runs it at `nice -n 19` against 12–14 concurrent processes, and **this repo has measured
its own contention band at 3.5×–4.4×** (`.config/nextest.toml`, the rete cohort: 8.13→35.39,
7.98→29.42, 13.77→48.72). The trie's `5860` against its own curve's expected ~1100 is **≈5.3× —
inside that band.** The array column grows smoothly and linearly across all eight cardinalities.
**The array did not get fast; one trie sample got slow.**

## The tree already decided this, and three sites predate the rule

`src/rete/kernel/census.rs:234-238`, justifying the entire `GATHER_VISITS` instrument:

> *"**Counting the EXAMINATIONS — rather than the wall-clock — is what makes the keyed-gather gate
> honest.** A timing wall can pass for reasons that have nothing to do with the mechanism … and
> **it is flaky under load** … whatever the machine was doing at the time."*

**Surveyed:** of nine tests under `src/rete/kernel/tests/` using `Instant::now()`, exactly **one**
asserts an ordering between measured durations — this file. Every other cost test times, prints, and
asserts on **counts**.

And this file already excludes two of its own neighbours for the same reason, in ad-hoc prose:

- `:146` — *"measured effect (1.0-1.9x) is too small to gate without flaking"*
- `:264` — *"five-way comparison with no single assertable ordering"*

Three sites, one category, three different spellings, none greppable.

## ⛔ A correction to the split as first proposed

I proposed *"the faithfulness gate stays on the floor, the timing assertions go."* **Grounding says
otherwise.** `bindings_extend_array` **does not ship** — `grep` finds it nowhere outside this file.
The array is a rejected candidate, so the faithfulness gate does not guard production; its own
comment states its purpose:

> *"Faithfulness gate FIRST: the twin must produce the same logical binding set, **or the timings
> below are comparing two different computations**."*

It is the **benchmark's precondition**. What splits is the **assertions**: the deterministic guard
stays, the four timing-ordering gates are deleted.

## THE ONE CONTRACT DECISION

**All three become `#[ignore]`d diagnostics carrying a runed reason; the four timing-ordering
assertions are deleted; the vocabulary is minted and gated.**

`token_bindings_representation_dominance` keeps its faithfulness gate and its table, loses its four
ordering assertions, and gains `#[ignore]` with a rune. Its verdict becomes a **dated six-sample
measurement in the doc comment** — the census K/L pattern, where a 2026-08-01 reading was dated and
kept rather than deleted.

⚠ **Not the `benches/` move.** Stone K's ruling (*"benchmarks live in `benches/` … a benchmark is
not in the test binary at all"*) is the right long-term home and these three CAN reach it — the
twins need only `Value`, `Arc` and `rpds`, so unlike `dispatch_keyword_head_value_perf` no
`pub`-ification is required. **Named as the follow-on, deliberately not taken here**, so the
convention and the relocation are not conflated in one strike.

## ★ The vocabulary — minted here, gated here

The three reasons split into two distinct claims, and the tree has no word for either:

| category | the claim | decisive test the reason must answer |
|---|---|---|
| `below-resolution` | the instrument cannot separate the hypotheses — the margin is inside the noise | **name the noise floor and the margin, and show the margin is smaller** |
| `no-falsifier` | nothing achievable can make the check fail — a green is not evidence | **name what you tried to falsify it with, and why that cannot work** |

Assignments:

- `:146` → **`below-resolution`** (a 1.0–1.9× effect inside a 3.5–4.4× band)
- `:264` → **`no-falsifier`** (*"no single assertable ordering"* — nothing to assert)
- `token_bindings_representation_dominance` → **`below-resolution`** (the red above, with its arithmetic)

⛔ **`rune:excusare(…)` is INVISIBLE to `no_unknown_ward_rune` today** — `categories_on` searches
only for wards in `WARD_VOCABULARIES`, which holds `perspicere` and `purgare`. Shipping the rune
without the registry row would be a marker with no checker, which is the exact pattern this arc has
spent the day removing. **So the strike mints it:** the `excusare` row in that gate, and the
vocabulary table in `docs/CONVENTIONS.md` — *"the table is the definition, this is the gate."*

`perennial` is the ward's own and joins the row. The other two are **proposed upstream** — the
request is at `~/work/NOTE-excusare-lacks-a-term-for-a-gate-that-cannot-be-built.md` — and the table
must say so, with the path, so a reader knows which categories the grimoire blesses and which this
tree is using pending acceptance.

## Out of scope = REJECTED

- **A tolerance** on the assertion to keep it on the floor. A margin picked to stop a red is a
  threshold tuned from our own noise — the move R60 exists to refuse.
- **min-of-k to keep it gating.** It would help and it does not change the category: the claim is
  wall-clock on a shared box, and `census.rs` already ruled such a wall is not a gate.
- The `benches/` relocation (above — the follow-on).
- `binding_cardinality_distribution` (`:407`) — a census-count diagnostic, not timing. Stays.

## Floor arithmetic, predicted

`token_bindings_representation_dominance` becomes ignored, so **5471 → 5470 run, 21 → 22 skipped.**
Any other movement is STOP-3.

## STOP triggers

1. The six samples show the trie **losing** at the largest cardinality in a majority → STOP and
   report. That is a representation finding, not a harness one, and R60's cut is genuinely in question.
2. `no_unknown_ward_rune` reds after the registry row lands → STOP; the row's shape is wrong.
3. Any floor count other than the predicted −1 run / +1 skipped → STOP.
