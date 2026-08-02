# BRIEF — Step 0 for `Token.bindings` → `PMap`: the width census + the boundary A/B

Design: `DESIGN-STONE-token-bindings-promoting.md`. **This brief MEASURES. It changes no
representation.** `Token.bindings` stays `rpds::HashTrieMapSync` throughout; the stone does not
proceed past this measurement, and the measurement decides its shape.

## What is being decided

A promoting map gives a Token with ≤8 bindings the array arm (which wins build, lookup, clone and
drop) and promotes past 8 (where the trie wins extend by 3.4×). The risk is **repeated boundary
crossing**: a Token that grows past 8 pays an array→trie rebuild, and if that happens routinely the
rebuild is paid per token per fire and the stone is a loss.

Two questions, and only these two:

1. **How WIDE are Token bindings, across the nine axes?**
2. **How OFTEN does a Token cross the threshold?**

## Part 1 — the width census

The census infrastructure exists and is the pattern to copy: `census_count(&'static str)` in
`src/rete/kernel.rs`, collected by `with_count_census`, read by tests like
`node_share_filter_eval_census` (same file's `#[cfg(test)] mod tests`). Read one of those first —
it is the shape.

Instrument, in `kernel.rs`:

- **Width at creation and after each extend.** Wherever a `Token` is built or extended
  (`extend_token` is the extender; `grep` for the other construction sites), bucket
  `bindings.size()`: `token:w0`, `w1` … `w8`, `w9plus`.
- **★ THE CROSSING COUNTER, which is the one that decides it.** In `extend_token`, when the
  binding count goes from `<= 8` to `> 8`, `census_count("token:crossed-8")`. This is the array→trie
  rebuild a promoting map would pay. Also count `token:extends` (every extend) so the crossing count
  has a denominator — a raw crossing count with no total says nothing.

Then a test that fires **all nine grid axes** at their standard sizes and prints one table: per
axis, the width histogram, `extends`, `crossed-8`, and `crossed-8 / extends` as a percentage.

The nine axes and their worlds are already set up in that test module — `NODE_SHARE_WORLD` and its
siblings; follow how `node_share_filter_eval_census` builds and fires one, and do the same for each.

**Non-vacuity, asserted not assumed:** every axis must report `extends > 0`. An axis that recorded
zero extends contributes a row of zeroes that reads like good news.

## Part 2 — the boundary A/B

Out of fire, same harness style as `node_share_where_cost_decomposition` in the same test module —
read it; it is the worked reference for interleaving and medians.

Build a token's bindings up to **7, 8, 9, and 12** entries by successive insertion, two ways:

- **arm A** — `rpds::HashTrieMapSync` with `insert_mut` (today)
- **arm B** — `crate::value::pmap::PMap` with `assoc` (promoting; already built and proven)

For each width, time: **build** (successive insertion from empty), **lookup** (every key once),
**clone**, and **drop**. Interleave the arms within each rep — never run one arm's block then the
other's. 15 reps, medians. Report a table: width × operation × arm, with the ratio.

Assert the arms agree on content before timing them — a fast arm computing something else is not a
measurement.

## What to report

The two tables, verbatim, plus a plain reading of:

- the width distribution — where the mass actually is
- **`crossed-8 / extends`** per axis, which is the risk
- whether arm B loses materially below 8, and whether arm A still wins above it

**Do not recommend a threshold and do not recommend whether to build the stone.** Report the
numbers. The ruling is the builder's.

## STOP triggers — rejection criteria. Ship nothing; report the gap.

**STOP-0.** If `crossed-8 / extends` is high on the live axes, that is the finding — **report it
plainly**. Do not try a different threshold to make the number look better, and do not soften the
reading. A measurement that gets tuned until it flatters the plan is the failure this whole stone's
discipline exists to prevent.

**STOP-1.** If instrumenting requires changing `Token.bindings`' type, or any behaviour — STOP. This
brief adds counters and a test. If a counter cannot be placed without a semantic change, say where.

**STOP-2.** If the nine axes cannot all be fired from the test module (a world that will not build,
an axis whose driver differs) — STOP and report which; do not silently census eight and present it
as nine.

## Blast radius

`src/rete/kernel.rs` — counters in the token paths plus two tests in its `#[cfg(test)] mod tests`.
Nothing else. **No `.wat` file. No representation change. No behaviour change.**

## The gate

1. `cargo nextest run --release` — the **Summary line**, 4260 passing, 0 failed. Never a piped exit
   code (`grep -c` returns 1 on a zero count and looks like a failure).
2. `cargo clippy --release --all-targets` — 0.
3. Both tables printed, with the non-vacuity assertions live.

## You are a rider, not the orchestrator

**Ending your turn ENDS you** — it does not suspend you, and nothing will wake you. There is no
notification coming. Run every verification in the FOREGROUND and block on it; your turn ends when
the numbers are in your hands, not when a command is launched. Do not commit, do not push, do not
stash. Report what you ran and what it said.
