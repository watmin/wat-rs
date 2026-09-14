# SCORE 4b — checkpoint #60: two walls meet; fold the repairs, close 2a4's two flaws, census every step

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `cec02a996` BRIEF 4b. Finding 18. 2a4b: `d19e65885`.
Folded #50: `6a5b92609` (was `3ee40f787`). Folded #60: `6bbe1b777` (was `2528967fb`).
Tag `replay-pre-fold` = old tip `7d50e6edd`.

```
floor  scripts/floor.sh   .floor/2026-09-14T22-03-17Z
       Summary [ 220.761s] 5495 tests run: 5495 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

Old #60 floor `.floor/2026-09-14T01-43-05Z` was RED (5485 passed, 5 failed, STOP-4, not re-run).
This floor is the checkpoint at the new #60 **plus** 2a4b: the five red tests pass, plus five new
2a4b tests (3 `stdlib-source-path?` wat-tests, `tracked_wat_dir_is_exactly_stdlib_sources`,
`declared_stdlib_types_retracts_old_variant_singletons`). 5485 + 5 recovered + 5 new = 5495.

## A — fold. Proof at `cec02a996` (BRIEF-4b, after cherry-pick of this brief)

`git diff replay-pre-fold cec02a996 --stat` names exactly 8 files:

```
 src/config.rs                                      |   1 +
 .../fixes/drop-unconsumed-negation-bind.wat        | 263 +++++++++++++++++++++
 .../replay/drop-unconsumed-negation-bind/ORACLE    |   5 +
 .../drop-unconsumed-negation-bind/after.post       |   4 +
 .../drop-unconsumed-negation-bind/before.pre       |   4 +
 .../replay/drop-unconsumed-negation-bind/stdin     |   1 +
 wat-scripts/fmt/rules/defrecord.wat                |   2 +-
 wat-scripts/fmt/rules/kwargs.wat                   |  46 ++--
 8 files changed, 302 insertions(+), 24 deletions(-)
```

Nothing else. 50 `REPLAY(grok-rete #` subjects #11–#60 unchanged. No cherry-pick conflicts (STOP-1
did not fire). 2a4b `d19e65885` sits on top of that fold; it is not part of A's proof.

## B — recorded migration `drop-unconsumed-negation-bind`

Stdin is `path:line:col:var` from the wall (`bootstrap/era/probe-T/sites-60.txt`), never re-derived.
Edit: drop the 3-child bind at `ast-span` plus one adjacent space (trailing preferred). Guard: exact
`(?<var> <- :<field>)` of the reported var; missing site is a no-op (idempotent); wrong-shape-of-var
refuses. Multi-site: later line/col first, one splice + re-read.

Fixture `wat-scripts/fixes/replay/drop-unconsumed-negation-bind/` `{FILE}:3:37:?more`.
`every_recorded_migration_is_fixtured_or_runed` GREEN (ORACLE spec quote is a contiguous header
substring: `remove the child list whose ast-span starts at line:col and one adjacent space`).
Shard 11 (the stem) GREEN. Postcondition: `kwargs.wat` `--check` rc 0, `defrecord.wat` `--check` rc 0;
TIME ONE FILE FIRST on kwargs, then defrecord. `pprintln_doc_row` byte golden unchanged (floor).

24 sites applied into folded #50.

## C — 2a4b `d19e65885`

1. **`TypeEnv::retract_for_door_replace` retracts variant singletons** via `compose_variant`.
   `declared_stdlib_types_retracts_old_variant_singletons` GREEN: `E.V` answers `[b, c]`, `E.W` is
   gone. RED under the mutation that drops the singleton loop (captured previous session): panic
   `src/check.rs:23757` left `[("V", ["a"])]` right `[("V", ["b", "c"])]`. Restored, GREEN here.

2. **`stdlib-source-path?` is only `starts-with "wat/"`.** Three wat-tests GREEN (`wat/core.wat` true;
   `crates/wat-edn/wat-edn-clj/wat/shared.wat` false; `examples/console-demo/wat/main.wat` false).
   RED under the mutation that restores `contains "/wat/"` (captured previous session):
   `deftest_wat_tests_fix_stdlib_source_path_edn_clj_shared_is_not_stdlib` actual `true` expected
   `false`. Restored, GREEN here.

   Gate `tracked_wat_dir_is_exactly_stdlib_sources` GREEN (scan of `path: "` filtered to `wat/`-
   prefixed; comment-string probes `s3a-probe.wat` are not rows).

   `convert.sh` cds to `$ABS_OUT` so TARGETS are repo-relative (`wat/gen.wat`). Codemod path stays
   absolute. one-param-spec CONTEXT may stay absolute (tmp ctx-tree).

   **Deviation from the brief's "refuse an absolute path loudly":** dropped. `{FILE}` fixtures and
   convert.sh context pass absolute paths; a loud refuse killed `positional-ctor-to-map` /
   match-arm / variant-separator replay. Absolute is simply not stdlib (user door).
   `every_recorded_migration_replays_positional_ctor` GREEN.

