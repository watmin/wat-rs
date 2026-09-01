//! `:wat::core::{Record/field-at,to-record,record->map,Record/assoc,Record/same-data?,
//! struct-field,struct-new,variant}` — arc 255 Stone the-record-family, ALL SEVEN aggregate
//! verbs: construction (`struct-new`, `variant`), field read (`Record/field-at`, `struct-field`),
//! conversion (`to-record`, `record->map`), and write (`Record/assoc`), plus the type-blind
//! comparison (`Record/same-data?`).
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-record-family.md`.
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-record-family.md`.
//!
//! `Record/field-at` was the first of these homed (arc 255 Stone A-2-ii-b-0); the other six join
//! it here, each a thin `#[wat_intrinsic]` delegate over its pre-existing named fn in
//! `src/runtime.rs` (all seven now `pub(crate)` so this module can reach them) — **no body
//! moves.**
//!
//! ⛔ **The struct pair (`struct-new`/`struct-field`) is IN SCOPE, not deferred.** An earlier draft
//! of this stone's DESIGN drew a five-verb fork and parked the pair on a contradiction that does
//! not exist: `accessor_meta`'s first guard (`src/rete/purity.rs`) is
//! `if !head.contains('/') { return None; }` — `struct-field` has no slash, so `accessor_meta`
//! never speaks about it; the two verbs were never in conflict. And `struct-field`'s body (read
//! whole) is a plain indexed read of an already-evaluated `Value::Aggregate` — arc 293.R2.2's own
//! comment says the old `Nature::Struct` guard it once carried "was a pre-unification artifact;
//! record + holon-record field accessors now use this same primitive." It is the unified
//! field-read every record/holon-record/struct accessor calls, not a struct-only verb. Purity is
//! a property of the VERB (same input, same output), never of what it hands back — a struct field
//! may hold a live, mutable handle, but every verb that could DO anything with that handle is
//! refused one step later by the recursive purity walk, which is where the effect actually lives.
//! See `wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat` for the measured
//! evidence (a live `Lru` handle read before and after two mutations, same object both times).
//!
//! ⚠ **`@Totality` is measured per verb here, never copied across the family** — the collection
//! readers proved why (`assoc`/`conj` were `Partial` on inner helpers a container-gate reading
//! never reaches). All seven measure `Partial`: each raises on a value inside its declared domain
//! that the domain-level (type or `Aggregate`) gate does not by itself exclude — an unregistered
//! record/struct/enum class, an unknown field/variant name, an out-of-range index, or a type
//! mismatch on a write. See each delegate's own "Totality ground" for its cited line.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::core::Record/field-at record index) -> :T` — arc 234 Stone 234.2a.
///
/// Positional accessor for a Record/HolonRecord Aggregate: returns `fields[index]`. Consumed by
/// the Stone 234.2b `defrecord` macro's per-field accessor codegen. Homed here arc 255 Stone
/// A-2-ii-b-0 with its real (2) arity declared; the hand-rolled `args.len() != 2` guard in
/// `eval_record_field_at` retires. The body is unchanged, now in `src/record/access.rs`.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an effect).
/// Past that, the body only reads the already-evaluated receiver's `fields` vec (rejecting
/// anything that is not a non-`Struct` `Aggregate`) and indexes it — no
/// `eval_inner`/`apply_function` on caller-supplied code beyond the two argument evaluations.
/// Pure ∧ Deterministic.
///
/// **Totality ground — pinned in the DESIGN, measured at the site:**
/// `eval_record_field_at`'s bounds check, `if index < 0 || (index as usize) >= fields.len()`,
/// returns `Err(RuntimeErrorKind::TypeMismatch)` on an out-of-range index — an
/// `EvalBreak::Diagnostic`, which "surfaces to user code as an error"
/// (`src/value/signal.rs`'s own doc on the variant), i.e. a raise, not a wat-level
/// `Option`/`Result` the caller can `match`. Per
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`, a raise is not a
/// matchable outcome regardless of how deterministic or well-located it is. `Partial`.
///
/// **Expand-time ground —** Pure ∧ Deterministic and safe to evaluate during expansion; a
/// `Partial` verb can still be expand-time-legal, exactly as `macros/eval.rs` says for
/// `:wat::i64::/`'s division-by-zero. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     record :wat::core::Record the receiver — a Record/HolonRecord Aggregate (not Struct)
/// @arg     index :wat::core::i64 the zero-based positional field index; raises a TypeMismatch if negative or out of bounds
/// @ret     :T the field value at `fields[index]`
/// @example (:wat::core::do (:wat::core::defrecord :probe::FieldAtExample [sk <- :wat::core::i64]) (:wat::core::Record/field-at (:probe::FieldAtExample :sk 7) 0)) #=> 7
/// @see     :wat::core::Option/expect
#[wat_intrinsic(":wat::core::Record/field-at")]
pub(crate) fn eval_record_field_at(
    record: &WatAST,
    index: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::access::eval_record_field_at(&[record.clone(), index.clone()], list_span, env, sym)
}

/// `(:wat::core::to-record x :S) -> :S$core-record` — arc 293 K3-revise.
///
/// One of the PAIR of projection verbs (the other, `:wat::holon::to-record`, produces a
/// `$holon-record` — portable EDN plus hologram; this one produces the plain `$core-record`
/// tier). Projects `x`'s surface attributes (as declared by the registered `SurfaceDef` named
/// `:S`) into a freshly-built `Record`-nature `Aggregate`: for each surface member it looks up
/// `x`'s type's accessor method (`<concrete-type>/<field>`) and applies it. Projection is
/// ONE-WAY UP: there is no `to-struct` (retired 293 K3-revise) — you already hold the struct.
///
/// **Purity ground:** arg0 `x` is evaluated by ordinary call-by-value; arg1 `:S` is a literal
/// surface keyword, never evaluated (`parse_projection_args`). The body then looks up each
/// surface member's accessor `Function` in `sym` and applies it via `apply_function` — but this
/// is NOT caller-supplied code: the accessor is the concrete type's own compiler-registered
/// projection (the same `<Type>/<field>` binding `Record/field-at`'s generated callers use),
/// resolved by name from the type registry, not passed in by the caller of `to-record` itself.
/// Applying a fixed, registry-resolved accessor is exactly the shape `Record/field-at` already
/// counts as pure. Pure ∧ Deterministic.
///
/// **Totality ground — measured, not copied.** `parse_projection_args` (`src/record/project.rs`) gates
/// arity and the type registry; `project_surface_attrs` (same file) is where the domain-level gate
/// (any `x`, any registered surface `:S`) meets a VALUE-level hole it does not close: for a
/// surface member `fname`, `sym.get(&format!("{concrete_type_fqdn}/{fname}"))` can miss — the
/// concrete type simply may not implement that surface member's accessor — and the `None` arm
/// raises `RuntimeErrorKind::UnknownFunction`. That is a raise on a value inside the declared
/// domain (a real `x`, a real registered surface), not a type the domain excludes. `Partial`, the
/// same container-gate/value-hole shape `assoc`'s Record arm carries.
///
/// **Expand-time ground —** no ambient state read beyond the type registry (already required for
/// ordinary evaluation), no effect performed. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     x :T the value being projected; its concrete type must implement every field accessor the surface declares
/// @arg     surface :wat::core::keyword a literal surface keyword (e.g. :my::Surface), NOT evaluated; must name a registered `SurfaceDef`
/// @ret     :T the freshly-built `$core-record`-tier Record Aggregate carrying the surface's fields
/// @example (:wat::core::Record/field-at (:wat::core::to-record (:wat::core::Fault/of "boom") :wat::core::Error) 0) #=> "boom"
/// @see     :wat::core::Record/field-at
#[wat_intrinsic(":wat::core::to-record")]
pub(crate) fn eval_to_core_record(
    x: &WatAST,
    surface: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::project::eval_to_core_record(&[x.clone(), surface.clone()], list_span, env, sym)
}

/// `(:wat::core::record->map record) -> (:wat::core::HashMap :- [:wat::core::keyword T])` — arc
/// 234 Stone 234.3a.
///
/// Extracts a record's field-name→value map: keys are `:wat::core::keyword`s built from the
/// registered `AggregateDef`'s declared field names, values come from `fields` by declaration
/// order. Zero-field record → empty map. The core `record->map` primitive `Record/same-data?`
/// (below) is itself built on.
///
/// **Purity ground:** the one arg is evaluated by ordinary call-by-value. Past that, the body
/// (`record_field_map`) only reads the already-evaluated receiver's `class`/`fields` and the
/// registered `AggregateDef`'s field names, building a `HashMap` — no `eval_inner`/
/// `apply_function` on caller-supplied code anywhere. Pure ∧ Deterministic.
///
/// **Totality ground — measured, not copied.** `record_field_map` (`:18125`) is the shared
/// helper both `record->map` and `Record/same-data?` call. Its domain-level gate is
/// `Value::Aggregate(a) if a.nature != Nature::Struct` — admits any record/holon-record — but
/// within that admitted domain it still looks the class up in the TypeEnv
/// (`types.get(&type_key)`) and raises `RuntimeErrorKind::MalformedForm` when the class is not
/// registered there: a hole the checker's `:wat::core::Record` umbrella param type does not
/// close (nothing about being typed `Record` guarantees the concrete class is registered).
/// `Partial`, the same container-gate/value-hole shape `assoc`'s Record arm carries.
///
/// **Expand-time ground —** reads the type registry (already required for evaluation), no
/// effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     record :wat::core::Record the receiver — a Record/HolonRecord Aggregate whose class is registered in the TypeEnv
/// @ret     (:wat::core::HashMap :- [:wat::core::keyword T]) field-name keyword → field value, one entry per declared field
/// @example (:wat::core::do (:wat::core::defrecord :probe::ToMapExample [sk <- :wat::core::i64]) (:wat::hashmap::get (:wat::core::record->map (:probe::ToMapExample :sk 3)) :sk)) #=> (:wat::core::Some 3)
/// @see     :wat::core::Record/same-data?
#[wat_intrinsic(":wat::core::record->map")]
pub(crate) fn eval_record_to_map(
    record: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::update::eval_record_to_map(std::slice::from_ref(record), list_span, env, sym)
}

/// `(:wat::core::Record/assoc record key new-value) -> :wat::core::Record` — arc 234 Stone
/// 234.3b.
///
/// Write verb in the polymorphic record-y family: returns a NEW `Value::Aggregate` (same nature)
/// with the field named by `key` replaced by `new-value`. The original record is unchanged
/// (immutable, Arc-functional). For a `HolonRecord`, the hologram is rebuilt in lockstep with the
/// positional fields (PARITY invariant).
///
/// **Purity ground:** all three args are evaluated by ordinary call-by-value. Past that, the body
/// (`record_assoc_inner`) classifies the already-evaluated receiver, resolves the field index via
/// the TypeEnv, and rebuilds a same-kind `Aggregate`/hologram — no `eval_inner`/`apply_function`
/// on caller-supplied code anywhere. Pure ∧ Deterministic.
///
/// **Totality ground — measured `Partial`, `Record/assoc` is `assoc`'s sibling and shares exactly
/// that shape (read the inner helper, not copied from `assoc`'s prose).** `record_assoc_inner`'s
/// domain-level gate is `Value::Aggregate(a) if a.nature != Nature::Struct`, but WITHIN that
/// domain it raises on two VALUE-level holes the gate does not see: `key` naming no field on the
/// record's registered class → `RuntimeErrorKind::UnknownField` (`:18319`, the
/// `record_def.field_names().position(...)` miss), and `new_val`'s type variant differing from
/// the old field's → `RuntimeErrorKind::TypeMismatch` (`:18337`, `old_type != new_type`). `Partial`.
///
/// **Expand-time ground —** reads the type registry (already required for evaluation), no
/// effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     record :wat::core::Record the receiver — a Record/HolonRecord Aggregate
/// @arg     key :wat::core::keyword the field name written; raises UnknownField if the record has no such field
/// @arg     new_value :T the value written at `key`; raises a TypeMismatch if its type variant differs from the original field's
/// @ret     :wat::core::Record a NEW record, same class/nature, with `key` bound to `new_value`; the original is unchanged
/// @example (:wat::core::do (:wat::core::defrecord :probe::AssocExample [sk <- :wat::core::i64]) (:wat::core::Record/field-at (:wat::core::Record/assoc (:probe::AssocExample :sk 1) :sk 9) 0)) #=> 9
/// @see     :wat::core::assoc
#[wat_intrinsic(":wat::core::Record/assoc")]
pub(crate) fn eval_record_assoc(
    record: &WatAST,
    key: &WatAST,
    new_value: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::update::eval_record_assoc(
        &[record.clone(), key.clone(), new_value.clone()],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::core::Record/same-data? a b) -> :wat::core::bool` — arc 237 Stone S-C.2d.
