# CIRCUMSPICERE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Cast LAST, after sixteen inward wards. Returned verbatim, before synthesis. Orchestrator verification follows.

**Sweep performed:** read `Cargo.toml` (test-target table + workspace/lints), `build.rs` (full, 112 lines), `.config/nextest.toml` (full, 362 lines), `.github/workflows/ci.yml` (full), `tests/rete/mod.rs`, `src/rete/kernel/mod.rs`, and the bodies of `src/rete/kernel/tests/{accum_cost,accum_alpha_cost,gather_probe_cost,rank_and_instrument,strat_cost,cascade_cost,harvest_cost,fanout_cost,node_share_cost}.rs`. Not a re-read of the 306-file corpus's internal correctness — only its build/run/perimeter.

## FINDING (sharpest, and the one I'm reporting)

**Surround:** default-behaviour egress / unenforced invariant — a measured cost class given a bespoke timeout override under one test name, silently absent for its siblings.

**Location (two coordinates — code, and the doc it contradicts):**
- Doc: `.config/nextest.toml:283-330` (the "rete cohort" block). It measures `accum_fire_phase_census` at **8.13s alone / 35.39s under floor (4.35x contention)** and on that basis gives it `slow-timeout = 90s/180s, priority = 98` via `[[profile.default.overrides]] filter = 'binary_id(wat::rete) or test(accum_fire_phase_census) or test(deftest_wat_tests_rete_fuzz) or test(deftest_wat_tests_rete_tms)'`. Comment at `:208-209` explains the `or test(accum_fire_phase_census)` clause exists *because* `accum_fire_phase_census` lives in `binary_id(wat)` (the lib unit-test binary via `src/rete/kernel/tests/`, `#[cfg(test)] mod tests;` at `src/rete/kernel/mod.rs:44`), not `wat::rete`, so the broader `binary_id(wat::rete)` substring cannot reach it.
- Code: the identical (200,200)-sized workload (`accum_phase_census(200, 200)` / `accum_count_census(200, 200)`) — the exact call the override was written to cover — is called by **at least eight other `#[test]` functions in that same lib binary, none named in the override**:
  - `src/rete/kernel/tests/accum_cost.rs:610` `accum_leftover_split` — `for _ in 0..RUNS{accum_phase_census(200,200)}`, `RUNS=3` (line 607)
  - `src/rete/kernel/tests/accum_cost.rs:1663` `accum_seen_fire_context_split` — same, `G=200,W=200`, `RUNS=3` (line 1662)
  - `src/rete/kernel/tests/accum_alpha_cost.rs:79` `accum_alpha_leftover_split` — `RUNS=3` (line 78)
  - `src/rete/kernel/tests/accum_alpha_cost.rs:387` `accum_alpha_seed_after_fold_split` — `RUNS=3` (line 386)
  - `src/rete/kernel/tests/rank_and_instrument.rs:1153` `cell_rank_after_fanout` — `RUNS=3` (line 1150)
  - `src/rete/kernel/tests/rank_and_instrument.rs:1237` `cell_rank_after_grid`
  - `src/rete/kernel/tests/rank_and_instrument.rs:1323` `honest_cell_rank_after_arm`
  - `src/rete/kernel/tests/gather_probe_cost.rs:31` `gather_index_is_built_once_per_alpha_and_keyset` — single `accum_count_census(200,200)` call

  I confirmed by direct grep of `.config/nextest.toml` that **none of these eight names, nor any filter that would match them, appears anywhere in the file**.

**What the inward guard saw here:** nothing — this is a `.config/nextest.toml` fact; all inward wards were confined to the 306 corpus files and none read the nextest config. `secare`'s 4D1 (the `ARM_BUILDS` race) is the only inward finding that touches this same binary, and it's a different mechanism entirely.

**Why this is a real risk, not a hypothetical:** the file's own numbers make the arithmetic explicit. `accum_fire_phase_census`'s body (`accum_cost.rs:169-247`) runs a 4-point ladder `[(25,50),(50,100),(100,200),(200,200)]` *plus one extra* `accum_phase_census(200,200)` call — i.e. two (200,200)-class calls dominate its 8.13s. Four of the eight siblings above run **three** such calls in a bare `RUNS=3` loop with no cheaper ladder points at all — a heavier shape than the test that was measured and given headroom. Projected at this cohort's own recorded contention band (3.5x-4.4x, `.config/nextest.toml:326`, explicitly stated to apply to "a future test added here"), these land at 30-53s+, over the **default profile's 15s warn / 30s kill** that applies to everything not named in an override — the same class of gap your predecessor found on 2026-08-26 for `deftest_wat_tests_rete_fuzz`.

