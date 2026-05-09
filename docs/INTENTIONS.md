# Intentions

**Wat is engineered for user-directed LLM coauthoring.** Every
design choice — what's in, what's out, what's awkward, what's
forced — flows from one frame: *the language exists so a human
articulates intent and an LLM implements it reliably*.

This doc names that frame and the disciplines that flow from it.
It's the WHY behind the substrate. The HOW lives in
`USER-GUIDE.md`, `CONVENTIONS.md`, and the per-feature docs.

---

## The frame

Most languages assume a human author writing every line. Wat
assumes a human-directed LLM coauthor — a hybrid where:

- The **human** holds intent: the goal, the architecture, the
  constraints, the judgment about what to ship and what to defer.
- The **LLM** holds expression: turns intent into form, follows
  the substrate's grammar, runs tests, writes code at a pace
  humans can't.

The handoff between intent and expression is where errors live.
A language that's permissive at this seam (many ways to express
the same idea, mutable state, dynamic dispatch, late binding) is
LLM-hostile: the model picks inconsistently, drifts across files,
and accumulates subtle bugs the human can't trace back to a
single decision.

A language that's **strict** at this seam (one canonical form per
task, no synonyms, no escape valves, mutation-free, statically
type-checked, brutally honest in its diagnostics) is LLM-friendly:
the model defaults to correct because correct is the path of
least resistance.

Wat is the second kind. Purposefully.

---

## The disciplines

Five constraints that flow from the LLM-first frame.

### 1. One canonical path per task

For each task category, wat ships exactly one form. No synonyms.
No alternates. No "ergonomic shortcuts" that mean the same thing
as the canonical form.

| Task | Form |
|---|---|
| Iteration → see [`ITERATION-PATTERNS.md`](./ITERATION-PATTERNS.md) | 7 canonical patterns |
| Function definition (named) | `:wat::core::defn` |
| Function value | `:wat::core::fn` |
| Iteration to fixpoint | `defn` + tail call (TCO) |
| State sharing | three tiers (immutable Arc / ThreadOwnedCell / spawned program) — see [`ZERO-MUTEX.md`](./ZERO-MUTEX.md) |
| Module-local binding | `:wat::core::def` |
| Local binding | `:wat::core::let` |

If you find yourself wanting to express something two ways, one of
the ways is wrong. The substrate rejects synonyms by construction.

### 2. Brutal honesty in diagnostics

When something's wrong, the substrate tells the LLM **exactly**
what shape was expected and what shape was found, with the
canonical migration recipe inline.

Example (arc 168 walker output):

```
let bindings must be a vector `[name expr name expr ...]`.
Got legacy nested-pair-list `((name expr) (name expr) ...)`.

Migration:
  - Outer brackets change from `(...)` to `[...]`.
  - Inner pair-lists `(name expr)` flatten to alternating
    `name expr` inside the outer vector.
  - Destructure binders stay as a vector of symbols:
    `((a b c) rhs)` becomes `[[a b c] rhs]`.

Example:
  Before:  (:wat::core::let ((x 1) (y 2)) (+ x y))
  After:   (:wat::core::let [x 1 y 2] (+ x y))
```

The diagnostic IS the migration recipe. The LLM reads it,
applies the translation mechanically, moves on. No reverse-
engineering, no guessing.

This is **substrate-as-teacher** — see
[`SUBSTRATE-AS-TEACHER.md`](./SUBSTRATE-AS-TEACHER.md). Failures
are not crises; they are work items the substrate emits for the
LLM to execute.

### 3. Mutation-free by construction

Wat has no `set!`, no `var`, no mutable bindings. State changes
happen via:
- Returning new values from pure functions
- Sending messages to programs (the third tier — see ZERO-MUTEX)
- Atomic primitives at substrate level (rarely user-facing)

For an LLM coauthor, mutation-free means **local reasoning**. The
model can read a function and know exactly what it does without
chasing through global state, mutable closures, or hidden
side effects. Every value is what it appears to be.

### 4. Force naming

If something's worth recursing, it's worth naming via `defn`. If
something's worth registering at module scope, it's worth a `def`.
Anonymous local recursion is unsupported by design.

Names are documentation. Named functions are profileable, testable
in isolation, debuggable by stack trace, and discoverable by LLM
introspection (`:wat::help :user::my-function`, when arc 018 ships).

The cost: a small ergonomic tax (you can't write a tiny one-shot
recursive lambda inline). The benefit: every iteration in your
codebase has a name, a type signature, and a test.

### 5. Static type-check at startup

Every form is checked before any program runs. Type mismatches,
unresolved references, malformed bodies — all surface at startup,
not at runtime.