///
/// Type-BLIND record data equality: compares the field-name→value maps of two records (via the
/// same `record_field_map` helper `record->map` delegates to), ignoring class (type) AND flavor
/// (base vs holonic). Distinct from `=` (arc 238, type-strict): `Pt[x:0,y:0] same-data?
/// Coord[x:0,y:0]` → `true`.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value. Past that, the body
/// builds two field maps (`record_field_map`, twice) and delegates to `values_equal` — total map
/// equality already ruled elsewhere, not re-implemented here — no `eval_inner`/`apply_function`
/// on caller-supplied code anywhere. Pure ∧ Deterministic.
///
/// **Totality ground — measured, not inherited from `record->map` by naming alone.** This verb
/// reaches `record_field_map` (`:18125`) TWICE, once per argument — the exact same
/// unregistered-class `MalformedForm` hole `record->map` carries, hit on either `a` or `b`.
/// `Partial`.
///
/// **Expand-time ground —** reads the type registry (already required for evaluation), no
/// effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     a :wat::core::Record the first record — a Record/HolonRecord Aggregate whose class is registered in the TypeEnv
/// @arg     b :wat::core::Record the second record — same requirement
/// @ret     :wat::core::bool true iff `a` and `b`'s field-name→value maps are equal, regardless of class or flavor
/// @example (:wat::core::do (:wat::core::defrecord :probe::PtEx [sk <- :wat::core::i64]) (:wat::core::defrecord :probe::CoordEx [sk <- :wat::core::i64]) (:wat::core::Record/same-data? (:probe::PtEx :sk 0) (:probe::CoordEx :sk 0))) #=> true
/// @see     :wat::core::record->map
#[wat_intrinsic(":wat::core::Record/same-data?")]
pub(crate) fn eval_record_same_data(
    a: &WatAST,
    b: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::update::eval_record_same_data(&[a.clone(), b.clone()], list_span, env, sym)
}

