# BRIEF — make a gather examination countable only one way

**Floor GREEN when you are done.** The keyed-gather ratio may move; that is data, not a STOP.

## Read in order

1. **`DESIGN.md`** — including its correction of the source finding: 3 real sites, not 5.
2. **`src/rete/kernel/census.rs`** — the `census_gather_visit` declaration and its argument for why
   examinations, not wall-clock, make the gate honest.
3. **The three uncounted sites**: `fire/acc.rs:407`, `fire/pass/accumulate.rs:252`,
   `fire/mod.rs:~1980` (the no-`SeedCmp` arm that maps over the whole bucket).
4. **The seven counted sites** — `grep -rn census_gather_visit src/rete/` — for the shape that
   already works.
5. **`tests/rank_and_instrument.rs::keyed_gather_visits_do_not_scale_with_group_count`** — the gate,
   its non-vacuity guard, and the `ratio <= 2.0` assertion.
6. **`tests/lint/no_raw_network_keys_in_oracle.rs`** — the lint shape F1 produced: one verb, an
   exemption list, a mutation proof. Copy it.

## The work

**1. Count the three.** Whatever helper you introduce, a `bucket.iter()` in a gather arm must count
per element yielded.

**2. Read the ratio, both ways.** Report `small`, `big` and the ratio BEFORE (today) and AFTER. If it
crosses 2.0, **STOP and report** — see STOP-1.

**3. The lint.** Refuse a raw bucket walk in the gather modules outside the counting helper.
**Mutation-prove it**: re-introduce a raw walk, confirm RED, restore, quote both. Give it a
non-vacuity guard — a file list that can go silently empty is the defect this arc keeps finding.

**4. Exemptions are runed, with reasons that name what the walk does** — not "it looks fine".
`bucket.len()` and `is_empty()` examine nothing; if either ever becomes a walk, it needs counting,
not a rune.

## Blast radius

`src/rete/kernel/fire/` + `src/rete/kernel/census.rs` + one lint. **No engine behaviour change** —
census is `#[cfg(test)]`; same facts, same rows.

## STOP triggers

1. **If the ratio crosses 2.0 after counting, STOP and report both readings.** Do not raise the
   threshold. That crossing is the regression the gate was built to catch, surfacing for the first
   time — it needs its own strike, not a constant.
2. **If counting changes any FACT-level result, STOP.** Census is test-only; a behaviour change means
   the helper touched the engine.
3. **If a `*_cost` gate moves, STOP and report.** `census_gather_visit` is `#[cfg(test)]` but the
   helper is not; an iterator wrapper on a hot path can cost.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-explain-order/` — one verb, a lint with a visible exemption list, mutation-proved, and a
SCORE that says plainly which gate is the proof and which is regression cover.
