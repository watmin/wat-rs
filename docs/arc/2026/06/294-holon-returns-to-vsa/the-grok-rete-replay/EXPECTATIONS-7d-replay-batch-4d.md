# EXPECTATIONS 7d — replay batch 4d, grok-rete #221 → #225 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 5 steps, correctly subjected | `verify-step-record.sh <start> HEAD 221 225` | exit 0; `step-range: #221..#225 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on #222 #223 #224 #225 | only `docs/`/`.md` |
| E3 | every produced `.wat` checks | `--check` on #221's new fixture and both stdlib files | rc 0 |
| E4 | **the two-phase stdlib convert actually ran** | #221's body names both `wat/` files and a rebuild between phases | both named; no `UNREGISTERABLE wat/` anywhere |
| E5 | **2a4d's per-SET world held for a 2-file set** | `convert.sh` output for #221, read in full | both members converted; no member's refusal stripped the other's world |
| E6 | #221's named tests | `-E 'test(probe_arc278_oracle_accumulate_supersedes)'` | green |
| E7 | finding 24's walls on the new `.rs` test file | `-E 'test(no_inlined_edn) + test(no_loose_string_assert) + test(no_inlined_wat)'` | green, and **N > 0 selected** |
| E8 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 |
| E9 | **test-count delta ACCOUNTED FOR, not merely green** | predict from the diff (`#[test]` ±, `#[ignore]` ±) BEFORE the floor runs | predicted == actual, exactly |
| E10 | spot re-run of the walls | orchestrator re-runs lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E11 | no hazard outside the expected stdlib pair | `git diff --name-only` vs `wat-scripts/fixes/`, `absent-on-main.tsv` | none (`wat/rete/oracle/*` IS expected here — that is the step) |
| E12 | no knowingly-red commit | no repair commit after #225 | none |
| E13 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E14 | **every repair is visible to `push`** | `git replace -l`; gate re-run under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; gate exit 0 either way |

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate. Read each run's own `N tests run` and require N > 0; a row whose
filter selected nothing is UNMET, not passed. (Filters match the module-qualified path, so naming the
file/module selects every test in it.)

**Runtime prediction:** ~30–50 min. #221 is the only substantial step — a two-phase stdlib convert plus a
rebuild plus two new test files; #222–#225 are four small docs cherry-picks.

**Trap doors named in advance:**
- **#221 is the whole batch's risk.** It is the first two-stdlib-file step since 2a4d, and the stone exists
  precisely because a union over RAW sources let one member's refusal strip the shared world. If the union
  misbehaves the symptom is a *sibling* file converting differently than it does alone — not an error.
  **Read `convert.sh`'s output for both members rather than trusting a green exit.**
- **The baked-stdlib trap:** `wat/` files are `include_str!`ed, so any probe or `--check` run before
  `cargo build --release` measures the OLD world. This has produced a false green here before.
- **A new `.rs` test file** pulls in the test-hygiene walls that a topic-driven gate set will miss
  (finding 24 — that exact omission took a floor red at 2a4d).

**What would make me reject the batch:** an `UNREGISTERABLE wat/` line anywhere (STOP-9 ignored); any
`refs/replace/` entry; or a `--check`/gate green that does not reproduce after `cargo build --release`.
