# DESIGN STONE — 118.B2d · a GENERIC satisfier cannot bind the surface's type param

**Door 2.** Found by B2b, 2026-08-18; pre-existing since B1 (`488eacd0`). Sibling of B2c (door 1).
Evidence and the shared framing: `NOTE-118.B2b-two-doors-the-checker-opened-and-the-runtime-did-not.md`.

## The defect, with its control

```wat
(:wat::core::defn :my::eats [c <- :wat::core::Seqable<wat::core::i64>] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] c)))

;; CONTROL — the concrete container fed DIRECTLY.  --check: CLEAN.  B1a works.
(:my::eats (:wat::core::Vector :wat::core::i64 1 2 3))

;; THE DEFECT — the same value through the surface method.  --check:
;;   :my::eats: parameter #1 expects :wat::core::Seqable<wat::core::i64>; got :wat::stream::Stream<T>
(:my::eats (:wat::core::Seqable/seq (:wat::core::Vector :wat::core::i64 1 2 3)))
```

`Seqable/seq` on a `Vector<i64>` must yield `Stream<i64>`. It yields `Stream<T>`, `T` free.

The control is what makes this narrow: surfaces are not broken; the METHOD'S RETURN does not carry
the receiver's instantiation.

## The mechanism — named by the checker's own comment

`src/check.rs:4926-4948`, the parametric-surface member-type resolution. Its comment states the
design assumption outright:

> *"a `defsurface :S<T> …` member's declared type is a bare, UNRESOLVED `:T` in `s.members` —
> correct only PER-SATISFIER. Resolve it by looking up the receiver's concrete satisfier scheme
> `<ConcreteType>/<method>`, which `extend-type` registered **with the surface's `<T>` already
> substituted to that satisfier's concrete binding (e.g. `T=i64` for
> `(extend-type :IntBox :Holds<i64> …)`)**."*

**That assumption holds only for a MONOMORPHIC satisfier.** `(extend-type :IntBox :Holds<i64>)` binds
`T → i64` at registration, so the stored scheme's return is already concrete and path (1) can hand it
back verbatim.

`Seqable<T>` is different in kind: it is satisfied by **generic containers**.
`(extend-type :wat::core::Vector :wat::core::Seqable<T>)` binds `T → T` — a type VARIABLE, not a
type. So the registered `wat::core::Vector/seq` scheme returns `Stream<T>` with `T` free, path (1)
finds it, returns it unchanged, and **nothing ever instantiates `T` from the receiver's actual
element type.**

Path (2) — the arc-170 "abstract parametric-surface receiver" branch — *does* bind the surface's
params from a receiver's args, which is exactly the operation needed. But it is guarded by
`parametric_head_fqdn(&head) == protocol_fqdn`: it fires only when the receiver IS the surface
itself. A `Vector<i64>` receiver is not, so it never runs.

**The missing case, stated exactly:** *a satisfier whose surface binding is itself a type variable,
called on a concrete receiver.* Neither path covers it. Path (2) already contains the machinery
(`s.type_params` zipped against the receiver's args, then `rename`) — it is pointed at the wrong
receiver shape.

## ⛔ A HYPOTHESIS THAT WAS REFUTED BY RUNNING IT — do not re-derive this

Before the above, the obvious story was: *the extend-type target head is BARE
(`:wat::core::Vector`, not `Vector<T>`), so `self` has no element type and there is nothing to bind
`T` from — spell the target parametrically and it fixes itself.*

**Tried it.** All four `extend-type` targets in `wat/seq.wat` rewritten to `Vector<T>` /
`PersistentVector<T>` / `List<T>` / `Stream<T>`, rebuilt, re-run:

1. **The type error did not move.** Still `Stream<T>`. The bare head is not the cause.
2. **It broke the language.** The impl registers under `wat::core::Vector<T>/seq`, but
   `satisfier_method_keys` (`src/check.rs:9551`) looks up the exact `format_type`, then a
   `T`/`Xt` last-slot rewrite, then the **bare head** — and `register_extend_type_surface_impls`
   (`src/runtime.rs:1111`) keys on `ed.type_name`. The two no longer meet:
   `unknown function: type ':wat::core::Vector' does not implement surface method 'seq'`.

Reverted; the tree is back to the bare spelling, rebuilt, re-verified. **The parametric spelling is
strictly worse, and the cost of learning that was one 32-second rebuild instead of a wrong mechanism
written into a stone.** `[[feedback_a_design_sentence_is_not_the_disk]]`

## Why nothing caught it

`core-seqable.wat` (B1's own coverage) calls `(into [] (Seqable/seq v))`. `into`'s Stream clause is
itself `Stream<T>`, so a free `T` unifies with it happily and the row goes green. **The loss only
surfaces when the consumer wants a CONCRETE element type**, and until B2b's tests no consumer did.

## Consequence today

`(Seqable/seq v)` is not usable as a general "coerce to seq" spelling in typed code — its result can
only be fed to something equally polymorphic. That is a real hole in the surface B1 minted: the ONE
method `Seqable<T>` has cannot have its result typed.

`wat-tests/core/core-seq-walkers.wat` routes around it with `(map identity v)`, which is a better
test anyway; the workaround costs that file nothing, but it is a workaround and it is named there.

## What must be true before this is briefed

1. **A disconfirming probe, committed** — the repro + control above as a `tests/types/` fixture,
   RED at HEAD on the repro and GREEN on the control, so the fix's green means *this* and not
   something adjacent. Does not exist yet.
2. **The blast radius measured, not guessed.** Path (1) is the path EVERY parametric surface's method
   call takes — `Dialable`, `Holds`, the arc-293 transport surfaces. A change there is not local to
   `Seqable`. Enumerate the parametric surfaces in the corpus and what each one's satisfier binding
   is (concrete vs variable) BEFORE touching the arm.
3. **The four questions on the fix's shape** — extend path (2)'s guard, add a third path, or bind at
   registration. Not yet posed; posing them needs (2).

**Do not brief this stone without all three.** The one hypothesis that felt obvious was already
wrong once.

## Out of scope — affirmative cuts

- **Door 1** (`DESIGN-STONE-118.B2c-…`) — the runtime clause selector. Independent: different file,
  different mechanism, different fix. They share only a cause-in-common (`Seqable<T>` is half-wired).
- **B3.** Its precondition is met and does not depend on either door.
