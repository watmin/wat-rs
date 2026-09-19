# DESIGN — census D: `match:key-alloc` is not a `match:` quantity

Census audit section D (`../vigilia-2026-09-05/recon/census-name-audit.md:65-75`), re-grounded at
HEAD 2026-09-06.

## ⚠ Severity, stated honestly first

**Latent. No live number is wrong.** Both consumers are safe, and safe for a reason:

- `fanout_cost.rs:215-229` reads it over a WHOLE FIRE and asserts **exactly 0**. A union counted to
  zero means zero from each member, so the conflation cannot mislead. Its own label already says
  *"(RHS + alpha, both compiled — expect 0)"*, and it carries a `prod:derivations == 40_000`
  non-vacuity guard so a dead fire cannot pass as proof.
- `alpha_discrimination.rs:423-424` arms the census around **direct matcher calls only**, asserts
  `compiled == 0`, and uses the interpreter's nonzero reading purely as a `> 0` liveness check —
  never attributing it to a mechanism.

This strike is therefore about a **name and two false comments**, not a wrong measurement. It is
worth doing because every performance claim in this arc rests on these counters, and because the
trap is already one step away: the moment a whole-fire read expects a NONZERO count, the `match:`
prefix will invite an attribution the population does not support.

## The finding

Two bump sites, both in `matcher.rs`: `:679` (the Bind arm) and `:930` (inside `resolve_operand`).
But `resolve_operand` has **three** callers, two of which are not matching:

- `src/rete/eval_insert.rs:287` — `resolve_rhs_value`, once per `?var` per derived fact (`:then` RHS)
- `src/rete/step_payload.rs:45` — step-payload rendering

So the population is *"binding-key `String` allocations anywhere `resolve_operand` runs"*. The
`match:` prefix names one of three callers.

**And a comment states the opposite.** `eval_insert.rs:154-155`:

> *"A String allocated per derived fact for a class name fixed at compile time — NOT counted by
> `match:key-alloc`, which arms only the two resolve_operand sites."*

Read in `eval_insert.rs` — the file whose own `resolve_rhs_value` bumps the counter — that sentence
tells the reader the RHS path is outside the counter's population. It is inside it.

## THE ONE CONTRACT DECISION

**Rename to `bindkey:alloc`. Do NOT thread provenance through `resolve_operand`.**

The attribution split (a `census_key: &'static str` parameter, so each caller names itself) is
**affirmatively rejected here**, on the tree's own stated principle — `accum_cost.rs:85-92`, C10:

> *"Discriminating the arms is still a hot-path engine edit for an instrument's benefit."*

`census_count` is a release no-op (`#[cfg(not(test))] fn census_count(_name) {}`), so the cost
argument is weaker here than in C10's case — but the principle is the tree's and this strike does
not have the evidence to overturn it: **no consumer wants the attribution today.** A future one
that does should overturn C10 deliberately, with the need on the table, not inherit an engine
signature change made speculatively.

`bindkey:alloc` is true of all three callers, and a reader who needs attribution now sees a name
that does not promise it.

## Why renaming is right here and splitting was right for B

Census B split because **two mechanisms lived at two different sites** — the names could be made
true for free, and a gate wanted to tell them apart. Here one mechanism (rebuilding a binding-key
`String`) lives at one site with three callers, and **no gate wants to tell the callers apart**.
Splitting would buy nothing and cost an engine signature. The cure is to stop the name claiming a
provenance the counter never had.

## Out of scope = REJECTED

- Threading provenance through `resolve_operand` (above — and record the rejection at the counter's
  declaration so the next reader finds the reasoning, not a fresh temptation).
- Any engine behaviour change. This strike changes a `&'static str` and comments.
- Census E–M. One section per strike.

## STOP triggers

1. Any measured value changes → STOP; this is a naming strike.
2. A consumer turns out to depend on the `match:` prefix semantically (not just the string) → STOP
   and report it; the DESIGN's "no gate wants attribution" claim would be wrong.
3. The rename tempts you into the provenance parameter → STOP. That is the rejected option and it
   needs the builder, not this strike.
