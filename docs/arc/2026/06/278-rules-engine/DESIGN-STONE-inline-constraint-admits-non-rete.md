# DESIGN-STONE — the inline alpha constraint admits non-rete ops, and law A never sees it

> **Status: DRAWN 2026-08-06, grounded by run.** Found while answering *"are both LHS and RHS now
> total?"* immediately after #83 armed the accumulator fence (`c6d16df2`).
>
> **The answer to that question is NO, and this stone is why.** The three EXPRESSION fences
> (`where` · `:then` · accumulator) are armed and honest. But `where` is **not the only expression
> surface on the LHS**, and the other one has no fence at all.
>
> Blocks **#49 `compiled_where`** — `compiled_cond.rs` already dispatches these ops, so a jump table
> drawn today inherits the hole.

## The finding, proven by run — not reasoned

A **fact pattern** may carry an inline constraint clause beside its bindings:

```clojure
;; COMPILES AND FIRES TODAY. Law A never sees it.
(:probe::Reading (?loc <- :location) (?v <- :value) (:wat::core::> :value 10))
```

Reconnaissance artifact: `wat-scripts/scratch-pad/probe-inline-constraint-bypasses-law-a.wat`.
Measured `{:rule-count 2 :flagged-count 1}` with the control rule present and the value-3 fact
correctly excluded — so the constraint compiled **and discriminated**, it did not merely parse.

### Why the fence cannot see it — two grammars, one surface

`compile-condition` (`wat/rete.wat:679`) branches on exactly four shapes:

```
is-where  ·  is-not  ·  is-exists  ·  is-accumulate      → else: a FACT PATTERN
```

Only `is-where` and `is-accumulate` carry a fence. A keyword-headed constraint matches none of the
four, so it goes to the fact-pattern branch — and the pattern's own children are classified by a
**completely separate grammar in Rust**, `classify_rete_clause` (`matcher.rs:331`), which matches
**six literal strings**:

```rust
":wat::core::=" | ":wat::core::not=" | ":wat::core::<" | ":wat::core::>"
| ":wat::core::<=" | ":wat::core::>=" => ReteClauseShape::Constraint { … }
```

Nothing on that path consults `pure?` / `deterministic?` / `total?` / `primitive?`.

**This is instance 5 of the arc's recurring defect class — a match on a literal STRING that no
exhaustiveness check can see.** It is also *not new information*: the law-A seam (`SEAM-2026-08-05-lawA.md`, superseded 2026-08-06 by the single `SEAM.md`; text preserved in git history)
names this exact site as instance 2 of four, and describes it correctly —

> *"the inline LHS pattern equality is matched by the literal `:wat::core::=` in
> `matcher.rs:374-380` — a grammar entirely separate from the fence, so `=` means two things by
> position in a `defrule`."*

It was recorded as a **naming** curiosity and never asked as a **fence** question. Same site, one
day apart. `[[feedback_a_claims_support_does_not_travel_with_the_claim]]`.

### The totality half — 4 of the 6 are partial, 2 are not

`matcher.rs:450-463`, the interpreter's own arm:

```rust
":wat::core::="    => a == b,                                          // total
":wat::core::not=" => a != b,                                          // total
":wat::core::<"    => compare_values(&a, &b)? == Ordering::Less,       // ← the `?`
":wat::core::>"    => compare_values(&a, &b)? == Ordering::Greater,    // ← the `?`
":wat::core::<="   => compare_values(&a, &b)? != Ordering::Greater,    // ← the `?`
":wat::core::>="   => compare_values(&a, &b)? != Ordering::Less,       // ← the `?`
```

`=`/`not=` are plain value equality — **total**. The four ordering comparators go through
`compare_values`, whose `?` propagates the incomparable-operands error. That is the generic
comparator's domain hole, live, reachable from a rule condition, on a surface no fence governs.

