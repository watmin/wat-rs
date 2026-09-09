# EXPECTATIONS — item (c) stone A

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **no double-count** | a span: `incr :requests` ×3 → flush → `incr` ×2 → `close`; sum the emitted `:requests` metrics | **exactly 5**. An 8 is the totals-not-deltas bug, and it is the whole reason this stone precedes the timers |
| 2 | logs are batched | N logs under a small threshold, count `write-logs` calls | **< N**. Today it is exactly N (a batch of one per line) |
| 3 | logs survive to the flush | the batch that lands | every log conj'd before the flush is in it, in order |
| 4 | duration aggregate unchanged | one `timed` set, read the metrics | `<name>/count` (Count) and `<name>/duration` (Nanos, the SUM) — byte-identical naming to today |
| 5 | duration fidelity added | same run | one `<name>/sample` (Nanos) **per sample**, values equal to the samples |
| 6 | reset actually resets | flush twice with no activity between | the second flush emits nothing, not a repeat |
| 7 | `close` is the remainder | flush, then `close` with nothing new | `close` emits nothing and still reports `Done` |
| 8 | threshold from the contract | `git diff` — find the size threshold | read from the op's declared `:max-request-bytes`; **a literal is a FAIL** (STOP-3) |
| 9 | no timer smuggled in | `grep -n 'Alarm\|NoReplyAndArm\|-flush' wat/telemetry/span.wat` | none. Stone B |
| 10 | blast radius | `git diff --stat` | `wat/telemetry/span.wat` (+ `wat/telemetry.wat` only if the durable field forces it). No `Journal`, no `Numeric`, no surface |
| 11 | the existing gates hold | `cargo nextest run --release -E 'test(probe_arc278_span)'` | pass — `probe_arc278_span_service` asserts a counter of 2 reaches the store through the real chain |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5137+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 60–120 minutes. Most of it is extracting `close`'s fold into a reusable
emit-and-reset; the accumulator and the fidelity metric are small.

## Trap doors, named in advance

- **Two emission paths** (one for `close`, one for the flush). Passes rows 2–5 and 12; only row 1
  catches it. This is THE failure of this stone.
- **Emitting totals instead of deltas.** Same shape, same row.
- **A literal byte threshold.** Works today, silently diverges from the server's cap the moment the
  contract changes. Row 8.
- **Dropping the fidelity samples on reset before emitting them.** Row 5.
- **Firing on nothing** — a "buffer" that still writes through per log passes rows 1, 4–12. Row 2 is
  the only one that catches it.
