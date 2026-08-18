# MEASURED — B2 is BLOCKED. The `Var` gate excludes every verb B2 collapses.

**2026-08-18, against `f08c5d1a`.** Written because the builder asked *"are you ready to spawn a
shadowdancer?"* and the honest answer was **no**. This is the FM 2-bis disconfirming probe that
found out why — before a rider flight, not during one.

## The probe

A probe attempting exactly B2's composition: ONE `defn` over `Seqable<T>`, a **lazy producer**
(`stream/lazy` + `match (next …)` + `stream/cons`), recursing on the `rest` a `NextOutcome::Item`
hands back. It is now GREEN and lives at
`wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat` — **see the RESOLVED section at
the bottom**; what follows is the state while B2 was blocked.

**RED — 5 type errors.** And their *distribution* is the finding:

```
line 73  keep-one param #2 expects Seqable<wat::core::i64>; got Vector<wat::core::i64>
line 75  …                                                 got PersistentVector<wat::core::i64>
line 77  …                                                 got List<wat::core::i64>
line 79  …                                                 got Stream<T>
line 88  …                                                 got Stream<wat::core::i64>
```

**All five are EXTERNAL call sites. The RECURSIVE call inside the definition produced ZERO errors.**

So the thing I had named as the crux — *"a Stream handed to a `Seqable<T>` parameter, recursively,
inside the fn's own definition"* — **works.** The one-clause lazy-producer shape is sound. What
fails is calling it from outside.

## The discriminator, isolated to one variable

```wat
:d::a<T> [s <- Seqable<T>]                  called with Vector<i64>   →  GREEN
:d::b<T> [probe <- :T  s <- Seqable<T>]     called with Vector<i64>   →  RED
```

The **only** difference is whether an earlier parameter already pinned `T`. Once it does, the
parameter's type is the **concrete instantiation** `Seqable<wat::core::i64>` rather than
`Seqable<?N>` — and stone 118.3-B's fix is **`Var`-gated**:

> `src/check.rs:14894` — `else if eargs.iter().any(|t| matches!(t, TypeExpr::Var(_)))`

Inside the definition, `T` is the fn's own type parameter — still a `Var` — so the gate fires and
the recursion checks. At an external call site with a concrete container, `T` is resolved, the gate
does not fire, and the arm above it (exact string match) can never match a builtin against a surface
name.

**★ Every verb B2 collapses takes `f` / `sep` / `idx` BEFORE the collection.** So every one of them
pins `T` first. **B2 is blocked completely, not partially.**

## ⚠ THIS WAS ALREADY ON THE RECORD, AND I PRUNED IT

The previous seam's STILL-OPEN carried:

> *"The `Var`-gate excludes concrete surface instantiations (`[s <- :Seqable<i64>]`); a surface with
> >1 type param is untested."*

**I dropped that line when I rewrote the seam** an hour before running this probe — and then
rediscovered the same fact by walking into it. Curare's job is to prune what went **stale**, and
this was **live**. Pruning is not free: a line removed from the breadcrumb is a fact the next self
must pay to re-learn. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]` has a mirror —
*a live note deleted is a blocker rediscovered.*

The line is restored to the seam, now with the measured mechanism attached instead of just the
symptom.

## The gate was RIGHT for its tenants — this is a new case, not a bug

`check.rs:14885-14893` explains the gating deliberately: the arm's existing tenants — `Dialable`,
`TypedCapability`, `Handle` — are **baked per-instance with fully concrete args**
(`TypedCapability<Echo::Op,Echo::Reply>`, never a bare `S`/`R`), so the exact-string arm above
decides them byte-identically and this branch is unreachable for their calls.

`Seqable<i64>` is genuinely new: **concrete args where the actual is a BUILTIN CONTAINER**, whose
name can never string-match the surface's. Nothing in the existing tenant set exercised that
combination, which is exactly why the gate reads as correct and is nonetheless insufficient.

## What this means for the build order

A stone is owed **before** B2:

```
B1a  widen the surface-satisfaction arm to CONCRETE instantiations —
     when the string arm has not matched AND the actual's family extend-types the surface,
     bind-and-unify regardless of whether eargs still contain a Var.
```

⚠ **The risk is precisely named by the gate's own comment:** widening reaches `Dialable` /
`TypedCapability` / `Handle`, which currently resolve on the string arm. B1a must prove they are
**observationally unchanged** — that is its load-bearing row, not the `Seqable` case.

## What the probe cost, and what it saved

~15 minutes. A rider briefed to "collapse the six verbs to one `Seqable` clause each" would have hit
this in its first edit, with no way to distinguish "the design is wrong" from "the substrate has a
gap" — the exact corner FM 2-bis exists to prevent, and the corner that cost ~2 hours and a killed
sweep the last time it was skipped.

**And the probe is not a loss.** It proved the harder half: the lazy-producer shape, the
`stream/lazy` + `match (next …)` + `stream/cons` body, and the recursive `Stream`-into-`Seqable`
call **all work.** B2's design is sound; only its call sites are blocked. The probe became B1a's
RED-to-GREEN gate row — see RESOLVED, below.

---

## ⛔ RESOLVED — B1a landed; this probe is GREEN and back on disk

**Superseded 2026-08-18 by stone 118.B1a.** The `Var` gate at `check.rs:14894` was removed; the
probe now lives at `wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat`, is
loader-gated, and prints:

```
2,4 | 2,4 | 2,4 | 2,4
0,1,2,3,4
0,2,4
```

**RED→GREEN on a committed artifact — 5 type errors to 0.**

### Two corrections to what this file said while it was open

1. **I claimed a RED `.wat` could not be committed at all, and embedded the source here instead.**
   Wrong: the project already has the mechanism — **`.wat.bad`**, a fixture extension no gate
   loads, driven from Rust via `startup_from_file(...).expect_err` (exemplar:
   `tests/types/probe_arc170_parametric_surface_param.rs`, itself the swap-gate test for
   `Dialable`). I reached for "embed it in prose" without searching for how the repo already
   keeps deliberately-ill-typed fixtures. The embedded copy is deleted; it was duplicate and
   would have rotted. `[[feedback_search_for_the_mechanism_not_in_the_broken_callers_neighbourhood]]`

2. **One of the five errors was never B1a's to fix.** After the widening, four cleared and one
   remained: `got :wat::stream::Stream<T>` — the return of the **dotted** method `Seqable/seq`,
   whose type comes back carrying the surface's declared letter `T`, uninstantiated. Isolated: a
   concrete `Stream<i64>` from an ordinary `defn` satisfies `Seqable<i64>` fine; only the dotted
   method's return does not. That is **task #95** (a dotted call head is not type-checked), not
   surface satisfaction. The probe's 4th slot was changed to a concrete `Stream<i64>` so it
   measures B1a rather than #95 — and the #95 instance is recorded here rather than dropped.

The negative controls B1a owes are kept, not narrated:
`tests/types/probe_stone_118_b1a_neg.{rs,wat.bad}`.
