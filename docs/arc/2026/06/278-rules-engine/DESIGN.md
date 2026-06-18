# Arc 278 — the wat rules engine (RETE / Clara-shaped, VSA-matched)

> **STATUS: OPEN — 277 is proven, design in progress (2026-06-17).** arc 277 landed and is proven: THE
> SWEEP (`27688ec9`) cleaned the corpus with the self-fixing toolchain, so the `form → findings` rule
> abstraction earned its keep empirically. Surfaced the moment that abstraction was recognized as a
> *production-rules system* — builder: *"did we just declare the need for something like Clara?… wat-rules
> basically just described itself."* The bar (builder, 2026-06-17): a **competent Clara/RETE engine, done
> right, NO deferrals** — he ran Clara at AWS on hard reasoning at scale and respects RETE deeply. This
> doc is the live co-design; the surface (forms / WM / unification / query) is not yet drawn into stones.

## The trigger

`wat-lint` (arc 277) defines a rule as `(form → Vector<Finding>)` — an `LHS` pattern (match a form) →
`RHS` action (emit a finding/fix), a registry of rules, run over a fact base (the parsed forms). That is
the production-rules paradigm, named. The current `wat-lint` impl is *rule-based* (map the rule set over
the forms) but **not** a rules-*engine* — no RETE network, no incremental re-matching, no rule **chaining**
(one rule's output feeding another's LHS), no fact base richer than "the forms." Arc 278 builds the
engine; `wat-lint` is its proving ground and first consumer.

## Prior art (collision, recorded straight — and it is the builder's OWN tool)

- **`:wat::form::matches?` (arc 098, `src/form_match.rs`) — OUR OWN prior art, the closest one.** A
  *Clara-style single-item pattern matcher* already shipped in the substrate: grammar
  `(:demo::Trade (= ?side :side) (= ?qty :qty) (= ?side "buy") (> ?qty 10))` — binds logic vars
  `?side`/`?qty` to struct fields then constrains them, with six compare ops (`= not= < > <= >=`). It is a
  **query over structured data that needs no forward-chaining engine** — exactly the "clara-y / prolog-y
  queries" the builder remembered. The worked use is `examples/interrogate/wat/main.wat` ("Q2: matches? —
  the Clara predicate over the lifted struct value"). This is the engine's **alpha-node test, already
  built**: one fact, intra-condition variable binding, constraints. Arc 278 adds what it lacks — the
  working-memory record, multi-condition JOINS (the beta network: the same `?var` unifying *across*
  conditions/facts), `fire-rules!`, the `defrule`/`defquery` forms, and `query`.
- **`scripts/challenges/007-batch` + `docs/challenges/007-batch/RETE_CHALLENGE.md` (holon root) — the VSA×Clara
  conceptual prior art** ("Holon-Powered Rete": VSA/HDC complementing Clara; the `defrule [Order (= status :pending)
  (> total 10000)] [Customer (= id ?customer-id) …] => (insert! …)` shape). Revisit when the VSA-matched LHS
  (the novel horizon) is built.
- **Clara** — the Clojure RETE rules engine the builder ran **at AWS** for the Shield DDoS pipeline (per
  BOOK.md / the chronicle: "Clojure for the Kinesis KCL interop and the rule engine, Clara"). The
  reach-and-find pattern at career scale: reached for a clean way to organize lint rules, landed on a
  great he'd already shipped in production.
- **Drools / CLIPS** — the JVM and the classic production-rules systems.
- **Forgy's RETE (1982)** — the incremental pattern-matching algorithm underneath all of them: a
  discrimination network that re-evaluates only what changed, so N rules over M facts is not N×M per tick.
- The coordinate recurs across the builder's work: Clara @ AWS → the **eBPF tail-call rule-trees** in
  `holon-lab-ddos` (1M rules in O(tree depth)) → `wat-lint` rules over AST. A place he stands.

## What is genuinely ours (the two reasons to build it, not just import the idea)

1. **Homoiconic rules.** wat is EDN — a `defrule` is *data* natively (quote/unquote, the same tongue you
   compute in). Clara's rules are Clojure data too, but ours are AST-native and live in the substrate
   that already self-rewrites (`fix.wat`) and self-analyzes (`deporder`). The rules engine is one more
   self-hosted tool, configured in the language it runs on.
