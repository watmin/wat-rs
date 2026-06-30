# The Aggregate Model & User Forms — the canonical contract (2026-06-29)

> **This is the contract every remaining 293/294 strike is held to.** Settled across the long co-design of
> 2026-06-28/29. The governing law: an aggregate is ONE citizen; the **holder enum is its only specialness**;
> everything else is holder-blind. Pairs `AGGREGATE-AUDIT.md` (the parity ledger) + `CLOSE-SEQUENCE-293-294.md`
> (the live order).

## The 7 governing principles

1. **ONE citizen — the aggregate** (`class` + positional `fields`). `struct` / `record` / `holon-record` are the
   *same thing wearing a holder*, never three kinds.
2. **The HOLDER ENUM is the only specialness.** `{ Struct, Record, HolonRecord }`. It is consulted at **exactly three
   boundaries** and nowhere else: **comms** (can it leave the locus?), **edn-repr** (the wire form), **assignability**
   (the `:holder` bound on a surface, contravariant). Construction, field access, identity, the data — all holder-blind.
3. **Every OPERATION is holder-blind + uniform** — ONE primitive each, defined once, used by all holders: construct
   (`aggregate-new` ✓), field-read (one `aggregate-field`), update (one `aggregate assoc`), `aggregate->map`,
   `aggregate->form`. The operations get an **aggregate** home; they do NOT get a per-holder namespace
   (`:wat::core::Record/field-at` will never exist — `struct-field` + `Record/field-at` die into ONE).
