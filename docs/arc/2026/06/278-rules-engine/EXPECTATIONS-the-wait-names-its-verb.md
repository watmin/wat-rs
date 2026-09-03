# EXPECTATIONS — the wait names its verb

Written **before** the strike. Re-run by me on a quiet box (`ps -eo args | grep -E
'cargo|nextest|release/wat'` empty first). The result cannot move these.

## THE BASELINE — measured by me, 2026-09-03, five runs, quiet box

```
total=8000; distinct=8000; dup=0        on all five
publish  25582 / 26170 / 26604 / 26735 / 26403 ms      spread 4.5%
         → 299–313 deliveries/s
```

Five runs because a timing row needs a distribution, not a sample. That is not the flake re-run the
floor doctrine forbids — that rule protects **red assertions**, where a green re-run destroys
evidence. For a timing row the spread *is* the evidence.

| # | what | command | expected |
|---|---|---|---|
| 1 | ★★ **no magnitude comparison survives** | `grep -n 'wait' wat-scripts/queue/sqs.wat` | **zero** `<= 0` / `> 0` / `< 1` against a wait. The mode comes from the constructor. **This is the contract decision — row 1 failing fails the stone** |
| 2 | ★ the fork is a `match` | read `sqs.wat:487` | `match` on `:queue::Queue::Wait`, two arms, no `if` on a number |
| 3 | ★ zero cannot be spelled | a probe with `:UpTo (:wat::time::Millisecond 0)` | **does not type-check** (Stone A, rung 3) |
| 4 | ★ zero over the wire is refused, service lives | `probe-zero-at-the-boundary.wat` | still `zero=[MALFORMED…]; then=[ok:…]` — B-pre's guarantee must survive this stone |
| 5 | ★ the invariant holds | `./target/release/wat wat-scripts/fanout/circuit.wat`, **five runs** | `total=8000; distinct=8000; dup=0` on **all five** |
| 6 | throughput did not regress | same five runs | publish within **25.5–27.0 s**. Outside that band either way is a **finding to report**, not a number to tune |
| 7 | the codemod is real and recorded | `wat-scripts/fixes/<name>.wat` exists; re-run it | second run reports **0 changes** (idempotent) |
| 8 | the codemod's census, not mine | the finder's own count, reported before applying | **13 sites / 6 files** is my hypothesis. If the finder disagrees, **the finder is right** |
| 9 | the clamp is untouched in behaviour | `git diff wat-scripts/queue/sqs.wat` at `:737` | comment added, **condition and constants unchanged** |
| 10 | the six `1000000` arm delays untouched | `:297 :388 :470 :491 :578 :629` | unchanged |
| 11 | `sqs.wat:11-12` no longer half-lies | read it | says both: time types cross now (B-pre), and `now-ns`/`visibility-ns` stay i64 so a fixture drives the clock |
| 12 | prose the codemod cannot reach | `grep -rn 'wait-ns' --include=*.wat .` | only comments remain, and they were handled in a named manual pass — **not left lying** |
| 13 | helpers not merged | `sqs.wat:783,796,868` | three helpers still three. Merging is Stone D |
| 14 | the floor | `scripts/floor.sh`, **Summary line** | `5213 run / 5213 passed`, **or** the known Stone D race — see below |

## ⛔ ROW 14 — the one red that is not yours

`probe_async_publish::refused_subscriber_is_retried_not_dropped` carries a **live, proven,
deterministic race** that this stone does not fix: `take-one` destructively consumes the message
`wait-pending` then waits for. Reproducer: `probe-refused-retry-self-consumes.wat`, `gap=300 →
SPINS-FOREVER`, 3/3.

It has now passed twice and timed out once. **If it goes red, that is the race firing** — report it,
point at the reproducer, and do not chase it. **If it goes red, do not re-run to make it green:** a
green run is the coin landing the other way, not a disposition.

Any *other* red is yours.

## RUNTIME PREDICTION

**60–100 minutes.** The surface change is small and the shape is already proven working in
`probe-nonzeroduration-crosses-the-wire.wat`. The time is in writing and dry-running the codemod,
which is the right place to spend it. If the codemod fights you past ~45 minutes, STOP-2 — report
rather than hand-editing the corpus.

## TRAP-DOOR RISKS

1. **`Waiter/deadline-ns` must stay an i64.** It is a computed instant, not a mode. Converting it is
   scope creep and breaks the tick's `<= now` comparison, which is arithmetic on a measurement and
   is correct.
2. **The codemod cannot rewrite comments** — it walks forms. `circuit.wat:144` and
   `probe-parked-waiters-stop.wat:4,7` discuss `wait-ns` in prose. Row 12.
3. **`probe-refused-retry-self-consumes.wat:104-109` takes `wait-ns` as a parameter.** It is a
   scratch probe and it is the Stone D reproducer — it must keep working, because it is the evidence
   for the next stone.
4. **Two probes under `scratch-pad/` are in the census** and the `every_wat_scripts_file_loads` gate
   type-checks them. Missing one turns that gate red at the end.
5. **`SendResponse::Full` and the visibility fields are untouched.** Only the wait moves.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 not run, or run as a summary rather than the grep output. It is the contract decision.
- Row 5 or 6 from fewer than five runs.
- Row 8 reporting **my** census instead of the finder's. My last three censuses were each wrong in a
  different way — one omitted three constructors, one omitted an entire directory, one reported an
  empty grep as a fact about the tree.
- Row 7 not re-run — idempotence unverified means it is not a recorded migration.
- The corpus hand-edited because the codemod was inconvenient.
- Row 14 reported green after a re-run of a red.
