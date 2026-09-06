# DESIGN — a message is fanned once

**Stop re-fanning a partially-admitted message.** `wat-scripts/topic/sns-fanout.wat`. This is the
publish attack, and it is aimed by measurement rather than by the four models I got wrong first.

## WHY — the duplicates ARE the regression

`6f77abd4c` made the system correct and 2.7× slower. I proposed three causes and measured all
three dead:

| I claimed | measured |
|---|---|
| the busy-wait on partial resend | added the wait: publish 61944 → **59351**, dup 4226 → **4154**. Nothing |
| cost is per-call, 73 % unaccounted | **wrong model** — see below |
| 5.7 ms/call against a 143 µs floor | that quotient attributes *blocked* time to calls that were waiting |

★★ `SCORE-what-does-publish-actually-cost.md` had already ruled it, and I re-derived it wrongly
before reading it: *"The 37.3 s is main blocked on inbox capacity while workers drain; that wait is
exactly the `outbox` histogram, and it is not in the five units."*

**Publish is drain-bound.** The `outbox` histogram is the whole story:

```
before this regression   outbox  50-250=7966  250-1000=0     max 209ms   (8000 items)
now                      outbox  50-250=4982  250-1000=7148  max 562ms   (12161 items)
```

★★★ **1.53× the items, each waiting a bucket longer.** 12161 ≈ 8000 distinct + 4161 duplicates.
We handed the bottleneck half again as much work.

## THE MECHANISM

`publish` fans msg-major and reports `floor(pairs-accepted / nsubs)`. When the inbox admits a
prefix that does not land on a message boundary, the tail message is **partially fanned**: some of
its pairs are enqueued and will be delivered. The topic reports that message as *not accepted*, the
client re-publishes it whole, and every already-landed pair becomes a duplicate delivery.

Arithmetic: 4226 duplicates over 2000 messages at `nsubs 4` is **roughly one partial-fan per
message** — the steady state, not an edge case.

## ⛔ THE ONE CONTRACT DECISION — the topic COMPLETES what it started

When `k = pairs-accepted` is not a multiple of `nsubs`, the topic immediately sends the tail
message's remaining `nsubs - (k mod nsubs)` pairs — a top-up of at most `nsubs-1` — and reports
`ceil` when it lands.

- top-up lands → the tail message is whole; report it accepted; **no duplicate on re-publish**
- top-up is refused → report `floor` as today; the orphan pairs stay, the re-publish duplicates
  them, and `Seen` absorbs it

★ The topic already owns the fanout; owning its completion is the same responsibility. A message
half-delivered by a service that then tells the client "not accepted" is the service asking the
client to clean up after it.

⚠ **Atomicity is unchanged.** Each send is still one `Store/put`. This adds at most one small
second send per publish call — it does not make the publish partial in a new way; it removes a
partiality that already existed and was being paid for in duplicates.

## THE ALTERNATIVE, NAMED NOT HIDDEN

**Send one message's pairs per `Queue/send`** (`nsubs` pairs at a time). Then a partial message is
structurally impossible — each send lands whole or not at all. Cost: `M` sends per publish instead
of 1.

★ That is normally a bad trade, but **we have just measured that publish is drain-bound, not
round-trip-bound**, so more round trips may be close to free. If the top-up design does not close
the duplicate gap, measure this before inventing a third thing.

## OUT OF SCOPE — REJECTED

- **`setup` (10.2 s) and `stop` (11.7 s).** 22 s of a ~72 s run and never examined — but the
  builder's order is publish first, squeezed to its limit, then stop.
- **Caching `depth` to avoid the per-send `count-index`.** 517 µs × every send is real, but
  `SCORE-stop-fetching-rows-to-get-a-number` deleted those counters deliberately. Re-introducing
  one needs that stone read first, and it is not the drain-bound leader anyway.
- **Raising any `cap`.** The pressure system is the feature.
