# EXPECTATIONS — `produced_type` on a user-fn head, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| oracle, re-confirmed on your binary | `cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat` | ANCHOR `["pt::Rate"]`; MEASURE `["pt::first-rate"]` |
| arm 1 — native ANCHOR | the new test | `pt::Rate` — if this is wrong the extractor is broken, not divergent, and nothing below counts |
| arm 1 — native MEASURE | the new test | **PREDICTION: `pt::Rate`**, against the oracle's `pt::first-rate`. A refutation is a valid result (STOP-1) |
| arm 1 — oracle side driven, not printed | read the test | `eval_in_frozen` of `:wat::rete::rule-produces`, no hardcoded literal |
| arm 2 — facts differ? | the fixture, both engines | **UNKNOWN, and that is the point.** Outcome 1 (differ) = live oracle defect. Outcome 2 (unconstructible) = evidenced negative with verbatim refusals |
| floor | `scripts/floor.sh` | 5473 + new tests, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

Arm 1: 25–40 min. Arm 2: unbounded by nature — time-box the construction attempts and report what
was tried rather than grinding.

## Trap doors, named in advance

- **⛔ The negative reported as a shrug.** Outcome 2 is a legitimate finding ONLY if it carries the
  compiler's own refusal text per attempt. Without that it is indistinguishable from not having
  tried, and it would license a future hand to close this row on nothing.
- **A hardcoded oracle side.** The first SCORE in this strike family printed the oracle's numbers
  as format-string literals and was refuted. Drive both halves in one process.
- **A stale binary.** `wat/` is `include_str!`'d. `~/.cargo/bin/wat` is NOT current — use
  `cargo run --release --bin wat`. One measurement has already been retracted in this arc for this.
- **Fixing the direction before it is known.** Native resolving to the return type may be the
  RIGHT behaviour and the oracle may be the wrong one — that is what `16f504e14` found last time.
  Nothing here decides that, and arm 2 is what earns the right to.
- **A green arm 1 read as "the engines agree".** Arm 1 measures NAMES. Facts are arm 2. Do not let
  a green arm 1 close the row.
