# DESIGN — random is threaded

Drawn 2026-09-03. **Not struck.** The precondition for chaos, and independently useful.

## Why

**wat has no randomness interface at all.**

```
:wat::*::rand  0   :wat::*::random 0   :wat::*::shuffle 0
:wat::*::sample 0  :wat::*::entropy 0  :wat::*::choose  0
```

A language that has built a Rete engine, a queue, a topic, a store, telemetry and a self-hosted
codemod framework, and cannot draw a number. The entropy exists — `:wat::uuid::v4` mints randomly in
`src/intrinsic/uuid.rs` — it was simply never surfaced.

Chaos needs it, and needs it **reproducible**: this repo's floor doctrine is *a red is a red —
capture it, name the arm, never re-run it away*. An unseeded chaos failure can be captured but never
**re-derived**, so a fix can never be verified. That is not merely inconvenient; it manufactures the
"known flake" category the doctrine exists to annihilate.

## The one contract decision: the state is THREADED, never ambient

```wat
(:wat::rand::next  state)     -> (Tuple :- [i64 i64])   ;; (draw, next-state)
(:wat::rand::below state n)   -> (Tuple :- [i64 i64])   ;; draw in [0, n)
```

The caller holds the state and threads it. **No hidden global.** Two consequences, and the second is
why this shape rather than the obvious one:

**It is reproducible by construction.** Same seed, same sequence, replayable from a SCORE.

★ **It is `Deterministic`, and an ambient RNG would not be.** wat classifies on two orthogonal axes,
and the precedent is already tested:

```
src/rete/purity.rs:10    ":wat::uuid::v4 does no IO and mutates nothing -> it is PURE"
src/rete/purity.rs:1802  classify_native_fn(":wat::uuid::v4", Axis::Pure).is_ok()
src/rete/purity.rs:1803  classify_native_fn(":wat::uuid::v4", Axis::Deterministic).is_err()
```

`uuid::v4` is Pure but **not** Deterministic — so it is barred from every position that requires
determinism (`sigma` fns, rete rule bodies, anywhere the analysis demands it). An ambient
`rand::i64` would inherit exactly that limitation. **A threaded draw is a pure function of its
state, so it passes BOTH axes** and is usable everywhere in the language. The shape that makes chaos
reproducible is the same shape that makes randomness admissible at all.

## No new type

The state is an `i64`. **SplitMix64** — 64-bit state, one multiply-xor-shift round, well-studied —
needs nothing the language does not have. No `Rng` value, no new core type, no new axis.

## Seeding is the caller's, and the seed must be REPORTED

`:wat::rand::` mints nothing on its own. A run that wants a fresh schedule draws a seed however it
likes (`uuid::v4` is right there) — **and must print it**, because a seed that is not reported is a
run that cannot be replayed, which is the failure this whole stone exists to prevent.

That discipline belongs in the chaos stone's brief, not in this intrinsic. Named here so it is not
lost between them.

## Out of scope = REJECTED

- **An ambient/global RNG.** The contract decision; it would be non-Deterministic and unreplayable.
- **Distributions beyond uniform**, `f64` draws, shuffles, sampling. Add when a consumer exists —
  a wide surface with one user is an abstraction before its second.
- **The chaos injection itself.** This stone hands over a primitive and stops.
- **Seeding policy.** Above.
