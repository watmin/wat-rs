# DESIGN — STONE A-1: one rule for "can this value go here"

## Why — a defect today, with no variants anywhere near it

Measured at `4d1720f9f`, plain records, one `derive` edge:

```
(:usr::Child 1) → [p <- :usr::Parent]        check=0    the parameter SUBSUMES
(if b (:usr::Child 1) (:usr::Parent 2))      check=1    ⛔
   ":wat::core::if: parameter else-branch expects :usr::Child; got :usr::Parent"
```

**The same two types, the same program, opposite answers.** `assignable` (`check.rs:16960+`) walks
`is_subtype`; `infer_if` (`check.rs:8143`) says so in its own comment — *"The form's type is
unify(then, else)"* — and `unify` knows nothing about the subtype graph. `infer_send_prime`
(`check.rs:11619`) unifies its payload identically.

★ The diagnostic is not even symmetric: `if` treats the THEN branch as authoritative and names the
ELSE branch as the offender. Swap them and the message swaps.

This is P-1/P-2prereq/P-3's disease one layer down — those closed *"is this a type?"* answered
differently by different STORES; this is *"can this value go here?"* answered differently by
different POSITIONS. Arc 209 already ships depending on the subsuming half.

## The one contract decision — RULED BY THE BUILDER, 2026-09-08

> **Unify when either side is still a type VARIABLE; subsume when both are concrete.**

Unification is not only a check — it is **how a type variable gets solved** from the branches.

Three rival shapes were run against the four questions and collapsed:

```
always subsume                 removes variable solving. Every test green, inference weakened.
try assignable, else unify     LOOKS simpler — but assignable(T, Box) on an unsolved T succeeds
                               WITHOUT binding T, so it reduces to the above unless guarded by
                               "both concrete" — at which point it IS the ruled shape.
subtyping-aware unify          changes `unify` itself, the one function everything depends on.
                               FAILS Simple.
```

The status quo fails **Honest**: it makes the substrate assert that `Child` and `Parent` are
unrelated at `if` while `assignable` says one is a subtype of the other at a parameter.

## ⚠ The ruling's admitted cost, recorded so it is never read as a bug

A partially-concrete pair — `Box<T>` with `T` unsolved, meeting `Box.Full<i64>` — takes the unify
path and does **not** widen. An annotation is the fix. The builder was shown this before ruling.

## Where

```
src/check.rs:8143     infer_if           unify(then, else)
src/check.rs:11619    infer_send_prime   unifies payload against I
src/check.rs:16960+   assignable         the subsuming rule that already exists — the model to reuse
```

⛔ **Reuse `assignable`; do not write a second subtyping test.** Stone 251.8a's precedent stands:
four hand-rolled versions of one question were collapsed into one door, and `typevar.rs`'s header
records it. A fifth rule here would be the same defect this stone exists to remove.

## Out of scope — rejected, not deferred

- **Variants as types.** A-2, and this stone is its prerequisite. A-1 is verifiable and valuable
  with zero variant machinery, which is exactly why it is separate.
- **A (Parametric, Parametric) arm in `assignable`.** Needed by A-2 for `Box.Full<T> → Box<T>`;
  not needed here, where the isolation is Path-to-Path.
- **`{:keys}` widening** and **`defclause` routing** — A-2's, both.
- **Any change to `unify` itself.** Rejected above on Simple.

## The probe

`tests/types/probe_arc296_a1_one_rule_for_assignability.rs` — committed, THREE controls green and
ONE subject `#[ignore]`d. Two of the controls carry the weight:

- **`if_still_solves_a_type_variable_from_its_branches`** — the CAPABILITY GUARD, and the reason the
  ruled rule is conditional. `None` leaves `Option`'s `T` unsolved; `Some` pins it; unification is
  what solves it.
- **`if_still_refuses_unrelated_branch_types`** — the over-reach detector. A stone that made `if`
  accept anything would satisfy the subject and fail only here.
