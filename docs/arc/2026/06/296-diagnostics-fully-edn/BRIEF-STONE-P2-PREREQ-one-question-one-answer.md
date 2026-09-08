# BRIEF — STONE P-2 PREREQ: one question, one answer

## The work, in one paragraph

`is-type?` and the annotation wall disagree about whether a `use!`d foreign type is a type. Make
`use!` register the name it imports as a builtin leaf at freeze, so the name is in `TypeEnv` itself
— then `is-type?` becomes correct with no plumbing, and the wall's fourth store collapses into its
first. Seed from **this program's `use!` declarations**, never from the build-time registry.

## Read in order

1. `tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union.rs` — **the committed probe. Read
   it first.** Three controls green, one subject `#[ignore]`d; its header carries every measurement
   this brief rests on.
2. `docs/.../DESIGN-STONE-P2-PREREQ-one-question-one-answer.md` — the four questions on the two
   shapes, and why (A) fails Simple.
3. `src/freeze/env.rs:254-265` — **the room.** The block already collects `use_decls` from the
   residue via `collect_use_declarations`, then calls
   `validate_named_type_annotations(&types, &symbols, &use_decls)`. `types` is mutable in this
   function and `use_decls` is complete at that point.
4. `src/types.rs:613-630` — `register_builtin_leaf`. Private, with two `debug_assert!`s that fire on
   double registration. Read both before you call it.
5. `src/types.rs:2268-2347` — `register_builtin_types` Group 3, the hand-list, including the two
   `:rust::crossbeam_channel::*` rows that **stay**.
6. `src/resolve/rust_use.rs:19-58` — `collect_use_declarations`; note it already refuses anything
   `registry.has_type` does not know, which is why seeding from it cannot fabricate a name.

## Implementation sketch

After `use_decls` is collected and before the annotation wall runs, seed each declared path into the
type registry as a membership-only leaf:

```
for path in use_decls.list() {
    types.register_use_declared_leaf(path);   // membership only; `get` stays None
}
validate_named_type_annotations(&types, &symbols, &use_decls)?;
```

`register_builtin_leaf` is private and `debug_assert!`s on a duplicate. Give the seed path its own
small entry point that is **idempotent** — a name already present is a no-op, not a panic. The two
populations are disjoint today, so nothing collides now; a seed that panics on overlap is a
landmine for the first name that ever joins both.

⚠ Keep `get` returning `None` for these names. A `:rust::*` type has membership, not structure, and
`builtin_names`' field doc is explicit that fabricating a `TypeDef` was considered and rejected.

Once the name is in `TypeEnv`, `first_unknown_named_type`'s `use_decls.covers(p)` arm is redundant.
**Remove it and drop the now-unused parameter** if the probe and the P-1 probe both stay green —
that collapse is half the point of this shape. If removing it turns anything red, STOP and report:
that means the two stores are not equivalent and the design is wrong.

## Acceptance

```
cargo nextest run --release -E 'test(p2prereq)'        4 passed, 0 skipped   (subject un-ignored)
cargo nextest run --release -E 'test(p1_annotation)'   10 passed, 0 skipped  (P-1 must not regress)
```

Per-fixture, with the freshly built binary:

```
…__use_then_is_type.wat        stdout `true`    ← the subject; false today
…__no_use_is_not_a_type.wat    stdout `false`   ← MUST NOT FLIP
…__handlist_control.wat        stdout `true`
…__phantom_rust_name.wat       stdout `false`
```

## Blast radius

`src/freeze/env.rs`, `src/types.rs` (one new idempotent entry point), and — if the collapse holds —
`src/declare/typevar.rs` + `src/check.rs` losing the `use_decls` arm and parameter. No change to
`use!`'s grammar, to `resolve`'s call-head coverage rule, or to the hand-list's rows.

## STOP triggers — each is a REJECTION

**STOP-1.** If `no_use_is_not_a_type` turns `true` — STOP. That is the over-reach: membership seeded
from the build-time registry rather than from this program's declarations. It is the exact error the
two-fixture isolation exists to catch, and the fix is not to adjust the fixture.

**STOP-2.** If seeding requires making `register_builtin_leaf` `pub`, or removing either
`debug_assert!` — STOP and report. Those guard a deliberate privilege boundary; a new narrow entry
point beside them is the intended shape.

**STOP-3.** If removing the `use_decls.covers` arm turns anything red — STOP, restore the arm, and
report what went red with its verbatim block. That is a finding about the two stores not being
equivalent, and it outranks the collapse.

**STOP-4.** If any hand-list row must be deleted or edited to make this work — STOP. The crossbeam
pair cannot be `use!`d (`RustDepsRegistry` refuses it) and its rows are load-bearing.

## Tier

You edit and report. Run the two targeted probe binaries and the four per-fixture stdout checks.
**The orchestrator runs the floor and clippy centrally, once, after the tree is quiescent.** Report
what only you can: which sites you inspected, whether the collapse held, and what surprised you.
