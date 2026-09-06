# DESIGN — prove census E, and gate two counters that nothing watches

Census E (`c91121e5e`) moved `census_count("match:calls")` above `alpha_pattern`'s `?` so it counts
every invocation. **That cure is unproven and its SCORE says so.** This strike proves it, and closes
the hole the proof exposes.

## The hole

`match:calls` has exactly TWO sites in the tree: the bump (`matcher.rs:547`) and one read
(`fanout_cost.rs:217`) on a world where it is 0. `get` is `unwrap_or(0)`, so absent and zero are the
same value, and the consumer does not assert on the key. **Deleting the counter outright would
change nothing observable.** It is ungated in both directions — nothing proves it is alive, and
nothing would notice if it died.

Census E therefore landed a correct fix into an unobserved counter: better prose, no proof.

## What is already on disk

`src/rete/kernel/tests/alpha_discrimination.rs` drives **both** paths over the **same** corpus
(`facts` × `alpha_by_type`), inside `with_count_census`, hand-counting each loop:

| | loop hand count | census rows | counter |
|---|---|---|---|
| compiled (`:362-390`) | `calls += 1` | `compiled_rows` | `compiled:exec` |
| interpreter (`:396-414`) | `interp_calls += 1` | `interp_rows` | `match:calls` |

`alpha_match_inner_opts` is not recursive and has three thin wrappers; the census window is the
loop alone. So **one hand-count tick per bump, exactly.** Everything needed is already computed and
thrown away.

## What ships — three assertions

1. `interp_rows["match:calls"] == interp_calls` — the counter is alive AND counts every invocation.
2. `compiled_rows["compiled:exec"] == calls` — same claim for the other half.
3. `calls == interp_calls` — the two loops walked the same corpus, so the two counters are directly
   comparable.

Together these **realize the interpreter-vs-compiled call differential** that has been prose since
census B, in the one place that already holds both numbers.

Absence is caught for free: a missing key reads 0 through `unwrap_or(0)`, and 0 ≠ a nonzero hand
count.

## ⛔ THE TRAP — and it is the whole strike

Assertion 1 proves census E's widening **only if the corpus contains at least one `cond` that fails
`alpha_pattern`.** If every cond in this world is an alpha pattern, `match:calls == interp_calls`
holds both before and after E's move, and the gate passes for a reason that has nothing to do with
what it claims to prove — a gate with one possible outcome.

**So the proof is a mutation, not an assertion.** Revert E's move (put the bump back below the `?`),
run the test, and assertion 1 **must go RED**.

- **RED** → the corpus does contain a non-alpha `cond`, E's widening is real and now gated. Restore
  the move; done.
- **GREEN** → the corpus has no such `cond`. The assertion cannot prove E. **STOP-1**: report it and
  say what the corpus contains. The cure is then to add a non-alpha `cond` to the corpus so the
  population E widened is actually represented — but that is a corpus decision, not a silent
  addition, so it comes back here first.

## THE ONE CONTRACT DECISION

**The assertions go where the numbers already are** — inside
`compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, as a clearly-labelled second row,
with the test's doc comment gaining that row. Extracting a separate test would duplicate the corpus
construction for no gain, and this tree already writes multi-row tests (row 1 / row 2 / STOP-3).

## Out of scope = REJECTED

- Changing the corpus. If STOP-1 fires, that comes back as its own decision.
- Touching census E's move itself (except transiently, as the mutation).
- Any engine change. This strike adds test assertions only.

## STOP triggers

1. The mutation does not RED → **STOP and report.** The gate would be vacuous and the corpus is the
   reason; do not add facts to the corpus to force it without saying so.
2. `match:calls != interp_calls` at HEAD (unmutated) → STOP and report both numbers. Something else
   bumps the counter inside the census window and the DESIGN's "one tick per bump" is wrong.
3. `calls != interp_calls` → STOP. The two loops do not walk the same corpus and the differential
   claim is false.
