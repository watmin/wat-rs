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

## ⛔ CORRECTED — `role = eval` DOES emit a shim, and the eval fn's signature must change

I first wrote here that *"no dispatch shim is generated, for any role,"* reading the macro's header
paragraph and stopping there. **That is false, and it is the ③a-i failure class caught one step
earlier this time — before a rider was briefed on it rather than after.**

`crates/wat-macros/src/wat_special_form_impl.rs`, its own comment at the `role = eval` branch:

> *"arc 255 Stone the-eval-door — `role = eval` ALSO emits a callable pointer, so the registry's
> `handler` slot can dispatch this form directly. `role = check` keeps emitting source only — a
> check impl runs once, statically, and has no per-invocation call site to dispatch through."*

and the shim it generates calls the annotated fn with **four** arguments:

```rust
#fn_ident(args, list_span, env, sym)      // the canonical NativeHandler shape
```

```
role = check   source only, no shim   → `infer_program_self_peer`'s signature is UNCONSTRAINED ✓
role = eval    emits a shim           → the annotated fn MUST take (args, list_span, env, sym)
```

`eval_program_self_peer(args: &[WatAST], list_span: &Span)` takes **two**. Annotating it as-is does
not compile.

★ **The fix is mechanical and the call site is ready for it.** The macro's own note says an eval
impl's params are *"ALREADY in this exact order … no context-tail reordering to do"* — that is the
convention, and this fn predates it because it needs neither `env` nor `sym`. Widen it to the
canonical four with `_env` / `_sym` unused; its ONE caller, `dispatch_keyword_head_value`
(`src/runtime.rs:2381`), already holds both in scope — the enclosing fn's own parameters are
`(head, args, list_span, env, sym)`.

`sniff_return` then classifies its `Result<Value, EvalBreak>` as the bare-Value shape and the shim
wraps it to `TrackedValue`, the same fold `#[wat_intrinsic]` performs — nothing to invent.

★★ `macroexpand.rs`'s note that *"`role = eval` could NOT take the same shortcut"* is a THIRD
thing again: it is about STACKING two fqdns on one fn. `self-peer` is one fqdn with one eval fn and
needs no stacking. Three adjacent statements about `role = eval`, each true about something
different — which is why the codegen itself had to be read rather than any of them trusted.

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
**two attribute lines** on the two existing functions · **`eval_program_self_peer` widened to the
canonical `(args, list_span, env, sym)`** with `_env`/`_sym` unused, and its one call site updated.
No moves, no `TypeScheme`, no change to either arm's BODY.

## Out of scope = REJECTED

- **The other six families.** ⑤-i covers B/C/D; E and F+G are their own.
- **Retiring either arm.** `[[BRIEF-STONE-a-registered-row-may-not-keep-its-arm]]` is about rows
  whose dispatch the registry can take over. A special form is dispatched by the engine BY
  DEFINITION — the arms are the implementation the role pointers point AT.
