# TEMPERARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Returned verbatim, before synthesis. Orchestrator verification follows the ward's text.

**Scope actually read:** `tests/rete/` and `src/rete/kernel/tests/` per the manifest; `src/rete/kernel/**` consulted only for context (did not read for its own findings). Counts re-derived, not trusted from the handoff:

```
find tests/rete -type f | wc -l                    → 286
find tests/rete -name '*.wat' | wc -l               → 144
find tests/rete -name '*.rs' | wc -l                → 100
find tests/rete -name '*.edn' | wc -l               → 23
find tests/rete -name '*.wat.bad' | wc -l           → 19
find src/rete/kernel/tests -name '*.rs' | wc -l     → 20
```
Matches the handed-down counts. No `--include='*.rs'` traps needed here since I never grepped `tests/rete` for a bare token like `#[test]` without scoping to `.rs`.

### The sharp lead, resolved

Re-derived the "6 world-freezes at indent 8+ in 4 files" with `grep -nE '^\s{8,}.*(startup_from_source|startup_from_file|startup_beside)'` against exactly those 4 files — got the same 6 sites. Read each in its enclosing function:

| site | inside a loop? | verdict |
|---|---|---|
| `where_tree_branch_differential.rs:508` | `for axis in &uniform` (once per **distinct** axis file) | not waste — each iteration freezes different source |
| `where_tree_branch_differential.rs:519` | one-shot `{ }` block, not a loop | not waste |
| `fanout_cost.rs:64` (`traffic()`) | called 3× but with 3 different `world_src` (fanout/cascade/tri) | not waste |
| `gather_probe_cost.rs:1211` (`run()`) | called 2× with 2 different world sources (`STRAT_WORLD`, `ACCUM_AXIS_WORLD`) | not waste |
| **`accum_cost.rs:320`** | closure `shot(with_query)` called via `for _ in 0..RUNS { shot(false); shot(true); }` (`:384-386`), **same `world_src` re-frozen on every one of the 3 runs per variant** | **WASTE** |
| **`fanout_cost.rs:526`** | identical shape: `shot(with_query)` via `for _ in 0..RUNS { shot(false); shot(true); }` (`:593-596`) | **WASTE** |

So the crude indent heuristic gave 2 real hits out of 6 — settled, not left as a question.

### FINDING (sharpest) — loop-invariant world-freeze inside a "minimum-of-RUNS" benchmark

- **`src/rete/kernel/tests/accum_cost.rs:320`**, function `accum_query_harvest_split` (`:277`)
- **`src/rete/kernel/tests/fanout_cost.rs:526`**, function `fanout_three_leftover_split` (`:472`)
- **Pattern:** loop-invariant computation. Both freeze a world *inside* a closure that gets invoked `RUNS` (=3) times per `with_query` variant — 6 freezes total per test — even though `world_src` is fixed per variant across all 3 runs (`accum_cost.rs:315-318`, `fanout_cost.rs:521-524`). The freeze sits **before** `Instant::now()` (`accum_cost.rs:339`, `fanout_cost.rs:543`), so it does *not* corrupt the measured wall-clock/phase numbers — but it still burns the corpus's own flagged "single most expensive operation" 4 more times per test than needed, on every `cargo nextest run --release` floor pass (both are plain `#[test]`, not `#[ignore]`d — confirmed via `grep -n '#\[ignore\]'` returning nothing for either file).
- **Proof the tempered form is safe here, not just theoretical:** three sibling functions doing the *exact same* "with-query vs without-query, minimum of RUNS" measurement already hoist the freeze outside the closure and RUNS loop:
  - `src/rete/kernel/tests/strat_cost.rs:47` (freeze) → `:50` (closure with no freeze inside) → `:99` (`for _ in 0..RUNS`)
  - `src/rete/kernel/tests/cascade_cost.rs:224` (freeze) → `:227` (closure, no freeze) → `:282` (loop)
  - within `accum_cost.rs` itself, 5 of its other 7 `RUNS`-bearing functions (`:825, 1056, 1319, 1489, 1662`) freeze once *before* their own `for _ in 0..RUNS` loop.
  - Per-run freshness in the two offending tests comes from re-staging (`eval_in_frozen` on a fresh `seed_src`, `accum_cost.rs:330-337` / `fanout_cost.rs:534-541`), **not** from re-freezing — so nothing about firing mutates `world`, and sharing one frozen world across the RUNS loop (as the siblings already do) is correctness-safe.
