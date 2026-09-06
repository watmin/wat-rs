# EXPECTATIONS — `retract` removes ONE occurrence

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the redrawn axis is GREEN | `#grid/Verdict … :accuracy :match :oracle-accuracy :match :port-accuracy :match`, quoted raw |
| 2 ★ | before/after both recorded | pre-cure wat `[1 2]` vs clara `[0 1 2]`; post-cure both `[0 1 2]` |
| 3 ★ | nothing else moved | no value change outside the axis — the DESIGN's blast-radius claim, tested |
| 4 ★ | the old door is GONE | `grep -rn "remove-every-equal" .` → hits only in docs/history |
| 5 ★ | floor | **≥ 5465, and tell me the final number and exactly what moved it.** New arms are a PASS, not a failure — say which they are |
| 6 | `want_n` bumped | `CORRECTNESS_SIZES` 2 → 3, the behaviour change witnessed on the floor |
| 7 | order preserved | `remove-one` keeps every surviving fact's relative position |
| 8 | the fuzzer comment struck | `final-facts` sentence gone; **no other change** to that file |
| 9 | clippy | rc=0 |

★ load-bearing. **Row 1 is the cure's acceptance test; row 3 is what says the cure is precise.**

⚠ Row 5 is written as a direction, not an equality, deliberately: twice this session I pinned a
count that the same document's other rows made unreachable. A row is only a test if some achievable
result fails it and some achievable result passes it.

## Trap doors, named in advance

- **A last-occurrence or reordering fold.** `fire.wat:316-322`'s convergence proof rests on the
  sub-multiset property; a fold that reorders turns a termination proof into a false fixpoint —
  *"a silent wrong answer, worse than the defect."*
- **Curing derived multiplicity too.** Clara will still be `[0 1 1 2 2]` on the OLD seed and that is
  correct. Only the redrawn seed should match.
- **Landing the axis without the cure**, or the cure without the axis — either pushes a red
  (`ci.yml:262`).
- **Touching the TMS fuzzer beyond the one comment sentence.**
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A verdict claimed without the raw `#grid/Verdict` line.
- Any value moving outside the redrawn axis, unexplained.
- `remove-every-equal` still reachable in code.
- A fold that is not order-preserving.
- The axis and the cure in separate commits.
