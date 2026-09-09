# BRIEF — `store-ns`

Add `store-ns` as a **named field** on `TakeAcc` and on the queue's `:ephemeral` state, accumulate a
clock pair at the eight `Store/*` sites, and report `store-ms=` on the phases line.

An **instrument**. It resolves a fork. **Do not act on what it shows.**

**This is the edit that failed twice.** It parses now because `TakeAcc` exists — `store-ns` is a
named field, not a fourth Tuple slot.

## Read in order

1. **`sqs.wat:95`** — `:queue::TakeAcc`, three fields, inside `:messages`. **`ns` joins them.**
2. **`sqs.wat:456`, `:961`, `:1242`** — the three waiter folds, now `(Tuple TakeAcc box)`. `ns`
   accumulates through `TakeAcc/ns` exactly as `calls` does through `TakeAcc/calls`.
3. **The eight `Store/*` sites** — `:177` (scan-index), `:213` (put), `:269` (count-index), `:303`
   (count-index), `:413` (put), `:773` (delete), `:1075` (put), `:1124` (delete). Each gains a clock
   pair; each already increments `store-calls`.
4. **`sqs.wat:943`** — the `stats` arm constructor. `store-ns` joins `store-calls` as the **last**
   field.
5. **`circuit.wat:1848`** — `sum-store-calls`. **Copy its shape** for `sum-store-ns`.
6. **`probe-what-a-clock-read-costs.wat`** — the overhead, priced at ~322 ns/read.

## Blast radius — EIGHT files, FIFTEEN sites

Grepped **before** this brief: `grep -rn 'queue::Queue::StatsResponse::Ok' --include=*.wat .`

**Instrument (7 sites):** `sqs.wat` `:943` `:1375` `:1385` · `circuit.wat` `:1103` `:1820` `:1834`
`:1848`

**A trailing `_` and NOTHING else (8 sites, 6 files):**

| file | lines |
|---|---|
| `wat-scripts/topic/sns-fanout.wat` | `:219`, `:801` |
| `wat-scripts/scratch-pad/probe-three-waiters-wake.wat` | `:136`, `:145` |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | `:31` |
| `wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat` | `:50` |
| `wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` | `:70` |
| `wat-scripts/scratch-pad/probe-does-anyone-hold-a-service-name.wat` | `:62` |

⚠ `Topic::StatsResponse` stays `[n ticks]`. ⚠ The probes are floor members. No `wat/`.

## The work

**1. `ns <- :wat::core::i64` on `TakeAcc`** (`sqs.wat:95`) — a fourth named field.

**2. `store-ns` on `:ephemeral`** beside `store-calls`, init `0`, carried through every `State`
reconstruction that already carries `store-calls`.

**3. A clock pair at each of the eight sites** — `(epoch-nanos (now))` before and after the
`Store/*` call, difference added. Inside the folds it lands on `TakeAcc/ns`; elsewhere directly on
`store-ns`.

**4. `store-ns` last on `StatsResponse::Ok`**, after `store-calls`.

**5. `sum-store-ns` in `circuit.wat`**, shaped like `sum-store-calls`, reported as `store-ms=`
(**nanos on the wire, divide by 1 000 000 at the format**).

## STOP triggers

- **STOP-1** — if a **sixteenth** `StatsResponse::Ok` site exists, **STOP and name it.**
- **STOP-2** — if the no-args run differs in any **pre-existing** summary or phases field, **STOP.**
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not touch anything under `wat/`.** Timing inside the store is the next question,
  and only if the fork points there.
- **STOP-5** — **do not act on the number.** Report it. The fork decides the next stone.
- **STOP-6** — **leave the tree parsing.** Restore rather than hand back a non-loading tree.
- **STOP-7** — if `store-ns` cannot ride `TakeAcc` as a named field, **STOP and quote the checker.**
  That is the whole premise of this attempt.

## What is deliberately NOT gated

⚠ `store-calls`/pair varies run to run — measured this session at n=1000, same code, zero retries:
**4959 / 4982 / 4996 / 5002**. Report it; a band on it would be policing noise. That mistake has
been made four times in this arc and will not be made again here.
