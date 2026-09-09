# BRIEF — STONE A-1: one rule for "can this value go here"

## The work, in one paragraph

`if` and `send` decide whether a value fits by **unifying**; an ordinary parameter decides by
**subsuming** (`assignable`, which walks the subtype graph). So the same two types get opposite
answers in the same program. Make all three ask one question, under the ruled rule: **unify when
either side is still a type variable; subsume when both are concrete.**

## Read in order

1. `tests/types/probe_arc296_a1_one_rule_for_assignability.rs` — **the committed probe, first.**
   Three controls green, one subject `#[ignore]`d. Its header carries every measurement and names
   the ruled contract and its admitted cost.
2. `docs/.../DESIGN-STONE-A1-one-rule-for-assignability.md` — why three rival shapes collapsed.
3. `src/check.rs:8143` `infer_if` — *"The form's type is unify(then, else)"*, stated in its own
   comment. This is one of the two sites.
4. `src/check.rs:11619` `infer_send_prime` — the payload unification. The second site.
5. `src/check.rs:16960+` `assignable` — **the rule that already exists.** Read its arms; this stone
   reuses it, it does not restate it.
6. `src/declare/typevar.rs` header — stone 251.8a collapsed four hand-rolled versions of one
   question into a single door. That precedent governs here.

## Implementation sketch

At each of the two sites, in place of a bare `unify`:

```
if either side still contains a type VARIABLE  ->  unify   (solve it, exactly as today)
else                                           ->  assignable, in the direction the position means
```

For `if`, the position's meaning is the part to get right and to state in a comment: the form's type
is the one both branches can be seen as. For `send`, the payload is being supplied TO a declared
input type, which is the same direction an ordinary parameter already uses.

⛔ **Reuse `assignable`.** Do not add a second subtyping test anywhere. If `assignable` needs a new
arm to serve these positions, that is a finding to report before writing it, not a thing to add
quietly — a fifth spelling of one question is the defect this stone removes.

## Acceptance

```
cargo nextest run --release -E 'test(a1_one_rule)'   4 passed, 0 skipped   (subject un-ignored)
```

Per fixture:

```
…__parameter_subsumes.wat            --check EXIT 0     control
…__if_branches_subtype_related.wat   --check EXIT 0     the subject; EXIT 1 today
…__if_branches_unrelated.wat         --check EXIT 1     over-reach detector
…__if_still_solves_a_type_var.wat    --check EXIT 0     THE CAPABILITY GUARD
```

And the earlier stones must not move:

```
-E 'test(p1_annotation)'   10 passed   ·   -E 'test(p2prereq)' 4 passed   ·   -E 'test(p3_one_question)' 5 passed
```

## Blast radius

`src/check.rs`, two call sites plus whatever shared helper the "has a variable?" test wants. No
change to `unify`, none to `assignable`'s existing arms, none to `resolve`, none to any `.wat`.

**The corpus may speak, and that is the point** — sites that currently unify may now widen where
they previously errored. That direction is strictly more permissive, so a NEW red is a finding
worth a verbatim block, not a number to absorb.

## STOP triggers — each is a REJECTION

**STOP-1.** If `if_still_solves_a_type_var` goes red — STOP and report verbatim. That is the
capability the ruling exists to protect, and it is the one failure that must never be traded away.

**STOP-2.** If `if_branches_unrelated` goes green — STOP. The rule became "accept anything."

**STOP-3.** If serving these positions needs a NEW subtyping test rather than `assignable` — STOP
and report what `assignable` could not answer. A second door is the defect, not the fix.

**STOP-4.** If the "has a variable?" test cannot be written without walking a type expression by
hand — STOP and report. `walk_type_expr` already exists in `src/declare/typevar.rs` and is the
shared recursion; a hand-rolled walk beside it is a fifth walker.

**STOP-5.** If `send`'s direction turns out to differ from a parameter's — STOP and report the
reason before changing it. The design asserts they are the same direction; if the substrate
disagrees, the design is wrong and I re-plan.

## Tier

You edit and report. Run the four fixtures and the four targeted probe binaries. **The orchestrator
runs the floor and clippy centrally, once, after the tree is quiescent.** Report which helper you
used for the variable test, whether `assignable` served both positions unchanged, and what the
corpus said.
