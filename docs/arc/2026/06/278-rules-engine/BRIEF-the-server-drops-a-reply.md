# BRIEF — the server drops a reply

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`e57615294`, tree clean. Read `DESIGN-the-server-drops-a-reply.md` first.

## ⛔ YOUR FIRST ACT — the decision I could not settle

**Is the service's `state` in scope at the five `send-keep-serving?` call sites** (`service.wat`
`1672 1706 1776 1799 1838`-ish; confirm current numbers)?

- **Yes** → the rate and seed live in `:durable`, threaded through the loop's existing state.
- **No** → that option is dead. Report it; do **not** improvise option (b) or fall back to
  `:wat::rand::int`, which is **not Deterministic** and destroys replay.

I looked and could not establish it. Shipping the absence rather than a guess is what R1 v1 failed to
do, and it cost a strike.

## THE WORK

A rate-gated, seeded drop inside `send-keep-serving?`: draw before the send; on a hit **do not send**
and **still return `true`**. A drop is not `Stopped` — the loop keeps serving and the caller learns
nothing. T1's deadline turns that into a timeout → discard → redial → **retry**, and the retry is a
second claim on the same seq → `Dup` → **`seen-dups` moves.**

**Build both placements** — the drop *before* and *after* the ledger write — because the placement is
the fault and the pair is what proves it.

## ROOMS

1. **`wat/service.wat:3108-3114`** — the seam. The drop goes inside it.
2. **`wat-scripts/fanout/circuit.wat`** — the worker's `Seen/claim` with T1's deadline, and its
   `ServiceEvent` arms. **This is the call the drop targets, and the only one with a deadline.**
3. **`wat-scripts/fanout/circuit.wat`** — `:fanout::seen`'s claim arm and its `firsts`/`dups`
   counters. The ledger write is the line the placement is relative to.
4. **`wat-scripts/scratch-pad/probe-rand-is-usable-from-wat.wat`** — the seeded-draw idiom.
   `:wat::rand::int-from state lo hi` → `(Tuple new-state draw)`, both bounds `[lo, hi)`.
5. **`SCORE-chaos-is-a-rate.md`** — 3c's shape: rate 0 arms nothing, seed on the Record, replay by
   seed. Copy that discipline.

## STOP TRIGGERS

1. **`state` is not in scope at the send sites.** Report it. Do not improvise. STOP.
2. **You are about to use `:wat::rand::int`** (the ambient one). Not Deterministic — no replay. STOP.
3. **The drop returns `false`.** That stops the world; the fault is `true`. STOP.
4. **You are about to drop a call that has no client deadline.** Only `Seen/claim` has one. A drop
   elsewhere is a hang. STOP.
5. **The default rate is anything but 0**, or rate 0 still draws. STOP.
6. **`distinct` drops below 8000.** That is **loss** — report it with the mechanism; do not repair it
   here. STOP.
7. **You are about to add selectable eviction.** S39. STOP.

## HOW TO WORK

Foreground everything. Floor **after** the edit; Summary line, never a piped exit code. On an
unintended red: **do NOT re-run**, capture whole, name the arm.

⚠ `wat/service.wat` is stdlib. Stone C established that a change with no new `fix.wat` verb needs no
stash — edit, rebuild, done.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form, arc-109 NOTE.

Leave your work uncommitted. Prior comparable: `SCORE-a-client-has-a-deadline.md`.

## REPORT

- **your first act's answer**, before anything else
- **`seen-dups`, five runs, rate on.** Any non-zero is the result
- **both placements, same rate and seed**, side by side — before-write vs after-write
- **`distinct`** — and if below 8000, the mechanism, not a repair
- rate 0: floor `5214/5214`, `seen-dups=0`
- two runs at one seed: same `seen-dups`
- every STOP that fired
- **the honest deltas.** Ten of my counts have missed this campaign and the last four stones each
  found a citation of mine stale. What you find is the fact.