**Severity:** High for this class (an unenforced load-bearing invariant / negative-space finding, not a claim-contradiction, so not the top rank) — a red here would present as "flaky rete test," directly reproducing the exact failure-labeling `.config/nextest.toml:24-31` and `retries=0`'s own history were struck to prevent, in a target whose CLAUDE.md doctrine says "there is no such thing as a known flake."

**Closure:** extend the existing override filter's test-name alternation to cover this exact cost class — one line, same shape as the existing entry, not a redesign. (I did not make this edit — read-only ward.)

## Answers to the five questions

1. **What runs this corpus, and does anything run all of it?** Both halves reach CI: `tests/rete/` compiles as the single `[[test]] name="rete" path="tests/rete/mod.rs"` binary (`Cargo.toml:198-200`), and `src/rete/kernel/tests/` compiles into the lib's own `#[cfg(test)]` unit-test binary (`src/rete/kernel/mod.rs:44`). CI's only gate is `cargo nextest run --profile ci --release` (`.github/workflows/ci.yml:96`) with no `-p`/`-E` scoping, which by Cargo/nextest default covers every target in `default-members` (`Cargo.toml:17-26`). I found no path by which either half is skipped. **Settled: both run.**

2. **Recurrence of the 2026-08-26 unreachable-budget finding?** **Yes — this is the finding above.** It's the same shape one level worse: the earlier case was a filter that *should* have matched but didn't (substring miss); this case is a cost class that was recognized and given a bespoke override for **one test**, while known siblings calling the identical expensive primitive in the identical file were left uncovered.

3. **Does anything assert build.rs's module generation is complete?** Partially. `tests/kernel/probe_build_rs_autodiscovery.rs` is a deliberate "no-panic-is-the-proof" test: a file dropped into `tests/kernel/` with no committed mod-list entry, whose mere presence in the run report proves build.rs's walk-and-generate mechanism works. It proves the **mechanism** generically (same `fs::read_dir` code path for every group, `build.rs:44-49`) but does **not** exist inside `tests/rete/` itself — there is no `rete`-local instance of this proof. Given the mechanism is untyped per group, I judge this a sound-by-shared-code negative rather than a fresh finding, but flag that no one has verified the `rete` group specifically exercises this path.

4. **Any shipped claim about this corpus the code doesn't back?** I did not find a crisp, citable count/coverage claim about the rete corpus specifically in README or top-level docs (light grep only, per the builder's standing ruling that `docs/*.md` is stale-by-default and out of scope for repair). No finding here; not fully settled, low priority given the ruling.

5. **What does this corpus do to the machine?** 14 files under `tests/rete/` and part of `arm_lease.rs` spawn subprocesses/threads. The two I sampled (`probe_arc278_fixpoint_round_cap.rs:48`, `probe_arc278_import_accounting.rs:49`) spawn the locally-built `wat` binary via `Command::new(bin)` where `bin` traces to `build.rs`'s `WAT_RUNTIME_BIN_DEFAULT` (`build.rs:100-111`) — self-testing the CLI, not external egress. `arm_lease.rs:59` spawns a thread; same file underlying `secare`'s 4D1, not a new surface. **No default-behaviour egress (network, filesystem-outside-tree) found in the sample checked.**

## What I verified sound (stated method + result)

- **Cargo/build.rs compilation model, independently re-derived** (not reusing your handed-down conclusion): `Cargo.toml:198-200` is one `[[test]]` target; `tests/rete/mod.rs` is a 3-line `include!` stub; `build.rs:32-93` walks `tests/<group>/*.rs` and emits `#[path]` modules into `OUT_DIR`. Matches what you found — I re-read the primary sources myself rather than trusting the prior report.
- **No blanket override exists for the lib binary**: `grep -n "binary_id(wat)" .config/nextest.toml` returns only a *comment* (line 208) explaining the gap, never a rule. Anchored against the known-positive `binary_id(wat::rete)` and `binary_id(wat::lint)` rules, which do exist, so the absence is real, not a bad grep.
- **CI scoping**: read the whole `ci.yml`; the `test` job's single `cargo nextest run --profile ci --release` has no path/package restriction.

I corrected myself once mid-cast: I initially assumed `accum_fire_phase_census`'s cost was one `(200,200)` call and almost under-stated the sibling risk; re-reading its body (`accum_cost.rs:169-247`) showed a 4-point ladder plus one more call, which I used to calibrate the per-call cost estimate.

FINDINGS

---

# ⚖ ORCHESTRATOR VERIFICATION — my own read of the disk, this session

