# DESIGN — cure the oracle: `rule-produces` must resolve a fn head to its return type

## Why

`34ee46ce9` drove the defect and refereed it. The oracle DROPS a derived fact:

```
Src(1); Bad :- Src, k=2 (never fires); Rate :- Src, (not Bad), :then (mk-rate ?k); Out :- Rate

  NATIVE [Bad Rate Out] = [0 1 1]
  ORACLE [Bad Rate Out] = [0 1 0]     <- Out dropped
  CLARA  [Bad Rate Out] = [0 1 1]     <- the referee, re-runnable, agrees with native
```

`rule-produces` (`wat/rete/oracle/stratify.wat:46-67`) takes the first child of each `:then` form,
reads its name and strips a colon. For a user-fn head that yields the **function's** name, so the
sweep assigns the stratum to `a2::mk-rate`, `Rate` is never raised, and `Out` — consuming `Rate` —
computes `req-pos` from 0 and sits BELOW its own producer. Stratified fire threads facts forward
without re-firing lower strata, so `Out` never appears.

**The oracle is the wrong one**, same as `16f504e14`, and worse than a port bug: every differential
taken against the oracle inherits it.

## ⭐ The cure already exists in this tree, one file away

`wat/rete/compile.wat:775-798` performs exactly this resolution for the `:then` constructor door:

```
head-val0  (:wat::eval-ast! head)
is-fn-val  (:wat::core::= (:wat::core::type head-val0) "wat::core::fn")
head-fn    if is-fn-val then head-val0
           else re-resolve through the PRIME `:T'` keyword -> the constructor fn
ret-ty     (:wat::runtime::return-type-of head-fn)      ; already a COLON-FREE FQDN
```

**This is the arc's signature shape for the eleventh time: the tree had written the correct rule
down and applied it in exactly one place.** `compile.wat` resolves the head; `stratify.wat`, ~700
lines away in the same subsystem, re-derives the name by string surgery.

**And the recipe needs no fn/record discrimination.** A bare record head `(:pt::Rate :count ?x)`
evaluates to a KEYWORD, takes the PRIME branch, and `return-type-of` on that constructor yields
`pt::Rate` — the same answer the colon-strip gives today. One path, correct for both shapes.

## The one contract decision

**`rule-produces` adopts `compile.wat`'s resolution verbatim in shape, and the colon-strip is
DELETED, not kept as a fallback.** A fallback would silently restore today's wrong answer whenever
resolution failed, which is the defect returning through the channel the cure opened
(`[[probe-the-fix-for-the-defects-own-shape]]`). If a head cannot resolve, the oracle raises — the
same posture `compile.wat` already takes, and compile has already validated every head that reaches
stratification.

## The gate flips, and that flip IS the proof

`src/rete/kernel/tests/produced_type_userfn.rs` currently asserts DIVERGENCE — it is the "before"
picture. Both of its assertions must invert:

| assertion | before | after |
|---|---|---|
| `rule-produces` on a user-fn head | native `pt::Rate` / oracle `pt::first-rate` | **both `pt::Rate`** |
| facts `[Bad Rate Out]` | native `[0 1 1]` / oracle `[0 1 0]` | **both `[0 1 1]`** |

⚠ The ANCHOR must NOT move: `:pt::plain` stays `pt::Rate` on both sides. It is the non-vacuity
guard — a cure that made every answer empty would satisfy "they agree".

## Blast radius — this changes ORACLE BEHAVIOUR

Every differential taken against the oracle can shift: the grid three-way, the port check, the
TMS fuzzer, every `$oracle` probe. That is not a risk to be managed around, it is the point — the
oracle was wrong and things measured against it inherited the wrongness. **The full floor is the
instrument**, and any test that moves must be read as "was it wrong before?", never "un-break it".

## Files

- `wat/rete/oracle/stratify.wat` — `rule-produces`. `include_str!`'d, so a rebuild is mandatory.
- `src/rete/kernel/tests/produced_type_userfn.rs` — the two assertions flip.

## Out of scope = REJECTED

- `produced_type` on the native side. Clara agrees with native; native is not the defect here.
- `rule-negates` / `rule-consumes`. They read LHS patterns, where a head is a fact type by
  construction — no fn head is possible. Not touched, and not silently assumed: if the strike finds
  otherwise, that is a STOP.
- `probe_arc278_then_user_forms_userfn.wat:12-22`'s stale purity claim — rowed in `34ee46ce9`, its
  own strike.
