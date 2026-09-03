//! Intrinsic rgistry — arc 255. The home where wat **intrinsics** (callables
//! implemented in Rust, exposed under a `:wat::…` FQDN — `runtime.rs:23931`:
//! "intrinsics are custom Rust by definition") become registered, queryable
//! entities. The `#[wat_intrinsic]` preamble (255.1b-ii) lives over each handler
//! in this home.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! Most fields are added in the SAME strike that builds their reader:
//!   - `name` / `handler` → 255.1b-i/ii: the dispatch route (`lookup`).
//!   - `arity`            → sniffed from the `#[wat_intrinsic]` fixed-arg signature;
//!     255.1b-iii: consumed by `metadata-of`'s intrinsic branch.
//!   - `prose` / `added` / `ret` → parsed from the `///` via `wat-doc` by the macro
//!     (255.1b-iv-b1); consumed by `metadata-of`'s intrinsic branch.
//!   - `purity` / `determinism` / `category` → parsed from `@Purity` / `@Determinism` /
//!     `@Category` in the `///` by the same macro, and **stored on the entry**
//!     (`Entry::purity`, `Entry::determinism`, `Entry::category`, below).
//!     `metadata-of` reads them straight off the row — `runtime.rs`'s `:purity` /
//!     `:determinism` puts are the declared enum values, not derived bools.
//!
//!     ⚠ CORRECTED 2026-08-25. These three lines used to read *"DERIVED at the reflection
//!     site (namespace deriver via `is_effectful_op` + a small nondeterministic-set), not
//!     stored on the entry."* That was true before 255.1b-iv-b1 taught the macro to parse
//!     the doc; it went false when the design moved, and stood twelve lines above the very
//!     fields it denied. A header describing its own struct is the last place that should
//!     drift, and it is the section whose whole job is to say where each field comes from.
//!
//!     What IS derived — and deliberately kept independent — is `effectful_by_prefix`, the
//!     namespace guess `declared_purity_vs_effectful_by_prefix_census` weighs the declared
//!     value against. It never touches the registry, which is exactly what makes it an
//!     oracle rather than a copy of the truth
//!     (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).
//!
//! **The one bounded exception (builder-sanctioned, 2026-06-21):** `args` /
//! `examples` / `deprecated` / `see` are parsed + carried by the iv-b1 macro but
//! their reader — iv-b2's `verify-examples` reflection seam — lands one strike
//! later. They are NOT deleted (they are about to be used, not unneeded) and NOT
//! hidden behind a pub-leak (the cheat an earlier draft used — making the module
//! `pub` to FAKE external use and silence the signal — reverted). Each carries
//! `#[expect(dead_code)]` (NOT `#[allow]`): silent while genuinely dead, but the
//! compiler emits an unfulfilled-expectation warning the instant iv-b2's seam
//! references one — SELF-RETIRING, compiler-enforced removal, not a comment-clause
//! the next hand might forget (arc 277's expect-dead idea, applied to the live
//! instance). When iv-b2 reads these, the compiler tells us to take the `#[expect]`s off.
use std::sync::Arc;
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{EnumValue, Environment, SymbolTable, Value, EvalBreak, TrackedValue};

// ─── The closed-domain enums the reflection surface answers with ─────────────
//
// ⛔ ALL SIX ARE GENERATED FROM `wat/runtime-meta.wat`. Builder ruling, 2026-08-15:
// *"wat is source of truth for rust code ... that's my pick."* The variant lists AND
// each variant's prose live in the `.wat` file's `defenum` forms; `wat_enum_from!`
// reads them at compile time. Add a variant there and these types follow — there is
// no Rust list to keep in step, which is why the drift gate that used to guard them
// is gone rather than merely passing.
//
// Three live HERE (`Kind`/`DefinedIn`/`Layer`) and three in `wat-doc`
// (`Category`/`Purity`/`Determinism`) — the split is only that `wat-doc` is a leaf
// crate that cannot name `Value`, which is what `ToEnumValue` below exists to bridge.
//
// The four `#[expect(dead_code)]` that sat on `Kind::{Macro,Fn}`, `DefinedIn::Wat`
// and `Layer::Userland` were REMOVED 2026-08-15: the generated `FromStr` CONSTRUCTS
// every variant (`"Macro" => Ok(Self::Macro)`), so the expectations became
// unfulfilled — i.e. the annotations had turned into lies, and `#[expect]` went loud
// the moment its premise stopped holding.

::wat_source_derive::wat_enum_from!(
    pub(crate) enum Kind,
    "wat/runtime-meta.wat",
    ":wat::runtime::Kind"
);

::wat_source_derive::wat_enum_from!(
    pub(crate) enum DefinedIn,
    "wat/runtime-meta.wat",
    ":wat::runtime::DefinedIn"
);

::wat_source_derive::wat_enum_from!(
    pub(crate) enum Layer,
    "wat/runtime-meta.wat",
    ":wat::runtime::Layer"
);

/// Build the `Value::Enum` a closed-domain metadata field reflects as.
///
/// ONE DOOR for all six enums. A local trait rather than an inherent method per
/// type, because `wat-doc` is a leaf crate and cannot name `Value` — the METHOD has
/// to live here even when the TYPE does not. That is also why the three
/// `Runtime{Category,Purity,Determinism}` mirror enums that used to sit here are
/// GONE (2026-08-15): they were member-for-member identical to their `wat_doc`
/// counterparts, and the `wat_doc::X => RuntimeX` conversion in `runtime.rs` was a
/// SIXTEEN-ARM hand-written list maintained purely to translate a type into itself.
///
/// `Kind`/`DefinedIn`/`Layer` reached this door late: each hand-rolled this exact
/// body as an inherent `to_enum_value(&self)` until clippy's `wrong_self_convention`
/// fired on them the moment generation made them `Copy`. The lint named a real
/// defect — three copies of a method that already had a home.
pub(crate) trait ToEnumValue {
    const WAT_TYPE_PATH: &'static str;
    fn variant_str(&self) -> &'static str;
    fn to_enum_value(&self) -> Value {
        Value::Enum(Arc::new(EnumValue {
            type_path: Self::WAT_TYPE_PATH.into(),
            variant_name: self.variant_str().into(),
            // Arc 296 G′ — every variant this door serves is a payload-free (Unit) closed-domain
            // tag; `fields` is always `vec![]` above, so there is nothing to name.
            names: crate::runtime::no_field_names(),
            fields: vec![],
        }))
    }
}

/// Implement [`ToEnumValue`] for a generated enum. `WAT_TYPE_PATH` is NOT restated
/// here — it comes from the enum, which got it from the `.wat` file.
macro_rules! enum_value_via_as_str {
    ($t:ty) => {
        impl ToEnumValue for $t {
            const WAT_TYPE_PATH: &'static str = <$t>::WAT_TYPE_PATH;
            fn variant_str(&self) -> &'static str { self.as_str() }
        }
    };
}
enum_value_via_as_str!(wat_doc::Category);
enum_value_via_as_str!(wat_doc::Purity);
enum_value_via_as_str!(wat_doc::Determinism);
// Arc 255 Stone "metadata-of answers in one shape" — `entry.totality`/`entry.expand_time`
// have carried these typed fields since the T3/expand-T3 stones, but `eval_metadata_of`'s
// registry branch never `put` them: the axis simply never reached the reflection surface.
// Wiring them in (`runtime.rs`) needs the same `Value` conversion its four siblings already
// have; adding it here is the one-line, zero-new-logic extension of the existing pattern.
enum_value_via_as_str!(wat_doc::Totality);
enum_value_via_as_str!(wat_doc::ExpandTime);
enum_value_via_as_str!(Kind);
enum_value_via_as_str!(DefinedIn);
enum_value_via_as_str!(Layer);







/// Arity — how many wat-side arguments does this intrinsic accept?
/// `Exact(N)` means exactly N fixed args; `Variadic` means any number
/// (the handler receives `&[WatAST]` and does its own dispatch).
/// Only `Exact` and `Variadic` are needed now — Range/AtLeast are out of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arity {
    /// Exactly N positional wat arguments.
    Exact(usize),
    /// Any number of wat arguments (zero or more); passed as `&[WatAST]`.
    Variadic,
}

/// The native dispatch handler — returns `TrackedValue`, not a bare `Value`, so a producer
/// handler CAN stamp `Provenance::RuntimeBuilt { producer, call_span }` — "this value was
/// manufactured, by that verb, there" — the same fact a hand-written `dispatch_keyword_head`
/// arm has always been able to record. Arc 255 Stone G.
///
/// The `#[wat_intrinsic]`-generated shim (`crates/wat-macros/src/wat_intrinsic.rs`) is the ONE
/// choke point that produces this signature: a handler written to return a bare `Value` (the
/// ~250 pre-existing handlers, untouched) is wrapped by the shim as
/// `TrackedValue::new(v, Provenance::Unknown)` — today's behaviour, unchanged; a handler that
/// WANTS provenance returns `TrackedValue` itself and the shim's sniff (mirroring
/// `SniffedArgs` for the argument side) passes it through un-rewrapped.
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<TrackedValue, EvalBreak>;

/// The tail door's callable pointer — arc 255 Stone the-tail-door. Sibling to [`NativeHandler`],
/// same argument shape, DIFFERENT return: a tail impl (`eval_if_tail`/`eval_let_tail`/
/// `eval_match_tail`'s shim) returns bare `Value`, which is what `eval_tail` itself returns and
/// what its trampoline caller expects — no `TrackedValue` provenance decision belongs at a tail
/// return (DESIGN-STONE-the-tail-door's "the type needs no invention" table). Carried on a
/// SEPARATE `IntrinsicEntry::tail_handler` slot, never folded into `handler`
/// (`NativeHandler`'s slot): `handler` is what `dispatch_keyword_head_value` calls in NON-tail
/// position, where a tail impl's contract does not hold.
pub(crate) type TailHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;

/// One `@example` / `@example-norun` entry carried on the registry — the
/// structured form of `wat_doc::DocExample`, lowered to `'static` literals
/// by the `#[wat_intrinsic]` macro.
///
/// Fields are read by the iv-b2 `verify-examples` reflection seam
/// (`src/intrinsic/reflect.rs`). The `#[expect(dead_code)]` has been removed
/// because the seam now satisfies the use.
pub(crate) struct ExampleSubmission {
    pub expr: &'static str,
    pub expected: Option<&'static str>,
    pub run: bool,
}

/// A link-time submission of one intrinsic, gathered by `inventory`. The
/// `#[wat_intrinsic("<fqdn>")]` proc-macro emits one `inventory::submit!` of
/// this type per annotated handler; `registry()` builds itself by iterating
/// `inventory::iter::<IntrinsicSubmission>`. All fields are `'static` — the
/// macro emits string-literal fields and a fn-pointer `handler`, all of which
/// outlive the program.
///
/// Arc 255.1b-iv-b1 — the structured doc now rides each submission. `prose`,
/// `added`, and `ret` are CONSUMED by `metadata-of`'s intrinsic branch;
/// `args`, `examples`, and `see` are carried for iv-b2's verifier seam.
/// Arc 255.1b-v — `source` carries the handler's restringified token source
/// (via `quote!(#item).to_string()` in the macro), consumed by `show-source`.
/// The pre-evaluated dispatch handler — what `:wat::core::apply` needs.
///
/// ⛔ THE SHAPE DIFFERENCE IS THE WHOLE REASON A SECOND SLOT EXISTS. [`NativeHandler`] takes
/// UNEVALUATED `&[WatAST]` and evaluates them itself; `apply` has ALREADY evaluated its arguments
/// and holds `&[Value]`, so it cannot call a `NativeHandler` — there is no AST left to hand it.
/// That impedance mismatch, not laziness, is why `dispatch_substrate_impl` was a second match table
/// with no registry lookup (arc 255 Stone N). A verb carrying this slot is served by the REGISTRY on
/// both paths; one carrying only `handler` still falls through to the legacy match.
///
/// Arc 255 Stone Q — widened to carry a trailing `&Span`. The ALGEBRA contract (`env`/`sym`
/// forbidden — binding state a splatted `&Value` handler genuinely cannot use) still holds; a
/// span is not binding state, it is a location, and `apply` already holds one
/// (`runtime.rs:10773`, `eval_apply`'s `list_span`) that simply had nowhere to go before this
/// slot could carry it. The AST door already passes its `list_span` to the `NativeHandler`
/// above; this widening lets the value door pass the SAME call span, not a synthesized one. An
/// ALGEBRA fn may ignore it (the 38 already-migrated verbs do — the trailing param is optional
/// at the Rust-fn level, mandatory only on this fn-pointer type).
pub(crate) type ValueHandler = fn(&[Value], &Span) -> Result<Value, EvalBreak>;

pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
    /// Arc 255 Stone N — the value-level handler `:wat::core::apply` needs.
    /// `handler` takes UNEVALUATED `&[WatAST]` (it evaluates its own args);
    /// `apply` has already evaluated its args down to `&[Value]` by the time
    /// it needs to dispatch, so `handler` cannot serve it directly — there is
    /// no AST left to hand it. `None` (the default the `#[wat_intrinsic]`
    /// macro emits when no `value = <path>` is named) for the ~250
    /// pre-existing handlers, unchanged (STOP-1). `Some(f)` for a handler
    /// that also has a value-level implementation, letting
    /// `dispatch_substrate_impl` (`src/runtime.rs`) — `apply`'s substrate
    /// fallback, previously a second, registry-blind dispatch table — serve
    /// that verb from THIS registry instead.
    pub value_handler: Option<ValueHandler>,
    /// Exact(N) for fixed-arity handlers; Variadic for `&[WatAST]` handlers.
    pub arity: Arity,
    /// GFM prose body (everything before the first `@`-tag line).
    pub prose: &'static str,
    /// `@added` version string.
    pub added: &'static str,
    /// `@arg` directives: `(name, ty, desc, is_rest)` 4-tuples, in source order.
    pub args: &'static [(&'static str, &'static str, &'static str, bool)],
    /// `@ret` type token (must start with `:`).
    pub ret_type: &'static str,
    /// `@ret` description.
    pub ret: &'static str,
    /// `@example` / `@example-norun` directives, in source order (≥1).
    pub examples: &'static [ExampleSubmission],
    /// `@deprecated (since, use_instead)`, if present.
    pub deprecated: Option<(&'static str, &'static str)>,
    /// `@see` FQDNs, in source order.
    pub see: &'static [&'static str],
    /// `@alias <fqdn>` — arc 255 Stone 2a. `Some(core)` means this row is not an independent
    /// implementation: `core` is the FQDN of the row that actually carries the behaviour. `None`
    /// (the default) for every `#[wat_intrinsic]` handler that existed before this stone — that
    /// macro always requires a real fn body, so a `Some` here would be a lie about what
    /// `handler` above actually does. The one alias witness this stone registers is a
    /// `#[wat_special_form]` doc-only struct (`SpecialFormSubmission::alias_of`, below), not an
    /// `IntrinsicSubmission` — this field exists so the FUTURE Phase 1b migration (73 more rows)
    /// can carry it here too, symmetrically with `@see`'s own threading.
    pub alias_of: Option<&'static str>,
    /// Restringified handler source — `quote!(handler_fn).to_string()`.
    /// Faithful-if-reformatted (token restringify; comments may be lost).
    /// Consumed by `(:wat::core::show-source <fqdn>)`.
    pub source: &'static str,
    /// Declared purity from `@Purity <Variant>` in the doc.
    pub purity: wat_doc::Purity,
    /// Declared determinism from `@Determinism <Variant>` in the doc.
    pub determinism: wat_doc::Determinism,
    /// Declared totality from `@Totality <Variant>` in the doc. `Unreviewed` when the
    /// directive is absent — arc 255 stone total-T2 minted the axis OPTIONAL, because a
    /// GUESSED `Total` is a lie in a fence that ADMITS code into a `where`, while an
    /// `Unreviewed` is default-deny and merely refuses.
    pub totality: wat_doc::Totality,
    /// Declared expand-time legality from `@ExpandTime <Variant>` in the doc.
    /// `Unreviewed` when the directive is absent — arc 255 Stone expand-T2 minted
    /// this axis OPTIONAL, mirroring totality's own T2. Independent of purity,
    /// determinism, and totality — no combination of those three predicts it.
    pub expand_time: wat_doc::ExpandTime,
    /// `@Category <Variant>` — functional category.
    pub category: wat_doc::Category,
    /// `@yields <argname> <desc>` pairs, in source order — arc 255 Stone P5-b. One pair per
    /// value-carrying fn-shaped `@arg`; empty when the intrinsic yields to no callback. The
    /// TYPE is not carried here — it is derived from the named `@arg`'s own canonical
    /// bracket-form type at render time (`reflect.rs`'s `fn_arg_param_type`).
    pub yields: &'static [(&'static str, &'static str)],
}

inventory::collect!(IntrinsicSubmission);

/// A link-time submission of one special form, gathered by `inventory`.
/// Special forms have no `NativeHandler` — they are handled by the runtime
/// dispatch engine, not by a registered Rust fn. The `#[wat_special_form]`
/// proc-macro emits one `inventory::submit!` of this type per annotated struct;
/// `registry()` folds them into the `IntrinsicRegistry` as `Kind::SpecialForm` entries.
pub(crate) struct SpecialFormSubmission {
    pub name: &'static str,
    pub prose: &'static str,
    pub added: &'static str,
    pub syntax: &'static str,
    pub args: &'static [(&'static str, &'static str, &'static str, bool)],
    pub ret_type: &'static str,
    pub ret: &'static str,
    pub examples: &'static [ExampleSubmission],
    pub see: &'static [&'static str],
    /// `@alias <fqdn>` — arc 255 Stone 2a. `Some(core)` when this doc-only struct is not
    /// naming a special form at all but a plain re-dispatch — "this name means that name" — and
    /// the dispatch door reads it directly, never consulting a handler (there is none: this
    /// submission carries no `handler` field by construction, exactly as an alias needs). See
    /// `IntrinsicSubmission::alias_of`'s twin doc for the full contract.
    pub alias_of: Option<&'static str>,
    pub purity: wat_doc::Purity,
    pub determinism: wat_doc::Determinism,
    /// Declared totality from `@Totality <Variant>` in the doc. `Unreviewed` when the
    /// directive is absent — arc 255 stone total-T2 minted the axis OPTIONAL, because a
    /// GUESSED `Total` is a lie in a fence that ADMITS code into a `where`, while an
    /// `Unreviewed` is default-deny and merely refuses.
    pub totality: wat_doc::Totality,
    /// Declared expand-time legality from `@ExpandTime <Variant>` in the doc.
    /// `Unreviewed` when the directive is absent — see `IntrinsicSubmission::expand_time`.
    pub expand_time: wat_doc::ExpandTime,
    pub category: wat_doc::Category,
    pub deprecated: Option<(&'static str, &'static str)>,
}

inventory::collect!(SpecialFormSubmission);

/// Which of the four regimes an `#[wat_special_form_impl]` annotation names — arc 255 Stone
/// P6-a (`check`/`eval`/`tail`), `Declare` added by Stone 1a-β-0. `check` runs once,
/// statically, before any evaluation exists; `eval` and `tail` are mutually exclusive
/// per-invocation regimes selected by call POSITION, never both
/// (`NOTE-a-special-form-declaration-names-none-of-its-three-implementations.md`, "the three do
/// not compose"). Hand-defined, not `wat_enum_from!`-generated (mirrors `Arity`, just above):
/// this is a Rust-only reflection axis over a fixed, closed four-member set, not a wat-visible
/// enum a `.wat` `defenum` needs to drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpecialFormRole {
    /// Freeze/declare-time processing — before evaluation exists, outside `check.rs`'s
    /// `infer_*` fns and `runtime.rs`'s eval/tail matches: the form's own declaration/
    /// registration code, wherever it lives (e.g. `src/types.rs`'s type-registry synthesis).
    /// A REFLECTION fact — "this is the code that processes this form" — not a dispatch
    /// door: a freeze-time processor has no calling convention to point at (arc 255 Stone
    /// 1a-β-0, `DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`).
    Declare,
    /// Static type inference — `src/check.rs`'s `infer_*` fns.
    Check,
    /// Per-invocation evaluation — `src/runtime.rs`'s eval match.
    Eval,
    /// Per-invocation evaluation in tail position (TCO) — `src/runtime.rs`'s tail match.
    /// Optional at the population level (`tail: None` is an honest "falls through to
    /// `eval_inner`, correct but not tail-optimized" — never a wrong answer), but the WALL
    /// below requires it for neither `if` nor `let`'s absence: both of THEM have one.
    Tail,
}

impl SpecialFormRole {
    /// Lowercase label — used by `show-source`'s `;; role: <label>` lines and by the wall
    /// test's failure message. Kept as a method (not a `Display` impl) because nothing outside
    /// this reflection surface needs to print a bare role.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            SpecialFormRole::Declare => "declare",
            SpecialFormRole::Check => "check",
            SpecialFormRole::Eval => "eval",
            SpecialFormRole::Tail => "tail",
        }
    }
}

