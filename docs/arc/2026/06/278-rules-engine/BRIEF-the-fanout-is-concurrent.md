# BRIEF — the fan-out is concurrent

The topic's `-deliver` awaits each subscriber before sending to the next, so one message costs the
**sum** of four independent chains instead of the **max**. Split the send from the recv: issue all
four sends, then collect all four replies. One arm, one file, no surface changes.

## Read in order

1. **`DESIGN-STONE-the-fanout-is-concurrent.md`** — the shape, and the one contract decision (a raw
   send bypasses the client-side `:max-request-bytes` check; the server-side guard still fires).
   Read it first; it also records why `select` is not needed.
2. **`wat-scripts/topic/sns-fanout.wat`**, `-deliver`'s subscriber `foldl` (the `_n` binding, just
   after the outbox rebuild). **This is the change, and it is the only change.**
3. **`wat/service.wat:2197-2302`** — the generated client: `op-variant-kw`, then `send-recv-form`
   (`send` then `recv`), then the re-wrap of `Reply` into `RecvOutcome`. **This is the
   specification you are mirroring by hand.** ⚠ There is no worked call site to copy — raw
   `kernel::send` exists in `wat-scripts/probes/arc-170/` but only to raw spawned peers, never to a
   `defservice` client. You are writing the first one.
4. **`wat/service.wat:1534`** — how `Op` variants are named (`service-op-str` + `::{variant-pascal}`),
   and `wat-scripts/queue/sqs.wat`'s `arm-tick` helper for a live spelling
   (`(:queue::queue::Op::-Tick)`).

## The sketch

Load-bearing: **every send is issued before any recv**, and every outcome is faced. Illustrative:
how the peers and replies are held.

```wat
;; 1. issue all four, face each SendOutcome, keep nothing but "did it go"
_sent (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  p <- (:wat::kernel::Peer :- [...])] -> :wat::core::i64
          (:wat::core::match (:wat::kernel::send p (:demo::Sub::Op::Deliver
                                                     (:demo::Sub::DeliverRequest :msg msg)))
            (:wat::kernel::SendOutcome::Sent    (:wat::i64::+ acc 1))
            (:wat::kernel::SendOutcome::Closed  acc)
            (:wat::kernel::SendOutcome::Stopped acc)
            ((:wat::kernel::SendOutcome::Lost _c) acc)))
        0 (:demo::topic::State/subs s))

;; 2. only now collect, in the same order — total is bounded by the slowest, not the sum
_n (:wat::core::foldl
     (:wat::core::fn [acc <- :wat::core::i64  p <- (:wat::kernel::Peer :- [...])] -> :wat::core::i64
       (:wat::core::match (:wat::kernel::recv p)
         ((:wat::kernel::RecvOutcome::Message _r) (:wat::i64::+ acc 1))
         (_ acc)))
     0 (:demo::topic::State/subs s))
```

Two separate folds over the same peer vector. If the two are fused, nothing is concurrent and every
row below still passes except rows 1 and 2 — which is why row 1 is a constructed proof.

## Blast radius

**`wat-scripts/topic/sns-fanout.wat` only**, and within it only `-deliver`'s fan-out. No surface
changes, no new fields, no new ops. `wat/`, `src/`, `sqs.wat`, `circuit.wat` untouched.

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Concurrency must not be observable in
   what arrives. A lost or duplicated message here is the finding, not a tuning problem.
2. **If facing the outcomes correctly requires a surface change — STOP and surface it.** The
   OUTCOME WALL is not to be worked around, and a `_`-swallowed `SendOutcome` is not "faced".
3. **If `wat/`, `src/`, `sqs.wat` or `circuit.wat` need to change — STOP.**
4. **If the drain does not drop — STOP and report it, do not chase it.** That would mean the four
   chains were not independent, which is a finding about the topology worth more than the stone.

## Shape to copy

`SCORE-the-circuit-goes-persistent.md` for reporting a per-delivery slope rather than a wall time.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` for a running `wat`/`cargo` before any timing.

Write `SCORE-the-fanout-is-concurrent.md` when done. It will be graded by re-running.
