# Arc 232 — defprotocol + extend-type (the metaprogramming layer)

**Status:** ACTIVE (2026-05-23 night latest). Stone 232.0 + 232.0a SHIPPED. Stone 232.1 FM 2-bis probe SHIPPED 3/3 PASS at `f38e120`. Substrate empirically sufficient; macro work unblocked. See § STATUS below.

## Forward-correction 2026-05-23 night latest

Three changes to original 2026-05-22 night STUB design:

1. **Bind/inner renamed to Bind/{left,right} symmetric pair** (Stone 232.0a intueri cast catch; ship at commit `929679d` + `a1e4b02`). Read all `Bind/inner` references in this doc as `Bind/right`; `Bind/left` is the symmetric peer added during Stone 232.0a.

2. **Call-by-name question (line ~174-176 + Q8) answered.** Hypothesized `:wat::runtime::lookup-fn` does NOT exist (FINDING-CALL-BY-NAME-GAP.md disconfirmed it). Resolved by minting `:wat::core::apply` (Stone 232.0, commit `50e82d9`). The dispatcher uses `(apply -> :T <runtime-built-keyword> [args...])` syntax (current shape; `-> :T` is inline, no `[-> :T]` brackets per arc 145 lineage).

3. **Stone 232.1 = BUNDLE (defprotocol + extend-type macros).** Original split (232.1 defprotocol; 232.2 extend-type) retired via four-questions verdict 2026-05-23 night latest. Rationale: defprotocol's dispatcher is incoherent without at least one extending type; shipping defprotocol-alone fails Obvious + Honest + Good UX (caller gets a panic-generator until 232.2 ships). Bundle ships one complete-and-useful stone with end-to-end verification mirroring the FM 2-bis probe. Stone 232.2 slot RETIRED in the work-items table.

**Predecessors:**
- arc 226 ✓ — `:wat::holon::is?` + `extract_classifier` (the dispatch primitives this arc consumes)
- arc 228 ✓ — collection classifier-wrap (so types HAVE classifiers to dispatch on)
- arc 230 ✓ — variant retirement (uniform classifier encoding everywhere)
- arc 227 ✓ — defrecord (user-defined types exist to extend)

**Open trigger:** when a real use case writes the same classifier-cond dispatch THREE times. Likely surfaced by Truth Engine (per `project_truth_engine`), MTG enterprise (per `project_mtg_next`), or trading-lab v2. Until then, ad-hoc cond chains are honest; defrecord + namespace-bound methods cover the v1 surface.

## Origin

Dialogue 2026-05-22 night, after Stone 227.1 v3 (defclass single-data) shipped:

> *"this extend idea... this is just a macro who writes a method into the things's namespace so dispatch can work? ... i did so much meta programming ruby - shit like this all the time.. i didn't realize .. wow ... yes .. i want this."*
>
> *"we are getting very close to clojure now."*

The realization: the typed-entities doctrine + arc 226 dispatch + arc 230 uniform encoding makes defprotocol natural to host. Open extension via macro-generated defns. Same shape as Ruby's `define_method` / `class_eval`, but compile-time instead of runtime. The Clojure convergence becomes explicit.

## What this IS (Ruby intuition mapped)

```ruby
# Ruby:
class Voltage
  def initialize(magnitude); @magnitude = magnitude; end
end

module Formattable
  def format; raise NotImplementedError; end
end

# Open extension — add Formattable to any class after the fact
Voltage.include(Formattable)
class Voltage
  def format; "#{@magnitude}V"; end
end
```

The Clojure (and wat) equivalent splits this cleanly:

```
;; defrecord = data shape (Stone 227.1+; what we shipped + planned)
(:wat::holon::defrecord :ns::Voltage :fields [magnitude <- :wat::core::f64])

;; defprotocol = interface declaration + auto-generated dispatcher
(:wat::holon::defprotocol :ns::Formattable
  (format [self] -> :wat::core::String))

;; extend-type = macro that writes defns into the type's namespace
(:wat::holon::extend-type :ns::Voltage :ns::Formattable
  (format [self] (str (:ns::Voltage/magnitude self) "V")))
```

