# DESIGN — the client backs off intelligently

**Exponential backoff with full jitter and a cap**, replacing the publisher's fixed 1 ms retry.
`wat-scripts/fanout/circuit.wat` only — a client-side change, no contract, no server.

## WHY — the constant is a guess sitting in the middle of a trade

Measured this session, sweeping the one constant, `distinct=8000` at every point:

```
await   publish   retries   asleep    % of publish asleep
  1ms    22395      3807     4835 ms       22%
  5ms    19619      1784     9402 ms       48%
 10ms    18805      1130    11605 ms       62%
 25ms    18472       540    13646 ms       74%   ← best
 50ms    19962       318    15986 ms       80%
```

★★ **The publisher sleeps three times as long and the wall gets shorter.** So the publisher is not
the rate limit — the drain is, and every wake takes a turn on the serialized queue actor that a
worker could have used. Below ~5 ms it wakes too often and steals turns; past ~25 ms it sleeps
through room that appeared. The optimum is a point on a contention/latency trade, and **no fixed
number should be sitting on it.**

⚠ Deleting the wait is not the answer — measured: pure spin gives `publish 22794`, **worse** than
1 ms. The wait is load-bearing; only its value is arbitrary.

## ⛔ THE ONE CONTRACT DECISION — full jitter, not plain exponential

```
ceiling = min(CAP, BASE << attempt)
delay   = rand::int-from seed 1 ceiling        ;; uniform in [1, ceiling]
attempt = 0 on ANY acceptance, including partial
```

**Full jitter** — a uniform draw over the whole window — rather than plain exponential (every
retry lands at the same instant) or equal jitter (half fixed, half random). It is the variant AWS
published as minimising both contention and total completion time, and *contention is exactly what
the sweep measured*.

★ **The draw starts at 1, not 0.** Classical full jitter allows a zero delay; here
`:wat::time::NonZeroDuration` makes zero unrepresentable — a wait of zero is a different operation
(spin), and this arc already struck the mode-spelled-as-a-magnitude that allowed it. The type
forbids the classical form, and the measurement says the type is right: spin is worse.

★ `:wat::rand::int-from` is **threaded** (`state → (state', value)`, pure ∧ deterministic), so the
backoff is reproducible from a seed and the benchmark stays repeatable.

## THE TWO CONSTANTS, AND WHY THEY ARE NOT THE OLD PROBLEM

`BASE = 1 ms` — the smallest legal wait. `CAP = 100 ms` — past the observed turn-up at 50 ms, so
the curve is bounded on the far side.

★★ These are **bounds, not an operating point.** The client discovers where to sit each run
without anyone naming it, and neither bound needs to be right to within a factor of two. The 25 ms
was a value tuned to one workload; these are limits on a policy.

## ⛔ WHAT WOULD MAKE THIS NOT WORTH SHIPPING

The fixed-25 ms optimum is **18472 ms**. If adaptive backoff does not beat it, it is not better —
it is merely more principled, and the SCORE must say exactly that rather than dress a tie as a win.

## THIS IS HALF OF A COMPOSITION — the other half is named, not deferred

`receive` already takes `Wait :Immediate | :UpTo [NonZeroDuration]`. `send` takes nothing, so a
sender that wants to wait for room **must** poll. The queue knows precisely when room appears — it
is the thing that dequeues — and has no way to say so.

The composed design: **park with a deadline; if the deadline fires, back off with jitter and park
again.** That is what a real SQS client does (long-poll, then backoff), and it is why `receive` has
the wait mode already.

⚠ Backoff goes first **deliberately**: it is client-side only, and it gives parking something to be
measured against. Shipping parking first would credit it with wins a better client would have got
anyway.

## OUT OF SCOPE — REJECTED

- **`Wait :UpTo` on `send`.** The composed second half. Its own stone, measured against this one.
- **Tuning `BASE`/`CAP` to beat 18472.** That would re-create the magic value with extra steps.
- **`setup` / `stop`** — 16 s, and publish is not finished.
- **Raising any `cap`.** The pressure system is the feature.
