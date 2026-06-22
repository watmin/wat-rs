# DESIGN-STONE — the special-form doc-contract (`if`/`let` exemplar)

**Status:** STRIKE-READY draft. Freezes the special-form half of the intrinsic
doc-contract, the way `bytes` froze the value-intrinsic half. Proven on two
forms — `if` (the `@arg`-fits shape) and `let` (the `@syntax`-carries shape).

> Companion to `DESIGN-STONE-firm-doc-contract.md` (value intrinsics) +
> `DESIGN-STONE-spec-complete.md`. This adds the SPECIAL-FORM kind. The
> registration *entry* stays constant across both kinds; the *macro* is bespoke.

---

## 1. Why a second macro at all

A value intrinsic has ONE uniform handler (`(args, span, env, sym) -> Result<Value>`)
— `#[wat_intrinsic]` sniffs its arity and emits a dispatch shim. A special form
has **no single handler**: `if` alone is dispatched at `eval_if` (runtime.rs:7018),
`eval_if_tail` (3240, TCO), `step_if` (22617, CEK), *and* inferred bespoke in
check.rs:3936. It cannot be captured by one `fn` pointer.

**Therefore the special-form macro is a doc-DECLARATION + cross-check macro, not a
handler-wrapper.** Dispatch stays exactly where it is (inline, multi-site). The
registry owns only the form's DOC + reflection record.

| | dispatch | doc / reflection |
|---|---|---|
| value intrinsic | registry (`handler: Some`) | registry |
| **special form** | **stays inline** (eval / tail / step / checker) | **registry** |

## 2. The constant registration contract (unchanged shape)

The `IntrinsicEntry`/`IntrinsicSubmission` struct is the constant both kinds share
— it is what `render-doc` / `metadata-of` / fuzzy-search / the wiki read uniformly.
Two backward-compatible additions:

- **`handler: Option<NativeHandler>`** — `Some(fn)` for intrinsics; **`None`** for
  special forms (dispatch is elsewhere). The dispatch route (`registry().lookup`)
  returns the handler only when present; a `None` entry is reflection-only.
- **`Kind::SpecialForm`** — a new variant on the EXISTING `Kind` enum
  (`intrinsic/mod.rs:47`, today `Macro|Fn|Intrinsic`; mirrors
  `:wat::runtime::Kind`). NOT a new parallel `FormKind`. The `defenum` in
  `wat/runtime-meta.wat` gains `:SpecialForm` to match (iv-c invariant).

No other field changes. A special-form entry carries the same `prose`, `added`,
`see`, `category`, `examples`, `source` fields; `args`/`ret_type` may be empty
(see §4); a new `syntax` field carries the grammar (§4).

## 3. The macro: `wat_special_form!`

Bespoke front-door in `crates/wat-macros`. Anchors on a **per-form marker item**
(a unit struct) so the `///`-on-item doc parser is reused verbatim:

```rust
/// <prose...>
/// @added 1.0.0  @category ControlFlow  @purity preserving  @determinism preserving
/// @syntax (if <cond> <then> <else>)
/// @arg cond :wat::core::Bool  the condition
/// @ret :T  the taken branch's value
/// @example (:wat::core::if :true 1 2) #=> 1
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;
```

The macro: parses the `///` firm-grammar doc, enforces the special-form marker
set (§4), and `inventory::submit!`s the entry with `handler: None`,
`kind: SpecialForm`. The unit struct is zero-cost (never instantiated); it exists
only to carry doc attrs and give `show-source` a home (§6).

## 4. The special-form marker set (what differs from intrinsics)

| marker | rule | witness |
|---|---|---|
| `@syntax` | **MANDATORY** — the grammar, `(head <slot> …)` with `<x>+`/`<x>*` | replaces the hand-built `signature: HolonAST` sketch; cross-checked against the form's inline arg-count |
| `@arg`/`@ret` | **present where slots are typed value-positions** (`if`/`and`/`or`); **omitted** where slots are structural (`let`/`match`/`quote`) | the runnable `@example` (run through check+eval) |
| `@example` | **MANDATORY** ≥1, runnable | runtime — same as intrinsics |
| `@Category` | closed enum `Category`; values `Encoding`, `Reflection`, `ControlFlow`, `Binding` (+ `Quoting`, `Definition` as forms migrate) | value ∈ `Category` variants (compile-error on unknown) |
| `@Purity` | closed enum `Purity { Pure, Effectful, Preserving }` (Option A) | value ∈ `Purity` variants; `Pure`/`Preserving` ⟺ `!is_effectful_op` |
| `@Determinism` | closed enum `Determinism { Deterministic, Nondeterministic, Preserving }` | value ∈ `Determinism` variants |
| ~~`@pure`/`@deterministic`~~ | **annihilated** — the bool markers are gone for BOTH kinds; replaced by the enum markers above | — |

### The enum-marker convention (`@<EnumName> <Variant>`, exact case)

Every CLOSED-ENUM-valued marker follows ONE grammar rule: the marker name IS the
Rust enum (`Purity`/`Determinism`/`Category`), the value IS a variant verbatim
(exact case). The CASE is the discriminator — **Capitalized marker = closed enum;
lowercase marker** (`@arg`/`@ret`/`@example`/`@added`/`@see`/`@syntax`) **=
structured/freeform**. The three enums live in `crates/wat-doc` (the leaf crate the
parser, macro, and main crate all reach, like `Arity`); the value parses straight
into the enum (no case-mapping); the cross-check is structural (value ∈ variants).

