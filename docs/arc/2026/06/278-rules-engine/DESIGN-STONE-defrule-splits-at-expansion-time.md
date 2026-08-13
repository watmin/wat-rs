# DESIGN STONE — `defrule` lifts its `where` bodies; the compiler stops carrying rete's grammar

> # ⛔ BLOCKED 2026-08-12 — STRUCK, STOPPED, NOT BUILT. Read this box before anything below it.
>
> A rider was released against the brief and **STOPPED correctly before touching `wat/rete.wat`**,
> tree left clean. The blocker is real, was verified independently by the orchestrator, and it
> invalidates the § THE SHAPE sketch as written.
>
> **THE GAP: the lifted defn needs its parameters TYPED, and the macro cannot know the type.**
> `defrule`'s macro body runs at macro-expansion time — a phase **before the type registry is
> populated**. Verified by run (a *calling* probe; the orchestrator's first attempt defined the
> macro without calling it and returned a meaningless EXIT=0):
>
> ```
> field-names-of form: unknown type ':usr::Temp'
> ```
>
> even for `:wat::rete::Rule`, a stdlib type baked into `rete.wat` itself. The rider also ruled out,
> by run: a bare type variable (`expects :wat::core::i64; got :T`), `:wat::type::Infer` (wired only
> for collection-literal element types, not fn signatures), and `:wat::core::Value` (does not coerce
> to a concrete param type).
>
> **AND THE OBVIOUS RESCUE IS ALSO DEAD.** Clara passes the FACT and destructures, so its fn's
> parameter type is the fact's — knowable from the pattern. wat's rete has **no whole-fact binding**:
> `ReteClauseShape::Bind { var, field }` (`src/rete/matcher.rs:295`) binds a FIELD. There is no fact
> to pass. The type can only come from *"what type is field `:c` of `:usr::Temp`"*, which is the
> unanswerable question.
>
> **AND THERE IS NO TYPE EXPRESSION TO DEFER IT WITH.** Emitting a type the checker resolves later
> (when types *are* registered) has no form: `TypeExpr` is `Path | Parametric | Fn | fresh var`
> (`src/types.rs:72-92`). No type-level field accessor exists.
>
> ★ **THE GAP IS NOT RETE-SPECIFIC, and that is the load-bearing observation for the re-plan.** Any
> DSL that lifts a user-written body into a typed defn hits this identical wall. This is the same
> shape as the privilege being deleted: not rete needing an exception, but **the language missing
> something every DSL needs.**
>
> **UNRULED — the builder's call, not the orchestrator's:**
> 1. a macro-phase reflection door (types available earlier, or a narrow "fields of a declared type")
> 2. a type-level field accessor the macro emits and the checker resolves
> 3. a whole-fact binding in the rete surface (Clara's shape) — changes the user surface this stone
>    protects
>
> Everything below § THE SHAPE's parameter question remains PROVEN and is unaffected: the mention
> ships transitively (7/5/7), a macro can mint a lifted top-level defn (STOP-3 cleared), `Rule` stays
> pure EDN, and the four REFUTED shapes stay refuted.

**Ruled 2026-08-12. Rewritten after the probes killed two earlier shapes — read § REFUTED before
proposing anything.** The builder's ruling:

> *"i fucking hate seeing dsl machinery hard coded in our rust — this means /every fucking dsl we
> envision/ requires rust changes — we're telling every fucking user 'nope, you suck, fuck you,
> can't do it'"*

and on our `defrule` declining to do what Clara's does: *"sounds like we are wrong."*

## The defect — rete's own comment names it

`wat/rete.wat:2403-2404`:

> *"The macro is kept **TRIVIAL**: it quotes both vectors as-is… `make-rule` does the per-element
> split at **runtime**."*

`defrule` performs no transformation. It quotes patterns and `where` bodies **together**, so code
lands inside a runtime `quote` — and the compiler needed a private door to reach back in. That door
is every piece of rete grammar hardcoded in Rust: `Boundary::MakeRule`, `is_where_form()`,
`check_make_rule_when`, the four `normalize_make_rule*` fns, `expand_make_rule{,_when}`.

**None of that is the price of rete being complicated. It is the price of a macro that punted.**
Clara's `defrule` splits at expansion time (`clara-rules/.../compiler.clj:405` emits a *named* `fn`
via `mk-node-fn-name`), which is why Clojure needs no new language form. The macro is the mechanism,
and it belongs to the DSL author.

## Strategic frame (the builder's — this is step one, not cleanup)