## D — census from #61

`scripts/replay/census.sh`: `git ls-files '*.wat'` → `--check` `-P32`, `path rc` into ignored
`.census/`. BRIEF-1 § One-step 3 gained the two rows; STOP-8 is on the STOP list.

Baseline at HEAD (new #60 + 2a4b): `.census/2026-09-14T22-02-27Z.txt` — 2049 files, 200 failing
(old tip 2047 / 219). First capture was vacuous (all rc 0: `$?` taken after `sha256sum`); fixed
in the same 2a4b commit before floor/run5 (`rc=$?` before the hash; dropped `-n1` vs `-I`).

Control: `--diff` new census as previous, `bootstrap/era/probe-T/census-rc-2528967fb.txt` as
current → **STOP-8 for exactly 19 files**, exit 8:

```
STOP-8 tests/cli/pprintln_doc_row.wat rc 0 -> 1
STOP-8 tests/cli/metadata_of_example_formats.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/rules/defrecord.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/rules/kwargs.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-all.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-r11.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-r3.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-let.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-r4.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-tables.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-examples.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run-types.wat rc 0 -> 1
STOP-8 wat-scripts/fmt/run.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-comments-both-sides.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-file-width-census.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-file-ends-with.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-registry-row-pretty.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-fmt-dump.wat rc 0 -> 1
STOP-8 wat-scripts/scratch-pad/277-width-offenders.wat rc 0 -> 1
```

The two rule files plus their 17 loaders. 219 − 19 = 200 remaining failures.

## The bar

| item | result |
|---|---|
| A's proof | **PASS.** 8 files at `cec02a996`; 50 subjects unchanged. |
| Checkpoint floor at new #60 + clippy | **PASS.** `.floor/2026-09-14T22-03-17Z` 5495/5495, clippy 0. (Floor after C; composition repairs are in #50/#60.) |
| B fixture + postcondition + pprintln golden | **PASS.** Gate GREEN; kwargs/defrecord `--check` 0; golden unchanged. |
| C fixtures RED under mutation, GREEN with it | **PASS.** Both mutations captured previous session; GREEN this session. |
| `run5.sh` unchanged (0 losing; chain vs main ≥ 1370) | **PASS.** See table. |
| D census at new #60; STOP-8 control = 19 wall files | **PASS.** |

## run5 (`bootstrap/era/probe-S/run5.sh` on `d19e65885`, clean tree)

| tool | wall (one process) | files LOSING | files GAINING | UNRESOLVED |
|---|---|---|---|---|
| match-arm vs `probe-L/wL` | 149 s | **0** | 1 | 10 (ladder 15) |
| positional-ctor vs `probe-L/pT` | 30 s | 1 (same 2a2 artifact) | 7 | 173 (today 7663) |
| variant-separator vs the census tool | 56 s | **0** | 67 | report lines 38 |

Chain vs main: identical **1370**, differs 119; classifier CHAIN-FAILS **28** (same as 2a2-REFUTE / 2a4).
START 22:07:46Z DONE 22:17:27Z. VS report lines 38 (2a4: 35) — not a losing file; 0 losing holds.

## Folded hashes (#50–#60)

| N | C → old replay | C → folded |
|---|---|---|
| 50 | `4354afc6e` → `3ee40f787` | `4354afc6e` → `6a5b92609` |
| 51 | `aaa62272e` → `6d2f83b75` | `aaa62272e` → `027a78011` |
| 52 | `014f99452` → `25cd7eaf6` | `014f99452` → `6fee919cf` |
| 53 | `d758a7ba9` → `2faa14838` | `d758a7ba9` → `9cfdfc58d` |
| 54 | `2361bf8b3` → `f6d73fefd` | `2361bf8b3` → `66fa44357` |
| 55 | `fc949d002` → `ec7cf39a1` | `fc949d002` → `77dd78794` |
| 56 | `fcfe1b291` → `50240389c` | `fcfe1b291` → `118d6d48d` |
| 57 | `6241ae1d8` → `7ecc337b4` | `6241ae1d8` → `8105e22d6` |
| 58 | `17d010638` → `3f57263a8` | `17d010638` → `419935bcf` |
| 59 | `841914cc8` → `1fd0f5b21` | `841914cc8` → `957278f6e` |
| 60 | `d039b29e5` → `2528967fb` | `d039b29e5` → `6bbe1b777` |

Not pushed. Batch 2 is a separate brief.