/// `(:wat::core::struct-field record index) -> :T` — arc 293.R2.2.
///
/// Positional field accessor for ANY `Value::Aggregate` — record, holon-record, or struct alike.
/// The name is a fossil of the pre-unification world: arc 293.R2.2's own comment records that the
/// old `Nature::Struct` guard this primitive once carried "was a pre-unification artifact; record
/// + holon-record field accessors now use this same primitive." It is the unified field-read
/// every generated accessor calls, not a struct-only verb — see this module's header and
/// `wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat`.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value. Past that, the body
/// matches the evaluated receiver against `Value::Aggregate`, bounds-checks the index, and
/// indexes `fields` — no `eval_inner`/`apply_function` on caller-supplied code anywhere. A field
/// may itself hold something impure (e.g. a live `Lru` handle, per the scratch-pad's evidence),
/// but purity is a property of the VERB (same input, same output), never of what it hands back —
/// the handle is inert once returned, and every verb that could act on it is refused by the
/// recursive purity walk one step later. Pure ∧ Deterministic.
///
/// **Totality ground — measured, not assumed from the family.** Two raises, both on values inside
/// the declared `Aggregate` domain: a non-`Aggregate` receiver → `RuntimeErrorKind::TypeMismatch`
/// (`:16109`); an out-of-range index → `RuntimeErrorKind::MalformedForm` (`:16148`, `index >=
/// inner.fields.len()`). `Partial`.
///
/// **Expand-time ground —** Pure ∧ Deterministic, no ambient state, no effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Projection
/// @arg     record :wat::core::Record the receiver — any Aggregate (record, holon-record, or struct)
/// @arg     index :wat::core::i64 the zero-based positional field index; raises a TypeMismatch on a non-Aggregate receiver, or a MalformedForm if the index is out of range
/// @ret     :T the field value at `fields[index]`
/// @example (:wat::core::do (:wat::core::defstruct :probe::StructFieldExample [sk <- :wat::core::i64]) (:wat::core::struct-field (:probe::StructFieldExample :sk 5) 0)) #=> 5
/// @see     :wat::core::Record/field-at
#[wat_intrinsic(":wat::core::struct-field")]
pub(crate) fn eval_struct_field(
    record: &WatAST,
    index: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::access::eval_struct_field(&[record.clone(), index.clone()], list_span, env, sym)
}

