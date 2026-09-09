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
