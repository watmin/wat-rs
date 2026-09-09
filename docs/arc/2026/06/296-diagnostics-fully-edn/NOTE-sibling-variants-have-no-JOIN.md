# NOTE — sibling variants have no JOIN, and that is A-1's gap

**Measured 2026-09-08**, arc 296 stone A-2. The work is preserved at `a75bc4ce2` and reverted;
the loud refusal is restored and the floor is green at 5280/5280.

## What the floor saw that two riders' targeted checks could not

```
600 × #wat.check/TypeMismatch, dominant form:
  ":wat::core::match: parameter arm #4 expects :probe::Outcome::Message;
                                           got :probe::Outcome::Closed"
  ":wat::core::match: parameter arm #4 expects :usr::my-sift::SiftRulesResponse::RequestTooLarge;
                                           got :usr::my-sift::SiftRulesResponse::Fatal"
```

A `match` returns a **different sibling variant from different arms**. That is not exotic — it is
what every enum-returning function in the corpus does.

## ★★★ The gap, stated exactly

```
A-1 gave  if/send/try-send  SUBSUMPTION      directional: is A assignable to B?
they need                   a JOIN           both A and B are subtypes of a common parent
```

`Some` and `None` are **siblings**. Neither is assignable to the other. Their least upper bound is
`Option`. `join_if_branches` tests one-directional `assignable` between the two branch types, which
answers a question these pairs cannot satisfy in either direction.

This was invisible before variants existed as types, because every arm produced the same **erased**
enum type and the pairs were trivially equal.

⚠ **So A-1 is incomplete, not wrong.** Subsumption was the right first rule and it holds. A join is
a second, larger rule, and it is a language decision: *what is the type of a form whose branches
produce different types?*

## ⛔ The scope cut bought nothing

The rider scoped `register_variant_types` to `!is_reserved_prefix` — stdlib enums excluded — to
escape the 1228-error first measurement. The floor shows the failures landing on `:probe::`,
`:usr::`, `:arena::`: **all user namespaces.** The cut RELOCATED the gap out of the stdlib and into
user code, and cost the capability for `Option`/`Result` — the population the feature is for.

★ And it re-introduced `is_reserved_prefix` as an exclusion inside the campaign built to delete it.

## ⛔ A second wall fired, and it is the recurring class

```
🔥 A SECOND NAME PARSER — 1 site hand-rolls one of the five name-grammar call-shapes
   OUTSIDE crates/wat-reader/src/identifier.rs
```

The variant FQDN was built by hand-rolled string surgery. `CLAUDE.md` names this as the recurring
class — *"a macro-generated name is built by string concatenation, so it is where generics get
silently mangled"* — and arc 109 built this exact lint after a census found 33 sites that had
already disagreed. Any reland mints variant names through `identifier.rs`'s accessors.

## ⛔⛔ MY ACCEPTANCE ROWS COULD NOT SEE EITHER FAILURE

Fourteen rows. Two riders, independently, 14/14.

```
every fixture spells :usr::Box   → a stone that EXCLUDES stdlib passes every row
no fixture has two SIBLING       → the join gap cannot appear
variants in one match
```

Both riders were honest and thorough; the sonnet rider flagged its own STOP violation prominently
rather than burying it, and the grok re-run independently confirmed the scope cut was load-bearing.
**The acceptance criteria were the defect.** Fourth instance today of a row that cannot fail.
`[[feedback_a_green_test_can_prove_nothing]]` ·
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

⚠ **And my brief was internally contradictory**: it predicted *"a non-zero first count — the
fail-count is the progress meter, not a gate"* while carrying STOP-1 *"if widening goes red, STOP."*
The rider had to choose which governed, and said so.
`[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

## What A-2 needs before it can be re-drawn

```
1  a RULING on the join       what is the type of a form whose branches produce different types?
                              Least upper bound over the subtype graph is the obvious answer and
                              it is a language change, not a fix.
2  fixtures that can FAIL     one spelling a stdlib enum (:wat::core::Option::Some) and one with
                              two SIBLING variants in a single match. Without these, any reland
                              passes for the same reasons this one did.
3  names minted through       identifier.rs, never rfind/rsplit.
   the one grammar
```