`@Purity Preserving` = "adds no effect; inherits from sub-forms" (the third value,
agreed via four-questions). This unifies VALUE intrinsics too: `bytes` re-fits
`@pure true`→`@Purity Pure`, `@deterministic true`→`@Determinism Deterministic`,
`@category Encoding`→`@Category Encoding`. The Entry's `pure: bool`/`deterministic:
bool` become `purity: Purity`/`determinism: Determinism`; `category: String`
becomes `category: Category`.

### The witness model for special forms

Special forms lack a diffable `TypeScheme` (their inference is bespoke in
check.rs), so the **runnable `@example` is the primary witness** — and it's a
STRONGER one than for intrinsics: each example runs through the FULL check+eval
pipeline, so a wrong `@ret`/`@arg` type fails to type-check and a wrong semantics
evals wrong. `@syntax`'s arity cross-checks the form's inline validation. The
build-fail-on-divergence property holds, routed through the example rather than a
scheme diff.

## 5. The two exemplars (the two shapes)

**`if` → `src/intrinsic/special/control_flow.rs`** — the `@arg`-fits shape:
```rust
/// Evaluate `cond`; when true, evaluate and return `then`, else `else`.
/// The untaken branch is never evaluated — that is why `if` is a special form.
/// @added 1.0.0  @Category ControlFlow  @Purity Preserving  @Determinism Preserving
/// @syntax (if <cond> <then> <else>)
/// @arg cond :wat::core::Bool  the condition
/// @arg then :T  returned when cond is :true
/// @arg else :T  returned when cond is :false
/// @ret :T  the taken branch's value; both branches unify to T
/// @example (:wat::core::if true 1 2)  #=> 1
/// @example (:wat::core::if false 1 2) #=> 2
//  ^ NOTE: bare `true`/`false` are BoolLit → :wat::core::Bool; `:true`/`:false`
//    are KEYWORDS and would fail `if`'s `@arg cond :Bool` check. The doc-contract's
//    example-as-witness caught this author's bug in the DESIGN draft (2026-06-22).
#[wat_special_form(":wat::core::if")] pub(crate) struct If;
```

**`let` → `src/intrinsic/special/binding.rs`** — the `@syntax`-carries / no-`@arg` shape:
```rust
/// Bind each <expr> to its <binder> in order (later see earlier), then evaluate
/// the body in the enriched scope, returning the last form.
/// @added 1.0.0  @Category Binding  @Purity Preserving  @Determinism Preserving
/// @syntax (let [<binder> <expr> ...] <body>+)
/// @ret :T  the value of the final body form
/// @example (:wat::core::let [x 1 y 2] (:wat::core::+ x y)) #=> 3
#[wat_special_form(":wat::core::let")] pub(crate) struct Let;
```

`let` carries NO `@arg` — `@syntax` is the spec for its structural slots.

## 6. Reflection on a handler-less form

- `render-doc` / `metadata-of`: read the registry entry uniformly; report
  `Kind: SpecialForm`, the `@syntax` line, prose, category, purity/determinism,
  examples. (No code change beyond reading the new fields + `Option<handler>`.)
- `show-source`: a special form has no single handler body to restringify →
  returns the **`@syntax` grammar + a pointer to the inline dispatch sites**
  (not a fn body). Honest about where the form actually lives.

## 7. The crutch that dies (qualified annihilation)

`SpecialFormDef { signature: HolonAST, doc_string: Option<String> }`
(`src/special_forms.rs`) is a HolonAST DATA crutch (the scout confirmed: a syntax
sketch is structural data, not a vector) with a deferred `doc_string: None` slot.
This strike **annihilates it**: `@syntax` (a `&'static str`) replaces the
`signature` sketch; the registry entry replaces `doc_string`. The hand-built
`build_registry` sketch + its test die. Net: one HolonAST crutch site removed,
one deferred slot closed, ZERO new HolonAST planted (every entry field is a
string literal).

## 8. Bundled mechanical step (cheap-now)

Rename `IntrinsicEntry`/`IntrinsicSubmission`/`IntrinsicRegistry` →
`FormEntry`/`FormSubmission`/`FormRegistry` (4 files: `mod.rs`, `reflect.rs`,
wat-macros `lib.rs` + `wat_intrinsic.rs`) — they hold both kinds now; the
`Intrinsic*` name is a mild lie for special forms. Done in this pass since it
touches the same files; droppable if it complicates the contract proof.

## 9. Out of scope = rejected (affirmative cuts)

- The 205-value-arm migration (rides the proven exemplars after this).
- A per-form `TypeScheme` for special forms (they have none; example is the witness).
- wat-level macro/`defn` doc (a separate mechanism entirely — user-decided).
- `@category Quoting`/`Definition` values (added when those forms migrate, not now).

## 10. North-star probe (the disconfirming test)

`wat-tests/reflect/special-form-doc-if-let.wat`: `render-doc`/`metadata-of` on
`:wat::core::if` and `:wat::core::let` return their `@syntax` + prose. **RED at
HEAD** (both unregistered → `lookup_entry` None → render-doc raises). **GREEN**
when the two entries register.