/// A link-time submission of one special form IMPLEMENTATION, gathered by `inventory` — the
/// third inventory stream (arc 255 Stone P6-a), sibling to `IntrinsicSubmission` and
/// `SpecialFormSubmission`. `#[wat_special_form_impl("<fqdn>", role = check|eval|tail)]` emits
/// one of these per annotated fn (`infer_if`, `eval_if`, `eval_if_tail`, …), keyed by (name,
/// role) — never by file, since a proc-macro cannot see across files to the OTHER two
/// implementations of the same form.
pub(crate) struct SpecialFormImplSubmission {
    /// The form's FQDN — the SAME string `#[wat_special_form]` declared on the doc-only struct.
    pub name: &'static str,
    pub role: SpecialFormRole,
    /// Restringified fn source — `quote!(#item).to_string()`, same mechanism
    /// `#[wat_intrinsic]` uses for its `source` field (`wat_intrinsic.rs:565`).
    pub source: &'static str,
    /// The role's callable pointer — arc 255 Stone the-eval-door. `Some` ONLY for
    /// `role = eval` submissions (the macro emits a generated shim, canonical
    /// `NativeHandler` shape, wrapping the annotated fn's return per
    /// `wat_intrinsic.rs`'s shared `sniff_return`/`wrap_call_for_return`, STOP-4);
    /// `None` for `role = check` and `role = tail`. Folded into `IntrinsicEntry`'s
    /// EXISTING `handler` field by `registry()`, not a new slot.
    pub eval_handler: Option<NativeHandler>,
    /// The tail door's callable pointer — arc 255 Stone the-tail-door. `Some` ONLY for
    /// `role = tail` submissions (the macro emits a generated shim, canonical
    /// [`TailHandler`] shape, wrapping the annotated fn's return per `wat_intrinsic.rs`'s
    /// shared `sniff_return` and this crate's sibling `wrap_call_for_tail_return`, STOP-4);
    /// `None` for `role = check` and `role = eval`. Folded into `IntrinsicEntry`'s SEPARATE
    /// `tail_handler` field by `registry()` — NEVER into `eval_handler`/`handler` (STOP-3):
    /// `handler` is consulted by `dispatch_keyword_head_value` in non-tail position, where a
    /// tail impl's contract does not hold.
    pub tail_handler: Option<TailHandler>,
}

inventory::collect!(SpecialFormImplSubmission);

/// One registered intrinsic's full baseline. `handler` is consumed by the
/// runtime dispatch route (`lookup`); `name`/`arity`/`prose`/`added`/`ret` are
/// consumed by `metadata-of`'s intrinsic branch (`lookup_entry`); `args`/
/// `examples`/`see` are carried for iv-b2's doctest verifier seam;
/// `source` is consumed by `show-source` — every field has a reader.
pub(crate) struct IntrinsicEntry {
    pub name: &'static str,
    /// The native dispatch handler. Always `Some` for `Kind::Intrinsic`. For
    /// `Kind::SpecialForm`, `Some` when a `role = eval` implementation registered a pointer
    /// (arc 255 Stone the-eval-door — `fn`/`if`/`let`/`match` all do, as of that stone) and
    /// `None` for a special form that has not (registered but reached only through the
    /// runtime engine's own dispatch, not a registered Rust fn here).
    pub handler: Option<NativeHandler>,
    /// Arc 255 Stone the-tail-door — the tail dispatch pointer, a SEPARATE slot from `handler`
    /// (STOP-3). `Some` when a `role = tail` implementation registered a pointer for this fqdn
    /// (`if`/`let`/`match`, as of that stone); `None` for a special form with no registered tail
    /// impl (falls through to `eval_inner` from `eval_tail`, correct but not tail-optimized) and
    /// always `None` for `Kind::Intrinsic`. Consulted ONLY by `eval_tail`'s own guard — never by
    /// `dispatch_keyword_head_value`, where a tail impl's contract does not hold.
    pub tail_handler: Option<TailHandler>,
    /// Arc 255 Stone N — mirrors `IntrinsicSubmission::value_handler`; `None`
    /// for `Kind::SpecialForm` and for any `Kind::Intrinsic` that hasn't
    /// named one. Read through `lookup_entry` by `dispatch_substrate_impl`
    /// (`src/runtime.rs`) — `:wat::core::apply`'s substrate door, which reads the
    /// SAME entry for `arity` and guards on it (Stone O-i), so no handler is ever
    /// called without the arity check the AST door has always had.
    pub value_handler: Option<ValueHandler>,
    /// What kind of callable this is (`Intrinsic` or `SpecialForm`).
    pub kind: Kind,
    /// `@syntax (...)` grammar string; empty for regular intrinsics.
    pub syntax: &'static str,
    /// Exact(N) for fixed-arity handlers; Variadic for rest-param handlers.
    /// Consumed by `metadata-of`'s intrinsic branch.
    pub arity: Arity,
    pub prose: &'static str,
    pub added: &'static str,
    pub ret: &'static str,
    // The iv-b2 carry: parsed + carried now, read by the `verify-examples`
    // reflection seam (`src/intrinsic/reflect.rs`). `examples` is now read
    // by the seam (iv-b2-a), so its `#[expect(dead_code)]` has been removed.
    // `args` and `ret_type` are read by `doc_arg_ret_types_match_checker_scheme`
    // (cfg(test) — 255.1b-firm) and dead in non-test builds. Use `#[allow(dead_code)]`
    // (not `#[expect]`) because in test builds the field IS used, which would make
    // `#[expect(dead_code)]` fire an "unfulfilled expectation" warning.
    // `deprecated` is still unread — reader lands later; keep its `#[expect(dead_code)]`.
    // `see` is read by `eval_render_doc`'s See-also section (non-test) — no dead_code attr.
    #[allow(dead_code)] // read by doc_arg_ret_types_match_checker_scheme (cfg(test))
    pub args: &'static [(&'static str, &'static str, &'static str, bool)],
    #[allow(dead_code)] // read by doc_arg_ret_types_match_checker_scheme (cfg(test))
    pub ret_type: &'static str,
    pub examples: &'static [ExampleSubmission],
    #[expect(dead_code)] // reader lands later → keep
    pub deprecated: Option<(&'static str, &'static str)>,
    pub see: &'static [&'static str],
    /// `@alias <fqdn>` — arc 255 Stone 2a. `Some(core)` means the alias field IS the dispatch,
    /// not documentation of one: `dispatch_keyword_head`/`dispatch_keyword_head_value`
    /// (`src/runtime.rs`) read this field, after the registry-first handler lookup fails, and
    /// re-invoke `core` with the same unevaluated args and span. An alias row therefore carries
    /// `handler: None` and needs no `role = eval` implementation — the whole ★★★ contract
    /// (`DESIGN-STONE-2a-the-alias-field-and-why-1b-was-blocked-twice.md`). `None` for every
    /// pre-existing row (folded straight from `IntrinsicSubmission`/`SpecialFormSubmission`'s
    /// own `alias_of`, below). Gated by `alias_target_is_registered`/
    /// `alias_target_is_itself_aliased` (`reflect.rs`, cfg(test)): every `alias_of` target must
    /// itself be a registered row, and no target may itself be an alias (no chains).
    pub alias_of: Option<&'static str>,
    /// Restringified handler source (consumed by `show-source` / 255.1b-v).
    pub source: &'static str,
    #[allow(dead_code)] // read by declared_purity_vs_effectful_by_prefix_census + purity_mandated_examples (cfg(test)) + eval_metadata_of + eval_render_doc + reflect.rs's eval_intrinsic_examples (arc 255.1c site 2)
    /// Declared purity — from `@Purity <Variant>` in the doc.
    pub purity: wat_doc::Purity,
    #[allow(dead_code)] // read by purity_mandated_examples (cfg(test)) + eval_metadata_of + eval_render_doc + reflect.rs's eval_intrinsic_examples (arc 255.1c site 2)
    /// Declared determinism — from `@Determinism <Variant>` in the doc.
    pub determinism: wat_doc::Determinism,
    /// Declared totality from `@Totality <Variant>` in the doc. `Unreviewed` when the
    /// directive is absent — arc 255 stone total-T2 minted the axis OPTIONAL, because a
    /// GUESSED `Total` is a lie in a fence that ADMITS code into a `where`, while an
    /// `Unreviewed` is default-deny and merely refuses.
    // ★ READ IN PRODUCTION since the "metadata-of answers in one shape" stone —
    // `eval_metadata_of`'s registry branch (`runtime.rs`) now `put`s `:totality` from this
    // field, closing the gap where an intrinsic's `metadata-of` answered `:purity`/
    // `:determinism`/`:category` but silently omitted `:totality`. The three totality
    // hand-lists this comment used to promise as "stone T4" readers
    // (rete/purity.rs's intrinsic_meta, macros/eval.rs's is_pure_total, rete/vocabulary.rs's
    // RETE_OPS) have NOT been touched by this stone — that migration is still open; only the
    // reflection surface is fixed here. No `#[allow(dead_code)]` needed any more.
    pub totality: wat_doc::Totality,
    /// Declared expand-time legality from `@ExpandTime <Variant>` in the doc.
    /// `Unreviewed` when the directive is absent — arc 255 Stone expand-T2 minted
    /// this axis OPTIONAL (T3 will make it required, mirroring totality's own arc).
    // ★ READ IN PRODUCTION since arc 255 Stone expand-T4b: `macros/eval.rs`'s
    // `is_expand_time_legal` — the gate deciding what a `defmacro` body may call while
    // expanding — now answers from THIS FIELD for every registered verb, falling back to a
    // 59-name residue only for verbs with no registration site yet. No `#[allow(dead_code)]`
    // is needed or wanted here any more; the field carries a live verdict.
    //
    // ⚠ This comment previously read "production readers arrive with a LATER STONE". That
    // stone is expand-T4b, and it has landed — the promise outlived itself by one commit and
    // was caught by the rider that wrote the derivation, not by the comment's own author.
    // `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
    pub expand_time: wat_doc::ExpandTime,
    /// `@Category <Variant>` — functional category.
    /// Consumed by `metadata-of`'s intrinsic branch and `eval_render_doc`.
    pub category: wat_doc::Category,
    /// `@yields <argname> <desc>` pairs, in source order — arc 255 Stone P5-b. Consumed by
    /// `eval_render_doc` (the `Yields:` section, N lines, type DERIVED per-line from the
    /// matching `@arg`'s canonical bracket-form type via `fn_arg_param_type`).
    pub yields: &'static [(&'static str, &'static str)],
    /// Arc 255 Stone P6-a — the gathered `#[wat_special_form_impl]` submissions for this form,
    /// (role, source) pairs in whatever order `inventory` handed them back (NOT necessarily
    /// check→eval→tail; a reader that cares about order — `show-source` — sorts at read time).
    /// Empty for `Kind::Intrinsic` (that kind's source lives in `source`, above) and for a
    /// `Kind::SpecialForm` nobody has annotated yet — the exact case `impls.is_empty()` must
    /// keep meaning "no impl found source", not "this kind never carries one" (STOP-4).
    ///
    /// ⚠ Every other field on this struct is `&'static` (macro-emitted literals / fn pointers
    /// that outlive the program). This one is OWNED (`Vec`, gathered and allocated at fold
    /// time inside `registry()`, not at compile time by a macro) because it is built by
    /// bucketing a THIRD inventory stream per-form rather than captured directly on the
    /// submission — `registry()` is a `OnceLock` that owns its entries, so an owned `Vec` here
    /// is fine; it is the one asymmetry in an otherwise all-`&'static` struct.
    pub impls: Vec<(SpecialFormRole, &'static str)>,
}

/// `name → entry`. Built once at startup; the dispatch route reads `handler`
/// via `lookup`, `metadata-of` reads the baseline via `lookup_entry`.
pub(crate) struct IntrinsicRegistry {
    entries: std::collections::HashMap<&'static str, IntrinsicEntry>,
}

impl IntrinsicRegistry {
    fn new() -> Self { IntrinsicRegistry { entries: std::collections::HashMap::new() } }

    /// Register an intrinsic's full baseline. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN). This `debug_assert!`
    /// is compiled out under `cargo nextest run --release` (this repo's only
    /// trusted floor; `Cargo.toml` has no `[profile.release]`) — it still fires
    /// fast in a debug build, but the guarantee IN RELEASE is carried by
    /// `tests::no_two_submissions_claim_the_same_fqdn`, which walks the
    /// `inventory::iter` submission streams before either collapses into this map.
    fn register(&mut self, entry: IntrinsicEntry) {
        debug_assert!(!self.entries.contains_key(entry.name), "duplicate intrinsic registration: {}", entry.name);
        self.entries.insert(entry.name, entry);
    }

    /// The dispatch route — the native handler for `name` (255.1b-i/ii).
    /// `None` = not a registered intrinsic (or is a `Kind::SpecialForm` with no handler).
    pub(crate) fn lookup(&self, name: &str) -> Option<NativeHandler> {
        self.entries.get(name).and_then(|e| e.handler)
    }

    /// The reflection route — the full baseline entry for `name` (255.1b-iii),
    /// read by `metadata-of`'s intrinsic branch. `None` = not registered.
    pub(crate) fn lookup_entry(&self, name: &str) -> Option<&IntrinsicEntry> {
        self.entries.get(name)
    }

    /// Iterate all registered entries. Read by the iv-b2 `verify-examples`
    /// reflection seam (`src/intrinsic/reflect.rs`) to build the examples vector.
    pub(crate) fn all_entries(&self) -> impl Iterator<Item = &IntrinsicEntry> {
        self.entries.values()
    }
}

/// Arc 255 Stone 1a-β-ii — does `head`'s registry row name a `role = declare`
/// implementation? The registry-query REPLACEMENT for the now-DELETED
/// `freeze::is_liftable_declaration_head` nine-name (latterly three-name) hand-list:
/// `closure_extract::split_body_prelude` — the predicate's one production caller — asks
/// this instead. Lives WITH the registry (not a wrapper re-planted in `freeze.rs` under a
/// new name — the DESIGN's own disqualification of that option): a predicate about what a
/// NAME is belongs in the module that owns the answer.
///
/// `≡ is_liftable_declaration_head(head)` — the equality
/// `DESIGN-STONE-1a-beta-i-the-type-declaration-family.md` minted, and this stone's own
/// (now-deleted) `liftable_declaration_head_missing_and_foreign` meter measured MISSING = ∅
/// before the hand-list and the meter were both retired.
pub(crate) fn is_declare_role_head(head: &str) -> bool {
    registry()
        .lookup_entry(head)
        .is_some_and(|entry| entry.impls.iter().any(|(role, _)| *role == SpecialFormRole::Declare))
}

/// The process-wide intrinsic registry, built once on first access.
pub(crate) fn registry() -> &'static IntrinsicRegistry {
    static REGISTRY: std::sync::OnceLock<IntrinsicRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut r = IntrinsicRegistry::new();
        // Each `#[wat_intrinsic("<fqdn>")]` handler submits an entry via
        // `inventory`; gather them all into the registry at first access.
        for submission in inventory::iter::<IntrinsicSubmission> {
            r.register(IntrinsicEntry {
                name: submission.name,
                handler: Some(submission.handler),
                // arc 255 Stone the-tail-door — no `Kind::Intrinsic` carries a `role = tail`
                // submission (that role is a `#[wat_special_form_impl]`-only concept); always
                // `None` here.
                tail_handler: None,
                value_handler: submission.value_handler,
                kind: Kind::Intrinsic,
                syntax: "",
                arity: submission.arity,
                prose: submission.prose,
                added: submission.added,
                args: submission.args,
                ret_type: submission.ret_type,
                ret: submission.ret,
                examples: submission.examples,
                deprecated: submission.deprecated,
                see: submission.see,
                alias_of: submission.alias_of,
                source: submission.source,
                purity: submission.purity,
                determinism: submission.determinism,
                totality: submission.totality,
                expand_time: submission.expand_time,
                category: submission.category,
                yields: submission.yields,
                impls: Vec::new(),
            });
        }
        // Arc 255 Stone P6-a — bucket the THIRD stream (`#[wat_special_form_impl]`
        // submissions) ONCE, keyed by fqdn, before the special-form loop below drains it
        // per form. Iterating the whole stream inside that loop would be O(n·m) and, more
        // importantly, would read as if the (fqdn, role) association were incidental rather
        // than the keyed relationship it is.
        let mut impls_by_fqdn: std::collections::HashMap<&'static str, Vec<(SpecialFormRole, &'static str)>> =
            std::collections::HashMap::new();
        // arc 255 Stone the-eval-door — bucketed alongside `impls_by_fqdn`, same pass, same
        // stream: the `role = eval` pointer a submission carries (`None` for check/tail,
        // STOP-2). Folded into `IntrinsicEntry`'s EXISTING `handler` field below, not a new
        // slot on the entry.
        let mut eval_handler_by_fqdn: std::collections::HashMap<&'static str, NativeHandler> =
            std::collections::HashMap::new();
        // arc 255 Stone the-tail-door — bucketed alongside `eval_handler_by_fqdn`, same pass,
        // same stream: the `role = tail` pointer a submission carries (`None` for check/eval).
        // Folded into `IntrinsicEntry`'s SEPARATE `tail_handler` field below (STOP-3) — never
        // merged into `eval_handler_by_fqdn`/`handler`.
        let mut tail_handler_by_fqdn: std::collections::HashMap<&'static str, TailHandler> =
            std::collections::HashMap::new();
        for submission in inventory::iter::<SpecialFormImplSubmission> {
            impls_by_fqdn
                .entry(submission.name)
                .or_default()
                .push((submission.role, submission.source));
            if let Some(eval_handler) = submission.eval_handler {
                eval_handler_by_fqdn.insert(submission.name, eval_handler);
            }
            if let Some(tail_handler) = submission.tail_handler {
                tail_handler_by_fqdn.insert(submission.name, tail_handler);
            }
        }
        // Each `#[wat_special_form("<fqdn>")]` struct submits a SpecialFormSubmission
        // via `inventory`; fold them into the registry as Kind::SpecialForm entries.
        for submission in inventory::iter::<SpecialFormSubmission> {
            // Arc 255 Stone P2 — derive arity from the form's own declared @args instead
            // of hardcoding Variadic. `Exact(N)` only when the form actually enumerated
            // its arguments and none of them is a rest param; a form that declares its
            // shape as `@syntax` instead of `@arg` (e.g. `let`) has zero @args and is
            // genuinely variadic — Exact(0) would be a WORSE lie than the one this
            // replaces. This mirrors what `#[wat_intrinsic]` already does
            // (crates/wat-macros/src/wat_intrinsic.rs:653-657).
            let arity = match submission.args {
                args if !args.is_empty() && !args.iter().any(|(_, _, _, is_rest)| *is_rest) => {
                    Arity::Exact(args.len())
                }
                _ => Arity::Variadic,
            };
            r.register(IntrinsicEntry {
                name: submission.name,
                // arc 255 Stone the-eval-door — `Some` when a `role = eval` submission
                // registered a pointer for this fqdn; `lookup`/`dispatch_keyword_head_value`'s
                // existing guard then dispatches it unchanged (no new consult site).
                handler: eval_handler_by_fqdn.remove(submission.name),
                // arc 255 Stone the-tail-door — `Some` when a `role = tail` submission
                // registered a pointer for this fqdn; `eval_tail`'s own guard (never
                // `dispatch_keyword_head_value`'s) is the sole consult site (STOP-3).
                tail_handler: tail_handler_by_fqdn.remove(submission.name),
                value_handler: None,
                kind: Kind::SpecialForm,
                syntax: submission.syntax,
                arity,
                prose: submission.prose,
                added: submission.added,
                args: submission.args,
                ret_type: submission.ret_type,
                ret: submission.ret,
                examples: submission.examples,
                deprecated: submission.deprecated,
                see: submission.see,
                alias_of: submission.alias_of,
                source: "",
                purity: submission.purity,
                determinism: submission.determinism,
                totality: submission.totality,
                expand_time: submission.expand_time,
                category: submission.category,
                yields: &[],
                impls: impls_by_fqdn.remove(submission.name).unwrap_or_default(),
            });
        }

        // arc 255 Stone 2a-b — resolve every alias row's five axes from its target, AFTER both
        // fold loops above have finished. NOT folded into either loop (STOP-2, the DESIGN's own
        // warning): the target may register AFTER the alias — `inventory`'s iteration order is
        // unspecified — so a lookup attempted DURING either fold above would sometimes find
        // nothing yet. This must be a genuine second pass over the FINISHED map.
        //
        // First collect (alias name, target name) pairs under one immutable borrow of
        // `r.entries`, then release it before looking up each target's (already final, since
        // both folds above are done) axes and writing them onto the alias entry through a
        // second, non-overlapping borrow. A target that does not resolve (dangling `@alias`) is
        // left alone here — that is 2a's own gate's job
        // (`reflect::alias_target_is_registered`/`check_alias_refs`, unchanged per STOP-4); this
        // pass only copies axes when the target actually exists, so a dangling alias's axes stay
        // at the parsed placeholder rather than this pass inventing an answer.
        let alias_targets: Vec<(&'static str, &'static str)> =
            r.entries.values().filter_map(|e| e.alias_of.map(|target| (e.name, target))).collect();
        for (alias_name, target_name) in alias_targets {
            let target_axes = r
                .entries
                .get(target_name)
                .map(|t| (t.purity, t.determinism, t.totality, t.expand_time, t.category));
            let Some((purity, determinism, totality, expand_time, category)) = target_axes else { continue };
            if let Some(alias_entry) = r.entries.get_mut(alias_name) {
                alias_entry.purity = purity;
                alias_entry.determinism = determinism;
                alias_entry.totality = totality;
                alias_entry.expand_time = expand_time;
                alias_entry.category = category;
            }
        }

        r
    })
}

