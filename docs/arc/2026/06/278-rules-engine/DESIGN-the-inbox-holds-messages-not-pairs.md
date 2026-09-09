# DESIGN — the inbox holds messages, not pairs

**DESIGN ONLY. No BRIEF, and four rulings are owed before one can be written.** Arc-shaped: it changes
what the topic's first tier *means*.

## The builder's model, and it is already half-built

> *"topic accept → first-queue → [worker pool] fanout producers → second queue → [worker pool]
> subscriber writer → subscriber"*

The topology exists. `j` topic-workers already sit between an inbox queue and `m` subscriber queues.
**The ×nsubs expansion is simply on the wrong side of tier 1.**

```
today:  publisher ─10 msgs─▶ Topic ──×nsubs──▶ inbox (PAIRS, cap 64) ─▶ workers ROUTE ─▶ subs
                                     ↑ sns-fanout.wat:93

target: publisher ─10 msgs─▶ Topic ───────────▶ inbox (MESSAGES)      ─▶ workers EXPAND ─▶ subs
```

`sns-fanout.wat:93` builds `msgs × nsubs` bodies inside `Topic::publish`; `:113` sends them; `:476`–`:560`
is the worker doing `Queue/receive` → `split body "|"` → `Queue/send qpeer`. **The worker parses a
subscriber index out of a body it could have generated itself**, and it already holds `sub-addrs` and
already loops `0..nsubs`.

## ★★ What this fixes, each traceable to a measured or read fact

| | |
|---|---|
| **Admission decouples from `nsubs`** | Three publishers × max batches × `nsubs=5` is 150 pairs against 64 slots today. The publisher is refused for a number that is the topic's private business |
| **Tier 1 stops scaling with fanout width** | `nsubs=10000` turns 10 messages into 100,000 rows at publish time |
| **The `rem`/`need` top-up path DELETES** | `:120-128` repairs a message smeared across both tiers. With messages in tier 1, a message is either accepted-not-expanded or expanded — never both. **The state it repairs stops existing** |
| **`Accepted c` regains its plain meaning** | *c of your messages*, not *c derived from `pairs / nsubs`* |
| **The `:cap 64` question dissolves** | 64 *messages* is a different object from 64 *pairs*. No sweep needed |

## The two states, and why no third table is needed

```
tier 1 unacked  =  "accepted, not yet expanded"
tier 2 unacked  =  "expanded, not yet delivered to THIS sub"
tier 2 empty for a msg  =  "delivered to all subs"
```

**A queue with a visibility timeout and an ack IS a durable outstanding-work record.** At-least-once
falls out of the same primitive twice: a producer dying mid-expansion re-expands (duplicate notices,
which at-least-once permits — R69 is on exactly this point); a writer dying retries **one notice, to one
sub**. That granularity is why the `(msg, sub)` pair must be a row.

## The fork, worked and recommended

| | pairs materialised | ack-after-all-M |
|---|---|---|
| store ops | 2 × N × M | N × M |
| retry granularity | one stuck pair | **all M re-sent** on any single failure |
| duplicates on retry | none | some subs see it twice |

**Recommend pairs.** At `M=10000`, one backpressuring subscriber makes ack-after-all re-send ten
thousand notices. The extra writes buy granular retry, and the pair is the unit the retry targets.

## Why the risk is measurable now, which it was not this morning

```
drain    near-linear after the GSI fix   (4.44-4.58 against 4.0)
fill     linear   (2.015 / 2.021)
setup    CONSTANT ~12.4 s
inbox    flat on all four store ops — the sweep's own control
```

This is a **coherence** change, not a throughput gamble: any regression shows against a clean, verified
baseline. That was not true at any earlier point in this arc.

## ⛔ RULINGS OWED — a BRIEF cannot be written without these

1. **One arc or one stone?** It touches the topic's shape, the worker's job, and the circuit's wiring.
   Opening or aiming an arc is the builder's ruling, never a side effect.
2. **Does tier 2 become a third `Queue` instance**, inheriting visibility/redelivery/acks for free — or do
   workers write straight to the `m` sub-queues with a completion record? The fork above recommends the
   former; the ruling is yours.
3. **What is the inbox cap, in messages?** Or does it become a parameter as `sub-cap` already is
   (`run-with`, default 32, 8192 on the CLI)? Five hardcoded `:cap 64` sites exist —
   `circuit.wat:2178` on the measured path, four in demo entries.
4. **Where does the subscriber list live?** `nsubs` is on the topic's `:durable` today. If the worker
   expands, does it own `sub-addrs` alone, or does the topic keep a copy?

## ⚠ AND ONE GAP I HAVE NOT CLOSED

`sns-fanout.wat:548` holds a **second `bodies` fold**, inside the worker region. I have not read what it
does. **A BRIEF written without understanding it would be the Layer-2 failure
`COMPACTION-AMNESIA-RECOVERY.md` §4 records** — briefing work whose substrate assumption is false. It is
the first thing to establish once the rulings land.

## OUT OF SCOPE — REJECTED

- **The residual drain superlinearity** (4.44–4.58 vs 4.0). Smaller than the term already removed; its
  own question.
- **`scan-index`'s slope** — needs a server-side instrument, which the planner probe proved a caller-side
  one cannot substitute for.
- **The counter-carrier ~10 % drain tax** — known cause, known fix, independent.
- **`mem.wat`** — same DDB contract, no index structure, a different cause.
