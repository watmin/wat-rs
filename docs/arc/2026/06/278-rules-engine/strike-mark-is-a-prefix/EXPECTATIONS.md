# EXPECTATIONS — D1: the mark is a prefix length

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the break attempt is REPORTED | either a driven violation with its reading, or an explicit "I could not construct one, and here is what I tried" |
| 2 ★ | the catch-up is tail-only | it indexes `right[already..]`; no writer ignores the mark |
| 3 ★ | the PREFIX property is asserted | the test checks the indexed elements are `right_elements[0..mark]`, not only that a count equals a sum |
| 4 ★ | every existing guard survives | the applicability checks, the ★ two-writers-met check, and `a_single_hashjoin_shape_is_refused_as_inapplicable` all still pass and are unweakened |
| 5 | census numbers | unmoved, or the move is surfaced as a finding (STOP-1) |
| 6 | floor | **0 failed** |
| 7 | clippy | rc=0 |
| 8 | blast radius | `src/rete/kernel/` only |

★ load-bearing. **Row 6 is the deliverable.**

## Trap doors, named in advance

- **Weakening the existing test to fit the new assertion.** Its header records two prior corrections;
  it is the most carefully built probe in this arc. Add, do not subtract.
- **A prefix assertion that is also free.** If it holds by construction after the tail-only cure,
  say so — and then its value is regression cover, not proof. Do not present it as a proof it is not.
  Mutation-prove it: revert the catch-up to the whole-memory walk and show the assertion reddens.
- **Re-opening bucket internals.** STOP-3.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- The break attempt unreported.
- The catch-up left pushing the whole memory while the test is adjusted to tolerate it.
- Any existing guard weakened or deleted.
- A new prefix assertion that cannot redden under the reverted catch-up, presented as a proof.
- A red floor of any size.
