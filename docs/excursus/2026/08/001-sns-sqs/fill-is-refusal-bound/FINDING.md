# FINDING — `fill` is refusal-bound, and the queue parks receivers but not senders

Builder: *"let's do the fill p axis."* Done, and it is a dead end — for a reason that names a real
asymmetry.

---

## ⛔⛔ CORRECTED 2026-09-10 — THE TITLE IS WRONG. `fill` IS NOT REFUSAL-BOUND.

**Read this before anything below.** Everything in this document about *where* the refusals come from
held up exactly. The **inference from it did not**, and it was promoted into `~/work/BREADCRUMB.md` as
an owed ruling (*"park the sender — ~191 wasted round-trips per run"*) before it was ever tested.

Builder: *"we created this 64 cap… i don't know if we can justify its existence at this stage."* So the
inbox cap became a swept parameter (`inbox-cap`, argv 12, default 64) instead of a chosen constant, and
it was swept — 5 values × 3 interleaved cycles, quiet box, through `scripts/capped.sh`:

| `inbox-cap` | inbox `refused` | `publish-attempts` | wall-ms | `fill` | `rt-total` | `rt-store` |
|---|---|---|---|---|---|---|
| 64 | 191 | 391 | 21215 | 2656 | 10850 | 6418 |
| 128 | 177 | 377 | 21325 | 2690 | 10865 | 6434 |
| 256 | 151 | 350 | 21475 | 2658 | 10957 | 6507 |
| 1024 | **0** | **200** | 21117 | 2624 | 11120 | 6666 |
| 8192 | **0** | **200** | 21074 | 2605 | 11101 | 6656 |

Means of 3. `distinct=8000 dup=0` on **all 15 runs** — correctness never in question.

**The cap is exactly, solely the cause of the refusals.** At ≥1024 `publish-attempts` falls to 200 — one
call per batch, zero retries. That half of the original reading is confirmed.

⛔ **And removing every refusal moves nothing.** −141 ms of 21 215 is **−0.7 %**, against a spread
*within cap 64 alone* of 484 ms (**2.3 %**). The effect is smaller than one arm's own noise.

★★★ **Worse: round-trips go UP.** `rt-total` +251, `rt-store` +238 — eliminating 191 refusal crossings
**added ~250 crossings elsewhere.** The mechanism is on the disk at `sqs.wat:470`: `send` runs its one
`count-index` and tests `take = 0` **before** the store put, so a refusal costs *one cheap count and no
write*. The cap was not wasting work — it was keeping the queue shallow, which makes every `count-index`
and `scan` cheaper. Raise it and the publishers run ahead, the inbox deepens, and the store pays more
than the refusals ever cost.

### What this rules

- **The bound stays, and it is justified** — `the-queue-is-bounded/SCORE.md` measured unbounded-batched
  sqlite at 1568/s with **2.6 s** e2e against cap 32 at 1383/s with **148 ms**. 18× the latency for 12 %
  of the throughput, and it is the builder's own lockstep/backpressure model.
- **The number 64 is not *unjustifiable* — it is *inconsequential*** at this configuration, which is a
  different and more honest verdict than either side of the original fork. It costs nothing and buys
  nothing measurable. It stays, now swept rather than chosen.
- ⛔ **"Park the sender" is struck as a perf item.** Its entire premise was ~191 wasted round-trips. The
  round-trips are real and they are not waste. If it is ever drawn it must be drawn as a *symmetry*
  argument (the queue parks receivers and refuses senders), never a throughput one — and §"the
  asymmetry" below should be read with that correction applied.
- **`fill-stale-max = 0` at every cap, 8192 included.** Decoupling publishing from fan-out did **not**
  make the fill poller sample silence, so `circuit.wat:1481`'s concern about sizing K is untouched by
  the cap — the cap costs the instrument nothing and gains it nothing.

★ **Three axes now agree, which is what makes this a conclusion rather than one more null:** `p` flat,
`j` saturating at 2, the cap inconsequential. `fill` is bound by none of publisher count, consumer-tier
count, or admission — it is bound by the **store work of moving 8000 messages through the inbox into the
sub queues.** That is `rt-store` at 59 % of the budget, and the only live lever on it is the patch banked
behind the nullary `Store::DeleteResponse::Success`.

⚠ **Scope of this correction:** one configuration, `2000 4 3 8192 true 1000`. It says the cap is
inconsequential *there*, on a quiet box, at 3 runs per arm. It does not say a depth cap can never matter.

---

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
