# MEASURED — B2 is BLOCKED. The `Var` gate excludes every verb B2 collapses.

**2026-08-18, against `f08c5d1a`.** Written because the builder asked *"are you ready to spawn a
shadowdancer?"* and the honest answer was **no**. This is the FM 2-bis disconfirming probe that
found out why — before a rider flight, not during one.

## The probe

A probe attempting exactly B2's composition: ONE `defn` over `Seqable<T>`, a **lazy producer**
(`stream/lazy` + `match (next …)` + `stream/cons`), recursing on the `rest` a `NextOutcome::Item`
hands back. **Its full source is embedded at the bottom of this file** — it is RED, and a RED `.wat`
cannot live under `wat-scripts/` without breaking the loader gate (see the last section for why it
is not `#[ignore]`d instead).

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
call **all work.** B2's design is sound; only its call sites are blocked. The probe becomes B1a's
RED-to-GREEN gate row (restore instructions and expected output are at the bottom of this file).

---

## THE PROBE ITSELF — kept here, deliberately NOT as a live file

⛔ **This probe is RED, and `every_wat_scripts_file_loads` type-checks every `.wat` under
`wat-scripts/`.** A RED probe there breaks the floor — verified, not assumed: the gate went
`1 failed` with exactly the five errors above.

**The obvious escape is an `#[ignore]`, and it is refused.** "Commit RED probes ignored" is a house
convention this project has already identified as *the mechanism that built the ignore pile*
(`[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`, arc 294.j — we drove
`#[ignore]` 200+ → 13, then nearly re-grew it by that exact rule). So the source lives in the
record instead of in a gated directory with a licence attached.

**★ RESTORE IT AS B1a's GATE ROW.** Copy the block below back to
`wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat`. It must go from **these five
errors to GREEN**, and its run must print:

```
2,4 | 2,4 | 2,4 | 2,4
0,1,2,3,4
0,2,4
```

That is a RED-to-GREEN transition on a committed artifact — the honest shape of a fix's proof, and
strictly better than a probe that was green all along.

```wat
;; probe-118B2-one-clause-lazy-producer.wat — the DISCONFIRMING PROBE for stone 118.B2.
;;
;; Written BEFORE B2's brief, per FM 2-bis: for a non-trivial substrate composition, grep is
;; insufficient — write the ten-line probe that attempts exactly the composition and run it. The
;; recovery doc's worked example of skipping this cost ~2 hours and a killed sweep.
;;
;; ═══ WHAT B2 CLAIMS, AND WHAT IS ACTUALLY UNPROVEN ═══════════════════════════════════════════
;;
;; B1 (488eacd0) proved a CONSUMER over `Seqable<T>` works: `[s <- Seqable<T>] -> i64`, called with
;; all four containers. Every verb B2 collapses is a LAZY PRODUCER, which is a different shape and
;; has never been run. Three things have to hold at once, and only the first is proven:
;;
;;   1. `Seqable/seq` resolves on all four containers                      ✅ proven, B1
;;   2. a `stream/lazy` body can `match` on `(next …)` and `stream/cons`   ❓ never run in this shape
;;   3. ★★ THE CRUX — the RECURSIVE call passes `rest`, a **Stream**, into a parameter typed
;;      `Seqable<T>`, INSIDE the definition of the very fn being defined.                   ❓
;;
;; If (3) fails, the one-clause design collapses and every `<verb>-stream` TWIN comes straight
;; back — because the twin exists precisely to be the Stream-typed thing a clause can recurse into.
;; So (3) is not a detail of B2; it IS B2.
;;
;; PASS = prints all four lines below. FAIL = a TypeMismatch naming `Seqable<?N>` vs a concrete
;; container, which would be the 118.3-B defect resurfacing at a *recursive* site.
;;
;; ⚠ NOTE THE ORDER OF EVIDENCE: `--check` alone is NOT sufficient here. Task #95 (confirmed live
;; 2026-08-17) — a DOTTED call head is not type-checked at all, and `Seqable/seq` is dotted. This
;; probe must be RUN. `[[feedback_a_green_test_can_prove_nothing]]`

;; ─── (2) + (3): ONE clause, lazy producer, recursing on a Stream through a Seqable param ─────
;; This is exactly what `keep` / `map-indexed` / `dedupe` / `distinct` / `interpose` become in B2.
;; Under the old world this needs FIVE defclause arms plus a `-stream` twin.
(:wat::core::defn :probe::keep-one<T,U>
  [f    <- :wat::core::Fn(T)->wat::core::Option<U>
   coll <- :wat::core::Seqable<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f value)
          ;; ★ (3) — `rest` is a Stream<T>, handed to a Seqable<T> parameter, recursively.
          ((:wat::core::Some v) (:wat::stream::cons v (:probe::keep-one f rest)))
          (:wat::core::None (:probe::keep-one f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; A STATE-CARRYING producer — the harder half of the family (`keep-indexed`, `map-indexed`,
;; `dedupe`, `distinct` all thread an accumulator across the walk). Same crux, plus a threaded arg.
(:wat::core::defn :probe::index-one<T>
  [idx  <- :wat::core::i64
   coll <- :wat::core::Seqable<T>] -> :wat::stream::Stream<wat::core::i64>
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::stream::cons idx (:probe::index-one (:wat::core::+ idx 1) rest)))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; An unbounded source — proves the migrated shape stays LAZY (termination is the assertion).
(:wat::core::defn :probe::nat
  [i <- :wat::core::i64] -> :wat::stream::Stream<wat::core::i64>
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::nat (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [keep-even (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::Option<wat::core::i64>
                 (:wat::core::if (:wat::core::= 0 (:wat::core::% x 2))
                   (:wat::core::Some x)
                   :wat::core::None))]
    (:wat::core::do
      ;; ONE definition, FOUR container kinds at the call site — the payoff. Expect 2,4 / 2,4 / 2,4 / 2,4
      (:wat::kernel::println
        (:wat::core::string::join " | "
          (:wat::core::Vector :wat::core::String
            (:wat::core::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::Vector :wat::core::i64 1 2 3 4 5))))
            (:wat::core::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::PersistentVector 1 2 3 4 5))))
            (:wat::core::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::List/of 1 2 3 4 5))))
            (:wat::core::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::Seqable/seq (:wat::core::Vector :wat::core::i64 1 2 3 4 5))))))))
      ;; state-carrying, over a List. Expect 0,1,2,3,4
      (:wat::kernel::println
        (:wat::core::string::join ","
          (:wat::core::into [] (:probe::index-one 0 (:wat::core::List/of 9 9 9 9 9)))))
      ;; LAZINESS over an INFINITE source through the migrated shape. Expect 0,2,4 — and it must
      ;; TERMINATE; an eager collapse here would hang rather than print.
      (:wat::kernel::println
        (:wat::core::string::join ","
          (:wat::core::into [] (:wat::core::take (:probe::keep-one keep-even (:probe::nat 0)) 3)))))))
```