After macro expansion, you can call `(:ns::Formattable/format some-voltage)` and it routes to `:ns::Voltage/format`. New types can extend Formattable without changing the protocol or other types.

## What the macros expand to

### defprotocol expansion

```
(:wat::holon::defprotocol :ns::Formattable
  (format [self] -> :wat::core::String)
  (parse  [self <- :wat::core::String] -> :ns::Formattable))
```

Expands to (sketched):

```
(:wat::core::do
  ;; Polymorphic dispatcher per method — uses extract_classifier to route
  (:wat::core::defn :ns::Formattable/format
    [self <- :wat::holon::HolonAST] -> :wat::core::String
    (:wat::core::match (:wat::holon::extract-classifier self)
      (:wat::core::Some name)
        (:wat::core::call-by-name
          (:wat::core::keyword/of name "/Formattable-format")
          self)
      :None
        (:wat::core::panic
          (:wat::core::str "no Formattable impl for unknown classifier: " self))))

  (:wat::core::defn :ns::Formattable/parse
    [self <- :wat::core::String] -> :ns::Formattable
    ;; same shape — extract classifier from a CONTEXT (e.g., expected return type)
    ;; OR route by string content (parse is unusual — typically protocols dispatch on first arg)
    ...))
```

Each protocol-method becomes a polymorphic verb at `:ns::Formattable/<method-name>` that uses `extract_classifier` (from arc 228) to route. The actual implementation lives in the EXTENDING type's namespace under a mangled name like `:ns::Voltage/Formattable-format` to avoid collision with type-native methods.

### extend-type expansion

```
(:wat::holon::extend-type :ns::Voltage :ns::Formattable
  (format [self] (str (:ns::Voltage/magnitude self) "V"))
  (parse  [self] (:ns::Voltage (:wat::core::parse-f64 self))))
```

Expands to (sketched):

```
(:wat::core::do
  ;; Each method body becomes a defn in the extending type's namespace
  ;; with a mangled name encoding both the type AND the protocol
  (:wat::core::defn :ns::Voltage/Formattable-format
    [self <- :ns::Voltage] -> :wat::core::String
    (str (:ns::Voltage/magnitude self) "V"))

  (:wat::core::defn :ns::Voltage/Formattable-parse
    [self <- :wat::core::String] -> :ns::Voltage
    (:ns::Voltage (:wat::core::parse-f64 self)))

  ;; (Optional) Register at expand-time that :ns::Voltage extends :ns::Formattable
  ;; — enables compile-time check that protocol-using callers can rely on it
  (:wat::runtime::register-extension! :ns::Voltage :ns::Formattable))
```

That's it. Pure macro expansion. The "magic" is just `defn` calls in the right namespace + a polymorphic dispatcher that knows where to look.

## Why the mangled name (`Voltage/Formattable-format` not `Voltage/format`)

Without mangling, collisions are possible:
- `:ns::Voltage/format` could be a type-native method (defined by `defn` directly, no protocol)
- `:ns::Voltage/format` could be the Formattable extension

If both exist with the same name, ambiguity. Mangling (`Voltage/<protocol>-<method>`) keeps protocol implementations in their own subnamespace.

Convention from Clojure: protocols use a single underscore-separated prefix; we mirror with the FQDN-friendly `Protocol-method` form inside the type's namespace.

## When this earns its weight (vs ad-hoc dispatch)

Without defprotocol, polymorphic dispatch requires hand-written cond chains:

```
(:wat::core::defn report [v <- :wat::holon::HolonAST] -> :wat::core::String
  (:wat::core::cond
    (:ns::is-Voltage? v)  (:ns::Voltage/format v)
    (:ns::is-Celsius? v)  (:ns::Celsius/format v)
    (:ns::is-Distance? v) (:ns::Distance/format v)
    :else (:wat::core::panic "...")))
```

