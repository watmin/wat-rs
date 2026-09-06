# BRIEF — prove census E; gate `match:calls` and `compiled:exec`

Read `DESIGN.md` first, especially THE TRAP. **The mutation is the deliverable, not the
assertions** — three `assert_eq!`s that cannot fail would leave census E exactly as unproven as it
is now.

## Read in order

1. `src/rete/kernel/tests/alpha_discrimination.rs:360-390` — the compiled loop. `calls += 1` per
   `exec_compiled` call, inside `with_count_census` producing `compiled_rows`.
2. `:396-414` — the interpreter loop. `interp_calls += 1` per `alpha_match_inner` call, inside
   `with_count_census` producing `interp_rows`. Same `facts`, same `alpha_by_type`.
3. `:417-422` — the existing `get` helper (`unwrap_or(0)`). Reuse it; absence reads 0 and a nonzero
   hand count catches it.
4. `:334-345` — the test's doc comment. It gains a second row describing what you add.
5. `src/rete/matcher.rs:547-548` — census E's move: the bump, then `let pat = alpha_pattern(cond)?;`.
   This is what you transiently revert for the mutation.
6. `src/rete/compiled_cond.rs:956-963` — the other half of the parallel, for the comment you write.

## Sketch

```rust
// ROW 3 — the two call counters are live, count every invocation, and are comparable.
assert_eq!(
    get(&interp_rows, "match:calls"), interp_calls,
    "match:calls is {} but the loop made {interp_calls} calls — the counter misses invocations \
     (census E moved it above `alpha_pattern`'s `?` so it would not)", get(&interp_rows, "match:calls")
);
assert_eq!(get(&compiled_rows, "compiled:exec"), calls, "…");
assert_eq!(calls, interp_calls, "…same corpus, so the two counters compare…");
```

## The mutation — this is the strike

1. With the assertions in place, **revert census E's move**: put `census_count("match:calls")` back
   *below* `let pat = alpha_pattern(cond)?;`.
2. Run the test.
3. **Expected RED** on the `match:calls` assertion, with the two numbers differing. Quote it verbatim.
4. Restore the move; confirm green.

If step 3 comes back **GREEN**, that is STOP-1: the corpus contains no `cond` that fails
`alpha_pattern`, the assertion cannot prove census E, and you should report what the corpus holds
(how many distinct `cond`s, and whether any is a non-alpha form) rather than adding facts to force
a red.

## Blast radius

`src/rete/kernel/tests/alpha_discrimination.rs` only — three assertions and a doc row. **No engine
change. No new counter. No corpus change.**

## STOP triggers

1. The mutation does not RED → STOP and report the corpus contents. Do not add facts to force it.
2. `match:calls != interp_calls` at unmutated HEAD → STOP, report both numbers; something else
   bumps inside the window.
3. `calls != interp_calls` → STOP; the loops do not share a corpus.
4. You find yourself editing `matcher.rs` other than transiently for the mutation → STOP.

## Prior result to copy for shape

`../strike-census-B-compiled-calls/SCORE.md` — starred rows, the live mutation quoted verbatim from
both sides, and the honest delta named.