2. **The VSA-matched LHS — the real novelty.** Swap RETE's *exact* pattern match for **coincidence**
   (`coincident?`, similarity over a floor) as the matcher: rules fired by *similarity*, not equality —
   a **fuzzy / probabilistic production-rules engine**, the holon substrate as the fact network, `bundle`
   as the working memory, `cosine` as the match. That is not Clara; it is Clara's matcher replaced by the
   thing holon already is. (This is the deep one — note it; the exact-match engine ships first.)

## Charter rules — what the engine SHIPS WITH

The engine is not just a `defrule` runtime; it ships with a standing set of **charter rules** (the
substrate's own quality rules, migrated off `wat/lint.wat`'s hand-written predicate ladders onto the
engine as its first real rule-set). These are required, not optional — the engine is born watching the
corpus. The migrating set: `concat-abuse`, `nested-if-=-ladder`, deporder load-order (rule-zero). Plus
ONE the engine must ship with that the hand-written lint never had, surfaced by arc 277.1c-fix (the fix
tool caught writing the very smell it abolishes — `277/REALIZATIONS.md` R2):

### `nested-if-boolean-collapse` (the generalized boolean-ladder rule)

> **REQUIRED charter rule.** The narrow `nested-if-=-ladder` (which keys on `(if (= VAR LIT) true …)`
> only) is a *special case* of this. The engine ships the general rule; the narrow one folds into it.

- **LHS (pattern):** an `(if cond then else)` form in which — recursively through the `else` chain —
  **every leaf branch is a boolean literal** (`true`/`false`). I.e. the whole nested `if` evaluates to a
  boolean and is therefore a boolean *expression* wearing control-flow clothes. The conditions are
  arbitrary (`(= v l)`, `(contains? s c)`, any predicate) — the rule keys on the **boolean-leaf
  structure**, not the condition shape.
- **RHS (action):** emit a finding + a fix that rewrites the ladder to the equivalent `and`/`or`/`not`
  combination. Worked examples (both real, from this campaign):
  - `(if (= x "a") true (if (= x "b") true (if (= x "c") true false)))`
    → `(:wat::core::contains? (:wat::core::HashSet :T "a" "b" "c") x)` (the `= VAR LIT → true` subcase —
      a HashSet membership; this is the existing `nested-if-=-ladder` rewrite, now subsumed).
  - `(if (contains? inner "\"") false (if (contains? inner "{") false (if (contains? inner "}") false true)))`
    → `(:wat::core::not (:wat::core::or (contains? inner "\"") (contains? inner "{") (contains? inner "}")))`
      (the `pred → false … true` subcase — a negated disjunction; the exact form 277.1c-fix shipped and
      had to hand-clean because the narrow rule could not see it).
- **Why it MUST ship with the engine:** the hand-written lint missed it (narrow predicate), and the
  toolchain *demonstrably wrote it itself*. A rules engine whose charter is "the corpus stays clean by
  construction" cannot omit the most general form of the smell it already half-catches. Shipping the
  general rule closes the class; the narrow rule was the placeholder.

(The fix's RHS obeys the output contract below — for the boolean-collapse the rewrite is a pure fact, no
name-holes; contrast the concat→format fix whose compound case yields holes.)

## The plan (the swap — author-adjacent / prime-drop)

1. **277 first.** `wat-lint` ships and is *proven working* (rules catch real bad forms across the corpus,
   `wat-fix` applies, `wat-fmt` formats). This proves the rule abstraction is correct before any engine
   is built — the proving ground.
2. **Build the engine beside the working lint.** A general `defrule` + RETE matcher (exact-match first),
   rule chaining, an incremental fact network. It does NOT touch `wat-lint` yet — it sits proven on its
   own deftests, primed.
