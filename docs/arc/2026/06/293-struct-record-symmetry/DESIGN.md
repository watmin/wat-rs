# Arc 293 — struct/record symmetry: one aggregate surface, decomplected and homed

**Status: SCOPED (2026-06-25).** Surfaced by arc 291's `/from-map` ergonomic-constructor work. **291 is
BLOCKED on this arc** (builder: *"291 is blocked on this new arc — this is our priority"*). The ergonomic
constructors (`/from-map`) cannot land until the underlying asymmetry is removed.

## The bug (builder, verbatim)

> *"we cannot operate on structs and records trivially — that's a fucking catastrophic bug — decomplection
> is highest priority."*

> *"this also hints another issue… we need symmetries here… (in our idealized clojure forms we're building
> towards) `(wat.core/defstruct …)` `(wat.core/defrecord …)` `(wat.holon/defrecord …)`."*

> *"make sure we move whatever is applicable `src/<some-ns>/<some-name>.rs` out of `src/*.rs`."*

wat **records** and **structs** are **isomorphic aggregates** — named, typed, fixed-field, positional
construction, named accessors — differing in exactly ONE semantic: **EDN-portability** (a record crosses the
wire; a struct does not — the arc-291 4b-i law, `is_portable_type(Struct) → false`). Yet they cannot be
operated on through one surface: the construction conventions differ (`(R 1 2)` vs `(S/new 1 2)`), there is
no `/from-map` for either, and the machineries that build them are entirely parallel and non-shared. Two
structurally-identical types that every consumer must special-case *by kind* is a coherence failure — the
abstraction leaks. "Coherence is the engine" (260); this breaks it. (Memory:
`feedback_uniform_operation_or_decomplect_is_catastrophic`.)

## The three symmetry axes (this arc closes all three)

1. **MECHANISM** — one shared emission surface (positional ctor · named accessors · `/from-map` · predicate),
   parameterized only by the one real difference `{portable?, construct-intrinsic}`.
2. **NAMING** — clojure-faithful PEER forms. Records use the Java-flavored `Type::def`; structs already use
   the idiomatic `def<thing>`. Unify on Clojure's `defrecord`/`defstruct`:
   | today | → arc 293 | role |
   |---|---|---|
   | `:wat::core::defstruct` | `:wat::core::defstruct` (unchanged head; gains a macro layer) | non-portable body aggregate |
   | `:wat::Record::def` | **`:wat::core::defrecord`** | EDN-portable base record |
   | `:wat::holon::Record::def` | **`:wat::holon::defrecord`** | holon-parented EDN record |
   This lands exactly on Clojure: `defrecord` + positional `->Name` + map `map->Name` ≡ our `defrecord` +
   positional ctor + `/from-map`. WE-LAND-ON-THE-GREATS — the symmetry isn't invented, it's where Clojure stands.
3. **HOME** — lift the aggregate machinery OUT of the `src/*.rs` megafiles into one `src/<aggregate>/` home.
   The shared home IS the structural expression of the decomplection.

## Grounded current state (read against the disk 2026-06-25)

### The layers — primitive/parse/Rust-gen are ALREADY symmetric; the gap is the user-macro layer

| layer | record | struct | symmetric? |
|---|---|---|---|
| type-reg PRIMITIVE (declaration head) | `recordtype` | `defstruct` | ✅ parallel |
| PARSE | `parse_recordtype` (`types.rs:2029`) | `parse_defstruct` (`types/defstruct.rs`) | ✅ parallel |
| TypeDef | `TypeDef::Record(RecordDef)` (`types.rs:197`) | `TypeDef::Struct(StructDef)` (`types.rs:124`) | ✅ parallel |
| Rust method-gen (ctor + accessors) | `register_record_methods` (`runtime.rs:1254`) | `register_struct_methods` (`runtime.rs:895`) | ✅ parallel |
| construct intrinsic | `:wat::Record::of` (`eval_record_of` `runtime.rs:13183`) | `:wat::core::struct-new` (`eval_struct_new` `runtime.rs:11628`) | ✅ parallel |
| access intrinsic | `:wat::Record/field-at` (`eval_record_field_at` `runtime.rs:13310`) | `:wat::core::struct-field` (`eval_struct_field` `runtime.rs:11747`) | ✅ parallel |
| infer | `infer_record_of` (`check.rs:12243`) + holon | (struct infer in check.rs) | ✅ parallel |
| value repr | `Value::wat__Record{class_fqdn, struct_form}` (+`wat__holon__Record`) | `Value::Struct(StructValue{type_name, fields})` | distinct (intentional — the wire law) |
| **USER MACRO** | **`Record::def` macro** (`wat/Record.wat`) — emits `(do (recordtype…) (defn ctor…) (accessors…))` | **NONE** — user writes `defstruct` (the primitive) directly | ❌ **THE GAP** |
| **`/from-map`** | **absent** | **absent** | ❌ **THE GAP** |

