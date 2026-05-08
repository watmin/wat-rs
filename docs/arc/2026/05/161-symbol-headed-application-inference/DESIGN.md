# Arc 161 — Symbol-headed application inference

**Status:** opened 2026-05-07 on `arc-159-wip-substrate-and-sweep` branch.

**Gates:** arc 159 closure waits on arc 161 closure (same pattern as
arc 160; arc 159's binding-shape change ships cleanly only after every
substrate inference gap it surfaced is closed).

## Background — the discovery vehicle

Arc 159 dropped per-binding `:T` annotation from `:wat::core::let`,
relying on inference to type bindings from their RHS. Arc 160 fixed
parametric variant constructor inference (Ok/Err/Some FQDN keyword
paths). One workspace failure remained:

```
deftest_wat_telemetry_test_svc_tel_null_translator
  failure: :wat::core::length: parameter (dispatch dispatch)
           expects one of: (:Vec<T>) | (:HashMap<K,V>) | (:HashSet<T>);
           got (<unresolved>)
```

Minimal repro:

```clojure
(:wat::core::define
  (:test::null-translator
    -> :wat::core::Fn(wat::core::i64)->wat::core::Vector<wat::core::i64>)
  (:wat::core::fn ((_x :wat::core::i64) -> :wat::core::Vector<wat::core::i64>)
    (:wat::core::Vector :wat::core::i64)))

(:wat::core::define
  (:test::probe -> :wat::core::i64)
  (:wat::core::let
    ((t (:test::null-translator))     ; t : Fn(i64)->Vector<i64>
     (result (t 7)))                  ; (t 7) → ?? — inference returns None
    (:wat::core::length result)))     ; length: <unresolved>
```

`length` (arc 146 dispatch, strict) errors `<unresolved>`; `first`
(positional accessor, lenient) silently accepts a fresh var — that's
why the active-translator sibling test passes.

## The substrate gap

`src/check.rs::infer_list` lines 4606-4613 is the fall-through after
Region A (keyword heads) and Region B (bare-Symbol constructors).
For Symbol-headed application — calling a let-bound Fn value — the
substrate explicitly bails:

```rust
// Non-keyword head (bare symbol or inline expression). Not typed
// at this layer pending your call on explicit let-binding type
// annotations. Recurse into args so nested keyword-headed calls
// still get checked.
for item in items {
    let _ = infer(item, env, locals, fresh, subst, errors);
}
None
```

Pre-arc-159, the legacy let-annotation `((t :Fn(...)) (rhs))` typed
`t` directly via the binder's declared type; downstream uses didn't
need application inference because the binding was annotated.
Post-arc-159, `t`'s type comes from `(rhs)` (a Fn value here) and
the application `(t arg)` falls into this no-op branch.

## Goal

Make Symbol-headed (and inline-expression-headed) function-value
applications infer correctly. Pattern mirrors the existing
`infer_spawn` value-head branch (`src/check.rs:7556-7589`):

```rust
// Inferred head value
let head_ty = infer(&items[0], env, locals, fresh, subst, errors);
let surface_ty = match &head_ty {
    Some(t) => t.clone(),
    None => {
        // Recurse into args so nested errors still surface
        for arg in &items[1..] {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
};
let fn_ty = reduce(&surface_ty, subst, env.types());
match fn_ty {
    TypeExpr::Fn { args: ps, ret } => {
        // arity check + per-arg unify + return apply_subst(*ret, subst)
    }
    _ => {
        errors.push(CheckError::TypeMismatch { ... });
        for arg in &items[1..] {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        None
    }
}
```

## Scope

### In scope

- Symbol-headed application: `(t arg1 arg2 ...)` where `t` is
  let-bound (or fn-param-bound) to a `Fn(...) -> R` value
- Inline-expression-headed application: `((make-fn) arg1)` where
  the inline expression infers to a `Fn` value

### Out of scope

- Generic-Fn instantiation per call (rank-1 HM only; the Fn type is
  whatever the binding's RHS resolved to; no per-call freshening
  beyond what `reduce` already provides)
- Higher-kinded types
- Type-class / trait-style dispatch on the Fn value (that's arc 146
  dispatch territory)

## Slice plan

### Slice 1 — fix

Apply the fix at `src/check.rs::infer_list` lines 4606-4613.
Mirror `infer_spawn`'s value-head branch (lines 7556-7589).
~30-50 LOC delta in one function.

Verify the 1 currently-failing test passes. Verify the 2027
baseline tests stay green. Verify the minimal repro
(`/tmp/probe.wat` shape) type-checks cleanly.

### Slice 2 — closure paperwork (orchestrator-side)

INSCRIPTION + 058 changelog row + cross-references. Closes arc 161.
Unblocks arc 160 closure → unblocks arc 159 closure.

## Cross-references

- **Arc 159 v3** — (the user-visible binding-shape change) waits on this
- **Arc 160** — sibling substrate inference fix (variant constructors);
  same pattern: arc 159 surfaces; substrate fix ships separately
- **Arc 158a** — already shipped; orthogonal
- **Arc 145** — paid-for lesson "declared type is redundant when
  inference suffices" — arc 161 closes another corner where
  inference wasn't sufficient
- **Memory `feedback_v1_backout_dependency_arc.md`** — naming
  pattern: arc 161 ships separately; arc 159 closure cites this
- **WIP branch `arc-159-wip-substrate-and-sweep`** at `7ae2093` —
  arc 160 slice 2 fix; arc 161 builds on top

## Why arc 161 is the right shape

User direction 2026-05-07: *"we do the hard, honest work for the
longest term solution… we open new arc as nececssary, they gate
closure of prior arcs as necessary. this mass refactor work is
meant to catch exactly these kinds of problems."*

Section 12 of the recovery doc (foundation discipline): *"don't
bridge; investigate the gap. Don't defer; pivot. The friction IS
the diagnostic."*

The four questions:
- Obvious — `(t arg)` should infer when `t : Fn(...) -> R`. Mirror
  the existing keyword-headed function-application path; same
  shape Clojure / ML / every Lisp-family language has.
- Simple — single site, single pattern, atomic pieces (lookup,
  reduce, match, apply). The reference branch (`infer_spawn`
  7556-7589) is on disk.
- Honest — closes the real gap; doesn't restore legacy annotation;
  doesn't add new entity kind.
- Good UX — best possible (no annotation needed).

The mass-refactor discipline turned this gap up. Per user
direction: *"this mass refactor work is meant to catch exactly
these kinds of problems."*