- **Dimension that grows:** bounded today at `RUNS = 3` (not corpus-size-driven), but it multiplies exactly with that constant — if `RUNS` is ever raised to tighten the noise floor (a realistic edit; this file already leans on "MINIMUM across runs, not mean" for exactly that reason), the waste scales 1:1 with it.
- **Estimated savings:** freezes drop from 6→2 per test (3× reduction in freeze count, i.e. ~67% of this specific benchmark's freeze cost); order-of-magnitude, not micro-benchmarked (no `cargo` run performed, per the read-only mandate).
- **Tempered direction:** hoist — move `let world = startup_from_source(...)` out of the `shot` closure to just above it, once per test (matching `strat_cost.rs`/`cascade_cost.rs` exactly), since `with_query` doesn't change `world_src`'s content across the RUNS loop, only across the two outer `shot(false)`/`shot(true)` calls — so 2 freezes (one per variant) replace 6.
- **Cost-file caveat, addressed directly:** this repetition is entirely **outside** the timed region (`Instant::now()` starts after the freeze in both), so it does not contaminate either test's own measurement — this is an ordinary "hot but not corrupting" temperare finding, not the sharper "corrupts its own number" class the scope correction calls out. I checked for that sharper class specifically (world-freeze *inside* a timed/counted region) at all 6 candidate sites and found none — every freeze at every site precedes its `Instant::now()`/census-counting block.

### Checked and ruled clean (with method)

- **Quadratic loops:** grepped `for … in …\.iter()` followed by `.iter().any(`/`.contains(&`/`.iter().find(` across both trees (79 raw hits); the only co-located pattern (`gather_probe_cost.rs:662-664, 696-698`) scans a small per-element `pairs` slice (bound-variable count, not corpus/data-size scaling) — not quadratic in a growing dimension.
- **Cache-lie pattern:** grepped `-i cache` across both trees. `alpha_discrimination.rs:75,84 / :253,265 / :403,413` (`field_names_cache: HashMap`) is an actual `entry(...).or_insert_with(...)` memo (already carries a `rune:perspicere(read-once)` from a different spell) — genuinely tempered, not a lie. No other `cache`-named field recomputes unconditionally.
- **Trivial "loop-invariant" false positive:** `where_tree_branch_differential.rs:506` calls `grid_dir()` once per axis inside the row loop; read `grid_dir()`'s body (`:324-326`) — it's a bare `Path::join` on `env!(...)`, no filesystem I/O — falls under "single redundant call when the source is cheap," correctly unflagged per the spell's own exclusion.

### Not exhaustively swept

209 `.clone()` / 302 `format!` sites were not individually audited for redundancy beyond the spot checks above — given this corpus's own doctrine that most of `src/rete/kernel/tests/` is instrumentation whose repetition is the point, and that the session's explicit sharpest lead (the 6 indent-8+ freezes) was the highest-value target and is now fully resolved, I judged further blind clone/format sweeping to have low expected yield relative to the two verified sites above. Flagging this gap explicitly rather than implying full coverage.

### Ranking

1. `accum_cost.rs:320` / `fanout_cost.rs:526` — the loop-invariant world-freeze pair above. Sharpest: verified against 3 independent tempered siblings in the same file family, verified the redundancy sits outside the timed region (doesn't corrupt its own number), verified both are unignored `#[test]`s that pay the cost on every floor run.

No other findings met the bar for a report.

FINDINGS

---

# ⚖ ORCHESTRATOR VERIFICATION — my own read of the disk, this session

## ✅ THE FINDING VERIFIES, AND THE STRUCTURAL CONTRAST IS EXACT

Both sites read as claimed, and the contrast against the tempered siblings is as clean as this cast has produced:

| file | order of the three constructs |
|---|---|
| **`accum_cost.rs`** ⛔ | `:314` `let shot = \|with_query: bool\| -> Shot {` → **`:320` `let world = startup_from_source(&world_src, …)` INSIDE** → `:339` `let t0 = Instant::now();` → `:384` `for _ in 0..RUNS {` |
| **`fanout_cost.rs`** ⛔ | same shape — freeze at `:526`, `Instant::now()` at `:543`, `for _ in 0..RUNS` at `:596` |
| **`strat_cost.rs`** ✅ | **`:47` `let world = startup_from_source(WORLD, …)` OUTSIDE** → `:50` `let shot = \|with_query: bool\| -> Shot {` → `:99` `for _ in 0..RUNS {` |
| **`cascade_cost.rs`** ✅ | **`:224` freeze OUTSIDE** → `:227` `let shot = \|with_query: bool\| -> Shot {` → `:282` `for _ in 0..RUNS {` |

Same closure name (`shot`), same return type (`Shot`), same `RUNS` loop — **and the freeze on the other side of the closure boundary.** `const RUNS: usize = 3` confirmed at `accum_cost.rs:282` and `fanout_cost.rs:477`, so the arithmetic holds: 2 variants × 3 runs = **6 freezes**, tempered to **2**.

⭐ **The "does not corrupt its own measurement" claim is the one I most wanted to check, and it holds** — `Instant::now()` is at `:339` and the freeze at `:320`, so the freeze precedes the timed region. The ward volunteered this distinction unprompted, correctly separating "hot" from "hot AND self-contaminating," which was the scope correction's whole point.

## ⚠ ONE NUANCE THE WARD'S OWN EVIDENCE UNDERSTATES — the siblings are NOT an exact precedent

`strat_cost.rs:47` and `cascade_cost.rs:224` freeze a **bare constant** (`WORLD`); `accum_cost.rs:828`, `:1059` and `:1322` likewise freeze `ACCUM_AXIS_WORLD`. The two offenders freeze a **variant-dependent** `world_src` built by `format!` from `with_query` (`accum_cost.rs:315-318`). **So the siblings prove the SHAPE — freeze outside the closure — but none of them is the variant-dependent case.**

This does not weaken the finding: the tempered form for a variant-dependent source is two freezes hoisted above the loop rather than one, **which is exactly the arithmetic the ward gave** (*"2 freezes (one per variant) replace 6"*). Recorded so a later hand does not read the siblings as a drop-in template and collapse two worlds into one.

⚠ Minor: the ward cited *"`:825, 1056, 1319, 1489, 1662`"* for those hoisted siblings. Those addresses are the `const RUNS: usize = 3;` lines; the freezes sit ~3 lines below (`:828`, `:1059`, `:1322`). Read as function identifiers the citation is fair; read as freeze sites it is off by three. The substance — that they hoist — verifies.
