# DESIGN — Records as first-class types via an is-a hierarchy (`derive` + `isa?`)

**Arc:** 237 (polymorphism consolidation). **Status:** DESIGN locked 2026-05-25.
**NOT a new arc.** This is the record-*consumer* of arc 237's type machinery —
`typeunion` + `defclause` were built FOR this. (User direction 2026-05-25,
strongly opposed to spinning a separate arc: *"we do all of this work in the
current arc."*)

**Mandate (user, 2026-05-25), verbatim intent:** *"we slay this dragon for all
time ... I never want to fight this boss again ... I hate type theory shit."*
This doc is the boss-fight map. A future (post-compaction) instance must be able
to execute from it WITHOUT re-deriving the reasoning. If you are that instance:
read this top-to-bottom and you have the whole fight.

---

## 0. Names — LOCKED by intueri (2026-05-25)

We are Clojure-inspired, not Clojure-shackled. Intueri (the naming spell) was
cast and **rejected both bare Clojure names** as borrowed-taxonomy mumble in
wat's *statically-typed* context:

| role | **wat name (LOCKED)** | Clojure reference | reads as |
|---|---|---|---|
| the is-a edge      | **`:wat::core::typesub`** | `derive` | `(typesub :ns::Sphere :wat::holon::Record)` |
| the directional check | **`:wat::core::subtype?`** | `isa?` | `(subtype? :ns::Sphere :wat::Record)` → true |

Why these win: **honesty** — wat is statically typed, so this is genuine
*subtyping* the checker enforces; `subtype?` tells that truth where `isa?`
imports Clojure's dynamic-dispatch flavor. **Shared stem** — `typesub`/`subtype?`
mirror each other (Clojure's `derive`/`isa?` don't). **Family fit** — `typesub`
beside `typealias`/`typeunion`; `subtype?` beside `conforms?` (its `type × type`
sibling). Arg order self-documents: sub/child first.

> Below, where the prose says `derive`/`isa?` it means the Clojure *reference*;
> the wat names are `typesub`/`subtype?` per this section.

---

## 1. The boss — why this dungeon sucked

Three tangled symptoms, one root:

- **Records aren't types.** `defrecord` (arc 234/227, `wat/Record.wat`) is a
  MACRO that emits `defn`s — a constructor returning the opaque umbrella
  `:wat::Record`, accessors, a predicate — and registers **no** `TypeDef`.
  Per-class identity (`:my::Circle`) lives only as a runtime tag (`class_fqdn`
  on `Value::wat__Record`). The only record-related `TypeDef` is the opaque
  zero-field umbrella `:wat::Record` (types.rs ~1277).
- **The `is-X?` asymmetry.** `register_type_predicates` (runtime.rs ~3198)
  synthesizes a ∀T `is-<Name>?` (body `(conforms? v :FQDN)`, returns `bool`,
  never type-errors) for every TypeEnv `TypeDef` (Struct/Enum/Newtype/Union).
  Records aren't in the TypeEnv, so their predicate is macro-emitted with the
  narrowing param `[v <- :wat::Record]` — it type-*errors* on a non-record
  instead of returning `false`. Four siblings get the clean form; records don't.
- **No flavor distinction.** Every `Value::wat__Record` is forced dual-form
  (struct_form + holon_form) per arc 234. That imposed VSA on users who just
  wanted a struct, and gave nowhere to impose VSA *constraints* on the values.

**Root:** you cannot register a type from the macro surface, there's no place
for a record class in the type system, and there's no is-a relation to express
"this record is a more-specific kind of that record."

---

## 2. The split that started it (user framing 2026-05-25)

`defrecord` did two jobs (rust struct + VSA hologram) and forced both on
everyone. Break it in two:

- **`:wat::Record::def`** — base record. **Just the rust struct.** No holon_form.
  VSA is NOT imposed.
- **`:wat::holon::Record::def`** — holonic record. struct **+ holon_form** (the
  VSA hologram).
- **Relationship:** `:wat::holon::Record` **is-a** `:wat::Record`. *"A holonic
  record is a base record and more — the holographic composition."*
  - a func wanting a base record CAN accept a holonic record (subtype
    substitutes for supertype);
  - a func wanting a holonic record CANNOT accept a base record.

This split is **why arc 237 built**:
- **`defclause`** (guards/`:ensure` — Erlang guards / Ruby ensure): to impose
  VSA constraints on holonic-record values that base records don't carry.
- **`typeunion`** (fractal closed sum): `(:typeunion :Foo [:i64 :f64])` then
  `(:typeunion :Baz [:Foo :String])` — transitive nesting confirmed in
  `collect_union_members` (types.rs ~3031), both at check-time and runtime.

---

## 3. The convergence — Clojure was already standing in this room

Clojure draws **three** lines most languages blur into one. We touched all three:

| Clojure                                   | axis                                  | wat status            |
|-------------------------------------------|---------------------------------------|-----------------------|
| `derive` / `isa?` / `ancestors`           | **hierarchy** (is-a, open, directional) | ← **THE GAP**        |
| `defprotocol` / `extend` / `satisfies?`   | **behavior** (can-do)                 | arc 232 (in-flight)   |
| spec `s/or`, sum                          | **closed alternation**                | `typeunion` ✓ shipped |

"Holonic is-a record" is the **hierarchy** axis = Clojure's `isa?`/`derive`.
- NOT a `typeunion` — that's a *closed, symmetric* sum (right for `:i64 | :f64`;
  its unify is symmetric per doctrine — check.rs ~14613 — so it would *leak*:
  a value typed as the bare umbrella would wrongly satisfy a holonic-only slot).