/// `(:wat::core::struct-new :T field1 field2 ...) -> :T` — arc 296 G-1.
///
/// Generic struct/newtype constructor: `:T` is a literal keyword naming a registered
/// `TypeDef::Aggregate(Nature::Struct)` or `TypeDef::Newtype`; the remaining args are evaluated in
/// order and become the fields (for a `Newtype`, exactly one, positionally `<Type>/0`).
///
/// ⛔ **VARIADIC — real minimum ONE, not two.** §1 of this stone's brief predicted the same
/// `args.len() < 2` shape `variant` measures; the fn's actual first guard is `if args.is_empty()`
/// (`:15765`) — a zero-field struct (`(struct-new :T)`) is admitted. STOP-1: the arity table
/// disagreed with the read, and this is the corrected, measured fact, not the predicted one.
///
/// **Purity ground:** arg0 (`:T`) is a literal keyword, never evaluated; every remaining arg is
/// evaluated by ordinary call-by-value. Past that, the body only classifies the literal keyword,
/// looks the class up in the TypeEnv, and builds a `Value::Aggregate` — no `eval_inner`/
/// `apply_function` on caller-supplied code anywhere. Pure ∧ Deterministic.
///
/// **Totality ground — measured.** Two raises inside the declared "args present" domain: arg0 not
/// a keyword literal → `RuntimeErrorKind::MalformedForm` (`:15779`); the named class not
/// registered as a struct or newtype → `RuntimeErrorKind::MalformedForm` (`:15810`). `Partial`.
///
/// **Expand-time ground —** no ambient state beyond the type registry (already required for
/// evaluation), no effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     xs… :wat::core::Value arg0 is a literal keyword naming a registered struct/newtype type; the rest are the field values, evaluated in declaration order (exactly one for a Newtype)
/// @ret     :T the newly constructed struct/newtype Aggregate
/// @example (:wat::core::do (:wat::core::defstruct :probe::StructNewExample [sk <- :wat::core::i64]) (:wat::core::struct-field (:wat::core::struct-new :probe::StructNewExample 4) 0)) #=> 4
/// @see     :wat::core::struct-field
#[wat_intrinsic(":wat::core::struct-new")]
pub(crate) fn eval_struct_new(
    xs: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::construct::eval_struct_new(xs, list_span, env, sym)
}

