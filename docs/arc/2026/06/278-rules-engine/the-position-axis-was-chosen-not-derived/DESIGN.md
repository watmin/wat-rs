# DESIGN — type-check `accumulate`'s `:from` clauses

## Why

`src/rete/validate/mod.rs:296`, three arms below the `where` fence arm cured at `6f1d93451`,
carrying the same `Design call 3` sentence:

```rust
// Design call 3 — accumulate's `:from` inner gets fact-type-HEAD validation only;
// its own clauses and the acc-form's reducer body are out of scope.
ReteClauseShape::Accumulate { from, .. } => {
    validate_fact_type_head_only(from, rule_name, types, errors);
}
```

`validate_fact_type_head_only` (`typing.rs:146`) checks one thing: is the fact type registered.
Driven at `07eba8226` — `string::=` over an `i64`-bound var:

| position | verdict |
|---|---|
| inside `accumulate`'s `:from` | ⛔ **ACCEPTED**, starts up clean, rc=0 |
| the identical predicate in a plain condition | ✅ `ConstraintTypeMismatch` |

Fourth instance of one class — D10 (`:then` RHS), D11 (nested `:then`), the `where` fence, this.
**Tracked nowhere** (grepped `RETE-OPEN-WORK.md` and the arc's work lists).

⚠ **This is the VALIDATOR's type check, not `reachability.rs`'s reachability ledger.** Those are
different instruments answering different questions, and the FINDING's first draft conflated them;
read its amendment before reasoning about the ledger. Nothing here touches `reachability.rs`.

## ⛔ THE ONE CONTRACT DECISION

> **`:from`'s inner gets the SAME full validation `not`/`exists` already get — `validate_when_entry`
> recursion — not a bespoke clause-type-check.**

`Not`/`Exists` sit one arm above and do exactly this:

```rust
ReteClauseShape::Not(inner) | ReteClauseShape::Exists(inner) => {
    validate_when_entry(inner, rule_name, types, binds, errors);
}
```

Driven and confirmed correct: `not` inner and `exists` inner both refuse the ill-typed predicate.
`:from`'s inner **is** a condition — the same shape those wrap. Giving it a second, parallel
validation path would be a second grammar to drift, which is the defect `classify_rete_clause`
exists to prevent.

⚠ **The rejected alternative:** reuse `check_fence_interior` (yesterday's cure). It walks an
*expression*; `:from` holds a *condition*. Same category error as the ledger framing struck in the
FINDING.

## ⛔⛔ THE TRAP THAT MAKES THIS BIGGER THAN THE FENCE CURE

⛔ **CORRECTED 2026-09-10 — THIS CLAIM WAS FALSE, AND IT WAS MY GREP.** I searched for
`rete::accumulate`; the form has no such keyword (`clause.rs:67`: `(?result <- (<acc-form>) :from
(<inner>))`). Measured properly: **37 `.wat` files carry a `:from` accumulate condition, nineteen
of them GRID CELLS with Clara twins.** The trap below still holds for a different and better
reason — see the FINDINGs — but "no users" was never the reason.

The fence cure had `legal_fences_still_compile` — an over-rejection guard built from forms the
corpus already proved legal. **Here no such corpus exists.** Widening validation on a surface with
no users means:

- **no regression signal.** A cure that over-rejects legal accumulate rules reds nothing, because
  nothing uses them. The floor stays green while the surface silently narrows.
- **the positive corpus must be BUILT, not harvested.** That is this strike's real work and its
  largest risk — it requires knowing what a legal `accumulate` looks like from the *spec*, not from
  usage.

⭐ **So the deliverable is two halves, and the second is the load-bearing one:** the check, and a
positive fixture set of legal `accumulate` rules that must keep compiling. D10's header names this
exact failure on the `:then` side — *"a cure that refuses every operand it cannot type passes every
refusal probe and still stops a corpus of legal rules from compiling."*

## Files

| file | change |
|---|---|
| `src/rete/validate/mod.rs:296` | the arm recurses; its comment is rewritten, not deleted |
| `tests/rete/` | the probe: one refusal fixture, and the positive corpus |

## Out of scope = REJECTED

- **The accumulate REDUCER body** — the other half of that same comment. Not measured in the
  FINDING and not cleared; it needs its own driven finding first, not a guess bundled here.
- **`reachability.rs` and its `CallSite` axis.** Deliberately narrow, argued at the site, tracked
  in `RETE-OPEN-WORK.md:432` with the rule for expanding it. Untouched.
- **Minting corpus uses of `accumulate`.** Test fixtures are not corpus adoption.
