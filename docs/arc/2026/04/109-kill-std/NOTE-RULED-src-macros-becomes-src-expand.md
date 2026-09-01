# NOTE — ⊘ RULED: `src/macros/` becomes `src/expand/`; `crates/wat-macros` keeps its name

> **Builder, 2026-09-01:** raised the collision — *"we need to rename `wat-rs/crates/wat-macros`…
> that's where `wat-rs/src/macros` will land… or is there not a conflict, and macros just are
> macros?"* → after the reasoning below: ***"expand has been reasoned… nice."***
>
> **The name is decided. The rename is NOT scheduled** — see *When*, below.

## The two things, and why they are genuinely different

```
crates/wat-macros    proc-macro = true. 8 exports:
                     #[wat_intrinsic] #[wat_special_form] #[wat_special_form_impl]
                     #[wat_value] #[wat_dispatch] #[restricted_to] · wat::main! · wat::test!
                     → RUST macros that BUILD the substrate. Audience: substrate developers.

src/macros/          error · error_edn · eval · expand · parse · registry   (8 files, ~5,060 lines)
                     → WAT's macro ENGINE: defmacro registration, expansion, macro errors.
                     Audience: the language.
```

Both are called "macros" and neither is wrong. The collision arrives when `src/<home>/` becomes
`crates/<crate>/` (the builder's stated trajectory, ROAD step 2).

## ★★★ THE RULING, AND ITS REASON IS NOT THE COLLISION

**`src/macros/` → `src/expand/` → eventually `crates/wat-expand`.**
**`crates/wat-macros` keeps its name.**

The collision is the *occasion*, not the argument. Two independent facts decide it:

**1. The repo already names passes with verbs and domains with nouns — and `macros` is the outlier.**

```
PASSES (verbs)   check · resolve · freeze · declare · load · lower
DOMAINS (nouns)  numeric · collection · holon · string · stream · types · value · edn · kernel · process
the outlier      macros
```

`src/macros/` **is a pass.** `src/freeze.rs:892` describes the pipeline as
*"(`register_defmacros` → `expand_all`)"*, and `freeze.rs:2588` pins a load-bearing phase-order
invariant: **`expand_all` precedes `register_defines`**. It runs over the whole program between
resolve and check. `macros` names what it OPERATES ON; every other pass here is named for what it
DOES.

**2. The codebase already calls this pass "expand" — 108 times.**

```
expand_all    66      the entry point
expand_form   30      the recursive step
ExpandBatch   12      the result type
────────────────
             108      versus "macros", which survives only as the folder name
```

★ **The rename does not impose a new name. It makes the module agree with its own API.**

**And `crates/wat-macros` is already idiomatic for ITS reader.** In Rust, `foo-macros` conventionally
means *"proc macros for foo"* (`tokio-macros`, `serde_derive`). A substrate developer reading
`crates/wat-macros` gets exactly the right expectation.

## ⛔ Why not the other direction

Renaming the proc-macro crate (`wat-proc-macros` / `wat-derive`) and letting `src/macros/` inherit
`wat-macros` is defensible — but it fights the **Rust ecosystem** convention to satisfy a **local**
one, and it leaves the pass still named for its object while every sibling pass is named for its act.
The chosen direction fixes an inconsistency that exists **independently of the collision**; the other
direction only moves the collision.

★★ And it makes the wrong thing **unrepresentable rather than managed**: two things called "macros",
disambiguated by prefix, stays a trap forever. One thing called macros is a wall.

## The honest counter-argument, recorded

A wat *user* asking "where do macros live?" looks for `macros/`. But they want the **language
feature** — `defmacro`, which lives in `wat/core.wat` and the user guide. They are not reading
`src/`. The crate name's audience is the substrate developer.

⚠ **And one genuinely arguable point:** `wat-expand` names the *pass*, but the crate would also hold
`registry.rs` (`MacroRegistry`) and `error.rs`. If the crate should mean "everything about wat's
macro system" rather than "the expansion pass", the other direction becomes the better answer. Ruled
toward `expand` because the pass IS the module's reason to change — registry and errors serve it.

## When — measured, and deliberately NOT now

```
rename cost      50 `crate::macros::` refs across 16 files · 18 more in tests   = 68 sites
behaviour        zero
liftability      macros -> runtime 28 refs, but 26 are FACADE artifacts
                 (`crate::runtime::X` reaching a `crate::value::` type)
                 runtime -> macros 18
                 → roughly TWO genuine references from acyclic
```

**Do it immediately before the crate migration, not now.** Three reasons:

1. 68 sites of pure churn would collide with every in-flight decomposition stone.
2. `src/macros/` does not move to `crates/` until the src decomposition finishes — the builder's own
   sequencing.
3. It is cheapest once the **facade re-point sweep** has already touched these files
   (`[[NOTE-the-crate-boundary-is-the-real-cut-and-eight-homes-are-cyclic]]`), which would also take
   this home from 28 back-edges to ~2 and make it liftable in the same motion.

⬜ **Not scheduled. The name is settled so that whoever meets the collision does not re-litigate it.**