3. **Swap `wat-lint` onto it.** The naive map-over-rules retires; lint's rules re-home onto the engine
   (their `(form → findings)` shape is already the engine's rule shape, so the migration is mechanical).
   `wat-lint` becomes the engine's first real consumer; the rule-of-three is satisfied by construction.
4. **Then the other consumers.** `deporder`, the DDoS detection lab (the eBPF rule-trees' wat sibling),
   and the verification market — each a fact base + a rule set over the one engine.

## The query layer + the two-engines decision (builder, 2026-06-17)

**arc 278 = the Clara/RETE engine; `defquery` is its native, complete query layer.** `defquery` is not a
lesser query — it is the RETE network read out (same condition language, same logic vars, same
non-redundant joins already computed by `fire-rules`, composes with the EDN/DAG/state-blob; a query is just
another terminal node). It is the UX the builder chose at AWS and the right paradigm for reactive reasoning
over a fact base (lint/DDoS/verification facts).

**core.logic (miniKanren / Prolog) is a DIFFERENT engine — a separate future arc, built on need.** It is
backward-chaining goal-directed *search* with **full bidirectional structural unification** (vars unify
with vars and partial terms), backtracking, generativity (run a relation backwards), recursive relations —
none of which is RETE's forward-chaining incremental-working-memory model. It would not even share the
network. So it is NOT a flavor of `defquery` to fold in here; grafting a half-miniKanren onto the RETE
engine makes two engines, both worse. **wat hosts BOTH as independent engines** — that is the point: a
substrate with Clara AND core.logic is a serious reasoning platform. Build core.logic **when a real Prolog
interface need surfaces** (don't build the forcing function, [[feedback_dont_build_the_forcing_function]];
when the need is real, block-and-build it as its own arc, [[feedback_deferred_dep_becomes_necessary_block_and_build]]).

**Shared substrate, NOT gold-plated toward it.** The two engines naturally share substrate: EDN facts, the
`?var` logic-variable reader, the binding-map shape, and a unification primitive. RETE needs only the
*equality-join* subset of unification; core.logic needs full structural. Build RETE's unification for
RETE's needs — if it factors cleanly it becomes a stepping stone the future core.logic arc reuses, but do
NOT over-generalize it now for a hypothetical (speculative generality is the forcing function in disguise).
Plant the flag; let the need reveal the rest.

## Out of scope / discipline

- **No deferral of the engine's competence** (builder steer, 2026-06-17) — the real RETE (sharing +
  memories + delta propagation + truth maintenance + real unification + `defquery`) is the deliverable;
  decomposition into stones is method, not deferral; no naive stand-in we'd rip out. See the NO-DEFERRAL
  block above.
- **VSA-matched LHS is an additive second MATCHER, not a deferred core.** The exact-match matcher
  (`form::matches?`-grade) ships as the engine's matcher; the fuzzy/coincidence matcher (`cosine`/`bundle`
  over a floor) is a *second impl behind the same matcher seam* — the novel horizon, slotted in when
  built, not blocking the competent exact engine. Design the matcher as a seam so VSA drops in.
- **No config** (per the toolchain doctrine, arc 277): the engine has one correct behavior; rules are
  data, suppression is a rune, not a knob.

## The output contract — pure facts + name-holes, never faked judgment (builder, 2026-06-17)

The engine is **pure: pure in → pure out, no IO in the reasoning**.

> **EXTREMELY RIGID RULE (builder, 2026-06-17): the RHS *action* tooling must be PURE — always.**
> Not only the LHS matching/reasoning: the RHS too. A `defrule`/`defquery` RHS may ONLY *construct and
> return data* — the fix record, the query bindings — never act on the world.
>
> **The precise boundary — IO in the BOOKENDS, the ENGINE is the pure middle** (exactly Clara's
> `insert → fire-rules → query/act`):
> 1. **Ingest (IO allowed):** IO MAY query external sources for facts and INSERT them into working
>    memory. Building the fact base by reading the world is fine — that is not the engine.
> 2. **Fire (PURE — the engine):** forms, matcher, unification, querying, RHS — top to bottom, NO IO
>    (no `read-file`/`write-file`/`println`/peer-send). The RHS only returns data, or inserts *derived*
>    facts back into the in-memory working memory (still pure — mutates the in-memory fact base inside the
>    pure run, not the world; that is RETE-explicit + later, v1 is exact-match + querying).
> 3. **Act (IO allowed):** once rules are done firing, IO MAY process the resulting state — the consumer
>    (the sweep driver / `lint-fix-file` in `wat-scripts/fixes/`) reads the fix-map and applies it.
>
> So "the engine shall never do IO" means **the fire phase**. This is the structural cure for fake
> correctness: the engine cannot act on the world, only describe — every result is inspectable data
> before disk.

The detection conditionals
(currently `if`/predicate ladders scattered in `wat/lint.wat` — `concat-abuse?`, `concat-head?`,
the ladder detector, …) all move onto `defrule` LHS; the engine consumes a fact base and produces a
**map of things to fix** — DATA, not effects. A separate IO layer (`lint-fix-file` / the sweep driver in
`wat-scripts/fixes/`) applies the map.

