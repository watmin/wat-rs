//! Intrinsic registry — arc 255. The home where wat **intrinsics** (callables
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
pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
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
    /// Restringified handler source — `quote!(handler_fn).to_string()`.
    /// Faithful-if-reformatted (token restringify; comments may be lost).
    /// Consumed by `(:wat::core::show-source <fqdn>)`.
    pub source: &'static str,
    /// Declared purity from `@Purity <Variant>` in the doc.
    pub purity: wat_doc::Purity,
    /// Declared determinism from `@Determinism <Variant>` in the doc.
    pub determinism: wat_doc::Determinism,
    /// `@Category <Variant>` — functional category.
    pub category: wat_doc::Category,
    /// `@yields <type> <desc>` type token — the type handed into the fn-arg callback.
    /// `None` when the intrinsic does not yield to a callback.
    pub yields_type: Option<&'static str>,
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
    pub purity: wat_doc::Purity,
    pub determinism: wat_doc::Determinism,
    pub category: wat_doc::Category,
    pub deprecated: Option<(&'static str, &'static str)>,
}

inventory::collect!(SpecialFormSubmission);

/// One registered intrinsic's full baseline. `handler` is consumed by the
/// runtime dispatch route (`lookup`); `name`/`arity`/`prose`/`added`/`ret` are
/// consumed by `metadata-of`'s intrinsic branch (`lookup_entry`); `args`/
/// `examples`/`see` are carried for iv-b2's doctest verifier seam;
/// `source` is consumed by `show-source` — every field has a reader.
pub(crate) struct IntrinsicEntry {
    pub name: &'static str,
    /// The native dispatch handler. `Some` for `Kind::Intrinsic`; `None` for
    /// `Kind::SpecialForm` (special forms are dispatched by the runtime engine,
    /// not by a registered Rust fn).
    pub handler: Option<NativeHandler>,
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
    /// Restringified handler source (consumed by `show-source` / 255.1b-v).
    pub source: &'static str,
    #[allow(dead_code)] // read by declared_purity_vs_effectful_by_prefix_census + purity_mandated_examples (cfg(test)) + eval_metadata_of + eval_render_doc + reflect.rs's eval_intrinsic_examples (arc 255.1c site 2)
    /// Declared purity — from `@Purity <Variant>` in the doc.
    pub purity: wat_doc::Purity,
    #[allow(dead_code)] // read by purity_mandated_examples (cfg(test)) + eval_metadata_of + eval_render_doc + reflect.rs's eval_intrinsic_examples (arc 255.1c site 2)
    /// Declared determinism — from `@Determinism <Variant>` in the doc.
    pub determinism: wat_doc::Determinism,
    /// `@Category <Variant>` — functional category.
    /// Consumed by `metadata-of`'s intrinsic branch and `eval_render_doc`.
    pub category: wat_doc::Category,
    /// `@yields <type>` type token — the element type handed to the fn-arg callback.
    /// `None` when the intrinsic does not yield to a callback.
    /// Consumed by `yields_type_matches_fn_arg_param` (cfg(test)) and `eval_render_doc`.
    #[allow(dead_code)] // read by yields_type_matches_fn_arg_param (cfg(test)) + render-doc
    pub yields_type: Option<&'static str>,
}

/// `name → entry`. Built once at startup; the dispatch route reads `handler`
/// via `lookup`, `metadata-of` reads the baseline via `lookup_entry`.
pub(crate) struct IntrinsicRegistry {
    entries: std::collections::HashMap<&'static str, IntrinsicEntry>,
}

impl IntrinsicRegistry {
    fn new() -> Self { IntrinsicRegistry { entries: std::collections::HashMap::new() } }

    /// Register an intrinsic's full baseline. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN).
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
                source: submission.source,
                purity: submission.purity,
                determinism: submission.determinism,
                category: submission.category,
                yields_type: submission.yields_type,
            });
        }
        // Each `#[wat_special_form("<fqdn>")]` struct submits a SpecialFormSubmission
        // via `inventory`; fold them into the registry as Kind::SpecialForm entries.
        for submission in inventory::iter::<SpecialFormSubmission> {
            r.register(IntrinsicEntry {
                name: submission.name,
                handler: None,
                kind: Kind::SpecialForm,
                syntax: submission.syntax,
                arity: Arity::Variadic, // special forms handle their own arity
                prose: submission.prose,
                added: submission.added,
                args: submission.args,
                ret_type: submission.ret_type,
                ret: submission.ret,
                examples: submission.examples,
                deprecated: submission.deprecated,
                see: submission.see,
                source: "",
                purity: submission.purity,
                determinism: submission.determinism,
                category: submission.category,
                yields_type: None,
            });
        }
        r
    })
}

