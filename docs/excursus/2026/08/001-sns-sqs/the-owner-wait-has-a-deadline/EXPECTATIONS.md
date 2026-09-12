# EXPECTATIONS — the owner wait has a deadline

**Written before the strike.** Graded against the orchestrator's OWN reads and runs.

## Scorecard

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑ STOP-1 answered with EVIDENCE | read the SCORE | the coerced `select` on a lineage peer either ran or refused, with the **verbatim** runtime result quoted — not an opinion |
| 2 | shape chosen follows from row 1 | read the SCORE | (A) wat-level if select works; (B) Rust primitive only if it provably refuses |
| 3 | ⭑ a silent peer no longer hangs | the new test | `<S>/stop` on a never-replying service returns `GaveUp` with `last="TimedOut"` |
| 4 | …and that test HANGS on the parent commit | run it against `140df30d8` | hangs / times out — proving the test can fail |
| 5 | `TimedOut` is reachable | `grep` + the row-3 run | produced by the loop, not only by `call-by-deadline` |
| 6 | the bound is still in ONE place | `grep -n 'budget-ms' wat/service.wat` | all occurrences inside `owner-recv-loop` |
| 7 | budget stays a parameter | read | `budget-ms` still an argument, not a baked constant |
| 8 | the four method bodies unchanged | `git diff` | only `owner-recv-loop` (+ Rust if shape B) |
| 9 | floor | `./scripts/floor.sh` → Summary | `5237 passed`, 22 skipped, 0 FAIL (+ any new deftest) |
| 10 | tests compile | `nextest --release --no-run` | exit 0 |
| 11 | clippy | `clippy --release --workspace --all-targets -- -D warnings` | exit 0 |
| 12 | happy path | `… 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 13 | chaos | `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | exit 0, `distinct=100;dup=0` |
| 14 | `StopOutcome`/`GateOutcome` untouched | `git diff` | shapes unchanged; only reachability |

## Runtime prediction

**45–90 min if shape (A)** — one function body. **2–4 h if shape (B)** — a Rust constructor plus the
recv path, and `-D warnings` to satisfy.

## What makes this PARTIAL

- Row 1 answered by assertion rather than a quoted runtime result → the stone rests on the same
  unverified claim it exists to correct.
- Row 4 missing → a fix for a hang with no test that hangs on the old commit is unproven. This is the
  `peragrare` point: a green from an instrument never shown to fail is silence.
- Row 3 green but row 6 red → the deadline works and the bound got duplicated; the class re-opens.

## Trap-doors

1. ⛔ **The NOTE's blocker is refuted but not replaced.** The probe killed "select can't mix tiers" as
   the *first* failure and did not reach the runtime check. STOP-1.
2. **`Peer` is `<S,R>`, send-type first.** Read backwards once already this session.
3. **`peer-wire?` vs `peer-process`** — two tier predicates, used by different call sites. Choose and say.
4. **`Handle :- [T]`** carries a `Shared | Wire` marker; it is in scope inside the macro and not outside.
   That is what stopped the orchestrator's probe, and it is not evidence about `select`.
5. **A test that hangs forever is a floor TIMEOUT, not a failure.** Give row 3's test a bound so it
   reports rather than wedging the suite.