- NOT a `defprotocol` — that's the *behavior* axis (what operations dispatch on
  the value), orthogonal. Using it for "is this a record" conflates membership
  with behavior.

This is the arrival-where-a-great-stood signal (`user_no_literature`): three
independent needs landed exactly on Clojure's three-way split. **Convergence #17
candidate** (`isa?`/`derive` hierarchy — the fourth leg beside #15's
defrecord+defprotocol+extend-type+satisfies?).

---

## 4. The exact additions (the complete slay-list)

### NEW — the two mechanisms (the dragon itself)

1. **`derive` — the is-a edge.** A standalone child→parents relation, Clojure's
   hierarchy. Orthogonal to `TypeDef` kind (a tag can be anything). Home: a
   registry on `TypeEnv` (e.g. `HashMap<String, Vec<String>>`), reachable from
   BOTH check.rs and runtime.rs (TypeEnv is available in both). For records the
   split-macros call `derive` internally; a user-facing `(:wat::core::derive
   :child :parent)` is OPTIONAL (ship only if a non-record user hierarchy
   surfaces — minimal-form).

2. **`isa?` / `is_subtype(sub, super)` — the directional check.** Transitive
   walk UP `sub`'s parent chain looking for `super`. THE core new primitive.
   - **Lives BESIDE `unify`, never inside it.** `unify` is symmetric/structural
     (makes two types equal); subtyping is directional. At the function-argument
     boundary the checker accepts an arg when `is_subtype(arg_ty, param_ty)
     || unify(arg_ty, param_ty)`.
   - Two homes: **check.rs** (static accept at `[v <- :wat::Record]`) and
     **runtime.rs** (`conforms?` gains the same parent-walk).

### CONSUMER PLUMBING — records use the hierarchy

3. **Records register a `TypeDef`** so `register_type_predicates` synthesizes the
   ∀T `is-<Name>?` (returns `bool`, never type-errors — kills the asymmetry) and
   `conforms?` resolves nominally. (They register NONE today — the root bug.)
   Reuse the struct-TypeDef shape (+ the derive edge) rather than minting a new
   `TypeDef::Record` kind UNLESS the probe shows a field is genuinely needed.