4. **NO inheritance.** A type is `holder + own fields`, flat. There is **no `:parent`** — the only thing in a parent
   slot is a holder-root (= the holder). Reuse-of-shape is **surface-splice** `[~@:SomeSurface  own <- :T]` (uniform
   across all three holders), never a nominal base. `program::Env` matures into a **surface** ("must be a record with
   these minimal properties"). defservice `:durable-parent` is **not** inheritance — it is the holder selector
   (`:wat::Record` / `:holon`), already correct.
5. **Requirements are SURFACES — a bare holder is illegal.** A parameter over an aggregate MUST be: a **concrete
   type** (exact), a **surface** (the accessors you read), or **`:wat::core::Value`** (the absolute top, for generic
   guts). A bare holder root in a `[x <- …]` slot is an **Any** — its base state has nothing, so you can do nothing
   with it; it is FORBIDDEN. (`[r <- :wat::core::Record]` does not exist; write `[r <- :SomeSurface]`.)
6. **The holder bound is contravariant.** As a *requirement*: `struct` accepts struct+record+holon (widest — "I just
   need the fields"); `record` accepts record+holon; `holon` accepts holon only (narrowest). Choosing `:holder
   :struct` on a surface is the widest receiver but **forfeits the edn-portability guarantee** on what you got.
7. **The edn wall lives at the LOCUS BOUNDARY, never as a param.** `Value` = absolute top (in-locus; admits structs,
   sockets, resources). **`EdnValue`** = the edn-repr top (`holder != Struct`) — a **wire-boundary** type (294.d:
   what `send'`/`recv'`/the plain-EDN wire speak), NOT a constraint baked into engines. The **locus decides**: a
   thread-local engine takes `Value` (anything); a process/remote engine forces edn at the **comms wall** (`send'`
   rejects a non-portable value), not in its signature. (Confirmed live: rete uses facts opaquely — as map keys,
   stored, reflected via `type` — never by named field; `[fact <- :wat::Record]` → `[fact <- :wat::core::Value]`.)

## The holder trit
`Struct (−1)` in-locus, non-portable, holds resources, never crosses · `Record (0)` edn-repr, crosses ·
`HolonRecord (+1)` edn-repr + VSA. `is_portable = holder != Struct` (`types.rs:138`). `:wat::core::Value` is the
universal top (arc 278).

## The user forms (the UX)

```clojure
;; ── DECLARATION — holder-keyed; flat; DRY by surface-splice (NOT inheritance) ──
(:wat::core::defstruct  :geo::Pt     [x <- :i64  y <- :i64])
(:wat::core::defrecord  :geo::Circle [color <- :String  radius <- :f64])
(:wat::holon::defrecord :geo::HC     [color <- :String  radius <- :f64])
(:wat::core::defrecord  :app::MyEnv  [~@:app::EnvLike  debug? <- :bool])   ; splice a surface's fields, flat

;; ── OPERATIONS — uniform, holder-blind, one form each ──
(:geo::Circle "red" 2.0)              ; construct — bare type name (all holders)
(:geo::Circle/color c)               ; field read — per-type accessor (all holders)
(:wat::core::assoc c :color "blue")  ; functional update — ONE form (all holders)
(:wat::core::aggregate->map c)       ; → map — ONE form (all holders)

;; ── SURFACE — the requirement interface (fields and/or methods + an optional :holder bound) ──
;; The :holder value is a HOLDER-ROOT KEYWORD (the enum's canonical name), NOT a magic symbol.
;; ONE vocabulary everywhere: declaration parent, surface :holder, defservice durable selector.
(:wat::core::defsurface :app::EnvLike
  :holder :wat::core::Record                 ; the enum value, = the holder-root keyword
  [host <- :String  port <- :i64])

;; ── FUNCTION SIGNATURES — a surface / a concrete type / Value; NEVER a bare holder ──
(:wat::core::defn :app::serve [e <- :app::EnvLike]      …)   ; any value satisfying the surface
(:wat::core::defn :app::exact [c <- :geo::Circle]       …)   ; exactly a Circle
(:wat::rete::assert            [fact <- :wat::core::Value] …)  ; the guts: accept anything, reflect
```

## Where hosted
`:wat::core::` — `defstruct` `defrecord` `defsurface` `defn` `assoc` `aggregate->map` `aggregate->form`; the holder
roots `:wat::core::Struct` / `:wat::core::Record` (renamed from `:wat::Record`). `:wat::holon::` — `defrecord`
(holon); holder root `:wat::holon::Record`. Substrate primitives (`aggregate-new`, the one field-read) live in an
**aggregate** home, never user-visible.

## What the model ANNIHILATES (reshape: from "support" to "destroy")
- nominal inheritance · `:parent` · `collect_all_record_fields` · inherited-field storage + abs-idx · the record
  subtype edges · (so **decl-b.1.0** and its probe are DELETED — `aggregate-new`'s own-only arity becomes correct).
- `struct-field` vs `Record/field-at` → ONE `aggregate-field` (GAP-1).
- `Record/assoc`-only → ONE `aggregate assoc` (GAP-2); `struct->form` vs `record->map` → uniform (GAP-3/4).
- bare-holder params → surfaces or `Value` (the rete/program `[x <- :wat::Record]` migration).
- `struct-new` / `Record::of` / `holon::Record::of` → `aggregate-new` (c.2a ✓; of-funcs die in c.2b).
- **the split holder vocabulary** (declaration uses root types · surface `:holder` uses magic `:struct`/`:record`/
  `:holon-record` · defservice uses `:holon`) → ONE vocabulary: the three holder-root keywords. The holder↔keyword
  mapping is hand-written in **5 sites** (`surface.rs:323`, `types.rs:2126` `root_holder_of`, `value.rs:1120`,
  `runtime.rs:6705`, `observe.rs:326` — two emit the stale `wat::Record`) → the **enum owns it**:
  `Holder::root_keyword()` + `Holder::from_root_keyword()`, every site calls these, the 5 hand-matches die, and the
  `:wat::Record` → `:wat::core::Record` rename falls out of the same change.

---

# THE COMPLETE KIT — surfaces, projection, extension (2026-06-29 co-design)

> **The landmark UX forms. Settled in one long four-questions co-design (2026-06-29). Thesis the builder named at
> close: *"we burned inheritance to the ground and lost nothing."* Inheritance · `defprotocol` · the extend-type
> confusion all collapse into FOUR tools, no loss. These forms are the canonical exemplars — when in doubt, match them.**

## A surface is a PURE CONSTRAINT — two feature kinds, either set may be empty
The surface declares constraints; **users satisfy them**. It carries **no impls**. The `:features` syntax itself
separates the two axes:
- **`name <- :Type`** — an **ATTRIBUTE** (data; a `<Keyword, EdnRepr>` cell). The record/data contract. *This* is
  what a record holds.
- **`(name [self …args] -> ret)`** — a **METHOD** (behavior; a function over the aggregate — the aggregate in
  position 0 plus other typed args, computing a typed value). The protocol contract. **This is `defprotocol`,
  subsumed.** Multi-arg, full-arity, single-dispatch on `self`.

attributes-only surface = a data contract · methods-only = the old `defprotocol` · both = a rich contract.
**A record can NEVER hold a function** (least of all a 2-arg one) — so behavior is never data, never frozen.

## The capability lattice — FOUR edges between holders, now COMPLETE
| edge | mechanism | how |
|---|---|---|
| **DOWN** the ladder (holon→core→struct) | assignability | **implicit, free** (a holon has everything a core has) |
| **UP** the ladder (struct→core→holon) | **`to-record`** | **explicit** — the ladder forbids implicit up; you BUILD a new value at the higher tier |
| **FOREIGN → surface** | **`extend-type`** | **explicit** adapter — teach a type you don't own |
| **OPAQUE carry** | **`Value`** | move-only — receive anything, use nothing, only move |

## `to-record` — the data projection (the only honest up-cast)
`(:wat::core::to-record x :S)` → a **core**-record · `(:wat::holon::to-record x :S)` → a **holon**-record (hologram
derived from the projected attributes). It is **the DATA face**: it carries the surface's ATTRIBUTES only; methods
are behavior, never carried. *"i get back an aggregate with an attribute set populated and whatever limits imposed
on the kind of aggregate."*
- **Return type = the surface's macro-emitted backing record `:S$record`** — a real, registered, instantiable
  `AggregateDef` (fields = S's attributes; holder = the target tier). **NOT a second authored copy** — the macro
  derives it from the ONE `:features` spec, exactly as `defrecord` derives its `:T/field` accessors; it cannot drift
  because there is no second hand. (The apparatus was corrected on this — derivation ≠ duplication.)
- Precondition: `x` must satisfy S's attributes (checked → `(to-record x :S)` type-checks iff x has S's data).
- **Lossy by exactly the surface** (it names what survives). Output tier **≥ S's `:holder` floor** (so the result
  satisfies S). You never project UP to a struct (there is no `wat.struct/to-record`).
- Use: get a STRUCT's data across the wire (struct never crosses → project to a portable record); or LIFT any
  aggregate into VSA (the surface chooses which fields form the holographic structure).

## `extend-type` (the REAL form) + `extend-surface` (the macro)
- **`(extend-type T S (m [self :- T  x :- …] -> … body) …)`** — bind a type T's method impls for surface S. **The
  ONE canonical `ArgSpec`.** Same `:…/method` registration key as ambient `(defn :T/method …)` — extend-type and a
  plain defn are two front-doors to one mechanism. **Un-demoted** from "foreign-only adapter" to the **general
  per-type satisfaction door** (your own types OR foreign). The impl is *a function that exists and is called* —
  never frozen into data.
- **`(extend-surface S (m [self x] body) …)`** — a wat **`defmacro`** (purely macro territory; no new core form, no
  new Rust `ArgSpec`). It expands to `(extend-type S$record S …)`, **filling the method types from S's declaration**
  → the user writes **body only**. Default impls over the surface's OWN attributes (read via the surface accessor
  `(:S/attr self)`, never a spelled `$record`). **NOT a second argspec** — the typeless `[self x]` is macro INPUT
  that elaborates into the one canonical `ArgSpec`; nothing new is stored, so the 7-times-ArgSpec-heresy cannot recur.
