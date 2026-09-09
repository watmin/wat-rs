# EXPECTATIONS — perf 1: the incremental byte measure

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **the differential** | across a range of batch shapes/sizes, compare the running total against `string::length(edn::write(request))` | **equal**, or **≥** with the direction stated (STOP-1). The only gate plausible-but-drifting arithmetic cannot pass |
| 2 | ★ **no drift after a partial drain** | sink accepts one chunk, refuses the next; re-check row 1 against the remaining suffix | still equal/≥. A drift here is invisible until a batch is refused |
| 3 | the cost is fixed | re-run `probe-span-log-cost.wat` | per-doubling ratio approaches **2×**. Baseline was 224 / 618 / 1848 ms — 2.76× then 2.99× |
| 4 | never under-count | the direction, stated in the SCORE | exact or high. An under-count ships an over-cap batch the server refuses |
| 5 | flush points unmoved | same input, before vs after | identical flush points (STOP-2) — a cost change, not a semantics change |
| 6 | chunker untouched | `git diff` on `write-*-batched` | empty (STOP-4) |
| 7 | reset with the accumulator | flush to empty, then check the total | zero |
| 8 | prior gates hold | `cargo nextest run --release -E 'test(probe_arc278_span)'` | all pass, **assertions unedited** |
| 9 | no runtime/surface change | `git diff --stat src/runtime.rs`; Span op count | empty; 5 |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5163+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 45–90 minutes; the framing arithmetic and its differential gate carry it.

## Trap doors, named in advance

- **Summing item lengths and ignoring container framing.** Under-counts by a few bytes per item —
  invisible until a batch sits just under the cap and the server refuses it. Row 1.
- **A total that drifts after a partial drain.** Item (b) leaves a suffix; the total must match it.
  Row 2, and it is silent until it bites.
- **Fixing the cost by weakening the trigger** (measuring every Nth call, say). That moves flush
  points. Row 5.
- **Firing on nothing:** rows 1, 2, 4–10 all pass if the code still re-encodes the batch and merely
  also keeps a total. Row 3 is the only one that catches it.