4. **The macro split** — `:wat::Record::def` (base) / `:wat::holon::Record::def`
   (holonic). Each emits: the TypeDef registration + a `derive` to the right
   parent (base → `:wat::Record`; holonic → `:wat::holon::Record`, which itself
   derives `:wat::Record`) + constructor/accessors/predicate. **HARD CUT** from
   the old single `defrecord` — no alias (arc 234.6 discipline).

5. **Base records drop `holon_form`.** `Value::wat__Record.holon_form` becomes
   `Option<Arc<HolonAST>>` (None = base "rust", Some = holonic "rust+holon").
   This is the "which def declared it" the user named — fixed at def-time, read
   off the value/TypeDef. **Recommended** over base=`Value::Struct`: preserves
   all existing record tooling (`/field-at`, constructor, accessors already work
   on `wat__Record`); the ONLY base/holonic difference becomes the presence of
   holon_form, which is exactly the "rust" vs "rust+holon" line. (The one
   decision to confirm at probe time; base=Struct is the rejected alternative —
   bigger refactor, splits the record path across two Value variants.)

### ALREADY DELIVERED — no new work (confirmed by the 2026-05-25 crawl)

- `typeunion` (237.1) — the closed sum tool; **untouched** (stays for
  `:i64 | :f64`, fractal). NOT repurposed for records.
