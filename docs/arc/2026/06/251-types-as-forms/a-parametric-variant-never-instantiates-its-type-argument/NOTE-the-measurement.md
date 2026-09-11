# NOTE — a parametric variant singleton never instantiates its type argument

**Found 2026-09-10.** The builder wrote this signature to ask a question about a *different*
proposal, and it turned out to be a live defect:

```wat
(wat.core/defenum user/Demo :- [T] wat.enum/Pure
  Has    [has :- T]
  HasNot [])

(wat.core/defn user/process :- T
  [demo-has :- (user/Demo.Has :- [T])]   ;; <- naming the VARIANT, with an argument
  :- T
  (wat.core/let [{:keys [has]} demo-has] has))
```

## The measurement

```
PARAMETRIC variant as a param, DESTRUCTURE   ⛔  "body produces :T; signature declares :wat::core::i64"
PARAMETRIC variant as a param, ACCESSOR      ⛔  same
PARAMETRIC enum as a param, via MATCH         ✓
NON-parametric variant as a param             ✓
```

`(:u::D.Has :- [:wat::core::i64])` accepts the argument in the signature and then **ignores it**.
Both read paths — `{:keys [has]}` destructuring and the `D.Has/has` accessor — hand back the
**uninstantiated** `:T`. Reaching the same field through the enum and a `match` instantiates
correctly.

## What is NOT the cause

The singleton **does** carry its parameter. `TypeEnv::register_variant_types` (`src/types.rs:1107`,
arc 296 A-2 RELAND-2 mechanism ④) gives each variant singleton the parent's type params filtered to
those *its own fields consume*:

```rust
let consumed = collect_free_type_vars_in(&member_types);
let variant_type_params = e.type_params.iter().filter(|p| consumed.contains(p)).cloned().collect();
```

`has <- :T` consumes `T`, so `D.Has` has `[T]`. The declaration side is correct. **What is missing
is applying the supplied argument at the USE site** — the singleton is instantiated everywhere
except when a signature names it directly.

## Why the subtyping is NOT implicated

The builder's rule, verified three-for-three on a non-parametric enum:

```
Demo.Has -> Demo     slot    ACCEPTED   (and the receiver must match on it)
Demo.Has -> Demo.Has slot    ACCEPTED   (no match needed)
Demo     -> Demo.Has slot    REFUSED    (widening is one-way)
```

`Demo.Has <: Demo` is implemented exactly as stated. This defect is instantiation, not subtyping.

## ⚠ NOT BISECTED

The machinery this depends on (singleton synthesis, arc 296) is months old and the dot flip did not
touch it, so this is **almost certainly pre-existing** — but that is a claim, not a measurement. A
bisect against `7ccce48ba` is one build and has not been run.

## ★ How it was found, which is the part worth keeping

It was not the target. A four-row harness written to verify the builder's SUBTYPING rule came back
**all-refused, including rows that must accept** — obviously wrong rather than quietly wrong. Had
the harness merely mis-measured one row, it would have been believed.

`[[feedback_a_green_test_can_prove_nothing]]` has a mirror: an instrument that fails LOUDLY and
IMPLAUSIBLY is worth more than one that fails plausibly. Three proposals died to measurement in the
hour before this (an `ann-form` ascription, binding the enclosing enum, same-head covariance) and
none of them would have surfaced this, because each was aimed at the variance question.
