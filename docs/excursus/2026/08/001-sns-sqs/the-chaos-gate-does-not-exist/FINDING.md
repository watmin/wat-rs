# FINDING — the chaos gate does not exist, and when switched on it goes RED

Builder: *"let's see if we've grown this to a durable and resilient solution."* Switching the
fault injection on answered it in one run.

## ⛔ TWO OF SEVEN CHAOS SCENARIOS FAIL, REPRODUCIBLY

`cargo nextest run --release --run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'`

```
Summary [ 109.817s] 7 tests run: 5 passed, 2 failed, 5252 skipped        exit 100

FAIL [ 10.027s]  drop_ack_tiny          drop-ack-bp  = 1000  (10 % of QUEUE ack replies dropped)
FAIL [ 10.694s]  r2_drop_before_tiny    drop-mark-bp = 1000  (10 % of seen-store MARK replies dropped)
PASS             drop_check_tiny · drop_recv_tiny · r2_drop_after_tiny
PASS             r2_drop_after_write (n=2000) · r2_drop_before_write (n=2000)
```

Whole log captured beside this file as `CAPTURED-chaos-all7.log`. The arm, verbatim:

```
#wat.kernel/AssertionFailure
  :message "drained-never: last=[0/0][0/0] outbox=19 attempts=100 elapsed=2086;
            check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"
  :location circuit.wat:2342  :fanout::require!
  :frames   circuit.wat:2496  :fanout::run-with   ← :user::drop-ack-tiny
```

`r2_drop_before_tiny` is identical in shape: `outbox=19 attempts=100 elapsed=2226`.

## What that message means, read from the source

`poll-until-drained*` exits only when **both** `sweep-drained?` (every subscriber queue empty)
**and** `box = 0`, where `box` is `:fanout::topic-outbox` — the **topic inbox's depth**.

So at the moment of failure: **every subscriber queue is empty (`[0/0][0/0]`) while 19 of 100
message-deliveries are still sitting in the topic inbox.** The consumer side finished; the fanout
did not.

⚠ **And no counter explains it.** `ack-retries=0`, `ack-exhausted=0`, `mark-exhausted=0`,
`check-exhausted=0` — every retry/exhaustion counter reads zero while 19 entries are stuck. Whether
those 19 are **permanently lost** or **awaiting a redelivery the 2 s budget did not reach** is
**unresolved by the instruments that exist.** That gap is itself a finding: the harness can detect
that the system did not drain and cannot say why.

★ Both failures report **exactly 19**, under two different knobs. Both use `drop-seed 42`, so the
count is presumably seed-determined rather than coincidental — worth confirming by varying the seed,
which is the first thing the stone should do.

## ⛔⛔ THE DEEPER FINDING: these tests could never have caught it

All seven chaos scenarios are, in the tree as it stands:

1. **`#[ignore]` with no reason string** — so they never execute in the floor. They are 7 of the
   **22 skipped** on every green Summary line.
2. **Assertion-free.** Every one calls its fixture and `eprintln!`s the result. `grep` for `assert`
   between `#[ignore]` and the closing brace of all nine tests in the file returns **nothing**.

So even un-ignored they would pass regardless of outcome. **The entire fault-injection surface of
this system is switched off AND toothless.** The two reds above surface only because the wat itself
raises `assertion-failed!` — the Rust test contributes no gate at all.

⛔ **I quoted `5237 passed, 22 skipped` four times today as evidence of correctness and never once
asked what the 22 were.** *A green from an instrument that was never asked is silence, not proof.*

★ The existing wall `tests/lint/ignore_reason_justified.rs` polices the **content** of an ignore
reason (no bare "circle back" promises) but not its **existence** — a bare `#[ignore]` with no reason
sails through unaudited. That is a second, narrower hole worth closing.

## ⛔ MY OWN TWO DOCTRINE VIOLATIONS, in one command

1. **I piped the first chaos run through `grep`**, so when two tests went red I had no failure text —
   the exact truncation `scripts/floor.sh`'s header was written about, committed one command after I
   quoted that header to an executor.
2. **I then re-ran the two failing tests alone, and they went green** (8.6 s / 9.2 s vs 10.8 s /
   10.7 s). That is precisely the *"re-run went green and destroyed the only evidence"* scenario the
   same header describes. I got the evidence back only because the failure reproduces under load —
   had it not, the finding would have been unrecoverable and I would have caused that.

★ The pass-on-rerun is not noise, and it should not be filed as flakiness: **it reproduces with all
seven running (twice) and passes with two running.** The failing runs are ~2 s SLOWER than the
passing ones, and the parallel set includes two n=2000 circuits at ~31 s. That points at a
**load-sensitive budget**, not a random flake — the drain budget is `n×m` attempts of 5 ms plus
`count(qclients)+1` stats calls per attempt, so its real wall time depends on how contended the box
is. A resilience property tuned to an idle machine is a finding in its own right.

