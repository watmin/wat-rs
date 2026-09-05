# DESIGN — the window gets a test

**Make the s3 window a gated property instead of a 1-in-18 flake.**
`wat-scripts/fanout/circuit.wat` + `tests/services/probe_arc278_sane_circuit.rs`. Correctness of
the *test surface*. No perf work.

## WHY — the only untested case is the one that can go catastrophically wrong

`probe_arc278_sane_circuit.rs` already tests two of three redelivery cases:

| case | test | asserts |
|---|---|---|
| redelivery is real, unabsorbed | `redelivery_is_visible_…` | `distinct=1`, `total>1`, `dup>0` |
| redelivery arrives **after** the receipt | `redelivery_is_absorbed_by_the_consumer` | `total=1` |
| redelivery arrives **mid-processing** | **none** | — |

The third is the **s3 window** — `DESIGN-a-ledger-is-a-receipt` cell s3, *"died AFTER emit →
duplicate, not loss"*. Under record-after, a redelivery that arrives before the receipt is
written finds `Absent` and emits again.

★ It fired **once in eighteen runs** during the last strike. That is the only evidence it exists
in the circuit, and a 1-in-18 event is not a property — it is a rumour.

★★ **The dangerous neighbour is loss, not duplication.** If some future change makes that window
drop the message instead of duplicating it, nothing in the tree goes red. That is what this
stone gates.

## HOW THE WINDOW IS FORCED

`vis = 200 ms`, and the worker naps **350 ms between `check` and `emit`**:

```
A receives, checks -> Absent, sleeps
  t=200  visibility expires; B receives, checks -> Absent (A has not marked), emits, marks, acks
  t=350  A wakes, emits, marks (idempotent), acks (row gone -> Success no-op)
```

Deterministic on a 150 ms margin — the same margin the existing absorbed fixture already relies
on. **No new probe:** the nap exists, it only moves; the timing is proven by the sibling fixture.

## ⛔ THE ONE CONTRACT DECISION

**The test gates `distinct = 1` and REPORTS `total` and `dup`.**

I have written an observation as a gate three times in this arc, and corrected it in the last
SCORE: *a row must state what must HOLD, not what was last observed.* Applying it here rather
than describing it —

- **`distinct = 1` must hold.** The message produced at least one outcome. Loss reds the test.
- **`total = 2` today, and that is an observation.** If a later change makes the woken worker
  re-check and skip, `total` becomes 1 — **an improvement, not a regression.** A gate on `total=2`
  would red on a better design and pin the current one by accident.

★ The test's name carries its purpose: **`redelivery_mid_processing_never_loses`**.

## ⚠ ONE NAME IS WRONG AND THIS STONE EXPOSES IT

`mk-worker`'s `delay-ms` naps **after the mark, before the ack** — it models a slow ack. The new
nap models slow *work*. Two different delays cannot share one name, so: `delay-ms` →
`ack-delay-ms`, and the new one is `work-delay-ms`. **6 call sites, all in `circuit.wat`** — a
rename, not a migration.

## FILES

`wat-scripts/fanout/circuit.wat` (the rename, the new nap, one new fixture) and
`tests/services/probe_arc278_sane_circuit.rs` (one new test). **This test runs on the floor** —
it is deterministic and needs no chaos.

## OUT OF SCOPE = REJECTED

- **Changing the window's behaviour.** This stone *documents* the duplicate; making the woken
  worker re-check is a design change with its own trade-off and its own stone.
- **Touching either existing redelivery test.** They cover their cases correctly.
- Rung 3, and all perf work.
