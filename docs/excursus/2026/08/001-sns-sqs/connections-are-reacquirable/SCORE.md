# SCORE — connections are re-acquirable

**STRUCK.** Executor: grok, 2026-09-03. An Address is soul; a Peer is body.
Every dialing service keeps the address in `:durable`. `Lost` means reap,
re-dial, do not ack. Visibility expiry is the retry; S13's Seen absorbs
a landing that already happened. No substrate change.

```
Summary [ 356.931s] 5191 tests run: 5191 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T08-11-03Z/`

## The path is unexercised

Nothing today can break a pipe while the peer lives. That is the chaos
stone's injection. Rows 1–4 prove the recovery path is expressible, wired
and gated. **Proving it fires belongs to the next stone.** Do not invent
a fault to prove it early.

## Soul / body

`:durable` holds the Address; `:ephemeral` holds the Peer. Init is
`[record]` only. Start sites pass the address inside the Record.

- `queue::queue` — `store-addr`
- `demo::topic` — `inbox-addr`
- `demo::topic-worker` — `inbox-addr`, `sub-addrs`
- `fanout::worker` — `queue-addr`, `seen-addr`
- `fanout::held-worker` — `queue-addr`
- `fanout::seen` does not dial

## Lost is reconnect + do not ack

Twenty arms. Zero resolve to `assertion-failed!`. Closed/Stopped stay
fatal (dead peer, not a broken pipe — STOP-2). Redial failure is the
same: the peer is gone, and that is supervision, named so it is not
folded in.

The hard site is a `Lost` inside the topic-worker's foldl over
subscribers. Some took the batch; one did not. The failed bucket is not
acked. Redelivery of rows that already landed is S13 doing its job.

Queue `ack` Lost does not delete the row and does not decrement
in-flight. It replies `Ack Ok` so the worker does not hang; the work
itself is unacked. Visibility + Seen absorb.

Helpers (`face-start`, `recv-envelopes`) return nil / empty. They do
not hold a durable address.

## Gate

`tests/services/probe_connections_reacquirable.rs` drives
`probe-redial-from-durable-addr.wat`:

```
durable-addr=ok;before=yes;redial=yes;after=yes
```

Floor: `probe_connections_reacquirable::redial_from_durable_addr_works` PASS.

## Five runs, sqlite (the fixture), `2000×4×3`

All `total=8000; distinct=8000; dup=0`.

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 26.627 s | 726 ms | **300** |
| 2 | 25.515 s | 692 ms | **314** |
| 3 | 26.748 s | 781 ms | **299** |
| 4 | 26.642 s | 745 ms | **300** |
| 5 | 25.313 s | 699 ms | **316** |

Median **300/s** against 303–325/s. Overlap; reported, not chased.
Reconnect is on the failure path and costs nothing when nothing fails.

## What landed

- Address in `:durable` on every service that dials.
- Every `Lost` arm reaps, re-dials, stores the fresh Peer, does not ack.
- Redial probe gated. `wat/`, `src/` empty.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ the soul is kept | ✅ every dialing service holds Address in `:durable` |
| 2 | ★ a lost pipe is no longer fatal | ✅ 20/20 Lost arms: zero `assertion-failed!` |
| 3 | ★ no Lost arm acks | ✅ none acks the work; queue ack Lost does not delete |
| 4 | ★ the mechanism is gated | ✅ `durable-addr=ok;before=yes;redial=yes;after=yes` in the floor |
| 5 | nothing is lost at weight | ✅ `total=8000; distinct=8000; dup=0` five times |
| 6 | no counter, no backoff | ✅ none |
| 7 | no substrate change | ✅ `wat/`, `src/` empty |
| 8 | throughput | ✅ **299–316/s** against 303–325/s |
| 9 | floor | ✅ 5191/5191, `FLOOR=0`, `.floor/2026-09-03T08-11-03Z/` |
