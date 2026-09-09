# EXPECTATIONS — the inbox holds messages, not pairs

Written **before** the strike. The largest change in this campaign; correctness rows outrank every
timing row.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **fanout is complete** | n=1000/2000/4000, `vis-ms=1000` | `distinct = n×m` and `dup = 0`. **Every subscriber gets every message.** This outranks everything below |
| 2 | ⛔ **the ack still follows the sends** | read the diff | the worker acks the inbox **after** the sub-queue writes (`:560` before `:579` today). That ordering is what makes a mid-expansion death safe |
| 3 | ★ **`Accepted c` is in MESSAGES** | read the diff + a run | `c ≤ count(msgs)`, never derived from `pairs / nsubs`. The publisher's admission no longer mentions `nsubs` |
| 4 | ★ **the `rem`/`need` top-up is DELETED** | `git diff` | gone from the file, not merely unreached. The state it repaired cannot occur |
| 5 | ★ **the `"{i}|"` tag is gone** | `grep` | no subscriber index written into a body at publish, and no `split` recovering one in the worker |
| 6 | ★★ **the inbox stops refusing** | the per-tier line | inbox `refused` → **0** (it was **1209–1233**). `:cap 64` is UNCHANGED, so this answers the cap question empirically: 64 *messages* is a different object from 64 *pairs* |
| 7 | **inbox depth reads in messages** | the per-tier line | `accepted` on the inbox tier ≈ `n`, not `n×m` |
| 8 | **the per-op counters still reconcile** | the per-tier line | `put+delete+count+scan` ns and calls sum to `store-ns`/`store-calls`, remainder 0 |
| 9 | **no phase regresses** | the sweep | `drain`, `fill`, `setup`, `collect` within their known bands (drain 4000/1000 was 4.44–4.58; fill linear; setup ~12.4 s constant) |
| 10 | **blast radius** | `git diff --stat` | `wat-scripts/topic/sns-fanout.wat`, plus `circuit.wat` **only** if the report line needs it. **No `wat/`. No `src/`** |
| 11 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 12 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

⛔ **Row 1 is the whole stone.** This changes who performs the fanout. If a subscriber misses a message,
nothing else matters — `distinct` is exactly the instrument that detects it, and it must equal `n×m` at
every size.

⛔ **Row 2 is the safety property that must survive untouched.** Today the worker sends to the sub queues
and *then* acks the inbox. A worker dying between those leaves the inbox entry to expire and be
re-processed — duplicate deliveries to subs already written, which at-least-once permits and the `seen`
dedupe absorbs. **Reverse that ordering and a crash loses messages silently.**

★★ **Row 6 is a falsifiable prediction, and it answers the cap question without touching the cap.** The
inbox refuses 1209–1233 times per run today because a 40-body batch cannot fit in 64 slots. With messages,
a batch is ≤10. If `refused` goes to 0, `:cap 64` is proven a non-constraint **by measurement** rather than
by my argument — and STOP-3 forbids changing it, because changing it would destroy the measurement.

★ **Row 4 says deleted, not unused.** The `rem`/`need` top-up exists to repair a message smeared across
both tiers. With messages in tier 1 that state cannot occur, so the repair must be *removed* — dead code
that reads as safety is the defect class this campaign has been pulling out all session.

## Runtime prediction

**2–3 hours.** The largest change here: publish stops expanding, the worker starts, the top-up goes, the
framing changes on both sides. Then ~7 minutes of sweep and ~8 of floor. Expect more edit sites than the
sketch names — every prior stone did.

## Trap-doors

- **The worker already holds `sub-addrs`** and already loops `0..nsubs`. It should need no new plumbing; if
  it does, that is STOP-4.
- **Framing changes on both sides at once.** Publish emits `"{msg}|{t0b}"`; the worker emits
  `"{msg}|{t3b}"` per sub. A mismatch will surface as a parse failure or a wrong `key-of`, not as a silent
  loss — but check `distinct` first regardless.
- **`t0b` vs `t3b`** — the DESIGN records that late subscribers already carry a later timestamp and that
  this is the implemented contract. Do not try to make them equal.
- **`send-all` vs `send`** — publish uses `send-all` with a batch; the worker's per-sub write uses `send`.
  Keep each as it is.
- **`:max-entries [msgs 10]`** is contract. Do not raise it to fit anything.

## What this stone does NOT claim

⚠ It does **not** remove `cap` from the Queue record — row 6 measures whether the inbox cap still binds.
⚠ It does **not** address the residual drain superlinearity, `scan-index`'s slope, or the counter-carrier
tax.
⚠ It does **not** touch `wat/`, `mem.wat`, or the store.
