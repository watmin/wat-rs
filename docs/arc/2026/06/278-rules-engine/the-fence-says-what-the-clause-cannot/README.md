# the fence says what the clause cannot

**Opened 2026-09-09.** Started as a bug report from the `compute` host (`sns-sqs` branch, 586 commits
divergent), became a question about **wat's expressivity** rather than about a missing check.

## The one-paragraph version

A trailing `(:wat::rete::where …)` whose predicate reads only one condition's bindings is a **beta
TestNode** — it filters *after* the joins are built. The same predicate written **inline** in that
condition is **alpha** — it filters at fact entry. On a three-way self-join the difference is
**C³ tuples materialised before the filter runs**, which is a self-DoS from a one-line mistake. The
compute host proposed refusing the hoistable fence. **Driving that refusal against our corpus proved
the two forms are not interchangeable — and that in nine cases the inline form is not
*expressible*.** The limiting factor is wat's clause grammar, not the author's judgement.

## Read in order

| file | what's in it |
|---|---|
| `FINDING-the-fence-says-what-the-clause-cannot.md` | the measurements — the C³ cost, the 24 failures and their four mechanisms, the Clara lab, and the gap in the expressivity corpus |
| `DESIGN-widen-the-clause-then-refuse-the-fence.md` | the proposed path, and why both obvious cures were rejected |

## Related, elsewhere

- `strike-where-fence-hoistable/` — the check, built and reverted **twice**. Recoverable at
  `f47a9fccc`.
- `strike-hoist-the-corpus/` — the codemod (`wat-scripts/fixes/hoist-where-into-condition.wat`),
  proven and idempotent, applied and reverted when the floor found the gap.
- `compute:docs/excursus/2026/08/001-sns-sqs/the-topic-forgets-its-subscriber-count/` — the origin,
  including `FINDING-the-finder-hoists-nothing.md`.
