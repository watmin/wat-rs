# BRIEF — a peer is dead only when redial fails

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`.
Read `DESIGN-a-peer-is-dead-only-when-redial-fails.md` first.

## THE WORK

A severed connection reports `Lost` on its first touch and **`Closed` on every touch after**. The
corpus treats that `Closed` as a dead peer and asserts — 17 times across three files — while the
`Lost` arm one step earlier redials and carries on. Same fault, opposite disposition, and nobody
chose it: nothing in the tree had ever severed a connection, so only the first-touch path was
designed. Give every fatal `Closed` arm the shape its `Lost` sibling already has: **redial the
address; assert only if the redial fails.** `Stopped` does not change.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-closed-is-recoverable.wat`** — **run it first.** It is the
   premise of this whole stone and it is already green, 3/3:
   `a-small=ok; a-big=lost; a-again=closed; a-REDIAL=ok; b-still=ok`. After the sever, and after the
   stale handle returned `Closed`, re-dialing the same address works. You are not discovering
   whether recovery is possible.
2. **`wat-scripts/scratch-pad/probe-frame-cap-severs-one-conn.wat`** — **how to sever a connection
   on demand**: a `:max-frame-bytes` of 256 and an oversized body. This is your tool for proof row 1.
   Note `:max-frame-bytes` (deployment) is separate from `:max-request-bytes` (contract, which
   replies `RequestTooLarge` instead of tearing down).
3. **`wat-scripts/fanout/circuit.wat:224-232`** — ⭐ **THE EXEMPLAR.** The `Lost` arm that recovers:
   redial `seen-addr`, and assert only on `ConnectOutcome` failure with *"peer is dead, not a broken
   pipe"*. Every arm you change becomes this shape against its own address.
4. **`wat-scripts/fanout/circuit.wat:221-223`** — the `Closed` arm that dies, four lines above its
   own exemplar.
5. **`wat-scripts/topic/sns-fanout.wat:112,143,317,334,364`** — the topic's existing redial sites,
   for the address-in-scope patterns you will need.
6. **`wat-scripts/queue/sqs.wat`** — 5 fatal `Closed` arms, all 5 fatal. The queue holds a **store**
   peer; its address is `store-addr` in `:durable`.

## THE SHAPE

Each site needs **its own address in scope** to redial. That is the per-site work and it is why this
is not a codemod — do not try to write one.

```wat
;; before
(:wat::kernel::RecvOutcome::Closed
  (:wat::kernel::assertion-failed! "<svc>: <op> closed" …))

;; after — the Lost arm's shape, against this arm's own address
(:wat::kernel::RecvOutcome::Closed
  <re-dial <addr> from :durable; on Connected, carry on with the fresh peer;
   on any ConnectOutcome failure, assertion-failed! "<svc>: redial <what> failed
   — peer is dead, not a broken pipe">)
```

## STOP TRIGGERS

1. **A `Closed` arm has no address in scope to redial.** That is a real finding about the state
   shape, not something to work around. STOP and report which arm.
2. **You are about to change a `Stopped` arm.** `Stopped` means shutdown, not fault. S26. STOP.
3. **You are about to touch `wat/service.wat`.** That is the reactor's own peer handling, a
   different layer and stdlib. S27. STOP.
4. **You cannot make the assertion fire in proof row 2.** If `Closed` can no longer kill anything,
   the recovery is unfalsifiable and the stone is a deleted wall, not a fixed one. STOP and report.
5. **The circuit's invariant moves.** `distinct=8000; dup=0`. Any change is a finding — capture it,
   do not tune it away.
6. **You are about to add a retry limit or backoff.** Out of scope; chaos will inform that policy.
   A redial that keeps failing already asserts. STOP.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop — three riders on this arc died that way.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ `probe_async_publish::refused_subscriber_is_retried_not_dropped` carries a timing-coupled
assertion (S24). If it fails **loudly with `after-drain=got`**, that is the known race naming
itself, not your regression — report it and point at S24.

Leave your work uncommitted. Prior comparable result for shape: `SCORE-the-instrument-fits-the-question.md`.

## REPORT

- **proof row 1 in full**: a worker severed mid-run, and the run still finishing `distinct=8000;
  dup=0`. This is the row that matters — a unit probe proves redial works; this proves the *system*
  survives it
- **proof row 2**: the assertion still firing against a genuinely dead service, message verbatim
- the count of `Closed` arms you actually changed, against my 17
- any arm with no address in scope
- the circuit: five runs, `total`/`distinct`/`dup`/publish ms
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas, especially where this brief did not match the disk.** My census is by arm this
  time rather than by bare token — the last one would have swept a latency histogram into a queue
  rename — but it is still mine. **The count you find is the fact.**