/// `(:wat::core::variant :Enum :Variant field1 field2 ...) -> :Enum` — arc 048.
///
/// The internal primitive auto-synthesized tagged-variant constructors invoke: users write
/// `(:Enum::Variant arg1 arg2)`, which dispatches to a `Function` whose body is a single `variant`
/// call with the enum's type path and the variant name baked in as keyword literals. Unit
/// variants do NOT route through here — they are pre-built `EnumValue`s returned directly from
/// `SymbolTable.unit_variants`.
///
/// ⛔ **VARIADIC, real minimum TWO — confirmed against §1's table**, unlike its sibling
/// `struct-new` (STOP-1 above): the fn's first guard is `if args.len() < 2` (`:15852`),
/// exactly the minimum the brief predicted.
///
/// **Purity ground:** arg0 (enum type path) and arg1 (variant name) are literal keywords, never
/// evaluated; every remaining arg is evaluated by ordinary call-by-value and becomes a field.
/// Past that, the body only classifies the two literals, looks the enum/variant up in the TypeEnv,
/// and builds a `Value` — no `eval_inner`/`apply_function` on caller-supplied code anywhere.
/// Pure ∧ Deterministic.
///
/// **Totality ground — measured.** Raises on values inside the declared "two keyword-headed args"
/// domain that the domain-level shape does not exclude: the type path not a registered enum →
/// `RuntimeErrorKind::MalformedForm` (`:15917`); the variant name not found on that enum (and not
/// a unit variant) → `RuntimeErrorKind::MalformedForm` (`:15937`). `Partial`.
///
/// **Expand-time ground —** no ambient state beyond the type registry (already required for
/// evaluation), no effect. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     xs… :wat::core::Value arg0 the enum's type path (literal keyword), arg1 the variant name (literal keyword, leading `:` stripped), the rest are the variant's field values in declaration order
/// @ret     :T the newly constructed enum value carrying the named variant and its fields
/// @example (:wat::core::do (:wat::core::defenum :probe::VariantExample :wat::enum::Pure :V [sk <- :wat::core::i64]) (:wat::core::= (:wat::core::variant :probe::VariantExample :V 6) (:probe::VariantExample::V 6))) #=> true
/// @see     :wat::core::struct-new
#[wat_intrinsic(":wat::core::variant")]
pub(crate) fn eval_variant(
    xs: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::construct::eval_variant(xs, list_span, env, sym)
}

