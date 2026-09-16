# EXPECTATIONS 7i — replay batch 4i, grok-rete #301 → #320 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which could not be

A row a CORRECT tree cannot satisfy is as broken as one a defect can satisfy (finding 34 §2). The cure is
to RUN a row before demanding it. Stated honestly rather than claimed wholesale:

**Run against `c8a206928` and ALL PASS** (current HEAD differs only by the docs-only 4h records commit):
E16 (origin an ancestor; `refs/original/` empty), E17 (0 replace refs), E9 (0 `.wat` across grok's
#301–#320), E8 (0 `wat/` or `wat-scripts/fixes/` paths in range), E2 (13 docs-only), E2b (7 code steps),
plus the three trap pre-conditions: **#315 is genuinely empty in grok (0 files)**, **#302 touches
`wat-scripts/perf/grid/run-axis.sh`**, and **#310 adds
`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs`**. Our `*_cost.rs` corpus measured at **8
files** against grok's single repaired file.

**CANNOT be run until the batch lands — predictions, not verified rows:** E3, E4, E5, E6 (the #310 gate
does not exist yet, so its greens, its runes and its repair site cannot be checked), E7 (those tests are
not present), and E11–E13 (they depend on a floor the executor is forbidden to run).

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <start> HEAD 301 320` | exit 0; `step-range: #301..#320 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on all **13** (#301 #303 #305 #307 #308 #309 #311 #312 #314 #315 #316 #318 #319) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 7 code steps (#302 #304 #306 #310 #313 #317 #320) | each carries ≥1 non-docs file |
| E3 | **#310's new gate is GREEN** | `-E 'test(census_name_read_by_a_cost_test_is_emitted)'` | all tests run, PASS — neither `unresolved` nor `hollow` fires |
| E4 | **every rune is a DECLARATION** | each `rune:lint(census-name-retired)` line added in range | names an exact token AND a reason ≥40 chars naming a mechanism; **zero blanket reasons** |
| E5 | **#310's repair landed AT #310** | `git show --name-only` on the #310 REPLAY commit | it carries the repaired `*_cost.rs` files itself, not a later step |
| E6 | ⚠ **our `gather_probe_cost.rs` divergence SURVIVES** | the struck apportionment assert (finding 32) is still absent as LIVE CODE | absent — the one prose match in the strike's own comment does NOT count |
| E7 | named tests at the 7 code steps | `-E` per step | green, **N > 0 selected** |
| E8 | **ZERO hazard paths in range** | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | **none at all** |
| E9 | **no `.wat` anywhere in range** | `git diff --name-only <start>..HEAD \| grep -c '\.wat$'` | **0** |
| E10 | ⚠ **finding 33 grepped at #302** | the SCORE / REPLAY-LOG | #302 records that the `.sh` side of `run-axis.sh` was read — this is the file the class is named for |
| E11 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 — **the orchestrator's row** |
| E12 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor runs | predicted == actual, exactly |
| E13 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E14 | no knowingly-red commit | no repair commit after #320 | none |
| E15 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E16 | **NO PUBLISHED HISTORY REWRITTEN** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD`; `git for-each-ref refs/original/` | ancestor **YES**; `refs/original/` **EMPTY** |
| E17 | every repair visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; exit 0 either way |
| E18 | **no verdict line is WRAPPED** | grep each pattern across the range | every required line on ONE line; `--diff` and `no STOP-8` contiguous |
| E19 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every landing-time action that changed a gate, its inputs, or its ledger appears in the row that gate satisfies |
| E20 | every `-E` filter selected N > 0 | each run's own `N tests run` line | N > 0 everywhere |
| E21 | ⛔ **NO COUNTERPART ACTIVITY** | `ls .floor/` for runs the orchestrator did not start; `git status --porcelain` for artifacts the executor did not write; `.pulsare/` mtimes unchanged | **no `pulsare_yield`, no foreign floor, no foreign artifact, frozen root untouched** |
| E22 | **#315 recorded despite being empty** | `git show --stat` on the #315 REPLAY commit | 0 files, and the body says grok's own commit carries no tree change |

## ⛔ E21 is new, and it exists because batch 4h's executor woke a counterpart

4h's executor called `pulsare_yield`. It wrote into the FROZEN root and woke a counterpart that ran its
own `scripts/floor.sh` **inside the working tree**, contending with the orchestrator's gates — a wall run
took 61.2s against a 28.3s baseline. The values survived because three independent measurements agreed,
but a contended gate is a false result, and for a timing-sensitive suite it would have mattered.
**An affordance that is not named is not forbidden**: BRIEF-7i forbids `pulsare_yield` by name.

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is indistinguishable from
a working gate.

**Runtime prediction:** ~80–120 min. Thirteen docs cherry-picks are quick; the weight is #310 (the fourth
new gate plus a repair across up to 8 `*_cost.rs` files) and #320 (4 `.rs`).

**Trap doors named in advance:**
- **#310 — the fourth new grok lint gate; it WILL land red.** Grok repairs one file; we have eight.
  Repair at the step. A rune is a declaration, never a blanket suppression.
- **#302 — the finding-33 hot spot**, the file whose embedded `perl` substitution sat broken for weeks.
- **#315 — genuinely empty in grok.** Still needs a REPLAY commit (`--allow-empty`).
- **All 7 code steps are in `*_cost.rs`** — the timing-sensitive class behind findings 28 and 32. Report
  any new ratio/threshold assertion rather than accepting it.

**What would make me reject the batch:** the #310 gate weakened, allowlisted, or its red deferred; a rune
with a blanket reason; a correct census name renamed to dodge the gate; any `refs/original/` entry or the
pushed tip no longer an ancestor; any `refs/replace/` entry; a fabricated trailer SHA; any `pulsare_yield`
or other signal out of the sandbox; or a SCORE green whose landing-time cost is disclosed only in the log.
