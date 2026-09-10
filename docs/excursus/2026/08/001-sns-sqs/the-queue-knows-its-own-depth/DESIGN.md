# DESIGN — the queue knows its own depth

## Why

Under the builder's **networking-first** ruling, process-boundary crossings are the cost. `b8f894eca`
made them visible for the first time: **`rt-total` ≈ 11 250 per standard run, of which `rt-store` is
6689 — 59 %.** Broken down by operation:

```
put    2007  30.3%   data — send + claim re-put, ~2 per batch per tier
count  1997  30.2%   ⛔ NOT DATA
scan   1604  24.2%   data — one per receive
delete 1010  15.3%   data — one per ack batch
```

**The data path is near-minimal.** `count` is the only non-data category, and it is 30 % of the store
budget and ~18 % of the entire round-trip budget.

## ⛔ And my first read of where `count` goes was wrong

I reported that `stats` makes three `count-index` calls with two of them identical. **It does not call
`total` at all** — `sqs.wat:1266` charges `store-calls + 2`, the two `depth` calls, and nothing else.

`total` is called at **`sqs.wat:470`, on the SEND path**, to compute admission (`lim = cap + 1`, then
`room = cap − depth`). Its own comment says *"`total` is ONE count-index."* So:

```
~1191 (60%)  ADMISSION  — one count-index on EVERY SEND, hot path
~ 806 (40%)  STATS      — depth's two count-index calls
```

★ **The dominant observability cost is not observability at all — it is a capacity check on the hot
path.** That is a better stone than the one I described, and it only appeared because I read the caller
instead of assuming it.

## What it delivers

**`total` becomes a maintained field, not a store query.** The queue already carries eight counters in
the `:queue::Counters` record landed at `9f1392630`; row count is exactly that kind of quantity.

**Two crossings disappear per site:**

1. **Admission** (`:470`) reads the maintained value → **one count-index gone from every send.**
2. **`stats`** (`:1266`) then needs only the *visible* count at `now-ns`, deriving
   `unacked = total − visible` → **depth drops from two count-index calls to one.**

Combined, `count` should fall from ~1997 toward ~400 — **~1600 crossings, 24 % of the store budget and
~14 % of the whole run.**

## ⭑ It is EXACT, and it is MORE faithful — not a trade

**Exact:** only two operations change the number of rows — an accepted send (`+n`) and an ack (`−n`).
Visibility expiry does **not**: an expired message is still a row. So a maintained count is not an
approximation of the store; it is the same number arrived at by addition.

★★ **More faithful:** real SQS publishes **`ApproximateNumberOfMessages`.** Our exact `count-index`
query is *less* faithful than a maintained counter, not more. **This is the opposite of the push-invert
trade rejected earlier** — that one bought speed by diverging from the referent; this one moves toward
it.

## The one contract decision

⛔ **The maintained value must be provable against the store, not merely asserted.** A counter that can
drift is a lie that reads like a fact, and this campaign has retired several of those today.

So the stone ships **a drift check**: at a point where the store is already being queried (`stats`'s
remaining `depth` call, or `-tick`), compare maintained `total` against a queried one and **raise on
mismatch**. ⚠ Not a counter, not a log line — a **raise**, because a silent drift would corrupt
admission, and admission over-accepting past `cap` is a contract violation, not a metric error.

If the check itself costs a crossing, it runs **only on a path that was already querying** — never as a
new call. That is the standing discipline from `b91d70184`: the instrument must not inflate what it
measures.

## Out of scope = rejected

- **`put`, `scan`, `delete`.** Near-minimal for a claim-based queue: send + visibility-claim re-put,
  one scan per receive, one delete per ack batch. Whether that *protocol* is right is a
  DDB-faithfulness question, not a waste question. **Affirmatively cut.**
- **The poll loop** (`rt-poll` 353, 3.1 %). Its store amplification lands in `depth`, which this stone
  halves anyway.
- `rt-unknown ≤ 1795` — the three classes bounded rather than counted at `b8f894eca`. Closing them needs
  every counter's **unit** documented, which is its own stone and the cheap prerequisite to four
  estimate errors made today.
- `fill`, `drain`, `setup`, tier-1 fault injection, the chaos assertions.

## ⚠ What must not be claimed

⚠ **This is a crossing-count win, not a latency win.** On IPC at 2.8 ms/crossing it is worth ~4.5 s
serialised — but ~3× concurrency is achieved, so the wall clock may barely move. **A SCORE reporting a
wall-clock speedup has measured noise**; the number that matters is `rt-store` and `count`.
⚠ And the drift check must never be reported as "no drift observed" from a run that never exercised
expiry or refusal. Say which paths were exercised.
