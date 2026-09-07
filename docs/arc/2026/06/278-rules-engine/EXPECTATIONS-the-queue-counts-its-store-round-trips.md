# EXPECTATIONS — the queue counts its store round trips

Written **before** the strike. This stone is judged on **the counter**, never on what the counter
reveals.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ `store-calls` is reported | `… circuit.wat 500 4 3 8192 true` | `store-calls=` present on the phases line |
| 2 | it counts round trips, not messages | compare to `queue-receive-calls` | `store-calls` **>** `receive-calls` at every n |
| 3 | it scales with the work | n=500 / 1000 / 2000 | roughly **doubles** as n doubles |
| 4 | no timing was added | `grep -n 'time::now' sqs.wat` | **no new** occurrences around `Store/*` sites |
| 5 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; `store-calls` additive |
| 6 | correctness untouched | n=2000 fill-first | `total=8000;distinct=8000;dup=0` |
| 7 | curve undisturbed | n=500 / 1000 | within **±15 %** of 3922 / 3160 |
| 8 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 9 | scripts load | `cargo nextest run --release every_wat_scripts_file_loads` | green |
| 10 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± added) |

⚠ **Row 7 is the one that could quietly fail.** A counter is nearly free, but `sqs.wat` is on the hot
path and the increment sits inside a serializing actor. If the curve moves more than the band, the
instrument is changing the thing it measures and that is a finding, not a rounding error.

⚠ **Row 3 says "roughly doubles", not a band.** The exact ratio is the *output* of this stone. Pinning
it would be gating on the answer.

## Runtime prediction

**30–45 minutes.** One field, eight increments, a dozen `State` reconstructions, one sum, one format
string. The floor is the long pole.

## Trap-doors named in advance

- **`State` reconstruction sites.** `receive-calls` is carried through ~a dozen of them; the new
  field must ride every one. Each miss is a compile error — but a miss that copies `receive-calls`
  into `store-calls` by cut-and-paste is **not**, and would read as a plausible number.
- **Positional destructures.** `StatsResponse::Ok` is matched positionally in several places. Append
  the field **last**; a middle insertion silently reassigns every binding after it.
- **The `:990` / `:1039` sites** are the batch put/delete inside a different arm than `:384` / `:729`.
  Both pairs are real round trips; counting only one pair halves the number invisibly.
- **One increment per ROUND TRIP, not per row.** A `put` of ten rows is **one** call. Counting rows
  would inflate it ~10× and look like the discovery this stone is not making.

## The fork this resolves — stated so it cannot be retrofitted

With `store-calls` per pair in hand:

- **flat** → op count is linear, **cost per op is rising with table size**. The next stone times the
  store ops, and will know which to time.
- **rising** → we issue more store work per message than anyone thought. The slope is a **count**
  problem, not a cost problem — cheaper to fix, and nobody suspected it.

★ **No prediction is offered about which fires.** Both are real outcomes; a stone that passes only on
one of them is measuring its own hope.

## What this stone does NOT claim

⚠ **It does not explain the slope** (3922 → 3160 → 2477 pairs/sec). It makes it decomposable.

⚠ **It does not time anything.** Deliberately — two `time::now` reads per store op is ~34 000 extra
clock reads at n=2000, inside the phase under measurement.

⚠ **It does not touch `collect`** (15.7 s measured today vs 5.9 s in the tracker) or the
`Lost`/`Closed` → `Exhausted 0` conflation. Both open, both unrelated.