⛔ **It must not be called a flake.** It is a reproducible red under a stated load.

## What the stone must do

1. **Vary `drop-seed`** — establish whether `outbox=19` is seed-determined and whether the failure
   survives other seeds. One run each at three seeds.
2. **Separate lost from slow.** Raise only the *drain budget* (not any cap, not any rate) and see
   whether the 19 clear. If they clear, this is a budget/liveness defect; if they do not, messages
   are being lost under 10 % ack-reply loss and that is a correctness defect. **Either answer is the
   deliverable.**
3. **Give the seven tests assertions** — `distinct = n×m` and `dup = 0` at minimum — so they can fail.
4. **Decide their ignore status against the floor's 30 s wall.** The tiny ones run in 8–10 s and can
   plausibly gate; the two n=2000 ones take ~31 s and cannot. Any that stay ignored need a
   **checkable reason**, per the existing lint's own standard.
5. ⚠ Do **not** widen a rate, a cap, or a timeout to make a red go green. The reds are the product.

---

# ⭑ THE INVESTIGATION — read on the disk, not reasoned

## ⛔⛔ THE HARNESS HAS A HOLE EXACTLY WHERE THE NEWEST SAFETY PROPERTY LIVES

`circuit.wat:2168` — the **subscriber** queues, built in a fold:

```
:record (:queue::queue::Record :cap sub-cap :store-addr … 
          :drop-recv-bp drop-recv-bp :drop-ack-bp drop-ack-bp :drop-seed drop-seed)
```

`circuit.wat:2178` — the **inbox** queue:

```
:record (:queue::queue::Record :cap 64 :store-addr …
          :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0)     ← HARDCODED ZERO
```

**Fault injection reaches tier 2 only. Tier 1 — the inbox — can never drop anything.**

★★ Which means **the inbox stone's safety property has never been executed.** `ack-after-sends`
with `ok = min over subscribers`, the thing `e0c552bf0` was written to protect and whose comment says
*"reverse the order and a crash mid-expansion loses the message SILENTLY"* — is verified by **reading
the code only.** No test can reach it as this harness is wired, because reaching it requires the
inbox's own receive or ack to fail, and both are pinned to zero.

## ⛔ And the code that would handle it is untested and silent

`sns-fanout.wat`, the worker's inbox-ack arms:

```
((RecvOutcome::Message _ar)  inbox)               ← success, keep the peer
((RecvOutcome::Lost _cause)  → reconnect)          ← continues
(RecvOutcome::Stopped        → assertion-failed!)
((RecvOutcome::Closed)       → reconnect)          ← continues
((RecvOutcome::TimedOut)     → reconnect)          ← continues
```

Three of the five arms **reconnect and move on. No retry of the ack, and no counter incremented.**
So a lost inbox-ack reply would be absorbed *invisibly* — which is why `ack-retries=0` and
`ack-exhausted=0` are not reassuring: those counters live in the circuit's consumer, not here, and
this path has no counter at all.

⚠ **This code is on the safety-critical path and is currently unreachable by any test.**

## The queue applies the mutation, then drops the reply

`sqs.wat:1097`, reached **after** the store delete has already returned:

```
(:wat::service::Outcome::Continue s-a
  (:wat::core::if hit? :wat::core::None                     ← reply suppressed
                       (Some (Reply::Ack (AckResponse::Ok)))))
```

So a "dropped ack" is a **lost reply to a completed mutation**, not a lost mutation. The rows are
gone; only the acknowledgement vanished. That is the right fault to model (it is what a network
partition after commit looks like), and it makes ack idempotence the property under test.

## ⚠ The `outbox=19` chain — a HYPOTHESIS, labelled, with its test named

Because the drops are on tier 2, the inbox backing up is a **downstream consequence**, not a direct
fault. The chain that fits every number observed:

1. A consumer's ack to a sub queue is applied but its reply is lost.
2. The consumer treats the ack as unfinished; the sub entry stays until visibility expiry.
3. Sub queues retain entries and approach **`sub-cap 32`**.
4. A full sub queue **refuses** the worker's fan-out send, so `ok = min over subs` falls.
5. The worker therefore acks **less of the inbox** — by design, that is the safety property working.
6. The inbox stops draining. `outbox=19` at budget exhaustion.

★ **This is not established.** It is consistent with `[0/0][0/0]` (subs eventually drain) and with
every retry counter reading zero (the counters that would move live on paths the drops never touch).
**Its test: vary `sub-cap` alone and see whether `outbox` tracks it.** If raising `sub-cap` clears
the 19, the chain holds and this is a *liveness/backpressure* interaction, not message loss. If it
does not, the hypothesis dies and something else is holding the inbox.

## What the investigation changes about the stone

