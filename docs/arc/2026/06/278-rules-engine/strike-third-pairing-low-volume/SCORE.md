# SCORE — the third pairing, and a header of mine that denied it

> **Written after the orchestrator's own re-run.** Every row below was driven on this machine at
> HEAD `a9d433885` + the strike. The rider's figures are noted where they differ; none was taken
> on trust.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 | ★ the third pairing runs at low volume | ✅ `check-grid-three-way.sh` — **12 axes, 12.7 s**, re-driven by the orchestrator |
| 2 | ★ `parametric-erasure` has a Clara twin | ✅ **static** `parametric-erasure.clj`; no `gen-` script, no LADDER rung, `run-all.sh` byte-identical |
| 3 | ★ the twin REDs on D7's defect | ✅ **re-driven by the orchestrator** — see below |
| 4 | the harness reads CLARA and attributes the pair | ✅ rider's mutation 2, with **identical cardinality on both sides** (25 vs 25) — a count-based check passes it |
| 5 | every set non-empty | ✅ guard 2 checks **all three** columns; mutation 3 → `VACUOUS`, not `match` |
| 6 | a divergence names WHICH PAIR | ✅ three pairings reported separately, both sets + symmetric difference |
| 7 | all axes green at HEAD | ✅ **12/12 ALL THREE MATCH** (was 11/11 + one absence) |
| 8 | runtime ≤ 120 s | ✅ **12.7 s** — one JVM for all axes, not one per axis |
| 9 | floor / lints / clippy | ✅ **`5376 tests run: 5376 passed (1 slow), 21 skipped`** (425.7 s), **0 FAIL rows**, lints **228**, clippy rc=0. `every_parity_script_is_invoked_by_ci_or_a_test` **PASS** — the nextest-side proof the new script is covered |
| 10 | no `src/` change | ✅ empty diff in **index AND worktree**; `alpha.rs` md5 back to HEAD after mutation 1 |
| 11 | the script is invoked | ✅ CI `parity` job. Rider mutation-proved it: deleting the `run:` line REDs `every_parity_script_is_invoked_by_ci_or_a_test` |
| 12 | no perf-artifact drift | ✅ no `gen-parametric-erasure.sh`, no rung, `run-all.sh` unchanged |

## ⛔ THE FINDING, AND IT IS THE ORCHESTRATOR'S OWN LINE

`parametric-erasure.wat:46`, present at HEAD `a9d433885`, read:

> *"Clara has no parametric records either, so there is no `.clj` twin to author."*

**Written the previous day, in the strike this orchestrator supervised and approved** — and sitting
in the file BRIEF read-item #5 sends the rider to for *"its rules and its canonical encoding."* It is
the exact reasoning the builder had already struck:

> *"clojure doesn't have holon's vsa/hdc tooling either — we need to push our boundaries where they
> make sense to do so"*

**This is the second consecutive strike whose ★ was a false, dated, authoritative claim in the file
the brief told the rider to trust.** Yesterday: a test asserting a comparison its own setup made
impossible. Today: a header denying a twin that took one file to author. Both were found by a rider
auditing its instructions rather than following them.

Struck at the site, with the argument: **Clara referees RULE SEMANTICS, not wat's type system.** The
erasure is what wat does to the *declaration*; what reaches the network is a bag of ordinary facts of
one class whose `v` fields hold different runtime types — which dynamically-typed Clojure expresses
as its native case. The twin reproduces the **derived set**, and `parametric-erasure.clj:48-50`
cycles `v` by `k mod 3` (Long / String / Tag) to rebuild exactly the mixed-packability workload.

## Mutation 1, re-driven by the orchestrator — and it shows what the third pairing BUYS

`git checkout 523152b31 -- src/rete/kernel/fire/pass/alpha.rs` (+4/−155, md5 confirmed changed
*before* the run), release rebuild, then the harness:

```
[parametric-erasure] ⛔ native != clara  =>  THE FAST PATH IS WRONG (size [200])
[parametric-erasure] ⛔ oracle != native  =>  A PORT BUG (size [200])
grid-three-way: FAILURES above (11 of 12 axes agreed, 12s)
```

**Two diagnoses printed, and a third withheld.** There is no `oracle != clara` line: Clara and the
spec agree with each other, so the reference engine **independently ratifies 600 as the truth** and
the fault is localised to native.

★ **That is the entire argument for the third pairing, in one output.** Yesterday's port gate, on
this identical defect, could only report *"two of our engines disagree"* — it had no third opinion to
break the tie. Restored; `alpha.rs` md5 back to HEAD; harness green again at 12/12.

## The rider's other two mutations

- **2 — corrupt a generated Clara program** (`gen-negation.sh`, `(map :?k …)` → `(map (fn [r] (inc (:?k r))) …)`):
  `oracle != clara` **and** `native != clara`, and **no** `oracle != native` — exactly the predicted
  pair set. Cardinality identical on both sides (25 vs 25), so a count-based check passes it.
- **3 — empty one axis's sets**: `VACUOUS — clara=0 native=0 oracle=0`, exit 1.

## Honest deltas — including three corrections to this orchestrator's own artifacts

- **The probe's vacuity guard checked two of three columns.** `[ -z "$c" ] || [ -z "$n" ]` — clara
  and native, never the oracle. So EXPECTATIONS row 5's *"11/11 non-empty"* pre-value was measured
  on two thirds of the data. The counts printed were correct; the guard could not have caught an
  empty oracle. The shipped harness checks all three.
- **Rows 9 and 11 of EXPECTATIONS contradict each other.** Row 9 demands *"≥ 5376 **plus every arm
  you drive**"*; row 11 accepts CI wiring. A CI-wired shell gate adds **zero** nextest tests, so the
  floor stays exactly 5376. The rider flagged this rather than gaming it. **The clause was sloppy:
  "every arm you drive" presumes arms land as nextest tests.**
- **⚠ AND THE CONSEQUENCE BELONGS IN THE RECORD: the three-way lives in CI, not on the floor.**
  `./scripts/floor.sh` does **not** run it. The rider's reasoning is sound — a Rust test would need
  a JDK on every dev machine, and a java-optional test is a check that reports success without
  running, a failure mode this repo has already paid for. `every_parity_script_is_invoked` proves
  *something* runs the script; **CI is that something.**
- **Row 8's "43 s" was a number to BEAT, not a baseline to hold** — and the BRIEF's read-item #2
  said so (one JVM, all rows) while row 8 did not. The two artifacts pulled opposite ways. The
  finished gate does 12 axes in 12.7 s against the probe's 11 in 43 s.
- **The underscore trap was stated as a filename rule; it is a classpath constraint.** All 38 static
  `.clj` here are dash-named and run in script mode. Following the brief literally would have landed
  `parametric_erasure.clj` alone among 38 dash-named siblings. The rider kept the dash name and
  stages it under an underscore in the harness's temp dir.
- The rider's own first-draft `run-all.sh` citations were off and it corrected them to `:81-87` /
  `:89-99` by grepping. The DESIGN's and BRIEF's cites (`run-all.sh:85`, `check-where-shapes.sh:140`)
  verified exact.

## What this strike closes

**C9 is closed.** All three pairings now run: `clara vs native` on the perf ladder (47 recorded
grids), `oracle vs native` on the floor at low volume (`ed555d02e`, 12.2 s), and `clara vs oracle`
in CI at low volume (this strike, 12.7 s). The builder's split — *"clara vs wat native is the typical
measurement; wat oracle vs wat native needs low volume so we don't waste hours"* — is now the shape
of the instrument.
