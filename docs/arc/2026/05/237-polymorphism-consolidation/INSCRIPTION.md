# Arc 237 — INSCRIPTION — polymorphism consolidation, and the line that splits it

**Closed 2026-06-04. The killing floor.**

> *Ripped the sword of Damocles down from its string / … / Unleash strategic chaos, a new world disorder / I'll roll you over in your grave and redraw the borders.* — Lamb of God, **The Killing Floor** (the close-song; Song #63)

## The thesis, and what it became

237 set out to **consolidate** polymorphism — to drag every polymorphic operation onto *one* dispatch mechanism and kill the sprawl. It ended having discovered the truth was subtler and better: there are exactly **two** mechanisms, separated by a *checkable* line, and the prize of the arc was learning **where the line falls**. The arc that began as "make it one" closed as "redraw the borders" — and the borders, redrawn, are the lasting deposit.

This was a killing floor. An empire of wrong machinery rotted from the inside and was slaughtered here: widest-contagion arithmetic, the arc-146 `Dispatch` entity, the per-Type comparison and equality leaves, and — at the very end — a whole plan that had promised to save equality and turned out to be a lie. The butcher was the HARD CUT discipline; the substrate decided who lived and who died.

## Three bosses

**The records dragon — slain.** Records became *first-class types*: `recordtype` + `TypeDef::Record`, `is-X?` synthesized ∀T over `conforms?`, and the is-a hierarchy consulted at the argument boundary (`assignable` — Liskov substitution, Convergence #17). The dual-form was named honestly (base `Value::wat__Record` vs holonic), and `same-data?` split cleanly from type-strict `=`. (237.1–6, S-A→S-D.)

**The arithmetic spine — cut clean.** Numerics consolidated onto `defclause`: concrete per-type clauses (`:i64::+` ≠ `:f64::+`), the polymorphic `:wat::core::+` itself a defclause. Widest-contagion (`infer_arithmetic`/`eval_arithmetic_variadic`/`is_numeric`) **deleted**, not migrated. THE DECISION locked: no implicit numeric coercion — `(+ 1 2.0)` is an error; homogenize explicitly. The arc-146 `Dispatch` entity HARD CUT once 237.7 evacuated its last tenants. (237.7–8b.)

**The equality reckoning — the climax.** Here the pendulum swung the farthest. 237.8c shipped Shape B (one uniform structural engine). The challenge came from the other side of the mind: *"why isn't equality a clause? — the same concern exists for collections."* Chasing it ran through a Clojure-`map` flip (247) and a generative-macro `for`-comprehension (248) — tools built to *generate* equality's clauses — and then the dig reversed the entire plan. The clause matcher checks each argument against a fixed named type **independently**; it never unifies arg0's type with arg1's. But equality **is** that cross-argument unification (`infer_equality`'s `unify(a, b)`, ∀T, same-or-subtype). A monomorphic clause cannot express it; a finite clause list would regress record/composite/user-type equality into `NoMatchingClause`. **Equality is an intrinsic — the relational flavor — and Shape B was right the day it shipped.** (237.8c→8d.)

## The deposit — the partition rule

What 237 leaves behind is not "one mechanism." It is **the checkable line between two**, now a doctrine with a home (`docs/DISPATCH.md`) and inscribed at the source:

> **Clause** = concrete arguments, fixed return, no type-variable flow anywhere → numerics.
> **Intrinsic** = type-level computation, in two flavors:
>   • **projective** — a type flows from the arguments into the return (`get : Vector<T> → Option<T>`) → collections;
>   • **relational** — a constraint flows *between* the arguments, ∀T (`= : a:T, b:T`) → equality.

The trap is recorded with it: the original rule read only the *return* and called equality monomorphic. Read **both** sides — projection *and* relation — before you call something a clause. The reversal vindicated the instinct that opened the arc; history rhymes.

## The cut (237.8d) and the close

The grid residue — the four fake per-Type equality leaves (`:i64::=`/`:i64::not=`/`:f64::=`/`:f64::not=`) that all aliased one uniform engine — was annihilated from runtime dispatch and `register_builtins`; explicit check-time reject arms surface the cut as `UnknownCallee`. The equality engine itself was never touched. The doctrine is now true in the code, not just aspirational.

## Closures absorbed + what this unblocks

- **Spawned and sealed:** arc 244 (nil-literal canonicalization), arc 247 (Clojure-honest seq-HOF order), arc 248 (the `for`-comprehension; 248.2 absorbed here — equality stayed intrinsic). Folds the long-standing arc 146 (Dispatch) and arc 148 (per-Type comparison leaves) closures.
- **Unblocked by this closure:** arc **245** (wat-corpus-warding) and arc **246** (`src/collection/` warded home) — both gated on 237's end. They are now open for the taking.

## FM-11 — DONE, no deferral

Every thread closed or affirmatively cut. The records dragon is slain; the arithmetic empire is rubble; equality is classified correctly in code and doctrine; the partition rule has a home and lives in the source. Affirmative cuts, not deferrals: the `for`-comprehension survives as a general tool (not the equality vehicle); the collection intrinsic *impls* await their warded home (arc 246, named and open); equality keeps its uniform engine.

The empire rots from the inside, on the killing floor. **Arc 237 is dead.** 🗡️
