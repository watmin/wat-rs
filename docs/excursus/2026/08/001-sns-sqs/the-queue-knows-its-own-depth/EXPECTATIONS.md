# EXPECTATIONS — the queue knows its own depth

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **correctness untouched** | `2000 4 3` ×3 | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise |
| 2 | ⛔ **the reconciliation identity holds** | every tier of every run | `put+delete+count+scan` sums to `store-calls`/`store-ns`, **remainder 0** |
| 3 | ⛔ **the drift check exists and RAISES** | read the diff; then a run | maintained vs queried compared on an already-querying path, mismatch → raise. Not a counter, not a log line |
| 4 | ⛔ **the drift check adds no crossing** | read the diff | it rides a query already being made. A new `count-index` for the check is STOP-1 |
| 5 | ★ **admission stops querying** | `sqs.wat:470` region | no `count-index` on the send path; the maintained field is read instead |
| 6 | ★ **stats halves its depth cost** | `sqs.wat:1266` | `store-calls + 1`, not `+ 2`; one `count-hi` at `now-ns`; `unacked = rows − visible` |
| 7 | ★ **`count` falls materially** | the tier lines | `count` from ~1997 toward ~400. ⚠ Direction and rough magnitude; I name no exact figure |
| 8 | ★ **`rt-store` falls** | the budget line | from ~6689 by roughly the `count` delta, and `rt-total` with it |
| 9 | **the cap still binds the same way** | inbox `refused` at `2000 4 3` | in the band measured at `e0c552bf0`+ (186–191 at n=2000). **Admission must not become more permissive** |
| 10 | **the stall gate still fires** | `circuit.wat 5 1 0 32 false 0` | rc 2, `drained-stalled` |
| 11 | **chaos still passes** | `--run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'` | `7 tests run: 7 passed` |
| 12 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 13 | **the floor holds** | `scripts/floor.sh` | **Summary line**: 5237 passed, 22 skipped, 0 FAIL, 0 TIMEOUT |
| 14 | **blast radius** | `git status --porcelain` | `sqs.wat` + the SCORE. Anything forced, named |

## ★★ Row 15 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 15 | ★★ **only accepted-send and ack change the row count** | and expiry does not |

⛔ **This is my claim and the whole exactness argument rests on it.** If a third writer exists — a
redelivery re-put that inserts rather than updates, a partial delete, a store-side rewrite, the
`seen-ids` path, anything — **the maintained value can drift and my design is wrong.** Report the path.
It costs me a DESIGN and buys the real shape.

⚠ And note my record: **four round-trip estimates today, all wrong** — 55× high, 2× low, 19 % high, and
a 19× re-ranking error. **This row is where that pattern would land next**, so verify it from the code
rather than from my sentence.

## ⚠ Row 16 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 16 | ⚠ no wall-clock speedup; drift not proven by an unexercised path | stated in the SCORE |

⚠ This is a **crossing-count** win. ~4.5 s serialised on IPC, but ~3× concurrency is achieved — **wall
clock may barely move, and that is the expected outcome, not a disappointment.**

⚠ And *"no drift observed"* means nothing from runs that never exercised **expiry** or **refusal**. The
SCORE must say which paths ran. A drift check that never faced a redelivery has not been tested.

## Runtime prediction

**2–3 hours.** The field is small; the risk is entirely rows 3–4 and 15 — placing a free drift check, and
proving the row-count invariant. Expect more sites than the sketch names; every stone today did.

## Trap-doors

- **`lim = cap + 1`** at `:470` — the query is deliberately bounded one past the cap so overflow is
  visible. A maintained field has no `lim`; make sure the admission arithmetic still distinguishes
  *at cap* from *over cap*.
- **`total`'s `_now-ns` is unused** (underscore-prefixed) — that is *why* it is maintainable. `depth`'s
  `now-ns` call is genuinely time-dependent and **must stay**.
- **`ack` deletes by id and may delete fewer than requested.** Decrement by what was actually deleted,
  never by `count(ids)`.
- **A refused send must not decrement or increment.** `Accepted 0` changes nothing.
- **`:queue::Counters` is the established carrier** (`9f1392630`) — eight counters already ride it. But
  note the measured lesson from that stone: adding a field to a record rebuilt on the hot path is not
  free. **Row 2's identity and row 7's numbers are how you tell.**
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, the tick, or the poll
  wait.** Comparability depends on the topology being byte-identical.
