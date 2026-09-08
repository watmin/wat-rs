# DESIGN — STONE P-2 PREREQ: one question, one answer

## Why

Stone Q shipped `:wat::runtime::is-type?` as *"one authority over three mechanisms."* Stone P-1's
RELAND-1 measured the question at **four stores** and widened the annotation wall to ask all four.
`is-type?` was not widened. So, on the pushed green tree `f302681e7`:

```wat
(:wat::core::use! :rust::sqlite::Connection)
(:user::f [c <- :rust::sqlite::Connection] …)      ;; the WALL accepts it
(:wat::runtime::is-type? :rust::sqlite::Connection) ;; -> FALSE
```

⛔ **Two contradictory answers to one question ship in one binary.** P-2 — *a variant is a type* —
rests on this verb: its acceptance rows are `is-type?` answers. A prerequisite, not a tidy-up.

## ★★★ The store it cannot see is two hand-curated, DISJOINT halves

```
is-type? :rust::crossbeam_channel::Sender   ->  true    typed into src/types.rs Group 3
is-type? :rust::sqlite::Connection          ->  false   use!'d in the same file
```

The registry answers differently for two `:rust::*` types whose only difference is whether someone
transcribed one into `register_builtin_types` — a list its own comment sources to *"a rider's
convergence on branch `arc109-type-refs-parked`."*
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

```
hand-list ONLY   :rust::crossbeam_channel::{Sender,Receiver}
                 annotated in wat/kernel/channel.wat, NEVER use!'d, and NOT in RustDepsRegistry —
                 `(use! :rust::crossbeam_channel::Sender)` is REFUSED: "rust symbol not available
                 in wat; declare it via its shim".
use!      ONLY   :rust::cache::Lru · :rust::sqlite::{Connection,ReadConnection}
```

⚠ **So the hand-list cannot be emptied.** The crossbeam rows are load-bearing and this stone keeps
them. Why crossbeam is absent from `RustDepsRegistry` is **UNMEASURED** — it is not this stone's
subject and must not be theorised about. `[[feedback_an_adjacent_implementation_is_not_the_subject]]`

## The shape — and why not the obvious one

Measured: `use_decls` is a **local** in `freeze/env.rs:258`, passed as an argument.
`SymbolTable` does not carry it, so `is-type?` — a runtime verb holding only `sym` — cannot reach
store 3 today.

| | **(A)** `SymbolTable` carries `UseDeclarations` | **(B)** `use!` seeds `builtin_names` |
|---|---|---|
| Obvious? | YES | **YES** — `builtin_names`' own field doc: *"Rust structs exposed to wat with no `TypeDef` to hold"*. A `use!`d foreign type is exactly that. |
| Simple? | **NO** — a new field on a struct freeze ships across a process fork; every construction site and the fork boundary widen | **YES** — one call where `use_decls` and `types` are already both in scope |
| Honest? | YES | **YES** — uniform for the whole `use!`-able population |
| Good UX? | neutral — the second store must still be threaded to every future asker | **YES** — `is-type?` correct with ZERO plumbing, and the wall's `use_decls.covers` becomes redundant |

**(B), 4/4.** It **deletes a store** instead of propagating it — never construct the situation that
needs the patch. And it preserves the privilege `register_builtin_leaf`'s doc protects
(*"Not `pub`: only `register_builtin_types` seeds these"*), because **`use!` cannot fabricate a
name**: it already refuses anything `RustDepsRegistry::has_type` does not know.

## The one contract decision — pinned

> **Membership is seeded from THIS PROGRAM'S `use!` declarations, never from the build-time
> registry.** `use!` is a per-program declaration; a program that did not declare a foreign type
> does not have it — the same rule `resolve/walk.rs` already enforces for `:rust::*` call heads.

## Where

`src/freeze/env.rs:254-265` — the block that already collects `use_decls` from the residue and then
calls `validate_named_type_annotations(&types, &symbols, &use_decls)`. Both stores are in scope at
exactly that point.

## Out of scope — rejected, not deferred

- **Deleting the hand-list.** Measured impossible for the crossbeam pair; the rows stay.
- **Why crossbeam is not in `RustDepsRegistry`.** Unmeasured; its own question.
- **`derive` not validating its MARKER** (`src/types.rs:3410-3421` calls `register_subtype` with no
  membership check, so store 4 admits any name anyone derived from). Recorded at
  `NOTE-is-type-shares-the-blindness-…`; untested-as-written; not this stone.
- **The 5 pre-existing `dead_code` items** under `-D warnings`. Their own `purgare` stone.
- **P-2 itself.** This is its prerequisite.

## The probe

`tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union.rs` — committed, THREE controls green
and ONE subject `#[ignore]`d. Its isolation is the point: `use_then_is_type` and
`no_use_is_not_a_type` are **the same name in two programs**, differing only by the `use!` line, so
a fix that seeds from the build-time registry flips both and the detector catches it.
