# EXPECTATIONS — the drain gives up on a stall, not a budget

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the happy path is untouched** | `circuit.wat 2000 4 3 8192 true 1000`, 3 runs | `distinct=8000`, `dup=0`, inbox `accepted=2000`, verdict empty (no raise) |
| 2 | ⛔ **THE ASYMMETRY IS GONE** | `circuit.wat 50 2 2 32 false 0 0 1000 42`, alone ×3 **and** 8-concurrent ×1 | the **same verdict** in both conditions. Today: passes alone, 3–4 of 8 fail under load |
| 3 | ⛔ **a real stall is still caught** | reason it from the code, and say how you convinced yourself | a system that stops delivering must still produce a non-empty verdict. **A check that can no longer fail is worthless** |
| 4 | ★ **the two worlds are distinguishable** | the verdict strings | `drained-stalled` and `drained-timeout` are distinct, and each names `acks`, `outbox` and `elapsed` |
| 5 | ★ **zero extra round-trips** | read the diff | `acks` comes from the `Queue/stats` reply `depth-of` already makes. **No new stats call per poll** |
| 6 | ★ **`acks` is used for stall, never completion** | read the diff | the drained condition is still `sweep-drained?` **and** `box = 0`. No `Σacks == n×m` test anywhere |
| 7 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 8 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |
| 9 | **the five passing chaos scenarios still pass** | `--run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'` | `drop_check_tiny`, `drop_recv_tiny`, `r2_drop_after_tiny`, and both n=2000 write scenarios: unchanged |
| 10 | **blast radius** | `git status --porcelain` | `circuit.wat` + the SCORE. Anything the gate forced, named |

## ★★ Row 11 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 11 | ★★ **`acks` is monotone and already in the reply** | confirmed from the source, not assumed |

The entire design rests on `(:queue::Stats/acks qst)` being present in the reply `depth-of` already
receives, and on acks never decreasing. **If either is false, my premise is wrong and the stone
cannot be built as drawn** — report that and STOP. It is a complete and valuable result; it costs me
a design and buys a fact.

## ⚠ Row 12 — what this stone must NOT be credited with

| # | what | expected |
|---|---|---|
| 12 | ⚠ **the 9–19 stuck entries are not explained** | the SCORE must NOT claim delivery is proven |

This stone makes the question answerable and does not answer it. If the contended runs now report
`drained-timeout`, that says *"still progressing when time ran out"* — **suggestive, not proof.** Any
sentence claiming messages are no longer lost, or were never lost, is out of scope and unsupported.

## ⚠ Row 13 — a shift here is expected, not a regression

| # | what | expected |
|---|---|---|
| 13 | ⚠ **`drain` timings move** | re-baselined, and reported as a change of instrument |

`drain` is measured *through* this poller. Take fresh before/after numbers; do **not** compare against
the banked 1194 / 2506 / 5305, and do not call a shift a regression. ⚠ Note the instrument's own
floor: two measurements of identical code sit ~1 % apart between sessions, so nothing below ~1 % is
readable at all.

## Runtime prediction

**60–90 minutes.** One loop rewritten, one accessor widened, plus the asymmetry experiment
(4 invocations + one 8-way concurrent burst, ~3 min) and the floor (~8 min).

## Trap-doors

- **`acks` is not a completion test.** Under redelivery a message is acked more than once, so
  `Σ acks` can exceed `n×m`. Using it for completion would end the loop early and silently break
  row 1.
- **`depth-of` returns a `Tuple` of 2 today** and its callers destructure it. Widening it touches
  every caller — the compiler will name them; trust it over any list.
- ⚠ **`(:wat::core::Tuple …)` has no fourth accessor** (`wat/core.wat:1737`, recorded in a prior
  SCORE). If you need more than three values out, use a record, not a wider tuple.
- **`drained-unread` must stay** — it is a different failure (a stats call that could not be read) and
  it outranks both new verdicts.
- **The 8-concurrent burst is the instrument for row 2.** Run it through `capped.sh`; 8 copies at
  `--limit 2g` each sit comfortably inside the shared slice.
- **Do not change `:cap`, `sub-cap`, `vis-ms` defaults, or `:max-entries`.** Row 2's comparison
  depends on the scenario being byte-identical to the one measured at `088f2669f`.

## What this stone does NOT claim

⚠ Not the observer effect (2–3× the budget spent on the poller's own round-trips) — affirmatively cut.
⚠ Not the seven chaos tests' assertions or ignore status.
⚠ Not wiring the inbox into fault injection.
⚠ Not `wat/`, `src/`, `sqs.wat`, or `sns-fanout.wat`.