The load-bearing rule, surfaced by the arc-277.1c concat→format fix: **a fix that needs a JUDGMENT must
not fabricate it as a fact.** The concat→format auto-fix can mechanically name a *bare-symbol* slot
(`count` → `{count}`) — that is a fact. It CANNOT honestly name a *compound* slot (`(i64::to-string n)`)
— a good name is a judgment. So the engine's map entry for such a fix carries the **decomposition** (the
template skeleton + the value-ASTs + explicit **name-holes**), NOT a guessed name. The map-consumer
(a human, or a dedicated naming pass) fills the holes and applies; the engine never emits
`{arg0}`-style noise dressed as a real name. (This is why arc 277.1c-fix ships bare-symbol-only and
defers compound naming HERE — the four-questions killed the in-rule heuristic: not Obvious, not Simple,
and judgment-as-fact violates pure-in→pure-out.)

This shapes the engine's value type: a fix is `{location, kind, data, holes?}` — a hole is an unresolved
judgment the consumer must supply. A fix with zero holes is auto-applicable; a fix with holes is a
*proposal* the consumer completes. The engine stays pure either way.

### The fact base must carry POSITION-PURITY (the arc-283 sweep demanded it)

The arc-277 sweep proved a fix can be syntactically perfect and still illegal *in its position*: rewriting
a `string::concat` to `(format …)` is valid in a **runtime** position but refused inside a **defmacro
body** (`format` is a macro; the macro-eval purity gate, arc 249 F5, rejects it at expand time). The
blind sweep broke the whole stdlib (deftest 0/263); reverted.

So the engine's **fact base must include, per form, its expand-time/runtime POSITION** — "is this node
inside a `defmacro` body?" — as a first-class fact a rule's LHS can match on. A macro-introducing fix
(concat→format) gates on it: fire only where the position is runtime. The complementary half is the
**callable's purity class** as queryable metadata (arc 255 — `is X expand-time-legal?`); a rule asks
*both* — "is what I'm introducing legal where I'm introducing it?" — instead of guessing. Until the
engine carries position-purity, the concat→format fix stays form-local-unsafe (the SWEEP applies only
the position-independent `ladder→contains?`, since `contains?` is pure-total = legal everywhere).

## The lifecycle — a working-memory record that holds BOTH facts and rules (builder, 2026-06-17)

The central data structure is a **working-memory record that carries the facts AND the rules it will run**
when `fire-rules!` is called on it (Clara's *session*). The lifecycle is wat's freeze idiom — build
mutable, fire, query frozen:

1. **Construct** a working memory: `(working-memory rules…)` — a record holding the registered rules
   (rule registration happens ON the working memory) and an empty fact base.
2. **Write phase (IO in the bookend, inserts pure):** some DB / file / network call computes facts; you
   `insert` them into the working memory. Reading the world to build the fact base is IO and fine — it is
   outside the engine.
3. **`fire-rules!`** — the PURE engine runs: alpha tests (per-condition, = `form::matches?` extended),
   **beta nodes** (joins / unification across conditions — the same `?var` made consistent across facts),
   forward-chaining derives new facts into working memory. Returns the **frozen, fired** working memory.
4. **Read phase:** `query` against the frozen working memory → binding data; then the consumer does
   whatever IO it wants on that frozen state.

So the shape is **construct → write/insert (IO) → fire-rules! (pure) → query/read (then consumer IO)** —
the three-phase purity boundary above, made into one record's lifecycle. (Naming — `working-memory` /
`fire-rules!` / `insert` / `query`, and whether the build→frozen split is a typestate like
transient→persistent — goes through `intueri` before it is built.) `form::matches?` (arc 098) is the
alpha test we already own; the beta join + the working-memory record + `fire-rules!` + `query` are the new
work.

### Value-semantics + re-firing

The working memory is a **VALUE** — `fire-rules!` returns a frozen, fired working memory; it does not
mutate in place. To do more work after firing (run different rules, add facts, re-derive), you **construct
a NEW working memory FROM the frozen one** and attach whatever rules you want, then fire again. No in-place
re-fire; the frozen WM is an immutable seed for the next one. (Clojure's persistent→transient turned inside
out: a frozen value seeds a fresh builder.)

## EDN-representable at all times + the network as a renderable DAG (builder, 2026-06-17)

The whole session is **EDN-representable at every moment** — the working memory, the facts, AND the rules
serialize to an EDN string and reconstruct from one. Facts are records (EDN by the typed-record
discipline); rules are homoiconic EDN forms; so the session is data end to end. (Rides the
`EdnRepresentable` doctrine; unlocks serializable / pausable / portable rule evaluation — the
verification-market thread, [[project_metered_eval_verification_market]].)

