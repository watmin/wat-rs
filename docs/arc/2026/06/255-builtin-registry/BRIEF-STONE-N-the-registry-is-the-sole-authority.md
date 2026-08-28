# STONE N — the registry becomes `apply`'s authority too

DRAWN 2026-08-27 against `ee937ec55`.
**PRIOR ART — read first:** `git log -1 38f51c9fc` (**Stone G** — the identical shape one field over:
a missing slot in a function type, one choke point, a macro sniff, ~250 handlers untouched) and
`git log -1 ee937ec55` (**HOME-13's retraction** — the measurement that found this, and the error
that nearly deleted it).

**Builder, 2026-08-27:** *"the registry is all that remains at the end of this — it is the sole
authority for what exists."*

## ⛔ THE DEFECT — THERE ARE TWO AUTHORITIES, AND ONE IS INVISIBLE TO THE REGISTRY

```
direct call   eval → dispatch_keyword_head_value → REGISTRY (lookup at :5359) → handler
apply         eval_apply:10749 → dispatch_substrate_impl:11535 → a SECOND MATCH TABLE, 44 arms
                                 ↑ zero registry lookups anywhere in it or its caller chain
```

`:wat::core::apply` reaches a table the registry does not own. Forty-four verbs are dispatched there
by name, unmediated. **A verb can exist for `apply` and not for the registry, or diverge between
them, and nothing detects it.**

⚠ **This is NOT dead code.** HOME-13 was drawn on the belief that it was and is RETRACTED; the arms
are live and `(:wat::core::apply :wat::hashmap::length …)` reaches them. Do not delete them — this
stone makes them *removable* by making the registry answer for `apply`.

## ⛔ THE ONE CONTRACT DECISION — A SECOND SLOT, NOT A SECOND TABLE

The two paths differ in ONE thing: what they are handed.

```
NativeHandler       fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<TrackedValue, EvalBreak>
                    ↑ UNEVALUATED args; the handler evaluates them itself
dispatch_substrate_impl(impl_name: &str, vals: &[Value]) -> Option<Result<Value, EvalBreak>>
                    ↑ ALREADY-EVALUATED values; `apply` evaluated them
```

A registry handler physically cannot serve `apply` — there is no AST to hand it. That is why two
tables exist; it is an impedance mismatch, not laziness.

**But the implementations are already shared.** Measured: 25 of the 44 call the *same* `*_inner`
function from both sides —

```
apply    → dispatch_substrate_impl → ceval::hashmap_length_inner
direct   → registry handler → eval_inner(arg) → ceval::hashmap_length_inner
```

The registry handler IS the value-fn plus an arg-evaluating shell. **So the missing thing is a slot,
not an implementation:**

```rust
pub(crate) struct IntrinsicSubmission {
    pub handler: NativeHandler,                                  // AST path — unchanged
    pub value_handler: Option<fn(&[Value]) -> Result<Value, EvalBreak>>,   // ← ADD (apply path)
    …
}
```

`dispatch_substrate_impl` then becomes a registry lookup. **Every existing handler must keep
compiling untouched** — `value_handler` defaults to `None`, exactly as Stone G's default arm kept
~250 handlers compiling. Requiring all of them to change is **STOP-1**.

## The population — 25 lift, 19 need a decision

```
25  collection verbs   already call a shared *_inner  -> point value_handler at it. Mechanical.
19  arithmetic verbs   TWO PARALLEL IMPLEMENTATIONS:
       apply  → arith_i64_i64_inner(impl_name, vals, |a,b| a.checked_add(b))
       direct → eval_i64_arith(OP, …, |x,y,sp| i64_add_op(OP, x, y, sp))
```

⚠ **The 19 currently AGREE** — measured at the overflow boundary: `MAX + 1` raises on both paths,
`2 + 3` = 5 on both. So there is no live divergence to fix, and **that is the danger**: two
implementations with nothing holding them together. Collapsing them to one is the judgment in this
stone. **If they turn out to disagree anywhere, that is a finding — report it before collapsing.**

## Rooms — verified against `ee937ec55`

```
src/intrinsic/mod.rs:185-198        IntrinsicSubmission — where the slot goes
src/intrinsic/mod.rs:162            NativeHandler — Stone G's precedent, one field over
crates/wat-macros/src/wat_intrinsic.rs   sniff_return / SniffedArgs — the sniff pattern to MIRROR
src/runtime.rs:10749                eval_apply's ONLY call to dispatch_substrate_impl
src/runtime.rs:11535-11799          the 44 arms
src/collection/eval.rs              the shared *_inner helpers (the 25)
src/rete/purity.rs:2545             ⚠ the purity gate SCANS BOTH dispatch fns by name. Collapsing
                                    substrate_impl changes what that scan sees — measure the gate
                                    BEFORE and AFTER; HOME-12 went red on exactly this.
```

## STOP triggers — each REJECTS

1. **STOP-1 — you would change all ~250 existing handlers.** `value_handler: None` is the default.
2. **STOP-2 — you would delete the 44 arms in this stone.** This stone makes them removable; a
   separate stone removes them, on evidence, after the registry answers for `apply`.
3. **STOP-3 — the 19 arithmetic implementations disagree.** Report the divergence; do not silently
   pick one.
4. **STOP-4 — behaviour changes on either path.** Both must answer identically before and after.
5. **STOP-5 — a room's line number does not hold.**

## Acceptance

```bash
# 0. ★ THE REGISTRY ANSWERS FOR apply.
#    dispatch_substrate_impl consults the registry; a verb with a value_handler is served by it.
#    Prove it by SABOTAGE: point one verb's value_handler at a wrong result, show
#    (:wat::core::apply <verb> …) returns the sabotaged value, restore. Confirm the edit LANDED
#    before reading its output — a no-op probe returns a meaningless green.

# 1. ★ BOTH PATHS STILL AGREE. For three verbs incl. one arithmetic, run direct AND apply,
#    before and after, identical output. Include the overflow boundary: (:wat::i64::+ MAX 1).

# 2. every existing handler compiles untouched.
git diff --stat -- src/intrinsic/   # only the slot's mechanical additions

# 3. the purity gate: run it BEFORE and AFTER and report both counts.
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)'

# 4. cargo build --release --all-targets
```

## Report back with

Row 0's sabotage, and how you confirmed it landed. Row 1's before/after for all three verbs on both
paths. How many of the 44 now have a `value_handler`, and every one that does not, with the reason.
Whether the 19 arithmetic implementations agreed. The purity gate's counts before and after.
Anything this brief got wrong; what you did NOT do, and why.
