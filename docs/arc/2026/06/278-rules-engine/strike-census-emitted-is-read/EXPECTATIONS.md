# EXPECTATIONS — the EMITTED ⇒ READ census gate, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the gate exists and passes | `cargo nextest run --release -E 'test(census_emitted_name_is_read)'` | green, with every one of the seven disposed |
| ⭐ it is MUTATION-PROVED | add a `census_count("probe:never-read")` to non-test `src/` | **RED naming that counter.** Then remove it |
| the sets are the sibling's | read the file | no second EMITTED/READ implementation |
| `phase_end` untouched | `git diff` | none of the 14 timing marks moved |
| readers assert NONZERO | read any disposition-1 test | a nonzero assertion, not `assert_eq!(x, 0)` |
| the split is reported | the SCORE | per name: reader / deleted / runed, each with its reason |
| floor | `scripts/floor.sh` | 5475 + new tests, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

60–90 min. The gate itself is small if the sibling's sets are reusable; the seven dispositions are
the work, and deletions need the floor to confirm nothing read them after all.

## Trap doors, named in advance

- **⛔ All seven runed.** That is a ratchet wearing a gate's clothes, and the arc has deleted
  several. If it happens, say so in the SCORE in those words rather than presenting it as a pass.
- **A second definition of EMITTED or READ.** The sibling's sets carry measured counter-examples —
  the computed `ebucket`/`tbucket` families, and two scope cuts each made against a false RED. A
  parallel implementation will drift from those and will red on a correct tree.
- **A zero-expecting reader.** `assert_eq!(count, 0)` cannot distinguish a deleted counter from a
  measured zero — the exact failure the sibling gate was built to stop, reproduced in its mirror.
- **A gate that cannot fail.** If no plausible new counter would red it, it is decoration. The
  mutation row above is the proof, and it must be a counter added to the ENGINE, not to a test.
- **Deleting a counter that something outside `kernel/tests` reads.** The READ set is scoped to
  cost tests; a benchmark or a `wat` script could read one too. Grep wider before any deletion.