This forces the engine's **network to be DATA, not hidden imperative state.** The alpha nodes
(per-condition tests) and the beta nodes (joins) are reified as a **tree/DAG of typed node records** —
`AlphaNode` / `BetaNode` / `ProductionNode` (+ a root) referencing one another — such that the compiled
rule network **renders as a coherent DAG**. Alpha nodes shared across rules are the sharing that makes it a
DAG, not a tree; walking the node values emits a graph (dot/mermaid) for free. The network-as-data is
required from v1.

**Non-redundancy is the whole point — do not make a wasteful tree (builder, 2026-06-17).** RETE's value
over naive matching IS that it never runs a rule or a condition redundantly. Three mechanisms, all required
(they ARE the engine — without them it is not RETE, it is the wasteful tree):
1. **Node sharing** — rules sharing a condition share the alpha node; a shared LHS prefix shares beta
   nodes. A test is evaluated ONCE no matter how many rules depend on it. (Same sharing that makes the
   network a DAG — structure and speed are one mechanism.)
2. **Node memories** — alpha/beta memories store matched facts and partial-match tokens, so a partial
   match is never recomputed.
3. **Delta propagation** — a fact insert/retract propagates only the CHANGE through the affected nodes,
   never a full re-scan. N rules over M facts is therefore NOT N×M.
This is a hard performance requirement from v1, not a later optimization — the wasteful tree is forbidden
outright. (And it is *because* of memories + delta propagation that truth maintenance — retract a fact,
retract its consequences — falls out naturally rather than being bolted on.)

**The state blob (builder, 2026-06-17).** After `fire-rules!` runs, the user can ask for **the state
blob** — one self-contained EDN artifact carrying *the input facts, the final working memory, and the
associated rules*. It is the complete, reproducible record of a run: what went in, the rules that
transformed it, and the final (derived-fact-inclusive) state that came out. Hand someone the blob and they
can replay the rules over the input facts and confirm they reach the same final working memory — the
**verification property**, and the audit/debug artifact. Requestable on demand (an accessor on the frozen
WM), not necessarily always materialized. (Input facts are kept distinct from the final WM precisely so the
run is replayable — provenance, not just a snapshot.)

**NO DEFERRAL — we build the real RETE, done right (builder, 2026-06-17).** The builder ran Clara at AWS
to solve hard reasoning problems at scale and has deep respect for RETE; the bar is a **competent
Clara/core.logic-grade engine**, not a lesser placeholder. So we do NOT ship a "naive evaluator now,
real network later." The real discrimination network — alpha memories, beta **join nodes** with proper
cross-condition **unification** and node memories, incremental propagation, **truth maintenance** (retract
a fact → retract its logical consequences), negation/existence/accumulation as the design calls for, and
genuinely relational querying — is the deliverable.

**Decomposition is method, not deferral.** "Done right" still means examinare strikes: each stone ships a
**real component of the real engine** (a working join node, real unification, real truth maintenance) —
never a stub or a naive stand-in we would later rip out. The distinction the builder is drawing: do not
defer the *capability*; you may (must) decompose the *build*. We are not done until the competent engine
exists, and no stone calls itself done by shipping less than a real piece of it.

## The alpha condition language ALREADY EXISTS — `form::matches?` (grounded 2026-06-17)

`:wat::form::matches?` (arc 098, `src/form_match.rs`; documented USER-GUIDE.md:3598) is not merely "an
alpha test" — it is the **complete single-fact condition language**, already built, type-checked, idiomatic:
- **field-binding / destructuring** — `(= ?var :field)` pushes `?var → field-value` into scope for
  subsequent clauses. THIS is wat's destructuring; we do NOT import Clojure nested-map destructuring (a
  competing idiom over the one we already have).
- **comparisons** — `= < > <= >= not=`
- **boolean** — `and` `or` `not`
- **test escape** — `(where <wat-expr>)`: arbitrary wat expr evaluated in the binding scope → `:bool`.
  This IS Clara's `:test`.
- logic vars `?var` lex natively; Clara no-error semantics (non-match / wrong type → `false`).

So the per-fact (alpha) matching layer is **DONE**. What arc 278 genuinely adds is everything ABOVE one
fact: the **beta network** (cross-condition unification — a `?var` bound in one condition unifying against
the same `?var` in another, across DIFFERENT facts), the working-memory record, `fire-rules`, truth
maintenance, accumulators, the node DAG, and `defrule`/`defquery`/`query`. A reach-and-find: the engine
reaches for exactly the matcher a prior self planted in arc 098. This sharpens the decomposition — alpha is
not new work; beta + working memory + fire + TM are.

