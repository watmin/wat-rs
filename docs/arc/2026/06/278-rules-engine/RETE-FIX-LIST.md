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

*Nothing. Every entry on this list is closed.*

---

## CLOSED — E

### ~~E — `:then` kwargs in a RUNTIME-BUILT rule are read POSITIONALLY~~ · **CLOSED 2026-08-27**

**Found** 2026-08-27 · **Probe** `probe_arc278_then_kwargs_positional.rs::reversed_kwargs_derive_the_same_fact_as_declaration_order` (`#[ignore]`d; its declaration-order control is NOT)

```
;; the SAME rule, built as a `Rule` VALUE, written two ways
:then [(:tk::Two :a ?x :b ?y)]   ;; declaration order -> witness 3024   CORRECT
:then [(:tk::Two :b ?y :a ?x)]   ;; reversed          -> witness 24003  TRANSPOSED
```

`Src` facts `(0,7) (1,8) (2,9)`; 3024 is `(a=0,b=7) (1,8) (2,9)`, 24003 is `(a=7,b=0) (8,1) (9,2)`
— the same pairs in the wrong fields. A **declared `defrule`** with those same reversed kwargs
derives 3024, so the two doors disagree:

| rule built by | reversed-kwargs result |
|---|---|
| declared `defrule` | reordered, correct |
| runtime `Rule` value | **transposed** |

**Root.** `reorder_kwargs_by_field_name` rewrites `:then` kwargs into declaration order inside the
FREEZE-TIME `defrule` wall. A rule constructed at runtime never passes that wall, so its kwargs
reach `build_insert_fact` in written order and are consumed positionally. This is verbatim the
class arc 294's wall was built for — *"the RHS insert form takes kwargs POSITIONALLY with no
name-check or reorder... NOTHING screamed"* — closed for declarations, left open for values.

**⚠ WHY NO EXISTING GATE COULD SEE IT, and the lesson for the other fuzzers.** Row counts are
blind: a transposed `:then` derives exactly as many facts. And the ENGINE DIFFERENTIAL is blind
too — native and `$oracle` transpose identically, so they agree perfectly on the wrong answer.
`differential-fuzz-rules.wat` found it on its first run only because it compares an independent
VALUE witness (`sum(a*1000 + b)`, asymmetric so a swap moves the number) instead of a count.

**THE MECHANISM, per the builder:** *"the pattern we use all over is kwargs is default, they are
consumed to call the positional head, which is typically primed"*. So `(:Two :a ?x :b ?y)` is meant
to be consumed into a positional call on `:Two'`. An UNREORDERED kwargs list consumed that way maps
its values onto the primed constructor's slots **in written order** — which is precisely the
transposition measured above, and why the symptom is a swap rather than an error.

**Fix shape, and the builder's framing changes it.** The obvious patch is a second reorder before
the RHS is compiled — it cannot go at `arm-session`, which is too late (`compile-all` has already
built the network from those forms), so it would land in `compile-rule`/`compile_rhs`. But that
would make TWO places that know how kwargs become positional, and the defect is precisely that the
freeze-time one is not on every path. The better shape is ONE DOOR: route the rete `:then` through
the same kwargs→positional consumption the rest of the language uses, so the declaration order is
resolved by NAME wherever a `:then` is built. The wall's bespoke reorder then has nothing left to
do, rather than gaining a twin.

**Fixed at the ONE door.** `rete_kwargs_value_asts` (`src/rete/eval_insert.rs`) — the helper shared
by `build_insert_fact`, `compile_rhs` and `lower_construct` — returned
`args.iter().skip(1).step_by(2)`: the values in WRITTEN order, field names DISCARDED. Every caller
already held the type's declaration order; it now resolves by NAME there. The freeze wall's
`reorder_kwargs_by_field_name` becomes redundant rather than gaining a twin, which was the point —
two implementations of "kwargs become positional", only one of them on every path, WAS the defect.

Probe un-`#[ignore]`d; the declaration-order control still passes; the fuzzer gate that found it is
restored and now PINS 3024 rather than merely asserting the two sides are equal — if both drifted
to the transposed value together, "equal" would certify the defect as fixed.

**⚠ ONE THING I GOT WRONG ON THE WAY, and the floor caught it.** The first cut also returned `None`
for a MISSING field, refusing under-supplied kwargs. That reddened
`compiled_rhs::tests::compiled_rhs_result_identical_to_interpreter`, whose fixture deliberately
compiles `(:fan::Pair :key ?k :lid ?l)` against a THREE-field record to check both paths treat the
short form identically. Arity is the wall's question (`RhsMissingFields`), not this helper's — I
had fixed order and arity in one change when only order was broken. Narrowed to order alone.

