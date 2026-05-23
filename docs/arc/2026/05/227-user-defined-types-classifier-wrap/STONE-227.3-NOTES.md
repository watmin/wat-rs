# Notes for Stone 227.3 — inheritance via classifier-chain + is-a? predicate

**Status:** Future-stone scoping notes. Authored 2026-05-22 night during Stone 227.2 v2 sonnet flight, after user articulated the lineage-as-List intuition + presence? predicate composition.

## What 227.2 v2 ships (multi-field defrecord with field-list mandate)

```
(:wat::holon::defrecord :ns::Voltage [magnitude <- :f64, unit <- :String])
;; → constructor + accessors + predicate
;; → instance: Bind(Atom("ns::Voltage"), Bundle(Bind(Atom("magnitude"), Atom(val)), ...))
```

NO inheritance support yet. Each defrecord is independent.

## What 227.3 explores (inheritance via classifier-chain)

User direction 2026-05-22 night (during Stone 227.2 v2 sonnet flight):

> *"when you mention inheritance .... is this a holon List or holon Vector?.... i think inheritence is a list?... its strongly sequential?.."*
>
> *"this means we can in holon ops do a 'is-a?' check that's fuzzy - if you're in the lineage chain or not?.... we can use presence? predicate for this?..."*

Both observations are right. Inheritance = sequential parent-chain = List shape. `is-a?` = membership in the lineage chain = `presence?`-style composition.

## Inheritance encoding — classifier-chain via nested Bind

```
;; Stone 227.3 sketched form:
(:wat::holon::defrecord :ns::U8
  :extends :wat::core::Int
  :fields  [value <- :wat::core::i64])

;; OR via positional extends-list:
(:wat::holon::defrecord :ns::U8
  :extends [:wat::core::Int]
  :fields  [value <- :wat::core::i64])
```

**Instance encoding (extends classifier-wrap composition):**

```
Bind(Atom("ns::U8"),                       ; outermost: most-derived classifier
  Bind(Atom("wat::core::Int"),             ; one level up: parent classifier
    Bundle(                                ; innermost: actual data
      Bind(Atom("value"), Atom(42)))))
```

The lineage chain `[ns::U8, wat::core::Int]` is encoded as nested Bind structure. Each level adds one classifier atom; the innermost element is the data Bundle.

**Multi-parent (deferred to 227.3+):** if multiple inheritance is wanted later, the chain becomes a tree/DAG; encoding could be `Bundle` of `Bind(Atom("Parent"), Bind(...))` at the appropriate level. Single inheritance v1 first.

## Why List (not Vector) for the lineage

| Property | List fit | Vector fit |
|---|---|---|
| Sequential traversal (walk parent-up) | YES | YES |
| Mutation = prepend (subclass prepends to chain) | YES (cons is O(1)) | NO (would shift) |
| Random access by index | no use case | overkill |
| EDN-honest sequence type for ordered relationships | LIST `'(...)` | (indexed; different intent) |
| Substrate primitive | `:wat::core::List<T>` (arc 220 Stone 220.4; LinkedList-backed) | `:wat::core::Vector` |

LinkedList semantics match the access pattern. arc 220 Stone 220.4 minted `:wat::core::List<T>` as LinkedList-backed — that's the substrate primitive for the extracted lineage chain.

## The is-a? predicate family — three variants

