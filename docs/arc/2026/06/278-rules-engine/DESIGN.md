# Arc 278 — the wat rules engine (RETE / Clara-shaped, VSA-matched)

> **STATUS: STUB — queued, BLOCKED behind arc 277.** Surfaced 2026-06-17 the moment `wat-lint`'s rule
> abstraction (`form → findings`, a registry, run over a fact base) was recognized as a *production-rules
> system*. The builder: *"did we just declare the need for something like Clara?… wat-rules basically
> just described itself."* The plan, set explicitly: **once `wat-lint` (arc 277) is done and proven
> working, build the RETE/Clara engine and swap `wat-lint` onto it.** Not started; do NOT start before
> 277 lands.

## The trigger

`wat-lint` (arc 277) defines a rule as `(form → Vector<Finding>)` — an `LHS` pattern (match a form) →
`RHS` action (emit a finding/fix), a registry of rules, run over a fact base (the parsed forms). That is
the production-rules paradigm, named. The current `wat-lint` impl is *rule-based* (map the rule set over
the forms) but **not** a rules-*engine* — no RETE network, no incremental re-matching, no rule **chaining**
(one rule's output feeding another's LHS), no fact base richer than "the forms." Arc 278 builds the
engine; `wat-lint` is its proving ground and first consumer.

## Prior art (collision, recorded straight — and it is the builder's OWN tool)

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

## Out of scope / discipline

- **Do NOT build before 277 is proven.** The whole point of the sequence is that lint earns the
  abstraction empirically first; building the engine speculatively is exactly the forcing-function the
  project forbids. Blocked, on purpose.
- **Exact-match RETE first; VSA-matched second.** The fuzzy/coincidence matcher is the novel horizon, not
  the v1 — ship the deterministic engine, prove the swap, then explore VSA-fired rules.
- **No config** (per the toolchain doctrine, arc 277): the engine has one correct behavior; rules are
  data, suppression is a rune, not a knob.

## The output contract — pure facts + name-holes, never faked judgment (builder, 2026-06-17)

The engine is **pure: pure in → pure out, no IO in the reasoning**. The detection conditionals
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