---

## CLOSED — A, B, C, D

---

## CLOSED — entry D

### ~~D — wat ACCEPTS a binding inside `:not` that Clara REFUSES~~ · **CLOSED 2026-08-26**

> **Found** 2026-08-26 while writing C's Clara twin — an ACCEPTANCE divergence no wat-vs-wat
> differential could ever see, since both sides of one are wat. It surfaced only because a Clara
> twin got written, which is an argument for the twins beyond arbitration.
>
> #### The ruling, and it was not "Clara is right"
>
> Builder: *"clara feels correct here — is our expression even logical to support?"* Probed on the
> live binary rather than reasoned about, and the answer split three ways:
>
> | shape | verdict |
> |---|---|
> | `(:not (Temp (?c <- :c) (< ?c 20)))` — fresh bind, consumed by an inner constraint | **legal, and the bind is REQUIRED** — wat constraints reference variables, so it is the only way to say "no Temp under 20". Clara needs no variable because its constraints name the field directly |
> | `(:not (Reading (?loc <- :loc)))` after `(Station (?loc <- :loc))` | **legal** — `?loc` is already bound, so inside the negation it is a USE, not a declaration |
> | `(:not (S2 (?s <- :k)))` — fresh bind, consumed nowhere | **refused** |
>
> So Clara's blanket rejection is stricter than wat needs: `(?x <- :f)` is OVERLOADED in wat —
> declaration when fresh, correlation when already bound. A "no binds in negations" rule would
> have destroyed `where-not-bound` and `where-not-fact`.
>
> #### The precedent the builder named, and it fits exactly
>
> *"we have a similar restriction on parametric types — we refuse forms who declare a type and
> never use it."* That is arc 109's `DESIGN-STONE-a-param-spec-must-be-consumed`
> (`TypeErrorKind::UnconsumedTypeParam`), whose own rationale transfers with one word swapped:
> *"an unused param still discriminates types ... the declaration is rejected for READABILITY: a
> reader cannot tell a deliberate tag from a leftover edit."* Declared must be consumed.
>
> #### What made it worth more than tidiness
>
> The same rule kills a strictly worse shape: a fresh bind consumed OUTSIDE the negation. It does
> fail at fire (`unbound symbol`) — but the `:where` referencing it runs only when the negation
> PASSES, so it **hides behind the data**. Measured, same binary, same rule: fact present →
> `n=0`, exit 0; fact absent → UnboundSymbol, exit 1. It compiles, survives any test whose
> fixtures happen to contain a matching fact, and dies the first time production has none.
>
> #### The fix — declaration-check time, the strongest rung (builder's call)
>
> `src/rete/validate.rs` — arc 294's freeze-time `defrule` wall, which already walks `:when`
> recursively through wrappers. Two located variants, `UnconsumedWrapperBind` and
> `EscapedWrapperBind`. The rule:
>
> > A variable **fresh** inside a `:not` is a declaration and must be consumed within that same
> > `:not`. A variable already declared elsewhere is a use, and is untouched.
>
> **It never asks whether a variable is bound** — only whether every DECLARATION of it sits inside
> one negation. That is what keeps it clear of the trap `RhsUnresolvableOperand` names in its own
> doc comment: binder analysis that UNDER-collects rejects legal rules, *"the one failure a wall
> must not have."*
>
> #### ⚠ THE NEAR-MISS, RECORDED BECAUSE IT IS THE MOST USEFUL THING HERE
>
> The wall covered `:exists` too for exactly one build — Clara traps both, and `validate.rs`'s own
> doc comment asserted *"`exists` binds nothing outward."* **That comment is false**, and acting on
> it would have rejected a live accuracy axis: `leading-exists.wat` is a leading `:exists` binding
> `?loc`, and the axis reads that binding back out of the query rows by string key
> (`PersistentMap/get p "?loc"`) to build `:derived`. A consumer in HOST CODE is invisible to any
> syntactic check, so **no syntactic check may judge an outward-binding construct.**
>
> Caught by sweeping all 46 grid axes before committing, one step from shipping. `:not` is safe to
> judge because it binds nothing outward BY CONSTRUCTION — it admits a token precisely when no
> fact matched — and the runtime says so out loud. The false comment is corrected in place, and
> `an_unconsumed_bind_inside_exists_is_left_alone_because_exists_binds_outward` is the regression
> guard: textually identical to the refused shape but for the wrapper.
>
> **Blast radius, measured:** 6 violations in 4 files across 1562 `.wat` files, **zero in the
> stdlib** — all fixtures, probes and the grid. Each fixed by dropping the dead bind to the bare
> class. `where-not-not` re-checked against Clara after the edit: byte-identical, as a dead bind
> must be. **Known scope:** the wall sees DECLARED rules (`make-rule`/`make-query`); rules the
> fuzzer builds at runtime bypass it, as they bypass every other check in this validator.

