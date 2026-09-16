# EXPECTATIONS — a row the fence admits must be a row the executor can run

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,202 plus every arm you drive. Exceeding it is a PASS.** Report the final
number; do not tune the work to hit one.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the repro is RED before | `cargo run --release --bin wat -- docs/arc/…/experiri-acc-head.wat` | `unknown rete-defn :wat::rete::core::PersistentVector/length` |
| 2 | the control is GREEN before | same, on `experiri-acc-wrapped.wat` | `"fired"` — the op works in this position when wrapped |
| 3 | the gate is RED before the ladder | the new reachability gate, run against HEAD | **FAIL**, naming the row it could not run as an acc head |
| 4 | the repro is GREEN after | row 1's command | fires; same answer the wrapped control gives |
| 5 | the gate is GREEN after | row 3's command | passes |
| 6 | the gate can FAIL | revert the ladder, re-run the gate | **RED** — then restore |
| 7 | the eligible set is computed, not named | read the gate | it derives rows from `RETE_OPS`; `grep -c 'PersistentVector/length' src/rete/reachability.rs` finds no hard-coded row driving the sweep |
| 8 | the set is non-empty | the gate's own guard | STOP-3: an empty set must fail loudly, not pass quietly |
| 9 | no hollow tests landed | `grep -c 'println!' <the new gate>` | the gate asserts; a `println!`-only test is the banked harness's defect re-committed |
| 10 | `:65-68` no longer lies | read `src/rete/reachability.rs:60-80` | it no longer says the accumulator position is unmodelled |
| 11 | blast radius | `git diff --stat` | `expr_ir/mod.rs` + `reachability.rs`. **`positions-3-4.rs.txt` is NOT appended anywhere** |
| 12 | lints | `cargo nextest run --release -E 'binary_id(wat::lint)'` | all green — the rider runs this one, per last strike's floor red |
| 13 | the floor | `./scripts/floor.sh`, Summary from the captured log | **≥ 5,202 + every new arm**, zero FAIL rows, exit 0 |
| 14 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof

Row 3 → row 5 proves the ladder. Row 6 proves the gate is not vacuous, and it is the one that
matters: **the last strike's vacuity mutation showed that my own control could not see a check that
refused everything.** Ask of this gate not "does it pass" but **what does it fail on** — and answer
it by reverting the ladder, not by reasoning.

Per arm, state: **proven** (driven, red→green), **reachable but not driven**, or **not reachable,
and why**. An unreached arm named as unreached is a pass; an unreached arm not mentioned is a fail.

## Runtime prediction

45–60 minutes. The ladder is perhaps 15 lines; the gate is the work, and `reachability.rs` is a
1,917-line module with its own vocabulary to learn before adding to it.

## What would make this strike a failure even if every test passes

**Appending the banked harness.** Seven `println!` tests on the floor, in an arc that removed 26
tests that asserted nothing, in the same week its README was corrected to say so. Row 9 and row 11
both exist to catch it.

The second failure shape: **hard-coding `PersistentVector/length`.** The gate would then prove one
row works and say nothing about the class — and the class is the whole point, since the finding is
"a site that admits by one registry and dispatches by another", not "this row is broken". Row 7
catches it.

The third: **a gate that passes because its eligible set is empty.** Row 8.
