# BRIEF — is the queue saturated?

Add `handler-ns` to the queue: cumulative time spent inside op handlers. Sample it at the drain
boundaries and report `drain-busy-ms` per queue.

An **instrument**. It measures utilisation, `ρ = busy / wall`. **Do not act on what it shows.**

## Read in order

1. **`DESIGN.md`** — the poll-interval elimination (the poller is not the
   slope) and the single-server model this tests.
2. **`sqs.wat:745`, `:959`, `:998`** — `Invocation/start-ns` and `SelfInvocation/start-ns`, already
   used in this file. **The handler's entry timestamp is given to you.**
3. **`sqs.wat:983`** — the `stats` arm constructor; `handler-ns` joins `store-ns` as the **last**
   field on `StatsResponse::Ok`.
4. **`sqs.wat`** — the `:ephemeral` block, where `store-calls` / `store-ns` live. `handler-ns` sits
   beside them.
5. **`circuit.wat:1862`** (`sum-store-ns`) — **copy its shape** for `sum-handler-ns`, and the
   existing `drain-store-ms` boundary sampling for the delta.

## The work

**1. `handler-ns <- :wat::core::i64`** on `:ephemeral`, init `0`.

**2. Accumulate at every arm return.** `(epoch-nanos (now)) − (Invocation/start-ns ctx)` added to
`handler-ns` in each `queue::State` reconstruction that ends an op. The `-tick` arm uses
`SelfInvocation/start-ns`.

⚠ **This is the widest mechanical edit of the arc: 30 `queue::State` constructions.** Uniform, and
every miss is a compile error. `TakeAcc`'s named fields are what make it survivable — before them,
an edit of this shape killed two consecutive strikes.

**3. `handler-ns` last on `StatsResponse::Ok`**, after `store-ns`.

**4. `sum-handler-ns` + `drain-busy-ms`** in `circuit.wat`, sampled at the same two boundaries
`drain-store-ms` already uses, **divided by `m`**.

## Blast radius — EIGHT files, SIXTEEN sites

Grepped **before** this brief.

**Instrument (8 sites):** `sqs.wat` `:983` `:1436` `:1446` · `circuit.wat` `:1103` `:1820` `:1834`
`:1848` `:1862`

**A trailing `_` and NOTHING else (8 sites, 6 files):** `sns-fanout.wat` `:219` `:801` ·
`probe-three-waiters-wake` `:136` `:145` · `probe-the-server-manages-its-own-capacity` `:31` ·
`probe-stats-sees-an-expired-unacked` `:50` · `probe-depth-derived-from-the-index` `:70` ·
`probe-does-anyone-hold-a-service-name` `:62`

⚠ `Topic::StatsResponse` stays `[n ticks]`. The probes are floor members. No `wat/`.

## STOP triggers

- **STOP-1** — if a **seventeenth** `StatsResponse::Ok` site exists, **STOP and name it.**
- **STOP-2** — if an arm has a return path where `Invocation/start-ns` is unreachable, **STOP and
  name the arm.** Do not substitute a second `now` read at entry; the framework's timestamp is the
  definition of "handler start".
- **STOP-3** — if the no-args run differs in any **pre-existing** summary or phases field, **STOP.**
- **STOP-4** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-5** — **do not touch anything under `wat/`.**
- **STOP-6** — **do not act on the number**, and do not net out the poller's share. Report it.
- **STOP-7** — **leave the tree parsing.**
- **STOP-8** — if `drain-busy-ms` comes out **less than `drain-store-ms`**, **STOP and report both.**
  Busy time includes store time by construction; busy < store means the accumulation is wrong.

## What is deliberately NOT gated

⚠ `store-calls`, `store-ms` and `drain` all vary run to run — at n=1000, same code, zero retries:
**4959 / 4982 / 4996 / 5002**. Report them; a band would police noise. Five rows in this arc have
already made that mistake.