For LLM coauthors, this means **the type checker IS the test
loop**. Write code → run startup → see errors → translate →
repeat. The cycle is fast, mechanical, and lossless.

Compare to dynamic languages where type errors surface at runtime,
sometimes far from the point of mistake. Those are LLM-hostile —
the model fixes a symptom in one place while the cause lives
somewhere else.

---

## What the user gets

- **A language they can direct without micromanaging.** Tell the
  LLM what you want; the substrate's strictness keeps the LLM
  honest about what it's writing.
- **Code that's uniformly traceable.** No mixed-style codebases.
  Every iteration looks like every other iteration. Every
  recursive function is named. Every module-level binding is
  in the symbol table.
- **Failures that are migration recipes.** A failing test in wat
  is a directive; the LLM can execute it without your
  intervention.
- **A long lifespan.** The constraints that make wat LLM-friendly
  also make it human-friendly years later. Code reads the same
  way it was written.

---

## What the LLM gets

- **Zero ambiguity about which form to pick.** The path-of-least-
  resistance is the path-we-want.
- **Diagnostics that teach.** Every error message names the
  expected shape and the migration to it. The model corrects
  mechanically.
- **Local reasoning.** Mutation-free + statically-typed means a
  function's behavior is determined by its inputs and its body.
  No hidden state to track.
- **Substrate-checkable contracts.** Type signatures, name
  bindings, position rules (`def` only at top level), arity
  rules — all enforced at startup. The model can't ship code
  that violates them.
- **Failure as data.** When the model gets something wrong, the
  substrate tells it precisely what's wrong. The model fixes
  it; the human doesn't have to mediate.

---

## What this protects against

- **LLM hallucination of forms.** A form must exist in the symbol
  table to be called. Hallucinated function names fail at startup
  with `UnresolvedReferences`. The model can't fake a primitive
  into existence.
- **LLM drift across files.** One canonical form per task means
  every file uses the same shapes. No mixed-style codebases that
  arise when the model picks different syntax in different places.
- **LLM overcomplication.** No synonyms means no tempting "let me
  use this fancier shape here." There's one form; you use it.
- **Hidden state regressions.** Mutation-free means changes are
  visible at the call site. The model can't accidentally introduce
  shared mutable state that breaks a future test.
- **Type drift.** Static checking at startup means incompatible
  changes surface immediately. A signature change with downstream
  consumers fails until every consumer is migrated.

---

## The path forward

Wat is being built in tiers. The substrate (this repo) is the
foundation. Above it:

- **Foundation toolkit**: formatter (`wat-fmt`), linter (`wat-lint`),
  coverage (`wat-cov`), documentation (`wat-doc`), interactive
  evaluator (`wat-repl`), runtime help (`wat-help`). These make
  every wat program reviewable, testable, and discoverable.
- **App stack**: HTTP server / router / client / api spec, schema
  validation (positive security at boundaries), CLI argument
  parsing, kwarg macros. These let wat programs participate in
  the outside world.
- **Network tier**: mutually-authenticating wat-vms with
  cryptographic identity, content-addressed programs, verifiable
  execution. Distributed wat by construction.

Each tier inherits the LLM-first disciplines from the substrate.
The formatter has one canonical output. The linter enforces one
canonical style. The HTTP server has one canonical handler shape.
The network's signed eval boundary makes execution verifiable
across machines.

The end state: a complete vertical stack where a human articulates
intent at any layer, and the LLM implements reliably down to bytes
on the wire.

---

## Cross-references

- [`ITERATION-PATTERNS.md`](./ITERATION-PATTERNS.md) — the seven
  canonical iteration shapes; concrete demonstration of "one
  canonical path per task"
- [`ZERO-MUTEX.md`](./ZERO-MUTEX.md) — the three tiers of state
  ownership that replace mutation
- [`SUBSTRATE-AS-TEACHER.md`](./SUBSTRATE-AS-TEACHER.md) — failure
  engineering applied at the substrate level
- [`CONVENTIONS.md`](./CONVENTIONS.md) — naming + namespace rules
- [`COMPACTION-AMNESIA-RECOVERY.md`](./COMPACTION-AMNESIA-RECOVERY.md) §
  5 — the four questions framework that gates every architectural
  decision
- [`USER-GUIDE.md`](./USER-GUIDE.md) — the practical how-to;
  every section is an instance of the disciplines named here

---

*Wat doesn't take features away to be parsimonious. It takes
features away because every feature an LLM-coauthor doesn't need
is a feature an LLM-coauthor can misuse. The substrate's
strictness is a gift to the human directing it: their intent
arrives at execution intact.*