Rete's remaining cost is **interpreted wat, ~95% of the work in large stress tests**. The
purity/determinism/totality campaign exists to make rete's forms *compilable*; the jump table (#49)
implements function calls; two of three compilers are built (`src/rete/compiled_cond.rs`,
`compiled_rhs.rs`), this is the third, and the other two get failed over once it lands.

> **This is the proving ground for making wat itself compilable. rete first, wat second.**

## THE SHAPE — what `defrule` emits

```clojure
;; user writes (UNCHANGED — 222 call sites across 78 files do not move):
(:wat::rete::defrule :usr::ok-rule
  :when [(:usr::Temp (?c <- :c))
         (:wat::rete::where (:usr::big? ?c))]
  :then [(:usr::Hot :c ?c)])

;; defrule emits — one LIFTED defn per where body, plus one MENTION:
(:wat::rete::core::defn :usr::ok-rule$where0 [?c <- :wat::core::i64] -> :wat::core::bool
  (:usr::big? ?c))                                   ;; ordinary top-level code

(:wat::core::defn :usr::ok-rule [] -> :wat::rete::Rule
  (:wat::core::let [$where0 :usr::ok-rule$where0]    ;; the MENTION — ships the dep
    (:wat::rete::make-rule "usr::ok-rule"
      (:wat::core::quote [(:usr::Temp (?c <- :c))
                          (:wat::rete::where (:usr::ok-rule$where0 ?c))])
      (:wat::core::quote [(:usr::Hot :c ?c)]))))
```

**`Rule` does not change.** `lhs` stays `PV<WatAST>`; the quote holds a *call form*.

