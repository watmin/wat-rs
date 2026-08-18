# NOTE — `Seqable<T>` is HALF-WIRED. Two doors, found by 118.B2b, both pre-existing.

**Found 2026-08-18 while migrating the six walkers. Neither is caused by B2b** — B1 (`488eacd0`)
minted the surface and B1a (`eab12e05`) opened the first door; nothing until now had walked through
the other two. Both are reproduced below with controls.

## Why they were invisible until today

B1a's acceptance proved one thing: **a concrete instantiation satisfies a parametric surface
PARAMETER, for a plain `defn`.** That is genuinely fixed and it still holds — the control below
passes. Every verb B2 collapsed was a single-arity `defn` taking a `Seqable<T>` and passing the
value straight through. That path is the ONLY one that had a consumer.

B2b was the first work to (a) put a `Seqable<T>` on a `defclause` ARM, and (b) feed a surface
METHOD'S RESULT into a concrete consumer. Each hit a different unwired door.

`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]` — B1a's green answered
"does a concrete type satisfy a parametric surface parameter?", and was read as "surfaces work."

---

## Door 1 — a `defclause` arm typed with a SURFACE never dispatches

```
no clause of :wat::core::reductions matched (3 args);
called with (fn, i64 `0`, Vector `[1, 2, 3, 4]`);
clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector)
```

**Where.** `value_matches_type_by_name`, `src/runtime.rs:8760`, the `TypeExpr::Parametric` arm. It
resolves the value to a `StreamContainer` and requires the declared head to equal that container's
canonical name (`wat::core::Vector`, `wat::stream::Stream`, …). `wat::core::Seqable` is not one of
them, so the arm can never match anything. The CHECKER accepts the call; the runtime then refuses it.

**This is the same bug as the arc-278 record-top fix, twenty lines up in that same function**, whose
comment already states the principle:

> *"the RECORD-TOP must dispatch, or the runtime disagrees with the checker … The result was a
> program that type-checks and dies at runtime with `NoMatchingClause`."*

A surface is the container-top. Same disagreement, one arm down.

**Blast radius.** Only `defclause`. A plain `defn` never reaches this selector — which is why the
four single-arity verbs B2b migrated (`remove`, `take-while`, `drop-while`, `take-nth`) work.

**Consequence today.** No multi-arity verb can go over `Seqable<T>`. `reductions` therefore keeps
ten per-container arms delegating to one walker, mirroring `reduce`. Those ten arms are waiting on
this door.

**The fix's shape**, and the arc-278 comment supplies its safety argument: when the declared head
names a `TypeDef::Surface`, match if the value's type has the surface's impls registered — extend-type
records them as functions keyed `"<TypeName>/<method>"` (`register_extend_type_surface_impls`,
`src/runtime.rs:1111`), so the check is runtime-visible. It needs `sym` threaded into
`value_matches_type_by_name` (the caller, `select_defclause_clause`, already holds it). Per the
record-top precedent this only ADDS a supertype, so no call that dispatches today can stop.

---

## Door 2 — a surface METHOD'S RETURN loses the receiver's instantiation

`Seqable/seq` is declared `[self <- Seqable<T>] -> Stream<T>`. Called on a `Vector<i64>` it should
give `Stream<i64>`. It gives `Stream<T>`, with `T` unbound.

**Reproduced, with the control that makes it mean something:**

```wat
(:wat::core::defn :my::eats [c <- :wat::core::Seqable<wat::core::i64>] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] c)))

;; CONTROL — the concrete container fed DIRECTLY.  --check: CLEAN.  (B1a works.)
(:my::eats (:wat::core::Vector :wat::core::i64 1 2 3))

;; THE DEFECT — the same value through the surface method.  --check:
;;   :my::eats: parameter #1 expects :wat::core::Seqable<wat::core::i64>; got :wat::stream::Stream<T>
(:my::eats (:wat::core::Seqable/seq (:wat::core::Vector :wat::core::i64 1 2 3)))
```

The control is what separates "surfaces are broken" (false) from "the METHOD'S RETURN does not carry
the instantiation" (true, and much narrower).

**Why nothing noticed.** `core-seqable.wat` calls `(into [] (Seqable/seq v))`, and `into`'s Stream
clause is itself `Stream<T>` — an unbound `T` unifies with it happily. The loss only surfaces when
the consumer wants a CONCRETE element type. B2b's tests were the first such consumer.

**Consequence today.** `(Seqable/seq v)` cannot be used as a general "coerce to seq" spelling in
typed code. `wat-tests/core/core-seq-walkers.wat` builds its lazy sources with
`(map identity v)` instead — which is a better test anyway (a real lazy-over-lazy composition), so
this cost the stone nothing beyond finding it.

---

## Disposition

**Neither is B2b's to fix.** B2b removes the stdlib's three-call Stream walk; that is done and the
census is zero. These two are what stand between here and *every* verb being one definition over any
seqable — the actual end state route B is aimed at.

Both are now drawn, each as its own stone:

- **Door 1** → `DESIGN-STONE-118.B2c-a-surface-typed-clause-arm-never-dispatches.md`. Runtime clause
  selector; precedent is twenty lines up in the same function.
- **Door 2** → `DESIGN-STONE-118.B2d-a-generic-satisfier-cannot-bind-the-surface-param.md`. Checker;
  the mechanism is now characterized (see below), and **one obvious hypothesis was refuted by
  running it** before it could reach a stone.

**AMENDED 2026-08-18, after the lair study door 2 was waiting on.** This file first said door 2 was
"NOT drawn — it needs its own lair study." The study is done. Two things came out of it:

**(a) The mechanism, from the checker's own comment.** Path (1) of the parametric-surface member
resolution (`src/check.rs:4926`) assumes the satisfier bound the surface's `T` to a CONCRETE type at
`extend-type` time — *"e.g. `T=i64` for `(extend-type :IntBox :Holds<i64> …)`"*. `Seqable<T>` is
satisfied by GENERIC containers, so the binding is `T → T`, a variable; the stored scheme's return
stays `Stream<T>` and nothing instantiates it from the receiver. Path (2) has exactly the machinery
needed but is guarded to fire only when the receiver IS the surface. **The missing case: a satisfier
whose surface binding is itself a type variable, called on a concrete receiver.**

**(b) A REFUTED hypothesis, recorded so nobody re-derives it.** The obvious story — *the extend-type
target head is bare (`:wat::core::Vector`, not `Vector<T>`), so there is nothing to bind `T` from* —
was tested by rewriting all four targets parametrically and rebuilding. **The type error did not
move**, AND the parametric spelling broke registration outright (`satisfier_method_keys` resolves via
the bare head; `register_extend_type_surface_impls` keys on `ed.type_name`), giving
`unknown function: type ':wat::core::Vector' does not implement surface method 'seq'`. Reverted and
re-verified. Cost: one 32-second rebuild, instead of a wrong mechanism inscribed in a stone.
`[[feedback_a_design_sentence_is_not_the_disk]]` It needs its own lair study first.
