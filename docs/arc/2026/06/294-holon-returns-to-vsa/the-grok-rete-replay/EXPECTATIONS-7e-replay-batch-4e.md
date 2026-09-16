# EXPECTATIONS 7e — replay batch 4e, grok-rete #226 → #240 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 15 steps, correctly subjected | `verify-step-record.sh <start> HEAD 226 240` | exit 0; `step-range: #226..#240 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on #227 #228 #229 #231 #232 #235 #236 #237 #239 #240 | only `docs/`/`.md` |
| E3 | **#233's verdict lines match its DIFF, not its kind** | its body | `census` + `nested-program-gate` present; `lint-subset`/`kind(lib)`/`doctest` ABSENT (it has no `.rs`) |
| E4 | #226's two-phase convert actually ran | #226's body names `wat/rete/syntax.wat` and a rebuild between phases | both present; no `UNREGISTERABLE wat/` anywhere |
| E5 | the divergent MACRO was handled | `convert.sh` output for #226, read in full | the macro converted or explicitly reported; G1 class behaviour visible, not assumed |
| E6 | every non-`wat/` `.wat` checks | `--check` on each produced fixture | rc 0 |
| E7 | **the `wat/` `--check` row is DISPROVED, not faked** | pre-step vs post-step `ReservedPrefix` counts from identical path shapes | delta equals the defns the step adds; both rc=1 |
| E8 | named tests at the shared/code steps | `-E` per step (#226 #230 #234 #238) | green, **N > 0 selected** |
| E9 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 |
| E10 | **test-count delta ACCOUNTED FOR** | predict from the diff (`#[test]` ±, `#[ignore]` ±) BEFORE the floor runs | predicted == actual, exactly |
| E11 | spot re-run of the walls | orchestrator re-runs lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E12 | no hazard outside #226's stdlib file | `git diff --name-only` vs `wat-scripts/fixes/`, `absent-on-main.tsv` | none (`wat/rete/syntax.wat` IS expected — that is #226) |
| E13 | no knowingly-red commit | no repair commit after #240 | none |
| E14 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E15 | **every repair visible to `push`** | `git replace -l`; gate re-run under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; gate exit 0 either way |
| E16 | **no verdict line is WRAPPED** | `git log --format=%b` over the range, grep each pattern | every `census:`/`nested-program-gate:`/`lint-subset:`/`kind(lib):`/`doctest:` matches on ONE line |

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate. Read each run's own `N tests run` and require N > 0; a row whose
filter selected nothing is UNMET, not passed.

**Runtime prediction:** ~60–90 min. #226 is the substantial step (two-phase stdlib + a divergent macro +
five shared `.rs`, two of them main's most-diverged files); #230/#234/#238 are small `.rs` steps; ten docs
cherry-picks.

**Trap doors named in advance:**
- **#226** — two-phase, a divergent stdlib macro (G1, closed by 2a4c), and `src/check.rs` + `src/runtime.rs`,
  which have diverged most between the two sides. Expect re-expression, not clean application.
- **#233 carries a `.wat` and no `.rs`** — the record gate derives requirements from the diff, so it needs
  census + nested-gate and NOT the `.rs` walls. The inverse misjudgement cost a record repair at #215.
- **The baked-stdlib trap:** `wat/` is `include_str!`ed, so any probe before `cargo build --release`
  measures the OLD world. This has produced a false green here before.
- **`--check` on `wat/`** is unsatisfiable by construction — disprove it with a pre/post `ReservedPrefix`
  delta, never skip it silently and never invent a pass.

**What would make me reject the batch:** an `UNREGISTERABLE wat/` line anywhere; any `refs/replace/` entry;
a `--check` or gate green that does not reproduce after `cargo build --release`; or #233 carrying `.rs`
verdict lines it cannot have earned.