- A `to-record`'d `$record` is a satisfier of S → it **inherits the `extend-surface` default for free** (data from
  `to-record`, behavior from `extend-surface`, each written once).

## `self` is a normal typed binder — never special (2026-06-29)
A surface method's `self` is **just `:TheSurface`**, written like any other binder: `(add [self <- :acc::Adder  x <-
:wat::core::i64] -> …)`. There is **no auto-fill, no special first position, no off-by-one** — an argspec is a flat list
of typed binders, and position 0 is not a case. (`self`'s type is the **surface** — "any satisfier" — not the
`$record`; a concrete `defn`/`extend-type` impl re-types `self` to its concrete target.) This makes the
**293.4e-pre.i "self double-counted" miscount structurally unrepresentable** — there is no special path to mis-handle.
`extend-surface` is then the *one* deliberately-relaxed form: it fills **every** binder (self **and** args) from the
surface — sugar over a uniform argspec, not a carve-out for `self`. **Migration:** today's `[self]` (untyped, special)
becomes `[self <- :TheSurface]` across existing surfaces — pairs with K0's holder migration. **K0 includes a
self-reference cycle-guard** (a standard occurs-check; the surface names itself, so HEAD's checker stack-overflows
until the guard lands) — explicit-self ships clean with it.

## The landmark forms (canonical — match these)
```clojure
;; (1) a surface = pure constraint: an ATTRIBUTE (data) + a METHOD (behavior). no impls.
(:wat::core::defsurface :acc::Adder :holder :wat::core::Record
  :features [n <- :wat::core::i64                                              ; attribute — data
             (add [self <- :acc::Adder  x <- :wat::core::i64] -> :wat::core::i64)])  ; method — behavior; self is a normal binder

;; (2) project the DATA up a tier — returns the macro-emitted backing record :acc::Adder$record
(:wat::core::to-record some-thing :acc::Adder)     ; -> :acc::Adder$record {n …}   (attributes only)

;; (3) default impl — write the BODY once; types come from the surface; WHERE ARE THE TYPES? in the contract.
(:wat::core::extend-surface :acc::Adder
  (add [self x] (:wat::core::i64::+ (:acc::Adder/n self) x)))
;;   ── expands to the REAL form (fully typed; the one ArgSpec) ──
;; (:wat::core::extend-type :acc::Adder$record :acc::Adder
;;   (add [self :- :acc::Adder$record  x :- :wat::core::i64] -> :wat::core::i64
;;     (:wat::core::i64::+ (:acc::Adder/n self) x)))

;; (4) per-type / foreign impl — the same real form, named target (un-demoted from foreign-only):
(:wat::core::extend-type :wat::holon::Vector :acc::Adder
  (add [self x] …))                                  ; foreign Vector taught to satisfy; holder derived + checked
```
**The four-tool kit:** `defsurface` (constraint) · `to-record` (data) · `extend-type` (impls — real) ·
`extend-surface` (impls — sugar). The full holder demo (3 holders, ambient satisfaction, the ladder, the foreign
adapter, Value) lives runnable at **`wat-scripts/demos/aggregates/showcase.wat.disabled`** (RED until built; rename
to `.wat` when green and the wat-scripts load gate owns it).

## One substrate dependency the kit needs
`extend-surface`'s macro (and `to-record`'s type-fill) need **expand/check-time access to a surface's method
signatures + its `$record` name** — a *read* reflection seam on the surface, not a new shape. That is the only
substrate touch the sugar requires; everything else is the existing `extend-type` / `aggregate-new` machinery.
