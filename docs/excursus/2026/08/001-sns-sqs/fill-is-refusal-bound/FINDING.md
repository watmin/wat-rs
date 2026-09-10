# FINDING — `fill` is refusal-bound, and the queue parks receivers but not senders

Builder: *"let's do the fill p axis."* Done, and it is a dead end — for a reason that names a real
asymmetry.

## ⛔ `p` does not help. The publishers were never the bottleneck.

`p` (publisher count) had **never been exercised**: `:user::run-p*` exists and is called from nowhere,
and every other caller passes the literal `1`. Reached it via a temporary CLI slot (since reverted):

| `p` | `fill` | `setup` | `asleep` | inbox `refused` | `distinct`/`dup` |
|---|---|---|---|---|---|
| 1 | 2634 | 12484 | 230 | 191 | 8000 / 0 |
| 2 | 2728 | 12924 | 490 | 205 | 8000 / 0 |
| 4 | 2690 | 13813 | 721 | 206 | 8000 / 0 |

★ **`p > 1` works** — correctness holds at every value — and **`fill` is flat** while `asleep` *triples*.
More publishers simply take turns being refused, sleeping concurrently. `setup` rises ~440 ms per extra
publisher, the per-process spawn cost. **A dead end, and it cost one measurement to prove.**

★ And `refused` **equals** `full-retries` exactly at every `p` (191/191, 205/205, 206/206) — every refusal
produces exactly one retry.

## ⛔ AND MY EXPLANATION WAS WRONG BEFORE THE NEXT MEASUREMENT

I wrote *"one topic-worker fans everything — that's the serialization point."* **`twhandles` is a `foldl`
over `(range 0 j)`**: there are **j** topic-workers, three at the standard config. **Tenth correction this
session**, and it came from reading the fold I had already grepped past.

So the axis that matters is `j`, and it does relieve the wall — up to a point:

| `j` | workers | `fill` | `drain` | inbox `refused` | `asleep` |
|---|---|---|---|---|---|
| **1** | 4 | **6163** | 2894 | **696** | **1729** |
| **2** | 8 | **2794** | 2443 | 214 | 237 |
| 3 ← shipped | 12 | 2669 | 2440 | 188 | 225 |
| 6 | 24 | 2716 | 2644 | 182 | 221 |

★★ **`fill` saturates at j=2** — j=1 is 2.3× worse, and j=6 is no better than j=3. The shipped
configuration is already past the knee. **Adding workers is also a dead end.**

## ⭑⭑⭑ So what IS `fill`? Refused round-trips.

```
200 publish calls, but publish-attempts = 391   →  191 attempts are REFUSALS
2700 ms / 391 attempts = 6.9 ms per attempt     ≈  3–6 crossings at ~1.0–2.2 ms each
```

**Roughly half of `fill`'s round-trips are spent being told "no".** Each refusal costs a full crossing
plus a backoff sleep, and produces nothing.

## ⭑⭑⭑⭑ AND THE ASYMMETRY THAT CAUSES IT

`:queue::Waiter` is `[conn-id, queue, limit, visibility-ns, deadline-ns]` — a parked **receiver**. A search
for any send-side equivalent finds **nothing**. On a full queue the sender gets
`SendResponse::Accepted 0` (`sqs.wat:519`, `:774`) and is expected to back off and try again.

★★★ **The queue parks receivers and refuses senders.** It has the machinery for one side of that and not
the other — and it is the *same* machinery: a `conn-id` held until the condition is met, then a `Directed`
push. A parked sender would cost **one** crossing instead of two-plus-a-sleep.

⚠ **And this is a place where our model and the referent genuinely differ.** SQS has no per-queue depth
cap, so it has no refusal path to park — the situation cannot arise there. Our `:cap 64` creates it, and we
answer it with client-side backoff. **So "park the sender" is not an SQS-faithfulness argument; it is an
argument about the cap we chose to add.** That distinction matters, because the last two constants that
went wrong here went wrong by being tuned against the simulation rather than the referent.

## What this hands the builder

⛔ **Three rulings, and none of them is "raise the cap"** — batch limits and caps are contract here, and
`:cap 64` was deliberately frozen at `e0c552bf0` so a prediction could be tested against it.

1. **Park the sender.** Symmetric with the existing receive-waiter, removes ~191 wasted round-trips per
   run, and reaches `sqs.wat`. ★ It is also the *same shape* as the `asks` gap at `a4f2d7f7b` — a
   participant that must wait being made to poll instead of being woken.
2. **Accept it and document why.** The cap is deliberate backpressure; refusal-plus-backoff is a legitimate
   protocol for it, and 191 wasted crossings per 2000 messages may simply be the price of a bounded queue.
3. **Question the cap itself** — not raising it to fit a caller, but asking whether a 64-slot inbox is the
   right *model* when the referent has no cap at all.

★ Everything measured here says the same thing three ways: **`fill` is not slow because of who publishes
or who drains. It is slow because half its crossings are refusals**, and refusals exist because we added a
cap the referent does not have.
