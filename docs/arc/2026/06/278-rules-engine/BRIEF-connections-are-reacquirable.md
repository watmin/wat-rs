# BRIEF — connections are re-acquirable

Services throw away the `Address` they dialed, so a broken pipe cannot be recovered — and all 20
`RecvOutcome::Lost` arms are `assertion-failed!`. Move the address into `:durable`, and make `Lost`
mean *reap, re-dial, do not ack* instead of *die*.

## Read in order

1. **`DESIGN-STONE-connections-are-reacquirable.md`** — the contract decision (recovery is reconnect
   plus not-acking; the retry is visibility expiry) and the scope boundary (broken pipe, **not** dead
   peer).
2. **`wat-scripts/scratch-pad/probe-redial-from-durable-addr.wat`** — **already run, already green**:
   `durable-addr=ok;before=yes;redial=yes;after=yes`. This is your worked reference for all three
   unknowns, including that the handle-lifetime wall permits an in-arm dial. **Do not re-derive it.**
3. **`tests/services/probe_queue_visibility.rs`** — the retry path this leans on, proven and gated.
4. **The 20 `Lost` arms**: `grep -n 'RecvOutcome::Lost' wat-scripts/fanout/circuit.wat
   wat-scripts/topic/sns-fanout.wat wat-scripts/queue/sqs.wat`. Every one is the work.

## The sketch

Load-bearing: the address is `:durable`, and the failure path does **not** ack. Illustrative: naming.

```wat
:durable   [… , target <- (:wat::kernel::Address :- [Op Reply])]
:ephemeral [peer <- (:wat::kernel::Peer :- [Op Reply])]

;; a Lost arm becomes:
((:wat::kernel::RecvOutcome::Lost _cause)
  (:wat::core::let
    [fresh (…connect (…/target (…/durable s))…)
     s'    (…State … :peer fresh)]
    ;; return s' WITHOUT acking — visibility expiry brings the work back
    (:wat::service::Outcome::Continue s' … )))
```

## Blast radius

`wat-scripts/fanout/circuit.wat`, `wat-scripts/topic/sns-fanout.wat`,
`wat-scripts/queue/sqs.wat` — every service that dials. **`wat/` and `src/` untouched**; the probe
proves they need not change.

## STOP triggers

1. **If a `Lost` arm ends up acking — STOP.** Acking an unknown-outcome request is exactly how a
   message is lost forever, and it is the one thing this stone must not do.
2. **If reconnect needs a NEW address (the peer is gone, not the pipe) — STOP and surface it.** That
   is supervision and rediscovery, explicitly a different stone.
3. **If you find yourself adding a retry counter, backoff, or attempt state — STOP.** Same rejection
   as S13: visibility expiry is the mechanism.
4. **If `wat/` or `src/` need to change — STOP.** The probe says they do not.
5. **If `total=8000; distinct=8000; dup=0` breaks — STOP.**

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — capture,
name the arm, do not re-run. Check `ps` before timing. **Five runs, report the spread**, and report
**deliveries/s**, not wall.

Write `SCORE-connections-are-reacquirable.md` when done. Graded by re-running.