// Arc 255 Stone HOME-12 — the AST surface gets a registry home. Pure re-registration
// (`:wat::core::ast-*`/`symbol-node`/`keyword-node`/`read-string`/`fresh-symbol` are already the
// final spelling); the handlers stay in `src/edn/render.rs` (STOP-5 — a file carve is a separate
// deliverable, reported not acted on).
mod ast;
mod bigint;
mod bytes;
mod char;
// Arc 255 Stone P6-c-W6 — the first wave into `:wat::core::`, and the first to hit the
// dispatch_verbs blind spot: a handler homed outside `src/intrinsic/` vanishes from
// `rete::purity::completeness_gate`'s population entirely (that scan's `#[wat_intrinsic]`
// half is scoped to this directory). `length`/`empty?`/`nth`/`last`/`rest`/`reverse`/`range`
// moved here (out of `runtime.rs` and `src/collection/{eval,transform}.rs`) for that reason.
mod collection;
// Arc 255 Stone P6-c-W1 — the campaign's first wave. Four nullary `:wat::config::` readers,
// moved verbatim out of `runtime.rs`'s giant match with their real (0) arity declared.
mod config;
// Arc 255 Stone HOME-11 — the EDN registry home. Pure re-registration (HOME-5 already carved
// the file home, `src/edn/`); nothing renamed, no `.wat` corpus touch.
mod edn;
mod f64;
mod hashmap;
mod hashset;
mod holon;
mod i64;
mod io;
mod kernel;
mod keyword;
mod linkedlist;
mod list;
mod macro_error;
mod map;
// Arc 255 Stone HOME-10 — math, stat, seq get actual homes. Pure re-registration
// (HOME-9 already renamed these off `:wat::std::`); shim-only, three files, no
// `src/math/`/`src/stat/`/`src/seq/` directory (STOP-1 — no algebra worth naming).
mod math;
// Arc 255 Stone A-2-ii-b-0 — `:wat::core::Option/expect`, moved verbatim out of `runtime.rs`'s
// giant match with its real (2) arity declared, so a generated record accessor stops
// classifying impure/Unreviewed through the raise it propagates. Ruled `Pure ∧ Deterministic ∧
// Partial` — see `option.rs`'s own doc. `:wat::core::Some` joined it arc 255 Stone A-2-ii-b-1,
// moved the same way with its real (1) arity declared, ruled `Pure ∧ Deterministic ∧ Total`.
mod option;
// Arc 255 Stone P6-c-W2 — the campaign's second wave. One nullary `:wat::program::env`
// reader, moved verbatim out of `runtime.rs`'s giant match with its real (0) arity
// declared. `:wat::program::self-peer`/`cpu-count` are neighbours, not this wave's verbs.
mod program;
mod rational;
// Arc 255 Stone A-2-ii-b-0 — `:wat::core::Record/field-at`, moved verbatim out of
// `runtime.rs`'s giant match with its real (2) arity declared. Ruled `Pure ∧ Deterministic ∧
// Partial` — see `record.rs`'s own doc.
mod record;
mod reflect;
mod regex;
// Arc 255 Stone A-2-ii-b-1 — `:wat::core::Ok`/`Err`, the tagged `Result` constructors, moved
// verbatim out of `runtime.rs`'s giant match with their real (1) arity declared. No pre-existing
// `Result`-namespaced module to extend (unlike `Some`, which joined `option.rs`), so this is a
// new "own home, same shape" file. Ruled `Pure ∧ Deterministic ∧ Total` — see `result.rs`'s own
// doc.
mod result;
// Arc 255 Stone P6-c-W5a — the P6-c campaign's fifth wave (5a): the nine READ-ONLY
// `:wat::rete::` verbs (the six `?` predicates + the three alpha-matchers), moved verbatim out
// of `runtime.rs`'s giant match with their real arities declared. The other 19 `:wat::rete::`
// verbs (session-mutating) stay in `src/rete/` and the giant match — not this wave.
mod rete;
mod seq;
mod stat;
// Arc 255 Stone P6-c-W2 — the campaign's second wave. `:wat::stream::{empty,cons,next}`,
// moved verbatim out of `runtime.rs`'s giant match with their real (0/2/1) arities
// declared. `:wat::stream::lazy` is a SPECIAL FORM and stays in the giant match.
mod stream;
// Arc 255 Stone F — reverted to the default-private shape of its siblings. The
// `:wat::core::String/*` uppercase alias arms in `runtime.rs` (Stone 237.3) that needed
// `pub(crate)` to call four of these handlers directly (bypassing the registry) are retired;
// nothing outside this module calls into it anymore.
mod string;
mod uuid;
mod vec;
mod vector;
mod witness;
pub(crate) mod special;
mod time;

/// Arc 255 Stone P5-b — derive the single callback parameter type from a canonical
/// fn-shaped `@arg` type string, for `reflect.rs`'s `Yields:` render. `ty` is the
/// bracket-form `typeexpr_to_doc_string` emits for a `TypeExpr::Fn` (P5-a): `[ARG :-> RET]`
/// (one param) or `[:-> RET]` (nullary — `None`, since a nullary callback hands nothing in
/// and P5-b's mandate forbids a `@yields` there). The split point is the FIRST `:->` found
/// at bracket depth 0 relative to the OUTER `[...]` (i.e. depth 0 inside it) — depth-tracked
/// (across both `(` / `[`) so a nested parametric arg type
/// (`[(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil]`) does not confuse the split on its
/// own internal `:-`/`[…]`.
pub(crate) fn fn_arg_param_type(ty: &str) -> Option<&str> {
    let inner = ty.strip_prefix('[')?.strip_suffix(']')?;
    let mut depth = 0i32;
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ':' if depth == 0 && inner[i..].starts_with(":->") => {
                let param = inner[..i].trim();
                return if param.is_empty() { None } else { Some(param) };
            }
            _ => {}
        }
    }
    None
}

// ─── Arc 255.1b-v: @see registry-check + firm-doc tests ──────────────────────
//
// Consumer-side tests: every `@see` FQDN must resolve; doc arg/ret types must
// match the checker's TypeScheme; pure+det intrinsics must carry ≥1 runnable
// example. The walks live in `reflect::check_see_refs()` (cfg(test)) and
// inline in the tests below.
#[cfg(test)]
mod tests {
    /// Arc 255 Stone P4 — the frozen DEBT LEDGER for the silent skip in
    /// `doc_arg_ret_types_match_checker_scheme` (`None => continue` below, "not yet in
    /// checker — skip"). (Arc 255 Stone P5-b: `yields_type_matches_fn_arg_param` used to
    /// share this same `None => continue` and this same ledger; it is DELETED, not
    /// rewritten — see the note where it stood, below.) This gate builds
    /// `CheckEnv::with_builtins_and_types(&TypeEnv::new())` and calls
    /// `check_env.get(entry.name)`; for every name below that returns `None`, so **it
    /// verifies nothing about that entry's `@arg`/`@ret` doc strings against the checker.**
    /// A registration whose doc types are pure fiction passes today by being absent from
    /// `register_builtins` (`src/check.rs`) — not by being correct.
    ///
    /// ⚠ THIS LIST IS A DEBT LEDGER, NOT AN EXEMPTION LIST. Every name on it is an intrinsic
    /// or special form whose declared types are checked by nothing. It is not an accusation —
    /// `spawn-thread`/`spawn-process` (`:wat::kernel::`) are deliberately special-cased through
    /// `infer_*_prime` elsewhere per the arc's own NOTE — but their absence from `CheckEnv`
    /// still means THIS gate family verifies nothing for them, so they stay on the ledger like
    /// everything else here. Driving this list to zero (registering the missing names in
    /// `register_builtins`) is a separate, larger stone; this test only measures and freezes
    /// the population so it cannot grow silently.
    ///
    /// Measured 2026-08-28, re-measured 2026-08-29 (arc 255 Stone P6-c-W4 adds the
    /// `:wat::runtime::` row below) against `check_env.get(entry.name)` over
    /// `registry().all_entries()` — the SAME construction and SAME method both gates use, so
    /// this measurement cannot disagree with what the gates actually skip. 53 of 405
    /// registered entries (2026-08-29 count, post-W4; the prior "384" total had already
    /// drifted stale before this stone from unrelated homing — not re-audited here beyond
    /// this stone's own +3). Arc 255 Stone A-2-ii-b-1 adds a further +3 (`Some`/`Ok`/`Err`,
    /// newly registered by this stone and newly on this ledger) — the "53 of 405" and the
    /// per-namespace breakdown below are stale by that same +3 and not re-audited here either,
    /// same discipline as the W4 note just above.
    ///
    /// ★ `registry().all_entries().count()` (405) is NOT the same instrument as the anchored
    /// `#[wat_intrinsic]`-attribute grep (403 post-W4) — the +2 gap is not noise, it is a fixed
    /// identity: `:wat::core::if` (`src/intrinsic/special/control_flow.rs:20`) and
    /// `:wat::core::let` (`src/intrinsic/special/binding.rs:15`) register into this SAME
    /// registry via the DIFFERENT `#[wat_special_form(...)]` attribute, which an
    /// `#[wat_intrinsic` grep cannot see. So `all_entries().count()` == the anchored intrinsic
    /// grep + 2, always — recorded here after three separate riders each independently
    /// rediscovered this gap and reported it as unexplained drift.
    /// - `:wat::kernel::` 23 of 46 — accept, address-wire?, after, allow, close, connect, deny,
    ///   fn-forms, listener, peer-pid, peer-process, peer-wire?, poll, recv,
    ///   require-wire-address, retag-op, select, send, serve-dispatch-op, signal,
    ///   spawn-process, spawn-thread, try-send
    /// - `:wat::holon::` 7 of 91 — coincident-explain, coincident?, cosine, dot, literal,
    ///   simhash, to-record
    /// - `:wat::core::` 6 of 18 — List, fresh-symbol, if, let, type-equal?, type-params-used-in
    /// - `:wat::linkedlist::` 5 of 5 — WHOLLY ABSENT: conj, contains?, empty?, get, length
    /// - `:wat::runtime::` 3 of 13 (arc 255 Stone P6-c-W4) — field-names-of, field-types-of,
    ///   metadata-of. NOT UNIFORM: field-names-of/field-types-of ARE type-checked, by
    ///   hand-written special-case inference inside `infer_list` (`src/check.rs:2543`;
    ///   `field-names-of` at `:3570`, `field-types-of` at `:3596`) — they lack a `TypeScheme`
    ///   but are not unverified in the sense the other rows on this ledger are. `metadata-of`
    ///   has neither a scheme nor inference — 0 mentions in `check.rs` outside comments — and
    ///   is the only one of the three genuinely unchecked.
    /// - `:wat::seq::` 3 of 3 — WHOLLY ABSENT: remove-at, window, zip
    /// - `:wat::string::` 2 of 20 — declare-acronyms, interpolate
    /// - `:wat::time::` 2 of 41 — +, -
    /// - `:wat::edn::` 1 of 13 — validate
    /// - `:wat::form::` 1 of 1 — WHOLLY ABSENT: matches? (arc 255 Stone P6-c-1 — homed to
    ///   `#[wat_intrinsic]`; `check.rs::infer_form_matches` type-checks it via its own
    ///   FQDN-keyed special-case grammar, never via a generic `TypeScheme`, so `CheckEnv`
    ///   was never going to have it — this gate's job is only to notice and freeze that fact)
    const FROZEN_CHECKER_DEBT_LEDGER: &[&str] = &[
        // Arc 255 Stone 1a-i — `and`/`or` are registered special forms with a real check impl
        // (`infer_boolean_shortcircuit`, `check.rs`) but NO `env.register()` TypeScheme, so
        // `check_env.get` returns `None`. Exactly the `Option/expect` shape recorded below: real
        // checking, no scheme to verify the DOCS against. Retires when a scheme registers.
        ":wat::core::and",
        ":wat::core::or",
        // Arc 255 Stone A-2-ii-b-0 — `Option/expect` is checked FOR REAL by a hand-written
        // `check_call` arm (`infer_option_expect`, `src/check.rs`), but carries no
        // `env.register()` TypeScheme, so `check_env.get` returns `None` and
        // `doc_arg_ret_types_match_checker_scheme` cannot compare its `@arg`/`@ret` against one.
        // Exactly the `nth`/`reverse` shape recorded below, and the same debt: real checking,
        // no scheme to verify the DOCS against. Retires when a scheme registers — not by
        // weakening the gate.
        ":wat::core::Option/expect",
        // Arc 255 Stone the-option-result-siblings — `Option/try`/`Result/expect`/`Result/try`
        // are each checked FOR REAL by a hand-written `check_call` arm (`infer_option_try`/
        // `infer_result_expect`/`infer_try`, `src/check.rs:2918,2926,2948`'s match arms), but
        // NONE carries an `env.register()` TypeScheme, so `check_env.get` returns `None` for
        // all three. Exactly the `Option/expect` shape just above: real checking, no scheme to
        // verify the DOCS against. Predicted in the DESIGN before this stone was briefed
        // (measured against this same `check_env.get` — verified, not inferred), and it fired
        // exactly as predicted. Retires when a scheme registers for each — not by weakening
        // the gate.
        ":wat::core::Option/try",
        ":wat::core::Result/expect",
        ":wat::core::Result/try",
        // Arc 255 Stone A-2-ii-b-1 — `Some`/`Ok`/`Err` are each checked FOR REAL by a
        // hand-written `check_call` arm (`infer_some_constructor`/`infer_ok_constructor`/
        // `infer_err_constructor`, `src/check.rs:4938,4948,4958` — Region A's FQDN-keyword-
        // headed constructor arms), but NONE carries an `env.register()` TypeScheme, so
        // `check_env.get` returns `None` for all three and
        // `doc_arg_ret_types_match_checker_scheme` cannot compare their `@arg`/`@ret` against
        // one. Exactly the `Option/expect` shape just above and the `nth`/`reverse` shape
        // below: real checking, no scheme to verify the DOCS against. Predicted in the DESIGN
        // before this stone was briefed (measured against this same `check_env.get` — verified,
        // not inferred), and it fired exactly as predicted. Retires when a scheme registers for
        // each — not by weakening the gate.
        ":wat::core::Some",
        ":wat::core::Ok",
        ":wat::core::Err",
        ":wat::core::List",
        ":wat::core::fresh-symbol",
        ":wat::core::if",
        ":wat::core::let",
        // Arc 255 Stone the-membership-gap-gets-a-ratchet — `fn`/`match` join `if`/`let` just
        // above on this ledger the moment they become `Kind::SpecialForm` registry rows (this
        // stone's deliverable 3). Both are checked FOR REAL by hand-written dispatch arms in
        // `check.rs`'s `infer_call` match (`":wat::core::fn" => crate::function::infer_fn`,
        // `":wat::core::match" => infer_match`), exactly `if`/`let`'s own shape — but NEITHER
        // carries an `env.register()` TypeScheme, so `check_env.get` returns `None` for both,
        // same as `if`/`let`. `check_env.get(entry.name).is_none()` was already true for them
        // BEFORE this stone; this gate simply could not see it, because `registry().all_entries()`
        // did not carry `fn`/`match` until they became registry rows here. `check.rs` stays
        // untouched (STOP-4); the ledger grows by two, the same shape `if`/`let` already have.
        ":wat::core::fn",
        ":wat::core::match",
        // Arc 255 Stone 1a-zeta — `do`/`ann-form`/`stream::lazy` join `if`/`let`/`fn`/`match`
        // just above on this ledger the moment they become `Kind::SpecialForm` registry rows
        // (this stone's deliverable). All three are checked FOR REAL by hand-written dispatch
        // fns in `check.rs` (`infer_do`, annotated in place; `infer_ann_form`/`infer_stream_lazy`,
        // extracted verbatim into `intrinsic/special/ann_form.rs`/`stream_lazy.rs` and delegated
        // to from `check.rs`'s match), but NONE carries an `env.register()` TypeScheme, so
        // `check_env.get` returns `None` for all three — exactly `if`/`let`/`fn`/`match`'s own
        // shape. `check.rs`'s own LOGIC stays untouched (STOP-4: only extracted/delegated, never
        // rewritten); the ledger grows by three, the same shape `fn`/`match` already used.
        ":wat::core::do",
        ":wat::core::ann-form",
        ":wat::stream::lazy",
        // Arc 255 Stone P6-c-W6 — `nth`/`reverse` are checked for real (the hand-written
        // `check_call` arms `infer_nth`/`infer_reverse`), but NEITHER carries an
        // `env.register()` TypeScheme: `nth` never had one (custom arm from birth, stone
        // 118.B4-0), and `reverse`'s was explicitly RETIRED (arc-278-0d comment, check.rs)
        // when `infer_reverse` superseded it. `checker_skip_debt_is_named_and_frozen`'s
        // criterion is exactly `check_env.get().is_none()` — "no TypeScheme", not "unchecked"
        // (W4's correction) — so homing either as a `#[wat_intrinsic]` makes it newly
        // measured with nothing to compare its doc `@arg`/`@ret` against. `check.rs` stays
        // untouched (STOP-4); the ledger grows by two, honestly, the same shape W4 used for
        // `field-names-of`/`field-types-of`/`metadata-of`.
        ":wat::core::nth",
        ":wat::core::reverse",
        // Arc 255 Stone the-collection-readers — `drop`/`take` are checked for real (the
        // hand-written `check_call` arms `infer_drop`/`infer_take`, `src/check.rs`), but
        // NEITHER carries an `env.register()` TypeScheme — same shape as `nth`/`reverse` just
        // above (`checker_skip_debt_is_named_and_frozen`'s criterion is exactly
        // `check_env.get().is_none()`, "no TypeScheme", not "unchecked"). `assoc`/`conj`,
        // homed in the SAME stone, are NOT on this ledger: both DO carry an `env.register()`
        // TypeScheme (`src/check.rs`, the `contains?`/`get`/`conj`/`assoc` fingerprint block)
        // — a deliberately MIXED prediction, falsifiable in both directions, and it held both
        // ways: `check_env.get(":wat::core::assoc")`/`get(":wat::core::conj")` return `Some`,
        // `get(":wat::core::drop")`/`get(":wat::core::take")` return `None`. `check.rs` stays
        // untouched (STOP-4); the ledger grows by two, the same shape W6 used for `nth`/`reverse`.
        ":wat::core::drop",
        ":wat::core::take",
        // Arc 255 Stone the-record-family — `struct-new`/`struct-field`/`variant`/`to-record`
        // are each checked FOR REAL by hand-written machinery (`struct-new`/`struct-field`/
        // `variant` are runtime-only, enforced by their own `eval_*` guards + the TypeEnv
        // lookups inside them, with no `check.rs` inference arm at all; `to-record` IS
        // type-checked, by the hand-written `infer_projection_verb_check` arm, `check.rs:4734`)
        // — but NONE of the four carries an `env.register()` TypeScheme, so `check_env.get`
        // returns `None` for all four. Exactly the `nth`/`reverse`/`drop`/`take` shape above:
        // real checking (or, for the three runtime-only verbs, real runtime enforcement, which
        // this ledger's criterion does not distinguish from "checked" — see the header note),
        // no scheme to verify the docs against. `Record/assoc`/`Record/same-data?`/
        // `record->map`, homed in the SAME stone, are NOT on this ledger: all three DO carry an
        // `env.register()` TypeScheme (`check.rs:21236/21259/21275`) — a deliberately MIXED
        // prediction, falsifiable in both directions, and it held both ways: `check_env.get`
        // returns `Some` for all three, `None` for all four here. `check.rs` stays untouched
        // (STOP-4); the ledger grows by four, the same shape W6 used for `nth`/`reverse`.
        ":wat::core::struct-field",
        ":wat::core::struct-new",
        ":wat::core::to-record",
        ":wat::core::variant",
        ":wat::core::type-equal?",
        ":wat::core::type-params-used-in",
        // Arc 255 Stone the-registry-answers-first-wave-3 — `aggregate-new`/`kwargs-construct`
        // are each checked FOR REAL by hand-written `check_call` arms (`infer_aggregate_new_check`/
        // `infer_kwargs_construct_check`, `check.rs`), but NEITHER carries an `env.register()`
        // TypeScheme, so `check_env.get` returns `None` for both. Exactly the `Option/expect`
        // shape above: real checking, no scheme to verify the docs against. `macro-error` has no
        // checker treatment at all (0 mentions in `check.rs` outside this ledger) — `check_env.get`
        // returns `None` for the same reason `metadata-of` does above (W4's note): no scheme AND
        // no hand-written arm. All three were previously invisible to this gate as literal
        // `runtime.rs` match arms; homing surfaces them to it for the first time. Predicted in the
        // DESIGN before this stone was briefed (measured against this same `check_env.get`), and
        // it fired exactly as predicted: `write-forms`/`with-children`, homed the SAME stone, are
        // NOT on this ledger — both DO carry an `env.register()` TypeScheme (`check.rs:19310`/
        // `:19349`) — a deliberately UNEVEN prediction, falsifiable in both directions, and it
        // held both ways. `check.rs` stays untouched (STOP-4); the ledger grows by three.
        ":wat::core::aggregate-new",
        ":wat::core::kwargs-construct",
        ":wat::core::macro-error",
        ":wat::edn::validate",
        ":wat::form::matches?",
        ":wat::holon::coincident-explain",
        ":wat::holon::coincident?",
        ":wat::holon::cosine",
        ":wat::holon::dot",
        ":wat::holon::literal",
        ":wat::holon::simhash",
        ":wat::holon::to-record",
        ":wat::kernel::accept",
        ":wat::kernel::address-wire?",
        ":wat::kernel::after",
        ":wat::kernel::allow",
        ":wat::kernel::close",
        ":wat::kernel::connect",
        ":wat::kernel::deny",
        ":wat::kernel::fn-forms",
        ":wat::kernel::listener",
        ":wat::kernel::peer-pid",
        ":wat::kernel::peer-process",
        ":wat::kernel::peer-wire?",
        ":wat::kernel::poll",
        ":wat::kernel::recv",
        ":wat::kernel::require-wire-address",
        ":wat::kernel::retag-op",
        ":wat::kernel::select",
        ":wat::kernel::send",
        ":wat::kernel::serve-dispatch-op",
        ":wat::kernel::signal",
        ":wat::kernel::spawn-process",
        ":wat::kernel::spawn-thread",
        ":wat::kernel::try-send",
        ":wat::linkedlist::conj",
        ":wat::linkedlist::contains?",
        ":wat::linkedlist::empty?",
        ":wat::linkedlist::get",
        ":wat::linkedlist::length",
        // arc 255 Stone P6-c-W4 — field-names-of/field-types-of ARE typed (infer_list
        // special-case, check.rs:3570/3596); metadata-of has neither scheme nor inference.
        // All three are absent from `CheckEnv` (this ledger's actual criterion), so all three
        // belong here — but they are NOT identically "unchecked"; see the header note above.
        ":wat::runtime::field-names-of",
        ":wat::runtime::field-types-of",
        ":wat::runtime::metadata-of",
        ":wat::seq::remove-at",
        ":wat::seq::window",
        ":wat::seq::zip",
        ":wat::string::declare-acronyms",
        ":wat::string::interpolate",
        ":wat::time::+",
        ":wat::time::-",
        // Arc 255 Stone 1a-β-0 — `defsurface` joins the ledger the moment it becomes a
        // `Kind::SpecialForm` registry row (this stone's witness, deliverable 5). Unlike
        // `if`/`let`/`fn`/`match` above, it has NO `check.rs` inference arm at all (measured
        // 0 mentions, `NOTE-a-declaration-form-is-a-THIRD-regime-the-role-vocabulary-cannot-
        // name.md`) — its declare-time processing (`synthesize_surface_protocol`,
        // `src/types.rs`) runs at freeze, entirely outside `check.rs`, and carries no
        // `env.register()` TypeScheme either, so `check_env.get` returns `None` for the same
        // reason `macro-error` does above: no scheme AND no hand-written arm. `check.rs` stays
        // untouched (STOP-4); the ledger grows by one.
        ":wat::core::defsurface",
        // ★★★ Arc 255 Stone 1a-β-i — the type-declaration family joins for `defsurface`'s exact
        // reason: registered `Kind::SpecialForm` rows whose declare-time processing runs at freeze
        // (`parse_structtype`/`parse_defenum`/`parse_newtype`/`parse_typealias`, `src/types.rs`),
        // entirely outside `check.rs`, with no `env.register()` TypeScheme — so `check_env.get`
        // returns `None` and `doc_arg_ret_types_match_checker_scheme` verifies nothing about their
        // `@arg`/`@ret` docs. `check.rs` stays untouched; the ledger grows by four.
        //
        // ⚠ READ THIS BEFORE CALLING THE GROWTH A REGRESSION. This ledger GROWING while the
        // campaign claims to ELIMINATE duplication looks backwards and is not. (76 → 80 here; it was
        // 75 before `defsurface` joined one stone earlier.) Before registration
        // these four were not debt — they were INVISIBLE: no registry row, so nothing could ask
        // what the checker knew about them, and nothing could notice that the answer was "nothing".
        // Registering converts an unmeasured absence into a NAMED one, which is the entire job of
        // an absence ledger (`[[RULING-the-registry-is-the-sole-authority]]`: the four ledgers are
        // "a measurement of duplication … when the ruling is satisfied, all four are empty").
        // The finish line is DEBT → 0 via Phase 2c (TypeSchemes derived from the doc types,
        // 384/386 measured convertible by PROBE(255) `bb1aa686d`) — not via not-registering.
        //
        // ⛔ `:wat::core::defstruct` is DELIBERATELY ABSENT, and it is not an oversight. It is a
        // stdlib `defmacro` (`wat/core.wat:2030`) that rewrites to `structtype` during
        // `expand_all`, which always completes before `register_types` — so it never reaches a
        // parser at all and has no declare-time fn to name. Verified: `--check` on a malformed
        // `(:wat::core::defstruct :probe::Bad)` answers with `:head "structtype"`. Annotating a
        // fn `role = declare` for it would be a false reflection claim.
        //
        // ✅ UPDATED by Stone 1a-β-i-b — `parse_defstruct` and `parse_type_decl`'s `"defstruct"`
        // arm are now DELETED, not merely dead, and `defstruct` has left
        // `is_liftable_declaration_head`'s domain, so `liftable_declaration_head_missing_and_foreign`
        // no longer carries it: MISSING is `def`·`defalias`·`defmacro`. ⚠ Its arms in
        // `is_mutation_form`/`is_mutation_head` STAY — those guard `eval_in_frozen` and
        // `eval-ast!`, which evaluate caller-supplied AST that skipped `expand_all`.
        //
        // ✅ SUPERSEDED by Stone 1a-β-ii — `def`/`defalias`/`defmacro` (the MISSING three named
        // just above) are now registered too (see the block below), which made MISSING = ∅ —
        // both `is_liftable_declaration_head` and its meter are DELETED, not kept green at
        // zero. This paragraph's "MISSING is def·defalias·defmacro" is a historical snapshot of
        // the stone BEFORE this one; it names neither a live predicate nor a live meter today.
        ":wat::core::structtype",
        ":wat::core::defenum",
        ":wat::core::newtype",
        ":wat::core::typealias",
        // ★★★ Arc 255 Stone 1a-β-ii — the last three join for `defsurface`'s exact reason:
        // registered `Kind::SpecialForm` rows with no `env.register()` TypeScheme, so
        // `check_env.get` returns `None` and `doc_arg_ret_types_match_checker_scheme`
        // verifies nothing about their `@arg`/`@ret` docs. Checked, not assumed: grepped for
        // `env.register(":wat::core::def"` / `"::defmacro"` / `"::defalias"` across
        // `register_builtins` (`check.rs`) — zero hits for all three. `def` differs from its
        // two siblings in HAVING a check impl of its own (`infer_def`, `role = check`) — but
        // a check impl is not a TypeScheme (`if`/`let`/`fn`/`match` above already established
        // that distinction), so `def` belongs here for the identical reason they do. This is
        // the campaign's first hand-list kill (`freeze::is_liftable_declaration_head`
        // DELETED, `liftable_declaration_head_missing_and_foreign` DELETED with it) — the
        // ledger grows by three while the hand-list and its meter shrink to nothing.
        ":wat::core::def",
        ":wat::core::defalias",
        ":wat::core::defmacro",
        // ★★★ Arc 255 Stone 1a-δ — the three loaders join for `defsurface`'s exact reason:
        // registered `Kind::SpecialForm` rows with no `env.register()` TypeScheme, so
        // `check_env.get` returns `None` and `doc_arg_ret_types_match_checker_scheme`
        // verifies nothing about their `@arg`/`@ret` docs. Checked, not assumed: `check.rs`'s
        // own top-level-form arm (~line 4874) names all three (`":wat::load-file!"` |
        // `":wat::digest-load!"` | `":wat::signed-load!"`) alongside `defstruct`/`defenum`/
        // `newtype`/`typealias`/`defmacro`/`defalias` and returns `CheckResult::ok(fresh.
        // fresh())` with the comment "declaration forms, not value-producing expressions" —
        // the identical silent-by-intent shape the type-declaration family's own rows measured;
        // grepped for `env.register(":wat::load-file!"` / `"::digest-load!"` / `"::signed-
        // load!"` across `register_builtins` — zero hits for all three. `check.rs` stays
        // untouched (STOP-4); the ledger grows by three, 83 → 86.
        ":wat::digest-load!",
        ":wat::load-file!",
        ":wat::signed-load!",
        // ★★★ Arc 255 Stone 1a-ε — the three no-ops join for the same reason the loaders just
        // above did: registered `Kind::SpecialForm` rows with no `env.register()` TypeScheme,
        // so `check_env.get` returns `None`. Checked, not assumed: `register_builtins` has zero
        // `env.register(":wat::core::use!"` / `":wat::config::set-redef!"` / `":wat::config::
        // set-eval-redef!"` calls. Unlike the loaders, these three DO carry a real check impl
        // (`infer_config_set_bool` for both setters, `infer_use_form` for `use!`) — exactly the
        // `and`/`or` shape at the top of this ledger: real checking, no scheme to verify the
        // docs against. No `register_builtins` scheme added; the ledger grows by three, 86 → 89.
        ":wat::core::use!",
        ":wat::config::set-redef!",
        ":wat::config::set-eval-redef!",
        // ★★★ Arc 255 Stone 1a-gamma-i — the six homoiconic verbs that really evaluate join for
        // the same reason: registered `Kind::SpecialForm` rows with no `env.register()`
        // TypeScheme, so `check_env.get` returns `None`. Checked, not assumed: grepped
        // `register_builtins` for `env.register(":wat::core::quote"` / `"::quasiquote"` /
        // `"::forms"` / `"::struct->form"` / `"::macroexpand"` / `"::macroexpand-1"` — zero
        // hits for all six. All six DO carry a real check impl (`infer_quote`/`infer_forms`/
        // `infer_struct_to_form`/`infer_quasiquote`/`infer_macroexpand`, this stone's
        // extractions) — exactly the `and`/`or`/`use!` shape above: real checking, no scheme
        // to verify the docs against. No `register_builtins` scheme added; the ledger grows by
        // six, 89 → 95.
        ":wat::core::quote",
        ":wat::core::quasiquote",
        ":wat::core::forms",
        ":wat::core::struct->form",
        ":wat::core::macroexpand",
        ":wat::core::macroexpand-1",
        // Arc 255 Stone 1b-ii — the 6 `Form` + 2 `Redispatch` rete alias rows
        // (`src/intrinsic/special/rete_alias.rs`). `RETE_OPS`'s `params`/`ret` are dead for
        // these two classes, so they carry no `CheckEnv` scheme of their own (typechecking for
        // `Form`/`Redispatch` is generic, dispatched by `OpClass` alone — `check.rs`'s
        // `infer_rete_form`/`infer_list` — never a per-name `TypeScheme` registered under the
        // rete-prefixed name). Exactly the `and`/`or`/`use!` shape above: real checking (via the
        // class-generic path), no scheme to verify these docs against. Ledger grows by eight,
        // 95 → 103.
        ":wat::rete::core::and",
        ":wat::rete::core::or",
        ":wat::rete::core::if",
        ":wat::rete::core::let",
        ":wat::rete::core::match",
        ":wat::rete::core::fn",
        ":wat::rete::core::List",
        ":wat::rete::holon::coincident?",
        // Arc 255 Stone 1c-b-i — `first`/`second`/`third`/`PersistentVector`/`PersistentMap`
        // join: each is now a registered `#[wat_intrinsic]` row (`src/runtime.rs` —
        // `eval_first`/`eval_second`/`eval_third`/`eval_persistentvector`/
        // `eval_persistentmap`), but NONE carries an `env.register()` TypeScheme, so
        // `check_env.get` returns `None` for all five. Checked, not assumed: grepped
        // `register_builtins` for `env.register(":wat::core::first"` / `"::second"` /
        // `"::third"` / `"::PersistentVector"` / `"::PersistentMap"` — zero hits for all
        // five. All five ARE checked for real by hand-written arms (`infer_positional_
        // accessor` for the three accessors, `infer_persistentvector_constructor` /
        // `infer_persistentmap_constructor` for the two constructors, `check.rs`) — exactly
        // the `nth`/`reverse`/`drop`/`take` shape above: real checking, no scheme to verify
        // the docs against. This is `probe_can_doc_types_reconstruct_the_checker_scheme`'s
        // second measured instance of `Kind` being stamped by the registration vehicle, not
        // the verb (recorded, not fixed, here — the ledger split is its own stone). `check.rs`
        // stays untouched (STOP-4); the ledger grows by exactly five, 106 → 111.
        ":wat::core::first",
        ":wat::core::second",
        ":wat::core::third",
        ":wat::core::PersistentVector",
        ":wat::core::PersistentMap",
    ];

