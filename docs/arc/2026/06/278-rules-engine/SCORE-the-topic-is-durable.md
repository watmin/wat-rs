# SCORE — the topic is durable

**STRUCK.** Executor: grok, 2026-09-03. Topic = publish surface + ONE queue-service
+ J internal workers. `Ok` is a store write of N rows, one per subscription.
Workers `Queue/send` to subscriber queues and ack only on Ok. Full is "do not
ack"; visibility expiry is the retry. The old `:ephemeral` outbox, `-deliver`,
`deliver-armed?`, `arm-deliver`, and the topic's own cap/`Full` are gone.

```
Summary [ 350.538s] 5188 tests run: 5188 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T04-31-03Z/`

First floor was red — captured, not re-run. Arm named below. Fix: circuit
topic-worker vis 5s so a loaded floor cannot expire an in-flight send+ack.
Row 3 keeps 200ms. Second floor is the one quoted.

## Rows 3 and 4 — the ones that cannot be faked

A subscriber that actually refuses (cap 1, filled with a dummy):

```
inflight=yes;after-drain=none;after-expiry=got
healthy=got;stalled=held;blocked=no
```

The worker holds the row unacked while the subscriber is full; after the dummy
is gone the real message is still invisible; after vis expiry it arrives. No
retry counter in the diff. A healthy sibling receives immediately; publish
does not block.

## Five runs, mem-store, `2000×4×3`

All `total=8000; distinct=8000; dup=0`. Inbox cap 64 (16 messages at M=4, the
old topic cap 16 in row units). Subscriber queues cap 32. Topic-worker vis 5s
on the circuit (200ms on the refusal probe).

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 51.020 s | **695 ms** | **157** |
| 2 | 51.403 s | **684 ms** | **156** |
| 3 | 52.189 s | **708 ms** | **153** |
| 4 | 51.649 s | **700 ms** | **155** |
| 5 | 51.817 s | **697 ms** | **154** |

Median **155/s**, e2e max **684–708 ms**. Against 921–954/s and 152–197 ms:
a durable write of N rows on the publish path, plus inbox-cap backpressure
(publish is 50.7–51.9 s of Full-retry against cap 64). **Reported, not chased.**
Did not batch the publish write.

`t3→t4 >1 s` count **0** every run (max 10–14 ms). The e2e mass sits in
250–1000 ms because `t0` is stamped on the first accept attempt and the inbox
write waits out Full; that is the backpressure, not a park-timeout regression.

**`dup=0` still holds and is not evidence of exactly-once.** At-least-once
permits duplicates; reliable IPC just never generates one. When item 3 injects
loss this invariant must change.

## What landed

- `demo::topic` durable field is `nsubs`. One scalar Queue peer (the inbox).
  `publish` does ONE `Queue/send` of N bodies `{idx}|{msg}`, then replies Ok
  or Full from the inbox. Stats are inbox `pending+in-flight` and inbox ticks.
- `demo::topic-worker` is the fanout-worker shape: park on the inbox, batch
  of 10, `Queue/send` to subscriber `q{i}`, ack only on Ok, Full leaves the
  row in-flight. No counter, no backoff, no per-row attempt state.
- Circuit: one mem-store inbox (cap 64) + J topic-workers; adapters deleted.
  Subscriber queues unchanged. Drain still waits inbox depth 0.
- Standalone 3 3: `wat-scripts/topic/run.wat` (the entry; `set-redef!` cannot
  live in a file the circuit `load-file!`s). Prints `"3 3"`.
- sqlite probe kept in lockstep; its inbox is still mem-store (STOP-5).

`wat/`, `src/` empty.

## Floor red on the way in

```
Summary [ 358.126s] 5188 tests run: 5187 passed (3 slow), 1 failed, 15 skipped
```

`.floor/2026-09-03T04-22-46Z/`. Arm:

```
wat::services probe_ex001_fanout::fanout_compute_is_complete_and_lossless
probe_ex001_fanout.rs:53
  left: "26"
  right: "24"
```

`total` is N×M. 12×2=24; 26 is two extra outcomes. Circuit topic-worker vis
was 200ms (the refusal-probe value). Under a loaded floor, send+ack of one
envelope exceeded 200ms, vis expired, a second worker re-sent. That is the
at-least-once path firing on a too-short window, not a flake. Circuit vis
is 5s so the happy-path ack cannot lose the race; the 200ms window stays on
`:user::refused-is-retried`.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ Ok means durable | ✅ `pending=1;durable=yes` — publish, then inbox depth, no workers |
| 2 | ★ unit is per-subscription | ✅ `rows=3;unit=per-sub` |
| 3 | ★ refused subscriber retried | ✅ `inflight=yes;after-drain=none;after-expiry=got`. No retry counter |
| 4 | ★ stalled does not stall others | ✅ `healthy=got;blocked=no` |
| 5 | ★ old outbox gone | ✅ `grep -n 'outbox\\|deliver-armed?\\|arm-deliver\\|-deliver' wat-scripts/topic/sns-fanout.wat` — zero hits |
| 6 | ★ nothing lost | ✅ `dup=0` every run (and not evidence of exactly-once) |
| 7 | ordinary queue-service | ✅ `queue::queue/start` + mem-store inbox |
| 8 | throughput, reported | ✅ **153–157/s** against 921–954/s |
| 9 | e2e histogram | ✅ max **684–708 ms** against 152–197 ms |
| 10 | no substrate | ✅ `wat/`, `src/` empty |
| 11 | floor | ✅ 5188/5188, `FLOOR=0`, `.floor/2026-09-03T04-31-03Z/` |