**Naming, grounded:** `$` is this codebase's established marker for macro-minted names — `$impl`
(`bracket.wat:298`, `core.wat:870`), `$core-record`/`$holon-record` (`core.wat:1812-1814`), all six
existing sites spelled out, none abbreviated. So `$where0`, **not** `$w0`. The binder and the FQDN
share the word so a reader of the mention knows exactly which lifted body it ships, and `0/1/2…` line
up positionally with the conditions — which matters because order is load-bearing in the network.
`_where0` is **wrong**: `_` means *intentionally unused*, and this binding is the mechanism that
ships the dep — naming it `_` invites a cleanup pass to delete it and break delivery silently
(and per task #67, `_`-prefix also slips must-use gates).

## PROVEN — all four requirements, by run, with non-vacuity controls

| requirement | mechanism | evidence |
|---|---|---|
| `Rule` stays pure EDN (snapshot `{facts,rules}` + the wire survive) | no fn on the Rule | 293.W **refuses** a record field of fn type — `ImpureFieldInPureAggregate`, proven |
| the wat ORACLE can evaluate it | it is an ordinary call form | `tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat` is this exact shape, green |
| the jump table can compile it | a call to a named fn LAW A has proven pure/deterministic/total | `purity.rs:1058` LAW A ARMED; `:1533` the transitive rete-primitive walk |
| **the dep SHIPS** | one code-position mention, **transitively** | `PC 7 · BASE 5 · MENTION-1 7` — the mention equals writing everything in call position |

The shipping numbers, with the controls that make them mean something:

```
probe_where_body_dep.wat      POSITIVE-CONTROL 6 · BASELINE 5 · SUBJECT 5   ← today: NOT shipped
probe_lift_transitive.wat     PC 7 · BASE 5 · MENTION-1 7                   ← lifted: shipped, transitively
```

`MENTION-1 == PC` is the load-bearing equality: **one** mention collects the lifted fn *and*
everything the lifted fn calls. The macro needs no free-symbol analysis — it mentions the name it
just minted.

## ⛔ REFUTED — do not re-propose these. Each died to a run, not an argument.

1. **quasiquote's `~` as the code hole.** For resolution it means *resolve in place*; for evaluation
   `runtime.rs:10891` **evaluates** the escape and `value_to_watast`s the result (`:11074`). The form
   is destroyed.
2. **Making `forms` resolve-transparent.** `forms` is the **child-program constructor** — it must not
   resolve locally because the universe it names is not this one (`spawn.wat:559`). Imposed the
   check: build green, positive control confirmed live, floor **4367/24**, failures clustering on the
   `forms` spec module, declaration-lift, and *the services child-program path*.
   `probe_resolver_quote_awareness.rs:19` states it outright — *"forms arguments are data, not live
   call heads"* — with a fixture naming `ghost-inner`/`ghost-other`, deliberately non-existent.
3. **A `fn` stored on the `Rule`.** 293.W containment: a record may hold only pure fields. Rules are
   records **because they cross the wire and persist** — R5's `{facts, rules}` snapshot. Clara can
   hold closures because its rules live in one JVM image and its durability blob is the separate
   version-fragile monster R5 documents; **we traded that away deliberately.**
4. **The `Condition` ADT** (`lhs: PV<Condition>`) and its lockstep oracle migration. Ruled 4/4 on
   2026-08-12 and then **voided**: it existed only to hold the fn that (3) proves cannot be held. The
   four questions were sound; the premise was not. `Rule.lhs` stays `PV<WatAST>`.

## What deletes — and the acceptance test is the deletion itself

After the lift, everything inside the quote that needed compiler attention has moved OUT of it: the
`where` body is an ordinary top-level defn (resolved, expanded, normalized, type-checked as code),
and what remains quoted is a call whose head **the macro itself minted** — so a typo is
unrepresentable, and the mention keeps it live.

Expected to delete: `Boundary::MakeRule` · `is_where_form()` · `check_make_rule_when` · the four
`normalize_make_rule*` fns · `expand_make_rule{,_when}`. `quote_boundary` becomes **language forms
only**; no library's name remains in the compiler.

★ **Do not assert the deletion — impose it.** Delete them and run the floor. A green floor is the
proof the lift made them dead; a red one names exactly what still depends on them (R52
`QVOD LEX ACCENDIT` — the fire is the worklist).

## Grounded facts (verified this session — do not re-derive)

- **The reflection marker is the SIGNATURE** (`src/rete/collect.rs:56-60`): zero-arg + return type
  `:wat::rete::Rule`. `defrule` must keep expanding to that; the body may do anything.
  `collect-rules` is untouched. ⚠ The lifted `$where0` defns return `:bool`, so they are **not**
  picked up as rules — the marker discriminates them for free.
- **`Rule`** (`wat/rete.wat:52`): `{name <- String, lhs <- PV<WatAST>, rhs <- PV<WatAST>}`.
- **`make-rule` producers:** `wat/rete.wat:2425` (`defrule`) and `wat/query.wat:189`
  (`sift-rules-defsvc`) in production; three scratch sites.
- **Scope numbers are rough orientation only — the compiler owns the true worklist.**

## STOPs

- **STOP-1** — if the split cannot keep `defrule` expanding to a zero-arg defn returning
  `:wat::rete::Rule`, halt: `collect-rules` finds rules by that signature and nothing else.
- **STOP-2** — `wat/query.wat:189` emits the same shape from a *generated* service. Migrate it in the
  same strike or the corpus splits into two rule dialects.
- **STOP-3** — if a lifted `$whereN` defn cannot be minted at top level from inside the macro's
  expansion (hoisting/registration), halt and surface it: the whole shape rests on that defn being
  ordinary top-level code.
- **STOP-4** — do NOT bundle the `Boundary::MatchesSubject` cleanup (same class, one step out — see
  `228b68fa`), and do NOT honour `MakeRule` in `closure_extract` as an interim patch: that adds a
  fourth Rust consumer of rete's name, which is what this stone deletes. **Known cost of holding
  that line: rule delivery stays broken until this lands** (`6/5/5`).

## Owed before the strike

- ~~An intueri cast on the lifted-name spelling~~ **CAST + WEIGHED 2026-08-12 — RATIFIED.**
  `<A>` = `:usr::ok-rule$where0` (the lifted body's fqdn) · `<B>` = `$where0` (the mention's
  binder). Target + full reasoning: `wat-scripts/intueri/defrule-lifted-where-naming.wat.intueri`.
  The deciding argument was the ward's, not mine: **the two committed probes already mint
  `$where0` verbatim**, so the composition was proven under this exact spelling.
  ⚠ **The INDEX is a design claim, not a measurement** — both probes mint exactly ONE lifted fn for
  a single-`where` rule. Nothing has exercised two `where` conditions in one rule. Index-by-position
  is chosen because it is the only scheme that survives an arbitrary predicate body (there is no
  name to borrow from `(:wat::rete::where (:wat::core::i64::> ?c 100))`), and **the strike owes that
  probe.**
- **Whether `is_where_form` genuinely dies or merely relocates.** The quoted `(:wat::rete::where …)`
  wrapper still marks a condition kind for the network builder. The claim is that nothing needs to
  look *inside* it any more — that is what the deletion test settles.

## Kin

R63 `INTERROGATIO VENATVR` (asking "how do we compile this?" audits everything it touches) ·
R60 `QVOD FAVET PRIMVM CADIT` (four of my own premises died getting here, and the answer improved
each time) · R59 `NISI FRANGAS NIHIL PROBAS` (every column above is a break, not a pass) ·
R5 (the `{facts, rules}` snapshot that makes the Rule a record) · R28 `SOLVIMVS NE MENTIRETVR`.