**The two non-redundant record paths (grounded, important):** `register_record_methods` fires ONLY for the
*typed* `recordtype` form (`field_types: Some` → else `continue`, `runtime.rs:1281`) — the typed-extensible
path used by `:wat::program::Env` and parent-chained records. `Record::def` emits an *untyped* `recordtype`
(name-strings) + its own ctor/accessors as wat forms, so `register_record_methods` skips it. They are NOT
redundant; they serve the macro path vs the typed-primitive path. **Structs have only the Rust-gen path**
(`register_struct_methods` over all `TypeDef::Struct`).

### The `/from-map`-must-be-a-macro constraint (the load-bearing fact)

Per `feedback_kwargs_is_always_a_macro` / the F5 fact (`project_macro_expansion_callable_surface`): *"if you
want kwargs, you write a macro."* `/from-map` reorders unevaluated `:x 1 :y 2` syntax at expand time → it
**must** be a macro. Rust method-gen (`register_*_methods`) cannot register a macro. Therefore the user-facing
form **must be a macro** that emits the `/from-map` companion. Records *have* a macro layer (`Record::def`)
to hang it on; structs do not. This is *why* `/from-map` is trivial for records and structurally homeless for
structs — same root as the missing struct macro layer.

### The reorder lever already exists

`kwargs-lower` (`wat/core.wat:202`) already does the exact `:field`→positional reorder via `pascal->kebab-in`
— but its emission *wraps* values in a `::Kwargs` record (`(~impl-kw ~@pos (~kwargs-ty ~@ovals))`). `/from-map`
wants the reordered values spliced directly into the ctor (`(~ctor ~@ovals)`, `n-pos=0`). Same reorder core,
different tail.

### Built-in structs (the wrinkle)

`register_struct_methods` serves BOTH user `defstruct` forms AND **15 Rust-registered built-in structs**
(`register_builtin(TypeDef::Struct(...))` in `types.rs` — `Bound`, `Launched`, …). Built-ins do NOT go through
a wat `defstruct` form, so the struct ctor/accessor generator cannot simply be deleted in favor of a macro.

### The machinery is smeared across the megafiles (the HOME axis)

| piece | file | line | notes |
|---|---|---|---|
| `register_struct_methods` / `register_record_methods` | `runtime.rs` | 895 / 1254 | megafile (32k) |
| `eval_struct_new` / `eval_struct_field` | `runtime.rs` | 11628 / 11747 | megafile |
| `eval_record_of` / `eval_holon_record_of` / `eval_record_field_at` | `runtime.rs` | 13183 / 13243 / 13310 | megafile |
| `infer_record_of` / `infer_holon_record_of` (+ struct infer) | `check.rs` | 12243 / 12361 | megafile (21k) |
| `parse_recordtype` / `StructDef` / `RecordDef` | `types.rs` | 2029 / 124 / 197 | megafile (3.8k) |
| `parse_defstruct` | `src/types/defstruct.rs` | — | ✅ already homed |
| `Value::Struct` / `Value::wat__Record` | `value/value.rs` | 195 / 345 | ✅ homed — STAYS (the reprs are not unified) |

No `src/aggregate/` (or `src/record/`+`src/struct/`) home exists. Lifting the above carves a few thousand
lines from `runtime.rs`/`check.rs` AND co-locates the struct+record machinery — the home becomes the surface.

### Blast radius (the rename)

| form | total files | `.wat` | `.rs` |
|---|---|---|---|
| `Record::def` | 85 | 10 | 75 |
| `holon::Record::def` | 9 | — | — |
| `defstruct` (already-correct head; no rename, only macro-layer add) | 60 | 12 | 48 |

`.wat` sites → **fix-wat** (`:wat::fix::rename-keyword-prefix`, boundary-aware, `feedback_lean_on_wat_migration_toolkit`).
`.rs` sites (wat-in-string fixtures) → the rename is an **unambiguous fully-qualified-keyword substitution**
(`:wat::Record::def` → `:wat::core::defrecord`) — the safest kind of mass edit; `wat-fixes-rust` does not
exist (`291/NOTE-wat-fixes-rust.md`), so these are scripted-substitution / hand-edits. A **retirement-table**
entry makes the old heads (`:wat::Record::def`, `:wat::holon::Record::def`) throw an exact teaching error so
nothing silently drifts.

