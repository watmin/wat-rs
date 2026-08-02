# NOTE — two findings from the `where-numeric` corpus family (2026-08-01)

The numeric family was expected to be the richest defect hunt in the fleet, because wat and Clojure
are most likely to genuinely diverge on integer sign semantics. **They did not diverge** — 10/10 rows
byte-identical, including negative operands through `quot` / `rem` / `mod`, a negative divisor, and a
row (`rem-mod-diverge`) whose whole predicate is `rem(a,6) != mod(a,6)`, pinning the sign difference
between the two as a live, agreed-upon behaviour rather than a comment.

That is a real result: the clj-parity engineered in the arc-278 numeric-tower stone (`quot` truncates,
`rem` takes the dividend's sign, `mod` the divisor's, floored — validated 16/16 against clojure
1.12.4) **survives unchanged when those ops act on bound rete variables inside a `where`**, not just
on literal operands in a bare expression. That was the open question the family existed to answer.

Two things it surfaced are worth carrying.

---

## 1. A raising predicate aborts the ENTIRE fire — on both engines. And they agree.

A `where` / `:test` that raises mid-predicate (division by zero; i64 overflow) does **not** skip the
offending token. It unwinds the whole `fire-rules` call:

- **wat** — `#wat.runtime/DivisionByZero`, located, process exits 1; nothing after the raising point runs.
- **Clara** — `ArithmeticException: Divide by zero` thrown out of `fire-rules`, process exits 1.
- Same agreement for overflow: wat `IntegerOverflow`; Clojure `ArithmeticException: long overflow`.

**Why this matters for #49a.** Task #49's own note already flagged the hazard for its part (b) —
indexing so a token doesn't test all N predicates *"can SUPPRESS A RAISE that surfaces today (unbound
var, div-by-zero in a non-matching rule's where) — a hidden failure, in the arc whose law forbids
them."* That was written as a **hypothesis**. This measures it and makes it concrete: the raise is
**observable, total, and identically shaped in both engines.**

So a compiled-`where` executor that tried to be helpful — skipping the poisoned token, or evaluating
predicates in an order that avoids the raise — would be **unfaithful to both oracles**, not merely
faster. The abort is the contract.

This also composes with the `where-boolean` family's short-circuit row: `and`/`or` short-circuit (so a
guarded division never raises), but an *unguarded* one takes the whole fire down. Both semantics are
load-bearing and a compiler must preserve both.

### Why the row is RETIRED from the corpus rather than kept

It was authored as row 11 and it worked — it raised, exactly as designed. But a row that raises makes
its pair **permanently RED**, which:

- destroys the ratchet for that family (you cannot detect a regression in a gate that always fails),
- fails the whole-corpus run for every other family and every future rider,
- and makes the raise an *uncaught abort* rather than an *asserted outcome*.

A gate that always fails teaches as little as one that can never fail. So row 11 is retired from the
dispatch on both sides; both `defrule`/`rule-` forms are kept in place, unreferenced, as the
executable record of the form.

**The right home for it is a NEGATIVE-CONTROL harness** — a separate artifact where the expected
outcome *is* the failure and the failure is asserted, not merely suffered. That artifact does not
exist yet and it now has two distinct seeds:

1. **a raising predicate** (this finding) — must abort, with a located `DivisionByZero`;
2. **an impure predicate** — must be REJECTED at check time by the purity fence.

(2) is the more urgent of the two. Every row in the entire corpus is currently a *positive*: 44 rows
saying "this works in both." Nothing yet demonstrates the purity fence rejects anything at all — and
"only pure, therefore compilable" is the premise the whole compilation plan rests on. A fence with a
hole would be invisible to this corpus by construction.

### A harness limitation, recorded honestly

Because a raise unwinds the process and the corpus convention runs every row in ONE process, **two
raising rows cannot coexist in one pair** — whichever comes first prevents the other from ever
running. The rider verified the overflow agreement via throwaway probes rather than a second row.
Any negative-control harness must run each raising case in its own process.

---

## 2. Comparison and arithmetic sit on OPPOSITE sides of the no-implicit-coercion line

Grounded by my own `--check` probes, not relayed:

| form | verdict |
|---|---|
| generic `:wat::core::<` — an `i64` bound var against an `f64` literal | **ACCEPTED** |
| `:wat::core::i64::+` — same-type | ACCEPTED |
| `:wat::core::f64::+` — given an `i64` argument | **REJECTED** (strict same-type) |

So `i64::+`/`-`/`*`/`/` and `f64::+` refuse cross-type operands (the arc-300 no-implicit-coercion
ruling), while the **generic comparison operators silently accept a mixed pair** — statically, and
dynamically inside a rete `where`. It is a landed corpus row (`where-numeric` row 7, `gencmp`), so
this is exercised behaviour, not a hypothetical.

**Not ruled here — this needs the builder's call**, and the two readings are genuinely different:

- **Deliberate.** Comparison across numeric types is well-defined and total (no result type to choose),
  where arithmetic would have to pick a result representation. Many languages draw the line exactly here.
- **An inconsistency.** Arc 300 ruled no implicit coercion; comparison appears not to have received
  the same treatment. It is also a shape the record names as recurring — *one side normalised and the
  other not* — which has been the root of three separate generics bugs in this arc alone.

What makes it worth a ruling rather than a shrug: a `where` clause is exactly where a user first meets
it, and if rete becomes the primary locus for program logic, this asymmetry becomes a thing every rule
author has to know. It is also directly a **compiler input** — #49a must know whether a compiled
predicate may mix numeric types in a comparison.
