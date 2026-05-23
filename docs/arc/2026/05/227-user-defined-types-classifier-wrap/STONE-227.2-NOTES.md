# Notes for Stone 227.2 — multi-field structs + methods (deferred)

**Status:** Future-stone scoping notes. Authored after Stone 227.1 v3 single-arg defclass spawn (user direction: "we should entertain something like defservice's method-closure form").

## What 227.1 v3 ships (single-arg defclass)

```
(:wat::holon::defclass :myapp::Voltage)
→ Instance: (Bind (Atom "myapp::Voltage") (Atom <data>))
→ Constructor: (:myapp::Voltage <data>)
→ Predicate:   (:myapp::is-Voltage? x)
```

Single-data wrapper. Newtype shape. No fields. No methods.

## What 227.2 explores (multi-field + methods)

Patterns to study from defservice (arc 209 DESIGN.md + SCORE-SLICE-1.md). **Note: defservice itself NOT YET SHIPPED — Stone A (spawn-program defmacro) is BRIEF'd; Stones B/C/D pending. The patterns below are from the DESIGN, not battle-tested implementation. Treat as "good form" reference, not "proven" precedent.**

### Pattern 1 — Multi-definition expansion via `do`-splice

Per SCORE-SLICE-1.md Audit 1:
> A macro can expand to `(:wat::core::do enum1 enum2 struct1 defn1 defn2 ...)`. `register_types` recurses into top-level `(:wat::core::do ...)` bodies via `splice_type_decls_user` (`src/types.rs:1450-1481`). `register_defines` recurses via `preregister_fn_defs_in_do` (`src/runtime.rs:1741`).

For multi-field defclass:

```
(:wat::holon::defclass :myapp::Voltage
  :fields {magnitude :wat::core::Float
           unit      :wat::core::String})
```

Expands to:

```
(:wat::core::do
  (:wat::core::defn :myapp::Voltage [magnitude unit]
    (:wat::holon::Bind
      (:wat::holon::Atom "myapp::Voltage")
      (:wat::holon::Bundle
        (:wat::holon::Bind (:wat::holon::Atom "magnitude") (:wat::holon::Atom magnitude))
        (:wat::holon::Bind (:wat::holon::Atom "unit")      (:wat::holon::Atom unit)))))
  (:wat::core::defn :myapp::Voltage/magnitude [v]
    (...extract magnitude field via Bundle traversal...))
  (:wat::core::defn :myapp::Voltage/unit [v]
    (...extract unit field...))
  (:wat::core::defn :myapp::is-Voltage? [v]
    (:wat::holon::is? v "myapp::Voltage")))
```

Field access auto-generated. Constructor takes positional or named args.

### Pattern 2 — Computed unquote for namespaced method names

Per `src/macros.rs:1069-1097` (arc 143) + `keyword/of` arc 170 Gap A:

```
~(keyword/of :myapp ::Voltage / ::magnitude)  →  :myapp::Voltage/magnitude
```

Lets defclass generate `:myapp::Voltage/<field>` accessors at expand time without string manipulation tricks. The keyword IS the namespace per `feedback_fqdn_is_the_namespace`.

### Pattern 3 — `state-as-self` for methods (defservice's contract)

Per arc 209 DESIGN § "Handler signature":

> Handlers take state as the FIRST argument. The dispatch loop threads state through each handler call. State is implicit-self via positional convention, not via implicit lookup.

For defclass methods:

```
(:wat::holon::defclass :myapp::Voltage
  :fields {magnitude :Float}
  :methods {double  [self]       (:myapp::Voltage (* 2.0 (:myapp::Voltage/magnitude self)))
            scale   [self by]    (:myapp::Voltage (* by (:myapp::Voltage/magnitude self)))
            magnitude+ [self n]  (:myapp::Voltage (+ n (:myapp::Voltage/magnitude self)))})
```

First param = `self` (the instance). Methods expand to:

```
(:wat::core::defn :myapp::Voltage/double [self]
  (:myapp::Voltage (* 2.0 (:myapp::Voltage/magnitude self))))

(:wat::core::defn :myapp::Voltage/scale [self by]
  (:myapp::Voltage (* by (:myapp::Voltage/magnitude self))))
```

