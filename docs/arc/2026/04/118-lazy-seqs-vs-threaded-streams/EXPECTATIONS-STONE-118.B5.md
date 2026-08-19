# EXPECTATIONS — STONE 118.B5 · native drain, wat oracle

Written before the strike. Rows 6–8 are the ORCHESTRATOR's; the rider does not touch the floor.

| # | what | who | expected |
|---|---|---|---|
| 1 | ★ differential `native ≡ spec` | rider | agree on Vector + PersistentVector receivers, empty / 1 / many |
| 2 | ★ non-vacuity of the differential | rider | perturbed → RED; reverted → byte-identical |
| 3 | ★ retention stays FLAT | rider | ≈0.38 B/elem across 100k→800k, as B3 measured |
| 4 | the drain closes most of the 44× | rider, the bench | drain-only falls well below 529ms; native concat unmoved at ~12ms |
| 5 | `into`'s clause arms untouched | rider, `git diff` | `wat/seq.wat:166`–onwards unchanged but for the spec rename |
| 6 | floor | **orchestrator** | ≥4760 run, 0 FAIL, 19 skipped |
| 7 | clippy | **orchestrator** | 0 |
| 8 | ignores | **orchestrator** | 13 |

**Rows 1 and 3 are load-bearing together.** Row 1 alone passes on a native that drains correctly and
retains everything — which is precisely the O(n) memory B3 deleted four stones ago. Row 3 alone
passes on a native that is lazy and wrong. Neither is the stone without the other.

## Independent prediction

**60–90 minutes.** The natives are a realize-loop each; the differential and the retention drive are
the bulk. The purity ledger is two entries, now that the gate is known in advance.

## Trap-doors named in advance

- **A native that collects the whole chain into a Vec while ALSO holding the head alive** passes the
  differential and fails row 3. That is the exact shape B3 deleted; row 3 is the only instrument that
  sees it.
- **The oracle must stay wat.** If `stream->pvec-spec` is "simplified" into a call to the native, the
  differential becomes a tautology. `[[feedback_an_oracle_must_be_written_in_the_other_language]]`
- **Two purity gates, no link between them.** Satisfying `is_pure_total` does nothing for
  `rete::purity::completeness_gate`. Both, or the floor goes red on the second.
- **`dorun` is out of scope** and will look tempting — it is `(into [] coll)` with the result binned,
  and this stone makes that waste cheaper without removing it. Leave it; it is tracked.
