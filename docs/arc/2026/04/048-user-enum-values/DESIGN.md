# wat-rs arc 048 — user-defined enum value support

**Status:** opened 2026-04-24. Third wat-rs arc post-known-good.
Lab arc 018 surfaced the second deeper substrate gap of the
session — the first was `first` returning T-on-Vec (arc 047);
this one is **user-defined enums can be DECLARED but not
CONSTRUCTED.**

The 2026-04-19 FOUNDATION-CHANGELOG entry on `:Option<T>` is
explicit: *"`:Option<T>` is the sole built-in enum in 058-030's
grammar"* with shipped surface `:None` (nullary keyword) +
`(Some <expr>)` (tagged-1 invocation). User enums via
`(:wat::core::enum :name ...)` were SPECIFIED by 058-030 but
never IMPLEMENTED for value construction — declarations register
in `TypeEnv` but no `Value::Enum` exists, no constructor
synthesis happens, and `match` only knows Option's variants.

**Surfaced now because** lab arc 018's standard.wat needs to
construct `Candle::Phase` — which has a `PhaseLabel` field —
in test fixtures. Lab has TEN user-defined enums (Side,
Direction, Outcome, TradePhase, Prediction, ScalarEncoding,
MarketLens, RegimeLens, PhaseLabel, PhaseDirection) that have
been declarable since arc 030+ but never instantiable. The gap
was latent until arc 018's first call site needed to materialize
one.

You're right — we thought we had it. We didn't. Time to make it.

---

## What ships

### Construction syntax — mirrors Option's two shapes

**Unit variant** — bare keyword evaluates to the variant value:

```scheme
:trading::types::PhaseLabel::Valley
;; ⇒ Value::Enum { type: "PhaseLabel", variant: "Valley", fields: [] }
```