/// `(:wat::core::aggregate-new :T field…) -> :T` — arc 294.c.2a, arc 255 Stone
/// the-registry-answers-first-wave-3.
///
/// The ONE nature-dispatched aggregate constructor: `:T` is a literal keyword naming a
/// registered `TypeDef::Aggregate`; the remaining args are the field values, evaluated in
/// declared order. `struct-new`/`defrecord`/`holon::defrecord` all route through this (via
/// `construct_aggregate`, `src/record/construct.rs`) — the macro-expanded form a record/struct's own
/// prime constructor lowers to. Homed here with its real arity declared; the body is unchanged,
/// still `crate::record::construct::eval_aggregate_new` / `construct_aggregate`.
///
/// ⛔ **VARIADIC, real minimum ONE — read from the fn's own first guard**, `if args.is_empty()`
/// (`src/runtime.rs:17429`+), same shape `struct-new`'s own arity correction above measured, not
/// inferred from that sibling: `kwargs-construct` immediately below shares this same guard,
/// confirmed independently at its own site.
///
/// **Purity ground:** arg0 (`:T`) is a literal keyword, never evaluated; every remaining arg is
/// evaluated by ordinary call-by-value and bound into a new container — no
/// `eval_inner`/`apply_function` on caller-supplied code beyond that. It opens nothing, reads no
/// ambient state, mutates nothing. This holds regardless of the target's `Nature`: a Struct MAY
/// hold a live resource, but resource ACQUISITION is a property of how a value was obtained, not
/// of the assignment that later carries it — any acquisition is caught independently at THAT
/// op's own head by the same recursive purity walk. `validate_aggregate_containment` (check.rs)
/// additionally REJECTS STARTUP for any pure-nature aggregate declaring an impure field, so a
/// pure aggregate can never smuggle one in either way. Pure ∧ Deterministic.
///
/// **Totality ground — measured, both named gaps CLOSED, not merely unarmed:** (a) was
/// `aggregate-new`-only, a CHECKER hole — `infer_aggregate_new_check` (check.rs) now unifies the
/// supplied positional values against the declared field count, closing the "wrong arity passes
/// `--check`, raises only at runtime" gap. (b) both this verb and `kwargs-construct`, for
/// `Nature::HolonRecord` — a FREEZE-TIME hole, closed by
/// `freeze::validate_holon_record_capacity`, which rejects STARTUP for any over-budget
/// HolonRecord before any rule ever compiles; the runtime bounds check in
/// `build_holon_hologram` is now an unreachable backstop. `Total`.
///
/// **Expand-time ground —** Pure ∧ Deterministic ∧ Total; safe to evaluate during expansion.
/// Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     xs… :wat::core::Value arg0 is a literal keyword naming a registered aggregate type; the rest are the field values, evaluated in declaration order
/// @ret     :T the newly constructed aggregate
/// @example (:wat::core::do (:wat::core::defrecord :probe::AggNewExample [sk <- :wat::core::i64]) (:wat::core::Record/field-at (:wat::core::aggregate-new :probe::AggNewExample 7) 0)) #=> 7
/// @see     :wat::core::kwargs-construct
/// @see     :wat::core::struct-new
#[wat_intrinsic(":wat::core::aggregate-new")]
pub(crate) fn eval_aggregate_new_home(
    xs: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::construct::eval_aggregate_new(xs, list_span, env, sym)
}

