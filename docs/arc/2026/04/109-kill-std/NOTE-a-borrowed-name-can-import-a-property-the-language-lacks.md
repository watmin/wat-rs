# ⛔ NOTE (arc 109) — a BORROWED NAME can import a property the language does not have. `foldr` was the instance.

**Written 2026-08-18 alongside `DESIGN-STONE-118.B6b-retire-foldr.md`.** Filed in 109 because the
*class* is a naming/substrate question that outlives arc 118, and 109 is where those live.

## The instance

`:wat::core::foldr` was correct — verified by run, `(foldr - 0 [1 2 3])` → `2` = `1-(2-(3-0))`. It
did exactly what a right fold should do.

**And it was still wrong to have**, because the name came from Haskell and carried an expectation the
substrate cannot satisfy. In Haskell, `foldr`'s whole reason to exist as a distinct verb is that it is
**lazy** — `foldr f z (x:xs) = f x (foldr f z xs)` puts the recursive call in an *argument* position,
so it is forced only if `f` forces it, and `foldr (||) False (repeat True)` returns immediately.

wat is **strict**: `apply_function` evaluates both arguments before calling. So the distinguishing
property cannot exist here, and what remained was `reverse` + `foldl` — which is literally what the
implementation was (`xs.iter().rev()` then accumulate). The two other strict languages that met this
question, Clojure and Ruby, both declined to name it at all.

## The class — what to check when importing a verb

A borrowed name arrives with its *home language's* semantics attached, in the head of every reader who
knows that language. When the home language and this one differ on an **evaluation property** — strict
vs lazy, eager vs deferred, total vs partial — the name can be correct on every input and still lie
about what it is for.

**The check, before minting a borrowed name:**

1. **Why is this a distinct verb in its home language?** If the answer names a property of the
   *evaluation model* rather than of the *operation*, ask whether we have that property.
2. **If we don't — what is left?** For `foldr` the honest answer was "a composition of two verbs we
   already have." That is a strong signal not to mint the name at all.
3. **What did the other languages sharing our evaluation model do?** Clojure and Ruby are both strict
   and both spell it `reverse` + `reduce`/`inject`. Two independent designers reaching the same
   conclusion under our constraints is worth more than one designer reaching a different one under
   different constraints.

★ **The failure is invisible to tests.** `foldr` passed everything it was asked, because correctness
was never the defect. Only reading the *reason the verb exists elsewhere* surfaces it — which is a
prior-art question, not a code question, and no gate in this substrate asks it.

## Kin, and the shape it shares with them

- `NOTE-seqable-has-no-name-in-wat.md` — the mirror: a concept we HAD with no name, where this is a
  name we had with no concept.
- `docs/arc/2026/06/278-rules-engine/UNADOPTED.md` and task #48 — the adoption-count inventory.
  ⚠ **`foldr` had 5 call sites, all tests or a rename-table string, and the count was NOT the
  argument for retiring it.** Zero consumers is not evidence of deadness (`insert-all` would have
  measured zero the day it landed). The argument was the ruling on what the thing IS. Any future use
  of an adoption count must clear that bar the same way. `[[feedback_no_consumers_does_not_mean_dead]]`
