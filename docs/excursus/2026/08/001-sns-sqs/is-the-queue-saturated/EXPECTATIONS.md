# EXPECTATIONS — is the queue saturated?

Written **before** the strike. Judged on **the instrument**, never on what it shows.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ `drain-busy-ms` reported | `… circuit.wat 500 4 3 8192 true` | on the phases line, **divided by `m`** |
| 2 | busy includes store | any run | **`drain-busy-ms` ≥ `drain-store-ms`** at every n — STOP-8 otherwise |
| 3 | busy is bounded by the phase | any run | `drain-busy-ms` ≤ `drain` (per queue, so this must hold) |
| 4 | it tracks the work | n=500 / 1000 / 2000 | grows monotonically with n |
| 5 | every arm accumulates | read the 30 `queue::State` constructions | **all** end-of-op sites add to `handler-ns`; none is skipped |
| 6 | overhead invisible | n=500 / 1000 / 2000 | `pairs/sec` within **±15 %** of 3759 / 3212 / 2418 |
| 7 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; `drain-busy-ms` additive |
| 8 | correctness untouched | n=2000 fill-first | `total=8000;distinct=8000;dup=0` |
| 9 | the eight ripple sites took `_` only | `git diff -- wat-scripts/topic wat-scripts/scratch-pad` | **8 insertions, 8 deletions** |
| 10 | nested-Tuple constructions still zero | `awk` on sqs.wat | **0** — the refactor's gain is not given back |
| 11 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 12 | scripts load | `every_wat_scripts_file_loads` | green |
| 13 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Row 2 is the correctness check on the instrument itself.** `handler-ns` includes the store calls
made inside handlers, so busy ≥ store by construction. If it is not, the accumulation missed sites.

⚠ **Row 5 is the one that fails silently.** A missed `State` construction is a compile error *only*
if the field is required; if it is copied from the old state instead of updated, it type-checks and
under-reports. **Read all 30.**

⚠ **No row asserts anything about `busy / wall`.** That ratio is the entire output.

⚠ **Nothing that varies run to run is banded.** Five rows in this arc have policed noise.

## Runtime prediction

**60–90 minutes.** One `:ephemeral` field, 30 accumulation sites, one `StatsResponse` field, the
16-site ripple, one sum, one boundary delta, one format field. The widest edit of the arc. The floor
is the long pole.

## Trap-doors named in advance

- **Copying `handler-ns` forward instead of updating it** type-checks and silently under-reports.
  This is the failure row 5 exists to catch, and it is why all 30 sites must be read.
- **`-tick` uses `SelfInvocation/start-ns`**, not `Invocation/start-ns` (`sqs.wat:998` is the
  precedent).
- **Nanos on the wire, millis at the format**, as `store-ms` and `drain-store-ms` already do.
- **`/ m`** — the sums span m queues.
- **The `stats` arm times itself**, so the poller's handler time is inside `handler-ns`. Expected;
  derivable from `poll-calls`; report it, do not net it out.

## The fork this resolves — stated so it cannot be retrofitted

- **`ρ = busy/wall` near 1, rising with depth** → the queue is saturated; the +81 % non-store growth
  is **induced queueing**, and the fix is utilisation: fewer ops through the one server, cheaper
  ops, or **more servers**. Only the last changes the exponent.
- **`ρ` low, or flat while the drain grows** → the queue is idle much of the drain; the time is in
  **IPC or scheduling**, and the search moves somewhere nobody has looked.

★ **No prediction.** Four forks in this arc have named or implied a winner they had not earned —
including the one immediately before this. The single-server model in the DESIGN is arithmetic that
fits, and eight mechanisms that also fitted are already dead.

## What this stone does NOT claim

⚠ **It does not explain the slope.** It decides which of two searches to run next.

⚠ **It does not make anything faster.** It is the third instrument in a row, and the last one needed
before a fix can be aimed rather than guessed.

⚠ **It yields the three-way split** the arc has been circling — `busy`, `queue compute = busy −
store`, `not-in-queue = drain − busy` — of which **the middle term has never been measured.**