## The target

Three clojure-faithful PEER macros — `defrecord` (core), `holon/defrecord`, `defstruct` (core) — over the
already-symmetric primitive/parse/Rust-gen layers, each emitting through ONE shared layer: positional ctor ·
named accessors · **`/from-map`** · predicate; differing only in `{portable?, construct-intrinsic}`. The two
`Value` reprs stay distinct so the **4b-i wire-boundary law remains a firm variant-level distinction**
(serialization + the type-gate are NOT touched). All the machinery lives in one `src/<aggregate>/` home.

```clojure
(:wat::core::defrecord  :my::Pt [x <- :i64  y <- :i64])   ; → ctor (:my::Pt 1 2) · (:my::Pt/x v) · (:my::Pt/from-map :x 1 :y 2) · is-Pt?
(:wat::holon::defrecord :my::HPt [x <- :i64 y <- :i64])   ; same surface, holon parent, holon::Record::of
(:wat::core::defstruct  :my::Cache [cap <- :i64  lru <- :my::Lru])  ; same surface, struct-new, NON-portable
```

### THE ARCHITECTURE DECISION — SETTLED: (C) full annihilation (four-questioned 2026-06-25)

How do the macros emit the ctor + accessors? Three options, run through the four-questions (flat YES/NO;
Obvious + Simple + Honest must hold before UX):

| | **A** — macros emit for user forms; `register_struct_methods` kept for the 15 built-ins | **B** — thin macros (from-map only); `register_*_methods` (relocated) gen for **all** | **C** — macros emit everything for **all** (built-ins → wat `defstruct`); `register_*_methods` **annihilated** |
|---|---|---|---|
| **Obvious?** | **NO** — structs get methods two ways (user-macro vs built-in-Rust); a *decomplection* arc shipping a dual path contradicts its thesis | YES — one gen path + one thin from-map macro | YES — ONE mechanism for every aggregate, user and built-in |
| **Simple?** | **NO** — two emission paths for one kind = braided | YES — one gen concept + one thin macro | YES — one concept, no Rust gen, no dual path |
| **Honest?** | marginal — claims "one surface" while keeping a struct split | YES — uniform surface; honest that gen stays Rust-relocated | YES — fully uniform AND self-hosted |
| **Good UX?** | (moot — disqualified) | YES | YES |
| **verdict** | **DISQUALIFIED** | PASSES (but defers the engine decomplect — a deferral in costume) | **PASSES + qualified annihilation + idealized self-hosted state — CHOSEN** |

**DECISION (builder, 2026-06-25): (C).** *"annihilation is our greatest pleasure."* `register_struct_methods`
and `register_record_methods` are **annihilated**; the 15 built-in structs migrate to wat `defstruct` forms;
every aggregate — user and built-in, record and struct — flows through ONE wat macro emitting the full
surface (typereg · positional ctor · named accessors · `/from-map` · predicate). The Rust floor shrinks to
the irreducible intrinsics (`struct-new` / `Record::of` / `field-at` / `struct-field`) + the `TypeDef`s +
parse. (A was my first, glib pick; the four-questions caught that it re-introduces the very split this arc
exists to kill — the discipline overturned the recommendation. Recorded per `feedback_self_prompt_injection`.)

**The one feasibility crux to conquer (the boss):** record **parent-inheritance** — `register_record_methods`
walks the parent chain at *registration* to flatten inherited fields into the ctor (`runtime.rs:1286`;
the `program::Env` typed-extensible chain). A wat macro emits at *expand* time and cannot, by default, query
its parent's field list. **293.0b probe:** is there (or can there cheaply be) a macro-expand-time reflection
intrinsic that yields a registered type's fields, so the macro flattens inherited fields itself? If YES → C
is total. If NO and one can't be cleanly minted → `register_record_methods` survives as the **single named,
bounded** Rust exception for the typed-extensible-parent path ONLY (never silent, never a wholesale B
fallback). Structs appear flat (no parent walk in `register_struct_methods`) → struct-side C is unobstructed
(confirm at strike-grounding). Default stance: **annihilate; make the substrate force the reflection
intrinsic into existence** (`feedback_substrate_forces_idealized_state`).

