# SCORE — the server drops a reply

**NOT STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.

```
Summary [ 363.997s] 5214 tests run: 5214 passed (3 slow), 19 skipped
FLOOR=0        my own run · rate-0 circuit distinct=8000; dup=0; seen-dups=0 ×5
```

The 19 skipped are 15 + **four new** placement cells, `#[ignore]`d because the floor runs rate 0.
No existing test was silenced.

## ★ ROW 0 WAS ANSWERED — and the answer dead-ends

**`state` is in scope at all five `send-keep-serving?` sites** (`1659 1693 1776 1799 1838`), along
with `new-state` / `final-state`. Option (a) is reachable at the call sites, and neither rejected
alternative was improvised.

But reachable is not sufficient:

- durable fields are **per-service**, so a generic `drop?` fed from state would hit **`stats`** as
  well as `claim` — the drop cannot be *targeted*
- and **adding fields to every Record sits before line 896**, which trips the bijection goldens

So the seam can carry a drop; the drop cannot yet be aimed.

## ⛔ THE GOLDENS HAVE GONE FROM INSTRUMENT TO OBSTACLE

R1 v2 died on them. R1 v3 turned them into an **instrument** — four tests that go red if any line
above 896 moves, which was exactly the tripwire that stone needed.

**Now they block a legitimate structural change from the other direction.** They pin **absolute line
numbers** to assert a **span**, so any addition above 896 is expensive regardless of correctness —
and adding a field to every service Record is precisely such an addition.

★ **That is now its own stone, and it sits in front of R2**: make the goldens assert what they mean
(the span's shape and message) without pinning where it starts. **S40.**

## ⛔ THE TABLE COULD NOT BE MEASURED AT WEIGHT

Drop-before at 8000 **hung publish**:

```
never-accepted; depth=744; cap=64; elapsed=60000
```

D2's liveness bound doing its job — depth 744 against cap 64, reported with what it saw rather than
"gave up."

**Each dropped claim costs a 5000 ms client deadline plus a retry.** At 10 % of 8000 claims that is
not chaos, it is saturation. ★ **The rate that produces a duplicate and the rate the system survives
may not overlap at T1's deadline** — which is a finding about **T1's 5000 ms**, not only about R2.
**S41.**

## ★ AND THE DISCIPLINE CALL WORTH KEEPING

Tiny `n=12` at 10 % gave `seen-dups=0` for **both** placements. My DESIGN said: *"if both cells
agree, the placement was never the variable."*

**The executor refused to read that as a result** — the run is too small to force hits, so agreement
there is noise, not evidence.

That is my own rejection criterion, correctly **declined because its precondition was not met.** A
weaker report would have claimed the table disproven, or claimed a pass; this one claimed neither and
said why. Third stone running the executor has stopped rather than deliver a plausible number.

## What is on disk and safe

The helper takes `drop?`; **every call site passes `false`.** Rate 0 draws nothing, the floor is
green, and the circuit is untouched at weight. The armed drop is still the claim arm's `None` reply
from T1's era, not the seam's.

## What R2 needs, in order

1. **S40 — free the goldens.** Until a Record can grow a field, the drop cannot be targeted.
2. **S41 — settle the deadline/rate relationship.** A drop rate that saturates the queue measures
   backpressure, not duplication. Either the deadline shrinks, the rate shrinks, or the measurement
   moves to a fixture small enough to force hits and large enough to produce a `distinct`.
3. **Then R2**, with row 2 measurable and row 3 (the predicted stranding) reachable. Neither was
   reached here: *"the run never produced a distinct."*

## Still open

**S40** (goldens pin absolute lines) · **S41** (drop rate vs survivable rate) · **S37** (userland
`defn` unreachable from a process impl) · **S38** (the freshness proof may not be floor-gated) ·
**S39** (selectable eviction) · **S15**–**S36**.
