# Arc 249 — macros are total, pure programs over forms (Clojure macro parity, raised)

**Status:** OPEN 2026-06-04 — DESIGN (scope revealed mid-arc). **The threading verdict was the
canary; this is what the arc was meant to be.** 249 opened as a small "should we build `->`/`->>`"
verdict. Building threading (249.1, shipped — `SCORE-249.1.md`, `DESIGN-249.1-threading.md`)
**revealed the real defect:** a wat macro body is a *template*, not a *program*, so the entire
idiomatic Clojure macro family (`->`, `->>`, `cond->`, `as->`, `when`, `condp`, `case`…) is
inexpressible in wat and must live in Rust forever — a ceiling on the clojure-on-rust / self-hosting
identity. The arc's true scope is lifting that ceiling. (Renamed from `249-threading-macro-verdict`;
precedent — it was already renamed once, `thread-last` → `threading-macro-verdict`.)

## The thesis

A wat macro body becomes a **total, pure program over forms** — not a quasiquote template. This is
**Clojure's macro model minus the two foot-guns Clojure tolerates by accident:**

| Foot-gun Clojure allows | wat forbids it — by construction |
|---|---|
| **Non-termination** — a macro can loop forever at expansion time | **Total**: macros are built from finite combinators, no open recursion → expansion always terminates and stays *visible* (the LLM-first virtue) |
| **Impurity / IO** — a macro can `slurp`/`println`/hit the network at compile time → non-reproducible builds, effects firing an unpredictable number of times | **Pure**: the macro-eval environment binds no `:wat::kernel::*` (the effect namespace), no clock, no randomness → expansion is a *deterministic* function of the source |

