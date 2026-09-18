# REVIEW — the axis is built as briefed, and the DESIGN was wrong. PARKED for strike 2.

**Verified independently:** rows 1, 2, 5, 6, 7 HOLD. Row 4 HOLDS — I read
`.floor/2026-09-06T07-21-21Z/` myself: `Summary [ 459.017s] 5460 tests run: 5460 passed (2 slow),
21 skipped`, zero FAIL lines. The Clara twin is honest (`(sort (map :?k …))`, no `distinct`), and
the executor documented the confound in its own `.wat` header rather than hiding it.

**Row 3 is the wrong row, and that is the orchestrator's fault.** The EXPECTATIONS graded on the
verdict STRING and never asked the mutation question: *would this verdict differ if the defect
were absent?* Driven at HEAD:

| state | wat | Clara |
|---|---|---|
| **zero retracts** | `[0 1 2]` | `[0 0 1 1 2 2]` |
| as shipped (retract F(0) once) | `[1 2]` | `[0 1 1 2 2]` |

The axis reports `MISMATCH` **before `retract` is ever called**. Its verdict is dominated by the
derived-multiplicity divergence — the one CLOSED AS JUSTIFIED — so it has one possible outcome
whether or not the retract defect exists. A proof with one possible outcome is not a proof.

**Consequence the SCORE gets wrong.** *"After that, this axis's three-way goes green and CI
three-way goes green with it"* is false. Post-cure wat is `[0 1 2]`, Clara `[0 1 1 2 2]` — still
`MISMATCH`, permanently, and `ci.yml:262` runs the three-way with no args. The single Clara line
`[0 1 1 2 2]` carries **one instance of each divergence pointing opposite ways**: the `1 1`/`2 2`
is Clara over-deriving (wat right, closed), the `0` is the retract witness (wat wrong).

## The redraw — driven, both engines

**Duplicate ONLY the retracted key.** `F(0)×2`, `F(k)×1` for k>0, `G(k)×1`; retract `F(0)` once:

```
wat / oracle : [1 2]      <- Out(0) destroyed
clara        : [0 1 2]    <- Out(0) survives
```

Sole witness is key `0`; zero contamination from the justified split. Post-cure wat becomes
`[0 1 2]` == Clara → the axis goes **green** and becomes the permanent regression gate.

## Disposition

**PARKED, not discarded.** The files stay in the tree uncommitted. They land in **strike 2**
(`remove-one` + this redraw + the TMS fuzzer's model re-derived from Clara), together with the
cure, because the axis alone reds a CI gate and the cure alone has no acceptance test. Strike 1
(`strike-factbag-one-owner/`) is behaviour-neutral and goes first.

## Also settled here, so strike 2 does not relitigate it

Driven: `acc::count` over `F(0)` inserted twice returns **2**; inserted once, **1**. Input
multiplicity is **wat's own observable semantics**, not an artifact of Clara's bookkeeping — the
engine can COUNT the duplicate but cannot REMOVE it. That is the argument for the cure, and it
needs no external reference. `BRIEF-STONE-4c-truth-maintenance.md:46-50` shows the origin: retract
was specified as *"mirror `merge-facts`"*, and `merge-facts` is set-semantic for a **termination**
reason that does not reach retraction. Inherited, never decided.
