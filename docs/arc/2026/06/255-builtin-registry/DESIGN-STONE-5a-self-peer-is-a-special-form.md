# DESIGN — STONE ⑤-A: `:wat::program::self-peer` is a SPECIAL FORM

Group **A** of `DESIGN-STONE-5-the-blanket-dies-closing-the-seven.md`, and the largest single
obstacle left: 15 direct refusals and most of the 97 downstream, because `wat/bracket.wat` — the
STDLIB — calls it at three sites.

## The fourth store

```
runtime arm     eval_program_self_peer          src/runtime.rs:11014
checker arm     infer_program_self_peer         src/check.rs:10117    ← a LITERAL match arm
TypeScheme      none
registry row    none
```

Stone ③ derived membership from the scheme store, which closed 484 names. **This verb is in
none of those populations.** Its contract lives in a literal `match` arm inside the checker's
inference — a store nothing can enumerate and no fold can reach.

⛔ **Do NOT give it a `TypeScheme`.** It already has an inference arm; a scheme would be a second
authority for one question, which is the defect this whole arc exists to end.

## Why a special form, and why this one lands where ③a-i could not

Both arguments are **type-shaped**, not values — `eval_program_self_peer` gates on
`is_type_arg_shaped` and refuses anything else. A form whose arguments are types is syntax, not a
call. `#[wat_special_form]` is exactly that shape.

★ Stone ③a-i was refused by `every_special_form_carries_check_and_eval_impls`, which REQUIRES a
registered special form to carry `#[wat_special_form_impl]` pointers for `role = check` and
`role = eval`. There, no such implementations existed. **Here both already exist as functions**,
at the two lines above. That wall is the reason this stone is small and that one was not.

★★ And the macro is lighter than ③a-i's failure suggested. `crates/wat-macros/src/wat_special_form_impl.rs`'s
own header: it captures `quote!(#item).to_string()` into a `source` field, **"the fn passed through
completely unchanged"**, plus an `inventory::submit!` recording the `(fqdn, role)` key. **No dispatch
shim is generated, for any role**, so neither existing fn's signature has to change and neither has
to move. `macroexpand.rs`'s note that *"`role = eval` could NOT take the same shortcut"* is about
STACKING two fqdns on one fn; `self-peer` is one fqdn with one eval fn and needs no stacking.

## The axes — four from `:wat::runtime::argv`'s precedent, one MEASURED past it

`:wat::runtime::argv` (`src/runtime.rs:11114`) is the same shape — a verb that reads an ambient
runtime value — and is already argued:

```
argv        @Purity Pure · @Determinism Deterministic · @Totality Unreviewed
            @ExpandTime Unreviewed · @Category Ambient
```

```
self-peer   @Category      Ambient       reads `services::current_self_peer()` — a runtime
                                         ambient, not a fact about a value the caller holds.
                                         `:Ambient`'s own prose, argv's ground.
            @Purity        Pure          reads the ambient; no mutation, no I/O. It does NOT
                                         create the peer — `current_self_peer()` RETURNS an
                                         existing one. Same ground as argv.
            @Determinism   Deterministic the ambient is installed once per locus and read many;
                                         within a locus the answer does not move. argv's ground.
            @Totality      Partial       ⭐ MEASURED, not deferred. Outside a spawned locus it
                                         RAISES: `MalformedForm — "no self-peer — only valid
                                         inside a spawned process service; root has no
                                         owner-link"`, exit 1. argv left this `Unreviewed`;
                                         here the failure path was read AND run, so `Unreviewed`
                                         would be the dishonest pole.
            @ExpandTime    RuntimeOnly   `:RuntimeOnly`'s own definition — "needs state that does
                                         not exist yet at expand time" — describes this verb
                                         literally. argv left this `Unreviewed` too.
```

★ Two of the five are STRONGER than the precedent because they were measured rather than inherited.
That is the direction an axis claim should move; copying `Unreviewed` forward would have been
copying a deferral.

## `@example` must be `-norun`, and the reason is measured

The verb raises outside a spawned process service — measured above. No inline example can execute,
so `@example-norun` is the honest form, with that raise as its stated reason. `:wat::kernel::raise!`
uses the same escape for the same kind of reason. `[[NOTE-a-norun-example-asserts-nothing]]` stands
and is why the reason must name the measurement rather than gesture at difficulty.

`@arg` is OPTIONAL for a special form — `:wat::core::let` carries `@syntax` and `@ret` and no
`@arg`, because its arguments are not a fixed value list. `self-peer`'s are types. Use `@syntax`.

## Blast radius

One new `src/intrinsic/special/program_self_peer.rs` (unit struct + doc contract) · one `mod` line ·
**two attribute lines** on the two existing functions. No signature changes, no moves, no
`TypeScheme`, no change to either arm's body.

## Out of scope = REJECTED

- **The other six families.** ⑤-i covers B/C/D; E and F+G are their own.
- **Retiring either arm.** `[[BRIEF-STONE-a-registered-row-may-not-keep-its-arm]]` is about rows
  whose dispatch the registry can take over. A special form is dispatched by the engine BY
  DEFINITION — the arms are the implementation the role pointers point AT.