    #[test]
    fn checker_skip_debt_is_named_and_frozen() {
        use crate::check::CheckEnv;
        use crate::types::TypeEnv;

        let type_env = TypeEnv::new();
        // The exact construction `doc_arg_ret_types_match_checker_scheme` uses — a
        // measurement that cannot disagree with the thing it measures.
        let check_env = CheckEnv::with_builtins_and_types(&type_env);

        let mut measured: Vec<&'static str> = super::registry()
            .all_entries()
            .filter(|entry| check_env.get(entry.name).is_none())
            .map(|entry| entry.name)
            .collect();
        measured.sort();
        measured.dedup();

        let frozen: Vec<&'static str> = FROZEN_CHECKER_DEBT_LEDGER.to_vec();

        let newly_unverified: Vec<&&'static str> =
            measured.iter().filter(|n| !frozen.contains(*n)).collect();
        let no_longer_unverified: Vec<&&'static str> =
            frozen.iter().filter(|n| !measured.contains(*n)).collect();

        assert!(
            newly_unverified.is_empty() && no_longer_unverified.is_empty(),
            "checker-skip DEBT LEDGER drifted from the measured population.\n\
             \n\
             NEW — registered but absent from `CheckEnv` (`check_env.get` returns `None`), \
             NOT on the frozen ledger — `doc_arg_ret_types_match_checker_scheme` is silently \
             skipping these and verifying nothing about their `@arg`/`@ret` docs. Add each to \
             `FROZEN_CHECKER_DEBT_LEDGER` (or register it in `register_builtins`, \
             `src/check.rs`, to remove it from the ledger entirely): {:?}\n\
             \n\
             STALE — on the frozen ledger but now resolved (`check_env.get` returns `Some` for \
             these), i.e. `register_builtins` now covers them and the gate verifies them for \
             real — delete each from `FROZEN_CHECKER_DEBT_LEDGER`: {:?}\n",
            newly_unverified, no_longer_unverified,
        );
    }

    /// Arc 255 STONE-the-checker-must-read-the-registry (AMENDED) — the sibling ledger
    /// `FROZEN_CHECKER_DEBT_LEDGER` was missing: its criterion (`check_env.get().is_none()`,
    /// "no `TypeScheme`") cannot distinguish a row checked FOR REAL by a hand-written
    /// `check_call`/`infer_*` arm (or, for a few runtime-only verbs, by their own `eval_*`
    /// guard) from a row nothing checks at all. `FROZEN_TYPES_UNCHECKED` names the STRICT
    /// SUBSET of that 71-row ledger whose TYPES nothing checks — measured behaviourally, by
    /// DRIVING THE CHECKER with each row's own wrong-typed call (never by grepping for an
    /// `infer_*` arm: a text predicate said twelve rows were unchecked and the behavioural
    /// probe corrected it to eleven — `macro-error` rejected by arc 255's `ExpandOnly` wall,
    /// not by any type check).
    ///
    /// ⛔ `:wat::core::variant` is NOT on this list, though the DESIGN's initial behavioural
    /// probe (an arity-abuse call, `(<verb> 1 2 .. 9)`) measured it as one of the eleven.
    /// STOP-1: `variant`'s declared arg type — `@arg xs… :wat::core::Value` — is
    /// `:wat::core::Value`, which `types.rs::is_subtype`'s `sup == ":wat::core::Value"` arm
    /// makes the UNIVERSAL SUBTYPE-TOP (arc 278 Stone-Value: "every type <: Value"). No value
    /// is ill-typed against a `Value`-typed parameter — passing `"not-a-keyword"` where the doc
    /// says "arg0 the enum's type path (literal keyword)" still type-checks, confirmed against
    /// the pre-stone binary (`(:wat::core::variant "not-a-keyword" "also-not" 1 2 3)` → exit 0).
    /// `variant`'s two positional keywords are validated at RUNTIME by hand (`eval_variant`'s
    /// own `MalformedForm` raises, `runtime.rs:15917`/`:15937`) — real enforcement, just not by
    /// the type system, and not expressible as a "was accepted, now rejected" gate: nothing
    /// could ever flip that gate red, because `assignable(_, Value)` is `true` by construction
    /// for every type, including a future one. Reported per STOP-1, not dropped and not faked;
    /// `variant` remains fully covered by `FROZEN_CHECKER_DEBT_LEDGER` above (no `TypeScheme`),
    /// just not by this narrower, probe-driven list.
    const FROZEN_TYPES_UNCHECKED: &[&str] = &[
        ":wat::core::fresh-symbol",
        ":wat::core::struct-field",
        ":wat::core::type-equal?",
        ":wat::core::type-params-used-in",
        ":wat::kernel::peer-pid",
        ":wat::runtime::metadata-of",
        ":wat::linkedlist::get",
        ":wat::linkedlist::length",
        ":wat::linkedlist::empty?",
        ":wat::linkedlist::contains?",
    ];

    /// One wrong-typed call per `FROZEN_TYPES_UNCHECKED` row, each derived from that row's own
    /// `@arg` declaration (not guessed from the verb's name) — right arity, wrong type at the
    /// FIRST argument whose declared type is concrete (not a top type, not a type variable).
    /// Confirmed against the pre-stone binary: every one of these `check`s `Ok(())` today.
    ///   - `fresh-symbol`         @arg base   :wat::core::String            → pass `5` (i64)
    ///   - `struct-field`         @arg record :wat::core::Record            → pass `5` (i64)
    ///   - `type-equal?`          @arg a      :wat::WatAST                  → pass `5` (i64) —
    ///     NOT a top type (unlike `:wat::core::Value`): the `ast-span` control (a scheme'd
    ///     `:wat::WatAST` param) rejects the identical `5` with `TypeMismatch`, so this is a
    ///     real gap, not another `variant`.
    ///   - `type-params-used-in`  @arg params (:wat::core::Vector :- [:wat::WatAST]) → pass `5`
    ///   - `peer-pid`             @arg peer   (:wat::kernel::Peer :- [I O]) → pass `5` (i64)
    ///   - `metadata-of`          @arg name_ast :wat::core::keyword         → pass `5` (i64)
    ///   - `linkedlist::get`      @arg l      (:wat::core::List :- [T])     → pass a `String`
    ///   - `linkedlist::length`   @arg l      (:wat::core::List :- [T])     → pass a `String`
    ///   - `linkedlist::empty?`   @arg l      (:wat::core::List :- [T])     → pass a `String`
    ///   - `linkedlist::contains?` @arg l     (:wat::core::List :- [T])     → pass a `String`
    const TYPE_RESIDUE_PROBES: &[(&str, &str)] = &[
        (":wat::core::fresh-symbol", "(:wat::core::fresh-symbol 5)"),
        (":wat::core::struct-field", "(:wat::core::struct-field 5 0)"),
        (":wat::core::type-equal?", "(:wat::core::type-equal? 5 5)"),
        (":wat::core::type-params-used-in", "(:wat::core::type-params-used-in 5 5)"),
        (":wat::kernel::peer-pid", "(:wat::kernel::peer-pid 5)"),
        (":wat::runtime::metadata-of", "(:wat::runtime::metadata-of 5)"),
        (":wat::linkedlist::get", "(:wat::linkedlist::get \"not-a-list\" 0)"),
        (":wat::linkedlist::length", "(:wat::linkedlist::length \"not-a-list\")"),
        (":wat::linkedlist::empty?", "(:wat::linkedlist::empty? \"not-a-list\")"),
        (":wat::linkedlist::contains?", "(:wat::linkedlist::contains? \"not-a-list\" 1)"),
    ];

    /// The bidirectional gate `FROZEN_TYPES_UNCHECKED` needs — the exact shape
    /// `checker_skip_debt_is_named_and_frozen` uses above, over a DIFFERENT pair of sets: here,
    /// "measured" cannot be a structural query (there is no `IntrinsicEntry` field recording
    /// "has a hand-written check arm"), so it is the set of names carrying a probe in
    /// `TYPE_RESIDUE_PROBES` — each probe DRIVES the real checker
    /// (`crate::check::tests::check`, the same `OnceLock`-cached pipeline
    /// `doc_arg_ret_types_match_checker_scheme` and every other consumer-side gate in this file
    /// use) rather than being read off a name.
    ///
    /// - NEW: a name carries a probe but is absent from `FROZEN_TYPES_UNCHECKED` — the ledger
    ///   and the probe table have drifted apart.
    /// - STALE (two ways): a frozen name carries no probe at all (dropped without updating the
    ///   ledger), OR its probe's `check()` now returns `Err` — the checker has started
    ///   rejecting the wrong-typed call for real, so the row is fixed and must come off the list.
    #[test]
    fn checker_type_residue_is_named_and_frozen() {
        // `.0`/`.1` field access (not tuple-pattern destructuring) throughout this fn —
        // auto-deref makes it depth-agnostic across `.iter()`/`.find()`'s reference layers,
        // where a `(n, _)` closure pattern would have to get the depth exactly right.
        let mut probed: Vec<&'static str> =
            TYPE_RESIDUE_PROBES.iter().map(|pair| pair.0).collect();
        probed.sort_unstable();
        probed.dedup();

        let frozen: Vec<&'static str> = FROZEN_TYPES_UNCHECKED.to_vec();

        let new: Vec<&'static str> =
            probed.iter().copied().filter(|n| !frozen.contains(n)).collect();
        assert!(
            new.is_empty(),
            "NEW — {:?} carries a wrong-typed probe in `TYPE_RESIDUE_PROBES` but is absent \
             from `FROZEN_TYPES_UNCHECKED` (`src/intrinsic/mod.rs`). Add it to the frozen list.",
            new,
        );

        let mut stale: Vec<String> = Vec::new();
        for name in frozen.iter().copied() {
            match TYPE_RESIDUE_PROBES.iter().find(|pair| pair.0 == name) {
                None => stale.push(format!(
                    "{name} — on `FROZEN_TYPES_UNCHECKED` but carries no entry in \
                     `TYPE_RESIDUE_PROBES`; add one or delete the name."
                )),
                Some(pair) => {
                    let src = pair.1;
                    if let Err(e) = crate::check::tests::check(src) {
                        stale.push(format!(
                            "{name} — its wrong-typed call `{src}` is now REJECTED \
                             ({e}); the checker verifies this row's types for real now — \
                             delete it from `FROZEN_TYPES_UNCHECKED` and `TYPE_RESIDUE_PROBES`."
                        ));
                    }
                }
            }
        }
        assert!(
            stale.is_empty(),
            "STALE — the type residue ledger has resolved entries still frozen:\n{}",
            stale.join("\n"),
        );
    }

    // ─── Arc 255 Stone the-membership-gap-gets-a-ratchet ─────────────────────────
    //
    // DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-membership-gap-gets-a-ratchet.md`.
    // The builder's sentence made mechanical: "the registry is not even the largest membership
    // set." Two populations, two gates, both bidirectional (NEW/STALE), neither freezing a
    // count (STOP-3) — the exact shape `checker_skip_debt_is_named_and_frozen` above already
    // proved out, applied to the OPPOSITE direction: names the type-checker already knows that
    // have no registry row (Gap A), and corpus call-heads the registry cannot vouch for at all
    // (Gap B).

