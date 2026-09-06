# BRIEF — measure whether census E's population is ever non-empty

Read `DESIGN.md` first. **This strike ships no engine change.** Its deliverable is a number and, if
that number is zero, one doc sentence. The tripwire is transient and must be gone at the end.

## Read in order

1. `src/rete/matcher.rs:538-552` — `alpha_match_inner_opts`. Line 547 is the `match:calls` bump
   (census E put it there); 548 is `let pat = alpha_pattern(cond)?;`. The `?` is what you replace
   with a `let … else` carrying the tripwire.
2. `src/rete/matcher.rs:429-453` — `alpha_pattern`. Read it so you can say in the SCORE what a
   `None` actually requires: a non-List, an empty List, or a Symbol head that is not a `FactBind`.
   Keyword-headed forms like `(:wat::rete::not …)` return `Some` and fail the later head check
   instead — that is why the branch may be narrow.
3. `src/rete/matcher.rs:481-525` — the three wrappers, so you can state that one tripwire covers
   all entry points.
4. `../strike-census-E-proof/REVIEW.md` — the STOP-1 finding this measures the cause of.

## Procedure

```
rm -f /tmp/arc278-alpha-pattern-none.log
# apply the tripwire (DESIGN has the exact shape)
./scripts/floor.sh                 # must stay GREEN
wc -l /tmp/arc278-alpha-pattern-none.log
sort /tmp/arc278-alpha-pattern-none.log | uniq -c | sort -rn | head -20
git checkout -- src/rete/matcher.rs
git diff --quiet src/rete/matcher.rs && echo CLEAN
```

Read the floor's **Summary line** from `.floor/latest/clean.log`, never a piped exit code.

## What to write

- The **line count**, quoted.
- If non-zero: the `uniq -c` output verbatim. Do not interpret which caller they come from — that is
  the next question (STOP-2).
- If zero: one sentence at `matcher.rs`'s census-E comment recording the measurement, its date, and
  the floor it was measured on — worded as **"never observed across the full suite"**, never
  "cannot happen". That distinction is the whole point of the sentence.

## Blast radius

`src/rete/matcher.rs` — transiently for the tripwire, then **one comment sentence at most**.
Nothing else. No test, no corpus, no counter.

## STOP triggers

1. Floor RED with the tripwire in → STOP; the tripwire is supposed to be behaviour-neutral.
2. File non-empty → report verbatim with counts and STOP; do not chase the caller in this strike.
3. `matcher.rs` not byte-identical to HEAD (modulo the one comment) at the end → STOP.
4. You are tempted to add a non-alpha `cond` anywhere → STOP. Disqualified on Honest; see DESIGN.

## Prior result to copy for shape

`../strike-census-E-proof/SCORE.md` — it reported a strike that did not land, in its own row 1,
without reframing the outcome. Same discipline here: a zero is a result, not a failure.
