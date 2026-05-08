# Arc 161 — Slice 1 BRIEF (the fix)

**Drafted 2026-05-07.** Slice 1 of arc 161 — apply the Symbol-headed
application inference fix.

## Working directory

`/home/watmin/work/holon/wat-rs` on branch
`arc-159-wip-substrate-and-sweep`.

## Workspace state pre-spawn

- HEAD: `7ae2093` (arc 160 slice 2 fix shipped)
- Working tree clean
- Workspace: 2027 passed / 1 failed
  (`deftest_wat_telemetry_test_svc_tel_null_translator`)
- The 1 failure repros minimally; sonnet's diagnostic is on disk
  in this arc's DESIGN.md

## Diagnostic recap

`src/check.rs::infer_list` lines 4606-4613 is the fall-through after
Region A (keyword heads) and Region B (bare-Symbol constructors). For
Symbol-headed application — calling a let-bound Fn value — the
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

Pre-arc-159, the legacy `((t :Fn(...)) (rhs))` annotation typed `t`
directly; downstream `(t arg)` didn't need application inference.
Post-arc-159, `t`'s type comes from `(rhs)` — and `(t arg)` falls
into this no-op.

Minimal repro (`/tmp/probe.wat`):

```clojure
(:wat::core::define
  (:test::null-translator
    -> :wat::core::Fn(wat::core::i64)->wat::core::Vector<wat::core::i64>)
  (:wat::core::fn ((_x :wat::core::i64) -> :wat::core::Vector<wat::core::i64>)
    (:wat::core::Vector :wat::core::i64)))

(:wat::core::define
  (:test::probe -> :wat::core::i64)
  (:wat::core::let
    ((t (:test::null-translator))
     (result (t 7)))
    (:wat::core::length result)))
```

Pre-fix:
```
:wat::core::length: parameter (dispatch dispatch) expects one of:
  (:Vec<T>) | (:HashMap<K,V>) | (:HashSet<T>); got (<unresolved>)
```

Post-fix: clean.

## Goal

Make `(t arg1 arg2 ...)` infer correctly when `t : Fn(P1, P2, ...) -> R`
is bound in `locals` (let-binding, fn-param, etc.). Also make
inline-expression heads (`((make-fn) arg1)`) infer when the inline
expression resolves to a Fn type.

## Fix shape — mirror `infer_spawn`'s value-head branch

The reference pattern is at `src/check.rs:7556-7589` (the
non-keyword arm of `infer_spawn`'s callee match). Adapt that
pattern for the general application case at lines 4606-4613.

### Step 1 — Replace lines 4606-4613 with value-head application

```rust
// Non-keyword head: Symbol bound to a Fn value, or inline
// expression whose value type is a Fn. Mirror `infer_spawn`'s
// value-head branch: infer the head, reduce, match Fn, apply.
let head_ty_opt = infer(head, env, locals, fresh, subst, errors);
let surface_ty = match head_ty_opt {
    Some(t) => t,
    None => {
        // Head couldn't be inferred (already reported elsewhere).
        // Recurse into args so nested errors still surface.
        for arg in &items[1..] {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
};
let fn_ty = reduce(&surface_ty, subst, env.types());
let (param_types, ret_type) = match fn_ty {
    TypeExpr::Fn { args: ps, ret } => (ps, *ret),
    other => {
        errors.push(CheckError::TypeMismatch {
            callee: "(value head)".into(),
            param: "#0".into(),
            expected: "function value (Fn(...) -> R)".into(),
            got: format_type(&apply_subst(&other, subst)),
            span: head.span().clone(),
        });
        for arg in &items[1..] {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
};
let call_args = &items[1..];
if call_args.len() != param_types.len() {
    errors.push(CheckError::ArityMismatch {
        callee: "(value head)".into(),
        expected: param_types.len(),
        got: call_args.len(),
        span: head.span().clone(),
    });
    for arg in call_args {
        let _ = infer(arg, env, locals, fresh, subst, errors);
    }
    return Some(apply_subst(&ret_type, subst));
}
for (i, (arg, expected)) in call_args.iter().zip(&param_types).enumerate() {
    if let Some(arg_ty) = infer(arg, env, locals, fresh, subst, errors) {
        if unify(&arg_ty, expected, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: "(value head)".into(),
                param: format!("#{}", i + 1),
                expected: format_type(&apply_subst(expected, subst)),
                got: format_type(&apply_subst(&arg_ty, subst)),
                span: arg.span().clone(),
            });
        }
    }
}
Some(apply_subst(&ret_type, subst))
```

### Notes on the callee label

The keyword-headed paths use `k.clone()` (the FQDN string) as the
`callee` field on errors. For value-head application, there's no
single canonical name — the head could be a Symbol, an inline
expression, or anything else. Use `"(value head)"` as a stable
label. If a future arc wants a more descriptive label, that's an
ergonomics tweak; the current label is honest and stable.

## Constraints

- **Substrate-only edits.** EXACTLY 1 file: `src/check.rs`. No
  other crate. No consumer wat edits. No new test files (the 1
  currently-failing test verifies the fix end-to-end).
- **DO NOT COMMIT.** Working tree dirty for orchestrator review.
- **Workspace MUST go from 1 failed → 0 failed.**
- **STOP at unexpected red.** Distinguish:
  - **Expected:** the 1 currently-failing telemetry test passes
  - **Unexpected:** any pre-existing test breaks. The substrate
    change is intended to be additive (filling a no-op branch);
    no regressions expected.
- **Time-box: 30 min wall-clock.**

## Pre-flight crawl

1. `docs/arc/2026/05/161-symbol-headed-application-inference/DESIGN.md`
2. `src/check.rs::infer_list` line 4606-4613 (the no-op branch)
3. `src/check.rs::infer_spawn` line 7556-7589 (the reference pattern)
4. `src/check.rs::reduce` line 9762, `apply_subst` line 9719 (helpers)
5. `src/check.rs::unify` (the keyword-headed application uses this
   for arg-vs-param unification; same here)

## Pre-flight verification

```bash
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | tail -5
```

Confirms 1 failure pre-fix (the telemetry null-translator test).

## Verification (after edits)

```bash
cargo test --release -p wat-telemetry deftest_wat_telemetry_test_svc_tel_null_translator 2>&1 | tail -5
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | tail -5
```

Expect: workspace = 0 failed; ~2028 passed.

## Reporting (~200 words)

- Pre-flight crawl confirmation
- Edit summary (LOC delta; what replaces lines 4606-4613)
- New test pass count (1 previously-failing now passes; no regressions)
- Path classification (Mode A / B / C)
- Honest deltas:
  - Did inline-expression heads (`((expr) arg)`) need separate
    handling, or did the unified `infer(head)` path cover both?
  - Did the helper extraction surface any other shared patterns
    that could be unified later?
  - Did any tests previously passing accidentally cover the gap
    by having type annotations that are now structurally redundant
    (the legacy form is rejected; this is a separate question
    about what the post-arc-159 surface area exercises)?

DO NOT commit. Orchestrator commits + scores after.

## Time-box

30 minutes wall-clock.

## Why slice 1 matters

This unblocks arc 160 closure (workspace = 0-failed enables INSCRIPTION
without lingering red), which unblocks arc 159 closure. Per the
mass-refactor discipline: *"this work is meant to catch exactly
these kinds of problems"* — arc 161 IS the catch.
