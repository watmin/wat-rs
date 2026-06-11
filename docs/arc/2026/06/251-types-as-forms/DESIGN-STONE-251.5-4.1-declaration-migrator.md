# DESIGN — Stone 251.5 / Slice 4.1: the grammar-bearing declaration migrator + `<T,U>`-drop

**Status: SHIPPED 2026-06-10 (`c89f1760`). Sonnet-built, orchestrator-weighed + extended.**
`wat-migrate/fix-decl.wat` (`:migrate::` ns, non-blessed, retires at the hard-cut). Probe
`probe_arc251_decl_migrator` 8/8; fix-source 8/8 + renderer 9/9 (no regression). **The weigh caught
the sonnet probe UNDER-COVERED** — it tested typealias/newtype/defn but missed **typeunion** (40
`.wat` instances) whose child[2] is a VECTOR of non-arrow'd member types that `fix-seq` mis-routes to
`wat.core/i64`. Closed it (`fix-types`/`fix-type-vector` + a typeunion branch). Empirically verified
defenum (variant tags stay keywords; fields convert) + defstruct. The first slice of Strike 4. Home:
the throwaway, non-blessed migrator (no perpetual grammar drift, per the four-Q keystone decision).

## The lair (grounded — fix.wat read + empirical)
`fix.wat`'s `fix-seq` (grammar-FREE local rules) ALREADY handles most of a declaration:
- declaration/defn NAME → `head-keyword?` fires on ANY `::`-keyword (`fix.wat:56`) → `keyword/to-symbol`
  → a symbol. Correct for a plain name.
- arrow'd type-slots (defstruct fields `[name <- :T]`) → post-arrow rule → `keyword/to-type-form`. ✓
- parametric type-slots → `type-shaped-keyword?` → `keyword/to-type-form`. ✓

**Two gaps remain — both empirically confirmed** (running blessed `:wat::fix::fix-source`):
- **Gap A — non-arrow'd core-scalar type-slot.** `(typealias :svc::Alias :wat::core::i64)` →
  `(wat.core/typealias svc/Alias wat.core/i64)`. The target `:wat::core::i64` hit `head-keyword?` →
  `keyword/to-symbol` → `wat.core/i64`. It is a TYPE → must be `wat.type/i64` (via `keyword/to-type-form`).
  (User-type targets like `:wat::holon::HolonAST` render identically under both verbs, so they're
  already correct — only CORE scalars in a bare type-slot diverge.)
- **Gap B — generic NAME corruption.** `(defn :wat::stream::map<T> [x <- :T] -> :T x)` →
  `(wat.core/defn (wat.stream/map T) [x :- T] :- T x)`. The defn name `map<T>` has `<`+`>` → MISFIRES
  through `type-shaped-keyword?` → `keyword/to-type-form` → a **parametric FORM** `(wat.stream/map T)`.
  The name slot becomes a LIST where a symbol must be. The name must be `wat.stream/map` (plain symbol,
  `<T>` dropped — safe now that 251.7 generalizes the bare sig vars).

`fix-seq` cannot fix either without knowing it is IN a declaration/name position — that is
grammar-bearing, so it does NOT belong in perpetual `fix.wat` (the keystone braid). → the migrator.

## The strike — the throwaway migrator
A wat function (non-blessed home, e.g. `wat-migrate/fix-decl.wat` or inline in the 4.2 drive) that is
a position-aware declaration/defn rewriter. Pseudocode:
```
fix-form(node):
  if node is a list whose head keyword ∈ DECLARATION-HEADS {defn, typealias, newtype, recordtype, defclause, def}:
     name-pos (arg1)      → name-fix: strip a trailing `<…>` suffix, then keyword/to-symbol → plain symbol
     type-slot positions  → keyword/to-type-form   (per-kind: typealias/newtype/recordtype arg2; others as mapped)
     all other children   → fix-source (recurse)
  else:
     fix-source(node)
```
- **DECLARATION-HEADS** is the bounded per-kind grammar table (which arg is a name, which are type-slots).
  This is the grammar-bearing knowledge; it lives ONLY here and retires at the hard-cut.
- **name-fix** (the `<T,U>`-drop): a name keyword `:wat::stream::map<T,U>` → strip the `<…>` suffix →
  `:wat::stream::map` → `keyword/to-symbol` → `wat.stream/map`. (Decision when built: name-fix as a
  grammar-FREE symbol-suffix-strip vs part of the migrator — four-Q; lean: in the migrator, since it
  only applies in name positions and is transition-only.)
- defstruct fields stay handled by `fix-seq` (arrow'd) — the migrator only overrides the bare
  name + bare non-arrow'd type-slots.

## The probe (RED at HEAD)
`tests/probe_arc251_decl_migrator.rs` (defines the migrator inline over the homoiconic bridge, like
the 258.3 fix-source probe; the build moves it to the non-blessed home + the 4.2 drive calls it):
- **C01 (gap A):** `(typealias :svc::Alias :wat::core::i64)` → `(wat.core/typealias svc/Alias wat.type/i64)`.
- **C02 (gap B):** `(defn :wat::stream::map<T> [x <- :T] -> :T x)` → `(wat.core/defn wat.stream/map [x :- T] :- T x)`
  (name a PLAIN SYMBOL, `<T>` dropped). LOAD-BEARING.
- **C03 (generic decl name + parametric target):** `(typealias :Foo<T> :wat::core::Vector<T>)` →
  `(wat.core/typealias Foo (wat.type/Vector T))`.
- **C04 (preservation):** `(typealias :wat::edn::Tagged :wat::holon::HolonAST)` →
  `(wat.core/typealias wat.edn/Tagged wat.holon/HolonAST)` (user types already correct under fix-source).
- **C05 (newtype + defstruct fields untouched):** a defstruct with arrow'd fields migrates fields via
  the existing arrow rule; only its name is handled by the migrator.

## Out of scope
- The actual corpus drive (4.2) + the rust-test-string runes (4.3) + hard-cuts (4.4).
- `defclause` clause-internal type-slots beyond the name — defclause clauses use `[args]` with
  arrows, so `fix-seq` handles them; confirm in the probe, STOP if a non-arrow'd clause type-slot
  surfaces.