Every new type that wants `format` requires updating this cond. Closed extension. Brittle.

With defprotocol:

```
(:wat::core::defn report [v <- :ns::Formattable] -> :wat::core::String
  (:ns::Formattable/format v))
```

New type extends Formattable; `report` works on it automatically. **Open extension.**

**Heuristic:** when you've written the same classifier-dispatch cond chain three times, you've earned defprotocol.

## How this composes with the chain

```
arc 225 ✓ bridge naming (Atom narrow; to-holon/from-holon)
arc 228 ✓ collection classifier-wrap (Map/Set/Vector/...)
arc 230 ✓ variant retirement (Symbol/Keyword/Tag/Nil as Bind compositions)
arc 226 ✓ type predicates (:wat::holon::is?, extract_classifier dispatch)
arc 227 ✓ defrecord (user-defined types via classifier-wrap)
arc 232   defprotocol + extend-type (THIS arc — stub claimed; design pending)
```

Each arc layers on the previous:
- arc 232 needs arc 227's `defrecord` (so user types exist to extend)
- arc 232 needs arc 226's `is?` + `extract_classifier` (the routing primitives)
- arc 232 needs arc 228's classifier-wrap (so types HAVE classifiers to dispatch on)
- arc 232 needs `:wat::core::apply` — the call-by-name primitive shipped at Stone 232.0 (commit `50e82d9`). Current syntax: `(:wat::core::apply -> :T <head-keyword> <leading-args...> <spread-vec>)`. The dispatcher uses runtime-built keywords via `keyword/from-string`.

**Historical note:** Original 2026-05-22 design hypothesized `:wat::runtime::lookup-fn` (per arc 201 reflection). FINDING-CALL-BY-NAME-GAP.md (2026-05-23) empirically disconfirmed that hypothesis (no such primitive existed; arc 201's reflection layer doesn't cover invocation). Stone 232.0 minted `:wat::core::apply` as the resolution. The substrate is sufficient; no further primitive needed.

## Open questions for arc 232 DESIGN (when it opens)

1. **Mangling convention** — `Voltage/Formattable-format` vs `Voltage/Formattable/format` (deeper namespace) vs other? Pick what reads well + is parseable.
2. **Dispatch ergonomics** — `(:ns::Formattable/format x)` (protocol-namespaced) vs `(:wat::holon::dispatch :ns::Formattable :format x)` (substrate-mediated)? Probably the first — protocol IS a namespace.
3. **Compile-time verification** — should `(defn foo [x <- :ns::Formattable] ...)` verify at check time that x's runtime type implements Formattable? Or defer to runtime dispatch?
4. **Default implementations** — `defprotocol` could allow providing a default body that fires if no `extend-type` was issued. Clojure has this. Useful for "most types want X; you can override."
5. **Multi-arg dispatch** — Clojure protocols dispatch on FIRST arg only. Multimethods (arc 146/147) handle multi-arg. Should wat protocols follow the single-dispatch convention OR allow multi-dispatch via richer key shape?
6. **Protocol composition** — Clojure lets a type extend MULTIPLE protocols. Same expected here. extend-type per protocol.
7. **Extending built-in types** — `extend-type :wat::holon::Vector :ns::Formattable ...` should work; the Vector classifier is "Vector" (from arc 228); dispatch routes accordingly. Verify substrate doesn't refuse.
8. **The substrate primitive `call-by-name`** — ANSWERED. Does not exist as `lookup-fn`; arc 201's reflection doesn't cover invocation. Resolved by Stone 232.0 minting `:wat::core::apply`. See FINDING-CALL-BY-NAME-GAP.md and SCORE-STONE-232.0.md.

## STATUS — ACTIVE (2026-05-23 night latest)