**⚠ NOT MEASURED, and it decides severity:** whether a cross-type inline compare (`> :location 10`
where `:location` is `String`) **raises mid-fire** or **silently answers false**. A raise aborts the
fire; a silent `false` is the NaN-shaped mask the stone calls worse (*the rule quietly does not
fire*). Two runs failed to distinguish them because the harness's control rule flagged the same
fact either way — an instrument scoped to the wrong question
(`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`). **The RED probe below settles
it with two distinct derived fact types.** Do not state the severity until it has run.

## The fix — force the per-type rete spelling. That is the whole design.

Builder's framing, and it is the same act as round 1c:

> *"the users are forced to measure the type they expect instead of an untyped dispatch? … we built
> the typed equality checks precisely to force the typing."*

```clojure
(:wat::core::> :value 10)                  ; today — untyped dispatch, partial
(:wat::rete::core::i64::> :value 10)       ; after  — the user names the type
```

**Monomorphising deletes the domain hole rather than handling it** — `i64::>` has no incomparable
case. This is the stone's own standing ruling (*"THE RETE SURFACE IS PER-TYPE, PERIOD"*), applied to
a site that never got migrated because law A does not reach it.

There is **no generic rete comparator and there must never be one** — verified on disk, `grep`
count 0. The 20 per-type rows already exist (`vocabulary.rs`): `i64`/`f64` × `< > <= >=` and
`i64`/`f64`/`string`/`bool`/`keyword`/`enum` × `= not=`.

**A cross-type constraint becomes a COMPILE error**, because the lhs is a field keyword of a
declared `defrecord` and its type is in hand — `(:wat::rete::core::i64::> :location 10)` cannot be
written when `:location` is `String`. That is the payoff: the unmeasured runtime question above
stops existing rather than being answered.

## ⛔ THE FOUR SITES — and only three of them scream

The six literals are matched in **four** places. Each `Constraint` consumer re-matches the op
independently; there is no shared table.

| # | site | on an unmigrated op | fails |
|---|---|---|---|
| 1 | `matcher.rs:374-380` | the grammar — decides whether it is a `Constraint` at all | — |
| 2 | `matcher.rs:450-463` | interpreter eval | **LOUD** — `unreachable!` |
| 3 | `compiled_cond.rs:236-248` | compiled dispatch | **LOUD** — `unreachable!` |
| 4 | `alpha_tree.rs:243` | discrimination-tree equality fan-out | **LOUD** — see the correction below |

Sites 2 and 3 both close with:

```rust
_ => unreachable!("classify_rete_clause: Constraint op outside the recognized set"),
```

— a comment asserting a closed set that is enforced only by the literal list in site 1. Widen site 1
without them and you get a panic, which is the honest outcome.

Site 4's arm is:

```rust
ReteClauseShape::Constraint { op: ":wat::core::=", lhs, rhs } => { … }
…
_ => {}
```

Migrate the grammar without it and `collect_equalities` returns **empty** — the alpha discrimination
tree loses its equality fan-out and degrades to a linear scan over every alpha of the class. That is
the mechanism R61 went back into `holon-lab-ddos` to recover (`tree.rs:75`'s `ShadowNode` equality
fan-out — the thing that closed the A0 `:winner :clara 0.745` cell).

### ✅ CORRECTED 2026-08-06, BY MUTATION — the gate already exists, and this stone was wrong about it

> **The first revision of this section read: *"Correct, still green, silently slow — nothing in the
> floor asserts on the tree's shape."* That is FALSE, and it was asserted from reading, one day after
> the record's own lesson about exactly that.**

Before building the STOP-1 gate this stone specified, I went looking for an observable and found the
gate **already on disk** — `alpha_tree_discriminates_candidates_to_about_one_at_50_100`
(`kernel.rs:6775`), which asserts *both* halves the stone asked for:

```rust
assert!(mean_with < 2.0,     "…the tree is correct but discriminates nothing");
assert!(mean_without > 10.0, "…the row-2 pass above would be vacuous");
```

**Mutation-proven this session.** Changing the literal at `alpha_tree.rs:243` to a never-matching
string and running the two alpha-tree rows:

