# Rete — the fix list

> Defects found and **not yet fixed**, each with a minimal reproduction and the gate that will
> turn green when it is. Populated by `wat-tests/rete/differential-fuzz.wat`; the method is
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

## ⛔ HOW AN ENTRY LEAVES THIS LIST — THE GRID IS PART OF THE FIX

**Builder's ruling, 2026-08-26:** *"when we approach fixing the wat-rete issues wat-gen
revealed... we must add them to our grid.... we must ensure we are completely accurate relative
to clara."*

A fix is NOT done when the `#[ignore]`d probe goes green. Each defect below is a shape the grid
did not cover — that is precisely why the fuzzer found them and the 57-query where-family corpus
did not. So closing an entry requires, in this order:

1. the minimal probe un-`#[ignore]`d and passing;
2. **a new grid axis for the shape** (`wat-scripts/perf/grid/`), so the shape is measured on every
   grid run and cannot silently regress. Note `grid_axes_run_and_derive_nonvacuously` asserts the
   on-disk sized axes EXACTLY equal `SIZED_AXES` — a new axis must be given a non-vacuous size
   deliberately, which is the gate that stops a vacuous addition;
3. **agreement with Clara on that axis** — `:match` and `:us`, not merely native-vs-`$oracle`.
   Clara is the third reference and the arbiter of what the semantics ARE. The `$oracle` says what
   we *intended*; Clara says what a rules engine *does*. Entry **C** below is exactly the case
   where those two can disagree, which is why it is marked as needing Clara BEFORE a fix.
4. the ratchet count lowered by the number of divergences the fix removed, with the family named.

**Accuracy relative to Clara is the acceptance criterion, not speed.** A fix that makes the probe
pass while moving an axis away from Clara has not closed the entry — it has traded a known defect
for an unknown one.

---

## CLOSED — the reproductions are kept in place

