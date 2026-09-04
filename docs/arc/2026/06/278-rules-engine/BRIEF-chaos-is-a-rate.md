# BRIEF — chaos is a rate

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`.
Read `DESIGN-chaos-is-a-rate.md` first — it carries the four proven probes, the seven carries, and
the mechanism that has no form.

## THE WORK

Give the circuit's workers an opt-in `-disrupt` internal arm. On a seeded draw it reaps its own
connection by sending an oversized frame, re-acquires the peer, threads the fresh peer into state,
then draws a delay and **re-arms itself**. Rate and seed live on the `Record` so a chaotic run
replays. **Rate 0 arms no alarm at all** — a service with chaos off pays nothing.

Every mechanism is already proven. You are composing four green probes, not discovering anything.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-disrupt-reaps-and-reacquires.wat`** — **run it first.** The
   shape: an internal arm that poisons its own peer and re-acquires. `before=ok:1/d=0;
   after=ok:2/d=1`. ⚠ It returns **empty alarms** — one-shot. **The re-arm is your work.**
   ⚠ Its `hit` arm collapses `Sink/ping` to `-1`/`-2` at `:123-124`. **Do not copy that.**
2. **`wat-scripts/scratch-pad/probe-rand-is-usable-from-wat.wat`** — the threading idiom for
   `:wat::rand::int-from state lo hi` → `(Tuple new-state draw)`. Both bounds `[lo, hi)`.
3. **`wat-scripts/fanout/circuit.wat`** — the worker, and 3c-pre's **always-on** poison in worker
   start. ⛔ **That was a proof instrument. Do not copy it.** Yours is rate-gated and re-arming.
4. **`wat-scripts/fanout/circuit.wat:884-900`** — the `process/post-spawn` grant idiom. A dialer is
   a stranger until granted; the grant fires owner-side after the fork and **before `:init` dials**.
5. **`wat-scripts/topic/sns-fanout.wat:22-27`** — why the grant exists, in prose.
6. **`wat/service.wat:89-95`** — `SelfOutcome`, which has **no reply field**. Your arm returns this.
7. **`docs/arc/2026/06/278-rules-engine/SCORE-a-peer-is-dead-only-when-redial-fails.md`** — the
   `Closed` recovery this stone leans on, and the captured red that proves thread locus does not tear.

## SKETCH

```wat
;; start — OPT-IN: rate 0 arms nothing at all
(:wat::core::if (:wat::i64::> rate-bp 0)
  <Outcome::Continue … arms: [(Alarm :after (Millisecond first-delay) :op :-disrupt)]>
  <Outcome::Continue … arms: []>)

;; -disrupt  [s ctx] -> SelfOutcome (no reply field: a disruptor has no caller)
;;   1. (rand::int-from seed 0 10000)      -> (seed', draw)
;;   2. if draw < rate-bp: send an oversized frame on the peer, re-acquire,
;;                         THREAD THE FRESH PEER INTO STATE, count it
;;   3. (rand::int-from seed' lo hi)       -> (seed'', delay)
;;   4. SelfOutcome::Continue <state with seed''> sends
;;        [(Alarm :after (Millisecond delay) :op :-disrupt)]      ;; ← RE-ARM
```

**Every outcome named.** No `-1`/`-2` collapse anywhere in what you write — a fallback that cannot
tell "the wire refused it" from "the peer died" is the defect this arc removed three times.

## STOP TRIGGERS

1. **You are about to call `:wat::kernel::close`.** It has no form here — `runtime.rs:25160`
   `#[restricted_to(… ":wat::kernel::")]`, and it reaps a spawned *child*, not a dialed `Peer`.
   Banked at `tests/kernel/probe_arc259_s2d_internal_only_close.wat.bad`. STOP.
2. **You are about to make thread locus tear.** In-process channels do not tear on an oversized
   frame; that is the fourth locus asymmetry, a property not a bug. **Do not invent a second
   mechanism to make thread look like process.** STOP.
3. **The fresh peer is not threaded back into state.** `Closed` then becomes an infinite loop **that
   looks like a hang**. Rejection criterion. STOP.
4. **`-disrupt` does not re-arm.** That is the probe, not the stone. STOP.
5. **The default rate is anything but 0**, or rate 0 still arms a timer. STOP.
6. **You are about to touch `wat/service.wat` or `src/`.** Neither is in scope. STOP.
7. **You are about to braid 3d** (the `None`-reply drop). Separate stone. STOP.
8. **The circuit's invariant moves and you are about to tune it.** ⛔ `dup > 0` under chaos is a
   **finding** — at-least-once permits duplicates and R69 records that our detector was blind to
   them for nine stones. Report the number. Do not tune it away, and do not weaken the assertion.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop — three riders on this arc died that way.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ S24 is live: `probe_async_publish::refused_subscriber_is_retried_not_dropped` carries a
timing-coupled assertion and can fail loudly with `after-drain=got`. That is the known race naming
itself, not your regression.

Leave your work uncommitted. Prior comparable result: `SCORE-a-peer-is-dead-only-when-redial-fails.md`.

## REPORT

- **the disruption count** across a run with the rate on. One firing fails the stone
- **two runs at the same seed**: the same count at the same points
- **rate 0**: no alarm armed at all — show it, do not assert it
- the circuit under chaos: `total`, `distinct`, **`dup`**, five runs
- the floor Summary line verbatim, with the default rate
- every STOP that fired
- **the honest deltas.** Six of my censuses have been wrong this campaign, each differently; the
  last would have swept a latency histogram into a queue rename. What you find is the fact.
