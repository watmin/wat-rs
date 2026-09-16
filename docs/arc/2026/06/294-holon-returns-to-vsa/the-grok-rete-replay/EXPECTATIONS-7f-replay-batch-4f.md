# EXPECTATIONS 7f — replay batch 4f, grok-rete #241 → #260 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <start> HEAD 241 260` | exit 0; `step-range: #241..#260 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on all **14** (#243 #244 #245 #247 #248 #249 #251 #252 #253 #255 #256 #257 #259 #260) | only `docs/`/`.md` |
| E3 | **#241's verdict lines match its DIFF, not its kind** | its body | `census` + `nested-program-gate` present; `lint-subset`/`kind(lib)`/`doctest` ABSENT (no `.rs`) |
| E4 | every produced `.wat` checks | `--check` each (#241's 1, #246's 4, #250's 1, #258's 5) | rc 0, or a deliberate `.bad` with its reason |
| E5 | ⚠ **our gather_probe_cost.rs divergence SURVIVES #254** | `grep -c 'h >= (b + m + e)' src/rete/kernel/tests/gather_probe_cost.rs` | **0** — the struck assert must NOT be restored |
| E6 | named tests at the shared/code steps | `-E` per step (#242 #246 #250 #254 #258) | green, **N > 0 selected** |
| E7 | finding 33's class was actively looked for | the REPLAY-LOG / SCORE | each rename-ish step records whether the `.rs`/`.sh` side was grepped — a "not applicable" is an answer, silence is not |
| E8 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 |
| E9 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor runs | predicted == actual, exactly |
| E10 | spot re-run of the walls | orchestrator re-runs lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E11 | no hazard in range | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | **none at all** — this range has no stdlib or moved-home row |
| E12 | no knowingly-red commit | no repair commit after #260 | none |
| E13 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E14 | **every repair visible to `push`** | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; gate exit 0 either way |
| E15 | **no verdict line is WRAPPED** | grep each pattern across the range | every required line matches on ONE line |

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate. Read each run's own `N tests run` and require N > 0.

**Runtime prediction:** ~70–100 min. Twelve docs cherry-picks are quick; the weight is #258 (17 files, 5
`.wat`, 8 `.rs`), #254 (11 `.rs`), and #246 (4 `.wat`).

**Trap doors named in advance:**
- **#258** — the largest step in the batch and the only one combining many `.wat` with many `.rs`.
- **#254 meets our deliberate divergence** in `gather_probe_cost.rs`. E5 is the scored row: our strike must
  survive. Restoring grok's assertion would reintroduce a gate that reddened ~13% of floors.
- **#241 carries a `.wat` and no `.rs`** — the path-based rule, whose inverse cost a record repair at #215.
- **finding 33's class**: any step mentioning a rename must have the `.rs`/`.sh` side grepped. Three
  instances so far were found only by reading the diff.

**What would make me reject the batch:** the struck assertion restored in `gather_probe_cost.rs`; any
`refs/replace/` entry; a `.wat` left failing `--check` without a stated, checkable reason; or #241 carrying
`.rs` verdict lines it cannot have earned.