## ✅ THE FINDING IS REAL, AND IT IS BIGGER THAN THE WARD MEASURED — NINE SIBLINGS, NOT EIGHT

I did not take the ward's list. I derived the population from the primitive instead of from its names — `grep -rn 'accum_phase_census(200, *200)\|accum_count_census(200, *200)\|accum_phase_census(G, *W)\|accum_count_census(G, *W)'` over `src/rete/kernel/tests/` — then walked back from each hit to its enclosing `fn`:

| call site | enclosing `#[test]` | in nextest.toml? |
|---|---|---|
| `accum_cost.rs:228` | `accum_fire_phase_census` (`:169`) | ✅ **7 mentions** |
| `accum_cost.rs:34` | **`accum_matcher_op_census` (`:33`)** ⭐ **the ward missed this one** | ⛔ 0 |
| `accum_cost.rs:658` | `accum_leftover_split` (`:606`) | ⛔ 0 |
| `accum_cost.rs:1717` | `accum_seen_fire_context_split` (`:1659`) | ⛔ 0 |
| `accum_alpha_cost.rs:89` | `accum_alpha_leftover_split` (`:75`) | ⛔ 0 |
| `accum_alpha_cost.rs:393` | `accum_alpha_seed_after_fold_split` (`:383`) | ⛔ 0 |
| `rank_and_instrument.rs:1196` | `cell_rank_after_fanout` (`:1149`) | ⛔ 0 |
| `rank_and_instrument.rs:1284` | `cell_rank_after_grid` (`:1233`) | ⛔ 0 |
| `rank_and_instrument.rs:1385` | `honest_cell_rank_after_arm` (`:1319`) | ⛔ 0 |
| `gather_probe_cost.rs:32` | `gather_index_is_built_once_per_alpha_and_keyset` (`:31`) | ⛔ 0 |

**One test in this cost class appears seven times in the config. Nine siblings appear zero times.** Every one of the nine is a plain `#[test]` — I checked the preceding line at all nine sites — and `grep -rn '#\[ignore\]'` over all four files returns **0**, so none is excluded by any other route either.

⭐ **The ward's NAMES were right for all eight it found; its ADDRESSES were consistently ~4 lines high** (it cited `:610`, `:1663`, `:79`, `:387`, `:1153`, `:1237`, `:1323` — those are `RUNS`/`const` lines near the test, not the calls, and the true enclosing `fn` lines are `:606`, `:1659`, `:75`, `:383`, `:1149`, `:1233`, `:1319`). Only `gather_probe_cost.rs:31` is exact. **Trap #5 for the third time this vigilia — and for the third time the substance survived the addresses.** Because I derived the population from the primitive rather than checking its list, I also found the tenth site it missed.

## ⭐⭐⭐ WHY THIS CLOSES THE VIGILIA AT FOUR FOR FOUR

The `.config/nextest.toml` block that grants the override **records its own reason for existing**: `accum_fire_phase_census` sits in `binary_id(wat)`, not `binary_id(wat::rete)`, so *"a `binary_id(wat::rete)` filter alone silently misses it."* Someone found that gap, understood it, and closed it — **for the one test they were looking at.** The nine siblings sharing its binary and its workload were never enumerated.

⛔ **And the file itself says what a red here would look like.** Its `retries = 0` note, struck at the builder's direction 2026-08-05: *"the second run passing DESTROYS the only evidence the first run produced, and the report says 'flaky' where the truth is 'failed, once, with an arm nobody kept.'"* A sibling timing out at the default 30s kill presents as exactly that — **a flaky rete test, in a repo whose doctrine says there is no such thing.**

⭐ This is the **fourth consecutive target** on which `circumspicere`, cast last, returned the sharpest finding — and the second time in this arc that its quarry was a budget the runner cannot grant. **Cast it last, every time.**

## ✅ ITS FOUR NEGATIVES, AND ONE HONEST NON-ANSWER

Q1 **settled** (both halves reach CI through an unscoped `cargo nextest run --profile ci --release`). Q3 **partially settled** with the gap named — `probe_build_rs_autodiscovery.rs` proves the mechanism generically but there is **no `rete`-local instance**. Q5 **settled** — the subprocess spawns are the locally-built `wat` binary via `build.rs`'s own path, self-testing the CLI, **no external egress**. Q4 it declined to chase and **said so plainly**, citing the builder's docs ruling rather than manufacturing a claim-vs-code finding. ⭐ A ward that returns *"I did not settle this and here is why"* is worth more than one that fills the gap.

⭐ And it **re-derived my own handed-down compilation-model finding from the primary sources rather than accepting it** — the exact behaviour that would have caught `complectens`'s inverted proof sixteen wards earlier.