> Each entry keeps its minimal reproduction, its measured table and its four-step closure.
> A closed entry is not deleted: the reproduction is the durable artifact, and the roots below
> generalise (B is "a kind-ordered pass sequence over a DAG is only correct while the DAG's
> kind-order matches it"; A and C are "a non-monotonic condition must not be evaluated inside
> the fixpoint"). The live list is **OPEN**, further down.

### ~~A — a LEADING accumulate emits one row per FIXPOINT ROUND~~ · **CLOSED 2026-08-26**

> **A and C are ONE defect. Read the C entry below for the root — it is the same root.**
>
> **Found** 2026-08-25 · **Probe** `probe_arc278_fuzzer_found_divergences.rs::a_leading_accumulate_passes_once_per_fire_not_once_per_round` — now live, not `#[ignore]`d.
>
> ```
> :when [(?n <- (:wat::rete::acc::count) :from (:user::W))
>        (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))]
> ```
>
> Rows tracked the fixpoint round count exactly, measured with an INERT chain — one deriving facts
> the query never reads, whose only role is to make the fixpoint iterate:
>
> | chain | rounds | native (before) | oracle | native (after) |
> |---|---|---|---|---|
> | 1 rule | 2 | **2** | 1 | 1 |
> | 2 rules | 3 | **3** | 1 | 1 |
>
> **The standing hypothesis in this entry was wrong, and worth recording as such.** It read: *"the
> accumulate pass has no equivalent of `leading_emitted`, so a parentless accumulate re-emits its
> token every round the way the leading filters used to."* The accumulate pass emits correctly. The
> defect was one level downstream, in how a QUERY is harvested — which is why the prior art
> (`71d0e700e`, `leading_emitted` persisting across rounds) genuinely did not reach it, though not
> for the reason guessed here.
>
> **Closed per this list's own exit rule, all four steps:**
> 1. probe un-`#[ignore]`d and passing, with its agreeing control still green;
> 2. grid axis `where-accum-lead-cascade.{wat,clj}` added — a leading accumulate in a QUERY under
>    an inert cascade of depth 0, 1 and 2, so the row count must be INDEPENDENT of the cascade and
>    any spread between the rows IS the defect. `where-accum-lead` has the accumulate but in a rule
>    with no cascade; `leading-exists` has the cascade but for the `:exists` form. Neither reached
>    this shape;
> 3. **Clara 0.24.0 ran and agrees** — `n=1` at all three depths, byte-identical to native-after-fix
>    and to the `$oracle`;
> 4. ratchet **72 → 0**, closed together with C: A was 18 of the 72 and C the other 54.

### ~~B — a SECOND `where` after an accumulate matches NOTHING~~ · **CLOSED 2026-08-26**

> **Root, and it was not where anyone looked.** A `:where` that binds NOTHING sorts into
> `sort-lhs`'s INDEPENDENT partition (`wat/rete/compile.wat`) and is placed **above** the
> accumulate — Clara's own deferral ordering, and correct. The graph is therefore
> `RootJoin → Test(> 1 0) → Accumulate → Test(>= ?n 2) → Production`. But the accumulate pass is
> **3.25** and the filter pass is **3.5**, so that leading Test had never been dispatched when the
> accumulate read its parent delta. The accumulate saw nothing, derived nothing, and the rule
> matched ZERO — while the same rule *without* the bindless `:where` matched fine, because then
> the accumulate's parent is the RootJoin. **The compiler was right; the engine's fixed pass
> order could not execute the graph it emitted.**
>
> **Fix:** a pre-dispatch loop in `src/rete/kernel/fire/pass/accumulate.rs` pulls a Test parent
> forward before the accumulate reads it, returning the set for the filter pass to skip (a Test
> dispatched twice against one parent delta duplicates its tokens into production). Reordering the
> two passes was rejected: the existing order exists so a `:where` ON the result-var sees its
> binding, and swapping would trade this defect for that one.
>
> **Closed per this list's own exit rule, all four steps:**
> 1. probe un-`#[ignore]`d and passing;
> 2. grid axis `where-accum-where-chain.{wat,clj}` added — two rules differing by exactly one
>    trivially-true trailing `:where`, so a diff isolates the tautology and nothing else. It
>    asserted the FAIL state before the fix (native `row 2 n=0`, oracle `n=3`);
> 3. **Clara 0.24.0 ran and agrees** — `row 1 n=3`, `row 2 n=3`, byte-identical to native-after-fix
>    and to the `$oracle`. Three independent references concur;
> 4. ratchet lowered **120 → 72**, and the 48 it removed are named: family B was 48 of the 66
>    `f=3` accumulate divergences, leaving 18 for family A and 54 for family C.
>
> Two rete tests regressed: none. 389/389 in the rete target.

<details><summary>original entry, kept for the reproduction</summary>

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

</details>

### ~~C — `:not` over a DERIVED class ignores the derivation~~ · **CLOSED 2026-08-26**

> **Found** 2026-08-25 · **Probe** `probe_arc278_fuzzer_found_divergences.rs::negation_over_a_derived_class_must_see_the_derivation` — now live, with BOTH controls still green.
>
> ```
> r1:  :when [(S1 (?k <- :k))] :then [(S2 :k ?k)]      ;; S2 exists ONLY by derivation
> qC:  :when [(:wat::rete::not (S2 (?s <- :k)))]
> ```
>
> 54 of the fuzzer's 76 divergences, **all at depth >= 1 and never at depth 0**.
>
> #### The Clara ruling this entry was waiting for
>
> This was the one entry marked ⚠ NEEDS A CLARA RULING BEFORE A FIX, because if a `defquery` were
> deliberately un-stratified then the ORACLE would have been wrong and 54 of the ratchet's 72 would
> not have been defects. **Clara 0.24.0 settled it 2026-08-26: a query's negation IS stratified
> exactly the way a rule's is** — the query-side row and the rule-side row print the same number.
> The oracle was right, native was wrong, and the arc's standing rule held unbroken.
>
> #### THE ROOT — and it is shared with A
>
> **A query's NON-MONOTONIC condition was evaluated inside the fixpoint instead of once against the
> closed world.** `harvest_query_memory` (`fire/mod.rs`) has two branches: a CLASS-SCAN query reads
> the closed bag and is correct; every other query — which means every `:not` and every accumulate
> — falls to the `else` branch and reads `wm.beta[parent]`. Beta memories, by the semi-naive
> fixpoint's own design contract, **accumulate across rounds and are never cleared**. So a
> constrained query reads tokens that were only ever true of the round that produced them:
>
> - **A** — the leading accumulate's parent gains a token every round; the harvest reads all of them.
> - **C** — the `:not` propagated its token in round 0, before the negated class existed. A later
>   round derives the fact; nothing retracts the token already in beta.
>
> That is why all 72 divergences were `:not` and accumulate and nothing else: non-monotonic is
> exactly the class a later round can invalidate.
>
> **How it was found, in one probe and no engine edits.** Adding an unrelated rule that negates a
> type — whose only effect is to push `max_s` above 0 and route through the stratified driver —
> made BOTH defects vanish with the queries themselves untouched (A: 3 rows → 1; C: passes →
> blocks). The path was the variable, not the query. That single observation merged two entries.
>
> **Fix:** `fire_unstratified` in `src/rete/kernel/fire/rules.rs` — the fixpoint, then constrained
> queries re-derived against the closed world via the door the stratified driver already used
> (`harvest_stratified_queries`). It is ONE door and **both** `max_s == 0` exits go through it: the
> AST door and the Export door (`fire_rules_from_deps`) each had the defect, and wrapping both call
> sites would have been a convention the second door had already disproved. Class-scan queries are
> deliberately untouched — they never read beta.
>
> **Closed per this list's own exit rule, all four steps:**
> 1. probe un-`#[ignore]`d and passing, both controls green;
> 2. grid axis `where-not-derived-in-query.{wat,clj}` added — query-side `:not` over a DERIVED
>    class, at depth 1 and depth 2, with the rule-side form as the contrast row IN THE OUTPUT. No
>    prior axis crossed the two: every query-side `:not` in the grid negates an INSERTED class
>    (`where-query-compat`'s Wind), and every derived-class `:not` lives in a RULE (`strat-neg`,
>    `negation`, `neg-consumer`);
> 3. **Clara 0.24.0 ran and agrees** — all five rows byte-identical to native-after-fix and to the
>    `$oracle`;
> 4. ratchet **72 → 0**, closed together with A.

---

## OPEN

### D — wat ACCEPTS a binding inside `:not` that Clara REFUSES

**Found** 2026-08-26, while writing C's Clara twin · **No probe yet** · ⚠ **NEEDS A RULING — this
one is a language decision, not a defect the `$oracle` can arbitrate.**

wat's own C fixture writes the negation as:

```
(:wat::rete::not (:user::S2 (?s <- :k)))
```

`?s` is bound inside the negation and used nowhere — it cannot be used anywhere, because a
negation that matches nothing binds nothing. wat compiles this silently. Clara 0.24.0 refuses it,
at compile time:

```
Using variable that is not previously bound. ... Note that variables used in negations are not
bound for subsequent rules since the negation can never match.
Unbound variables: #{?s}
```

**Why it is on this list.** It is a divergence in what the two engines ACCEPT rather than what they
compute, so no fuzzer run and no `$oracle` differential can see it — both wat sides agree, because
both wat sides are wat. It surfaced only because a Clara twin was written for an axis, which is an
argument for the twins beyond arbitration.

**Why it is not folded into C.** A fix for one is not a fix for the other, and this one has a real
question in front of it: is the binding a *user error wat should reject* (Clara's position, and the
`extirpare` reading — the wrong thing currently has a form), or a *harmless no-op wat tolerates*?
Rejecting is a source-compatibility change for any existing rule that writes it, including this
arc's own fixture. **The builder's call, not mine.**

---

## FIXED

- **B — a second `where` after an accumulate matched nothing.** Closed 2026-08-26; see the entry
  above for the root, the fix and the four-step closure. Ratchet 120 → 72.
- **A — a leading accumulate emitted one row per fixpoint round.** Closed 2026-08-26.
- **C — `:not` over a derived class ignored the derivation.** Closed 2026-08-26. A and C were ONE
  root; see C's entry. Ratchet 72 → **0**.

**The fuzzer's divergence count is now 0 and the gate is an equality, not a ratchet.** Native and
the `$oracle` agree bit-for-bit across all 1260 generated shapes. Any nonzero from here is a
regression carrying its own coordinate, not a known defect.

---

## Why the existing corpus could not see either

The accumulate axes (`accum`, `min-finding`) compare **derived facts**, and `production_delta`
dedups those by value: a rule deriving one distinct fact reads identically whether its token
passed once or four times. 37 of the 57 queries in the where-family corpus have that same shape.

Every query the fuzzer generates carries the rule's own LHS, so `query` reads beta — below the
dedup. That choice is why these were reachable at all, and it is the first thing to preserve if
the fuzzer is ever restructured.
