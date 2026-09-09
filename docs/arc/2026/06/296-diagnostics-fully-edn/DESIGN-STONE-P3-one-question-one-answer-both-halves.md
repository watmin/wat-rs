# DESIGN — STONE P-3: one question, one answer — both halves

## Why

Three positions in the substrate answer *"is this a type?"*, and on the pushed green tree
`680d6718e` two of them still disagree. Both disagreements are the same defect at two addresses.

```
HALF 1   same name, same program, NO user use!
         resolve   (call head)    :rust::sqlite::Connection::open   REFUSED
         is-type?                 :rust::sqlite::Connection         false
         the WALL  (annotation)   [c <- :rust::sqlite::Connection]  ACCEPTED   ⛔

HALF 2   is-type? :wat::spawn::Spawned  ->  false,  and the WALL accepts it as a bound   ⛔
```

## Half 1 — coverage is per-DECLARING-SCOPE, not per-program

`resolve` Pass 1 walks **user residue only**, so a stdlib `use!` never covers a user call head.
The annotation wall consults the **merged** `use_decls` — because RELAND-1 folded stdlib `use!`
into it so the stdlib's own annotations would stop screaming (838 files). That merge is precisely
what made the wall scope-blind, and it makes the annotation position *more permissive than the
call position for the same name in the same program*.

The rule, derived from what `resolve` already does:

> **A declaration's annotations are covered by the `use!` declarations of the scope that declared
> it.** A stdlib annotation by stdlib `use!`; a user annotation by user `use!`.

### ⚠ The predicate is `is_reserved_prefix` — and this is NOT the blanket returning

RELAND-1 deleted `if is_reserved_prefix(name) { continue; }` — a **SKIP**: those declarations were
not validated at all. This stone uses the same predicate to **select which set to validate
against**. Every declaration is still checked; only the reference set differs. A future reader
will see the name return and must be able to tell these apart, so the call site says so in
one line and the probe's header says it again.

⛔ If a shape can be found that derives the scope from the DECLARATION's own origin rather than
from its name prefix, prefer it and say so. The name-prefix reading is sound only because the
substrate already enforces that user source cannot define under `:wat::*` and the stdlib cannot
define outside it — the prefix IS the scope, by an existing wall, not by convention.

### Blast radius — MEASURED AT ZERO

Every non-stdlib `.wat` that annotates a `:rust::` type carries its own `use!` — all four
`wat_dispatch` fixtures do (`e4_shared.wat:3`, `e2_tuple`, `e1_vec`, `193a`). The only two hits in
a corpus sweep are a comment line in `wat-scripts/fixes/deprime-telemetry-sqlite.wat:13` and P-1's
own intentional refusal fixture. **This stone should turn nothing red that is not its subject.**

## Half 2 — `is-type?` asks store 4

```rust
// today, src/reflect/verbs.rs eval_is_type:
types.contains(&type_kw) || is_builtin_primitive(stripped)
```

`TypeEnv::is_subtype_parent` already exists (`src/types.rs:847`) and the annotation wall already
calls it. One disjunct, no plumbing.

`:wat::spawn::Spawned` exists ONLY as a derive parent (`wat/spawn.wat:235-236` — *"the owner-side
spawn-handle marker (typesub/derive axis; no methods)"*). There is no declaration form for a
marker: **deriving to it is what mints it**, which is Clojure's open `derive`/`isa?` hierarchy and
is what `subtype_edges`' own field doc cites.

### ⚠ Measured, and NOT this stone's to rule

```wat
(:wat::core::derive :usr::A :usr::TotallyMadeUpMarker)
(:wat::core::defn :user::f [m <- :usr::TotallyMadeUpMarker] -> …)   ;; --check EXIT 0
```

`derive` does not validate its marker. Two lines turn any phantom into an accepted type. Under the
open-hierarchy reading that is the mechanism working; the real hazard is that a **mistyped parent
silently mints a new marker** instead of relating to the intended one
(`[[feedback_a_wrong_name_does_not_fail_it_names_something_else]]`). Whether markers must be
declared-first is the builder's ruling. **It does not gate this stone in either direction**: one
question, one answer says the verb must agree with the wall, and it must agree whichever way that
rules.

## The one contract decision — pinned

> **Neither half changes what is a type. Both change WHO IS ASKED.** Half 1 narrows the wall's
> reference set to the declaring scope; half 2 widens the verb's store set to the wall's. After
> this stone the three positions agree in BOTH directions — refusing together without a `use!`,
> accepting together with one.

## Out of scope — rejected, not deferred

- **`derive` validating its marker.** Measured above; the builder's ruling; orthogonal.
- **The `:wat::*` call-head blanket** (`src/resolve/walk.rs:272`). Arc 255's, untouched.
- **The 5 pre-existing `dead_code` items.** Their own `purgare` stone.
- **Deleting the `:rust::crossbeam_channel::*` hand-list rows.** Measured impossible — not in
  `RustDepsRegistry`, so not `use!`-able.

## The probe

`tests/types/probe_arc296_p3_one_question_one_answer.rs` — committed, THREE controls green and TWO
subjects `#[ignore]`d. The widest control is `the_stdlib_still_loads`: if scope-awareness is drawn
wrong, nothing loads at all, and that fixture is the first to say so rather than the last.