```
      mean candidates/fact WITH the tree:      50.000
      mean candidates/fact WITHOUT (bypassed): 50.000   (the pre-stone linear scan)
    STOP-3: mean candidates/fact WITH the tree is 50.000 at [50 100], not ~1 —
            the tree is correct but discriminates nothing. Distribution: 50 candidates × 200 facts

    PASS  alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100
    FAIL  alpha_tree_discriminates_candidates_to_about_one_at_50_100
```

1.000 → 50.000, caught, **and the diagnosis names the exact failure mode**. Reverted; both green.

**So all four sites scream. Site 4 is not the trap; there is no silent site.**

And the run proves *why the two rows had to be separate*: under the same mutation the **superset**
row PASSED. A tree that wildcards everything is perfectly correct and buys nothing — correctness
cannot detect a discrimination regression, which is precisely the trap-door the discrimination row
was written against. Whoever split them was right.

**The lesson is #48's UNADOPTED class and R61 `PAR NON ARGVIT, NOSTRA ARGVVNT` in one act:** I
specified a gate, wrote a STOP around it, and was one command from building a duplicate of something
this repo already had — because I read the site instead of asking whether the record already covered
it. `[[feedback_ground_the_substrate_not_just_the_chronicle]]`, applied to our own tests.

## The RED probe — `tests/rete/probe_arc278_inline_constraint_law_a.rs`

Written and committed **before** any of the four sites move. Three rows; the first two are RED
today, the third is GREEN today and must **stay** green (it is the anti-regression gate for site 4).

| row | asserts | today |
|---|---|---|
| `untyped_inline_constraint_is_refused` | `(:wat::core::> …)` in a pattern is refused, message pinned by `assert_eq!` | **RED** — it compiles and fires |
| `per_type_inline_constraint_is_admitted_and_prunes` | `(:wat::rete::core::i64::> …)` compiles, fires, and discriminates | **RED** — grammar does not match it |
| `severity: raise_or_silent_false` | which one a cross-type compare does, via **two distinct derived fact types** so the counts discriminate | records the unmeasured fact |

**Site 4 needs no new gate — see the correction above.**
`alpha_tree_discriminates_candidates_to_about_one_at_50_100` (`kernel.rs:6775`) already covers it and
is mutation-proven. Run it, and its superset sibling, at the weigh.

## Migration size

**3 clauses in the corpus** (`grep` over `wat/ wat-tests/ wat-scripts/ tests/`). The migration is
nothing; the coupling is everything.

## STOPs

- **⛔ STOP-1 — site 4 moves in the SAME commit as site 1**, and
  `alpha_tree_discriminates_candidates_to_about_one_at_50_100` must be **run at the weigh**, not
  inferred from a green floor summary. It IS in the floor, so a full `scripts/floor.sh` covers it —
  but read that row by name, because a mutation there reports `mean 50.000` and its sibling superset
  row still passes.
- **⛔ STOP-2 — do not mint a generic rete comparator** to make this easy. Zero exist by ruling;
  a generic one would have to carry `:undefined` or an outcome enum to be admissible at all, which
  is strictly worse than naming the type.
- **⛔ STOP-3 — the refusal must TEACH** (R29 `RVINA ERVDIT`). Letting an untyped op fall through
  to `Unrecognized` → `MalformedClause` is wrong: the clause is well-formed, it is *non-rete*. Keep
  the six core spellings recognized by the grammar so the diagnostic can name the head **and its
  per-type twin**; refuse in `validate.rs`, where the span is already in hand.
- **⛔ Do not "fix" a negative control.** `probe_fence_names_the_head.rs` and the two deliberate-RED
  fence fixtures pin refusals; a rider that migrates them has broken what they measure.

## What this does NOT close

Stated so it is not re-derived, and so the next "is it total?" answer is honest:

1. **Totality is not termination.** The composition door admits recursion by design
   (`purity.rs`, back-edge ⇒ `Ok`). A jump table over a non-terminating predicate still hangs.
2. **An ingested fact field may already hold NaN.** The fence governs expressions, not data.
