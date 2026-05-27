# Arc 237 — Remaining stone order (LIVE TRACKER)

**This is a LIVE roadmap, NOT an immutable record.** Refactor it freely as stones ship
(same status as the cliffnotes index; the `feedback_inscription_immutable` exception —
SCOREs/INSCRIPTIONs are frozen, this tracker is not). A fresh post-compaction self:
read this to know what's left, the order, and **why** the order — then verify against
`git log --oneline | grep 237` before acting (git is truth; this tracker can lag).

Arc 237 is a **two-boss level** (no one-boss-per-level rule): the records-first-class
thread AND the arithmetic/dispatch consolidation (the original spine). The records
*dragon* is slain; what remains is one records *refinement* + the whole arithmetic boss
+ the shared closure.

---

## Ordering principle (user direction, 2026-05-26)

> **Momentum: finish the paths we're on, then loop back onto consumers.**

We are ON the records path (just shipped S-A1). Finish it (S-C → S-D), close the thread,
THEN move to the arithmetic tail (the consumer-migration boss), THEN the shared closure.

**Honesty on the cross-boss order:** records (Record.wat macro + `holon_form`) and
arithmetic (Dispatch + widest-contagion) touch **disjoint** machinery — neither boss
produces an artifact the other needs. So records-first is NOT a hard stepping stone;
it's the momentum + recency-of-context tiebreaker (the is-a hierarchy S-C rides on is
hot from S-A/S-A1/S-B.2). The *intra-boss* orderings below ARE hard.

---

## The order (✓ = shipped; verify via git)

```
RECORDS PATH (finish what we're on)
  S-C   mint :wat::Record::def (base, struct-only) / :wat::holon::Record::def (struct + holon_form)
  S-D   migrate defrecord callers → the right new macro (HARD CUT)         ← the records "consumer" loop-back
        └─ records thread CLOSED (inscription deferred to 237.9)

ARITHMETIC / DISPATCH BOSS (loop back onto consumers)
  237.7 arc-146 Dispatch entities → defclauses (length/empty?/contains?/get/conj/concat/assoc/dissoc/keys/values)
  237.8a arithmetic + comparison + holon-pair + time-arith → concrete-per-type defclauses (ADDITIVE; old path still standing)
  237.8b DELETE widest-contagion (infer_arithmetic / eval_arithmetic_variadic / is_numeric) + HARD CUT arc-146 Dispatch + update AnyBanned

SHARED CLOSURE
  237.9 INSCRIPTION + arc closure — folds arc 146 + arc 148 + records S-E. TERMINAL.
```

(8a/8b is a recommended split to FINALIZE at 237.8 design-time with a blast-radius grep —
not yet locked; see "Hard dependencies" for why the build/cut boundary matters.)

---

## Hard dependencies — DO NOT reorder across these

1. **237.7 → 237.8.** *Forced:* 237.8 HARD-CUTs the arc-146 `Dispatch` entity; you cannot
   delete the entity while collection ops still live in it — 237.7 evacuates its last
   tenants. *De-risking:* 237.7 proves the `Dispatch → defclause` recipe on the SAFE
   family before 237.8 applies it to the dangerous families + pulls the deletion trigger.
   (Grep-confirm at 237.7 time that those 10 ops are the COMPLETE Dispatch tenant set —
   FM-2: don't trust the list is exhaustive.)
2. **237.8a → 237.8b (build, then cut).** The widest-contagion deletion + Dispatch HARD
   CUT is tractable ONLY after the new concrete defclauses prove green coverage. Never
   delete blind. THE DECISION is locked: no implicit numeric coercion — `(+ 1 2.0)` →
   ERROR (no clause matches); homogenize explicitly. (`feedback_no_implicit_coercion`.)
3. **S-C → S-D.** Forced — can't migrate callers to macros that don't exist.
4. **237.9 LAST, gated on BOTH bosses.** Spawn-block winding: the arc cannot close until
   every thread under it closes. 237.9 is the single closure for the whole level
   (absorbs arc 146 + arc 148 + records S-E).

---

## Shipped so far (arc 237) — git-confirmed

```
Machinery + conformance doctrine:
  237.1 typeunion (TypeDef::Union + bounded-existential unify)              d40eb4a3
  237.2 defclause foundation (arity + type dispatch)                        bdd9eb6c
  237.3 :guard + :ensure clause-keywords                                    ee5e892c
  237.4 rich :NoMatchingClause + :PostconditionFailed                       5f7bb6e5
  237.5 :wat::core::conforms? general conformance primitive                 5d667123
  237.5.fix one wildcard-free Value::declared_type_name authority (✅✅✅)    990542a9
  237.6 auto-mint is-<Name>? as named convenience over conforms?            3ae844cb

Records thread (the dragon — SLAIN):
  S0   gate probe (macro-emitted type decls go first-class)                 da059f42
  S-A  is-a hierarchy mechanism (typesub + subtype? + is_subtype + roots)   d1e9cbe9
  S-B.1 records become first-class types (recordtype + TypeDef::Record)     89c01888
  S-B.2 defrecord emits recordtype + drops its predicate                    86aebfcb
  S-A1  assignable choke point — subtyping at the arg boundary (Liskov)     531ba9b7
```

Records dragon ("records aren't first-class types") = slain: records are TypeDefs, `is-X?`
synthesizes ∀T, and the is-a hierarchy is consulted at the argument boundary (Liskov,
Convergence #17). **S-C is a refinement** the slain dragon enables (base records stop
paying for `holon_form` they don't use; holonic ones still substitute for base via S-A1)
— real arc-237 work, but NOT load-bearing for "first-class."

---

## Next room

**S-C** — mint the `:wat::Record::def` / `:wat::holon::Record::def` flavor split. Hot
context (rides on S-A/S-A1/S-B.2), small surface, low risk. Then S-D closes the records
thread; then the arithmetic boss; then 237.9.