Arc 233 (substrate diagnostic-richness) SHIPPED + CLOSED. Arc 232 RESUMED with the enriched substrate in place. Stone 232.0a typed-entities reflection layer SHIPPED. Stone 232.1 FM 2-bis probe SHIPPED 3/3 PASS confirming the bundled defprotocol+extend-type composition works on the live substrate.

**What has shipped from arc 232:**
- Stone 232.0 — `:wat::core::apply` substrate primitive (commit `50e82d9`)
- Stone 232.0a — typed-entities reflection layer: `extract-classifier` + `Bind/left` + `Bind/right` (commit `a1e4b02`)
- Stone 232.1 FM 2-bis probe — defprotocol dispatch composition empirically proven (commit `f38e120`, 3/3 PASS)

**Next:** Stone 232.1 sub-DESIGN + BRIEF + EXPECTATIONS authoring (orchestrator-direct), then sonnet spawn for the macro substrate work. Stone 232.1 ships defprotocol + extend-type macros BUNDLED per Forward-correction § above.

**Rank-up status:** The arc 233 diagnostic substrate is in place + Stone 232.0a reflection primitives shipped. defprotocol's consumer-side iteration is the live validation of the rank-up; the FM 2-bis probe already showed it firing (probe 3's `UnknownFunction(":myapp::Unhandled/Formattable-format", Span { ... })` surfaces the missing verb name + span without any added scaffolding).

See `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` for the diagnostic-richness arc closure.

## Work-items (chain ordering; ACTIVE; bundle decision applied)