Variants are PascalCase per Rust convention ("we embody our host
language"). Lab's existing 10 enum decls migrate from lowercase
(`:valley`) to PascalCase (`:Valley`) as part of this arc.

The substrate's keyword evaluator gets a lookup table populated
at startup; the keyword dispatches if the path matches any
registered enum's unit variant.

**Tagged variant** — invocation form, fields positional:

```scheme
(:trading::types::Event::Candle 100.0 105.0)
;; ⇒ Value::Enum { type: "Event", variant: "Candle", fields: [100.0, 105.0] }
```

Each tagged variant gets an auto-synthesized function (per the
struct `/new` precedent) — `register_enum_methods` walks every
enum decl and emits per-variant constructors at keyword path
`:enum::Variant`.

The shape mirrors Rust exactly:
- Unit: `MyEnum::Variant`
- Tagged: `MyEnum::Variant(arg1, arg2)`

The `::` separator is canonical Rust namespace syntax, also our
keyword-path separator.

### Value representation — single generic variant

```rust
pub enum Value {
    ...
    Enum(Arc<EnumValue>),
}

pub struct EnumValue {
    pub type_path: String,    // ":trading::types::PhaseLabel"
    pub variant_name: String, // "valley"
    pub fields: Vec<Value>,   // empty for unit, populated for tagged
}
```

One Value variant covers every user enum. The discriminator is
`type_path` + `variant_name`; payload is `fields` (empty for
unit variants).

Built-in Option keeps its dedicated `Value::Option(Arc<...>)`
variant — no migration needed. Option is special-cased by name
(`:None` keyword, `Some` symbol) per existing implementation.
User enums use the generic `Value::Enum` variant.

### Type checking — register variants as ordinary functions

`register_enum_methods` (parallel to `register_struct_methods`)
walks every enum declaration in the `TypeEnv` and synthesizes:

- For each unit variant: register the keyword in a
  `unit_variant_constructors: HashMap<String, EnumValue>`
  lookup table the runtime checks in keyword dispatch.
- For each tagged variant: synthesize a `Function` entry at
  keyword path `<enum>::<variant>`, parameters typed per the
  variant's fields, return type `<enum>`, body invokes a new
  `:wat::core::variant` primitive.

The check phase picks up the synthesized functions through
`CheckEnv::from_symbols` — same path struct constructors take.

### Pattern matching — generalize beyond Option

`(:wat::core::match scrutinee -> :ResultType arm...)` currently
special-cases Option's bareword variants (`(Some x)`, `:None`).
Arc 048 extends to user enums via FULL-PATH variant constructors
in arm patterns (Rust + construction-symmetry — see Q&A discussion).

- Tagged-variant arm: `((:enum::Variant binder1 binder2 ...) body)`
  — head is the full-path variant constructor; positional binders
  match the variant's fields and are scoped to the body.
- Unit-variant arm: `(:enum::Variant body)` — bare keyword pattern,
  no binders.
- `_` wildcard remains for catch-all.
- Exhaustiveness check requires every variant to be covered (or
  include a wildcard arm).
- The match expression's TYPE is the unified type of all arm
  bodies (declared via `-> :Type` after the scrutinee), NOT the
  scrutinee's type.

The runtime extends `try_match_pattern` to handle `Value::Enum` —
match by `(type_path, variant_name)` pair, bind fields by position.

Example:
```scheme
;; Scrutinee is :Direction; arms return :Side; whole match is :Side
(:wat::core::match dir -> :trading::types::Side
  (:trading::types::Direction::Up   :trading::types::Side::Buy)
  (:trading::types::Direction::Down :trading::types::Side::Sell))
```

---

## Why one arc, not several

Bundling into arc 048:
- Construction syntax (unit + tagged)
- Value representation
- Type checker registration
- Pattern matching extension

Splitting would produce orphan substrate (e.g., shipping
`Value::Enum` without construction syntax means callers can't
make values). The cluster is one design, one shipment.

---

## Why not migrate Option to use Value::Enum

Option ships as `Value::Option(Arc<std::option::Option<Value>>)` —
a dedicated representation with direct mapping to Rust's
`Option<Value>`. Several substrate paths use it:
`:wat::kernel::recv` returns it; `:wat::core::get` returns it;
arc 047's `first/second/third`-on-Vec returns it.

Migrating Option to the generic `Value::Enum` would touch every
caller of `Value::Option`, with no semantic gain. Option stays
special-cased; user enums use the generic mechanism. **Two
representations coexist** — same as the substrate's
"first/second/third for tuples vs Vec" precedent (Tuple
positional accessors error on out-of-range; Vec ones return
Option per arc 047).

---

## Substrate-level consequences

After arc 048 ships:
- Every user enum declared via `(:wat::core::enum :name ...)`
  becomes constructible.
- Lab arc 018's Phase construction works; the 10 lab enums
  become usable for the first time.
- Future broker / observer / domain-event code can use enums
  for state machines (TradePhase: opening/holding/closing) and
  classifiers (Direction: up/down).
- Pattern matching on user enums becomes the natural form for
  classifier dispatch — same shape as `match holon { Atom => ... }`.

---

## Non-goals

- **Variants with named fields.** Rust supports both tuple-
  variants (`MyEnum::V(u32)`) and struct-variants
  (`MyEnum::V { x: u32 }`). 058-030's enum grammar uses tuple-
  style only (`(candle (open :f64) (close :f64))` is positional
  field declaration with names for documentation, but
  construction is positional). Arc 048 ships positional
  construction only; named-field syntax is a future arc if a
  caller surfaces.
- **Generic enums beyond Option.** `:Option<T>` is the only
  built-in parametric enum. User enums in arc 048 are
  monomorphic; if a future caller needs a parametric user enum,
  open its own arc.
- **Migrate Option to `Value::Enum`.** Discussed above; out of
  scope.
- **Rich variant introspection** (e.g., `(:enum-of value)` to
  recover the enum type). Add only if a real caller needs it.
- **Methods on enums via `define`.** Per 058-030's "function-IS-impl"
  rule, callers write per-enum functions naturally; no special
  support needed.
- **Tagged-variant patterns inside HolonAST destructuring.**
  Arc 048 covers user-enum match; HolonAST's variants
  (Atom/Bind/Bundle/...) are special-cased separately.
