# BRIEF — excursus 001 stone 6: `Envelope` moves into `:messages`

**Small, and it takes the floor back to green.** Stone 5 widened the surface guard's reach;
`wat-scripts/queue/sqs.wat` now correctly fails to freeze. This is the fix it is asking for.

## The work

Move the `:queue::Envelope` `defrecord` (currently `wat-scripts/queue/sqs.wat:36`, **beside**
the surface) **into `:queue::Queue`'s `:messages`** vector.

## It is a PURE MOVE — measured, not assumed

**A message may keep a non-prefixed name.** Verified on the reproduction: `:p::Item` declared
*inside* `:messages`, without renaming to `:p::Src::Item`, type-checks clean. So this needs **no
rename and no call-site churn** — lift the block, nothing else.

The same was confirmed against the real file: moving `Envelope` into `:messages` on a scratch
copy gave `--check = 0`.

## Why it is the right fix rather than a workaround

`wat/service.wat:792` ships `(S::surface-forms)` — and only that — into a forked child. A type
inside `:messages` crosses; a type beside the surface does not. `Queue::ReceiveResponse::Ok`
carries `(Vector :- [:queue::Envelope])`, so a forked worker must be able to name `Envelope`.

`wat/query.wat:500`'s *"stay top-level: they cross via stdlib"* exemption is real and does not
apply here — **`wat-queue` is userland.** See
`NOTE-a-userland-peer-surface-must-carry-its-domain-types-in-messages.md`.

## The gate — the grep precedent's standard, applied to a move

> *"mostly a MOVE of proven code, and **the counts are the proof it moved intact**"* (arc 278,
> `349a2ea52`).

The count here is the queue's own summary, and it must be **byte-identical** afterwards:

```
./target/release/wat wat-scripts/queue/sqs.wat   →   "bound=x;r1=a,b;r2=c;r3=;redel=b"
```

Plus: `--check` on `sqs.wat` returns **0**, `probe_ex001_queue` passes, and
`every_wat_scripts_file_loads` goes green. **Floor back to `FLOOR=0`.**

## ⛔ What this stone does NOT do

- **It does not touch `wat-scripts/fanout/circuit.wat`.** That program carries stone 4's
  foreign-read workaround, written when `Envelope` was unreachable. After this move the
  workaround is unnecessary and **may be actively wrong**. The floor only requires the circuit
  to *freeze*, so the floor should go green either way — but **the circuit's runtime behaviour
  is stone 7's**, and stone 7 is the re-attempt of the fan-out proof. If you notice what the
  workaround now does, **write it in the SCORE** so stone 7 starts informed.
- **It does not re-attempt the proof.** `dup=0` stays vacuous until stone 7.

## STOP triggers

1. **If the move requires a rename** — it should not; that was measured. If it does, **STOP**:
   the measurement was wrong and that matters more than the move.
2. **If the summary changes at all** — `bound=x;r1=a,b;r2=c;r3=;redel=b` must be byte-identical.
   A move that changes behaviour is not a move. **STOP and report the delta.**
3. **If anything outside `wat-scripts/queue/sqs.wat` needs to change — STOP and name it.**
4. **If the floor does not return to fully green — STOP**, capture whole, do NOT re-run.

## Blast radius

`wat-scripts/queue/sqs.wat` — one block moved. This excursus's SCORE. **Nothing else.**

## Verify — never through a pipe

```bash
./target/release/wat --check wat-scripts/queue/sqs.wat; echo "CHECK=$?"
./target/release/wat wat-scripts/queue/sqs.wat
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor is **5126 with 2 reds, both the queue.** This stone must take it to **FLOOR=0**.
