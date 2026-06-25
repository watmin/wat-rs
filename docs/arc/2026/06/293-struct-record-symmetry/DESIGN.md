# Arc 293 — the aggregate type system: structural surfaces over a nominal holder

**Status: SCOPED → MODEL SETTLED (2026-06-25).** Began as "struct/record construction symmetry" (surfaced by
arc 291's `/from-map` work); a long co-design grew it into the real thing: **wat's nominal/structural type
system, decomplected.** **291 is BLOCKED on this arc** (builder: *"291 is blocked on this new arc — this is
our priority"*). The arc's name (`struct-record-symmetry`) now undersells it — the true subject is the
aggregate citizen + its structural surface system. (Rename is an intueri cast; dir kept for stability.)

> **PATH NOTE (amend-with-recognition):** an earlier draft of this doc framed the arc as construction-surface
> unification + a record-inheritance (`parent`) machinery kept as a "named post-reg exception," and
> four-questioned a macro-emission fork (A/B/C → C). That four-questions stands (below, § Decisions). But the
> 2026-06-25 co-design **superseded the inheritance framing entirely**: `parent` is annihilated, replaced by
> **structural width-subtyping**. Superseded passages are marked `⊘ SUPERSEDED`; the path is preserved.

## The bug (builder, verbatim)

> *"we cannot operate on structs and records trivially — that's a fucking catastrophic bug — decomplection is
> highest priority."* · *"records are basically just structs with an implication they must be edn-repr."* ·
> *"the explicit `:satisfies`… feels wrong… it's an ambient 'you do or you don't'."* · *"methods are
> accessors."* · *"annihilation is our greatest pleasure."*

Two structurally-identical things (records, structs) that every consumer must special-case *by kind* is a
coherence failure — the abstraction leaks. (Memory: `feedback_uniform_operation_or_decomplect_is_catastrophic`.)

## WHAT THE ARC DELIVERS — the acceptance test (this exact program goes GREEN)

```clojure
;; ── THE SURFACE — a definterface is a set-of-accessor (fields and/or methods, uniformly) ──
(:wat::core::definterface :geo::Shape
  [color <- :wat::core::String]              ; FIELD-style accessor → satisfier exposes  :T/color -> :String
  (area  [self] -> :wat::core::f64)           ; METHOD accessor      → satisfier exposes  :T/area  [self] -> :f64
  (label [self] -> :wat::core::String))       ; METHOD accessor      → satisfier exposes  :T/label [self] -> :String

;; ── OWN TYPE #1 — Circle (core record). :geo::Circle/color is generated FREE by the field. ─
(:wat::core::defrecord :geo::Circle [color <- :wat::core::String  radius <- :wat::core::f64])
(:wat::core::defn :geo::Circle/area [self <- :geo::Circle] -> :wat::core::f64
  (:wat::core::f64::* 3.14159 (:wat::core::f64::* (:geo::Circle/radius self) (:geo::Circle/radius self))))
(:wat::core::defn :geo::Circle/label [self <- :geo::Circle] -> :wat::core::String
  (:wat::core::str "circle(r=" (:geo::Circle/radius self) ")"))
;;  ⇒ Circle exposes color+area+label ⇒ STRUCTURALLY satisfies :geo::Shape. No declaration.

;; ── OWN TYPE #2 — Square. Same surface, different fields. ─────────────────────────────────
(:wat::core::defrecord :geo::Square [color <- :wat::core::String  side <- :wat::core::f64])
(:wat::core::defn :geo::Square/area [self <- :geo::Square] -> :wat::core::f64
  (:wat::core::f64::* (:geo::Square/side self) (:geo::Square/side self)))
(:wat::core::defn :geo::Square/label [self <- :geo::Square] -> :wat::core::String
  (:wat::core::str "square(s=" (:geo::Square/side self) ")"))

;; ── THE MONKEYPATCH — teach a FOREIGN built-in (holon Vector) to be a Shape (you don't own it) ─
(:wat::core::extend-type :wat::holon::Vector :geo::Shape
  (color [self] -> :wat::core::String "grey")
  (area  [self] -> :wat::core::f64 (:wat::core::i64::to-f64 (:wat::core::length self)))
  (label [self] -> :wat::core::String (:wat::core::str "vector[" (:wat::core::length self) "]")))

;; ── POLYMORPHIC CONSUMER — accepts ANY Shape; dispatcher routes :T/<accessor> by runtime type ─
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
  (:wat::core::str (:geo::Shape/color s) " " (:geo::Shape/label s) " area=" (:geo::Shape/area s)))

(:wat::core::defn :geo::demo [] -> :wat::core::String
  (:wat::core::str
    (:geo::describe (:geo::Circle "red" 2.0))                  "  |  "
    (:geo::describe (:geo::Square "blue" 3.0))                 "  |  "
    (:geo::describe (:wat::core::Vector :wat::core::i64 10 20 30))))
;;  ⇒ "red circle(r=2.0) area=12.56636  |  blue square(s=3.0) area=9.0  |  grey vector[3] area=3.0"
```

**The deep point the test makes:** look at `color`. The interface only says *"expose `:T/color -> :String`."*
Circle backs it with a **field** (free accessor); the Vector backs it with a **method**. **Field-vs-method is
the satisfier's private choice — the interface sees only an accessor.** That is "methods are accessors," and it
dissolves the field/method seam end to end.

## The model (settled across the 2026-06-25 co-design)

### One aggregate citizen; the holder is the ONLY essential difference
Records and structs are isomorphic aggregates (named, typed, fixed-field, positional construction, named
accessors). A record *is* a struct + "must be EDN-representable" (the value repr literally nests a `struct_form`).

### HOLDER — **nominal** (the EDN kind-wall; you declare it)
The one thing that is *not* structural, because it's a **capability** the shape can't express:
- **struct** — non-EDN, in-locus, holds resources (sockets/caches); `is_portable_type(Struct) → false` (4b-i). *"i must have a struct — this can't cross the EDN boundary."*
- **core record** — EDN, crosses the wire.
- **holon record** — holographic EDN + VSA ops. *"i must have a holon record — i need VSA."*

The holder roots are existing lattice nodes (`:wat::core::Struct`, `:wat::Record`, `:wat::holon::Record`;
`holon <: core` is a seeded edge — requiring core *accepts* holon, requiring holon accepts only holon). This
is the R8 soul/body line as a type axis.

### SURFACE — **structural** (row-polymorphic; ambient, no declaration)
A surface is **a set of required accessors** `:T/<name>`. Satisfaction is **structural**: a type satisfies iff
it *has* the accessors (width subtyping — extra members are fine; *"the star-ness is good enough even with
rectangle properties"*). **No `:satisfies`, no `:parent`, no registered edge** — fit is *computed* at the use
site. The killer property (builder's argument): *"some user's lib may show up and i satisfy their requirement
but i didn't mark myself usable"* — **retroactive / open-world satisfaction.**

### Methods ARE accessors (the seam dissolves)
A field auto-generates its accessor; a method *is* an accessor (a fn at `:T/name`) you write. A surface lists
required accessors; the satisfier may back each with a field or a method — invisible to the interface and the
consumer. The dispatcher (`:Shape/area s` routes by `s`'s runtime type to `:T/area`) is generated from the
surface's method members, reusing arc 232's `extract-classifier` + `apply`.

**Surface methods are FULL-ARITY and SINGLE-dispatch (deliberate; not a `[self]`-only lock).** A surface method
declares its whole argspec — `(scale [self factor <- :f64] -> :Self)` — and the impl `:T/scale` is a `defn` of
any arity. Dispatch is **single-dispatch on the receiver** (self's type); the other args are plain typed values
passed through. That is what an interface IS (Go interface / Clojure protocol). **Multi-dispatch — pick the impl
by the types of *several* args — is NOT a surface concern; it is `defclause`'s.** `defclause` (arc 237,
`src/form_match.rs`) is wat's canonical multi-arity + clause-by-guard primitive: first-match-wins by per-position
type match across all args, with `:guard`/`:ensure`. We argued *against* baking multi-dispatch into surfaces
precisely because `defclause` already owns it, one-canonical-path. Three orthogonal tools, three questions:
`defsurface` = "has these accessors?" (single-dispatch interface) · `defclause` = "which clause for these arg
types?" (multi-dispatch) · `typeunion` = closed type-level set. Surfaces must NEVER grow multi-arg dispatch.

### `definterface` — a named **argspec** of accessors (subsumes `defprotocol`)
**Four-questioned (below): `definterface` = a named ArgSpec** (reuses the first-class `src/argspec/` home), NOT
a `typealias`-over-a-new-structural-type. It names a `[name <- :type …]` + `(method [self] -> :ret)` set, usable
two ways: a **structural constraint** in type position (`[s <- :geo::Shape]`) and **spliceable** into bodies for
DRY (`[~@:geo::Planar  radius <- :f64]`). It **subsumes `defprotocol`** (a protocol is a method-only surface) —
`defprotocol` is annihilated. **Name CROWNED `defsurface`** (intueri 2026-06-26 + builder): the only candidate
with zero false connotation — the builder's own word ("surface area exposed"). Rejected Level-1 lies:
`definterface` (Java-nominal `implements` baggage), `deftrait` (Rust/Scala `impl Trait for T`), `defspec`
(`clojure.spec` collision); `defshape` mumbles (implies an *exact* outline; a surface is a *minimum*). The
concept + internal variant = **`Surface`** / `TypeExpr::Surface` (user-vocab == compiler-vocab; `Row` rejected
as type-theory-obscure). `extend-type` is KEPT (not renamed), demoted to the foreign-type accessor adapter.

### `extend-type` — the typed, compile-checked foreign-accessor adapter (survives, demoted)
Its one real job: add accessors to a type you don't own (the monkeypatch — `:wat::holon::Vector`). Solves the
**Expression Problem**, safely: collisions are `DuplicateDefine` *compile errors*, bodies are type-checked
against the surface, no runtime class mutation (cf. Ruby/JS global monkeypatch chaos — *"insane in Ruby, sane
by types"*). The newtype-on-top discipline stays the default; the patch is for genuine foreign adapters.

### A parameter = **holder ∩ surface** (intersection)
`[s <- [:wat::holon::Record  :geo::Shape]]` = "a holon record that structurally has the Shape surface."
Holder-agnostic = name only the surface. A list in type position = intersection (satisfy ALL). Restriction
beyond a surface is via **nominal field types** (newtypes/decorated scalars) — *"we have decorated ints to choke
down incorrect passing."*

### What is ANNIHILATED vs what stays NOMINAL
- **Annihilated:** `:parent` / record inheritance · `register_record_methods`' parent-flattening · `register_struct_methods` · surface subtype-*edges* · `defprotocol`. ⊘ **And the phase-order problem evaporates** — structural fit is checked where `assignable` already runs (post-registration), so there is nothing to query at expand time (no reflection intrinsic to mint, no `expand_all`-precedes-`register_types` invariant to break — see `freeze.rs:2202`; the earlier "mint a `type-fields-of` intrinsic / break the phase order" exploration is ⊘ SUPERSEDED).
- **Stays nominal:** the holder/kind lattice (`holon <: core`, `Value` as top — arc 278, `derive` edges like `Thread' <: Peer'`) · nominal tagged sums (`defenum`, no anonymous unions — **arc 258 holds for sums**) · the newtype restriction escape-hatch.

## Where this lands — the greats (feeds R1)
A confluence derived by solving (*"why do we even need parent?"*), not by reading:
- **Row polymorphism** (Wand/Rémy/Cardelli; OCaml's object system) — "a type with *at least* these labeled members, extras allowed, matched by shape." The formal floor under "set-of-accessor + structural + width subtyping."
- **Go & OCaml** — structural interfaces, implicit satisfaction.
- **Haskell type classes / Clojure protocols** — retroactive open extension; the holon-Vector monkeypatch IS the **Expression Problem** (Wadler), solved.
- **Smalltalk / Alan Kay** — "an object is the messages it responds to," dispatch by receiver. (The arc lands on Kay *again* — same as 291 R1's messaging-OOP, now from the type-system side.)
- **Genuinely ours:** the **holder (nominal EDN-capability tag) × structural row-polymorphic surface** fusion — structural-open surfaces *on top of* a categorical, un-leakable kind wall. No prior language welds those.

## The decisions, four-questioned (on the record)

1. **Structural over nominal satisfaction (drop `:satisfies`).** Obvious (fits by shape) · Simple (one rule, no declaration machinery) · Honest (you ARE usable if you fit — no false barriers) · UX (open-world, never closes doors). Cost = accidental satisfaction, accepted (mitigated by nominal field types).
2. **`definterface` = named argspec** over `typealias`-of-a-new-structural-type. Wins Obvious + Simple + UX (reuses `src/argspec/`; one form = constraint AND spliceable field reuse; typealias can't hold a `[fields]` list today, grounded `types.rs` `parse_typealias` → `parse_type_node`).
3. **Methods are accessors** (`definterface` subsumes `defprotocol`); structural for both; `extend-type` = foreign-accessor adapter.
4. **⊘ Construction emission A/B/C → (C) full annihilation** (PATH-PRESERVED; now *easier* — with inheritance gone there's no parent-flattening to reproduce): `defrecord`/`defstruct`/`holon defrecord` peer macros emit the full surface as wat; `register_*_methods` annihilated; built-in structs migrate to wat `defstruct`. (A disqualified — re-introduced the very split the arc kills; the four-questions overturned the first glib pick.)

## Grounded current state (the machinery, mapped 2026-06-25)
- **Lattice** (KEEP, holder/kind only): `register_subtype`/`is_subtype` (`types.rs:450/3142`), `assignable` (`check.rs:14184`), `Value`-top (arc 278), 3 edge-producers `parent`/`derive`/`extend-type` (`types.rs:416/1573/1605`). Surface-satisfaction moves OFF edges → structural.
- **Construction** (UNIFY + annihilate the Rust gen): `register_struct_methods`/`register_record_methods` (`runtime.rs:895/1254`), `eval_struct_new/field`/`eval_record_of/holon/field_at` (`runtime.rs:11628–13310`), `parse_recordtype`/`parse_defstruct` (`types.rs:2029` / `types/defstruct.rs`).
- **Surface** (NEW): a structural-product matching path in `assignable` (does the candidate's type have ⊇ the required accessors), keyed off a named ArgSpec (`src/argspec/`) or inline `[fields]`.
- **Protocols** (`defprotocol`→annihilate / subsume; `extend-type`→keep+demote): arc 232 `extract-classifier`+`apply` dispatcher machinery reused under `definterface`.
- **Reprs** (UNTOUCHED — the wire law stays variant-level): `Value::Struct` / `Value::wat__Record` (`value/value.rs`).
- **Home:** mint **`src/aggregate/`** (CROWNED — intueri 2026-06-26 + builder; `src/product/` was the
  type-theory alt, not chosen) — lift the construction + surface machinery out of `runtime.rs`(32k)/`check.rs`(21k)/`types.rs` (the *"move out of `src/*.rs`"* directive); reprs stay in `value/`.

## Decomposition (sub-strikes → the demo as final GREEN gate)
- **293.0 — the acceptance probe.** The demo program above as a RED test (RED at HEAD: `definterface`/structural-surface/`/from-map`/the monkeypatch don't exist). Commit before build.
- **293.1 — the aggregate HOME.** Mint `src/<aggregate>/`; lift the construction + surface machinery (re-export at old paths, the 251.2/wat-reader pattern). Pure relocation, SET-diff ∅.
- **293.2 — construction symmetry + `defstruct`/`defrecord` peer macros + `/from-map`.** The shared emission layer; `Record::def → :wat::core::defrecord`, `holon::Record::def → :wat::holon::defrecord` (fix-wat the `.wat`, scripted-keyword-substitute the `.rs`, retirement-table the old heads); `register_*_methods` annihilated; built-in structs → wat `defstruct`. `/from-map` falls out of the shared layer.
- **293.3 — structural surfaces + `definterface` + holder∩surface params.** The named-argspec `definterface`; `assignable` gains structural-product matching; `[holder surface]` intersection in param position; `:parent` + surface-edges annihilated (structural width-subtyping replaces them).
- **293.4 — methods-are-accessors + `definterface` subsumes `defprotocol` + `extend-type` demotion.** Method members in `definterface`; the generated dispatcher; `extend-type` as the foreign-accessor adapter; `defprotocol` annihilated. **The demo (293.0) goes GREEN — the arc's gate.**
- **293.5 — close + amend.** Full workspace SET-diff ∅; home warded; amend 291's `CURRENT-STATE` to unblock `/from-map` → resume 291.

## Blast radius + migration
`Record::def` in 85 files (10 `.wat`, 75 `.rs`); `defstruct` in 60 (12 `.wat`, 48 `.rs`); 9 holon sites; 15
built-in structs. `.wat` → **fix-wat** (`feedback_lean_on_wat_migration_toolkit`); `.rs` wat-in-string fixtures →
unambiguous fully-qualified-keyword substitution (`wat-fixes-rust` is still the deferred NOTE); retirement-table
the old heads so nothing drifts silently.

## Out of scope (affirmative cuts)
- **Unifying the `Value` reprs** (one repr + a portability bit) — reopens serialization + the wire gate; the variant-level distinction IS the wire law. Keep.
- **The dotted clojure-symbolic surface** (`wat.core/defrecord` ↔ `:wat::core::defrecord`) — arc 251's axis; pick 251-ready names, don't switch notation here.
- **Anonymous structural *unions*** — NO. Sums stay nominal (`defenum`, arc 258). Structural is for product *surfaces* only.

## Pairs
`291/STRIKE-4b-struct-state.md` (the wire law) · `291/CURRENT-STATE.md` (the blocked `/from-map`) ·
`232-defprotocol-extend-type/DESIGN.md` (the dispatcher machinery `extend-type` reuses) ·
`237-polymorphism-consolidation/DESIGN.md` (`conforms?` / one-canonical-path) · `258-instinctive-conditionals`
(ADT/nominal-sums — held) · `feedback_uniform_operation_or_decomplect_is_catastrophic` ·
`feedback_kwargs_is_always_a_macro` · `feedback_substrate_forces_idealized_state` · `src/argspec/` (definterface's home).
