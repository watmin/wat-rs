# BRIEF — STONE A-2 RELAND-1: the join, and no scope cut

## The work, in one paragraph

RELAND-0 made variants types and hit two walls: sibling variants have no **join**, and the escape it
took — excluding `!is_reserved_prefix` enums — cut the capability for `Option`/`Result` while
relocating the same failures into user code. This reland adds the join **the builder has now ruled**,
applies variant types to **every** enum with no scope cut, and mints variant names through the one
name grammar.

★ **Start from `a75bc4ce2`.** That commit preserves RELAND-0's work: variant registration, the
head-level edge, `{:keys}` widened by shape, the ctor's `ret_type`, and the second erasure it found
in `:wat::core::variant`'s own inference. All of that was sound. Read its diff first; you are adding
the join and removing the cut, not starting over.

## ⛔ THE RULED CONTRACT — builder, 2026-09-08

> **The join of two types with the same head is that head applied to the pairwise joins of its
> arguments. The join of two variants of the same enum is that enum. Otherwise there is none.**

```
join(Option::Some, Option::None)            =  Option
join(Option::Some, Option)                  =  Option        (A-1's subsumption already covers this)
join(RecvOutcome<Some>, RecvOutcome<None>)  =  RecvOutcome<Option>
join(Option::Some, Result::Ok)              =  NONE → error, exactly as today
join(i64, String)                           =  NONE → error, exactly as today
```

★ **The structural rule is what avoids variance.** Both sides widen to the *same* type, so they
become identical and unification succeeds — nothing ever has to accept a subtype in an argument
position. RELAND-0 retreated from "invariant type args"; this contract makes that retreat unnecessary
rather than overturning it.

⛔ **The join does NOT search the subtype graph.** `subtype_edges` mixes `derive` membership with
`extend-type` protocol satisfaction (measured: 24 of 34 edges are protocol), so a general
least-upper-bound over it would return whatever protocol two types happen to share — an answer to a
different question. A variant's enum comes from variant registration, not from that map, and it is
the only parent the join consults.

## Read in order

1. `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` — **the probe, first.** NINE rows now. The
   three added rows are the ones RELAND-0 lacked and are the reason it shipped a scope cut nobody's
   tests could see.
2. `docs/.../NOTE-sibling-variants-have-no-JOIN.md` — the 103-failure floor, classified.
3. `git show a75bc4ce2` — RELAND-0's preserved implementation.
4. `src/check.rs` `join_if_branches` (A-1) — where the join belongs; it currently tests one
   directional `assignable`.
5. `src/check.rs` `infer_match` — *"the result type is inferred by unifying the arm bodies (like
   `if`)"*. 277 of the 600 failures came through here, so match arms need the same join.
6. `crates/wat-reader/src/identifier.rs` — **the one name grammar.** Every variant FQDN is minted
   through its accessors.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'   9 passed, 0 skipped   (all subjects un-ignored)
```

```
…__process_full_box.wat              EXIT 0    …__variant_widens_to_enum.wat        EXIT 0
…__ctor_carries_the_variant.wat      EXIT 0    …__match_still_works.wat             EXIT 0
…__stdlib_enum_variant.wat           EXIT 0  ← the scope-cut detector
…__sibling_variants_join.wat         EXIT 0  ← green NOW; must stay green
…__sibling_variants_join_in_match.wat EXIT 0 ← green NOW; must stay green
…__nonexistent_variant.wat           EXIT 1
…__enum_does_not_narrow.wat          EXIT 1, message must NOT contain "UnknownNamedType"
```

Earlier stones: `-E 'test(p1_annotation)'` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 ·
`p3_one_question` 5 · `a1_one_rule` 4.

## STOP triggers — each is a REJECTION

**STOP-1.** If you find yourself scoping variant registration by namespace, prefix, or any
"user-only" predicate — **STOP**. That is what RELAND-0 did; the floor showed it relocated the same
failures into `:probe::`, `:usr::` and `:arena::` while cutting `Option`. Report what forced it.

**STOP-2.** If either join control (`sibling_variants_join`, `…_in_match`) goes red — STOP with the
verbatim block. Those are green today; breaking them is the 600-failure regression.

**STOP-3.** If a variant FQDN is built with `rfind("::")`, `rsplit`, `format!` concatenation, or any
hand-rolled string surgery — **STOP**. `crates/wat-reader/src/identifier.rs` is the one grammar; a
lint enforces it after a census found 33 sites that had already disagreed. RELAND-0 tripped it.

**STOP-4.** If the join needs to search `subtype_edges` for a common ancestor — STOP and report. The
ruled contract is structural and consults a variant's own enum only.

**STOP-5.** If `src/record/construct.rs` must change — STOP. RELAND-0 proved this is checker-side.

## ⚠ Expect a large first failure count, and that is fine

The builder has ruled: *"if it's a hard migration… we've done those many times over."* RELAND-0's
first honest measurement was 1228. **The count is the progress meter, not a gate** — but classify the
remainder **by REASON** with one verbatim block per cluster before reporting any delta. A falling
total says nothing about kind.

## Tier

You edit and report. Run the nine fixtures and the six targeted `-E` filters. **You do NOT run
`scripts/floor.sh` or clippy** — the orchestrator runs those centrally, once. Report what only you
can: where you put the join, whether match and `if` share it, and the corpus response by reason.