| Stone | Purpose | Status |
|---|---|---|
| 232.0 | `:wat::core::apply` substrate primitive (Clojure's apply contract; convergence #16) | ✓ SHIPPED at `50e82d9` (2026-05-23); 18/18 PASS |
| 232.0a | **Typed-entities reflection layer** — `:wat::holon::extract-classifier` + `:wat::holon::Bind/left` + `:wat::holon::Bind/right`. Doctrine-imposed companions to the composition primitives. defprotocol's dispatcher consumes `extract-classifier`; defrecord accessor synthesis (separate later stone) composes `Bind/right` + `Bundle/children`. | ✓ SHIPPED at `a1e4b02` (2026-05-23 night latest); 10/10 PASS; rank-up demo confirmed |
| 232.1 FM 2-bis probe | `tests/probe_diagnostic_defprotocol_dispatch.rs` — manual defprotocol composition (no macros yet) proven on live substrate. Design substrate for the Stone 232.1 BRIEF. | ✓ SHIPPED at `f38e120` (2026-05-23 night latest); 3/3 PASS first-run |
| 232.1 | **BUNDLED — `:wat::holon::defprotocol` + `:wat::holon::extend-type` defmacros.** defprotocol generates one polymorphic dispatcher per method (template per FM 2-bis probe: `extract-classifier` + `string::concat` + `keyword/from-string` + `apply`). extend-type generates per-class `defn`s at mangled names (`:Type/Protocol-method`). Bundle ships one complete-and-useful stone; split rejected via four-questions 2026-05-23 night latest. | PENDING (sub-DESIGN + BRIEF in flight) |
| ~~232.2~~ | ~~`:wat::holon::extend-type` defmacro~~ | RETIRED — bundled into Stone 232.1 |
| 232.3 | Built-in-type extension proof (extend `:wat::holon::Vector` or similar with a sample protocol) | blocked on 232.1 |
| 232.4 | (separate stone OUTSIDE arc 232) — defrecord accessor synthesis: `:ns::Type/field-name` defns generated by defrecord macro using `Bind/right` + `Bundle/children` + name match. The gap is defrecord's, not defprotocol's; defprotocol v1 method bodies use the primitives directly. | NOT IN ARC 232 |
| 232.5 | INSCRIPTION + USER-GUIDE chapter | blocked on 232.3 |

### Trap-door audit (per `feedback_sonnet_writes_substrate` lesson from 232.0)

The arc 232.0 BRIEF authored invented `[-> :T]` syntax not present in canonical wat. Sonnet built against it; user caught it; orchestrator-direct fix violated the protocol. **Every subsequent stone's BRIEF audits for:**

- Invented syntax (cite canonical inline `-> :T` per arc 108 + defrecord.wat verbatim)
- Made-up primitive names (grep `src/runtime.rs` for every named verb before authoring)
- Wrong arg orders (verify against existing wat verbs; e.g., `Bundle/children` takes the bundle, not the bundle's name)
- Phantom dependencies (grep proves they exist; otherwise the dependency is its own prerequisite stone)
- Empirical probes for all non-trivial primitives (FM 2-bis: probe FIRST, commit, cite from BRIEF; sonnet mirrors evidence not assertions)

The substrate layer chain (232.0 + 232.0a) closes the typed-entities doctrine asymmetry. The macro layer (232.1+) builds on the closed substrate.

## Out-of-scope for arc 232 itself (subsequent stones)

- Performance optimization (cache dispatcher lookups; Clojure does this via invokevirtual + class-cache)
- Protocol satisfies? predicate (`(satisfies? :ns::Formattable some-value)` — useful but not v1)
- Method-call-as-data (`(apply-protocol-method :ns::Formattable :format args)` — reflection corner case)
- Protocol inheritance / extends (one protocol extending another)
- Generic algorithms over multiple protocols (`(map-formattable seq)`)

## When to BEGIN active work on arc 232

The arc number is CLAIMED + a stub design exists (this doc). Active work begins when one of these triggers fires:

**Heuristic:** when a real use case writes the same classifier-cond dispatch THREE times. Until then, defprotocol is theory; cond chains are honest.

Likely triggers:
- Truth Engine (per `project_truth_engine` memory) — validating LLM outputs across MANY data shapes; format/diff/serialize naturally cross-cutting
- MTG enterprise (per `project_mtg_next`) — card types, effect types, zone types — all want cross-cutting operations (cost, resolve, target)
- Trading lab v2 — order types, asset types — natural cross-cutting

When one of these surfaces 3+ cond-on-classifier chains, **arc 232's active work begins** with this DESIGN.md as the design substrate.

## What this realization MEANS for wat-on-Rust identity

User: *"we are getting very close to clojure now."*

This IS the convergence. Per CLIFFNOTES doctrine § "wat-on-Rust family lineage" — same triangle as Ruby-on-C, Clojure-on-Java, Elixir-on-BEAM. wat hosting defrecord + defprotocol + extend-type makes the Clojure surface explicit:

- defrecord — data
- defprotocol — interfaces
- extend-type — open binding
- multimethod (arc 146/147) — dispatch by arbitrary key
- defservice (arc 209) — mutex-protected mutable state (Erlang/Akka tier, beyond Clojure)
- defmacro — metaprogramming (already shipped)

wat has MOST of Clojure's core abstractions PLUS Erlang-style services + VSA-similarity type-checking. Maturity-shape ~ Clojure 2008-2009 per `project_wat_lineage`.

The convergence keeps validating: years of failure-engineering shape constraints that collapse to the same answers other greats found. Per `user_no_literature` — different starting points; same destination.

## Cross-references

- `feedback_fqdn_is_the_namespace` — namespace doctrine; defprotocol/extend-type honor it
- arc 226 SCORE — `extract_classifier` + `is?` (the dispatch primitives this builds on)
- arc 227 SCORE — defclass→defrecord rename (this notes doc post-dates the rename decision)
- arc 209 DESIGN — defservice handler-monadic shape (the (s, d) -> (s, D) pattern); extend-type's method bodies follow same shape for state-threading methods
- `feedback_refuse_easy_solutions` — defprotocol earns its weight when ad-hoc dispatch surfaces 3+ times
- `project_typed_entities_doctrine` — the foundation this builds on
- `feedback_simple_is_uniform_composition` — defprotocol IS uniform composition (one verb; N implementations; substrate routes)