---

## FIXED

- **B — a second `where` after an accumulate matched nothing.** Closed 2026-08-26; see the entry
  above for the root, the fix and the four-step closure. Ratchet 120 → 72.
- **A — a leading accumulate emitted one row per fixpoint round.** Closed 2026-08-26.
- **C — `:not` over a derived class ignored the derivation.** Closed 2026-08-26. A and C were ONE
  root; see C's entry. Ratchet 72 → **0**.

- **D — wat accepted a binding inside `:not` that Clara refuses.** Closed 2026-08-26 at
  DECLARATION-CHECK time, the strongest rung: `src/rete/validate.rs`. See D's entry for the rule,
  the arc-109 precedent it mirrors, and the `:exists` false positive it nearly shipped.

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

---

## ~~F — AN INLINE CONSTRAINT WITH A NESTED-CALL OPERAND IS SILENTLY UNSATISFIABLE~~ · **CLOSED 2026-08-28**

> **CLOSED 2026-08-28 — fixed in BOTH engines, gated at the CLASS, and the grid ruling satisfied.**
> The account below is kept in full because the mechanism is the lesson, not the line.
>
> **What it was:** a live, user-facing WRONG ANSWER with no diagnostic at any stage. Not found by
> the fuzzers, and not findable by any reading ward.
>
> **How it was fixed — and it was NOT a wall.** The first proposal was to refuse the form at
> compile time; the builder refused that: *"we made it such that every rete form can be compiled
> to a jump table... why is this any exception?"* It is not one. `compiled_cond` already imported
> the one core's `Expr`; what never landed with flip 3 was the LOWERING. A nested operand now goes
> through `expr_ir::lower_in_frame` into the same `Expr::Call`, the same opcode and the same
> `RETE_OPS` table the `where` fence uses, materialised by a new `Op::Eval` — which keeps
> `Op::Cmp`'s operands `Slot | Lit` so the per-fact fast path never moved.
>
> **THE ORACLE HAD IT TOO, and that is why five fuzzers and 5612 shapes were blind.** The new grid
> axis went red on its first floor run with the `$oracle` answering `n=0`. Native and the oracle
> did not merely agree — they SHARED the defect (FM 28 in its purest form), and Clara broke the
> tie, inverting this file's own rule: here the ORACLE was the stale one. Its fix is the
> interpreted twin, reusing the same core via `eval_computed_operand`.
>
> **THE GATE IS AT THE CLASS.** `rete/reachability.rs` now BANS `MATCHES-NOTHING` for every row in
> every position: a cell must FIRE or be REFUSED — one works, the other TEACHES; "compiled, ran,
> matched nothing, said nothing" is a red build. Mutation-proven: restoring the defect reddens all
> SIX ledger shards plus the specific probe plus the grid axis — eight independent gates.
>
> **Ledger after:** 55 rows fire in both positions (was 16), 18 refused inline WITH a diagnostic,
> 4 holon rows not generable, **ZERO silent**.

### The reproduction — three runs of one file, one line different

```wat
(:wat::core::defrecord :m::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :m::Out [k <- :wat::core::String])

(:wat::rete::defrule :m::rule
  :when
  [(:m::In (?k <- :k)
     (:wat::rete::core::i64::= (:wat::rete::core::i64::+ :v 2 :undefined 0) 12))]
  :then
  [(:m::Out :k ?k)])
```

Facts `v = 10` (10+2 = 12, the constraint HOLDS) and `v = 1` (1+2 = 3, it does not).

| form | rows | correct |
|---|---:|---:|
| no constraint at all | 2 | 2 — proves the rule installs and fires |
| the SAME expression inside a `(:wat::rete::where …)` fence | 1 | 1 — proves the expression is valid and the engine can evaluate it |
| the expression as an INLINE constraint | **0** | **1** |

