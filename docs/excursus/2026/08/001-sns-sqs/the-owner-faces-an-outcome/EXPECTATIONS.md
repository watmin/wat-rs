# EXPECTATIONS — the owner faces an outcome

**Written before the strike, so the result cannot move the goalposts.** Graded by the orchestrator
against its OWN re-run, never from the report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the 15 s crash stops being a crash | `./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | **exit 0**, `distinct=100;dup=0` |
| 2 | …and at the other measured seed | same, `… 20260910 …` | exit 0, `distinct=100;dup=0` |
| 3 | …and at 200 bp, the shipped gate's rate | same, `… 7 0 0 0 0 0 200` | exit 0 |
| 4 | happy path byte-identical in result | `… 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 5 | floor | `./scripts/floor.sh` → **Summary line** | `5237 tests run: 5237 passed`, 22 skipped, **0 FAIL** |
| 6 | tests compile (the gap that reddened us once) | `cargo nextest run --release --no-run` | exit 0, no `^error` |
| 7 | `StopOutcome` is faced, not defaulted | `grep -o 'StopOutcome::GaveUp' **/*.wat \| wc -l` | **≥ 1** — a type nobody ever constructs is theatre |
| 8 | no NEW `assertion-failed!` in the two generated bodies | read `wat/service.wat` `stop-method-body` / `hibernate-method-body` | only the `Message(other)` protocol-violation raise remains |
| 9 | the give-up names its bound | read the `GaveUp` construction | carries `waited-ms` AND `last` |
| 10 | `Gone` only for a gone peer | read the arm mapping | `Closed`/`Lost` → `Gone`; `TimedOut`/`Stopped`/`Malformed` → re-recv or `GaveUp`, **never** `Gone` |
| 11 | call-site census honest | `grep -oE '\(:[a-z0-9:_-]+/(stop\|hibernate) ' --include=*.wat -r . \| wc -l` plus the `.rs` pair | 45 `.wat` + 2 `.rs`; if the real number differs, the SCORE says so |
| 12 | chaos gate still 7/7 | the seven chaos tests' status | unchanged from before the strike |

## Runtime prediction

**60–110 minutes.** Two generated bodies is small; the 47 call sites are mechanical but spread across
three carriers, and the floor is ~500 s per run with likely two runs.

## What would make me call this PARTIAL rather than passed

- Rows 1–3 green but row 4 shifted → the outcome change altered the happy path; that is a regression,
  not a win.
- Row 7 zero → `GaveUp` exists and is unreachable, which is the `Reply::Failed` defect repeated (a
  variant nothing constructs is the same class as a variant nothing matches).
- Row 10 violated → the type is new and the collapse survives inside it.

## Trap-door risks, named before the strike

1. **`Admin::Stop` terminates the service** (`service.wat:2287`) — a re-ask converts a slow stop into a
   guaranteed `Gone`. The loop must re-RECV. STOP-1.
2. **A Rust-side twin of the stop template.** `send-recv-form` has one at `src/runtime.rs:6678`; stone
   1b's crash moved rather than cleared because only one copy was fixed. STOP-2.
3. **Three carriers of wat source.** `.wat`, `.rs` string literals, `.jsonl` fixtures. Stone 1a took
   two floor reds from exactly this. STOP-3.
4. **`grep -c` counts lines, not occurrences.** It reported 1 where there were 11 today.
5. **`cargo build --release` does not compile tests.** It passed clean while a test site was broken.
6. ⚠ **Rows 1–3 may still fail for a reason that is NOT this stone.** The disruptor is an aggressive
   fault and the earlier `"publisher stats lost"` death (`circuit.wat:2276`) has **no established
   mechanism** — the publisher died leaving a three-line log. If rows 1–3 fail with THAT signature
   rather than a stop-path one, this stone has still done its job: report it as a separate finding and
   do not chase it into scope.