The stone I sketched asked "lost or slow?". The investigation says ask two questions, and the second
is bigger:

1. **Is it backpressure?** Vary `sub-cap` alone (32 → 64 → 128) at fixed seed and rate. `outbox`
   tracking `sub-cap` confirms the chain. **Do not** raise it as a fix — raise it as an instrument,
   then put it back.
2. ⛔ **Wire the inbox into the fault injection.** `circuit.wat:2178`'s three hardcoded zeros are the
   reason tier 1's safety property is untested. Passing the same knobs there — or a separate pair —
   makes `ack-after-sends` and `ok = min` reachable for the first time. **That is the stone with the
   most value in it**, because it tests the newest and least-exercised invariant in the system.
3. Give the seven scenarios assertions (`distinct = n×m`, `dup = 0`) so they can fail on their own
   rather than only when the wat raises.
4. Rule on their ignore status against the 30 s wall: the five tiny ones run 8–10 s; the two n=2000
   ones take ~31 s and cannot gate.

⛔ Still: do not widen a rate, a cap, or a timeout to turn a red green. The reds are the product.

---

# ⭑⭑ PROBE B — RUN, AND IT REFUTED MY HYPOTHESIS. TWICE.

The CLI had no chaos surface at all — all six knobs were literal zeros at `circuit.wat:2580` — so
this probe first required giving it one. `argv 8/9/10` are now optional `drop-recv-bp` /
`drop-ack-bp` / `drop-seed`, defaulting to 0, so every existing invocation is unchanged (verified:
`200 2 2 32 false 1000` still gives `distinct=400 dup=0 disrupts=0`).

## The failure is LOAD-INDUCED, and nothing else

Same binary, same arguments, same `drop-seed 42`:

| condition | result |
|---|---|
| alone on a quiet box, `sub-cap` 32 / 64 / 128 / 256 | **PASS, all four** |
| 8 concurrent copies of the identical command | **4 of 8 FAIL** (exit 2) |

`attempts=100` is exhausted in every failure; `elapsed` 1627–2083 ms.

## ⛔ Hypothesis 1 — backpressure via `sub-cap` — REFUTED

The chain I wrote up (sub-ack reply lost → entries held → sub queues hit `cap 32` → worker's fan
refused → `ok = min` falls → inbox stops draining) predicted that `outbox` would track `sub-cap`.
**It does not.** `sub-cap=32`, the fixture's own value, passes cleanly when run alone. Capacity is
not the binding constraint.

## ⛔ Hypothesis 2 — `19` is seed-determined — REFUTED

I noted both failing tests reported exactly `outbox=19` and guessed the count came from the seed's
draw pattern. Under load with **the same seed 42**, `outbox` comes out **9, 10, and 19**. It tracks
contention, not the seed.

## ⛔ Hypothesis 3 — the budget cannot cover redelivery latency — REFUTED

If the 100-attempt budget were losing a race against 200 ms visibility expiries, shortening
visibility should have fixed it. Under identical 8-way load:

```
vis-ms = 200 (default)   4/8 fail
vis-ms =  50             4/8 fail
vis-ms =  20             3/8 fail
```

**Redelivery latency is not the variable.** Making redelivery 10× faster changed essentially nothing.

## ⭑ What IS established, and the arithmetic that points at the next hypothesis

- The failure is **purely load-induced**: passes alone at every `sub-cap`, fails 3–4 of 8 under
  self-contention.
- The budget is **always fully consumed** — `attempts=100` in every failure.
- **The budget is spent observing, not waiting.** 100 attempts × 5 ms of intended sleep = **500 ms**,
  yet `elapsed` is **1627–2083 ms**. So **1.1–1.6 s — two to three times the intended budget — goes
  into the poller's own round-trips**: `poll-until-drained*` makes `count(qclients) + 1 = 3` stats
  calls per attempt, i.e. **300 round-trips into the very queue processes the workers need**.

★★ **Next hypothesis, stated as one: the poller starves the system it is polling.** Its stats traffic
is a material share of the contention, so under load the completion check perturbs the completion it
is checking. If true, adding budget makes it *worse*, and the fix is a cheaper or rarer poll — which
means changing `poll-until-drained*`, a stone rather than a probe.

⚠ **Still unproven, and it is the correctness question:** whether those 9–19 inbox entries would
ever deliver. Every probe so far bounds *why the check fails*, not *whether delivery completes*. The
strongest evidence is indirect — the identical run completes cleanly when unloaded — and indirect is
not proof.

## ⛔ What this says about every green we have

The two chaos reds are **not flakes**, and now they are not mysteries either: they are a completion
check that fails under CPU contention. But the same fact indicts the harness's authority in the other
direction — **a `drained-never` failure does not distinguish "the system did not deliver" from "the
poller ran out of budget while measuring."** Until it does, neither its red nor its green carries the
meaning we have been reading into it.
