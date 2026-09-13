# Rete — the fix list

> Defects found and **not yet fixed**, each with a minimal reproduction and the gate that will
> turn green when it is. Populated by `wat-scripts/fuzz/rete-differential.wat`; the method is
> audit-and-accumulate — find them all first, assess after, and never lose the reproduction.
>
> **The `$oracle` is the arbiter here, and the arc's hardest-won lesson applies:** when native
> disagrees with the wat oracle, native is wrong, and the real question is which fixture was
> missing. Three engine divergences before these two, three times the references were right.

## How an entry earns its way onto this list

1. The fuzzer reports a divergence with its coordinate.
2. It reproduces **minimally and standalone**, outside the fuzzer harness — otherwise the finding
   is about the harness, not the engine.
3. A permanent probe asserts the CORRECT behaviour and is `#[ignore]`d, so the fix makes it pass
   and un-ignoring it is the completion step.
4. The fuzzer's ratchet count (`tests/rete/fuzz_rete_differential_live.rs`) accounts for it.

---

## OPEN

### A — a LEADING accumulate emits one row per FIXPOINT ROUND

**Found** 2026-08-25 · **Probe** `probe_arc278_fuzzer_found_divergences.rs::a_leading_accumulate_passes_once_per_fire_not_once_per_round`

```
:when [(?n <- (:wat::rete::acc::count) :from (:user::W))
       (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))]
```

Rows track the fixpoint round count exactly. Measured with an INERT chain — one that derives
facts the query never reads, whose only role is to make the fixpoint iterate:

| chain | rounds | native | oracle |
|---|---|---|---|
| 1 rule | 2 | **2** | 1 |
| 2 rules | 3 | **3** | 1 |

**This is the same class as the leading `:not`/`:exists` defect fixed 2026-08-24 in `71d0e700e`,
and that fix did not reach accumulate.** Worth reading before attempting this one: the correctness
mechanism there is `leading_emitted` persisting ACROSS rounds (declared at `fire/delta.rs:318`,
OUTSIDE the round loop). The round gate in `fire/pass/filter.rs` is a performance shortcut and
disabling it changes nothing observable — a distinction that cost an hour to learn, so do not
re-learn it here.

**Hypothesis, untested:** the accumulate pass has no equivalent of `leading_emitted`, so a
parentless accumulate re-emits its token every round the way the leading filters used to.

### B — a SECOND `where` after an accumulate matches NOTHING

**Found** 2026-08-25 · **Probe** `probe_arc278_fuzzer_found_divergences.rs::a_second_where_after_an_accumulate_must_not_kill_the_match`

Two queries differing by exactly one trailing, trivially-true `where`:

```
qB1  [(P1 (?a <- :k))  (?n <- acc::count :from (W))  (where (>= ?n 2))]              -> 1  = oracle
qB2  [(P1 (?a <- :k))  (?n <- acc::count :from (W))  (where (>= ?n 2))  (where (> 1 0))]  -> 0  != oracle 1
```

Independent of chain depth — reproduces at depth 0, so it is **not** a fixpoint issue. The
one-`where` form is the agreeing control and is gated NON-ignored, so a fix cannot succeed by
breaking the case that already works.

**Adjacent prior art:** a `:where` before two or more fact conditions silently matched nothing
(fixed 2026-08-24, `444ba9239`). Same smell — `where` placement relative to other LHS elements —
but a different arrangement, and the earlier fix does not cover it.

### C — `:not` over a DERIVED class ignores the derivation

**Found** 2026-08-25 · **Probe** `probe_arc278_fuzzer_found_divergences.rs::negation_over_a_derived_class_must_see_the_derivation`

54 of the fuzzer's 76 divergences, **all at depth >= 1 and never at depth 0** — precisely the
dependence stratified negation should have, which is what makes the count a diagnosis rather than
a symptom list.

```
r1:  :when [(S1 (?k <- :k))] :then [(S2 :k ?k)]      ;; S2 exists ONLY by derivation
qC:  :when [(:wat::rete::not (S2 (?s <- :k)))]
```

| world | native | oracle |
|---|---|---|
| no chain — S2 absent | 1 | 1 ✓ |
| chain present — S2 derived | **1** | **0** |
| control: is S2 actually there? | 1 | 1 |

**Both engines derive the fact. Only the oracle's negation sees it.** The third row is gated
non-ignored so this can never be misread as native failing to derive S2 — it derives it and
negates as though it had not.

**⚠ This one is a SEMANTICS question, not just a wiring bug, and deserves Clara before a fix.**
Stratified negation requires the negated class's stratum to be complete before the negation
evaluates. The oracle does that. Native does not — but it is worth establishing whether a
`defquery` is *meant* to be stratified the same way a `defrule` is, because if queries are
deliberately un-stratified then the ORACLE is the one that is wrong. Everything else on this list
is native-is-wrong by the arc's standing rule; this entry is the one where I would check the third
reference first.

---

## FIXED

*(none yet — this list is one day old)*

---

## Why the existing corpus could not see either

The accumulate axes (`accum`, `min-finding`) compare **derived facts**, and `production_delta`
dedups those by value: a rule deriving one distinct fact reads identically whether its token
passed once or four times. 37 of the 57 queries in the where-family corpus have that same shape.

Every query the fuzzer generates carries the rule's own LHS, so `query` reads beta — below the
dedup. That choice is why these were reachable at all, and it is the first thing to preserve if
the fuzzer is ever restructured.