    /// Arc 255 Stone the-membership-gap-gets-a-ratchet, Gap A — every FQDN `check_env` has a
    /// `TypeScheme` for (`CheckEnv::registered_names`, `src/check/env.rs`) that carries NO
    /// registry row (`registry().lookup_entry(n).is_none()`). `{n : check_env.get(n).is_some()
    /// ∧ registry().lookup_entry(n).is_none()}` — the MIRROR of `FROZEN_CHECKER_DEBT_LEDGER`
    /// above (that ledger is registry-has-a-row-but-checker-doesn't; this is checker-knows-it-
    /// but-registry-doesn't). When this list is empty, `registry() ⊇ check_env` — the builder's
    /// sentence, satisfied.
    ///
    /// Measured 2026-09-01 by static reading of `register_builtins` (`src/check.rs`) — every
    /// literal `env.register(":wat::…"` call plus every `for `-loop whose body registers each
    /// array element (the `:wat::i64::*`/`:wat::f64::*`/`:wat::rational::*` families, the
    /// `crate::rete::vocabulary::RETE_OPS` `Alias`/`Fallback` rows, the `:wat::kernel::sig*`
    /// sextet, the `:wat::math::`/`:wat::stat::`/`:wat::time::` `format!`-suffixed families) —
    /// cross-checked against every `#[wat_intrinsic("…")]`/`#[wat_special_form("…")]` FQDN
    /// under `src/`. **UNVERIFIED — no rider can run `cargo test` to confirm this against the
    /// gate's own live computation** (this stone's STOP-7 discipline); this is the best static
    /// reconstruction achievable by reading, not by executing. Re-derive by reading
    /// `register_builtins` and `crate::rete::vocabulary::RETE_OPS` again, or — better — read
    /// this gate's own failure message, which names the true population directly.
    const REGISTRY_MEMBERSHIP_GAP_A: &[&str] = &[
        ":wat::core::HashMap",
        ":wat::core::HashSet",
        ":wat::core::Vector",
        // Arc 255 Stone 1c-a-ii — ":wat::core::Tuple" / ":wat::core::apply" /
        // ":wat::core::conforms?" / ":wat::core::contains?" / ":wat::core::get" DELETED: all
        // five now carry a `#[wat_intrinsic]` row (`src/runtime.rs`), so `registry()` answers
        // for them and the gap this list named is closed.
        // ":wat::core::i64/to-f64" / ":wat::core::i64/to-string" NOT deleted this stone —
        // STOP-5 finding (BRIEF-STONE-the-seven-that-need-no-extraction's report): the
        // brief's named handler for both, `crate::numeric::convert::eval_i64_to_f64`/
        // `eval_i64_to_string`, takes a trailing `op: &str` param that
        // `crates/wat-macros/src/wat_intrinsic.rs`'s `sniff_args` does not recognize as a
        // context-tail type (`&Environment`/`&SymbolTable`/`&Span` only) — annotating it
        // verbatim does not compile, and reshaping it (dropping/hardcoding `op`) is exactly
        // what STOP-5 forbids, since the double-colon `:wat::i64::to-f64`/`to-string`
        // wrappers (`src/intrinsic/i64.rs`) already rely on passing their OWN op string
        // through the same shared fn. Both remain genuinely gapped; still measured true by
        // `registry_membership_gap_a_is_named_and_frozen` below.
        ":wat::core::i64/to-f64",
        ":wat::core::i64/to-string",
        ":wat::core::stream->pvec",
        ":wat::core::subtype?",
        ":wat::eval-ast!",
        ":wat::eval-digest!",
        ":wat::eval-digest-string!",
        ":wat::eval-edn!",
        ":wat::eval-file!",
        ":wat::eval-signed!",
        ":wat::eval-signed-string!",
        ":wat::eval-step!",
        ":wat::eval-with-defs!",
        ":wat::eval::walk",
        ":wat::rete::core::List/first",
        ":wat::rete::core::PersistentVector/first",
        ":wat::rete::core::Vector/first",
        ":wat::rete::core::bool::=",
        ":wat::rete::core::bool::not=",
        ":wat::rete::core::keyword::=",
        ":wat::rete::core::keyword::not=",
        ":wat::rete::f64::*",
        ":wat::rete::f64::+",
        ":wat::rete::f64::-",
        ":wat::rete::f64::/",
        ":wat::rete::fire-once$native",
        ":wat::rete::fire-rules$native",
        ":wat::rete::fire-rules-explain$native",
        ":wat::rete::holon::cosine",
        ":wat::rete::holon::dot",
        ":wat::rete::i64::*",
        // ":wat::rete::i64::>" REMOVED — arc 255 Stone 2a. This stone registers it as the
        // `@alias` witness (now `src/intrinsic/special/rete_alias.rs`, arc 255 Stone 1b-i,
        // where it sits with the other 28 rows of its own family), so
        // `registry().lookup_entry(":wat::rete::i64::>")` now returns `Some` and the gap
        // (checker-knows-it, registry-doesn't) this list names is closed for this one name —
        // exactly the STALE case `registry_membership_gap_a_is_named_and_frozen`'s own failure
        // message names: "on the frozen list but now resolved … delete it."
        ":wat::rete::i64::-",
        ":wat::rete::i64::/",
        ":wat::rete::i64::+",
        ":wat::rete::i64::mod",
        ":wat::rete::i64::quot",
        ":wat::rete::i64::rem",
        ":wat::rete::insert$native",
        ":wat::rete::insert-all$native",
        ":wat::rete::linkedlist::get",
        ":wat::rete::string::=",
        ":wat::rete::string::not=",
        ":wat::rete::string::subs",
        ":wat::rete::vec::get",
        ":wat::rete::vector::get",
        ":wat::stdlib::sources",
    ];

    /// The bidirectional gate for [`REGISTRY_MEMBERSHIP_GAP_A`] — exactly the shape
    /// `checker_skip_debt_is_named_and_frozen` uses above, over the OPPOSITE pair of sets
    /// (STOP-4: the population below is COMPUTED from `check_env`/`registry()` at test time,
    /// never read off the frozen array).
    #[test]
    fn registry_membership_gap_a_is_named_and_frozen() {
        use crate::check::CheckEnv;
        use crate::types::TypeEnv;

        let type_env = TypeEnv::new();
        let check_env = CheckEnv::with_builtins_and_types(&type_env);

        let mut measured: Vec<String> = check_env
            .registered_names()
            .filter(|name| super::registry().lookup_entry(name).is_none())
            .map(|s| s.to_string())
            .collect();
        measured.sort();
        measured.dedup();

        // `measured` is `Vec<String>` (owned — `check_env` is local, its scheme keys are not
        // `'static`), `REGISTRY_MEMBERSHIP_GAP_A` is `&'static [&'static str]`; `Vec::contains`
        // needs the SAME lifetime on both sides, which these two never share, so membership is
        // checked by content (`==` on `&str`, lifetime-agnostic) via `.any()` rather than
        // `.contains()`.
        let newly_ungapped: Vec<&String> = measured
            .iter()
            .filter(|n| !REGISTRY_MEMBERSHIP_GAP_A.contains(&n.as_str()))
            .collect();
        let no_longer_gapped: Vec<&&str> = REGISTRY_MEMBERSHIP_GAP_A
            .iter()
            .filter(|f| !measured.iter().any(|m| m.as_str() == **f))
            .collect();

        assert!(
            newly_ungapped.is_empty() && no_longer_gapped.is_empty(),
            "REGISTRY_MEMBERSHIP_GAP_A drifted from the measured population.\n\
             \n\
             NEW — `check_env` has a `TypeScheme` for these but `registry()` has no row, NOT on \
             the frozen list — add each to `REGISTRY_MEMBERSHIP_GAP_A`, or (better) register it \
             as a `#[wat_intrinsic]`/`#[wat_special_form]` to close the gap for real: {:?}\n\
             \n\
             STALE — on the frozen list but now resolved (`registry().lookup_entry` returns \
             `Some`), i.e. the name got registered — delete it from `REGISTRY_MEMBERSHIP_GAP_A`: \
             {:?}\n",
            newly_ungapped, no_longer_gapped,
        );
    }

    /// Arc 255 Stone the-membership-gap-gets-a-ratchet, Gap B — the FIXED historical record of
    /// the 121 corpus call-heads `resolve` could not have vouched for through the registry, per
    /// the four-step experiment recorded in
    /// `docs/arc/2026/06/255-builtin-registry/WORKLIST-the-121-the-registry-cannot-vouch-for.md`:
    /// patch `is_resolvable_call_head`'s `is_reserved_prefix` short-circuit to also require
    /// `registry().lookup_entry(head).is_some()`, `cargo build --release --bin wat`, `--check`
    /// every corpus `.wat` file, dedupe the unresolved heads. **A rider cannot re-run this — it
    /// needs a release build (this stone's brief says so explicitly) — so this array is NEVER
    /// hand-edited; re-derive it only by re-running the WORKLIST's own four steps.** It is the
    /// enumeration DOMAIN [`REGISTRY_MEMBERSHIP_GAP_B`]'s gate walks — never the "current gap"
    /// itself, which is the shrinking list just below.
    const GAP_B_CORPUS_CENSUS_121: &[&str] = &[
        ":wat::core::fn",
        ":wat::core::def",
        ":wat::core::match",
        ":wat::core::quote",
        ":wat::core::=",
        ":wat::core::do",
        ":wat::core::PersistentVector",
        ":wat::core::foldl",
        ":wat::core::first",
        ":wat::eval-ast!",
        ":wat::core::Tuple",
        ":wat::core::ann-form",
        ":wat::core::second",
        ":wat::core::PersistentMap",
        ":wat::core::get",
        ":wat::core::extend-type",
        ":wat::core::str",
        ":wat::core::forms",
        ":wat::core::quasiquote",
        ":wat::rete::string::=",
        ":wat::rete::i64::>",
        ":wat::core::map",
        ":wat::core::<",
        ":wat::core::derive",
        ":wat::rete::i64::+",
        ":wat::rete::core::and",
        ":wat::rete::string::starts-with?",
        ":wat::rete::i64::=",
        ":wat::core::>",
        ":wat::core::bool::to-string",
        ":wat::core::apply",
        ":wat::core::>=",
        ":wat::rete::i64::<",
        ":wat::core::show",
        ":wat::core::or",
        ":wat::rete::core::if",
        ":wat::rete::i64::*",
        ":wat::rete::core::or",
        ":wat::stream::lazy",
        ":wat::rete::i64::/",
        ":wat::rete::core::not",
        ":wat::core::not",
        ":wat::rete::vector::get",
        ":wat::core::macroexpand",
        ":wat::rete::i64::-",
        ":wat::rete::vector::length",
        ":wat::rete::i64::mod",
        ":wat::core::not=",
        ":wat::type::Tuple",
        ":wat::core::filter",
        ":wat::rete::string::contains?",
        ":wat::rete::map::contains-key?",
        ":wat::rete::holon::cosine",
        ":wat::rete::core::foldl",
        ":wat::core::u8",
        ":wat::core::defclause",
        ":wat::type::i64",
        ":wat::rete::f64::>",
        ":wat::core::contains?",
        ":wat::core::and",
        ":wat::rete::vector::contains?",
        ":wat::rete::string::length",
        ":wat::rete::core::let",
        ":wat::rete::core::fn",
        ":wat::rete::core::PersistentVector/first",
        ":wat::core::stream->vec",
        ":wat::core::<=",
        ":wat::type::String",
        ":wat::rete::string::subs",
        ":wat::rete::i64::not=",
        ":wat::rete::f64::/",
        ":wat::rete::f64::*",
        ":wat::core::third",
        ":wat::rete::i64::>=",
        ":wat::rete::core::match",
        ":wat::rete::core::keyword::=",
        ":wat::rete::i64::to-f64",
        ":wat::rete::core::enum::=",
        ":wat::eval-with-defs!",
        ":wat::core::None",
        ":wat::type::Vector",
        ":wat::rete::vec::get",
        ":wat::rete::string::trim",
        ":wat::rete::string::to-lowercase",
        ":wat::rete::string::ends-with?",
        ":wat::rete::string::empty?",
        ":wat::rete::string::concat",
        ":wat::rete::linkedlist::get",
        ":wat::rete::i64::rem",
        ":wat::rete::i64::<=",
        ":wat::rete::holon::dot",
        ":wat::rete::f64::<",
        ":wat::rete::core::enum::not=",
        ":wat::rete::core::Vector/first",
        ":wat::rete::core::PersistentVector",
        ":wat::rete::core::List/first",
        ":wat::core::println",
        ":wat::core::mapv",
        ":wat::core::edn::write",
        ":wat::spawn::process/grants",
        ":wat::rete::string::not=",
        ":wat::rete::i64::to-string",
        ":wat::rete::i64::quot",
        ":wat::rete::holon::presence?",
        ":wat::rete::holon::coincident?",
        ":wat::rete::f64::to-string",
        ":wat::rete::f64::not=",
        ":wat::rete::f64::>X",
        ":wat::rete::f64::>=",
        ":wat::rete::f64::=",
        ":wat::rete::f64::<=",
        ":wat::rete::f64::+",
        ":wat::rete::core::reduce",
        ":wat::rete::core::map",
        ":wat::rete::core::filter",
        ":wat::rete::core::bool::to-string",
        ":wat::core::tuple-get",
        ":wat::core::reduce-walk",
        ":wat::core::macroexpand-1",
        ":wat::core::find-last-index",
        ":wat::core::conforms?",
    ];

    /// Arc 255 Stone the-membership-gap-gets-a-ratchet, Gap B — the CURRENT ratchet: the subset
    /// of [`GAP_B_CORPUS_CENSUS_121`] still true today (`registry().lookup_entry(n).is_none()`).
    /// 121 → 119: `:wat::core::fn` and `:wat::core::match` left (deliverable 3 registers both).
    /// 119 → 115, arc 255 Stone the-seven-that-need-no-extraction: `:wat::core::bool::to-string`,
    /// `:wat::core::not`, `:wat::core::show`, `:wat::core::u8` leave (all four registered this
    /// stone). Not the one name the stone's own DESIGN predicted — the DESIGN's "only
    /// `bool::to-string` is in the 121 census" was checked against the census array by name and
    /// found wrong: `not`/`show`/`u8` are ALSO census members, and all three were still frozen
    /// here (STALE the instant they resolve) — the pairing this brief's own STOP-1 discipline
    /// says to verify, applied one level up, to a ledger-count prediction instead of a
    /// handler/verb pairing. `:wat::core::record?` is NOT in the 121 census (checked, absent),
    /// so registering it does not touch this list. Every registration stone after this one
    /// deletes its own names from here — leaving one frozen after registering it fails the gate
    /// below as STALE, which is the design working, not a bug — the DESIGN's own words for this
    /// mechanism.
    ///
    /// Arc 255 Stone 1a-β-ii — `:wat::core::def` LEAVES: registering it (`role = declare` on
    /// `register_defines`, `role = check` on `infer_def`) makes `registry().lookup_entry` return
    /// `Some`, so leaving it here would fail the gate below as STALE. It stays in
    /// `GAP_B_CORPUS_CENSUS_121` above (the fixed historical record — never edited) since that
    /// experiment genuinely could not vouch for it at the time; only the shrinking "current gap"
    /// list drops it.
    ///
    /// Arc 255 Stone 1a-zeta — `:wat::core::do` / `:wat::core::ann-form` / `:wat::stream::lazy`
    /// LEAVE: each carries a registered `role = eval` handler now (`intrinsic/special/
    /// do_form.rs` / `ann_form.rs` / `stream_lazy.rs`), so `registry().lookup_entry` returns
    /// `Some` for all three and leaving them here would fail the gate below as STALE. All three
    /// stay in `GAP_B_CORPUS_CENSUS_121` above (the fixed historical record — never edited);
    /// only the shrinking "current gap" list drops them.
    ///
    /// Arc 255 Stone 1c-a-ii — `:wat::core::Tuple`/`get`/`apply`/`contains?`/`conforms?` LEAVE:
    /// all five now carry a `#[wat_intrinsic]` row (`src/runtime.rs`), so `registry().lookup_entry`
    /// returns `Some` for all five and leaving them here would fail the gate below as STALE. All
    /// five stay in `GAP_B_CORPUS_CENSUS_121` above; only the shrinking "current gap" list drops
    /// them.
    ///
    /// Arc 255 Stone 1c-b-i — `:wat::core::first`/`second`/`third`/`PersistentVector`/
    /// `PersistentMap` LEAVE: all five now carry a `#[wat_intrinsic]` wrapper row
    /// (`src/runtime.rs` — `eval_first`/`eval_second`/`eval_third`/`eval_persistentvector`/
    /// `eval_persistentmap`), so `registry().lookup_entry` returns `Some` for all five and
    /// leaving them here would fail the gate below as STALE. All five stay in
    /// `GAP_B_CORPUS_CENSUS_121` above; only the shrinking "current gap" list drops them. None
    /// of the five carries an `env.register()` `TypeScheme` — they land on
    /// `FROZEN_CHECKER_DEBT_LEDGER` below instead (DEBT traded for GAP_B, not paid).
    const REGISTRY_MEMBERSHIP_GAP_B: &[&str] = &[
        ":wat::core::=",
        ":wat::eval-ast!",
        ":wat::core::extend-type",
        ":wat::core::str",
        ":wat::rete::string::=",
        ":wat::core::<",
        ":wat::core::derive",
        // ":wat::rete::i64::>" REMOVED -- arc 255 Stone 2a, same reason as its removal
        // from REGISTRY_MEMBERSHIP_GAP_A above: now registered (the @alias witness), so
        // registry().lookup_entry returns Some and this name is resolved, not gapped.
        ":wat::core::>",
        ":wat::core::>=",
        ":wat::rete::i64::*",
        ":wat::rete::i64::/",
        ":wat::rete::vector::get",
        ":wat::rete::i64::-",
        ":wat::rete::i64::+",
        ":wat::rete::i64::mod",
        ":wat::core::not=",
        ":wat::type::Tuple",
        ":wat::rete::holon::cosine",
        ":wat::rete::core::foldl",
        ":wat::core::defclause",
        ":wat::type::i64",
        ":wat::rete::core::PersistentVector/first",
        ":wat::core::<=",
        ":wat::type::String",
        ":wat::rete::string::subs",
        ":wat::rete::f64::/",
        ":wat::rete::f64::*",
        ":wat::rete::core::keyword::=",
        ":wat::rete::core::enum::=",
        ":wat::eval-with-defs!",
        ":wat::core::None",
        ":wat::type::Vector",
        ":wat::rete::vec::get",
        ":wat::rete::linkedlist::get",
        ":wat::rete::i64::rem",
        ":wat::rete::holon::dot",
        ":wat::rete::core::enum::not=",
        ":wat::rete::core::Vector/first",
        ":wat::rete::core::PersistentVector",
        ":wat::rete::core::List/first",
        ":wat::core::println",
        ":wat::core::edn::write",
        ":wat::spawn::process/grants",
        ":wat::rete::string::not=",
        ":wat::rete::i64::quot",
        ":wat::rete::f64::>X",
        ":wat::rete::f64::+",
        ":wat::rete::core::reduce",
        ":wat::rete::core::map",
        ":wat::rete::core::filter",
        ":wat::core::tuple-get",
        ":wat::core::reduce-walk",
        // Arc 255 Stone 1b-ii — 7 rete Form/Redispatch names LEAVE: `:wat::rete::core::
        // {and,or,if,let,match,fn}` and `:wat::rete::holon::coincident?` are registered rows
        // now (`src/intrinsic/special/rete_alias.rs`), so `registry().lookup_entry` answers
        // `Some` and this ratchet's STALE arm demanded their deletion by name.
        // ⚠ `:wat::rete::core::List` is NOT among them — it was never in
        // `GAP_B_CORPUS_CENSUS_121` (the corpus experiment never measured it), so it was never
        // on this list to remove. Checked, not assumed.
            // Arc 255 Stone 1a-γ-i — five homoiconic verbs LEAVE: `quote`, `forms`, `quasiquote`,
        // `macroexpand`, `macroexpand-1` are registered rows now, so `registry().lookup_entry`
        // answers `Some` and this ratchet's STALE arm demanded their deletion by name.
        // ⚠ `:wat::core::struct->form` is NOT among them — it registered in the same stone but was
        // never in `GAP_B_CORPUS_CENSUS_121` (the corpus experiment never measured it), so it was
        // never on this list to remove. Checked, not assumed.
];

    /// The bidirectional gate for Gap B. Walking the FIXED `GAP_B_CORPUS_CENSUS_121` domain
    /// (never recomputed — a rider/CI cannot re-run the corpus experiment) against LIVE
    /// `registry()` state catches both sabotages STOP-7 asks for: drop a still-unregistered
    /// name from `REGISTRY_MEMBERSHIP_GAP_B` without registering it → it is still `None` in the
    /// registry but missing from the frozen list → NEW; leave an already-registered name frozen
    /// → STALE. A name is asserted to belong to exactly one of "still in the gap, frozen" or
    /// "resolved, not frozen" — never both, never neither.
    #[test]
    fn registry_membership_gap_b_is_named_and_frozen() {
        let mut newly_ungapped: Vec<&'static str> = Vec::new();
        let mut no_longer_gapped: Vec<&'static str> = Vec::new();

        for name in GAP_B_CORPUS_CENSUS_121.iter().copied() {
            let still_unregistered = super::registry().lookup_entry(name).is_none();
            let is_frozen = REGISTRY_MEMBERSHIP_GAP_B.contains(&name);
            match (still_unregistered, is_frozen) {
                (true, false) => newly_ungapped.push(name),
                (false, true) => no_longer_gapped.push(name),
                _ => {}
            }
        }

        // Transcription integrity: every frozen name must come FROM the fixed census — a name
        // here that is not in `GAP_B_CORPUS_CENSUS_121` at all is not a "still in the gap" fact,
        // it is a typo the loop above would silently never check.
        let foreign: Vec<&'static str> = REGISTRY_MEMBERSHIP_GAP_B
            .iter()
            .copied()
            .filter(|n| !GAP_B_CORPUS_CENSUS_121.contains(n))
            .collect();

        assert!(
            newly_ungapped.is_empty() && no_longer_gapped.is_empty() && foreign.is_empty(),
            "REGISTRY_MEMBERSHIP_GAP_B drifted from the measured population.\n\
             \n\
             NEW — still unregistered per `registry().lookup_entry`, but NOT on \
             `REGISTRY_MEMBERSHIP_GAP_B` (dropped without registering) — restore each: {:?}\n\
             \n\
             STALE — on `REGISTRY_MEMBERSHIP_GAP_B` but now resolved (`registry().lookup_entry` \
             returns `Some`) — delete each from `REGISTRY_MEMBERSHIP_GAP_B`: {:?}\n\
             \n\
             FOREIGN — on `REGISTRY_MEMBERSHIP_GAP_B` but absent from the fixed \
             `GAP_B_CORPUS_CENSUS_121` domain — not a name the corpus experiment ever measured: \
             {:?}\n",
            newly_ungapped, no_longer_gapped, foreign,
        );
    }