**Exit code 0. Zero bytes on stderr.** No error, no warning, at freeze, at `compile-all`, or at
fire.

### It is not "ignored" — it is UNSATISFIABLE, which is the worse failure

A dropped constraint would OVER-match (2 rows). This UNDER-matches to zero: the rule never fires,
for any input, ever. And no value of the compared-against operand rescues it — expected `12` gives
0 rows and expected `0` gives 0 rows, which is how the first hypothesis (that the operand resolves
to its `:undefined` fallback) was REFUTED.

### Scope, stated exactly

Requires BOTH the inline-constraint position AND an operand that is a nested call. Inline
constraints over a `?var`, a field-ref or a literal are correct — 16 `RETE_OPS` rows fire there.
**But every inline constraint that COMPUTES anything is silently dead.** In the ledger's inline
column that is 39 of 77 rows.

### The mechanism, read off the disk

1. `classify_rete_clause` sees head `:wat::rete::core::i64::=` — a real rete comparator — so the
   clause is a perfectly good `ReteClauseShape::Constraint`. The freeze validator has nothing to
   object to. (This is why the OTHER 18 rows are honestly refused: unary ops and `Type/method`
   spellings are not Constraint-shaped, classify as `Unrecognized`, and the validator raises
   `MalformedClause`. The clause level was walled; the OPERAND level never was.)
2. `compiled_cond::compile_operand_expr` handles `Symbol` (`?var`), `Keyword` (field) and then
   `other => ast_literal_value(other).map(Expr::Lit)`. A nested call is a `WatAST::List`, is not a
   literal, and returns `None` — so the whole condition fails to compile and falls back.
3. The interpreted fallback `matcher::eval_clause` calls `resolve_operand`, which handles the same
   three shapes, returns `None`, and hits `_ => return None` (`matcher.rs:678`).
4. **`None` there means "this fact does not match".** The function's own doc says the quiet part:
   *"`None` on mismatch **or unresolvable operand**"* — two different facts, one return value.

### The fix is a HALF-FINISHED FLIP, not a new wall and not a new evaluator

`DESIGN-STONE-the-one-expression-core` records *"Flips 3–5 (`cond` / `rhs` / user acc folds)
LANDED 2026-08-17"*, and `compiled_cond.rs:66` really does `use crate::rete::expr_ir::Expr` —
`Op::Cmp { lhs: Expr, rhs: Expr }` can hold an `Expr::Call` **today**. What did not land is the
LOWERING: `compile_operand_expr` never calls `expr_ir::lower()`, the function the `where` fence
uses to turn a `WatAST` into an `Expr` DAG. So `cond` took the one core's TYPE and kept its own
three-case mini-lowering that stops at literals. That is the entire difference between the fence
answering 1 and the inline position answering 0.

⚠ **DO NOT "fix" this by refusing the form at compile time.** That was this instance's first
proposal and the builder refused it: a diagnostic would cement a limitation that exists only
because one function does not call another, and the arc has already ruled ONE expression core for
all four surfaces.

### Open question — answerable by probe, not by argument

Whether the two slot models compose. `compiled_cond` assigns slots against a fact's field list and
writes them with an `Op::Bind` prologue; `expr_ir::LowerCx` assigns slots by name against a
`SymbolTable`. A nested operand lowered through the core must have its slot READS line up with the
alpha prologue's WRITES. There may also be a deliberate reason it was left: alpha match is the
per-fact hot path, and the stone's zero-allocation gate is measured on exactly these arms.

### ~~The gate that turns green~~ · **ALL GREEN 2026-08-28**

- `rete::reachability::an_inline_constraint_that_computes_now_computes` — the three-row
  reproduction, with the fence and no-constraint forms as controls AND a discrimination control
  (asking for a value the arithmetic does not produce must select ZERO, or the operand is being
  evaluated but not compared).
- The ledger's class gate — `MATCHES-NOTHING` banned surface-wide.
- **The grid, per the builder's 2026-08-26 ruling.** `where-inline-computed.{wat,clj}`, registered
  in `WHERE_FAMILY`: one predicate written FOUR ways — inline and fenced, in wat and in Clara —
  byte-identical across all four. The axis exists because the corpus was structurally blind: the
  wat half held ZERO inline constraints across all 36 axes, and Clara's used `:test` in 128 rows
  against 7 in-pattern predicates. A shape corpus one-sided on POSITION cannot see a position
  defect, however many shapes it holds.
