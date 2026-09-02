# SCORE — the workers stop polling

**NOT STRUCK.** Adoption is withdrawn: a measured drain deadlock at circuit
weight, not a fixture-shape argument. Executor: grok, 2026-09-02.

The capability still exists. The circuit still polls. Finding (b) is still
false — this is a different failure, and it is about wake/drain at scale,
not `Admin::Stop`.

```
Summary [ 350.347s] 5183 tests run: 5183 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T06-31-24Z/`

## What landed on disk

`wat-scripts/fanout/circuit.wat` only:

- the false `never completes` comment is gone (row 3)
- `main` prints `queue-receive-calls` from `run*`'s second field
- `:wait-ns` on `:fanout::worker` is **still 0** — park was adopted, hung,
  and reverted so `wat circuit.wat` still completes
- `:fanout::held-worker` untouched
- `wat/` and `src/` empty. `sqs.wat` untouched (STOP-1)

Scratch evidence (not the stone):
`wat-scripts/scratch-pad/probe-three-waiters-wake.wat`.

## The scale matrix (named drivers, not a re-run of a hang)

| N | M | J | result | calls | distinct | drain ms | stop ms | wall s |
|---|---|---|---|---|---|---|---|---|
| 9 | 1 | 1 | drained | 9 | 9 | — | — | 1.4 |
| 9 | 1 | 2 | drained | 11 | 9 | — | — | 2.1 |
| 9 | 1 | 3 | drained | 11 | 9 | — | — | 2.1 |
| 12 | 2 | 2 | floor compute | — | 24 | — | — | 5.18 |
| 12 | 4 | 3 | ✅ | 60 | 48 | 311 | 1277 | 10.4 |
| 200 | 4 | 3 | ✅ | 812 | 800 | 5983 | 1261 | 16.2 |
| 500 | 4 | 3 | ✅ | 2012 | 2000 | 19150 | 1354 | 29.6 |
| 1000 | 2 | 3 | ✅ | 1234 | 2000 | 22089 | 1044 | 28.5 |
| 1000 | 4 | 2 | ⛔ hang | no tallies | — | — | — | killed 90s |
| 1000 | 4 | 3 | ⛔ hang | no tallies | — | — | — | killed 45s (snaps) / 90s |
| 2000 | 4 | 3 | ⛔ hang | no tallies | — | — | — | killed 180s |

Isolated 3-waiter wake at process locus
(`probe-three-waiters-wake.wat`): J=1,2,3 all `status=drained; p=0; f=0; got=9`
on the first wait check. The trap door named in EXPECTATIONS — "12 waiters
across 4 queues being woken" — is **not** "3 waiters never wake". Small N
wakes. The circuit hangs later.

`Admin::Stop` was not reached on the hanging runs. STOP-3 does not fire:
this is `wait-drained`, not Stop. `probe-parked-waiters-stop.wat` still
stands.

## The stuck tail (1000×4×3, drain-snap every 100 naps of 5 ms)

```
drain n=0    out=999  q0=p0f0; q1=p0f0; q2=p0f0; q3=p0f0;
drain n=100  out=903  all queues p0f0
drain n=400  out=652  all queues p0f0
drain n=500  out=583  q*=p0f1
drain n=1000 out=344  mixed leftover
drain n=1100 out=1    q0=p1f1; q1=p1f1; q2=p1f1; q3=p1f1;
drain n=2000 out=1    q0=p1f1; q1=p1f1; q2=p1f1; q3=p1f1;
```

Workers keep the queues empty for most of the run. Then **one topic outbox
message and one pending + one in-flight per queue freeze**. Stats still
answer (the queue actors are alive). `pending=1` is never taken. `in-flight=1`
is never acked. `outbox=1` never delivers. Same shape on all four queues.

That is why 180 s at N=2000 is a hang, not a slow drain: 500×4×3 drained
in 19 s, linear extrapolation for 2000 is ~76 s, and the snap is frozen
for thousands of drain iterations.

The drain-snap helper was diagnostic only and has been reverted. The
numbers above are from `.grok` terminal
`call-905ae219-4191-4e9a-8e5f-f2f69a9f8b1e-105`.

## Why SCORE-queue-long-poll is still spent

That SCORE withdrew adoption because the *old* fixture published everything
before consumers started, so `wait-ns 0` returning empty meant "not filled
yet". The sane circuit overlaps producers and consumers. Small-N park
**does** help that shape (`12×4×3` is 60 calls for 48 outcomes; `500×4×3`
is 2012 calls for 2000). The withdrawal's reasoning is not being re-derived
and is not being quietly reversed. A new deadlock at M=4 N≥1000 is why
adoption still does not land.

## STOP triggers, as hit

1. **Substrate / `sqs.wat`.** Not patched. The hang is the queue's park/wake
   path at process locus under fan-out load. Blast radius was
   `circuit.wat` only; fixing Directed-wake or the stuck tail would be a
   queue stone, not this one.
2. **`distinct=8000`.** Never observed at N=2000 — drain never returned.
   Sizes that completed were lossless (`dup=0`, `empty=1`).
3. **`Admin::Stop` hang.** Not this. Drain hung first. Worker count on the
   snapped hang: **J=3, M=4, wait-ns=250000000**.
4. **Ack batching / drain poller.** Not used to make a row pass.

## Productive-path TCO (constraint, not the hang)

A file-level `:fanout::worker-pump` `--check`s and then **zombies all 12
process workers** — children do not see sibling defns (same reason sqs
`take` is closed in `:init`). The first hung run this strike was that
(workers `[wat] <defunct>`). Inlined `-tick` with a 1 ms re-arm on both
empty and productive paths; workers stayed alive; the scale deadlock
remained. Canonical TCO cannot live in a sibling for these workers.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ empty polls gone (`calls < 20,000`) | ⛔ reverted. Parked calls at small N beat the floor; N=2000 never printed |
| 2 | ★ `total=8000; distinct=8000; dup=0` | ⛔ drain hung at N=2000 before tallies |
| 3 | ★ `never completes` gone | ✅ zero hits |
| 4 | ★ worker receive `wait-ns` non-zero | ⛔ reverted to 0 after the hang |
| 5 | shutdown ~ one `wait-ns` + teardown | ✅ on sizes that completed: stop=1277–1354 ms at 250 ms park, not 12×250 ms |
| 6 | `Admin::Stop` still works | ✅ where drain finished; hanging runs never reached Stop |
| 7 | no substrate change | ✅ `git diff wat/ src/` empty |
| 8 | held-worker untouched | ✅ still `:wait-ns 50000000` |
| 9 | phase split | reported for N≤500; N=2000 hung in drain |
| 10 | wall time | N=2000 hung. 500×4×3 wall **29.6 s**. Row 10 is not a target |
| 11 | floor | ✅ 5183/5183, `FLOOR=0`, `.floor/2026-09-02T06-31-24Z/` |

## Floor

```
Summary [ 350.347s] 5183 tests run: 5183 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T06-31-24Z/`. `every_wat_scripts_file_loads` included the wake probe.

## Next

The stuck tail is the stone worth more than this one: **why a parked
process-locus receive that `send` has already `take`n never completes,
leaving `in-flight=1` and a `pending=1` that no waiter will take, with
the topic's last outbox message blocked behind it.** It is the untested
half EXPECTATIONS named, observed at M=4 N≥1000, not at J=3 on one queue.

Do not tune `wait-ns` to hide receive-calls. Do not re-run the 2000×4×3
hang as a flake. Do not put finding (b) back in the comment.