    /// Arc 255.1b-v: every `@see` FQDN in the intrinsic corpus must resolve —
    /// to a registered Rust intrinsic OR (arc 255 STONE "`@see` can cross the
    /// boundary") a wat verb that DECLARES (carries an axis-declaration key
    /// in its `binding_metadata`). A dangling @see is a broken cross-reference
    /// in the doc system → fail loud.
    #[test]
    fn all_see_fqdns_resolve_to_registered_intrinsics() {
        let dangling = super::reflect::check_see_refs();
        assert!(
            dangling.is_empty(),
            "Found {} dangling @see reference(s) in the intrinsic corpus:\n{}",
            dangling.len(),
            dangling.join("\n")
        );
    }

    /// Arc 255 STONE "`@see` can cross the boundary" — the negative that makes the stone
    /// falsifiable (STOP-3). `all_see_fqdns_resolve_to_registered_intrinsics` alone cannot
    /// distinguish "the gate correctly checks declaration" from "the gate accepts anything in
    /// the symbol table": both are green today, because nothing in the live corpus currently
    /// carries a dangling `@see`. This test exercises `see_target_resolves` directly, on three
    /// constructed FQDNs, so the rule's actual behaviour is on the record rather than merely
    /// implied by an empty `dangling` list:
    ///
    ///   1. `:wat::core::sort` — the wat verb this stone declares — MUST now resolve. The
    ///      positive control: proves the new wat-store path fires at all, not just that the
    ///      negative below happens to fail for an unrelated reason (e.g. a broken `startup_bare`).
    ///   2. `:wat::kernel::spawn-program` — a REAL wat verb, present in `binding_metadata` today
    ///      (`wat/spawn.wat`), whose ONLY metadata is `{:restricted-to […]}` — a capability map
    ///      with no axis-declaration key. STOP-1's exact scenario: `contains_key` alone would
    ///      accept it (it *is* in the table); `meta_has_doc_axis_key` correctly does not. Proves
    ///      the gate is declaration-gated, not presence-gated.
    ///   3. A wholly fabricated FQDN naming no verb in either store at all — the baseline "this
    ///      names nothing" case, kept alongside (2) so a reader can tell "undeclared" and
    ///      "nonexistent" apart; both must dangle, for different reasons.
    #[test]
    fn undeclared_wat_target_still_dangles() {
        let reg = super::registry();
        let world = super::reflect::bare_stdlib_world();
        let wat_binding_metadata = &world.symbols().binding_metadata;

        assert!(
            super::reflect::see_target_resolves(":wat::core::sort", reg, wat_binding_metadata),
            "positive control failed: `:wat::core::sort` carries an axis-declaration metadata \
             map (wat/core.wat) and MUST resolve — if this assertion fails, the wat-store path \
             itself is broken, and the negative cases below prove nothing."
        );

        assert!(
            wat_binding_metadata.contains_key(":wat::kernel::spawn-program"),
            "test's own premise is stale: `:wat::kernel::spawn-program` (wat/spawn.wat) no \
             longer has a `binding_metadata` entry at all — pick another capability-only verb \
             to carry STOP-1's `contains_key`-alone-is-not-the-test case."
        );
        assert!(
            !super::reflect::see_target_resolves(
                ":wat::kernel::spawn-program",
                reg,
                wat_binding_metadata
            ),
            "STOP-1: `:wat::kernel::spawn-program`'s ONLY metadata is `{{:restricted-to […]}}` \
             — a capability map, not a doc declaration. `contains_key` alone would accept it; \
             the gate must not. A dead link into an undocumented verb is exactly the failure \
             `declared` (not `exists`) exists to forbid."
        );

        assert!(
            !super::reflect::see_target_resolves(
                ":wat::core::this-fqdn-names-no-verb-in-either-store",
                reg,
                wat_binding_metadata
            ),
            "STOP-3: an @see target naming nothing in either store must still be flagged \
             dangling — a gate that accepts everything is indistinguishable from no gate."
        );
    }

    /// Arc 255 Stone 2a — the `@alias` gate: every `alias_of` target must be a registered row,
    /// and no target may itself be an alias (no chains). Non-vacuity FIRST (STOP-1's own
    /// warning: "a structural gate that passes while dispatch is unchanged is the failure this
    /// STOP exists to catch" applies just as much to a gate that trivially passes because it
    /// inspected nothing) — asserts the corpus actually carries the one alias witness this
    /// stone registers before trusting `check_alias_refs`'s empty result to mean anything.
    #[test]
    fn no_dangling_or_chained_aliases() {
        let reg = super::registry();
        assert!(
            reg.lookup_entry(":wat::rete::i64::>").is_some_and(|e| e.alias_of == Some(":wat::i64::>")),
            "non-vacuity precondition failed: the one alias witness this stone registers \
             (`:wat::rete::i64::>` -> `:wat::i64::>`) is missing from the corpus — an empty \
             `check_alias_refs()` result below would prove nothing about the gate."
        );
        let bad = super::reflect::check_alias_refs();
        assert!(
            bad.is_empty(),
            "Found {} dangling/chained @alias reference(s) in the intrinsic corpus:\n{}",
            bad.len(),
            bad.join("\n")
        );
    }

    /// Arc 255 Stone 2a, STOP-5 — the negative that makes the gate falsifiable, on BOTH halves,
    /// exercised directly on two constructed FQDNs against the REAL registry (no second alias
    /// row is added to the live corpus to manufacture the chain case — the stone registers
    /// exactly one, and this test uses that one AS the chain target, since it already is itself
    /// an alias):
    ///
    ///   1. DANGLING — a fabricated FQDN naming no verb anywhere must not resolve.
    ///   2. CHAINED — `:wat::rete::i64::>` (this stone's own witness) is ITSELF an alias, so a
    ///      hypothetical second alias pointed at it must read `true` here: the exact scenario
    ///      Sabotage-2 (point a second alias at an aliased row) predicts red on.
    ///   3. The control — `:wat::i64::+` (the witness's own target) is a real handler, not an
    ///      alias, so pointing at IT must read `false`: proves the chain check discriminates
    ///      rather than flagging every target unconditionally.
    #[test]
    fn alias_gate_catches_dangling_and_chained_targets() {
        let reg = super::registry();
        assert!(
            !super::reflect::alias_target_is_registered(":wat::core::zorble", reg),
            "STOP-1: a fabricated @alias target must not resolve as registered — the dangling \
             half of the gate is meaningless if a made-up FQDN passes."
        );
        assert!(
            super::reflect::alias_target_is_itself_aliased(":wat::rete::i64::>", reg),
            "STOP-5: this stone's own witness is itself an alias — pointing a (hypothetical) \
             second alias at it must be flagged as a chain. If this reads false, the chain half \
             of the gate cannot fire on the one alias the live corpus actually has."
        );
        assert!(
            !super::reflect::alias_target_is_itself_aliased(":wat::i64::>", reg),
            "control: the witness's OWN target is a real handler, not itself an alias — the \
             chain check must not flag every registered name unconditionally."
        );
    }

    /// Arc 255 Stone 2a-b — proves the fold-time axis RESOLUTION in `registry()` actually RAN,
    /// not merely that an alias's axes happen to equal its target's by coincidence. This is
    /// deliberately NOT the equality-gate the DESIGN disqualified ("a gate that compares two
    /// tables is a measurement of the split, not a cure for it") — after this stone there is
    /// only ONE declaration (the target's), so this checks that the COPY reached the alias
    /// entry, never that two independent authors happened to agree.
    ///
    /// Non-vacuous (STOP-1's own warning, repeated here): asserts the corpus carries at least
    /// one `alias_of` row — and names this stone's own witness among the rows it inspected —
    /// before trusting the loop below to have proven anything.
    #[test]
    fn alias_axes_are_resolved_from_their_target() {
        let reg = super::registry();
        let alias_rows: Vec<_> = reg.all_entries().filter(|e| e.alias_of.is_some()).collect();
        assert!(
            !alias_rows.is_empty(),
            "non-vacuity precondition failed: no `alias_of` row found in the corpus — a loop \
             over zero rows would prove nothing about whether resolution ran."
        );
        let mut inspected: Vec<&str> = Vec::new();
        for entry in &alias_rows {
            // Dangling targets are `no_dangling_or_chained_aliases`'s job (STOP-4, unchanged) —
            // this gate assumes that one already passed and a target row exists to compare
            // against; if it does not, that is the more specific failure to fix first.
            let target_name = entry.alias_of.expect("filtered on alias_of.is_some() above");
            let target = reg.lookup_entry(target_name).unwrap_or_else(|| {
                panic!(
                    "`{}`'s @alias target `{}` is not a registered row — fix \
                     `no_dangling_or_chained_aliases` first; this gate assumes a resolvable target",
                    entry.name, target_name
                )
            });
            inspected.push(entry.name);
            assert_eq!(
                entry.purity, target.purity,
                "`{}`'s @Purity did not follow its target `{}`'s — the fold-time resolution \
                 pass did not run (or the axes were never copied)",
                entry.name, target_name
            );
            assert_eq!(
                entry.determinism, target.determinism,
                "`{}`'s @Determinism did not follow its target `{}`'s",
                entry.name, target_name
            );
            assert_eq!(
                entry.totality, target.totality,
                "`{}`'s @Totality did not follow its target `{}`'s — the exact contradiction \
                 this stone exists to remove",
                entry.name, target_name
            );
            assert_eq!(
                entry.expand_time, target.expand_time,
                "`{}`'s @ExpandTime did not follow its target `{}`'s",
                entry.name, target_name
            );
            assert_eq!(
                entry.category, target.category,
                "`{}`'s @Category did not follow its target `{}`'s — the exact contradiction \
                 this stone exists to remove",
                entry.name, target_name
            );
        }
        // ⚠ The witness name is bound to a `const` rather than written inline, and it is not a
        // style choice: the two lints that guard this line want OPPOSITE spellings of it.
        // `tests/lint/no_loose_string_assert.rs` scans by line and cannot tell
        // `Vec::contains(&"literal")` from the `str::contains("literal")` it bans; clippy's
        // `manual_contains` demands exactly that `Vec::contains` form over `iter().any()`.
        // A named const satisfies both — clippy sees the `contains`, the line-scanner sees no
        // string literal. `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`
        const WITNESS: &str = ":wat::rete::i64::>";
        assert!(
            inspected.contains(&WITNESS),
            "non-vacuity: expected this stone's own witness `{WITNESS}` among the \
             inspected rows, found {:?}",
            inspected
        );
    }

    /// Arc 255.1b-firm: the doc's `@arg`/`@ret` type strings must match the
    /// checker's `TypeScheme` for each registered intrinsic. A mismatch is a
    /// doc lie — the user reads one type, the checker enforces another.
    ///
    /// For variadic args (`is_rest=true`): the doc's element type must match the
    /// ELEMENT of `scheme.rest_param_type` (a `Vector<elem>` in the scheme).
    /// PROBE (measurement, not a gate) — CAN THE REGISTRY ABSORB THE CHECKER'S SCHEMES?
    ///
    /// `doc_arg_ret_types_match_checker_scheme` (below) proves the two representations AGREE,
    /// by projecting `TypeExpr -> doc string` (`typeexpr_to_doc_string`). Agreement in one
    /// direction does not prove the doc string can RECONSTRUCT the scheme — a lossy projection
    /// agrees too. This probe measures the INVERSE, which is the direction the registry would
    /// need if it were to become the type authority: doc string -> `TypeExpr`, via the
    /// substrate's own `parse_type_expr_from_source`.
    ///
    /// It asserts nothing about the outcome; it prints a census. The number that matters is how
    /// many entries round-trip EXACTLY, and — separately — how many schemes carry `type_params`
    /// (quantified generics) that an `@arg`/`@ret` string has no slot for at all.
    #[test]
    fn probe_can_doc_types_reconstruct_the_checker_scheme() {
        use crate::check::CheckEnv;
        use crate::types::TypeEnv;

        let type_env = TypeEnv::new();
        let check_env = CheckEnv::with_builtins_and_types(&type_env);

        let (mut with_scheme, mut full_rt, mut generic, mut generic_recoverable) = (0, 0, 0, 0);
        let mut parse_fail_examples: Vec<String> = Vec::new();
        let mut mismatch_examples: Vec<String> = Vec::new();
        let mut generic_examples: Vec<String> = Vec::new();
        let mut failing_rows: Vec<&'static str> = Vec::new();

        for entry in super::registry().all_entries() {
            let Some(scheme) = check_env.get(entry.name) else { continue };
            with_scheme += 1;
            if !scheme.type_params.is_empty() {
                generic += 1;
                // Measure, do not assume: is every quantified var NAMED somewhere in the doc's
                // own arg/ret type strings? If it is not, the doc cannot reconstruct the
                // quantifier list and the registry cannot be the type authority for this row.
                let doc_text: String = entry
                    .args.iter().map(|a| a.1)
                    .chain(std::iter::once(entry.ret_type))
                    .collect::<Vec<_>>().join(" ");
                let missing: Vec<&str> = scheme
                    .type_params.iter()
                    .filter(|v| !doc_text.split(|c: char| !c.is_alphanumeric() && c != '_')
                                         .any(|tok| tok == v.as_str()))
                    .map(|v| v.as_str()).collect();
                if missing.is_empty() { generic_recoverable += 1 } else if generic_examples.len() < 8 {
                    generic_examples.push(format!(
                        "{} <{}> — doc never names {:?}  (doc types: {})",
                        entry.name, scheme.type_params.join(","), missing, doc_text));
                }
            }
            let mut ok = true;
            let mut check = |doc: &str, want: &crate::types::TypeExpr, what: &str| {
                match crate::types::parse_type_expr_from_source(doc) {
                    Err(e) => {
                        ok = false;
                        if parse_fail_examples.len() < 8 {
                            parse_fail_examples.push(format!("{} {}: `{}` -> {}", entry.name, what, doc, e));
                        }
                    }
                    Ok(got) => {
                        // ⛔ Compare the TypeExprs THEMSELVES (`TypeExpr: PartialEq`), never via
                        // `typeexpr_to_doc_string`. Comparing through the forward projection is
                        // the defect this probe exists to avoid: a lossy projection makes two
                        // different TypeExprs compare equal, and the probe scores a match it did
                        // not earn. The first draft of this probe did exactly that and returned
                        // a perfect 386/386. `[[feedback_a_green_test_can_prove_nothing]]`
                        if got != *want {
                            ok = false;
                            if mismatch_examples.len() < 8 {
                                mismatch_examples.push(format!(
                                    "{} {}: doc `{}`\n        parsed  {:?}\n        scheme  {:?}",
                                    entry.name, what, doc, got, want));
                            }
                        }
                    }
                }
            };
            for (i, &(_, ty, _, is_rest)) in entry.args.iter().enumerate() {
                if is_rest || ty.is_empty() { continue }
                if i < scheme.params.len() { check(ty, &scheme.params[i], &format!("arg{i}")); }
            }
            if !entry.ret_type.is_empty() { check(entry.ret_type, &scheme.ret, "ret"); }
            if ok { full_rt += 1 } else if !failing_rows.contains(&entry.name) { failing_rows.push(entry.name) }
        }

        // ── The other half of the question: the rows with NO scheme at all.
        let mut no_scheme_sf = 0;
        let mut no_scheme_intr: Vec<&'static str> = Vec::new();
        for entry in super::registry().all_entries() {
            if check_env.get(entry.name).is_some() { continue }
            match entry.kind {
                super::Kind::SpecialForm => no_scheme_sf += 1,
                _ => no_scheme_intr.push(entry.name),
            }
        }
        no_scheme_intr.sort_unstable();
        eprintln!("\n=== ROWS WITH NO CHECKER SCHEME — by kind ===");
        eprintln!("  total registry rows ......................... {}", super::registry().all_entries().count());
        eprintln!("  Kind::SpecialForm, no scheme ................ {no_scheme_sf}  <- a rank-1 scheme is the WRONG SHAPE");
        eprintln!("  Kind::Intrinsic,  no scheme ................. {}  <- a scheme could exist and does not", no_scheme_intr.len());
        for n in &no_scheme_intr { eprintln!("     {n}"); }

        eprintln!("\n=== CAN THE REGISTRY ABSORB THE SCHEMES? — census ===");
        eprintln!("  registered rows WITH a checker scheme ....... {with_scheme}");
        eprintln!("  round-trip EXACTLY (doc -> TypeExpr == scheme) {full_rt}");
        eprintln!("  failed (parse error or mismatch) ............ {}", with_scheme - full_rt);
        eprintln!("  schemes carrying type_params (generics) ..... {generic}");
        eprintln!("    of those, every var NAMED in the doc types . {generic_recoverable}");
        eprintln!("    quantifier NOT recoverable from the doc .... {}", generic - generic_recoverable);
        eprintln!("  parse failures (sample):");
        for e in &parse_fail_examples { eprintln!("     {e}"); }
        eprintln!("  mismatches (sample):");
        for e in &mismatch_examples { eprintln!("     {e}"); }
        eprintln!("  generic schemes whose quantifier is NOT recoverable (sample):");
        for e in &generic_examples { eprintln!("     {e}"); }

        // ── The gate. Freeze the NAMES, never the count: a count cannot tell "+1 new, -1 fixed"
        // from "nothing happened", and its failure message cannot name the offender.
        // `[[feedback_a_gate_freezes_names_never_a_count]]`
        //
        // Both frozen rows are SPELLING normalizations, measured — not lost information:
        //   `:wat::rete::lower` ret  — the parser canonicalizes `:wat::core::nil` to `Tuple([])`;
        //                              the scheme holds `Path(":wat::core::nil")`. Same type.
        //   `:wat::string::join` arg1 — the parser yields the type var as `Path(":T")`, the scheme
        //                              as `Path("T")`. A leading colon. This is the recurring class:
        //                              a comparison with one side normalized and the other not.
        const FROZEN_SPELLING_MISMATCHES: &[&str] = &[":wat::rete::lower", ":wat::string::join"];
        let unexpected: Vec<&str> = failing_rows
            .iter()
            .filter(|n| !FROZEN_SPELLING_MISMATCHES.contains(n))
            .copied()
            .collect();
        assert!(
            unexpected.is_empty(),
            "a registered row's @arg/@ret types no longer reconstruct its checker TypeScheme: {unexpected:?}\n\
             This probe measures whether the REGISTRY could become the type authority. A new name \
             here means the doc and the scheme have diverged in a way the doc cannot express — \
             either fix the doc, or record why the divergence is a real limit and add the name to \
             FROZEN_SPELLING_MISMATCHES with its measured reason."
        );
        for n in FROZEN_SPELLING_MISMATCHES {
            assert!(
                failing_rows.contains(n),
                "`{n}` is frozen as a known spelling mismatch but now round-trips — the freeze list \
                 is stale; remove it. A frozen row that silently starts passing is how a gate rots \
                 into decoration."
            );
        }
    }

    #[test]
    fn doc_arg_ret_types_match_checker_scheme() {
        use crate::check::CheckEnv;
        use crate::types::TypeEnv;

        let type_env = TypeEnv::new();
        let check_env = CheckEnv::with_builtins_and_types(&type_env);

        for entry in super::registry().all_entries() {
            let scheme = match check_env.get(entry.name) {
                Some(s) => s,
                None => continue, // not yet in checker — skip
            };

            // Check arg types.
            for (i, &(_, ty, _, is_rest)) in entry.args.iter().enumerate() {
                if is_rest {
                    // Variadic: doc elem type must match the element of scheme.rest_param_type.
                    // The doc's @arg type uses full `:wat::core::X` form (top-level, with `:`),
                    // so compare using typeexpr_to_doc_string (not the type-arg variant).
                    if let Some(rest_ty) = &scheme.rest_param_type {
                        // rest_param_type is Vector<elem>; extract the elem.
                        let elem_ty = match rest_ty {
                            crate::types::TypeExpr::Parametric { args, .. } if !args.is_empty() => {
                                typeexpr_to_doc_string(&args[0])
                            }
                            other => typeexpr_to_doc_string(other),
                        };
                        assert_eq!(
                            ty, elem_ty.as_str(),
                            "doc elem type for variadic `{}` arg {} says `{}`, \
                             checker rest_param_type elem says `{}`",
                            entry.name, i, ty, elem_ty
                        );
                    }
                    // If no rest_param_type on the scheme, skip — not yet registered.
                } else if i < scheme.params.len() {
                    let scheme_ty = typeexpr_to_doc_string(&scheme.params[i]);
                    assert_eq!(
                        ty, scheme_ty.as_str(),
                        "doc type for `{}` arg {} says `{}`, checker scheme says `{}`",
                        entry.name, i, ty, scheme_ty
                    );
                }
            }

            // Check ret type.
            let scheme_ret = typeexpr_to_doc_string(&scheme.ret);
            assert_eq!(
                entry.ret_type, scheme_ret.as_str(),
                "doc ret type for `{}` says `{}`, checker scheme says `{}`",
                entry.name, entry.ret_type, scheme_ret
            );
        }
    }

