# BRIEF — census D: `match:key-alloc` → `bindkey:alloc`

Read `DESIGN.md` first, especially the severity note (this is latent — no live number is wrong) and
the ONE CONTRACT DECISION (the provenance parameter is **rejected**, not deferred).

A rename plus two comment corrections plus one recorded rejection. **No engine behaviour changes.**

## Read in order

1. `src/rete/matcher.rs:679` and `:930` — the two bump sites. `:930` is inside `resolve_operand`;
   `:679` is the Bind arm. Both become `"bindkey:alloc"`.
2. `src/rete/eval_insert.rs:287` (`resolve_rhs_value`) and `src/rete/step_payload.rs:45` — the two
   NON-matcher callers of `resolve_operand`. They are why the `match:` prefix is false. Neither
   changes; read them so you can write the counter's doc truthfully.
3. `src/rete/eval_insert.rs:154-155` — **the false comment.** It tells the reader the counter
   *"arms only the two resolve_operand sites"*, in the very file whose `resolve_rhs_value` bumps it.
   Rewrite so it says what is true: the class-name String is NOT counted, AND this file's own RHS
   resolution IS, through `resolve_operand`.
4. `src/rete/kernel/tests/alpha_discrimination.rs:334-338` — accurate about the SITES, but it is
   accurate only because this test arms the census around direct matcher calls. Add that condition,
   so the next reader knows the attribution is a property of the harness and not of the counter.
5. `src/rete/kernel/tests/fanout_cost.rs:210-229` — already labels the row *"(RHS + alpha, both
   compiled — expect 0)"*. Update the name; keep the label's honesty and the `prod:derivations`
   non-vacuity guard exactly as they are.
6. `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` — your safety net: a name read by a
   cost test but emitted nowhere is a build failure, so a missed reader cannot become a silent zero.

## Where the rejection gets recorded

At the counter's declaration (the `matcher.rs` bump site's comment), write down that per-caller
attribution was considered and rejected, with the reason from the DESIGN: no consumer wants it, and
C10 (`accum_cost.rs:85-92`) holds that discriminating for an instrument's benefit is an engine edit.
A future reader who needs attribution should find the reasoning, not re-derive the temptation.

## Blast radius

`src/rete/matcher.rs` (two literals + the declaration comment) · `src/rete/eval_insert.rs` (one
comment) · `src/rete/kernel/tests/alpha_discrimination.rs` (reads + one doc) ·
`src/rete/kernel/tests/fanout_cost.rs` (reads + label). **No `.wat`, no behaviour, no new counter,
no signature change.**

## Mutation proof

One counter, two bump sites, and the consumers assert ZERO — so a deleted bump cannot red them
(zero is what they expect). Say that plainly rather than inventing a proof:

- **Show the counter is live**: quote `alpha_discrimination.rs`'s interpreter reading
  (`interp_key_allocs > 0`) under the new name. That assertion exists precisely so a zero elsewhere
  is not vacuous, and it is the one place the counter's liveness is gated.
- **Show the rename is complete**: `grep -rn '"match:key-alloc"' src/ tests/` → no hits.

If you find a way to red a consumer by deleting a bump, report it — it would mean a consumer is
load-bearing in a way this DESIGN did not find.

## STOP triggers

1. Any measured value changes → STOP; naming strike only.
2. A consumer depends on the `match:` prefix semantically → STOP and report.
3. You reach for the provenance parameter → STOP. Rejected in DESIGN; it needs the builder.

## Prior result to copy for shape

`../strike-census-B-compiled-calls/SCORE.md` — starred rows, raw evidence, honest delta, and the
first-floor arms captured rather than re-run.
