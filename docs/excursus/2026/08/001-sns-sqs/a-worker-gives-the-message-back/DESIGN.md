# DESIGN — a worker gives the message back

**The `claim deadline exhausted` crash, and the chaos coverage that hid it.**
`wat-scripts/fanout/circuit.wat` only. Correctness. No perf work.

## WHY — the crash did not improve; the instrument moved

`circuit.wat:455-458`: after three failed `check` attempts the worker calls
`assertion-failed!` and **dies**, aborting the run.

It was measured at **3/6**, then **1/6**, then **0/6** across the last three strikes. That reads
like a fix. It is not one:

★ **The drop now lives in `mark` (`circuit.wat:145`). `check` is a pure read with no drop knob
at all.** The exhaustion arm fires only when `check` times out three times, so moving the chaos
off `check` made the crash unreachable. **Nothing was repaired.**

★★ Same class as every instrument failure this campaign has found: a number that improved
because the thing measuring it looked away. The 0/6 is the most misleading result of the day.

## THE SECOND, LARGER FACT

**We built four deadlines and we exercise one.** Chaos can currently be aimed at exactly one
call:

| call | droppable? |
|---|---|
| `Seen/mark` | ✅ current target |
| `Seen/check` | ⛔ knob removed when the drop moved |
| `Queue/ack` | ⛔ never had one |
| `Queue/receive` | ⛔ never had one |

Every move of the injector darkens the path it left. This stone restores `check` and makes the
two knobs independent; the queue-side pair is named and cut below.

## ⛔ THE ONE CONTRACT DECISION

**Exhausting the retry budget is not a fault. It is a decision to give the message back.**

On exhaustion the worker **skips that envelope entirely** — no emit, no `mark`, and critically
**no ack** — and continues with the rest of the batch. The message stays unacked, visibility
expires, and it is redelivered.

★ **The receipt discipline is what makes this safe.** Exhaustion happens at `check`, *before*
any emit and *before* any receipt. So a worker that gives up has done nothing observable: the
next receiver checks, finds `Absent`, and emits. **No loss, and no duplicate** — the give-back
is clean precisely because the previous stone moved the write after the work.

## ⚠ NO NEW PROBE, AND THAT IS A DECISION

Every assumption here is already covered by a committed probe or by code in this file:

- *a `check` drop knob works* — identical to `mark`'s, `circuit.wat:145`
- *an unacked message comes back* — `probe-visibility-redelivers.wat`, and the drop runs' own
  `seen-skipped`
- *give-back is loss-free* — `probe-a-ledger-is-a-receipt-not-a-lock.wat` cell s2

★ **The genuinely uncertain thing is behavioural, not structural: does the system converge when
`check` is lossy?** That cannot be probed in ten lines — it *is* the acceptance rows. Written
down so the missing probe is a choice rather than an omission.

## FILES

`wat-scripts/fanout/circuit.wat` only.

## OUT OF SCOPE = REJECTED

- **Queue-side drop knobs (`ack`, `receive`).** The same coverage gap, one file further out;
  its own stone, so this one's numbers stay attributable.
- **Retrying more than three times.** The budget is not the defect; what happens *at* the budget
  is. Changing both would confound them.
- **The redelivery fixture that kept its name and lost its meaning**, and the rung-3 census of
  undeadlined generated methods. Both open, neither touched.
- **All perf work.**