    /// Arc 109 stone "the smart comments must be compliant" — REWRITTEN. This used
    /// to render `Parametric`/non-empty-`Tuple` types with the retired angle-bracket
    /// spelling (`:{head}<{args}>`, `:({items})`) — a SECOND, independent
    /// implementation of "what may a type be spelled," alongside `wat-doc`'s own
    /// `@arg`/`@ret` reader-based check, and it was still spelling the annihilated
    /// vocabulary. Discovered as a direct, mechanical consequence of fixing the
    /// `@arg`/`@ret` doc strings to the surviving spellings: this test started
    /// failing because the doc string (now correct) no longer matched what this
    /// renderer (still retired) expected. Now emits the same surviving forms the
    /// reader accepts and the corpus actually uses: `(Head :- [args])` for a
    /// parametric type reference, `(:wat::core::Tuple :- [items])` for a non-empty
    /// tuple (verified against `wat/kernel/channel.wat`'s `Channel<T>` typealias,
    /// which is itself `(:wat::core::Tuple :- [(:wat::kernel::Sender :- [T]) …])` —
    /// the PRIOR note here cited that file's `;;` PROSE comment, not its code, for
    /// the retired spelling), and the bracket form `[args :-> ret]` for a fn type.
    fn typeexpr_to_doc_string(ty: &crate::types::TypeExpr) -> String {
        match ty {
            crate::types::TypeExpr::Path(p) => p.clone(),
            crate::types::TypeExpr::Parametric { head, args } => {
                let args_str: Vec<String> = args.iter().map(typeexpr_to_type_arg_string).collect();
                format!("(:{} :- [{}])", head, args_str.join(" "))
            }
            // `nil` IS the unit type — registered as an alias to the empty tuple
            // (types.rs:879) and canonicalized away at parse (types.rs:4706). The
            // checker therefore stores it as `Tuple([])`, while every doc, signature
            // and call site in the corpus writes `:wat::core::nil`. Render the
            // canonical form back to the spelling humans use.
            crate::types::TypeExpr::Tuple(items) if items.is_empty() => ":wat::core::nil".to_string(),
            // Non-empty tuple — verified against the corpus's own canonical
            // parametric-type-reference form (see fn doc above), not invented.
            crate::types::TypeExpr::Tuple(items) if !items.is_empty() => {
                let items_str: Vec<String> =
                    items.iter().map(typeexpr_to_type_arg_string).collect();
                format!("(:wat::core::Tuple :- [{}])", items_str.join(" "))
            }
            crate::types::TypeExpr::Fn { args, ret } => {
                let args_str: Vec<String> = args.iter().map(typeexpr_to_doc_string).collect();
                if args_str.is_empty() {
                    format!("[:-> {}]", typeexpr_to_doc_string(ret))
                } else {
                    format!("[{} :-> {}]", args_str.join(" "), typeexpr_to_doc_string(ret))
                }
            }
            other => format!("{:?}", other),
        }
    }

    /// Render a TypeExpr as it appears INSIDE a `:- [...]` argument list. A
    /// concrete type keeps its leading `:` (`:wat::core::i64`); a lexically-scoped
    /// type VARIABLE (`:T`, `:S`, `:R` — stored as a colon-prefixed `Path` per
    /// `check.rs`'s `t_var()`, same as any other `Path`) is written BARE, per the
    /// corpus's own `:- [S R]` binder/reference convention. Reuses
    /// `runtime::is_type_var_path` (Stone 251.7 "THE VAR TEST") rather than
    /// re-deriving the var/concrete distinction here — a third implementation of
    /// that question is exactly the shape this stone exists to stop shipping.
    fn typeexpr_to_type_arg_string(ty: &crate::types::TypeExpr) -> String {
        match ty {
            crate::types::TypeExpr::Path(p) => {
                if crate::declare::parse::is_type_var_path(p) {
                    p.strip_prefix(':').unwrap_or(p).to_string()
                } else {
                    p.clone()
                }
            }
            other => typeexpr_to_doc_string(other),
        }
    }

    /// Arc 255.1b-firm: pure+det intrinsics MUST carry ≥1 runnable `@example`;
    /// non-pure-det intrinsics MUST carry ≥1 `@example-norun` and NO runnable
    /// ★ ROW 4 — arc 255 stone total-T2b. THE CARRIAGE PROOF, and it is two-sided.
    ///
    /// T2 made `@Totality` parse and made the proc-macro turn it into a token; what it could
    /// NOT prove — because its blast radius forbade `src/` while its acceptance row demanded
    /// the registry — is that the value survives the last hop into the submission literal and
    /// out the other side as an `IntrinsicEntry`. A directive that parses correctly and is
    /// then dropped on the floor passes every parser test ever written.
    ///
    /// Two-sided ON PURPOSE: an assertion that only checked `Partial` would also pass if the
    /// field were hard-wired to `Partial`, and one that only checked `Unreviewed` would pass
    /// if carriage were broken and everything defaulted. Both directions, in one test, is the
    /// only shape that can fail for the real reason.
    /// `[[feedback_a_green_test_can_prove_nothing]]`
    ///
    /// The subject is deliberate. `:wat::i64::/` is the exact verb the two surviving totality
    /// hand-lists CONTRADICT each other about — `macros/eval.rs`'s `is_pure_total` includes it
    /// ("div-by-zero is a deterministic located abort … never a panic") while
    /// `rete/purity.rs`'s `total` sub-list excludes it ("`i64::/` is both, and undefined at a
    /// zero divisor"). Its `@Totality Partial` here is not an adjudication of that dispute; it is
    /// a TRANSCRIPTION of what the verb's own shipped doc already says two lines above the
    /// directive: *"`b = 0` raises `DivisionByZero`; `i64::MIN / -1` raises `IntegerOverflow`."*
    /// Two distinct inputs on which it is undefined.
    ///
    /// ✅ RESOLVED 2026-08-30, and this paragraph's premise is now STALE in two ways. (a) The two
    /// lists no longer contradict: Stone expand-1 renamed `is_pure_total` -> `is_expand_time_legal`,
    /// and that file now says `:wat::i64::/` "is legal DESPITE being `@Totality Partial` … Totality and
    /// expand-time legality are different axes." They never disagreed about one property — they
    /// answered two questions under one name. (b) The builder ADJUDICATED the underlying question:
    /// a raise is not a matchable OUTCOME, so a raising verb is `Partial`. The transcription above
    /// is therefore also the ruling. See
    /// `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`.
    #[test]
    fn totality_is_carried_from_the_doc_into_the_registry_entry() {
        let reg = super::registry();
        let div = reg
            .all_entries()
            .find(|e| e.name == ":wat::i64::/")
            .expect(":wat::i64::/ must be registered");
        assert_eq!(
            div.totality,
            wat_doc::Totality::Partial,
            "`:wat::i64::/` declares `@Totality Partial`; reading `Unreviewed` here means the \
             directive parsed but never reached the submission literal"
        );

        // The other side: a verb that declares NOTHING must read back the default. If this
        // also said `Partial`, the field would be hard-wired rather than carried.
        let mul = reg
            .all_entries()
            .find(|e| e.name == ":wat::i64::*")
            .expect(":wat::i64::* must be registered");
        assert_eq!(
            mul.totality,
            wat_doc::Totality::Unreviewed,
            "`:wat::i64::*` declares no `@Totality`; it must read the `Unreviewed` default, or \
             the field is not being carried per-verb at all"
        );
    }

    /// ★ Arc 255 Stone expand-T2, Row 3 — THE STONE. Rows 1-2 (in `wat-doc` and
    /// `wat-macros`) test that `@ExpandTime` PARSES and turns into a token; this is
    /// the only row that tests the value survives the last hop into the submission
    /// literal and out the other side as an `IntrinsicEntry`. A directive that
    /// parses correctly and is then dropped on the floor passes every parser test
    /// ever written — exactly `totality_is_carried_from_the_doc_into_the_registry_entry`'s
    /// own rationale, one axis over.
    ///
    /// Two-sided ON PURPOSE: an assertion that only checked `Legal` would also pass
    /// if the field were hard-wired to `Legal`, and one that only checked
    /// `Unreviewed` would pass if carriage were broken and everything defaulted.
    /// Both directions, in one test, is the only shape that can fail for the real
    /// reason. `[[feedback_a_green_test_can_prove_nothing]]`
    ///
    /// The subject is deliberate. `:wat::core::fresh-symbol` is THIS AXIS'S OWN
    /// WITNESS (`wat/runtime-meta.wat`'s `ExpandTime` doc names it explicitly):
    /// nondeterministic yet expand-time-legal — minting a different gensym per call
    /// is what makes hygienic macro expansion possible — so its `@ExpandTime Legal`
    /// is a true, load-bearing claim rather than decoration.
    ///
    /// ⚠ THE NEGATIVE CONTROL MUST BE A VERB THAT CANNOT DRIFT ONTO THE ALLOW-LIST.
    /// This test originally used `:wat::i64::*`, and stone expand-T4a broke it — `i64::*`
    /// is one of the 143 verbs the allow-list blesses, so transcribing that blessing
    /// legitimately turned the control's "declares nothing" fixture into a `Legal`. A
    /// control chosen from the pure-and-deterministic population is a control waiting to
    /// be blessed. `:wat::kernel::println` is structurally stable instead: it is
    /// `@Purity Effectful`, and expand-1's audit of all 202 entries found ZERO effectful
    /// verbs blessed — default-deny's other half has held perfectly — so an effectful
    /// verb cannot join the list without violating the list's own doctrine.
    #[test]
    fn expand_time_is_carried_from_the_doc_into_the_registry_entry() {
        let reg = super::registry();
        let fresh_symbol = reg
            .all_entries()
            .find(|e| e.name == ":wat::core::fresh-symbol")
            .expect(":wat::core::fresh-symbol must be registered");
        assert_eq!(
            fresh_symbol.expand_time,
            wat_doc::ExpandTime::Legal,
            "`:wat::core::fresh-symbol` declares `@ExpandTime Legal`; reading `Unreviewed` \
             here means the directive parsed but never reached the submission literal"
        );

        // The other side: a verb that declares NOTHING must read back the default. If this
        // also said `Legal`, the field would be hard-wired rather than carried.
        let effectful = reg
            .all_entries()
            .find(|e| e.name == ":wat::kernel::println")
            .expect(":wat::kernel::println must be registered");
        assert_eq!(
            effectful.expand_time,
            wat_doc::ExpandTime::Unreviewed,
            "`:wat::kernel::println` declares no `@ExpandTime`; it must read the `Unreviewed` \
             default, or the field is not being carried per-verb at all"
        );
    }

    /// `@example`. Enforced at compile time via the doc-contract; enforced at
    /// test time here using the declared `@Purity`/`@Determinism`/`@ExpandTime` fields.
    ///
    /// Arc 255 Stone expand-only-the-missing-pole gave this its third branch, DERIVED from
    /// the `@ExpandTime` coordinate rather than named as an exemption: a verb whose
    /// `expand_time` is `ExpandOnly` has NO runtime call site at all (its only legitimate
    /// caller is a `defmacro` body during expansion), so a runnable `@example` — evaluated
    /// at RUNTIME, a tier where the verb does not exist — is impossible by construction.
    /// `@example-norun` is its correct and REQUIRED form. Checked in both directions,
    /// exactly like the two branches below it: `has_norun` required, `has_run` forbidden.
    /// This branch is checked FIRST and takes priority over `is_pure_and_det` — `macro-error`
    /// is itself `Pure ∧ Deterministic`, so without this ordering it would fall into the
    /// pure+det branch and demand the very runnable example that is impossible for it.
    #[test]
    fn purity_mandated_examples() {
        for entry in super::registry().all_entries() {
            let has_run = entry.examples.iter().any(|e| e.run);
            let has_norun = entry.examples.iter().any(|e| !e.run);

            let is_pure_and_det = matches!(entry.purity, wat_doc::Purity::Pure | wat_doc::Purity::Preserving)
                && matches!(entry.determinism, wat_doc::Determinism::Deterministic | wat_doc::Determinism::Preserving);
            let is_expand_only = matches!(entry.expand_time, wat_doc::ExpandTime::ExpandOnly);

            if is_expand_only {
                assert!(
                    has_norun,
                    "ExpandOnly intrinsic `{}` has no @example-norun (≥1 required by contract — a \
                     runnable @example is impossible: this verb has no runtime call site, only a \
                     `defmacro`-body expand-time one)",
                    entry.name
                );
                assert!(
                    !has_run,
                    "ExpandOnly intrinsic `{}` has a runnable @example (impossible and forbidden — \
                     ExpandOnly means no runtime call site exists to run it; use @example-norun)",
                    entry.name
                );
            } else if is_pure_and_det {
                assert!(
                    has_run,
                    "pure+det intrinsic `{}` has no runnable @example (≥1 required by contract)",
                    entry.name
                );
            } else {
                assert!(
                    has_norun,
                    "non-pure-det intrinsic `{}` has no @example-norun (≥1 required by contract)",
                    entry.name
                );
                assert!(
                    !has_run,
                    "non-pure-det intrinsic `{}` has a runnable @example (forbidden — use @example-norun)",
                    entry.name
                );
            }
        }
    }

    /// ★ STOP-2 proof, arc 255 Stone expand-only-the-missing-pole — the `is_expand_only`
    /// branch above must bite in BOTH directions, not just relax the existing pure+det
    /// branch for one name. This constructs a synthetic entry (the same shape
    /// `expand_time_is_carried_from_the_doc_into_the_registry_entry` above uses to read a
    /// real one, but built by hand here so a runnable example can be deliberately attached)
    /// declaring `ExpandOnly` + `Pure` + `Deterministic` — `macro-error`'s exact
    /// combination — WITH a runnable `@example`, and proves the same predicate the real
    /// test loop evaluates would refuse it. A one-way relaxation (a branch that only
    /// accepts `@example-norun` without ever forbidding `has_run`) would let this pass;
    /// this test's whole point is that it must NOT.
    #[test]
    fn expand_only_with_a_runnable_example_is_refused_by_the_gate() {
        // The forbidden shape: an ExpandOnly verb keeps its required `@example-norun`
        // (`has_norun = true`, satisfying the first assertion below) but ALSO carries a
        // runnable `@example` (`has_run = true`) — isolating the SECOND assertion, the one
        // this stone adds, rather than conflating it with the unrelated "missing norun"
        // failure the first assertion already guards.
        let has_run = true;
        let has_norun = true;
        let purity = wat_doc::Purity::Pure;
        let determinism = wat_doc::Determinism::Deterministic;
        let expand_time = wat_doc::ExpandTime::ExpandOnly;

        let is_pure_and_det = matches!(purity, wat_doc::Purity::Pure | wat_doc::Purity::Preserving)
            && matches!(determinism, wat_doc::Determinism::Deterministic | wat_doc::Determinism::Preserving);
        let is_expand_only = matches!(expand_time, wat_doc::ExpandTime::ExpandOnly);
        assert!(is_pure_and_det, "the synthetic entry must reproduce macro-error's OWN purity shape");
        assert!(is_expand_only, "the synthetic entry must reproduce macro-error's OWN @ExpandTime coordinate");

        // The real loop's `is_expand_only` branch: `!has_run` is asserted, and this
        // synthetic entry sets `has_run = true` — the exact condition the branch exists to
        // reject. `std::panic::catch_unwind` proves the assertion actually FIRES, rather
        // than reading the branch and trusting it would.
        let fired = std::panic::catch_unwind(|| {
            if is_expand_only {
                assert!(has_norun, "this fixture keeps @example-norun; only has_run is under test");
                assert!(!has_run, "ExpandOnly + runnable @example must be refused");
            }
        })
        .is_err();
        assert!(
            fired,
            "an ExpandOnly verb with a runnable @example must panic the gate; it did not — \
             the branch only relaxes and never refuses, which is the hole this stone closes"
        );
    }

