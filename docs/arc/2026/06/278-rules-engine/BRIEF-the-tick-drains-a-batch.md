# BRIEF — the tick drains a batch

The topic's `-deliver` handles exactly one message per tick — 2000 ticks for 2000 messages,
measured. Make it drain up to **10**, so the timer round trip, the state construction and the outbox
rebuild are each paid once per ten messages instead of once per message. One arm, one file, no
surface changes.

## Read in order

1. **`DESIGN-STONE-the-tick-drains-a-batch.md`** — the numbers and the one contract decision (K is
   bounded because the topic is deaf while `-deliver` runs). Read first.
2. **`wat-scripts/topic/sns-fanout.wat`**, `-deliver` — the whole arm. Today it does: read state →
   rebuild `rest` (drop one) → fan out to four subs → re-arm via the level-triggered helper.
   **The rebuild must end up OUTSIDE the per-message loop.** That is the point of the stone, and it
   is the easiest part to get subtly wrong.
3. **`wat-scripts/fanout/circuit.wat`**, the worker's `-tick` — `:limit 10` on `Queue/receive`, the
   existing precedent for a bounded per-tick batch, and where K=10 comes from.
4. **`SCORE-the-sane-circuit.md` row 5** — "the worker is interruptible; `Admin::Stop` is taken
   between ticks". That property is what bounds K; read it so you do not trade it away.
5. **`wat-scripts/topic/sns-fanout.wat`**'s `arm-deliver` helper — arming is already level-triggered
   from state. **Do not touch it**; it already does the right thing when the outbox is still
   non-empty after a batch.

## The sketch

Load-bearing: **one rebuild per tick**, and the fan-out stays per message. Illustrative: the loop.

```wat
;; how many this tick — bounded, and never more than the outbox holds
k (:wat::core::if (:wat::i64::< (:wat::vector::length box) 10) (:wat::vector::length box) 10)

;; deliver each of the k heads — the EXISTING concurrent fan-out, unchanged, k times
_n (:wat::core::foldl
     (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
       (:wat::core::let [msg (…get box i…)]
         …all four sends, then all four recvs…))
     0 (:wat::core::range 0 k))

;; ONE rebuild, dropping k — not k rebuilds dropping one
rest (…foldl over (range 0 (length box - k)) taking (get box (+ i k))…)
```

## Blast radius

**`wat-scripts/topic/sns-fanout.wat` only**, and within it only `-deliver`. No surface changes, no
new fields, no new ops. `wat/`, `src/`, `sqs.wat` and `circuit.wat` untouched.

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Batching must not be observable in what
   arrives.
2. **If you find yourself draining until empty — STOP.** The DESIGN rules K bounded, and an
   unresponsive topic is a regression the circuit will not show you directly.
3. **If `wat/`, `src/`, `sqs.wat` or `circuit.wat` need to change — STOP and surface it.**
4. **If `topic-ticks` does not fall to roughly N/10 — STOP.** That is the instrument for this stone;
   if it does not move, nothing batched.
5. **If the per-delivery slope does not flatten — STOP and report it.** The rebuild is the only known
   superlinear term left; if amortising it by 10 does not flatten the slope, something else is
   superlinear and that is worth more than this stone.

## Shape to copy

`SCORE-the-fanout-is-concurrent.md` for reporting per-delivery across N, and for wiring the proof
into the floor rather than leaving it a one-off.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` for a running `wat`/`cargo` before any timing.

Write `SCORE-the-tick-drains-a-batch.md` when done. It will be graded by re-running.