## Decomposition (sequenced; refine once the fork above is settled)

> Order obeys `feedback_qualified_annihilations_are_priority` (the decomplect precedes the additive) and
> `examinare` (each strike re-grounds its sites + carries its own RED probe + weighs against the disk).

- **293.0a — DESIGN (this doc) + RED probe.** A probe asserting the target surface (`(:T/from-map :x 1)` and
  uniform construct/access on BOTH a record and a struct) — RED at HEAD. Commit before build.
- **293.0b — the parent-inheritance feasibility probe (the boss-scout).** Determine whether a
  macro-expand-time reflection intrinsic for a registered type's fields exists / can be cheaply minted (the
  crux above). Decides whether C is total or carries the one named `register_record_methods` exception.
- **293.1 — the aggregate HOME.** Mint `src/<aggregate>/` (intueri the name). LIFT `register_*_methods`,
  `eval_struct_new/field`, `eval_record_of/holon/field_at`, `infer_record_of/holon`, `parse_recordtype`,
  `StructDef`/`RecordDef` from `runtime.rs`/`check.rs`/`types.rs` into it (re-export at old paths to absorb
  churn — the 251.2 / wat-reader pattern). Pure relocation, zero behavior change, SET-diff ∅. Earns the
  megafile-shrink + co-locates the surface for what follows.
- **293.2 — the shared emission layer + `defstruct` macro + built-in migration + the FIRST annihilation.**
  Build the shared ctor/accessor/`from-map` wat emission. Introduce the `defstruct` MACRO; migrate the 15
  built-in structs (`register_builtin(TypeDef::Struct)`) to wat `defstruct` forms; **ANNIHILATE
  `register_struct_methods`.** Structs get `/from-map` + uniform ctor, all-wat.
- **293.3 — `defrecord` / `holon::defrecord` (rename + `/from-map`) + the SECOND annihilation.** Rename
  `Record::def` → `:wat::core::defrecord`, `holon::Record::def` → `:wat::holon::defrecord`; route both
  through the shared layer; records gain `/from-map`; **ANNIHILATE `register_record_methods`** (or reduce it
  to the single named parent-inheritance exception if 293.0b forces it). fix-wat the `.wat` sites;
  scripted-substitute the `.rs` sites; retirement-table the old heads. PRIME-suffix discipline
  (`project_prime_suffix_replaces_then_drops`) where it fits.
- **293.4 — close + amend.** The RED probe green on BOTH kinds; `register_*_methods` gone (or the lone named
  exception); full workspace SET-diff ∅; the home warded. Amend 291's `CURRENT-STATE` breadcrumb to unblock
  `/from-map` (now trivial, falls out of the shared layer) → resume 291.

## Out of scope (affirmative cuts)

- **Unifying the `Value` reprs** (`Struct` + `wat__Record` → one repr + a portability bit). More "pure," but it
  reopens serialization, the wire-boundary gate, and the closure-extract paths — high blast radius for no
  near-term gain. The variant-level distinction *is* the wire law; keep it. (A future arc may revisit.)
- **The dotted clojure-symbolic surface** (`wat.core/defrecord` ↔ `:wat::core::defrecord`) — that is arc 251's
  `:wat::` → `wat.` surface, a separate large axis. 293 picks 251-ready names; it does NOT switch notation.
- (Migrating the 15 built-in structs to wat `defstruct` forms is now **IN scope** — 293.2 — per the (C)
  decision; `register_*_methods` is annihilated, so the built-ins must move.)

## Intueri casts owed (standing naming discipline)

- the aggregate HOME name (`src/aggregate/`? `src/record/`? `src/nominal/`? — names both kinds + the shared surface)
- the shared emission-layer name (the macro/helper both `defrecord` and `defstruct` route through)
- (`defrecord`/`defstruct`/`holon::defrecord` + `/from-map` are builder/intueri-settled already)

## Pairs
`291/STRIKE-4b-struct-state.md` (the wire law that makes the reprs distinct) · `291/CURRENT-STATE.md` (the
blocked `/from-map` pick-up) · `291/NOTE-wat-fixes-rust.md` (the `.rs` migration boundary) ·
`feedback_uniform_operation_or_decomplect_is_catastrophic` · `feedback_kwargs_is_always_a_macro` ·
`feedback_lean_on_wat_migration_toolkit` · `feedback_substrate_forces_idealized_state` · the homes migration
(`255/VIGILATUM` / `OP-PLACEMENT`).
