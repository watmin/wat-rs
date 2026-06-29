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
