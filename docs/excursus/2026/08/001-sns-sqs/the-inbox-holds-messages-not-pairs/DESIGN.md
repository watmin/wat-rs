# DESIGN — the inbox holds messages, not pairs

**excursus 001, stone-shaped.** Three of the four rulings are answered; the substrate gap is closed.

## The builder's model — and the topology already exists

> *"topic accept → first-queue → [worker pool] fanout producers → second queue → [worker pool]
> subscriber writer → subscriber"*

```
today:  publisher ─10 msgs─▶ Topic ──×nsubs──▶ inbox (PAIRS, cap 64) ─▶ worker ROUTES ─▶ subs
                                     ↑ :93

target: publisher ─10 msgs─▶ Topic ───────────▶ inbox (MESSAGES)      ─▶ worker EXPANDS ─▶ subs
```

## ★★★ The subscriber index makes a round trip purely to survive the inbox

The gap flagged in the first draft is closed, and it enlarges the finding. `sns-fanout.wat:548` is **not**
a second expansion — it is the worker **stripping the index back out**:

| step | site | what happens |
|---|---|---|
| publish | `:93` | fold `msgs × nsubs` into bodies framed **`"{i}|{msg}|{t0b}"`** |
| send | `:113` | `send-all` those pairs into the inbox |
| receive | `:476` | worker takes them back out |
| parse | `:503` | `split body "|"` — recover `i` |
| bucket | ~`:540` | group by `i` |
| re-frame | `:548` | **drop `i`**, emit `"{b}|{t3b}"` |
| deliver | `:560` | `Queue/send` to sub `i`'s queue |

**`i` is written into the payload, carried through a durable queue, parsed back out, used to bucket, then
discarded before delivery.** It exists only because tier 1 is pair-shaped. It is a routing tag smuggled
through a data field.

## So the restructure deletes four things, not two

1. the `×nsubs` expansion at publish (`:93`)
2. the `rem`/`need` top-up (`:120-128`) — the repair for a message smeared across both tiers
3. the `"{i}|"` tag and its `split` parse (`:503`)
4. the bucketing — the worker already holds `sub-addrs` and already sends per-sub; with messages in
   tier 1 it fans directly

★ And what it *keeps* is the part that was already right: the worker pool, the per-sub queues, and the
two-tier shape.

## ⛔ RULINGS — three answered, one settled by the disk

**1. Tier 2 is another `Queue` instance.** `defservice :queue::queue` count = **1**. There is one Queue
service; the inbox and all `m` subscriber queues are instances of it on thread or process loci. Tier 2
needs no new service — it inherits visibility, redelivery and acks **for free**.

**2. No cap on the internal tiers.** The store exposes **no capacity concept** — grep found only
per-query page limits and a frame-bytes cap, so *"max items in the store"* would be new machinery, not a
relocation. And the standing rule settles the direction: **clients never see internal capacity
constraints.** `:cap 64` refusing a publisher — in a unit (pairs) the publisher cannot see and did not
cause — is exactly that violation. The store's real limits bound it; the per-tier backlog metric
(`96a840db0`) is what makes removing the cap observable rather than reckless. **That metric had to land
first, and it did.**

**3. Subscribers stay `:durable`.** `inbox-addr <- (:wat::kernel::Address :- [...])` is **already** on the
topic's `:durable`. An `Address` is EDN-expressible; a `Peer` is not. So `sub-addrs` as a Vector of
Addresses is durable-safe by the same rule already in force one line above it.

**4. Arc or stone** — this is excursus 001 work, not a commission, so: a stone here.

## The two states, and why no third table

```
tier 1 unacked  =  "accepted, not yet expanded"
tier 2 unacked  =  "expanded, not yet delivered to THIS sub"
tier 2 empty for a msg  =  delivered to all subs
```

**A queue with a visibility timeout and an ack IS a durable outstanding-work record.** At-least-once falls
out of the same primitive twice — a producer dying mid-expansion re-expands (duplicate notices, which
at-least-once permits; R69 is on exactly this point), and a writer dying retries **one notice to one
sub**. That granularity is why the `(msg, sub)` pair must be a row in tier 2 — it is the unit retry
targets.

## The fork, recommended

| | pairs materialised | ack-after-all-M |
|---|---|---|
| store ops | 2 × N × M | N × M |
| retry granularity | one stuck pair | **all M re-sent** on any single failure |

**Pairs.** At `M=10000`, one backpressuring subscriber makes ack-after-all re-send ten thousand notices.

## Why the risk is measurable now, and was not this morning

```
drain    near-linear after the GSI fix   (4.44-4.58 against 4.0)
fill     linear   (2.015 / 2.021)
setup    CONSTANT ~12.4 s
inbox    flat on all four store ops — the sweep's own control
delete   1.998 → 1.107 after the reverse mapping landed
```

A **coherence** change against a verified baseline, not a throughput gamble.

## OUT OF SCOPE — REJECTED

- **The residual drain superlinearity** (4.44–4.58 vs 4.0) — smaller than the term already removed.
- **`scan-index`'s slope** — needs a server-side instrument; the planner probe proved a caller-side one
  cannot substitute.
- **The counter-carrier ~10 % drain tax** — known cause, known fix, independent.
- **`mem.wat`** — same DDB contract, no index structure, different cause.