/// `(:wat::core::kwargs-construct :T :f1 v1 :f2 v2 … | :T v1 v2 …) -> :T` — arc 294 item (C),
/// arc 255 Stone the-registry-answers-first-wave-3.
///
/// The LIVE kwargs-construction form the `defrecord`/`defstruct` companion macro emits: resolves
/// `:T`'s (splice-merged) declared field order from the registry, reorders kwargs into that
/// order, then constructs exactly like `aggregate-new` immediately above (the same
/// `construct_aggregate` tail, `src/record/construct.rs`). A positional (non-kwargs-shaped) `args[1..]`
/// passes straight through unchanged, mirroring `build_insert_fact`'s kwargs-vs-positional test.
/// Homed here with its real arity declared; the body is unchanged, still
/// `crate::record::construct::eval_kwargs_construct`.
///
/// ⛔ **VARIADIC, real minimum ONE — read from the fn's own first guard**, `if args.is_empty()`
/// (`src/runtime.rs:17584`+) — the SAME minimum `aggregate-new` measures immediately above, but
/// confirmed independently at this verb's own site, not inferred from that sibling (STOP-1: two
/// verbs sharing a guard shape is not evidence either one's guard is correct without reading
/// both).
///
/// **Purity/Determinism/Totality ground —** identical reasoning to `aggregate-new` immediately
/// above: the kwargs reorder is a pure structural rearrangement of already-literal keyword/value
/// pairs (no `eval_inner`/`apply_function` on caller-supplied code beyond ordinary argument
/// evaluation) ahead of the SAME `construct_aggregate` tail, so the same two closed gaps
/// (`infer_kwargs_construct_check`'s checker unification; `freeze::validate_holon_record_capacity`'s
/// freeze-time HolonRecord capacity check) apply here too. Pure ∧ Deterministic ∧ Total.
///
/// **Expand-time ground —** Pure ∧ Deterministic ∧ Total; safe to evaluate during expansion.
/// Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     xs… :wat::core::Value arg0 is a literal keyword naming a registered aggregate type; the rest are either kwargs (`:field value …`) or positional field values
/// @ret     :T the newly constructed aggregate
/// @example (:wat::core::do (:wat::core::defrecord :probe::KwargsConstructExample [sk <- :wat::core::i64]) (:wat::core::Record/field-at (:wat::core::kwargs-construct :probe::KwargsConstructExample :sk 9) 0)) #=> 9
/// @see     :wat::core::aggregate-new
#[wat_intrinsic(":wat::core::kwargs-construct")]
pub(crate) fn eval_kwargs_construct_home(
    xs: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::record::construct::eval_kwargs_construct(xs, list_span, env, sym)
}
