# BRIEF — the thirteen schemes: bring the Persistent family under the type checker

Every `PersistentMap/*` and `PersistentVector/*` verb is **blanket-accepted** — no `TypeScheme`, so
`check.rs:5561` returns a fresh type variable and the checker sees nothing. Measured: all 13 accept
SEVEN arguments without complaint, while their 16 std twins reject with `ArityMismatch`. Full finding:
`NOTE-the-persistent-family-is-outside-the-type-checker.md`.

**This is not a design task. It is a transcription.** All 13 have a registered std twin whose scheme
already decides the type params, the arity and the return shape.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** No `cargo`, in any
form — `./target/release/wat` is prebuilt and will NOT reflect your changes. Foreground everything;
ending your turn ends you. Do not commit, push, stash, or revert.

## ★ ADDITIVE — measured, not assumed

A **bare** annotation must keep working. `wat/rete.wat` alone carries 166 bare
`<- :wat::core::PersistentMap` / `PersistentVector` sites and they must all still check.

This is safe, and the analogue proves it: `HashMap` HAS schemes and still accepts a bare annotation —
`[m <- :wat::core::HashMap] → (:wat::core::HashMap/length m)` checks clean today. Your schemes must
leave that property intact for the persistent family. **If any bare site breaks, STOP.**

## The rooms

**Helpers** — `check.rs:19849-19858`, local closures in one fn:

```rust
let k_var = || TypeExpr::Path(":K".into());
let v_var = || TypeExpr::Path(":V".into());
let hashmap_of = |k: TypeExpr, v: TypeExpr| TypeExpr::Parametric {
    head: "wat::core::HashMap".into(), args: vec![k, v] };
let hashset_of = |t: TypeExpr| TypeExpr::Parametric { … };
```

Add `persistentmap_of` and `persistentvector_of` in the same shape. The head strings are already
established elsewhere in the file — `"wat::core::PersistentMap"` (`check.rs:14083`, `:14127`) and
`"wat::core::PersistentVector"` (`:14195`, `:20094`). Use those exact strings.

⚠ Is there a `t_var()` and a `vector_of`? The `Vector/*` templates use them — find them rather than
minting duplicates.

**The 13 templates.** Each persistent verb copies the scheme at the line given, swapping the container
constructor. Confirm each by matching the surrounding code, never by trusting the number:

```
PersistentMap/get            ← HashMap/get            19990
PersistentMap/assoc          ← HashMap/assoc          20034
PersistentMap/dissoc         ← HashMap/dissoc         20043
PersistentMap/keys           ← HashMap/keys           20052
PersistentMap/values         ← HashMap/values         20061
PersistentMap/length         ← HashMap/length         19881
PersistentMap/empty?         ← HashMap/empty?         19924
PersistentMap/contains-key?  ← HashMap/contains-key?  19957
PersistentVector/get         ← Vector/get             19981
PersistentVector/conj        ← Vector/conj            20003
PersistentVector/length      ← Vector/length          19872
PersistentVector/empty?      ← Vector/empty?          19915
PersistentVector/contains?   ← Vector/contains?       19948
```

## ⛔ The contract decision

> **Each scheme is its twin's, verbatim, with only the container constructor swapped. Nothing is
> redesigned here.**

If a persistent verb's real runtime behaviour DIFFERS from its twin's — a different arity, a different
return, an extra arm — that is a **FINDING**. Report it; do not quietly write a different scheme. The
whole value of this stone is that 13 answers were already decided by 13 existing rows; inventing one
throws that away and hides a real divergence.

## Blast radius

`src/check.rs` only — two helper closures and thirteen `env.register` calls. No runtime change (the
runtime already dispatches these verbs correctly; only the CHECKER is blind). No `.wat`. No `tests/`.

## STOP triggers — each rejects; none is a fallback

1. Any existing **bare** persistent annotation stops checking. STOP — this step is additive.
2. A persistent verb's behaviour differs from its twin's. STOP; report which and how.
3. The change needs a runtime edit. STOP — the runtime already works; only the checker is blind.
4. A helper you need does not exist and its shape is not obvious from a sibling. STOP; do not guess.

## Acceptance criteria

- 13 new `env.register` calls; each mirrors its named twin with only the container swapped.
- ★ **The gate**: hand each persistent verb SEVEN arguments — all 13 must now reject. Today all 13
  accept. `docs/arc/2026/06/255-builtin-registry/PROBE-255.1c-io-every-verb-is-scheme-enforced.sh`
  is the shape to copy for that probe.
- ★ A declared V is now ENFORCED: a `(PersistentMap [String i64])` value used as a `String` must be
  REJECTED. It is accepted today; `HashMap` already rejects it. That is the whole point.
- Every bare annotation still checks — `wat/rete.wat` must be untouched and green.
