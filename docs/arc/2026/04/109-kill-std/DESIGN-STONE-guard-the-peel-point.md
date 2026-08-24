# DESIGN — STONE: guard the peel point (too many type arguments is silently swallowed)

> **Builder, 2026-08-24:** *"why did eval even accept the `:- [...]` fields at all?"*
>
> **Answer, measured: nobody decided it.** `src/check.rs:4977` peels the param-spec at the top of the
> keyword-head call path, **unconditionally, for every call**, before any dispatch. Its own comment
> says so: *"`args` shadows for every arm below (the surface-method arm, defclause dispatch, and the
> generic-call arm) — none of them extracts the marker itself anymore."* `eval-ast!` was never
> considered. It accepts a binder because it is a call.

## The defect, in one sentence

**The binder is peeled universally and validated nowhere.**

Measured this session, both live:

```
(:wat::eval-ast! :- [:wat::core::i64 :wat::core::String :wat::core::bool] e)
    → Ok [42]        ← eval-ast! declares ONE type param. Three given. Two silently dropped.

(:wat::eval-edn! :- [:wat::core::i64 :wat::core::String :wat::core::bool] "42")
    → Ok [42]        ← eval-edn! declares ZERO. Three given. All three silently dropped.
```

`instantiate_with_args` (`check.rs:16194`) is the whole story:

```rust
if scheme.type_params.is_empty() {
    return (scheme.params.clone(), scheme.ret.clone());   // type_args NEVER READ
}
for (i, tp) in scheme.type_params.iter().enumerate() {
    let bound = type_args.get(i).cloned().unwrap_or_else(|| fresh.fresh());
```

It iterates the **params** and indexes into the **args**. Extras are unreachable by construction.

| supplied vs declared | behaviour | verdict |
|---|---|---|
| fewer | missing become fresh inference vars | **correct** — partial application; inference completes it |
| **more** | extras never read | **silently swallowed** |
| any, against zero params | early return | **silently swallowed** |

## ★ This class has bitten before, and the record says so

`check.rs:4977`'s comment, written during arc 109:

> *"which is exactly how 'the call position' grew two implementations: emitting the binder at a
> surface-method call gave `ArityMismatch: expected 7 argument(s); got 9` because that arm never
> peeled the marker out of `args` at all."*

Third instance now: **surface-method calls** (fixed by hoisting the peel to one place) →
**the runtime's ten root-level eval forms** (fixed today, `STONE-the-binder-must-be-universal`) →
**this**. Each time, the binder was validated where it was WRITTEN and not where it LANDS.
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## The fix — one condition, at the one consumer

`src/check.rs:5635` is the ONLY place `type_args` reaches `instantiate_with_args`:

```rust
let (param_types, ret_type) = match &type_args {
    Some(concrete) => instantiate_with_args(scheme, concrete, fresh),
    None            => instantiate(scheme, fresh),
};
```

**Guard it:**

```
if concrete.len() > scheme.type_params.len()  →  REFUSE, and NAME the callee
```

That single comparison covers both swallow cases — zero-params-with-args is just `M > 0`. Fewer than
declared stays legal, because inference legitimately completes a partial application.

**Error kind: `CheckErrorKind::MalformedForm`**, matching the sibling error the peel point already
emits ~40 lines above for a *malformed* type-param argument (`check.rs:4986`). Same site, same
family, no new variant. The `reason` must state both counts and the callee.

**`:- []` STAYS LEGAL EVERYWHERE.** `:- []` ≡ absent is arc 109's ruling and macros emit it
unconditionally — `0 > N` is false for every N, so the guard admits it by construction rather than
by a special case. That is the point of writing it as `>`.

## What this would have prevented

Every thread pulled today: `eval-ast!` swallowing three args for one param; `eval-edn!` swallowing
three for zero; and — the one that matters — **eval's `∀T` would have had to justify itself the
first time anyone wrote a binder on it**, instead of quietly absorbing whatever arrived.

## Out of scope — affirmatively cut, with a named home

**Whether `eval-ast!` should be generic AT ALL.** Traced this session: arc 028 (2026-04-23) shipped it
`type_params: vec![]` — NOT generic. The `∀T` arrived at `a33642acf` (2026-04-29, *"arc 102 slice 1:
revert arc 066 — eval-ast! returns bare Value"*) as the residue of un-wrapping a return value, and
its own comment concedes the shape: *"Same trust-the-caller discipline… the caller annotates T with
the type they expect… Type-mismatched downstream ops fail at runtime."* A `T` unconstrained by any
argument is an **ascription**, not parametric polymorphism — and `:wat::core::ann-form` exists for
ascription. **That is its own stone and its own ruling; this one only stops the silence.**

## Acceptance

- `(:wat::eval-ast! :- [A B C] e)` — one declared param, three supplied — **REFUSED at check time**,
  and the diagnostic names `eval-ast!` and both counts.
- `(:wat::eval-edn! :- [A] "42")` — zero declared — **REFUSED**, same shape.
- `(:wat::eval-edn! :- [] "42")` — **STILL ACCEPTED**, identical to no binder.
- A genuine generic call with the correct count is unchanged; a genuine generic call with FEWER than
  declared still infers. **This is the row that proves the guard is `>` and not `!=`.**
- Floor green, every move accounted BY NAME; clippy 0.
