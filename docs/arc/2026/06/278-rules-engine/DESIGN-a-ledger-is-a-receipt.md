# DESIGN — a ledger is a receipt, not a lock

**The dead-owner loss.** `wat-scripts/fanout/circuit.wat` (+ one test that parses a renamed
field). Correctness; perf measured and reported, not optimised.

## WHY — the arc predicted this class in one line, months ago

`DESIGN-the-unknowable-state`:

> *"The consumer claims before it emits, so a lost claim-reply converts at-least-once delivery
> into at-most-once processing."*

`:fanout::seen` is written **on claim** (`circuit.wat:86-125`) and the worker emits only when
told it was first (`circuit.wat:491`). That makes the ledger a **lock**. A lock with no release
has exactly one failure mode:

```
A claims seq -> First -> the ledger holds seq for A -> A DIES before emitting.
Redelivery (A never acked). B claims -> not-first -> B stands down. B acks anyway.
NOBODY EMITTED. The message is consumed and gone.
```

⚠ Not hypothetical: **3 of 6 drop-after runs die at `claim deadline exhausted`** — a worker
dying while holding claims. The run aborts before the loss can be counted, which is the only
reason this has never shown up as a number.

## ⛔ A LEASE IS THE OBVIOUS FIX AND IS THE WRONG ONE

A TTL on ownership needs a clock and **still cannot tell a SLOW owner from a DEAD one**. It
trades a loss for a double-emit and adds a fencing question the circuit does not have. Rejected
before it was briefed.

## THE MEASUREMENT — `probe-a-ledger-is-a-receipt-not-a-lock.wat`, committed first

Same simulated death, two ledger disciplines:

```
s1 CLAIM-BEFORE  died before emit -> emitted=0   ⛔ LOST
s2 RECORD-AFTER  died before emit -> emitted=1   ✅ no loss
s3 RECORD-AFTER  died AFTER emit  -> emitted=2   ⚠ duplicate, NOT loss
```

★ **Record-after converts every loss into a duplicate.** s3 is the honest half: it does not
achieve exactly-once, and it is not claimed to.

## ⛔ THE ONE CONTRACT DECISION

**The receipt is written AFTER the outcome is emitted and BEFORE the ack.** The ledger stops
answering *"who holds this?"* and answers *"has this been reported?"* — the question the caller
is actually asking.

```
receive -> check(seq)
             Absent   -> emit, mark(seq), ack
             Recorded -> skip emit,        ack
```

★ The value type goes back to `bool`. **The original `bool` was the right type for the wrong
fact**: it meant *claimed*; it now means *reported*. Ownership disappears from the design — and
with it the dead-owner class, because nothing is held, so nothing can be held by a corpse.

★★ And note which lost reply now matters. Under claim-before, a lost `claim` reply is
catastrophic and unknowable — *did my claim land?* Under record-after, a lost `check` reply
means **nothing happened yet**, so retrying is free; the only reply that matters is `mark`, and
retrying `mark` is idempotent.

## THE INVARIANT — stated, because the last stone died for want of it

**`distinct = N`, `dup >= 0`.** This arc's standing ruling (`TRACKER`, 3-historical):
*"`dup=0` is a property of the transport, not of the design."* The previous BRIEF's STOP-3
asserted `dup=0` and contradicted it; that is not repeated here.

⚠ But `dup >= 0` is a **description of the irreducible window, not a licence.** A duplicate a
worker could have avoided is still a defect. Row 2 below holds `dup=0` where no worker dies —
because there, nothing is irreducible.

## FILES

`wat-scripts/fanout/circuit.wat`, and `tests/services/probe_arc278_sane_circuit.rs:124` which
parses `seen-dups` and must follow the rename.

## OUT OF SCOPE = REJECTED

- **The `claim deadline exhausted` crash** (3/6 tiny runs). It is what makes the dead-owner loss
  unobservable at circuit scale, and it is still its own stone.
- **A lease / TTL on ownership.** Rejected above, with the reason.
- **A worker-local emitted-set.** Record-after subsumes it: the shared receipt is the memory,
  and it survives the worker.
- **All perf work**, including the two seen round-trips this adds per message and the send-path
  double scan. Measured and reported; not optimised here.
