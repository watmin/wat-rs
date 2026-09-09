# BRIEF — STONE P-1b: a parametric annotation's HEAD is a named type too

## The work, in one paragraph

P-1 validates that an annotation names a real type — but only for the bare spelling. A **parametric**
annotation's head is never checked, so `(:usr::TotallyMadeUp :- [:wat::core::i64])` passes while
`:usr::TotallyMadeUp` is refused. One traversal is blind in one place; make it see the head, without
adding a second walker.

## Read in order

1. `tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type.rs` — **the committed probe,
   first.** Three controls green, one subject `#[ignore]`d; the header carries the measurement, the
   cause, and why the fix must not be a new walker.
2. `src/declare/typevar.rs` `walk_type_expr` — the traversal. Its `Parametric` arm is
   `{ args, .. }`: the head is destructured away. Note it ALREADY gained a `visit_var` callback in
   A-1 — that is the precedent for how to extend it.
3. `src/declare/typevar.rs` `first_unknown_named_type` — P-1's consumer, the visitor that must now
   receive the head.
4. `src/declare/typevar.rs` `collect_free_type_vars` / `walk_free_type_vars` — the OTHER consumer.
   It must keep ignoring the head: a head is not a type variable, and making it one would
   auto-generalize every parametric annotation in the corpus.

## Implementation sketch

Give `walk_type_expr`'s `Parametric` arm a visit of its head, delivered through a callback the
free-var caller ignores — the same shape `visit_var` already uses. Then
`first_unknown_named_type` treats a head exactly as it treats any other named path: type variable →
accept; bound param → accept; in the four-store union → accept; otherwise refuse.

⛔ **Do not add a second walker, and do not special-case the head inside the consumer.** One
recursion, more callbacks. `typevar.rs`'s own header records stone 251.8a collapsing four hand-rolled
versions of this question into one door.

⚠ A head is a Path in a position where an *arity* is also implied. If the head needs different
treatment from an ordinary path (e.g. a parametric head that is itself a bound type parameter),
that is a finding to report, not a special case to bury.

## Acceptance

```
cargo nextest run --release -E 'test(p1b_a_parametric)'  4 passed, 0 skipped   (subject un-ignored)
```

```
…__parametric_head_phantom.wat   --check EXIT 1, message names :usr::TotallyMadeUp
…__bare_head_phantom.wat         --check EXIT 1
…__parametric_arg_phantom.wat    --check EXIT 1
…__parametric_head_real.wat      --check EXIT 0    ⛔ the widest control
```

And the earlier stones must not move:

```
-E 'test(p1_annotation)' 10 · -E 'test(p2prereq)' 4 · -E 'test(p3_one_question)' 5 · -E 'test(a1_one_rule)' 4
```

## Blast radius

`src/declare/typevar.rs`, plus whatever `first_unknown_named_type` needs to receive the head. No
change to `assignable`, `unify`, `resolve`, or any `.wat`.

⚠ **The corpus will speak and I expect it to.** Every generic annotation in 845 files now has its
head checked for the first time. A red naming a REAL type is a finding about the union; a red naming
a phantom is the wall working. Classify by REASON before reporting any delta.

## STOP triggers — each is a REJECTION

**STOP-1.** If `parametric_head_real` goes red — STOP. Every generic in the corpus is a parametric
annotation; that row failing means the stone broke the world rather than closing a hole.

**STOP-2.** If the fix requires a second walker, or a head special-case inside a consumer — STOP and
report. The blind spot came from sharing a traversal; a fork of it is not the cure.

**STOP-3.** If `collect_free_type_vars` starts collecting heads — STOP. That would auto-generalize
every parametric annotation in the corpus, silently, and the floor might not even notice.

**STOP-4.** If a corpus red names a name you believe IS a real type — STOP and report it verbatim.
That is a finding about which store the union is missing, exactly as RELAND-1 was, and it is mine to
re-plan rather than yours to route around.

## Tier

You edit and report. Run the four fixtures and the five targeted probe binaries. **The orchestrator
runs the floor and clippy centrally, once.** Report which callback shape you used and what the
corpus said, classified by reason.