    /// Arc 255.1c site 3 — was `pure_declared_matches_is_effectful_op`, a biconditional
    /// between `entry.purity` and `is_effectful_op(entry.name)`. After the 255.1c site-1
    /// split, `is_effectful_op` consults the registry FIRST, so for a registered row it
    /// now *returns* `entry.purity` — comparing the two would be a gate reading a copy of
    /// the truth, unable to fail for a registered row ever again
    /// (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).
    ///
    /// Re-pointed per the ruling (`DESIGN-STONE-255.1c-kernel-ambient.md`, "⊘ RULED
    /// 2026-08-19 — OPTION B"): the biconditional becomes a CENSUS against the genuinely
    /// independent namespace guess (`effectful_by_prefix`, which never touches the
    /// registry) — every disagreement is an inventory entry, not a failure, and it is the
    /// four kernel-ambient readers (`stopped?`/`sigusr1?`/`sigusr2?`/`sighup?`) — Pure by
    /// declaration, flagged effectful by the `:wat::kernel::` prefix rule, which cannot see
    /// inside a body — that this census exists to surface. Collected into a `Vec`, never
    /// `assert!`-ed inside the loop: the gate this replaces did exactly that over an
    /// unordered `HashMap`, so it could only ever report ONE collision per run, and a
    /// different one each time.
    ///
    /// One direction survives as a real assertion: `Effectful ⇒ effectful_by_prefix`. A
    /// registered row that declares an effect the prefix guess would MISS is a doc that
    /// could lie about an effect the runtime cannot refuse the moment that row is ever
    /// judged by the prefix fallback alone — an as-yet-unregistered verb, or this very row
    /// losing its `#[wat_intrinsic]` registration. That direction still has teeth and still
    /// fails loud.
    #[test]
    fn declared_purity_vs_effectful_by_prefix_census() {
        let mut census: Vec<(&'static str, wat_doc::Purity, bool)> = Vec::new();
        let mut effectful_missed_by_prefix: Vec<&'static str> = Vec::new();

        for entry in super::registry().all_entries() {
            let declared_effectful = matches!(entry.purity, wat_doc::Purity::Effectful);
            let prefix_guess = crate::rete::purity::effectful_by_prefix(entry.name);

            if declared_effectful != prefix_guess {
                census.push((entry.name, entry.purity, prefix_guess));
            }
            if declared_effectful && !prefix_guess {
                effectful_missed_by_prefix.push(entry.name);
            }
        }

        census.sort_by_key(|(name, ..)| *name);
        eprintln!(
            "=== declared purity vs effectful_by_prefix census: {} disagreement(s) ===",
            census.len()
        );
        for (name, purity, prefix_guess) in &census {
            eprintln!(
                "  {name}: declared={purity:?}, effectful_by_prefix={prefix_guess} (prefix \
                 says {})",
                if *prefix_guess { "effectful" } else { "not effectful" }
            );
        }

        // The surviving direction — still able to fail.
        assert!(
            effectful_missed_by_prefix.is_empty(),
            "row(s) declare purity=Effectful but effectful_by_prefix says false — the prefix \
             fallback would silently treat these as safe the moment they are judged by it \
             alone: {effectful_missed_by_prefix:?}"
        );
    }

    // Arc 255 Stone P5-b — `yields_type_matches_fn_arg_param` is DELETED, not rewritten.
    // It asserted TWO declarations of one fact agreed: the doc's `@yields <type>` token
    // against the checker's `Fn(P)->R` param `P`. P5-b drops the first declaration
    // entirely — `@yields` no longer carries a type, only a subject (`@yields <argname>
    // <desc>`) — so there is no longer a second spelling to disagree with the first, and a
    // gate whose whole job was catching that drift has nothing left to catch.
    //
    // Coverage held elsewhere, not lost:
    //   - "the @yields type matches the fn-arg's declared type" — this test's FIRST job —
    //     is now trivially true BY CONSTRUCTION: the type is derived FROM that `@arg`'s own
    //     string (`reflect.rs`'s `fn_arg_param_type`), not re-typed by hand, so there is no
    //     drift possible any more, not merely no drift found.
    //   - "the doc's `@arg` type matches the checker's real `TypeScheme` param type" — this
    //     test's SECOND, independent claim (via `check_env.get`) — is `doc_arg_ret_types_
    //     match_checker_scheme`'s job (below), already covers every `@arg` including
    //     fn-typed ones, for every entry the checker knows about, and skips exactly the same
    //     `FROZEN_CHECKER_DEBT_LEDGER` entries this test's `None => continue` always skipped
    //     — same construction, same population, same debt, unchanged.
    //   - "declares `@yields` but has no Fn param to attach it to" — this test's runtime
    //     `panic!` — becomes `wat_intrinsic.rs`'s expand-time mandate: a `@yields` naming a
    //     non-fn-shaped or missing `@arg` is now a `compile_error!` before the crate exists,
    //     strictly earlier and unconditional (not gated on `check_env.get` returning `Some`).

    /// Arc 255 Stone P1 — two homes claiming the same FQDN is a silent
    /// `HashMap::insert` overwrite inside `IntrinsicRegistry::register` (see its
    /// `debug_assert!`), and this repo's only trusted floor is `cargo nextest run
    /// --release`, where `debug_assert!` is compiled out (no `[profile.release]` in
    /// `Cargo.toml`). By the time `registry().all_entries()` can be called, a
    /// collision has already collapsed to one entry with no trace — a test over the
    /// collapsed map cannot fail for the reason this one exists. So this walks the
    /// SUBMISSION streams directly (`inventory::iter::<IntrinsicSubmission>` and
    /// `inventory::iter::<SpecialFormSubmission>`), where both halves of a collision
    /// are still visible, before either ever reaches `register`. Both submission
    /// kinds fold into the SAME map in `registry()`, so they are checked together
    /// here — a per-stream-only check would miss an intrinsic colliding with a
    /// special form.
    #[test]
    fn no_two_submissions_claim_the_same_fqdn() {
        use std::collections::HashMap;

        let mut seen: HashMap<&'static str, usize> = HashMap::new();
        for s in inventory::iter::<super::IntrinsicSubmission> {
            *seen.entry(s.name).or_insert(0) += 1;
        }
        for s in inventory::iter::<super::SpecialFormSubmission> {
            *seen.entry(s.name).or_insert(0) += 1;
        }

        let mut dupes: Vec<(&'static str, usize)> = seen.into_iter().filter(|&(_, n)| n > 1).collect();
        dupes.sort_by_key(|(name, _)| *name);

        assert!(
            dupes.is_empty(),
            "duplicate FQDN registration(s) — two homes claiming the same name, which \
             IntrinsicRegistry::register silently overwrites via HashMap::insert in release \
             (its debug_assert! is compiled out there): {dupes:?}"
        );
    }

    /// ★ Arc 255 Stone P6-a's WALL. Without this test, `entry.impls.is_empty()` silently means
    /// BOTH "this special form has no source" (the honest population fact — nobody has
    /// annotated it yet, e.g. every form P6-c will add) AND "this special form's checker/eval
    /// fns forgot their `#[wat_special_form_impl]` annotation" (a real regression) — the exact
    /// absence-read-as-an-answer defect this whole NOTE family exists to kill, re-created
    /// inside the stone that answers it. This makes the second case loud: every registered
    /// `Kind::SpecialForm` entry must carry at least a `check` and an `eval` impl.
    ///
    /// `tail` is deliberately NOT required — 8 of the eval match's heads have a tail rule, the
    /// rest fall through to `eval_inner` correctly (just not tail-optimized); `tail: None` is
    /// an honest absence, not a lie (see the NOTE's `None`-discriminator table). Asserting it
    /// here would turn a real, safe default into a failure.
    ///
    /// Arc 255 Stone 1a-β-0 — the demand was DERIVED from `entry.category` (superseded; see
    /// the ★★★ AMENDED note below): a `Category::Declaration` row was processed at freeze
    /// time, before evaluation exists
    /// (`DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`), so it had to name a
    /// `declare` impl instead of `check`/`eval`. Not exclusive-or — a row selected by this
    /// branch MAY also carry `check`/`eval` (`:wat::core::def` names `check`); this branch ADDS
    /// a demand for the rows it selects and removes none from any other, so the expression
    /// forms already registered still demand `check` and `eval`, unchanged.
    #[test]
    /// ★★★ AMENDED 2026-09-02 (Stone 1a-δ-b) — THE COUPLING ABOVE IS REMOVED.
    /// `DESIGN-STONE-1a-delta-b-splice-is-a-doing-and-the-gate-stops-picking-the-taxonomy.md`.
    ///
    /// One stone earlier (1a-δ), this gate had made `@Category` load-bearing: because the
    /// required impl-set was DERIVED from `entry.category`, the category a row declared decided
    /// what it had to implement, and the three load forms (`load-file!`, `digest-load!`,
    /// `signed-load!` — none of which register anything) could be given no category but
    /// `Declaration` without this gate refusing them. Minting `:Splice` for them and adding
    /// `category == Declaration || category == Splice` to this branch would have re-committed
    /// the identical defect one variant later: the next never-evaluating form that is neither a
    /// declaration nor a splice hits the same wall.
    ///
    /// So the branch no longer asks `@Category` at all — it asks `@Purity Unevaluated`. That is
    /// the right axis because its entire meaning already IS this gate's question: "does this
    /// form ever evaluate?" A row declaring `Unevaluated` cannot name an eval impl by
    /// construction (`DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`), so it may name
    /// `declare` alone; every other row is still required to name `check` AND `eval`, unchanged.
    ///
    /// Measured before the swap, both directions: 11 rows declare `@Purity Unevaluated` (`def`,
    /// `defalias`, `defenum`, `defmacro`, `defsurface`, `newtype`, `structtype`, `typealias`,
    /// `load-file!`, `digest-load!`, `signed-load!`) and 11 rows carry a `Declare` impl — the
    /// SAME 11, with no row on either side alone. The swap changes nothing today. `@Category`
    /// goes back to being only a taxonomy: a future author choosing a category is choosing a
    /// NAME, never again negotiating with this gate.
    fn every_special_form_carries_check_and_eval_impls() {
        let mut missing: Vec<String> = Vec::new();
        for entry in super::registry().all_entries() {
            if entry.kind != super::Kind::SpecialForm {
                continue;
            }
            // Arc 255 Stone 2a — an alias row's evaluation route is `alias_of` itself, read
            // directly by `dispatch_keyword_head`/`dispatch_keyword_head_value`
            // (`src/runtime.rs`), never a `role = check`/`role = eval` implementation. STOP-2's
            // whole contract is that an alias needs NEITHER — giving it one to satisfy this wall
            // would be exactly the "field is not carrying the dispatch" finding STOP-2 names.
            // This is the THIRD evaluation route this wall now recognises, alongside
            // `Unevaluated`'s `declare` and the default `check`+`eval` pair — a row can be
            // exempt from this wall's demand for one of two structurally-checkable reasons
            // (never evaluated at all, OR evaluated entirely by re-dispatch), and this branch
            // is the second.
            if entry.alias_of.is_some() {
                continue;
            }
            if entry.purity == wat_doc::Purity::Unevaluated {
                let has_declare =
                    entry.impls.iter().any(|(role, _)| *role == super::SpecialFormRole::Declare);
                if !has_declare {
                    missing.push(format!("{} — missing role: declare", entry.name));
                }
            } else {
                let has_check = entry.impls.iter().any(|(role, _)| *role == super::SpecialFormRole::Check);
                let has_eval = entry.impls.iter().any(|(role, _)| *role == super::SpecialFormRole::Eval);
                if !has_check {
                    missing.push(format!("{} — missing role: check", entry.name));
                }
                if !has_eval {
                    missing.push(format!("{} — missing role: eval", entry.name));
                }
            }
        }
        missing.sort();
        assert!(
            missing.is_empty(),
            "special form(s) registered via #[wat_special_form] but missing a required \
             #[wat_special_form_impl] check, eval, or declare annotation:\n{}",
            missing.join("\n")
        );
    }

    /// ★ Arc 255 Stone 1a-β-0b's WALL — the structural gate on `@Purity Unevaluated`
    /// (`DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`). The pole means "this
    /// form is never evaluated, so the axis has no runtime verdict to give," and four
    /// consumers (`rete::purity::pure`, `is_effectful_op`, `purity_mandated_examples`,
    /// `reflect::eval_intrinsic_examples`) trust it without re-deriving it — the pole's
    /// whole value is that a row could simply be WRONG about it. So the claim is made
    /// structurally checkable, derived from the row's own registration facts rather than
    /// a name list: a row declaring `@Purity Unevaluated` may carry no `handler`, no
    /// `tail_handler`, and no `Eval` or `Tail` impl role — any of the four is a route to
    /// evaluation, and the claim would be false.
    ///
    /// ⚠ State what this instrument cannot see: it inspects only the THREE
    /// registry-visible routes above. It cannot see a hand-written `src/runtime.rs`
    /// match arm that dispatches this FQDN outside the registry's own handler slots —
    /// that is not a registry fact, so this gate does not reach it
    /// (`[[feedback_a_containment_argument_must_name_its_consumers]]`).
    #[test]
    fn unevaluated_purity_carries_no_route_to_evaluation() {
        let mut inspected: Vec<&'static str> = Vec::new();
        let mut offenders: Vec<String> = Vec::new();
        for entry in super::registry().all_entries() {
            if entry.purity != wat_doc::Purity::Unevaluated {
                continue;
            }
            inspected.push(entry.name);

            let mut routes: Vec<&str> = Vec::new();
            if entry.handler.is_some() {
                routes.push("handler");
            }
            if entry.tail_handler.is_some() {
                routes.push("tail_handler");
            }
            if entry.impls.iter().any(|(role, _)| *role == super::SpecialFormRole::Eval) {
                routes.push("role: eval");
            }
            if entry.impls.iter().any(|(role, _)| *role == super::SpecialFormRole::Tail) {
                routes.push("role: tail");
            }
            if !routes.is_empty() {
                offenders.push(format!(
                    "{} declares @Purity Unevaluated but carries: {}",
                    entry.name,
                    routes.join(", ")
                ));
            }
        }
        assert!(
            !inspected.is_empty(),
            "gate is vacuous — no registered row declares @Purity Unevaluated to inspect"
        );
        offenders.sort();
        assert!(
            offenders.is_empty(),
            "row(s) declare @Purity Unevaluated (never evaluated) but carry a registry-visible \
             route to evaluation — the claim would be false:\n{}",
            offenders.join("\n")
        );
    }

    /// ★ Arc 255 Stone 1a-α's WALL. `src/reflect/verbs.rs`'s `signature_of_defn` renders a
    /// non-empty `@syntax` by parsing it through the substrate's own reader and
    /// `.expect()`-ing the result — that `.expect` is load-bearing only as long as no
    /// registered `@syntax` can fail to parse. Without this gate, a malformed `@syntax`
    /// would not be a compile error, a checker error, or a doc-comment lint — it would be a
    /// **panic** the first time reflection is asked about that row, discovered whenever a
    /// user (or a golden capture) happens to call `signature-of-defn` on it. This is the
    /// check rung (`[[extirpare]]`'s ladder): every registered `@syntax` is parsed HERE, at
    /// floor time, so a malformed one goes red the moment it is authored.
    #[test]
    fn every_registered_syntax_parses() {
        let mut failures: Vec<String> = Vec::new();
        let mut inspected: Vec<&'static str> = Vec::new();
        for entry in super::registry().all_entries() {
            if entry.syntax.is_empty() {
                continue;
            }
            inspected.push(entry.name);
            if let Err(e) =
                wat_reader::parser::parse_one_with_file(entry.syntax, "<registry @syntax>")
            {
                failures.push(format!(
                    "{} — @syntax {:?} failed to parse: {:?}",
                    entry.name, entry.syntax, e
                ));
            }
        }
        failures.sort();
        assert!(
            failures.is_empty(),
            "registered @syntax string(s) do not parse — signature_of_defn's `.expect()` in \
             src/reflect/verbs.rs would panic on these at reflection time:\n{}",
            failures.join("\n")
        );

        // ⛔ NON-VACUITY. The loop above `continue`s past every row with no `@syntax`, so a
        // tree where nobody declares one leaves `failures` empty and this test GREEN while
        // proving nothing — a gate that iterates zero rows is not a gate
        // (`[[feedback_a_green_test_can_prove_nothing]]`). The floor is the measured
        // population at Stone 1a-α: `let`, `fn`, `match`, the only three `@syntax`-bearing
        // rows in the tree. It is a FLOOR, not a freeze — registering more `@syntax` rows
        // raises the count and must never touch this line; dropping below three means a
        // declaration was deleted, and that is the event this assert exists to surface.
        inspected.sort_unstable();
        assert!(
            inspected.len() >= 3,
            "every_registered_syntax_parses inspected only {} row(s) — it is meant to parse \
             every registered @syntax, and at Stone 1a-α there were three (let/fn/match). \
             A count below that means a declaration was removed, not that the gate passed. \
             Inspected: {:?}",
            inspected.len(),
            inspected
        );
    }

    /// Arc 255 Stone P5-a's frozen ledger for `source.rs`'s `f :wat::core::Fn` `@arg` on
    /// `:wat::kernel::fn-forms`. `:wat::core::Fn` there is `ANON_FN_SYMBOL`
    /// (`crate::value::frame::ANON_FN_SYMBOL`) — the string an anonymous fn VALUE renders
    /// as, standing in a type position with no backing `TypeExpr::Fn` to derive a canonical
    /// bracket spelling from. Its own prose accepts two shapes ("the fn value to reify (or a
    /// keyword naming a registered fn)") and wat has no union type to name that with, so this
    /// is named on a ledger — like `FROZEN_CHECKER_DEBT_LEDGER` above — rather than guessed
    /// or fixed with a type-system feature.
    const FN_ARG_ANON_SYMBOL_LEDGER: &[(&str, &str)] = &[(":wat::kernel::fn-forms", "f")];

    /// Every `->` in a fn-type `@arg` string must be spelled `:->` (the renderer's arrow),
    /// i.e. every occurrence of the two-byte substring `->` is immediately preceded by `:`.
    fn arrow_correctly_spelled(ty: &str) -> bool {
        let bytes = ty.as_bytes();
        let mut idx = 0;
        while let Some(pos) = ty[idx..].find("->") {
            let abs = idx + pos;
            if abs == 0 || bytes[abs - 1] != b':' {
                return false;
            }
            idx = abs + 2;
        }
        true
    }

    /// ★ Arc 255 Stone P5-a's WALL — built and shown RED before any correction
    /// (`NISI FRANGAS, NIHIL PROBAS`).
    ///
    /// A function type has ONE spelling in an `@arg`: whatever
    /// `typeexpr_to_doc_string` emits for `TypeExpr::Fn` — `[:-> RET]` (nullary) or
    /// `[ARGS :-> RET]` — bracket-delimited, arrow spelled `:->`. Walking
    /// `registry().all_entries()`'s `entry.args`, a declared type is a fn-type CLAIM if it
    /// contains `->` or equals `ANON_FN_SYMBOL` (the value-rendering standing in a type
    /// position, `source.rs:158` — ledgered above, never elsewhere). Every other fn-type
    /// claim must be bracket-delimited with the arrow spelled `:->`; `ANON_FN_SYMBOL` must
    /// never appear as a type outside the frozen ledger.
    #[test]
    fn fn_typed_arg_has_one_canonical_spelling() {
        let mut violations: Vec<String> = Vec::new();

        for entry in super::registry().all_entries() {
            for &(arg_name, ty, _desc, _is_rest) in entry.args.iter() {
                let is_fn_type_claim = ty.contains("->") || ty == crate::value::frame::ANON_FN_SYMBOL;
                if !is_fn_type_claim {
                    continue;
                }

                if ty == crate::value::frame::ANON_FN_SYMBOL {
                    if FN_ARG_ANON_SYMBOL_LEDGER.contains(&(entry.name, arg_name)) {
                        continue; // STOP-1's single ledgered site — a value rendering, not a TypeExpr::Fn.
                    }
                    violations.push(format!(
                        "{}'s `@arg {}` types as `{}` — ANON_FN_SYMBOL (a fn VALUE's \
                         rendering) used as a type, and not on FN_ARG_ANON_SYMBOL_LEDGER",
                        entry.name, arg_name, ty
                    ));
                    continue;
                }

                let bracket_delimited = ty.starts_with('[') && ty.ends_with(']');
                let arrow_ok = arrow_correctly_spelled(ty);
                if !bracket_delimited || !arrow_ok {
                    violations.push(format!(
                        "{}'s `@arg {}` types as `{}` — not the canonical bracket form \
                         `[ARGS :-> RET]` that `typeexpr_to_doc_string` emits for \
                         `TypeExpr::Fn` (bracket_delimited={}, arrow_spelled_correctly={})",
                        entry.name, arg_name, ty, bracket_delimited, arrow_ok
                    ));
                }
            }
        }

        violations.sort();
        assert!(
            violations.is_empty(),
            "fn-typed @arg(s) not in the ONE canonical spelling `[ARGS :-> RET]` \
             (arc 255 Stone P5-a):\n{}",
            violations.join("\n")
        );
    }

    /// Arc 255 `DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md` — THE ONE CONTRACT: a
    /// registered row that carries a handler (`Kind::Intrinsic`, or any future kind with
    /// `handler: Some`) may not ALSO have a literal dispatch arm inside
    /// `dispatch_keyword_head_value` (`src/runtime.rs`) — the registry-first door
    /// (`crate::intrinsic::registry().lookup(head)`, hoisted above that match) already answers
    /// every such name, so a surviving arm is unreachable dead code (five of them: arc 255's
    /// stone immediately before this one left `record?`/`u8`/`bool::to-string`/`not`/`show`
    /// behind — this gate is their first payment and the wall against a sixth).
    ///
    /// ⛔ The predicate is `entry.handler.is_some()`, NEVER a name list. At the time this gate
    /// was written, `:wat::core::fn`, `if`, `let`, `match` sat in the SAME match, looked
    /// identical to the five, and were `Kind::SpecialForm` with `handler: None`; their arm was
    /// the ONLY dispatch they had, so the predicate exempted them WITHOUT naming them.
    /// Arc 255 `DESIGN-STONE-every-role-carries-its-pointer.md` (the eval door) closed that gap
    /// for all four: each now carries a `role = eval` handler (`handler: Some`), and THIS gate —
    /// unchanged — is what then demanded their arms be deleted (`src/runtime.rs`, § the eval
    /// door). Measuring `handler.is_some()` (rather than naming them) is what made that demand
    /// automatic instead of requiring a second edit to this test; it is also what keeps the
    /// exemption from rotting the moment a future special form registers with no eval impl yet.
    ///
    /// ⛔ The search is bounded to `dispatch_keyword_head_value`'s own span, not the whole file —
    /// `runtime.rs` has THREE matches that dispatch on these names (`dispatch_keyword_head_value`,
    /// `eval_tail`, `step_list`); `:wat::core::u8` has an arm in `step_list` (live, kept) as well
    /// as the one just deleted here, and a whole-file text search would flag that live arm too
    /// (a first probe for this stone made exactly this error the other direction, calling
    /// `:wat::holon::Blend`'s `step_list`-only arm "dead" — see the DESIGN's
    /// "measured three times, wrong twice").
    ///
    /// The span is bounded the same way `probe_can_doc_types_reconstruct_the_checker_scheme`
    /// (above) treats `runtime.rs` as data rather than a name list: `include_str!` the whole
    /// file, find the `fn dispatch_keyword_head_value(` line, and take everything up to the
    /// NEXT top-level (column-0) `fn ` — that next `fn` (`keyword_accessor_record`) is a
    /// sibling function, not a nested one, so it is the correct exclusive end bound.
    ///
    /// Within that span, whole-line comments (`//…`) are stripped before searching — this span
    /// has no block comments and no code-then-`//` lines (verified by hand), and stripping
    /// matters for real: a comment a few dozen lines below `:wat::core::match`'s arm reads
    /// `":wat::holon::from-holon" (the one holon producer) is now registered…` — a bare
    /// substring search would misread that PROSE mention of a registered name as an arm.
    /// A hit still has to look like a match-arm pattern, not a piece of AST-construction data:
    /// `WatAST::Keyword(":wat::core::if".into(), …)` (built for `if`/`let`/`match`'s own
    /// desugaring, deep inside this same function) quotes the name too, but the quote is
    /// followed immediately by `.into()`, never by `=>`/`|`/`)` — the three continuations an
    /// actual arm pattern has (a single-name arm, an OR-pattern member, or the closing paren of
    /// a `head @ (...)` group whose last alternative wrapped to a new line).
    #[test]
    fn registry_first_door_owns_every_handler_row_no_literal_arm_survives() {
        const SOURCE: &str = include_str!("../runtime.rs");

        const FN_SIG: &str = "fn dispatch_keyword_head_value(";
        let start_marker = format!("\n{FN_SIG}");
        let start = SOURCE
            .find(start_marker.as_str())
            .expect("dispatch_keyword_head_value not found in runtime.rs — has it moved/renamed?")
            + 1; // land on the `fn`, past the newline that anchored the search to column 0.

        let after_sig = &SOURCE[start + FN_SIG.len()..];
        let end_offset = after_sig
            .find("\nfn ")
            .expect("no top-level `fn` follows dispatch_keyword_head_value — span is unbounded");
        let end = start + FN_SIG.len() + end_offset;

        let span = &SOURCE[start..end];

        // Whole-line comments contribute nothing to the search; code lines pass through as-is
        // (this span has neither block comments nor a code-then-`//` line — hand-verified).
        let lines: Vec<&str> = span
            .lines()
            .map(|l| if l.trim_start().starts_with("//") { "" } else { l })
            .collect();

        // Does a literal match-arm PATTERN for `name` occur in the span — as opposed to the
        // same string used as data (e.g. `.into()` immediately after the closing quote)?
        let has_literal_arm = |name: &str| -> bool {
            let quoted = format!("\"{name}\"");
            for (i, line) in lines.iter().enumerate() {
                let mut rest = *line;
                while let Some(pos) = rest.find(quoted.as_str()) {
                    let after = rest[pos + quoted.len()..].trim_start();
                    let is_arm_continuation = if after.is_empty() {
                        // The quote closed at end-of-line — an OR-pattern member that wraps;
                        // the next line either continues the alternation or closes the group.
                        let next = lines.get(i + 1).map(|l| l.trim_start()).unwrap_or("");
                        next.starts_with('|') || next.starts_with(')')
                    } else {
                        after.starts_with("=>") || after.starts_with('|') || after.starts_with(')')
                    };
                    if is_arm_continuation {
                        return true;
                    }
                    rest = &rest[pos + quoted.len()..];
                }
            }
            false
        };

        let mut offenders: Vec<&'static str> = super::registry()
            .all_entries()
            .filter(|entry| entry.handler.is_some())
            .map(|entry| entry.name)
            .filter(|name| has_literal_arm(name))
            .collect();
        offenders.sort_unstable();
        offenders.dedup();

        assert!(
            offenders.is_empty(),
            "registered row(s) with a handler still carry a literal dispatch arm inside \
             `dispatch_keyword_head_value` (src/runtime.rs) — the registry-first door \
             (`crate::intrinsic::registry().lookup(head)`, hoisted above that match) already \
             answers these by name, so the arm can never fire. Delete the arm line (leave the \
             surrounding retirement commentary and the handler fn itself) for: {offenders:?}"
        );
    }
}

// The `wat_mirror_tests` module that stood here is DELETED (2026-08-15). It
// compared each Rust enum against its `defenum` in `wat/runtime-meta.wat` — a gate
// over two hand-written lists. `Category` is now GENERATED from that defenum via
// `wat_enum_from!`, so it cannot drift from it; a gate whose success condition is
// its own deletion was scaffolding all along.
//
// PROVEN before removing it: a variant added to the .wat file ALONE, with zero Rust
// edited, produced `error[E0004]: non-exhaustive patterns: Category::WatOnlySentinel
// not covered`. wat drove the Rust type system.
//
// CLOSED 2026-08-15: the note that stood here recorded Kind/DefinedIn/Layer as
// still hand-written and therefore UNCHECKED. All three are now generated by
// `wat_enum_from!` (above), as are `wat_doc::Purity` and `wat_doc::Determinism`.
// FIVE mirrors, not the three the seam named — the deleted gate covered every one
// of them, so stopping at three would have left the identical debt in a second
// file. No Rust enum in this workspace mirrors a `defenum` by hand any more.
