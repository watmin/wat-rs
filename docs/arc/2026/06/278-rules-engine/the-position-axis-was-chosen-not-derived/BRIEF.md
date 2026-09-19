# BRIEF — type-check `accumulate`'s `:from` clauses

## The work

Make `src/rete/validate/mod.rs:296`'s `Accumulate` arm validate its `:from` inner the way the
`Not`/`Exists` arm one above it already does, so an ill-typed predicate inside `:from` is refused
exactly as the same predicate is refused in a plain condition — **and build the positive fixture
corpus that proves the widened check does not over-reject**, because no such corpus exists.

**Read `DESIGN.md` beside this file first.** It pins the one contract decision and names the trap
that makes this bigger than it looks.

## Read in order

| room | why you are here |
|---|---|
| `.../the-position-axis-was-chosen-not-derived/DESIGN.md` | the contract decision, the zero-corpus trap, what is affirmatively out of scope |
| `.../FINDING-the-position-axis-was-chosen-not-derived.md` | ⛔ **read its AMENDMENT** — the first draft conflated the validator with `reachability.rs`'s ledger and was struck |
| `src/rete/validate/mod.rs:288-300` | the two arms side by side. `Not`/`Exists` recurse; `Accumulate` does not. That difference is the whole strike |
| `src/rete/validate/typing.rs:146` | `validate_fact_type_head_only` — exactly what the arm does today: one registry lookup |
| `src/rete/clause.rs:67` | `Accumulate { from, .. }` and the `(?result <- (<acc-form>) :from (<inner>))` shape |
| `tests/rete/probe_arc278_fence_interior_types.rs` | ⭐ the shape to copy — yesterday's cure for the sibling arm. Three tests: the gap, the control that gives it meaning, the over-rejection guard |
| `tests/rete/probe_arc278_D10_then_field_types.rs` | its header's *"⛔ Why the not-knowable fixture is the load-bearing one"* — the same trap, on the `:then` side, stated better than I can state it |

## Sketch

```rust
// mod.rs:296 — the arm recurses. The comment is REWRITTEN, not deleted: it currently
// asserts the inner's clauses are out of scope, which is the claim being retired.
ReteClauseShape::Accumulate { from, .. } => {
    validate_when_entry(from, rule_name, types, binds, errors);
}
```

That is likely the entire source change. **The fixtures are the strike.**

## The positive corpus — this is the deliverable, not the check

Build a fixture of **legal** `accumulate` rules that must compile and fire, derived from the spec
and the acc-form vocabulary rather than from usage (there is none). At minimum it should cover a
`:from` whose inner carries: a plain bind, a well-typed inline constraint, a join variable bound in
an earlier condition, and an operand whose type is **not knowable** (`OperandType::UnboundInThisRule`
/ `ComputedNotDerivableHere`). That last one is what catches a cure that refuses what it cannot type.

## Blast radius

`src/rete/validate/mod.rs` (one arm + its comment), and new fixtures under `tests/rete/`. No change
to `typing.rs`, to `clause.rs`, to `reachability.rs`, or to any `.wat` under `wat-scripts/`.

## STOP triggers

Partitioned — check, corpus, scope. Every case falls in exactly one.

**STOP-1 (the check) — recursion refuses something `not`/`exists` accept.** `validate_when_entry`
was written for a `:when` entry; `:from`'s inner may differ in a way the arm's original author
knew about and I do not. If recursion produces an error on a rule you believe legal, STOP and
report the rule and the error — do not special-case your way past it. That reaction would be the
"second grammar" the DESIGN rejects.

**STOP-2 (the corpus) — you cannot construct a legal `accumulate` rule at all.** The form has zero
corpus uses; if the spec and the acc-form rows are not enough to write one that compiles and fires
TODAY, before your change, then STOP. A refusal probe with no positive control is exactly the
failure D10's header describes, and shipping the check without it is worse than not shipping.

**STOP-3 (scope) — the fix seems to need the reducer body, or `reachability.rs`.** Both are
affirmatively out of scope in the DESIGN. The reducer body has not been measured and needs its own
driven finding; `reachability.rs`'s narrow axis is deliberate, argued at the site, and tracked at
`RETE-OPEN-WORK.md:432`. Report and stop.

## The open question — answer it either way

The `Accumulate` arm's comment says the inner gets *"fact-type-HEAD validation only"*, deliberately,
as `Design call 3`. **I do not know why, and I am not asserting it was a mistake.** The sibling
`Where` arm carried an identical-looking justification that turned out to be a genuine hole — but
the `reachability.rs` exclusion carried one that turned out to be **correct and tracked**, and I
called it a defect before reading the record.

So: find out whether `Design call 3` recorded a REASON for excluding `:from`'s clauses. Check the
arc's design docs for it. **If there is a stated reason and it still holds, say so and stop — that
verdict is worth more than the cure.**

## Working rules

- `cargo nextest run --release`, never `cargo test`. One cargo build at a time.
- Commit only on green. Stage explicit paths. `git commit -F -` with a quoted heredoc.
- ⛔ Run `./scripts/floor.sh` and **read its result yourself**. If your tooling backgrounds it, poll
  to completion — match on process name (`ps -eo comm`), never a `pgrep -f` pattern that appears in
  your own command line, which matches your own shell and waits forever.
- **Your final message is incomplete unless it quotes the `Summary [...]` line from
  `.floor/latest/clean.log` verbatim, with that file's `FAIL` count.**
- On any red: do NOT re-run. Copy the failing test's whole stdout and stderr, name the arm, surface it.
