# EXPECTATIONS — C9's third pairing, at low volume

> ⚠ **This strike closes C9.** The port pairing landed `ed555d02e`; this is the Clara half. A report
> claiming anything beyond the three pairings at **correctness sizes** is out of scope — the perf
> ladder is not this artifact's business.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,376 plus every arm you drive.** An equality caps coverage downward while
looking like rigour. Row 9's pre-values are the floor you clear, never the number you reproduce.

## The scorecard — every pre-value driven at HEAD `ed555d02e`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the third pairing runs at low volume | **never run in 47 recorded grids** (`find . -name 'GRID-*.txt' -not -path './target/*'` → 47, `grep -l oracle-accuracy` → 0) | a harness, every perf axis |
| 2 | ★ `parametric-erasure` has a Clara twin | ⛔ **NONE** — probe prints `NO-CLARA-TWIN` | a **static** `parametric-erasure.clj` |
| 3 | ★ the twin REDs on D7's defect | — | mutation 1, naming `oracle≠native` |
| 4 | the harness reads CLARA and attributes the pair | — | mutation 2: `oracle≠clara` **and** `native≠clara`, NOT `oracle≠native` |
| 5 | every set non-empty | **11/11 non-empty at HEAD** (49·25·20·25·200·75·5·20·50·200·400) | mutation 3 → `VACUOUS`, not `match` |
| 6 | a divergence names WHICH PAIR | — | all three pairings reported separately, both sets + symmetric difference |
| 7 | all axes green at HEAD | **11/11 ALL THREE MATCH** | still 11/11, **+ parametric-erasure = 12/12** |
| 8 | runtime | **43 s**, one JVM per axis | ≤ 120 s — STOP-3 |
| 9 | floor / lints / clippy | **`5376 tests run: 5376 passed, 21 skipped`** (424.3 s, 0 FAIL rows), lints **228**, clippy rc=0, zero warnings | **≥ 5376** + arms, 0 FAIL, lints ≥ 228, rc=0 |
| 10 | no `src/` change | — | zero diff in **index AND worktree** (`git checkout` stages) |
| 11 | the script is invoked | `every_parity_script_is_invoked.rs` gates every `check-*.sh` | wired to CI or a Rust test, and **said which** |
| 12 | no perf-artifact drift | `run-all.sh` discovers `<axis>.wat` + `gen-<axis>.sh` | **no `gen-parametric-erasure.sh`**, no LADDER rung, `run-all.sh` unchanged |

## Runtime prediction

**70–100 minutes.** The harness is a copy of `check-query-compat.sh`'s shape; the `.clj` twin and
its mutation-1 proof are the work.

## Trap doors named in advance

- **Clojure namespace/filename.** Namespace `parametric-erasure` must live in
  `parametric_erasure.clj` or `clojure -M -m` cannot find it. Cost the orchestrator one failed run.
- **The tagged literal.** `:derived #wat.core/PersistentVector [...]` — a regex for `:derived \[`
  matches nothing, and elements are space- not comma-separated.
- **`git checkout <sha> -- <path>` STAGES.** `git diff --stat` reports nothing after a real
  mutation; that false negative invalidated a C16 proof this week. Verify restores by **hash**.
- **A silent skip.** An axis the harness cannot read must FAIL loudly, never be passed over — that
  is how an axis goes dark, and it is the failure this arc keeps re-finding.

## What would make this strike a failure even if every test passes

**Excluding `parametric-erasure` for lack of a Clara twin.** That is the axis carrying the shape
that cost this arc a day, and leaving it unrefereed reproduces C9's own defect — a differential
whose corpus has a hole exactly where the known bug lives, now with a green light over it. Rows 2
and 3 are the strike; row 1 alone is theatre.

**And a harness that reports only PASS/FAIL.** With three engines there are three pairings and they
diagnose three different faults. A red that does not name the pair sends the next reader to the
wrong subsystem.
