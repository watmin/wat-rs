# DESIGN — what does publish actually cost

**A measurement stone. No production change.** `wat-scripts/scratch-pad/` only.

## WHY — we optimized what the instrument could see; the rest is unattributed

Last stone took `publish` 51.3 s → 37.3 s and named the leader. But:

```
count-index   8000 × 467 µs  ≈  3.7 s
publish       37.3 s
UNATTRIBUTED  ~33.6 s
```

★ **We know what 11 % of publish is. We are guessing about the other 89 %.** The two standing
candidates — the `Record` rebuild on `receive`/`ack`, and select+timer per client call — are
**hypotheses**. This arc has been punished four times for picking a fix before measuring; the
next optimization must be aimed by a number, not a suspicion.

## ⛔ THE ONE CONTRACT DECISION

**Measure the primitives in isolation, then check whether unit × count explains the wall.**

That is exactly what closed the last case: the stage histograms (in-band) localized the
regression to `outbox`, and `probe-what-a-scan-costs.wat` (unit cost) quantified it. Neither
alone would have.

★★ **No production edit.** A stone that touches the hot path to measure the hot path cannot
report a clean number, and a measurement that costs a rebuild-and-revert cycle is one nobody
re-runs. This is a probe.

## WHAT TO MEASURE — five units, and one of them is the arc's terminal condition

| unit | why |
|---|---|
| **a bare service round trip, THREAD locus** | ★★★ **the interpretation + dispatch floor.** Every client call pays it. This is the number that says how close we are to *"interpretation is the leader"* |
| **a bare service round trip, PROCESS locus** | the same, plus IPC and framing. The circuit's queues are process locus |
| `Store/put`, one row, sqlite, at depth | **completely unmeasured**, and on the publish path 8000 times |
| `Store/count-index` | 467 µs measured — re-measure alongside so all five share a box and a run |
| `Store/scan-index` limit 1 | 504 µs — the query-with-materialization floor, for contrast |

★ **The bare round trip is the point of the stone.** If it is ~400 µs, then 8000 publishes × 2
calls ≈ 6.4 s of pure dispatch, and every future optimization is bounded by it. If it is ~50 µs,
dispatch is noise and the cost is in the store.

## THE ARITHMETIC THIS STONE MUST ATTEMPT

```
predicted  =  8000 × (put + count-index + 2 × round-trip)
actual     =  37.3 s
gap        =  actual − predicted        ← REPORT IT, whatever it is
```

⚠ **The gap is the deliverable, not the residue.** A small gap means the model is right and the
next stone aims at the largest term. A large gap means the cost is somewhere we have not thought
to look — and *that* is a finding worth more than the units.

## FILES

`wat-scripts/scratch-pad/probe-what-publish-costs.wat` (new).
`probe-what-a-scan-costs.wat` may be extended rather than duplicated — one harness is better.

## OUT OF SCOPE = REJECTED

- **Any production change.** No `wat/`, no `sqs.wat`, no `circuit.wat`, no `src/`.
- **Fixing anything.** This stone aims the next one; it does not fire.
- **In-band publish-path stamps.** A real option, and the fallback if the arithmetic does not
  close — but it edits the hot path, so it is only worth its cost once the unit numbers say the
  model is wrong.
- **Compiled wat.**
