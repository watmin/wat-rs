# EXPECTATIONS — a give-up has no form for attempts

**Written before the strike.** Graded by the orchestrator against its OWN re-run, never the report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `Verdict` has NO attempt variant | `grep -o 'Verdict::[A-Za-z]*' wat-scripts/fanout/circuit.wat \| sort -u` | exactly `Done`, `Stalled`, `Ceiling` — **no `Exhausted`, no `Attempts`** |
| 2 | `require!` takes the type, not a String | read `circuit.wat` `require!` | `[v <- :fanout::Verdict]`; **no `= ""` anywhere in it** |
| 3 | empty-string success is gone corpus-wide | `grep -o 'require! *(' -r wat-scripts \| wc -l` vs `grep -c '(:wat::core::= r "")' circuit.wat` | second is **0** |
| 4 | no attempt bound survives in the two rebuilt pollers | read `join-publishers*`, `poll-until-visible-zero*` | neither takes a `left`/attempts parameter; both take `stall-k` + `ceiling-ms` |
| 5 | `join-publishers*` give-up finally prints numbers | read its give-up path | constructs `Stalled` or `Ceiling` carrying elapsed + snapshot — today it raises with **none** |
| 6 | floor | `./scripts/floor.sh` → **Summary line** | `5237 tests run: 5237 passed`, 22 skipped, **0 FAIL** |
| 7 | tests compile | `cargo nextest run --release --no-run` | exit 0, no `^error` |
| 8 | happy path unchanged | `… 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 9 | chaos proof still green | `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | exit 0, `distinct=100;dup=0` |
| 10 | the verdict is TRIPPABLE (negative control) | hand-construct `Verdict::Stalled`, pass to `require!` | it RAISES and the message names K and elapsed |
| 11 | `sweep-drained?` reported, not blind-fixed | read the SCORE | a ruling WITH evidence about `= 0` vs `<= 0` and the `-1` sentinel; a change only if the evidence says so |
| 12 | `wat/service.wat` untouched | `git diff --name-only` | `owner-recv-loop` is out of scope; if it appears, STOP-4 fired |
| 13 | the two stall constants stay equal | `grep -o 'drain-stale-polls\|fill-stale-polls' … ` + read | both still 600 |

## Runtime prediction

**45–90 minutes.** One enum, one consumer, two pollers rebuilt and two migrated, all in one file;
the floor is ~500 s and will likely run twice.

## What makes this PARTIAL rather than passed

- Row 1 green but row 4 red → the type is clean and a caller still counts attempts internally; the
  gate is decorative.
- Row 10 red → a verdict nobody can trip. That is exactly the `GaveUp` question from the last stone,
  and the answer there was "reachable only via two of three arms" — I want this one checked, not
  assumed.
- Row 5 red → `join-publishers*` still gives up without numbers, which is the single worst instance in
  the class and the reason it is named first.

## Trap-doors named before the strike

1. **`-1` is the unreadable-tier sentinel.** It is why `sweep-drained?`'s `= 0` may be CORRECT.
   STOP-2 — report with evidence, do not "fix" on the word "pinnable".
2. **`require!`'s raise text may be asserted by a floor test.** Keep today's wording; if a golden
   disagrees, STOP-3 — do not patch the golden.
3. **Three carriers of wat source.** `.wat`, `.rs` string literals, `.jsonl`. Two floor reds came from
   exactly this in stone 1a.
4. **`grep -c` counts LINES.** Count occurrences with `grep -o … | wc -l`.
5. **`cargo build --release` does not compile tests.**
6. ⚠ **This gate does not close the hand-rolled case.** A new poller can still count attempts if it
   never calls the combinator; the type only stops it REPORTING one. That is the highest rung the
   material allows and the SCORE should not claim more.
