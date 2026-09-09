# SCORE — transient means try again

**STRUCK.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
`wat-scripts/queue/sqs.wat` + `wat-scripts/scratch-pad/probe-transient-means-try-again.wat`.
STOP-6 discharged by `105ecf16d` / `d0a160a9f`; this stone is the retry.

```
Summary [ 384.258s] 5216 tests run: 5216 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T03-55-50Z/`

An honest `:Transient` (nothing committed) is retried inside the queue. Everything
else still dies, and now says which one it was.

## THE RETRY

Send and ack, circuit `a1`/`a2`/`a3` shape, budget 3, 1 ms wait **only** on
`:Transient` (happy path never arms a timer).

```
:Success           → proceed
:Transient         → nap, retry the SAME request; exhausted →
                     "queue.send: store put transient, exhausted after 3"
                     "queue.ack: store delete transient, exhausted after 3"
:Constraint        → "queue.send: store put Constraint"  (ack: delete Constraint)
:Fatal             → "… Fatal"
:RequestTooLarge   → "… RequestTooLarge"
:RequestMalformed  → "… RequestMalformed"
```

No `_` remains on a store response at the three sites, or on take's re-put /
scan-index, or on depth/total count-index.

Take `scan-index` `:Transient` is **named**, not retried — inlining a1/a2/a3 around
the Success body would restructure `take`. The measured death was `put`, and that
path retries. Count-index `:Transient` is likewise named (send's cap gate); the
honest wrapper only fails `put`/`delete`, so row 1 never hits it.

## THE HONEST WRAPPER

`:bs::busy-store` returns `:Transient` **without applying**, decrements a left
counter, then forwards. That is SQLITE_BUSY, not the lying k-of-n wrapper.

```
RETRY=send=Ok;total=10;distinct=10
EXHAUST=send=Lost:disconnected
  stderr: queue.send: store put transient, exhausted after 3
CONSTRAINT=send=Lost:disconnected
  stderr: queue.send: store put Constraint
FATAL=send=Lost:disconnected
  stderr: queue.send: store put Fatal
ACK=got=10;ack=Ok
```

The client of a dead queue sees `Lost:disconnected`. The assertion **message**
is the named string. No `Lost` on the retrying cell.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ Transient no longer kills the queue | ✅ `send=Ok;total=10;distinct=10` |
| 2 | ★★ exhausted budget names itself | ✅ `queue.send: store put transient, exhausted after 3` |
| 3 | ★★ Constraint / Fatal die by name | ✅ `store put Constraint` / `store put Fatal` |
| 4 | ⛔ no `_` on a store response | ✅ three sites + take re-put/scan-index + depth/total count-index |
| 5 | ⛔ floor | ✅ `Summary [ 384.258s] 5216 tests run: 5216 passed (6 slow), 22 skipped` (5216 is the txn-close gate; EXPECTATIONS said 5215) |
| 6 | ⛔ publish+drain ×5 | median **24494 ms** vs before **23672** — see below |
| 7 | blast radius | ✅ `sqs.wat` + scratch-pad probe |

## ROW 6

`ps` before: grok 9.8 %, claude 5.9 %, else < 1 %.

```
publish        24290  24417  24246  24306  24269
drain            207    189    232    188    210
publish+drain  24497  24606  24478  24494  24479     median 24494
total=8000; distinct=8000; dup=0  ×5
```

Before median **23672**. +822 ms (~3.5 %) on a box with grok at 10 %. No timer on
the happy path. Delivery exact.

## STOP-1 / STOP-2 / STOP-3 / STOP-4 / STOP-5 / STOP-6

- **STOP-1** did not fire. Retry lives in the send/ack arms as local `once` fns.
- **STOP-2** did not fire. Only `:Transient` retries.
- **STOP-3** did not fire. Honest wrapper applies nothing; sqlite store is
  all-or-nothing + `close-then-err` (proven).
- **STOP-4** did not fire. No `SendResponse`/`AckResponse` variant added.
- **STOP-5** did not fire. `sqs.wat` + scratch-pad only.
- **STOP-6** discharged before this strike (`put2` reports its own cause).

## NOT TOUCHED

`wat/`. `src/`. Per-entry outcomes. The lying k-of-n wrapper. `busy_timeout`.

---

Tree uncommitted. Do not commit unless asked.