- `defclause` (237.2/.3) — the VSA-constraint guards/`:ensure` for holonic records.
- `conforms?` (237.5) — exists; gains the `isa?`-walk **extension** (#2 runtime home).
  Already falls back to `class_fqdn` for unregistered record names
  (runtime.rs ~16173).
- `is-<Name>?` synthesis (237.6) — exists; works the moment records are TypeDefs (#3).
- `declared_type_name` → per-class FQDN for records — **already there**
  (runtime.rs:1311; returns `class_fqdn`).
- struct `TypeDef` machinery — the template for #3.
- dual-form `Value::wat__Record` (arc 234) — holonic uses it as-is.

---

## 5. The honest divergence — our own path to the same room

Clojure's hierarchy is **runtime-only** because Clojure is dynamically typed
(`isa?` reads an atom; `defmulti` dispatches at call time; no static checker).
**wat is statically typed.** So our `isa?` must serve BOTH the type-checker
(static accept at the arg boundary) AND the runtime (`conforms?`, defclause).
We take Clojure's *concept* (hierarchy ≠ protocol ≠ sum; open directional is-a)
and implement it through the type system Clojure never had. Convergence on the
IDEA, not a transcription of the impl.

---

## 6. The lattice

```
:wat::Record                         (root — "any record")
  ├── :ns::Circle                    (base;     derive → :wat::Record)
  └── :wat::holon::Record            (derive → :wat::Record; "any holonic record")
        └── :ns::Sphere              (holonic;  derive → :wat::holon::Record)
```

- `[y <- :wat::Record]`         → accepts anyone is-a record (all of the above).
- `[x <- :wat::holon::Record]`  → accepts only holonic (Sphere, holon::Record); rejects Circle.
- `[x <- :ns::Circle]`          → accepts only Circle (leaf).

Directional: subtype substitutes for supertype, never the reverse. **No
symmetric leak** — this is precisely why it's `isa?`, not `typeunion`.

---

## 7. Decisions locked

1. **Three mechanisms, three jobs, never conflated:** `isa?`/`derive`
   (hierarchy) ≠ `typeunion` (sum) ≠ `defprotocol` (behavior).
2. **`is_subtype` lives beside `unify`,** not inside it (directional vs symmetric).
3. **HARD CUT:** `defrecord` → `:wat::Record::def` + `:wat::holon::Record::def`.
   No alias, no single-form fallback.
4. **Base record repr:** `holon_form: Option` (recommended; confirm at probe).
5. **All inside arc 237.** is-Foo? does NOT get "fixed" in isolation; it resolves
   when records become TypeDef nodes in the hierarchy.

---

## 8. The dungeon — rooms, traps, stepping-stones (engineered 2026-05-25)

### The lair (rooms — where the work lands)
- `src/types.rs` — `TypeEnv` (home of the new `typesub` edge-registry), `TypeDef`
  enum (~185), `register_with_span` (~263; idempotent + DuplicateType gate),
  `expand_alias` (~2788; skips Newtype — proof Newtype is nominal, the wrong
  cons-cell), `:wat::Record` opaque struct (~1277).
- `src/check.rs` ~14586 — `unify` (symmetric union arms ~14603-14624). `subtype?`/
  `is_subtype` slots **beside** it; the function-arg accept site becomes
  `is_subtype(arg, param) || unify(arg, param)`.
- `src/runtime.rs` — `register_type_predicates` ~3198 (synthesizes ∀T `is-X?`;
  DuplicateDefine guard ~3261), `conforms_check` ~16130 (gains the parent-walk;
  already has the unregistered-name→`class_fqdn` fallback ~16173),
  `declared_type_name` ~1311 (already per-class), `val_type_path` ~7513 (collapses
  records to `:wat::Record` — revisit only if per-class defclause dispatch wanted),
  `Value::wat__Record`.
- `wat/Record.wat` — the macro to split + HARD-CUT.

### The traps (perceived before we step)
- **T1 — macro-emit-before-register ordering.** Does a macro-emitted `typesub`/
  type-registration form get *seen* by the type-registration pass? If no → the
  macro calls a substrate registration hook instead. ← **the gate (S0).**
- **T2 — `subtype?` placement.** Must sit *beside* `unify`, never inside (folding
  directional into symmetric corrupts arithmetic unify — the symmetric-leak reborn).
- **T3 — `DuplicateDefine` collision.** Once a record TypeDef synthesizes `is-X?`,
  the macro must STOP emitting its own predicate (runtime.rs:3261 errors on collide).
- **T4 — `holon_form: Option` blast radius.** Touches every `Value::wat__Record`
  match site (Eq/Hash/Display/HolonRepresentable + `/field-at`). Substrate-as-teacher
  cascade — fail-count is the meter, not a crisis.
- **T5 — HARD-CUT migration.** `defrecord` → two macros breaks every existing
  record caller. Expected; sweep it.

### The stepping-stones (each enables the next; simple → complex)
- **S0 — GATE PROBE** (FM 2-bis, ~40 lines, **strike nothing until green**):
  T1 (macro-emit registration pickup) + T2 (`subtype?` slots beside `unify`,
  accept = `is_subtype || unify`, no existing-inference disturbance). If T1 fails →
  macro→substrate-hook fallback. STOP triggers are rejection criteria, not defer slots.
- **S-A** — substrate hierarchy: `typesub` edge-registry on `TypeEnv` + `subtype?`
  walk (check.rs, beside unify) + `conforms?` parent-walk (runtime.rs). Roots
  `:wat::Record` ⊃ `:wat::holon::Record` (holonic `typesub` base) as built-ins.
  *Probe:* the 4 lattice accept/reject cases (`[v <- :wat::Record]` accepts holonic;
  `[v <- :wat::holon::Record]` rejects base; leaf accepts only itself; directional —
  base-typed value rejected at a holonic slot).
- **S-B** — records-as-TypeDef: each record class registers a TypeDef + `typesub`
  edge → `register_type_predicates` synthesizes ∀T `is-X?`. *Probe:* `is-X?` on a
  non-record → `false`, NOT a type-error (the asymmetry dies here).
- **S-C** — macro split: `:wat::Record::def` (base) / `:wat::holon::Record::def`
  (holonic), each wiring `typesub` + base drops `holon_form` (`Option`). HARD CUT.
  *Probe:* base has no holon-form, holonic does; both first-class; both sit in the
  lattice.
  > ⚠ **SUPERSEDED 2026-05-26 — `holon_form: Option` is REJECTED (semantic abuse).** See
  > § DESIGN CORRECTION at the end of this doc. The honest shape is **two Value variants**
  > (rename existing → `wat__holon__Record`; mint base `wat__Record`), and S-C splits into
  > S-C.1 (rename) → S-C.2 (mint base) → S-C.3 (macro split). The `Option` repr below in
  > §§ 4/8 (lines ~150/213) is dead; the two-variant correction governs.
- **S-D** — migration sweep: existing `defrecord` callers → the right new macro
  (substrate-as-teacher cascade).
- **S-E** — INSCRIPTION (folds into the arc 237 closure stones).

---

## DESIGN CORRECTION (2026-05-26) — base vs holonic is TWO VARIANTS, not `holon_form: Option`

**Authoritative. Supersedes the `holon_form: Option` shape wherever it appears above
(§ 4 line ~150-151, § 8 trap T4 line ~213, S-C bullet line ~266).** The body stays as
the stepping-stone it was; read THIS for what to build. (User direction 2026-05-26, live
design dialogue — the `Option` shape was caught as semantic abuse before it shipped.)

### Why `Option` is rejected — semantic abuse

`holon_form: Option<Arc<HolonAST>>` with `Some` = holonic / `None` = base **overloads
`Option`'s meaning**. `Option` means *presence/absence of a value* — it must never be
read as "Some ⇒ this is a holonic record." That is flavor encoded in a convention a
reader has to learn — the exact convenience-variant dishonesty the substrate already
purged (arc 230 retired Symbol/Keyword/Tag/Nil into pure structure; arc 233 killed
`Value::Tracked` + the `#[wat_value]` macro structurally forbids meaning-bearing wrapping
variants). The flavor must be **structural — the value IS what it is**, decoded by
`match`, not by inspecting an `Option`. (`feedback_no_semantic_abuse_of_option`.)

### The record vs struct boundary (the clarifying fact)

- **`Value::Struct`** holds *any rust thing* — channels, fns, handles, non-EDN values.
- **A record is STRICTER:** its data is restricted to **holonic-representable = EDN-only**
  values. That restriction is what makes a record a record, and is why **base record ≠
  struct** (a struct would admit non-EDN data a record forbids). Reusing the struct
  variant for "base" is therefore WRONG.

### Base vs holonic = materialization, not data domain

Both hold the same EDN-restricted data; neither reduces what's supported.
- **Base (wat) record** materializes ONE flavor — the wat flavor (`struct_form`). The
  holon flavor is **latent** (projectable on demand, since the data is EDN-capable),
  just not stored.
- **Holonic record** materializes BOTH flavors of the same data — wat (`struct_form`) +
  holon (`holon_form`). It **implements the hologram**: one meaning, two simultaneous
  projections. *"A holonic record does not reduce the supported data; it holds two
  flavors of the data."*

### The honest representation — two distinct variants

- **Rename** existing `Value::wat__Record { class_fqdn, struct_form, holon_form }` →
  **`Value::wat__holon__Record`**. It already carries both flavors — it already IS the
  hologram; the current record was doing both jobs and forced the hologram on every user.
  It is **not wrong — it has the wrong name.** Renaming it to holonic is TRUTH (and aligns
  the variant with the seeded type `:wat::holon::Record`; frees the `wat__Record` name).
  Identity stays `holon_form` (arc 234 — correct, *for the holonic flavor*).
- **Mint** `Value::wat__Record { class_fqdn, struct_form }` = the reduced **wat (base)**
  record. EDN-restriction enforced at construction; structural Eq/Hash/Display/HolonRep
  over `(class_fqdn, struct_form)`; holon flavor projectable-on-demand, not stored.
- `match` distinguishes; no `Option`, no flag, no convention.

### The relation (locked)

`:wat::holon::Record` **`<:`** `:wat::Record` — **holonic is the subtype** (its structure
is a superset: it HAS the `struct_form` a base-wanting consumer needs, plus the holon
flavor). So:
- a func wanting a **holonic** record CANNOT receive a base (wat) record;
- a func wanting a **base (wat)** record CAN receive both base AND holonic.

This is exactly the Liskov direction S-A1's `assignable` already wired (holonic substitutes
for base; not vice-versa).

### Corrected stone sequence (replaces the single S-C "macro split")

- **S-C.1** — RENAME `Value::wat__Record` → `Value::wat__holon__Record` (the existing
  dual-form variant is honestly the holonic one). Pure mechanical sweep (~72 `holon_form`
  sites, mostly `runtime.rs`). **Baseline-preserving** (every current record is honestly
  holonic; behavior identical; frees the name). The safe foundation.
- **S-C.2** — MINT base `Value::wat__Record { class_fqdn, struct_form }` + per-variant
  Eq/Hash/Display/HolonRep (structural; holon-on-demand) + EDN-restriction at construction.
  Additive; nothing produces it yet.
- **S-C.3** — macro split: `:wat::Record::def` → base, `:wat::holon::Record::def` →
  holonic; wire `typesub`; wat-surface proof (base Eq/Hash structural; holonic substitutes
  for base via S-A1; func-wanting-holonic rejects base).
- **S-D** — migrate existing `:wat::Record::def` callers (base vs holonic — the fallout,
  dealt with, no hesitation; HARD CUT).
- **S-E** — folds into 237.9.

The `REMAINING-ORDER.md` tracker carries this corrected sequence.

---

## DESIGN CORRECTION 2 (2026-05-26) — field access is via the STRUCT; holon-ops are holonic-only; field names are a CLASS property

**Authoritative; refines CORRECTION 1.** The two-variant decision (CORRECTION 1) stands.
What changes: the *on-demand holon projection* idea in CORRECTION 1 is **dead**, replaced
by the user's Ruby is-a model (2026-05-26 live dialogue):

```ruby
class Record;        def initialize(fields); @fields = fields; end;       end   # the struct
class HolonicRecord < Record;  def initialize(fields); super; build_holon(fields); end;  end   # struct + holon
```

**HolonicRecord IS-A Record** — a holonic record *has the struct a base record has*, **plus**
the holon form. From that:

1. **Field access is variant-agnostic, via the STRUCT.** `(:field1 rec)`, the generated
   positional accessor `:ns::Rec/field1`, and `field-at` all read `struct_form` — for BOTH
   base and holonic. At the access site you do NOT know which variant you hold and do NOT
   need to (*"we don't know if this is a :wat::Record or a :wat::holon::Record in this
   invocation path"*). Holonic just *also* has more.
2. **Holon-ops go via `holon_form` — holonic ONLY.** A function needing the holonic
   representation uses the tooling holonic records provide. A base record has no holon_form;
   it cannot do holon-ops (the type system bars base from `:wat::holon::Record` params). There
   is **NO on-demand projection** — holonic *stores* both flavors (both always immediately
   available); base *has only* the struct. That is the entire point of the split.
3. **Field names are a CLASS property, not a value property.** The Ruby model: the class
   defines the attrs; the instance holds the values. So **`RecordDef` gains `field_names`**;
   `struct_form` stays positional `Arc<Vec<Value>>`. Name-based access = look up the index in
   the class's `RecordDef.field_names`, then `struct_form[index]`. Non-redundant (names live
   once, on the class), and it makes access variant-agnostic.

**The substrate bug this exposes (must fix):** today `keyword_accessor_record`
(`src/runtime.rs`) resolves field names by walking `holon_form`'s Bundle — i.e. field access
routes *through the holon form*. That is backwards: a base record has no `holon_form`, and
field access must not depend on it. Re-route name-based access through
`RecordDef.field_names` + `struct_form`.

### Re-sliced S-C (supersedes the S-C.1→S-C.3 list in CORRECTION 1; S-C.1 RENAME already SHIPPED `0c574661`)

- **S-C.2a** — `RecordDef` gains `field_names`; `recordtype` parses/stores them; the
  `:wat::Record::def` macro emits them. (Ripples back into S-B.1's `recordtype` shape — fine.)
- **S-C.2b** — re-route `keyword_accessor_record` (and any name-based path) through
  `RecordDef.field_names` + `struct_form`, NOT `holon_form`. Now variant-agnostic; baseline-
  preserving for holonic (same answers, new path).
- **S-C.2c** — mint base `Value::wat__Record { class_fqdn, struct_form }` (structural Eq/Hash;
  field access via the uniform 2b path; holon-ops error — holonic-only). The compiler cascade
  (or-pattern the identical struct sites; split only Eq/Hash + holon-op sites, where base
  errors).
- **S-C.3** — macro split (`:wat::Record::def` → base / `:wat::holon::Record::def` → holonic;
  static type distinction = constructor return type). **S-D** — migrate callers.

(`field_names`-on-`RecordDef` vs names-on-`struct_form` is the one impl seam; lean = `RecordDef`,
the Ruby-faithful non-redundant home, pending the build.)

Each stone runs the full crawl loop: sub-DESIGN → committed FM-2-bis probe →
BRIEF (read-in-order `file:line` + impl sketch + numbered REJECTION STOPs + cite
prior SCORE shape) → EXPECTATIONS (scorecard + band + trap-doors) → baseline
re-run → spawn `model:"sonnet"` background + `ScheduleWakeup`@2× → SCORE vs
independent local re-run → commit on green.

---

## 9. What we deliberately do NOT build (minimal-form guardrails)

- No `make-hierarchy` / local hierarchies — the TypeEnv IS the one hierarchy.
- No `underive` — types don't un-declare mid-program.
- No multi-arg vector `isa?` — `defclause` owns arg-dispatch.
- No full `ancestors`/`descendants` reflection — reflection layer (arc 201
  lineage), later, only when a consumer needs it.
- User-facing `derive`/`isa?` verbs — ship only when a user hierarchy beyond the
  record flavors surfaces; records' macros set edges internally for now.

---

## Cross-references

- `wat/Record.wat` — current `defrecord` (the thing being split).
- `docs/arc/2026/05/234-wat-record-hologram/` — dual-form records (the overreach
  being corrected: every record forced dual-form).
- `docs/arc/2026/05/235-records-with-rich-vsa-encodings/DESIGN.md` — VSA encoding
  richness (Thermometer/Blend/Permute); SEPARATE concern (encoding quality, not
  the flavor hierarchy). Reframed/absorbed: arc 235's "holonic richness" lands on
  the `:wat::holon::Record` flavor.
- `docs/arc/2026/05/232-defprotocol-extend-type/` — the BEHAVIOR axis (orthogonal).
- `src/types.rs` ~177 (UnionDef), ~1277 (`:wat::Record` opaque struct), ~2788
  (`expand_alias` — skips Newtype, proving Newtype is nominal/no-widening; that's
  why Newtype is the WRONG cons-cell for ancestry — its contract is
  non-substitutability).
- `src/runtime.rs` ~1298 (`declared_type_name`), ~3198 (`register_type_predicates`),
  ~16130 (`conforms_check`), ~7513 (`val_type_path` — collapses records to
  `:wat::Record`; revisit for defclause dispatch if per-class record dispatch is
  ever wanted).
- `src/check.rs` ~14586 (`unify` — the symmetric union arms; `is_subtype` goes
  beside this, the call-checking site is where accept becomes
  `is_subtype || unify`).
- Memory: `project_records_not_types` (the root cause; now SUPERSEDED in
  disposition — done in 237, not a separate arc), `project_typed_entities_doctrine`,
  `project_convergences` (add the isa?/derive leg).
