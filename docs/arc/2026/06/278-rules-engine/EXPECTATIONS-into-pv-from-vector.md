# EXPECTATIONS — `into (PersistentVector, Vector)`

Written BEFORE the strike so the result cannot move the goalposts. Every command is the
orchestrator's own re-run; nothing is graded on the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the RED gate flips | `(:wat::core::into (:wat::core::PersistentVector 1 2) (:wat::core::Vector :wat::core::i64 3 4))` | `#wat.core/PersistentVector [1 2 3 4]` (was `NoMatchingClauseAtCallSite`) |
| 2 | the receiver's KIND is preserved | same form | a **PersistentVector**, not a Vector — the whole point |
| 3 | the PV×PV clause too | `into` a PV from a PV | `#wat.core/PersistentVector [...]` |
| 4 | a permanent test exists | `cargo nextest run --release -E 'test(into)'` | the new `deftest'` present and green |
| 5 | semantics unchanged | `cargo nextest run --release` | **4243/4243**, Summary line read directly |
| 6 | the wall | `cargo clippy --release --workspace --all-targets -- -D warnings` | exit 0, **0** warnings |
| 7 | the nine axes still load | `target/release/wat --check` on each grid `.wat` | clean, all nine |
| 8 | **`:derived` is byte-identical** | run each axis at its smallest size, diff `:derived` against pre-stone output | **identical** — this stone is observationally inert |
| 9 | the win, MEASURED | wall clock of `echo '[40000]' \| wat grid/fanout.wat`, interleaved pre/post, loadavg gated < 1.5 | recorded with ranges |
| 10 | **`:native-ns` did NOT move** | the same runs' `:native-ns` | unchanged within noise — the fire is untouched |

Row 8 is the load-bearing correctness row and row 10 is the load-bearing honesty row. A stone that
speeds up the harness by changing what the harness computes is not a win; it is a corrupted
benchmark. Row 10 exists because if `:native-ns` moves, something touched the fire and the whole
grid history becomes incomparable.

## Independent prediction

**Runtime: 25–45 minutes.** One scheme registration, one dispatch arm, one wat clause, nine
one-line bodies, one test — with `Vector/concat` sitting there as the exact shape to mirror and the
native fn already written and in production use. Time-box at **2× the upper bound = 90 minutes**.

**Perf: RECORDED, NOT GRADED.** No threshold on row 9. The arithmetic says `vec->pvec` is one of
two interpreted 40,000-iteration passes in a ~5.1 s derive, so removing one pass could plausibly be
1–2 s — but the last two perf predictions this session were both wrong (2–3 ms predicted vs 2.4 ms
measured was fine; "the lead decays with scale" was refuted outright at the first test). Predict the
MECHANISM, measure the milliseconds: what is certain is that N interpreted closure invocations
become one native call.

## Trap doors named in advance

- **`vector_concat_inner` may coerce to Vector.** Then row 2 fails and STOP-1 fires. This is the
  single most likely blocker and it is why row 2 is separate from row 1.
- **Clause ambiguity.** A new clause can make a previously-unique match ambiguous (STOP-2).
  Reordering clauses to "fix" it hides the ambiguity rather than resolving it.
- **The second `Vector/concat` site (`runtime.rs:9672`).** Skipping it may leave the op working at
  runtime but not in the const/compile-time path — a split-brain the tests may not reach. The brief
  requires an explicit decision, not silence.
- **A changed `:derived`.** Row 8. If the interpreted fold and the native concat disagree on
  ordering, the workaround was never equivalent — that is a real finding about `concat`, and it
  must surface, not be smoothed by re-sorting.
- **Scope creep into the `map`+`enc` pass.** Explicitly rejected in the DESIGN. It is the bigger
  cost and the bigger stone; touching it here would confound row 9 beyond reading.

## What "done" means

Rows 1–8 and 10 pass on the orchestrator's own re-run, with exit codes read, before anything is
committed. Row 9 is recorded with ranges and the load average beside every sample, per
`feedback_a_benchmarks_shape_manufactures_its_result` — interleaved, never in blocks.
