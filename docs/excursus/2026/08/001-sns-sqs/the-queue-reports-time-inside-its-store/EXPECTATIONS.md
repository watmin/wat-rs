# EXPECTATIONS — the queue reports time inside its store

Written **before** the strike. Judged on **the instrument**, never on what it shows.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ `store-ms` is reported | `… circuit.wat 500 4 3 8192 true` | `store-ms=` on the phases line |
| 2 | it is bounded by the run | any run | `store-ms` **<** `total`; and `store-ms` ≥ 0 at every depth |
| 3 | it tracks the work | n=500 / 1000 / 2000 | grows monotonically with n |
| 4 | every counting site also times | read the eight `Store/*` sites | **8 of 8** accumulate `store-ns`; none counts without timing |
| 5 | overhead is invisible | n=500 / 1000 | `pairs/sec` within **±15 %** of 3976 / 3200 |
| 6 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; `store-ms` additive |
| 7 | correctness untouched | n=2000 fill-first | `total=8000;distinct=8000;dup=0` |
| 8 | the eight ripple sites took `_` only | `git diff -- wat-scripts/topic wat-scripts/scratch-pad` | **8 insertions, 8 deletions**, every one a trailing `_` |
| 9 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 10 | scripts load | `every_wat_scripts_file_loads` | green |
| 11 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± added) |

⚠ **Row 5 is the one that could quietly fail.** The overhead is priced at 0.20 % of the drain, so it
should be invisible — but priced is not measured *in situ*. If the curve moves beyond the band, the
instrument is changing what it measures and that is a finding, not a rounding error.

⚠ **Row 3 says "grows monotonically", not a ratio.** The ratio is the *output*. Pinning it would gate
on the answer.

⚠ **No row asserts anything about `store-ms / drain`.** That fraction is the whole point and must not
appear as an expectation.

## Runtime prediction

**45–60 minutes.** One field, eight clock pairs at sites already being edited, one sum, one format,
and the eight known ripple sites. The floor is the long pole.

## Trap-doors named in advance

- **Nanos on the wire, millis at the format.** Dividing early loses resolution: a single store op is
  ~100–400 µs, so per-op millisecond truncation would floor most of them to 0.
- **`State` reconstruction sites.** `store-ns` rides every arm that already carries `store-calls`.
  A miss is a compile error — but **copying `store-calls` into `store-ns` by cut-and-paste is not**,
  and would produce a plausible number that is really a count.
- **Positional destructures.** Append `store-ns` **last**, after `store-calls`. A middle insertion
  silently reassigns every binding after it across 15 sites.
- **The clock pair must bracket only the call.** Including the surrounding `State` rebuild would fold
  queue-side work into `store-ns` and pre-decide the fork this stone exists to resolve.
- **`store-ns` accumulates across the whole run** — fill and drain both. STOP-6 exists because a
  `store-ms` larger than `drain` is the expected, correct consequence of that, and must be reported
  as such rather than read as a bug.

## The fork this resolves — stated so it cannot be retrofitted

- **`store-ms`/pair grows in step with `drain`/pair** → the time is inside the store; the next
  question is the SQL, and the blast radius becomes the **stdlib**.
- **`store-ms`/pair flat while `drain`/pair grows** → **the store is exonerated**; the time is in the
  queue's own processing or in scheduling between actors, and the search moves somewhere nobody has
  looked.

★ **No prediction.** The previous stone's DESIGN named a winner inside one branch of its own fork and
was wrong to. This one states both and picks neither.

## What this stone does NOT claim

⚠ **It does not explain the slope** (3976 → 3200 → 2501 pairs/sec). It halves the search space.

⚠ **`store-ns` is cumulative, un-broken-down, and not split by phase** — the same limits as
`store-calls`. Any composition claim is unsupported by it.

⚠ **Store *contention* is already eliminated** — structurally, by reading: each subscriber store is
granted to exactly one pid (`circuit.wat:2101`) and the queue is a serializing actor, so at most one
store call is ever in flight. This stone is not measuring that.
