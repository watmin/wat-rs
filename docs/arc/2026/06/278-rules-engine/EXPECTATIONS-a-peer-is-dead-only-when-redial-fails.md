# EXPECTATIONS — a peer is dead only when redial fails

Written **before** the strike. Re-run by me on a quiet box. The result cannot move these.

## THE PREMISE — already green, 3/3, committed

```
probe-closed-is-recoverable.wat
  a-small=ok ; a-big=lost ; a-again=closed ; a-REDIAL=ok ; b-still=ok
  verdict = CLOSED-IS-RECOVERABLE
```

Circuit baseline, my five runs post-Stone-D: `dup=0` all five, publish `26750–27177 ms`.

| # | what | command | expected |
|---|---|---|---|
| 1 | ★★ **the system survives a real sever** | sever a live worker's connection mid-run | the worker **reconnects** and the run finishes `total=8000; distinct=8000; dup=0`. ⛔ This is the stone. A unit probe proves redial works; only this proves the system survives it |
| 2 | ★★ **the wall still fires** | redial pointed at a genuinely dead service | `"peer is dead, not a broken pipe"` still raises. ⛔ **A wall that never fires is a deleted wall** — if `Closed` can no longer kill anything, the recovery is unfalsifiable |
| 3 | ★ `Closed` arms converted | `grep -A2 'RecvOutcome::Closed' <3 files> \| grep -c 'assertion-failed!'` | the remaining fatals are only the **redial-failed** ones. My hypothesis: **17 arms move** |
| 4 | ★ `Stopped` untouched | `git diff` | **zero** `Stopped` arms changed. It means shutdown, not fault (S26) |
| 5 | `wat/service.wat` untouched | `git diff --name-only` | absent. Reactor layer, stdlib, S27 |
| 6 | no retry limit invented | `git diff` | no backoff, no attempt counter. Chaos informs that policy |
| 7 | the premise probe still green | `probe-closed-is-recoverable.wat` | unchanged, 3/3 |
| 8 | the invariant | circuit, **five runs** | `dup=0` on **all five** |
| 9 | throughput | same five runs | publish **25.5–27.4 s**, and **compare only against my own numbers** — cross-executor comparison is invalid (S25) |
| 10 | the floor | `scripts/floor.sh`, **Summary line** | `5213/5213` |

## ⛔ ROW 1 AND ROW 2 ARE ONE PAIR AND NEITHER COUNTS ALONE

Row 1 alone can be passed by deleting every assertion. Row 2 alone can be passed by changing
nothing. **Together they say: the fault is survived, and death is still detectable.**

This is the same shape as `SCORE-the-sane-circuit.md`'s row 2, which proved the in-flight term
load-bearing by *removing* it and demanding a failure. A recovery path that cannot be made to fail
has not been demonstrated — it has been assumed.

## RUNTIME PREDICTION

**60–90 minutes.** Seventeen arms, each mechanically similar but each needing its own address in
scope — that per-site variation is the work, and it is why a codemod is the wrong tool. If an arm
has no address reachable, that is STOP-1 and a genuine finding about the state shape.

## TRAP-DOOR RISKS

1. **`:max-frame-bytes` ≠ `:max-request-bytes`.** The first tears the connection down (what you
   want); the second replies `RequestTooLarge` and keeps it. `probe-frame-cap-severs-one-conn.wat`
   uses 256 / 524288 and is the working reference.
2. **The first touch after a sever is `Lost`, not `Closed`.** A test that only produces `Lost` has
   not exercised this stone at all. You must touch the stale handle **twice**.
3. **Redialing gives a NEW peer.** The fresh handle must be threaded back into state, or the next
   call uses the dead one and you get an infinite `Closed` loop that looks like a hang.
4. **`sqs.wat`'s 5 arms are all fatal and it holds a `store` peer** — its address is `store-addr` in
   `:durable`, which is exactly why the durable/ephemeral split exists.
5. **S24 is live.** `refused_subscriber_is_retried_not_dropped` can fail loudly with
   `after-drain=got` under load. That is the known timing-coupled assertion, not your regression.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 without a sever actually induced — "the circuit still passes" proves nothing if nothing broke.
- Row 2 not run, or reported as "the assertion is still in the source". It must **fire**.
- Row 3 reported as my number rather than the one found.
- A `Stopped` arm changed, or `wat/service.wat` touched.
- A redial loop with no fresh peer threaded back — that converts a death into a hang, which is
  strictly worse and is the failure mode this whole arc has been removing.
