# NOTE — match/cond: clause-as-bracket + drop the non-return `-> :T`; the conditional form needs a new name (NOT `cond`)

> **Deferred design decision (builder, 2026-07-21, arc 278).** Surfaced reviewing the caller.1
> `probe_arc278_call_site.wat` match form. **rete (arc 278) is the stepping stone** to the broader
> `-> :T`-in-non-return-positions annihilation that this rides on; the swap lands after that. Recorded here
> per the arc-109 `NOTE-*.md` convention (sibling of `NOTE-generic-bracket-syntax-edn.md`).

## The principle (the "why", general beyond match)
Scheme uses **one delimiter for two meanings**: `(...)` is both *apply this* and *group these*. Clojure split
them — `(...)` = application, `[...]` = ordered structure, `{...}` = associative structure — which is what makes
a Clojure dialect read cleanly. wat is mid-migration from the first to the second. **The rule: anywhere a form
is grouping rather than calling, it is a bracket — no exception.** A `match`/`cond` clause is *structure* (an
ordered pattern/test paired with a body), not a call, so it is a bracket. Then the only parens left inside a
clause are the genuine calls (the pattern-constructor `(:Some f)`, the body-call `(contains? …)`), and the eye
separates structure from application instantly. This is the same move `let`/`fn`/`defn`/`defrecord` already made
(bindings/params/fields are `[...]`); applying it to clauses makes `match`/`cond` *rhyme* with the rest of wat.

## The swap (match)
```clojure
;; TODAY (Scheme heritage — list-clauses + a NON-RETURN `-> :T` annotation):
(:wat::core::match file -> :wat::core::bool
  ((:wat::core::Some f) (:wat::core::string::contains? f "…"))
  (:wat::core::None     false))

;; TARGET — bracket-clauses, and the inline `-> :T` GONE (type inferred by per-arm unification):
(:wat::core::match file
  [(:wat::core::Some f) (:wat::core::string::contains? f "…")]
  [:wat::core::None     false])
```
Two changes, both ratified:
1. **Clause `(pattern body)` → `[pattern body]`** — the bracket principle above. Kills the Scheme double-paren
   `((`. Single body form (use `(do …)` for multiple, as Clojure).
2. **Drop the inline `-> :T`** — it is a `-> :T` in a **non-return position** (a match-arm-type annotation, not a
   fn return), and those are being annihilated (builder, 2026-07-21). The arm type is *inferred* (per-arm bodies
   unified to one type by the checker), not annotated at the form. rete is the stepping stone to the general
   non-return-`-> :T` removal; this swap lands with / after it.

## `cond` — appealing but the NAME is suspect; NEEDS A NEW TERM (intueri — OWED, not yet cast)
`match` and `cond` **already share the clause shape** today: `cond` (a defmacro, `wat/core.wat:1204`) is
`(cond (test body) … (:else body))` — head = a truthy *test*; `match` head = a *pattern* that binds. Unifying
them onto the one bracket-clause is appealing (`[test body]` / `[pattern body]`), BUT: **do not name the
bracket-claused form `cond`.** A Clojure user reads `cond` and expects **flat** `test expr test expr` pairs
(Clojure `cond`/`case`/`core.match` are all flat, no per-clause delimiter) — a bracket-claused `cond` would
*induce confusion* (builder). So the unified/bracketed conditional-dispatch form **needs a new term**, distinct
from Clojure's `cond`.
- **OWED: cast intueri** for that term when the swap is drawn (intueri owns all names — do not narrate one here).
  Constraint for the cast: a name for a *bracket-claused, ordered, first-match dispatch* form (test-dispatch
  and/or pattern-dispatch), that does **not** collide with a Clojure user's `cond` expectation.
- **OPEN:** whether `match` (pattern-dispatch, typed, exhaustive) and the test-dispatch form stay two forms or
  unify under the one new term. Not sold on unifying under `cond`. Decide at draw time.

## Grounded facts (verified 2026-07-21)
- `match` is a runtime special form — `eval_match` (`src/runtime.rs:13088`), tail twin `eval_match_tail`
  (`:3578`); current grammar requires `-> :T` between scrutinee and arms (`:3596`), each arm a list `(pat body)`.
- match **catch-all is `_`** (non-binding wildcard, `runtime.rs:13073`) or a **bare identifier** (binds the
  scrutinee as that name, `:13072`) — **not** `:else`. (So `:else` is cond's terminal; a `match` over a closed
  enum closes by **exhaustiveness**, needing no catch-all — e.g. `[:None false]` completes `Option`.)
- `cond` is a defmacro (`wat/core.wat:1204`), clauses `(test body)`, terminal `:else`.

## Execution (when drawn)
A **corpus-wide wat-fix codemod** (`wat-scripts/fixes/`, never hand-edits) — a semantics-preserving delimiter
flip (`(clause)` → `[clause]`) + the inline-`-> :T` drop; hard-flip (parser requires the bracket, no bad-form
education). Fold with any concurrent match-touching sweep to touch the tree once.

## Status
**DEFERRED.** rete (arc 278) is the stepping stone; lands with/after the non-return-`-> :T` annihilation.
The new term is an **owed intueri cast** at draw time.