## Naming (intueri cast, weighed 2026-06-17)

**Namespace: `:wat::rete`, home `wat/rete.wat`.** intueri confirmed the builder's lead. Honesty is the
deciding axis: `rete` names Forgy's actual algorithm — a *commitment* to being the real thing, which cannot
drift into a "rules-lite" placeholder without the name becoming a lie. Beats the runner-up `:wat::rules`,
which under-claims (every `if`-chain is "rules"; this is a compiled discrimination network) AND collides
with `:wat::lint::rule-*` (linter rules ≠ production rules). `:wat::logic` over-claims toward Prolog/core.logic
(the separate future engine); `:wat::engine` is a container-name (the macro-level `utils.wat`).

**Vocabulary** (intueri-weighed):
- `defrule`, `defquery` — OK (house `def*` idiom; `defquery` commits to the Clara read-out model).
- `working-memory` — keep (the RETE compound earns itself inside the `:wat::rete::` namespace).
- **`fire-rules` — no bang. CONFIRMED (builder, 2026-06-17).** intueri Level-1 catch on the builder's own
  `fire-rules!`: in the Clojure convention `!` signals side-effects/mutation, but this engine is PURE
  value-semantics (working-memory → frozen working-memory). Builder: "our rete may not perform IO during
  fire-rules (no bang; no mutations)." The bang is dropped — `fire-rules` speaks the true promise.
- `insert`, `query` — OK.
- `AlphaNode` / `BetaNode` / `ProductionNode` — keep (Forgy terms-of-art; correct jargon, same
  precise-over-colloquial justification as `rete` itself).

**Resolved by the second intueri cast (2026-06-17), weighed against the house convention:**
- **Persistent collections** (stone 0): `:wat::core::PersistentMap` + `:wat::core::PersistentVector` —
  beside `HashMap`/`Vector`. Full-English-word convention (`HashMap`/`HashSet`/`Vector`); `persistent` names
  the semantic property honestly; `PMap`/`PVec` fail Obvious (jargon compression, no house precedent);
  `SharedMap` LIES (suggests Arc-shared mutable, not structural sharing). Home `:wat::core::` — they are
  general-purpose, NOT rete-owned (`:wat::rete::` would lie). The type IS the opt-in.