This is the **third axis of "Clojure-shape + added rigidity,"** after **type strictness** (no
implicit coercion, arc 237) and **dialect honesty** (fn-first/threading, arc 247 + 249.1). wat is
Clojure where Clojure is right, and *stricter* where strictness eliminates a failure class. The
macro layer joins the type layer as a place wat is legitimately **better than the thing it is
faithful to**: a wat macro *cannot hang the compiler* and *cannot make a build irreproducible*.
(User framing, 2026-06-04: *"we already deviate from clojure on type strictness — this is another
axis of rigidity."*)

## The two invariants (the named deposit)

- **TOTAL** — every macro expansion terminates, in a bounded and visible number of steps.
- **PURE** — every macro expansion is a deterministic, effect-free function `forms → forms`.

Together: **macro expansion is a pure total function.** Same source → same expansion, on any
machine, any number of times, always terminating, always inspectable.

## How totality is free — combinators, not open recursion

We do **not** enforce termination with a checker or a fuel budget (a fuel limit is just
Turing-completeness with a leash — it fails loudly on legitimate deep work and still admits
non-termination as a concept). Instead:

> **The macro language exposes bounded combinators, not open recursion.**

A macro body is built from a **pure structural core**:
- iteration: `fold`/`reduce`, `map`, `filter` over *finite* forms (248's `for` is the `map` case)
- branching: `if` / `cond` / `match` on form shape
- form access + construction: `first`, `rest`, `cons`, `empty?`, `count`, list/vector builders,
  quasiquote `` ` `` / `~` / `~@`
- pure scalar ops: arithmetic, comparison, boolean, keyword/symbol manipulation
- `let` + lambdas *over the above*

Every one of these is a **finite catamorphism** — it consumes a strictly-finite structure and
cannot loop. A program composed only of them is **total by construction** (Turner's total-functional
discipline). No termination analysis required; the *absence* of open recursion is the proof.

**Claim to validate at strike:** this set is *exactly* enough for the idiomatic macro library —
`->`/`->>` = `foldl` + `if` + form-construction; `cond->` = fold-with-if; `when`/`unless` = `if`;
`condp`/`case` = fold/`map` over clauses; n-ary `and`/`or` = fold. The rare genuinely-recursive
tree-walking macro is **affirmatively out of scope** (it needs open recursion; a separate,
deliberate decision — exactly the door arc 248 left open, walked through only if a real need
surfaces).

## How purity is free — the kernel namespace is already the effect boundary

wat already segregates every effect under `:wat::kernel::*` (spawn, `println`/`eprintln`, `recv`/
`send`, channels, signal polling, the stop flag). The macro-eval environment simply **does not bind
`:wat::kernel::*`** (nor clock/randomness, of which wat has none in the pure core). Purity is not a
new mechanism — it is the existing effect namespace, withheld at expansion time.

## What this earns (the cleanup)

- **`for` (arc 248)** stops being a bespoke Rust template-built-in and becomes the `map` member of
  the combinator set — one of many, not a special case.
- **`->`/`->>` (249.1)** move from the Rust desugar (`src/macros.rs thread_desugar`) into **wat
  code** — threading becomes the **first wat-macro citizen** of the new model, and the Rust desugar
  is HARD CUT. `keyword/of` likewise reconsidered for wat-code rehoming.
- **`&form` / `&env`** become *meaningful*: once a macro body runs code, it can inspect the call
  form (`&form`) and the lexical environment (`&env`) — wat already threads `env`/the call span to
  the expander but a template can't query them. (Later stone; v1 is the combinator body.)

## The home (249.2a) — lift `macros.rs` before building the engine

`src/macros.rs` is flat (2415 lines) with six responsibility clusters (registry, error, parse/
register, the expander, the template walker, the built-ins). We are about to bolt a whole new
subsystem (the engine) onto it. Four-questions verdict (2026-06-04) — **lift it to a warded home
first, in-arc:**

- **Obvious?** YES — lift the foundational, multi-responsibility file *before* growing it;
  born-warded beats piled-into-a-bigger-flat-file.
- **Simple?** YES — a no-behavior-change structural split (responsibility → submodule, re-export),
  the proven homes-walk pattern (7 homes already stamped); it makes the engine *simpler* (a clean
  new submodule, not a 1000-line addition to a 2415-line flat file).
- **Honest?** YES, decisive — the DESIGN already concedes the threading change "rides macros.rs's
  *future* ward." Doing major macro work on a flat un-warded file while invoking the warded-substrate
  methodology is the asymmetry-as-defect the pattern names (arc 244 doctrine). Lift-first pays that
  debt; build-then-lift means re-touching everything (churn).
- **Good UX?** YES — the engine is born in an L1+L2=0 home; future macro work extends a stamped,
  LLM-extensible federation, not a monolith.

It *composes*: the engine is a new submodule the home gains, while 249.3/249.4 (threading + `for`/
`keyword/of` → wat code) **cut** the Rust built-ins out — the home gains the engine and sheds the
desugars in the same arc.

**Scope guard:** only `macros.rs` is 249's to lift. The engine reuses `eval_in_frozen`/`Environment`
from `runtime.rs`/`freeze.rs` — those are flat quarries with their own future arcs; the engine
touches them via a small restricted-environment constructor, *not* a lift. We do not drag them in.

## Mechanism — two candidate engines (a real fork, analyzed not pre-decided)

The macro body must *evaluate* at expansion time. wat already has an evaluator (`eval_in_frozen`,
`register_runtime_defs`) and the combinator prims already exist (45 sites: map/filter/foldl/foldr/
first/rest/cons/if). Two ways to run them at expansion time:

- **(i) Reuse the runtime evaluator in a fenced environment.** A macro-eval `Environment` that binds
  **only** the pure-combinator prim set (explicitly *not* `:wat::kernel::*`, and *not* arbitrary
  user `defn`s — which could recurse/effect). Run the existing evaluator over the body with quoted
  forms as data. Pros: no second evaluator; prims are battle-tested; homoiconic (`Atom`=quote /
  `Materialize`=unquote → forms already *are* data). Cons: must fence the environment so nothing
  effectful/recursive leaks in.
- **(ii) A dedicated total-pure mini-evaluator.** Pros: the restriction is structural (no effect/
  recursion machinery exists to leak — purity/totality by construction, not by fencing). Cons: a
  second evaluator to keep in step.

**Lean: (i) reuse, with a fenced environment** — the prims exist, the homoiconic substrate is built
for it, and "the macro environment binds the pure core and withholds the kernel" is a small honest
restriction. (ii) is the fallback if fencing proves leaky. Decide at the 249.2 strike via probe.

## The v2 horizon — purity as a TYPE property (where the type-strictness axis reconnects)

v1 fences by *namespace* (macro-eval binds the pure core, not `:wat::kernel::*`) and forbids calling
user `defn`s. The natural generalization — **a later stone or arc** — is a type-level
**`pure & total` effect** a function can carry, letting the checker *verify* a user fn is
macro-callable. That is the type-strictness axis (arc 237 lineage) doing the enforcement: a fn typed
`pure total` is admissible in a macro body; an effectful or openly-recursive one is a compile error.
The bridge between wat's two rigidity axes — flagged here, owned later.

## The close — scope settled by four-questions (2026-06-04)

**The revealed principle:** *cut what the engine **obsoletes** (in-arc, Honest-forced); defer what
it merely **enables** (follow-on, affirmatively-scoped).* The four-questions split the naive "lean
vs fuller" axis (FM-3 — too coarse to answer cleanly): leaving a redundant Rust `for` /
`thread_desugar` next to the engine's `map`/`fold` is the *two-ways* one-canonical-path + HARD-CUT
refuse — so the redundancy-cut is **forced into the arc**, not chosen. But the library the engine
*enables* (`cond->`, `when`, …) shadows nothing, so shipping it is not obligated — affirmatively
scoped to follow-on.

So **249 closes with:** the warded `src/macros/` home + the engine (249.2a/b) + `->`/`->>` as wat
code with the Rust desugar cut (249.3) + `for`/`keyword/of` rehomed/absorbed with their Rust
built-ins cut (249.4). Library + `&form`/`&env` = named follow-on.

## Slicing (proposed — stepping stones)

- **249.1 — threading verdict + Rust desugar. ✓ SHIPPED** (`6ba27ca0`, REMARKABLE one-shot). The
  canary. Becomes the behavioral contract for 249.3.
- **249.2a — lift `src/macros.rs` → `src/macros/` warded home.** No behavior change; split the flat
  2415-line file by responsibility into submodules (names via intueri cast), ward to L1+L2=0, earn
  the `vigilatum` stamp. Pays the "rides macros.rs's future ward" debt and gives the engine a home
  to be born in. Stepping stone: born-warded beats piled-into-a-bigger-flat-file. (See "The home"
  below.)
- **249.2b — the macro-eval engine** (in the new home). The fenced pure-combinator evaluator at
  expansion time (mechanism (i)/(ii) per the strike probe). A macro body may be a combinator program
  returning a form. Probe proves a *fold-shaped* macro expands (the minimal new power).
  **It MUST SUBSUME the existing computed-unquote eval** — the arc-143 `,(expr)` path
  (`unquote_argument`/`splice_argument`) today calls full *unsandboxed* `runtime::eval` at expand
  time (circumspicere finding F5 on the 249.2a ward: an impurity/determinism hole — an impure
  `,(expr)` makes the canonical AST, hence the hash, depend on runtime state, breaking
  hash-IS-identity). The engine gates expand-time eval to the pure-combinator set, closing F5 **by
  enforcement** — not a pure path *beside* the impure one. **The `src/macros/` `vigilatum` stamp is
  HELD until 249.2b closes F5 + the `env`/`sym` eval-context** (struere's wrong-level finding): a
  stamp claims *annihilation*, and "the stdlib happens to be pure" is convention, not annihilation.
  The 249.2a R2 sweep drives every *other* ward finding to zero; these two stand open, tracked here.
- **249.3 — threading reborn as wat code.** Re-implement `->`/`->>` as wat macros over the engine;
  **`tests/probe_arc249_threading.rs` is the contract** (same five gates). HARD CUT the Rust
  `thread_desugar` once the wat version passes — threading proves the model.
- **249.4 — cut what the engine obsoletes** (Honest-forced — part of the close, not optional):
  rehome `for` as a wat macro built on the engine (*doubles as engine proof*) + **HARD CUT** the
  Rust `for` built-in; absorb `keyword/of` as a blessed engine prim. The engine creates a
  `map`/`fold` surface, so the Rust `for`/`thread_desugar` become *second ways to do one thing* —
  one-canonical-path forces their removal in the same arc that creates the first way.
- **249.N INSCRIPTION** — closes when 249.2–249.4 land: engine + threading-as-wat-code + every
  redundancy cut. The idiomatic library (`cond->`/`when`/`condp`…) and `&form`/`&env` are
  **affirmatively-scoped follow-on** (enabled by the engine, not committed by 249 — a new arc as a
  caller needs them), per the principle below.
- **v2 (later)** — type-level `pure total` effect for user-fn macro-callability.

Each stone is probe-locked (FM-2-bis). Substrate surface names (the engine fn, any new module, the
combinator-set boundary) are **owed intueri casts at strike** — not hand-named here.

## Open questions (resolve in the deciding stones, grounded — not now)

- Mechanism (i) vs (ii): does fencing the shared evaluator hold, or is a mini-eval cleaner? (Probe.)
- The exact combinator boundary: is `match` on form-shape in, or only `if` + `first`/`rest`? (Lean
  **in** — pattern-matching forms is core to readable macros.)
- A macro body may call *other macros* (yes — combinator-only too) and *blessed pure prims* (yes);
  user `defn`s (no, until v2's type-level purity).
- Does any current template-macro rely on behavior the combinator model changes? (Audit at 249.2.)
- Gate placement: where does the rest of 249 sit relative to `235 → rejoin 232`? (Builder's call.)

## Refs

- The 249.1 canary + its contract: `DESIGN-249.1-threading.md`, `SCORE-249.1.md`,
  `tests/probe_arc249_threading.rs`, `src/macros.rs thread_desugar`.
- The map-only predecessor + its deliberately-left-open door:
  `docs/arc/2026/06/248-macro-comprehension/INSCRIPTION.md` ("anything more is a separate, deliberate
  decision").
- The macro substrate it builds on: `src/macros.rs` (sets-of-scopes hygiene, `expand_form`,
  `expand_macro_call`, `MacroDef`, `macroexpand`/`-1`); the effect boundary `:wat::kernel::*`
  (runtime.rs); the type-strictness lineage (arc 237, `feedback_no_implicit_coercion`).