mod bigint;
mod bytes;
mod char;
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
mod map;
// Arc 255 Stone HOME-10 — math, stat, seq get actual homes. Pure re-registration
// (HOME-9 already renamed these off `:wat::std::`); shim-only, three files, no
// `src/math/`/`src/stat/`/`src/seq/` directory (STOP-1 — no algebra worth naming).
mod math;
mod rational;
mod reflect;
mod regex;
mod seq;
mod stat;
// Arc 255 Stone F — reverted to the default-private shape of its siblings. The
// `:wat::core::String/*` uppercase alias arms in `runtime.rs` (Stone 237.3) that needed
// `pub(crate)` to call four of these handlers directly (bypassing the registry) are retired;
// nothing outside this module calls into it anymore.
mod string;
mod uuid;
mod vec;
mod vector;
mod witness;
mod special;
mod time;

// ─── Arc 255.1b-v: @see registry-check + firm-doc tests ──────────────────────
//
// Consumer-side tests: every `@see` FQDN must resolve; doc arg/ret types must
// match the checker's TypeScheme; pure+det intrinsics must carry ≥1 runnable
// example. The walks live in `reflect::check_see_refs()` (cfg(test)) and
// inline in the tests below.
#[cfg(test)]
mod tests {
    /// Arc 255.1b-v: every `@see` FQDN in the intrinsic corpus must resolve
    /// to a registered intrinsic. A dangling @see is a broken cross-reference
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

    /// Arc 255.1b-firm: the doc's `@arg`/`@ret` type strings must match the
    /// checker's `TypeScheme` for each registered intrinsic. A mismatch is a
    /// doc lie — the user reads one type, the checker enforces another.
    ///
    /// For variadic args (`is_rest=true`): the doc's element type must match the
    /// ELEMENT of `scheme.rest_param_type` (a `Vector<elem>` in the scheme).
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
                if crate::runtime::is_type_var_path(p) {
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
    /// `@example`. Enforced at compile time via the doc-contract; enforced at
    /// test time here using the declared `@Purity`/`@Determinism` fields.
    #[test]
    fn purity_mandated_examples() {
        for entry in super::registry().all_entries() {
            let has_run = entry.examples.iter().any(|e| e.run);
            let has_norun = entry.examples.iter().any(|e| !e.run);

            let is_pure_and_det = matches!(entry.purity, wat_doc::Purity::Pure | wat_doc::Purity::Preserving)
                && matches!(entry.determinism, wat_doc::Determinism::Deterministic | wat_doc::Determinism::Preserving);

            if is_pure_and_det {
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
            let prefix_guess = crate::runtime::effectful_by_prefix(entry.name);

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

    /// Arc 255 spec-complete: for entries with `@yields`, the declared type must match
    /// the fn-arg's (Fn(P)->R) param type P in the checker's TypeScheme.
    /// A mismatch is a doc lie — the user reads one callback-param type, the checker
    /// enforces another.
    #[test]
    fn yields_type_matches_fn_arg_param() {
        use crate::check::CheckEnv;
        use crate::types::TypeEnv;

        let type_env = TypeEnv::new();
        let check_env = CheckEnv::with_builtins_and_types(&type_env);

        for entry in super::registry().all_entries() {
            let yields_type = match entry.yields_type {
                Some(yt) => yt,
                None => continue, // no @yields — skip
            };

            let scheme = match check_env.get(entry.name) {
                Some(s) => s,
                None => continue, // not yet in checker — skip
            };

            // Find the arg whose scheme type is Fn(P)->R; assert @yields type == P.
            // The Fn arg is identified by TypeExpr::Fn in scheme.params.
            let mut found_fn_param = false;
            for param_ty in &scheme.params {
                if let crate::types::TypeExpr::Fn { args: fn_args, .. } = param_ty {
                    // @yields type must match the first (and only) Fn param.
                    let param_ty_str = if fn_args.len() == 1 {
                        typeexpr_to_doc_string(&fn_args[0])
                    } else {
                        continue;
                    };
                    assert_eq!(
                        yields_type, param_ty_str.as_str(),
                        "doc `@yields` type for `{}` says `{}`, \
                         but the fn-arg's scheme Fn(P)->R param P says `{}`",
                        entry.name, yields_type, param_ty_str
                    );
                    found_fn_param = true;
                    break;
                }
            }
            if !found_fn_param {
                panic!(
                    "intrinsic `{}` declares `@yields {}` but its TypeScheme has \
                     no Fn(P)->R param — register a Fn param in check.rs",
                    entry.name, yields_type
                );
            }
        }
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