- **Accumulators**: namespace `:wat::rete::acc::`, with the `acc/` shorthand in rule bodies. Members
  `acc/count` `acc/sum` `acc/min` `acc/max` `acc/average` `acc/distinct` `acc/all` `acc/group-by` (wat-ified
  from Clara's `grouping-by` — the `-ing` participle mumbles; `group-by` mirrors Clojure). Custom constructor
  `acc/accumulator` (NOT Clara's truncated `accum`, a Level-2 mumble).
- **State blob**: **`:wat::rete::Snapshot`** (type) + `snapshot` (accessor). ⚠ WEIGH-CORRECTION of intueri's
  proposed `ReteSnapshot`: the house convention intueri ITSELF cited — `:wat::lint::FixEdit`,
  `:wat::telemetry::Event`, `:wat::deporder::Violation` — puts the domain in the NAMESPACE, never repeated in
  the type name; so `:wat::rete::Snapshot`, not `…::ReteSnapshot`. (builder: confirm)
- **`retract`** — confirmed; the inverse of `insert` (term of art: CLIPS / Clara / Drools).

## Clara feature inventory — keep / cut / debate (IN PROGRESS, builder debates each)

Grounded in Clara's docs (clara-rules.org, 2026-06-17). Leans below are mine against the locked invariants
(pure engine / typed-record facts / value-semantics / non-redundant RETE / `defquery`); the builder
decides each as we go.

| Clara feature | lean | why |
|---|---|---|
| `defrule` / `defquery` / sessions / `insert` / `fire-rules` / `query` | **KEEP** | the core lifecycle |
| Fact expressions (type + constraints + `?f <-` binding) | **KEEP** | extends `:wat::form::matches?` (arc 098) |
| Boolean `:and` / `:or` / `:not` | **KEEP** | real reasoning needs negation |
| `:test` (predicate over bound vars, e.g. `(> ?a ?b)`) | **KEEP** | cross-condition joins beyond equality |
| Accumulators | **KEEP ALL + custom (decided)** | ship the full set — `count`/`sum`/`min`/`max`(+`:returns-fact`)/`average`/`distinct`/`all`/`grouping-by` — plus the `accum` custom constructor (reduce/combine/retract/convert/init). The `retract-fn` is what drives truth-maintenance over aggregates (incremental update on support loss, not recompute) |
| Truth maintenance — logical insertion + auto-retract of consequences | **KEEP** | the heart of declarative reasoning; falls out of memories+delta |
| Rules-as-data (defrule → a data map; build rules programmatically) | **KEEP** | native to wat (EDN-always); stronger than Clara here |
| Inspect / explain activations (`clara.tools.inspect`) | **KEEP/ADOPT** | ≈ our renderable DAG + state-blob (prior-art collision) |
| Durability / session serialization | **KEEP/ADOPT** | ≈ our EDN-always + state-blob (prior-art collision) |
| Side-effecting RHS (`insert!`/`retract!`/arbitrary Clojure IO) | **CUT** | RIGID rule: RHS pure → returns data / derives in-memory facts only |
| `insert-unconditional!` | **CUT (decided)** | builder: triple-denied — it has a bang, he doesn't want it, and it makes firing order significant (breaks the declarative model). ALL insertion is logical (TM-participating) |
| Salience (global rule-firing priority) | **CUT (decided)** | builder: never reached for it in 5+ yrs of Clara; order is STRUCTURAL (forward-chain dependency), not a priority knob — see Ordering below |
| Arbitrary fact-type (`:fact-type-fn` / maps-as-facts / `:ancestors-fn`) | **CUT (decided)** | builder: ALL facts are records, end of story — base `:wat::Record` or holon `:wat::holon::Record`; type hierarchy = our `derive`/typesub (arc 237/267) |
| `:exists` | **KEEP as sugar (decided)** | `:not` is the primitive (negative-join node); `:exists` ≡ `(:not (:not X))` — existential, fires once if ≥1, binds nothing, no multiplicity; free, zero extra engine machinery |
| Destructuring in conditions | **CUT Clojure-style — we already have ours (decided)** | wat's `(= ?var :field)` in `form::matches?` IS field-extraction/destructuring (binds field→`?var` into scope); Clojure nested-map destructuring would be a competing idiom — not the right fit |

## Conflict resolution / firing order — STRUCTURAL, not a knob (decided 2026-06-17)

**No salience.** The builder (5+ yrs of Clara) never reached for it, and it is the wrong model: a global
priority number is action-at-a-distance, and "no order" or "definition order" *both invite user mistakes*.
The right answer is the whole point of forward chaining — **order is STRUCTURAL: the data-dependency
order.** A rule whose LHS matches a fact another rule derives MUST fire after that rule; the forward-chain
DAG sequences them by construction. Users cannot mis-order because they do not order — the dependency
structure does. Independent activations (no derived-fact dependency between them) are **confluent**: their
relative order does not change the final fixpoint (pure derivation + truth maintenance guarantee it). So
firing order is forward-chain-structural where it matters, immaterial where it doesn't; salience is neither
needed nor offered.

**Facts are records, end of story** — base `:wat::Record` or holon `:wat::holon::Record`. No maps, no
arbitrary objects, no `:fact-type-fn`. The fact's type is its `class_fqdn`; the type hierarchy is our
existing `derive`/typesub edges (arc 237/267). This is the no-magic discipline as a structural law of the
engine.

> **The one subtlety in "order is structural":** negation (`:not`) over *derived* facts — the
> stratification problem — where a non-existence condition could depend on a fact another rule derives.
> Real RETE handles this structurally: the negative-join node propagates only when its negated input is
> stable, so the *network* resolves it, not a salience knob. Captured as a build concern for the beta/`:not`
> node; it does NOT reintroduce salience.

## Substrate prerequisite — persistent collections (grounded + decided 2026-06-17)

**Question (builder): do we need a new primitive for the tree/DAG, or do we have the tooling?**

**Representation — we have it.** Records (`AlphaNode`/`BetaNode`/`ProductionNode`) + `HashMap` (id→node
index) + `Vector` (child-id edges) + `fresh-symbol` (arc 274, ids) + `HashMap/get`/`assoc`/`keys` = a
DAG-as-data: EDN-serializable, renderable, value-semantics. No new primitive needed to BUILD or REPRESENT
the network. `deporder` already builds record-graphs with the HashMap idiom; `lint` walks trees recursively.

**Performance — the real gap.** Every wat collection is `Arc<std::…>` with **clone-on-write** updates:
`Vec(Arc<Vec>)`, `wat__std__HashMap(Arc<HashMap>)`, `wat__std__HashSet(Arc<HashSet>)`. `HashMap/assoc`
literally clones the whole map then wraps a new Arc (`src/collection/eval.rs:703`: "Arc strategy:
clone-then-new-Arc"). So one node-memory update is **O(n)**. RETE does many small memory updates per fact
insert (delta propagation); clone-on-write makes each O(n) — the wasteful tree reborn at the data-structure
level. Clara/Clojure get non-redundancy because their collections are **persistent (HAMT, structural
sharing)** → O(log n) immutable updates, cheap modified-copies.

**DECISION (builder, 2026-06-17): expose persistent collections (the expected Rust perf work).** The `im`
crate is ALREADY a wat dependency (`src/wat_edn_bridge.rs`), so exposing `im::HashMap`/`im::Vector` as wat
values is block-and-buildable, not a from-scratch HAMT. Deferred-dep-becomes-necessary
([[feedback_deferred_dep_becomes_necessary_block_and_build]]): the RETE non-redundancy bar REVEALS
clone-on-write as the bottleneck. Division of labor (the "does a macro need it?" boundary applied to perf):
- **Rust** = the persistent-collection primitive (`im::*` as wat values: structural-sharing
  `assoc`/`get`/`dissoc`/`keys`/`vals`, EDN round-trip) + any hot intrinsic profiling later demands. The
  expected, bounded perf work — and a **language-wide** win (clone-on-write is a latent perf issue for the
  whole values-flow-through language; RETE is just its forcing function).
- **wat** = the WHOLE engine on top: node types, network compile, alpha activation (reuse `form::matches?`),
  beta joins + unification, `fire-rules`, truth maintenance, accumulators, `defrule`/`defquery`/`query`, the
  DAG render, the state blob. Builder: if `im::*` lets nearly all the work be wat, "i'm fucking stoked."

## Decomposition (proposed — examinare strikes; each ships a REAL piece, no naive stand-ins)

0. **Persistent collections (RUST prerequisite).** Expose `im::HashMap`/`im::Vector` as wat values:
   structural-sharing `assoc`/`get`/`dissoc`/`keys`/`vals`, EDN round-trip. FM-2-bis probe: N incremental
   assocs stay cheap (structural sharing, not clone-on-write) + EDN round-trip. Stands alone as a
   language-wide substrate win; RETE non-redundancy is its forcing consumer. (Bookkeeping: own arc or 278.0 —
   builder's call.)
1. **Node types + network-as-data (wat).** The node records + the working-memory record holding the network
   (id→node persistent map); compile a rule-set → network with alpha-node SHARING; the EDN/DAG render. No fire.
2. **Alpha activation (wat).** `insert` a fact → through alpha nodes (reuse `form::matches?`) → alpha
   memories. Single-condition rules fire end to end.
3. **Beta joins + unification (wat — THE HEART).** Cross-condition `?var` unification; beta memories;
   partial-match tokens; beta-prefix sharing.
4. **`fire-rules` + production + truth maintenance (wat).** Delta propagation; logical insertion of derived
   facts; cascade-retract on support loss; frozen WM + state blob.
5. **`defrule`/`defquery`/`query` (wat).** The homoiconic surface; query read-out over the frozen WM.
6. **Accumulators (wat).** count/sum/min/max(+:returns-fact)/average/distinct/all/grouping-by + custom; the
   retract-fn drives TM over aggregates.
7. **Swap `wat-lint` onto the engine** — the rule-of-three consumer; `nested-if-boolean-collapse` lands as a
   rete rule in the migration.

Horizons (separate, on-need): VSA-matched LHS (a second matcher behind the seam); the core.logic relational
engine (a different engine wat also hosts).

## Four questions (sketch, to weigh when it opens)

- **Obvious?** A `defrule` reads as `LHS → RHS`; the engine runs rules over facts and fires actions —
  the production-rules paradigm everyone in the lineage already knows.
- **Simple?** RETE is *not* simple — it earns its complexity only at scale (incremental, chaining,
  many-rules). The discipline: build it when a consumer's scale demands it (lint at corpus scale + a
  second/third consumer), not before. The naive engine is simpler and correct for small rule sets.
- **Honest?** Mark the seam — what we have today is rule-based mapping; the engine is RETE; do not call
  the map an engine until the network exists.
- **Good UX?** One `defrule` surface, homoiconic; lint/deporder/DDoS all consumers of one engine; the
  fuzzy matcher (if built) makes "rules over similarity" a first-class, novel affordance.