Substrate primitives verified (grep'd 2026-05-22):
- `:wat::holon::presence?` exists (`src/check.rs:14261`, `src/runtime.rs:5008`) — VSA-algebra membership
- `:wat::core::List/contains?` exists (`src/runtime.rs:4853`) — structural exact match
- `:wat::core::Vector/contains?` + `:wat::core::HashSet/contains?` also exist
- arc 228 `extract_classifier` extracts the outermost classifier atom

**Three predicates serve three different questions:**

| Predicate | Semantics | Implementation | When to use |
|---|---|---|---|
| `(:wat::holon::is? x "ns::U8")` | Exact classifier at OUTERMOST Bind | arc 226 — `extract_classifier == Some("ns::U8")` | "Is x DECLARED as exactly U8?" |
| `(:wat::holon::is-a? x :wat::core::Int)` | Present anywhere in lineage chain | walk nested Binds OR `:wat::core::List/contains?` on extracted chain | Classical OO is-a; "Does x satisfy the Int contract structurally?" |
| `(:wat::holon::behaves-like? x :SomeClass)` | VSA-similarity to class prototype | `:wat::holon::presence?` (arc 226 Stone 226.3+ enhancement) | Fuzzy duck-typing; behavioral subtyping; future |

**is-a? is the structural inheritance check. presence? is the substrate primitive that ENABLES behaves-like? (future fuzzy version).**

## Composition with existing substrate

```
;; Lineage extraction helper (Stone 227.3 substrate addition):
(:wat::holon::lineage-chain instance) -> :wat::core::List<:wat::core::string>
;; Walks the nested Bind structure; collects classifier atoms in order;
;; returns List<String> from most-derived to root-parent

;; is-a? via composition:
(:wat::core::defn :wat::holon::is-a?
  [instance <- :wat::holon::HolonAST
   target   <- :wat::core::keyword]
  -> :wat::core::bool
  (:wat::core::List/contains?
    (:wat::holon::lineage-chain instance)
    (:wat::core::keyword/to-string target)))
```

Pure composition. No new substrate primitives beyond `lineage-chain` (which might itself be expressible via `extract_classifier` + recursive Bind walk).

## How extends parameter composes with field-list

Stone 227.2 v2 mandates field-list. Stone 227.3 adds `:extends` section while preserving the mandate:

```
;; Three forms in v3 (Stone 227.3):

;; No fields, no inheritance:
(:wat::holon::defrecord :ns::Tag [])

;; Fields, no inheritance:
(:wat::holon::defrecord :ns::Voltage [magnitude <- :f64])

;; Fields + single inheritance:
(:wat::holon::defrecord :ns::U8 :extends :wat::core::Int [value <- :i64])
;; OR with explicit fields section keyword:
(:wat::holon::defrecord :ns::U8
  :extends :wat::core::Int
  :fields  [value <- :i64])
```

**Open design question for Stone 227.3 BRIEF:** does `:extends` make `:fields` keyword mandatory? Or stay positional? Per `feedback_wat_llm_first_design` — one canonical path. If `:extends` is present, `:fields` becomes mandatory keyword. If `:extends` is absent, field-list stays positional. (Or: keyword-args ALWAYS once extends is supported. Pick the more LLM-honest path during BRIEF authorship.)

## Auto-generated artifacts (Stone 227.3 scope)

Beyond Stone 227.2's constructor + N accessors + predicate, inheritance adds:

- **Inherited accessors** — `:ns::U8/<parent-field>` if parent's fields are accessible from child (open design question: are parent fields inherited as accessors, or accessed via parent classifier?)
- **`is-a?` doesn't need per-type generation** — it's a single polymorphic verb that works for any classifier (provided by substrate or stdlib)
- **Constructor inheritance** — does `(:ns::U8 v)` need to call `(:wat::core::Int v)` first to construct the parent layer? Or does the macro generate the full Bind-chain directly? (Probably the latter — direct generation; no parent-constructor invocation needed.)

## Open questions for Stone 227.3 BRIEF (when authored)

1. **Single vs multiple inheritance** — v1 is single inheritance only (chain is List, not Tree). Multiple inheritance is Stone 227.3+ if needed; encoding gets complex (Bundle of chains? DAG flatten?).
2. **`:extends` keyword vs positional** — `(defrecord :T :extends :Parent [fields])` vs `(defrecord :T [parent <- :Parent fields])` (treating parent as a special field). Pick most LLM-honest path.
3. **Accessor inheritance** — does the child get the parent's accessors auto-generated in its own namespace? Or does the user navigate `(:Parent/field (:wat::holon::parent-of child-instance))`?
4. **Constructor signature with inheritance** — N child fields + M parent fields = N+M-arg constructor? Or child takes (parent-instance + child-fields)? Or child constructs parent + child separately and macro wires them?
5. **`is-a?` placement** — substrate primitive `:wat::holon::is-a?` OR auto-generated `:ns::is-U8-a?` per type? (Probably substrate primitive — polymorphic across all types.)
6. **`lineage-chain` substrate primitive** — does it need to be a new substrate Rust fn, or can it be a pure wat defn using `extract_classifier` recursively?
7. **Method dispatch through lineage** — when arc 232 (defprotocol) lands, can a protocol method bound to `:Parent` also dispatch on `:Child` instances via lineage walk? (Yes; this is the open-extension benefit.)

## Composition with arc 232 (defprotocol)

Stone 227.3's inheritance + arc 232's defprotocol compose beautifully:

```
;; Declare a protocol on the parent:
(:wat::holon::defprotocol :ns::Printable
  (print-it [self] -> :wat::core::string))

;; Extend the parent:
(:wat::holon::extend-type :wat::core::Int :ns::Printable
  (print-it [self] (:wat::core::Int/to-string self)))

;; Child inherits the implementation via lineage walk:
(:wat::holon::defrecord :ns::U8 :extends :wat::core::Int [value <- :i64])
(:ns::Printable/print-it (:ns::U8 42))
;; → dispatcher walks lineage: tries "ns::U8/Printable-print-it" (missing),
;;   falls back to "wat::core::Int/Printable-print-it" (present); returns "42"
```

The dispatcher uses `lineage-chain` + iteration to find the first matching impl. Open-extension of polymorphism through inheritance — exactly what Clojure protocols + Java interfaces give, hosted via wat's typed-entities doctrine.

This is the convergence point. Stone 227.3 (inheritance) + arc 232 (defprotocol) = full Clojure/Java OO surface, hosted on the 12-primitive substrate.

## What 227.3 should NOT do (scope discipline)

- Multiple inheritance (227.3+ if needed; single first)
- Mixin / trait semantics distinct from inheritance (different abstraction; future)
- Method override via child-namespace shadowing parent (open question; design carefully)
- `super` calls (parent-method invocation from child) — defer
- Reflective lineage introspection at runtime (just structural walk; full reflection is arc 201's territory)
- Cross-cutting protocol composition (arc 232; different stone)
- Inheritance for defservice (different abstraction; defservice's protocol model handles its own composition)

## Cross-references

- `project_typed_entities_doctrine` — the substrate foundation
- `project_defrecord_defservice_doctrine` — defrecord IS for immutable data; inheritance fits naturally
- arc 226 SCORE — `:wat::holon::is?` (the structural-classifier predicate this builds on)
- arc 226 NOTES (future 226.3+) — VSA similarity continuous variant (the future `behaves-like?` enabler)
- arc 220 Stone 220.4 — `:wat::core::List<T>` LinkedList (the substrate primitive for extracted lineage)
- arc 232 DESIGN.md — defprotocol + extend-type (composes with inheritance via lineage-walking dispatch)
- `feedback_wat_llm_first_design` — one canonical path per task; informs `:extends` keyword decision
- `feedback_fqdn_is_the_namespace` — lineage classifier strings are FQDN

## When to OPEN arc 227 Stone 227.3

Stone 227.3 is OPTIONAL within arc 227. Triggers:
- Stone 227.2 v2 ships cleanly (so the multi-field foundation is locked)
- A real use case needs typed inheritance (e.g., `:wat::core::U8 :wat::core::U16 :wat::core::U32` all extending `:wat::core::Int`; or domain types like `:trading::LimitOrder :extends :trading::Order`)
- Arc 232 (defprotocol) lands — at that point, lineage-walking dispatch becomes valuable

Until a real cross-cutting inheritance need surfaces, Stone 227.3 stays in design. defrecord without inheritance covers the common case (each type independent + queryable via `is?`).
