# SCORE (orchestrator) — STONE P-3: the three positions agree, both directions

**Independent re-run.** Floor and clippy central.

```
FLOOR EXIT=0    Summary [ 190.262s] 5265 tests run: 5265 passed, 18 skipped
CLIPPY          the same 5 pre-existing dead_code items. No new ones.
```

## ★★★ Row 11, measured on my own re-run — this is the stone

Same name, same substrate, one line of difference:

```
NO user use!     resolve (call head) check=1    wall (annotation) check=1    is-type? false
WITH user use!   resolve (call head) check=0    wall (annotation) check=0    is-type? true
```

Three positions that gave three answers this morning now give one, in both directions. Neither half
changed **what** is a type; both changed **who is asked**.

## Row 9 was a SHAPE claim, verified by reading — the blanket did not return

`src/check.rs:15266`:

```rust
// SCOPE SELECTION, not the reserved-prefix SKIP RELAND-1 deleted.
// Every declaration is still validated; only the reference set differs.
// Prefix IS the scope: user source cannot define under `:wat::*` / `:rust::*`.
let scope_decls = |name: &str| -> &UseDeclarations {
    if crate::resolve::is_reserved_prefix(name) { stdlib_use } else { user_use }
};
```

Exactly one `is_reserved_prefix` in the validator, inside a set-selecting closure. **No `continue`.**
`[[feedback_a_rejected_option_returns_in_new_clothes]]` — the row existed because the same predicate
returning could re-ship the blanket while passing every behavioural bar. It did not.

## ★ The rider closed a question my DESIGN could only ASSERT

I wrote: *"prefer a shape that derives the scope from the declaration's own origin rather than its
name prefix, if one can be found."* It went and looked, and reported the finding:

```
TypeDef            no origin field
Function           no origin field; `synthesized_for` is a companion mark, not a scope
Privilege          a registration-time GATE, not stored
```

**So prefix IS the scope — enforced by an existing wall (user source cannot define under `:wat::*`),
not adopted as a convention.** That is the derivation the design asked for and could not supply.
`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

And it closed trap-door 2 by looking rather than assuming: generated companions take the type's own
FQDN (`:usr::Point/x`), which is not reserved, so they take `user_use`. The shape I feared **does
not exist**, reported as an absence it searched for.

## ⚠ Two residues, named because they are structural

**① The verb has no scope to be aware of.** The wall is a compile-time, per-declaration check that
now knows which scope declared what. `is-type?` is a runtime query against one `TypeEnv`. So for a
*stdlib* annotation of `:rust::sqlite::Connection` the wall says yes and the verb says false — not
two answers to one question, but the honest per-program answer to a differently-scoped one. Inherent,
not a miss; recorded so nobody later reads it as a gap.

**② The wall and the verb still do not share ONE predicate.** The rider's own note, and it is right:
the verb must NOT ask `covers(stdlib)`, or stdlib `use!` leaks into `is-type?` and P-2's STOP-1
fires. They share `is_subtype_parent` and `contains`, not a single function. **A fifth store would
have to change both sites.** That is the standing cost of this shape and it is the thing to watch.

## Rows

| # | expected | actual |
|---|---|---|
| 1 | no user `use!` → EXIT 1, names the path | ✓ |
| 2 | with user `use!` → EXIT 0 | ✓ |
| 3 | ⛔ the stdlib still loads | ✓ `"loaded"` |
| 4 | derive marker → `true` | ✓ |
| 5 | ⛔ never-derived name → `false` | ✓ |
| 6 | `test(p3_one_question)` 5 passed, 0 skipped | ✓ |
| 7 | `test(p1_annotation)` 10 passed, 0 skipped | ✓ |
| 8 | `test(p2prereq)` 4 passed, 0 skipped | ✓ |
| 9 | no skip arm; scope-selection comment | ✓ **verified by reading the diff** |
| 10 | corpus unmoved | ✓ 845 files, 0 refusals — the census held |
| 11 | the three agree, both ways | ✓ **re-measured myself, above** |
| — | FLOOR | ✓ 5265/5265 |

All five STOPs held. STOP-5 held without being tempted: `derive` untouched.

## Disposition

**LANDED.** One question, one answer.

## ⬜ Carried, not absorbed

```
derive's MARKER          `(derive :usr::A :usr::Typo)` mints a new marker silently. Under the open
                         `derive`/`isa?` reading that is the mechanism; the hazard is a MISTYPED
                         parent. Declared-first vs open-minting is the builder's ruling.
the :wat::* call-head    src/resolve/walk.rs:272 — arc 255's founding defect, still standing for
blanket                  CALL HEADS. It gates the dot-notation flip.
5 dead_code items        pre-existing, already on origin/main, `-D warnings` only. A purgare stone.
the prefix-as-scope      load-bearing BECAUSE origin is not stored. If a generated function ever
dependency               lands under `:wat::*` carrying a user-source annotation, this stone's
                         premise breaks silently. Measured absent today.
```