**No state-hiding.** Methods are functions; instance is the first arg. Clojure tradition. Per `project_typed_entities_doctrine`: "OO without class hierarchy. Method dispatch = route by similarity between instance's class + method-registered class atoms" — Stone 227.3+ can integrate with arc 226's polymorphic predicate machinery for multimethod dispatch.

### Pattern 4 — Capability-grouped declarations (defservice's :admin / :user split)

defservice splits handlers by capability tier:

```
:admin {Provision [...] -> User
        Deprovision [...] -> nil
        Stop [] -> nil}
:user  {Get [] -> i64
        Increment [n :i64] -> i64}
```

For defclass: could group `:fields` (data) + `:methods` (operations) + `:invariants` (predicates checked on construction) + `:protocols` (shared method declarations). Each group generates its own type of substrate artifact.

### Pattern 5 — Type signature propagation

defservice carries type signatures through to the generated wrappers. For defclass:

```
:fields {magnitude :wat::core::Float
         unit :wat::core::String}
:methods {double [self -> :myapp::Voltage]
          scale [self by :wat::core::Float -> :myapp::Voltage]}
```

Signatures auto-thread to the generated defn arg-lists + return-type annotations. The check-layer reasons about user types via the predicate + signature combo.

## Open questions for Stone 227.2 BRIEF (when authored)

1. **Field map shape** — `{name type ...}` (Clojure map syntax) OR `[[name type] ...]` (vector of pairs) OR `:fields name type name type ...` (flat sequence)? Pick the most idiomatic with existing wat conventions.

2. **Accessor naming** — `:myapp::Voltage/magnitude` (slash-separated; defservice precedent) OR `:myapp::Voltage::magnitude` (FQDN-deeper)? Per `feedback_fqdn_is_the_namespace` the slash-form is established for method/accessor naming (e.g., `:wat::holon::Bundle/children`).

3. **Constructor positional vs keyword args** — `(Voltage 5.0 "V")` positional OR `(Voltage :magnitude 5.0 :unit "V")` keyword OR support both? Clojure-on-Rust suggests keyword-args natural; positional simpler.

4. **Mutability semantics** — wat is immutable; methods that "change" a field return a NEW instance with the field replaced. This is enforced structurally (immutable HolonAST) but the macro could provide ergonomics like `(Voltage/with-magnitude self 10.0)` auto-generated.

5. **Inheritance** — Stone 227.3 territory; classifier-chain via nested Bind: `(Bind (Atom "U8") (Bind (Atom "Int") (Atom 42)))` produces U8 instance queryable as either U8 (outer) or Int (inner via classifier-chain walk).

6. **Method dispatch via classifier-similarity** — Stone 227.4+ territory; arc 226 polymorphic `is?` extends to multimethod dispatch. `(defmethod some-op :myapp::Voltage [self ...])` registers a per-classifier handler; `(some-op instance ...)` routes by classifier similarity.

7. **Validation predicates** — `:invariants {magnitude (> magnitude 0.0)}` checked on construction. Optional v2+ feature.

## What 227.2 should NOT do (scope discipline)

- Multimethod dispatch (227.4+; needs arc 226 closure)
- Inheritance via classifier-chain (227.3)
- Cross-cutting protocols / interfaces (227.5+)
- Pattern matching on user types (substrate `match` extension; separate arc)
- VSA similarity scoring (different arc — 226.2+)
- Mutability primitives (wat is immutable; out of scope forever)

## Cross-references

- arc 209 DESIGN.md — defservice surface (the precedent shape)
- arc 209 SCORE-SLICE-1.md — proves defmacro infrastructure can do multi-definition expansion + computed unquote + protocol-grouped declarations
- arc 209 BRIEF-STONE-A.md — `:wat::kernel::spawn-program` defmacro shape (similar pattern)
- `src/macros.rs:1069-1097` — computed unquote (arc 143)
- `src/macros.rs:602-677` — keyword/of (arc 170 Gap A)
- `src/types.rs:1450-1481` — splice_type_decls_user (arc 170 Gap J)
- `src/runtime.rs:1741` — preregister_fn_defs_in_do (arc 170 Gap C)
- `wat/runtime.wat:17-32` — define-alias defmacro precedent
- arc 227 DESIGN.md — original (227.1 = single-data v1; 227.2+ = enrichment)
- [[typed-entities-doctrine]] memory entry
- `feedback_fqdn_is_the_namespace` — namespace doctrine
