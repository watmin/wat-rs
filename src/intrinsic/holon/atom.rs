//! `:wat::holon::*` bare-op intrinsics — arc 255 Stone HOME-8 (the VSA
//! surface gets a home), registry half. The 60 verbs that are not
//! `Hologram/*` (`hologram.rs`), `Engram*/*` (`engram.rs`),
//! `OnlineSubspace/*` (`subspace.rs`), or `Reckoner/*` (`reckoner.rs`) —
//! everything else HolonAST-shaped: the algebra constructors (`Bind`,
//! `Bundle`, `Permute`, `Blend`, `Thermometer`, the classified-collection
//! constructors `Map`/`Set`/`Vector`/`List`/`Tuple`), the `Value ⇄
//! HolonAST ⇄ WatAST` conversions (`Atom`, `to-holon`, `from-holon`,
//! `leaf`, `from-wat`, `to-wat`, `literal`, `to-record`), the classifier
//! predicates and projections (`is?`/`is-*?`, `extract-classifier`,
//! `Bind/left`/`Bind/right`, `Bundle/children`/`Bundle/first`), the
//! Thermometer/term surface (`therm-form`, `term::*`, `presence-floor`,
//! `coincident-floor`), the measurement primitives (`cosine`, `dot`,
//! `presence?`, `coincident?`, `coincident-explain`, `simhash`), the
//! `eval-*-coincident?` family, and the raw-`Vector` mirrors
//! (`encode`, `vector-bytes`, `bytes-vector`, `vector-bind`,
//! `vector-bundle`, `vector-blend`, `vector-permute`, `statement-length`).
//!
//! Every handler below is a thin binding shim: it evaluates its wat-side
//! args and delegates to the pure algebra in [`crate::holon`] (`ast.rs`,
//! `outcome.rs`, `require.rs` — lifted out of `runtime.rs` by the sibling
//! strike, `d43f758870`) or to the external `holon` VSA crate directly.
//! None of these bodies changed; only their position (a `runtime.rs` match
//! arm vs. a registry entry here) and their parameter ORDER (`list_span`
//! moved last, to match `#[wat_intrinsic]`'s variadic calling convention)
//! did.
//!
//! **`eval_holon_from_holon` (`from-holon`) is the ONE producer** among
//! these 60: pre-carve, it was the only one of the ~95 holon arms hoisted
//! into `dispatch_keyword_head`'s small `Result<TrackedValue, _>` fast
//! path (`runtime.rs`, "Producers + forms that preserve provenance"),
//! stamping `Provenance::RuntimeBuilt`. It keeps its
//! `Result<TrackedValue, EvalBreak>` return type here; Stone G's `sniff_return`
//! forwards it un-rewrapped. Every other verb below returned bare `Value`
//! before this carve (wrapped as `Provenance::Unknown` by
//! `dispatch_keyword_head`'s fallback) and still does — carrying that
//! forward is the behaviour-preserving move; stamping NEW provenance on
//! any of them would be a behaviour change this stone does not make.
//!
//! **Categories are a FRESH judgment call** — `:wat::holon::` had zero
//! registry presence before this stone, so there is no prior `@Category`
//! precedent to inherit for the VSA surface specifically. `Combine` (a
//! larger value built from operands of the same shape: `Bind`, `Bundle`,
//! `Blend`, and their raw-`Vector` mirrors); `Transform` (constructs or
//! converts a value: `Atom`, `to-holon`, `leaf`, `from-wat`, `to-wat`,
//! `from-holon`, `to-record`, `literal`, `encode`, `vector-bytes`,
//! `bytes-vector`, `simhash`, `statement-length`, `Permute`, `Thermometer`,
//! and the classified-collection constructors); `Projection` (extracts a
//! component: `term::template`/`slots`/`ranges`, `extract-classifier`,
//! `Bind/left`/`Bind/right`, `Bundle/children`/`Bundle/first`); `Probe`
//! (answers a question: every `is-*?`/`is?`, `presence?`, `coincident?`,
//! `coincident-explain`, the `eval-*-coincident?` family, `term::matches?`,
//! `presence-floor`, `coincident-floor`, `cosine`, `dot`, `vector-bind`,
//! `vector-bundle`, `vector-blend`, `vector-permute`).
//!
//! Wait — `vector-bind`/`vector-bundle`/`vector-blend`/`vector-permute`
//! return a `CombineOutcome` (a NEW vector or a dimension-mismatch arm),
//! same shape as `Bind`/`Bundle`/`Blend`/`Permute` one layer up (HolonAST
//! vs. raw `Vector`) — filed as `Combine` alongside their AST-level
//! counterparts, not `Probe`.
//!
//! `@Purity`/`@Determinism`: the `eval-*-coincident?` family (six verbs)
//! evaluates ARBITRARY embedded wat source (`parse_and_run`,
//! `run_ast_arg_for_eval_coincident`) before comparing the results — filed
//! `Effectful`/`Nondeterministic` since the embedded program can do
//! anything, including I/O. Everything else here is a pure function of its
//! arguments over the program's fixed encoding config (dimension, floors) —
//! `Pure`/`Deterministic`.
//!
//! Only four `:wat::holon::` verbs are rete-classified
//! (`src/rete/purity.rs:647`, builder-ruled 2026-08-01) — none of the 60
//! below are among them, and this carve adds none (STOP-7).
//!
//! arc 255 Stone H-1b — 52 of these 60 handlers declare their real fixed
//! arity (`&WatAST` per parameter) instead of `args: &[WatAST]`; their
//! hand-rolled arity checks are gone, replaced by the check
//! `#[wat_intrinsic]` now generates from the declared parameter count,
//! exactly as Stone H-1a did for `subspace.rs`/`engram.rs`/`reckoner.rs`/
//! `hologram.rs`. Eight stay variadic: `from-holon` is genuinely 1-or-3
//! arity (STOP-1, untouched); `literal` and the six `eval-*-coincident?`
//! verbs each hand off their arity check to a helper shared with another
//! call site whose error shape (`op` text for `literal`; `RuntimeErrorKind`
//! itself — `MalformedForm`, not `ArityMismatch` — for the coincident
//! family) would silently change under the generated check, so they were
//! left exactly as they were (STOP-2). See
//! `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-H-holon-adopts-the-kernels-interface.md`.

use std::sync::Arc;
use std::collections::HashSet;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::holon::*;
use crate::runtime::{
    coincident_of_two_values, coincident_q_from_values, cosine_outcome_from_values,
    dot_outcome_from_values, eval_form_digest_coincident_shared,
    eval_form_signed_coincident_shared, eval_inner, eval_quote, expect_string_value,
    pair_values_to_vectors, parse_and_run, parse_projection_args, presence_q_from_values,
    program_dim, project_surface_attrs, require_bundle, require_encoding_ctx, require_i64,
    run_ast_arg_for_eval_coincident, wrap_as_eval_result, PairedVectors,
};
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, Provenance, RuntimeError, RuntimeErrorKind, SymbolTable,
    TrackedValue, Value, ValueSnapshot, AggregateValue,
};
use holon::{encode, HolonAST, Similarity};

/// `(:wat::holon::from-holon h)` -> the wat `Value` a HolonAST composition
/// `h` decodes back to — the inverse of `to-holon`/`leaf`/`Atom`/`Map`/
/// `Set`/`Vector`/`List`/`Tuple`. A 3-arg form
/// `(from-holon h -> (:wat::core::HashMap :- [K V]))` disambiguates an
/// empty `Map`-classified Bundle, which is otherwise shape-indistinguishable
/// from an empty `Set`/`Vector`/`List`/`Tuple`. Raises on a bare,
/// unclassified Bundle or other non-decodeable composite.
///
/// Arc 225 Stone 225.1 — renamed from `:wat::core::atom-value` (namespace move +
/// honest rename). Kept as the gravestone: erase it and the next reader re-mints
/// the retired spelling innocently.
///
/// THE ONE PRODUCER in this file: stamps `Provenance::RuntimeBuilt` (see
/// module doc).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     args… :wat::core::Value the HolonAST to decode, alone or with a `-> :T` type-hint suffix
/// @ret     :T the decoded wat value
/// @example (:wat::holon::from-holon (:wat::holon::leaf "role")) #=> (:wat::holon::from-holon (:wat::holon::leaf "role"))
/// @see     :wat::holon::to-holon
#[wat_intrinsic(":wat::holon::from-holon")]
pub(crate) fn eval_holon_from_holon(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    const OP: &str = ":wat::holon::from-holon";
    // Accepts 1 arg (no type hint) or 3 args with optional `-> :T` annotation
    // for disambiguating empty Bundle: `(from-holon h -> (:wat::core::HashMap :- [K V]))`.
    // Arc 216 Stone 3 — the 3-arg form is the only way to signal "empty Bundle = empty HashMap".
    if args.len() != 1 && args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    // Extract optional type annotation from `-> :T` suffix.
    // When args.len() == 3: args[0] = holon expr, args[1] = `->` symbol, args[2] = type keyword.
    let _hint_is_hashmap = if args.len() == 3 {
        // Validate `->` symbol.
        match &args[1] {
            WatAST::Symbol(s, _) if s.as_str() == "->" => {}
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "expected `->` at position 2 for type annotation; got {}",
                            other.variant_name()
                        ),
                    },
                )
                .into());
            }
        }
        // Check if the type keyword starts with :wat::core::HashMap.
        // Keywords include the leading colon in their value (":wat::core::HashMap").
        match &args[2] {
            WatAST::Keyword(k, _) => k.starts_with(":wat::core::HashMap"),
            other => {
                return Err(RuntimeError::new(
                    other.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "expected type keyword after `->` for annotation; got {}",
                            other.variant_name()
                        ),
                    },
                )
                .into());
            }
        }
    } else {
        false
    };
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    let holon = match v {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // Arc 230: Symbol/Keyword/Nil variants retired. Recognise via accessors.
    // These checks must come before the `match &*holon` because the compositions
    // are Bind variants that would otherwise fall to the classifier-dispatch arm.
    // Arc 233 Stone 233.2.j: construct TrackedValue::new directly (no Value::Tracked wrap).
    let prov = || Provenance::RuntimeBuilt {
        producer: ":wat::holon::from-holon",
        call_span: list_span.clone(),
    };
    if let Some(s) = holon.as_symbol() {
        // nil composition (symbol("nil")) → Value::Unit.
        if s == "nil" {
            return Ok(TrackedValue::new(Value::Unit, prov()));
        }
        return Ok(TrackedValue::new(
            Value::wat__core__keyword(Arc::new(s.to_string())),
            prov(),
        ));
    }
    if let Some(s) = holon.as_keyword() {
        // Keyword composition: restore leading colon for the Value round-trip.
        return Ok(TrackedValue::new(
            Value::wat__core__keyword(Arc::new(format!(":{}", s))),
            prov(),
        ));
    }
    match &*holon {
        // Arc 221 Stone 221.2 — HolonAST::Char leaf → Value::wat__core__Char.
        // Arc 233 Stone 233.2.j — use TrackedValue::new directly.
        HolonAST::Char(c) => Ok(TrackedValue::new(Value::wat__core__Char(*c), prov())),
        HolonAST::String(s) => Ok(TrackedValue::new(
            Value::String(Arc::new(s.to_string())),
            prov(),
        )),
        HolonAST::I64(n) => Ok(TrackedValue::new(Value::i64(*n), prov())),
        HolonAST::F64(x) => Ok(TrackedValue::new(Value::f64(*x), prov())),
        HolonAST::Bool(b) => Ok(TrackedValue::new(Value::bool(*b), prov())),
        HolonAST::Atom(inner) => Ok(TrackedValue::new(
            Value::holon__HolonAST(inner.clone()),
            prov(),
        )),
        // Arc 228 Stone 228.1 — classifier-dispatch replaces arc 216 heuristic Bundle dispatch.
        // The outermost form is now Bind(Atom(String(name)), Bundle(items)) for all collections.
        // Dispatch by classifier name:
        //   "Map"    → HashMap (arbitrary-K Binds in inner Bundle)
        //   "Set"    → HashSet (bare items in inner Bundle)
        //   "Vector" → Vec    (positional I64-key Binds in inner Bundle)
        //   "List"   → List   (bare items in inner Bundle, order-preserving)
        //   "Tuple"  → Tuple  (positional I64-key Binds in inner Bundle)
        // Per HARD CUT discipline: bare Bundles (no classifier) error with diagnostic.
        // The `-> (HashMap :- [K V])` consumer-hint form is preserved for empty-Map classifier.
        other => {
            if let Some(classifier) = extract_classifier(other) {
                // Classifier-wrapped collection: extract inner Bundle items.
                let inner_bundle = match other {
                    HolonAST::Bind(_, inner) => inner,
                    _ => unreachable!("extract_classifier returned Some for non-Bind"),
                };
                let items = match inner_bundle.as_ref() {
                    HolonAST::Bundle(items) => items,
                    _ => {
                        return Err(RuntimeError::new(
                            args[0].span().clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: OP.into(),
                                expected:
                                    "classifier-wrapped Bundle (Bind(Atom(name), Bundle(...)))",
                                got: Box::new(ValueSnapshot::unavailable(
                                    "classifier-wrapped non-Bundle inner",
                                )),
                            },
                        )
                        .into());
                    }
                };
                match classifier.as_str() {
                    "Map" => {
                        // Map: inner Bundle contains Bind(K, V) pairs → HashMap.
                        let n = items.len();
                        // Empty Map always returns empty HashMap regardless of hint.
                        #[allow(clippy::mutable_key_type)]
                        let mut map: std::collections::HashMap<
                            Value,
                            Value,
                        > = std::collections::HashMap::with_capacity(n);
                        for child in items.iter() {
                            match child {
                                HolonAST::Bind(k_holon, v_holon) => {
                                    let k_val = from_holon_item(k_holon, OP, args[0].span())?;
                                    let v_val = from_holon_item(v_holon, OP, args[0].span())?;
                                    map.insert(k_val, v_val);
                                }
                                _ => {
                                    return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                                        op: OP.into(),
                                        expected: "Bind(K, V) child in Map classifier-Bundle",
                                        got: Box::new(ValueSnapshot::unavailable("non-Bind child in Map classifier-Bundle inner items"))
                                    }).into());
                                }
                            }
                        }
                        Ok(TrackedValue::new(
                            Value::wat__std__HashMap(Arc::new(map)),
                            prov(),
                        ))
                    }
                    "Set" => {
                        // Set: inner Bundle contains bare items → HashSet.
                        let mut set: HashSet<Value> = HashSet::with_capacity(items.len());
                        for item in items.iter() {
                            let v = from_holon_item(item, OP, args[0].span())?;
                            set.insert(v);
                        }
                        Ok(TrackedValue::new(
                            Value::wat__std__HashSet(Arc::new(set)),
                            prov(),
                        ))
                    }
                    "Vector" => {
                        // Vector: inner Bundle contains positional Bind(I64, _) pairs → Vec.
                        let n = items.len();
                        let mut pairs: Vec<(i64, Value)> = Vec::with_capacity(n);
                        for child in items.iter() {
                            match child {
                                HolonAST::Bind(k, v) => {
                                    let idx = match k.as_ref() {
                                        HolonAST::I64(i) => *i,
                                        _ => {
                                            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                                                op: OP.into(),
                                                expected: "I64 positional key in Vector classifier-Bundle",
                                                got: Box::new(ValueSnapshot::unavailable("non-I64 Bind key in Vector classifier-Bundle"))
                                            }).into());
                                        }
                                    };
                                    let elem = from_holon_item(v, OP, args[0].span())?;
                                    pairs.push((idx, elem));
                                }
                                _ => {
                                    return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                                        op: OP.into(),
                                        expected: "Bind(I64, _) positional child in Vector classifier-Bundle",
                                        got: Box::new(ValueSnapshot::unavailable("non-Bind child in Vector classifier-Bundle inner items"))
                                    }).into());
                                }
                            }
                        }
                        pairs.sort_by_key(|(k, _)| *k);
                        let elems: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
                        Ok(TrackedValue::new(Value::Vec(Arc::new(elems)), prov()))
                    }
                    "List" => {
                        // List: inner Bundle contains sequential bare items → wat::core::List.
                        // Order-preserving (LinkedList; items were stored front-to-back).
                        let mut list = std::collections::LinkedList::new();
                        for item in items.iter() {
                            let v = from_holon_item(item, OP, args[0].span())?;
                            list.push_back(v);
                        }
                        Ok(TrackedValue::new(
                            Value::wat__core__List(Arc::new(list)),
                            prov(),
                        ))
                    }
                    "Tuple" => {
                        // Tuple: inner Bundle contains positional Bind(I64, _) pairs → Tuple.
                        // Same internal structure as Vector; outer classifier distinguishes.
                        let n = items.len();
                        let mut pairs: Vec<(i64, Value)> = Vec::with_capacity(n);
                        for child in items.iter() {
                            match child {
                                HolonAST::Bind(k, v) => {
                                    let idx = match k.as_ref() {
                                        HolonAST::I64(i) => *i,
                                        _ => {
                                            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                                                op: OP.into(),
                                                expected: "I64 positional key in Tuple classifier-Bundle",
                                                got: Box::new(ValueSnapshot::unavailable("non-I64 Bind key in Tuple classifier-Bundle"))
                                            }).into());
                                        }
                                    };
                                    let elem = from_holon_item(v, OP, args[0].span())?;
                                    pairs.push((idx, elem));
                                }
                                _ => {
                                    return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                                        op: OP.into(),
                                        expected: "Bind(I64, _) positional child in Tuple classifier-Bundle",
                                        got: Box::new(ValueSnapshot::unavailable("non-Bind child in Tuple classifier-Bundle inner items"))
                                    }).into());
                                }
                            }
                        }
                        pairs.sort_by_key(|(k, _)| *k);
                        let elems: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
                        Ok(TrackedValue::new(Value::Tuple(Arc::new(elems)), prov()))
                    }
                    _ => Err(RuntimeError::new(
                        args[0].span().clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "known classifier: Map, Set, Vector, List, or Tuple",
                            got: Box::new(ValueSnapshot::unavailable(
                                "unknown classifier name in collection Bind",
                            )),
                        },
                    )
                    .into()),
                }
            } else {
                // No classifier — bare Bundle or other structural form.
                // Per arc 228 HARD CUT: unclassified collections are not decodeable.
                // Bare Bundle must be upgraded to classifier-wrapped form first
                // (use to-holon on the collection Value, or a Pascal-Case constructor).
                Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "primitive leaf, Atom, or classifier-wrapped collection (Bind(Atom(name), Bundle(...))) as produced by to-holon or :wat::holon::Map/Set/Vector/List/Tuple constructors",
                    got: Box::new(ValueSnapshot::unavailable("unclassified HolonAST (bare Bundle, Bind without Atom-String classifier, Permute, Thermometer, Blend, or other composite)"))
                }).into())
            }
        }
    }
}


/// `(:wat::holon::Atom h)` -> a HolonAST `Atom` wrapping `h`. Marks `h` as
/// an opaque, non-decomposable unit under `statement-length` and the
/// classifier machinery — the escape hatch collection classifiers wrap
/// their classifier name in.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     h :wat::holon::HolonAST the HolonAST to wrap, alone
/// @ret     :wat::holon::HolonAST an `Atom` wrapping the given HolonAST
/// @example (:wat::holon::Atom (:wat::holon::leaf "role")) #=> (:wat::holon::Atom (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::Atom")]
pub(crate) fn eval_holon_atom_constructor(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (from `wrap_holon_as_atom`) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let v = eval_inner(h, env, sym)?.value_owned();
    wrap_holon_as_atom(v, h.span())
}


/// `(:wat::holon::to-holon v)` -> a HolonAST composition encoding wat value
/// `v`, dispatching on `v`'s runtime type (records, collections,
/// primitives). The general-purpose encoder; `leaf` is its primitive-only
/// fast path.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     v :T the value to encode, alone
/// @ret     :wat::holon::HolonAST the HolonAST composition encoding `v`
/// @example (:wat::holon::to-holon "role") #=> (:wat::holon::to-holon "role")
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::to-holon")]
pub(crate) fn eval_holon_to_holon(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (from `to_holon_inner`) locates at `v`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let val = eval_inner(v, env, sym)?.value_owned();
    to_holon_inner(val, v.span())
}


/// `(:wat::holon::leaf v)` -> a HolonAST primitive leaf for `v`
/// (`i64`/`f64`/`bool`/`String`/keyword/`nil`). Raises on anything else —
/// use `Atom`/`from-wat`/`to-holon` instead.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     v :T the primitive value to wrap, alone
/// @ret     :wat::holon::HolonAST the primitive leaf
/// @example (:wat::holon::leaf 5) #=> (:wat::holon::leaf 5)
#[wat_intrinsic(":wat::holon::leaf")]
pub(crate) fn eval_holon_leaf(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `v`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::leaf";
    let val = eval_inner(v, env, sym)?.value_owned();
    let h = match val {
        Value::i64(n) => HolonAST::i64(n),
        Value::f64(x) => HolonAST::f64(x),
        Value::bool(b) => HolonAST::bool_(b),
        Value::String(s) => HolonAST::string(s.as_str()),
        // Arc 230: Keyword → HolonAST::keyword() composition (Bind(Atom("Keyword"), Atom(s))).
        // keyword() strips the leading colon; same semantics as arc 221 Stone 221.4b.
        Value::wat__core__keyword(k) => HolonAST::keyword(k.as_str()),
        // Arc 230: Value::Unit (wat nil) → HolonAST::nil() composition.
        // HolonAST::nil() = Bind(Atom("Symbol"), Atom("nil")); supersedes HolonAST::Nil.
        Value::Unit => HolonAST::nil(),
        other => {
            return Err(RuntimeError::new(
                v.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "primitive (i64/f64/bool/String/keyword/nil); \
                           use :wat::holon::Atom to wrap a HolonAST, \
                           :wat::holon::from-wat to lower a quoted form, \
                           :wat::holon::to-holon for other types",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(h)))
}


/// `(:wat::holon::from-wat a)` -> a HolonAST composition lowering quoted
/// wat form `a` (typically produced by `:wat::core::quote`) structurally.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     a :wat::WatAST the quoted form to lower, alone
/// @ret     :wat::holon::HolonAST the HolonAST composition encoding the form's structure
/// @example (:wat::holon::from-wat (:wat::core::quote (:wat::i64::+ 1 2))) #=> (:wat::holon::from-wat (:wat::core::quote (:wat::i64::+ 1 2)))
/// @see     :wat::holon::to-wat
#[wat_intrinsic(":wat::holon::from-wat")]
pub(crate) fn eval_holon_from_wat(
    a: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `a`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::from-wat";
    let v = eval_inner(a, env, sym)?.value_owned();
    let h = match v {
        Value::wat__WatAST(a) => watast_to_holon(&a),
        other => {
            return Err(RuntimeError::new(
                a.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (typically from :wat::core::quote); \
                           use :wat::holon::Atom for HolonAST inputs, \
                           :wat::holon::to-holon for other types, \
                           :wat::holon::leaf for primitives",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(h)))
}


/// `(:wat::holon::to-wat h)` -> the quoted wat form `h` lowers back to —
/// the inverse of `from-wat`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     h :wat::holon::HolonAST the HolonAST to raise back to a form, alone
/// @ret     :wat::WatAST the reconstructed quoted form
/// @example (:wat::holon::to-wat (:wat::holon::from-wat (:wat::core::quote x))) #=> (:wat::holon::to-wat (:wat::holon::from-wat (:wat::core::quote x)))
/// @see     :wat::holon::from-wat
#[wat_intrinsic(":wat::holon::to-wat")]
pub(crate) fn eval_holon_to_wat(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let h = match eval_inner(h, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::to-wat".into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::wat__WatAST(Arc::new(holon_to_watast(&h))))
}


/// `(:wat::holon::literal form)` -> `(:wat::holon::to-holon (:wat::core::quote
/// form))`, fused: quotes `form` without evaluating it, then lowers the
/// quoted form to a HolonAST composition directly (shares `:wat::core::quote`'s
/// `eval_quote`, which is genuinely shared between the two verbs and stays
/// in `runtime.rs`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     args… :wat::core::Value the unevaluated form, alone
/// @ret     :wat::holon::HolonAST the HolonAST composition encoding the form's structure
/// @example (:wat::holon::literal (f x)) #=> (:wat::holon::literal (f x))
/// @see     :wat::holon::from-wat
#[wat_intrinsic(":wat::holon::literal")]
pub(crate) fn eval_holon_literal(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    to_holon_inner(eval_quote(args, span)?, span)
}


/// `(:wat::holon::to-record x surface)` -> a `wat::holon::HolonRecord`
/// Aggregate projecting `x`'s fields per `surface` (a `$field` probe
/// shape) into a named, hologram-backed record.
///
/// ⚠ The `@example` below needs `:probe::Pt`/`:probe::Planar` (a
/// `defstruct`/`defsurface` pair, verified against
/// `wat-scripts/scratch-pad/probe-home-8-examples.wat`) declared in the
/// SAME program before it will run — `to-record` has no zero-declaration
/// call shape, unlike every other verb in this file. Not registered in
/// `check.rs` (`doc_arg_ret_types_match_checker_scheme` skips it), so
/// there is no scheme this doc's types could drift from either.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     x :wat::core::Value the value projected
/// @arg     surface :wat::core::keyword the surface probe `x` is projected onto (a literal keyword, not evaluated)
/// @ret     :wat::core::Value a `wat::holon::HolonRecord`-classed Aggregate
/// @example (:wat::core::do (:wat::core::defstruct :probe::Pt [x <- :wat::core::i64]) (:wat::core::defsurface :probe::Planar :nature :wat::core::Struct :features [x <- :wat::core::i64]) (:wat::holon::to-record (:probe::Pt :x 3) :probe::Planar)) #=> (:wat::core::do (:wat::core::defstruct :probe::Pt [x <- :wat::core::i64]) (:wat::core::defsurface :probe::Planar :nature :wat::core::Struct :features [x <- :wat::core::i64]) (:wat::holon::to-record (:probe::Pt :x 3) :probe::Planar))
#[wat_intrinsic(":wat::holon::to-record")]
pub(crate) fn eval_to_holon_record(
    x: &WatAST,
    surface: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::to-record";
    // `parse_projection_args` still takes a slice — it is shared verbatim with
    // its other call site in `runtime.rs` (a projection verb outside this
    // file's scope), so its own signature is untouched. The two named
    // params are re-packed into the 2-element slice it expects; its own
    // hand-rolled arity check is therefore unreachable through this call
    // site (the `#[wat_intrinsic]`-generated check now enforces arity 2
    // before this body runs at all) but is left in place because the fn is
    // still reached directly, with a real slice, from that other call site.
    let call_args = [x.clone(), surface.clone()];
    let (x_val, surface_kw, surf) = parse_projection_args(OP, &call_args, list_span, env, sym)?;
    let (field_names, field_values) = project_surface_attrs(&x_val, &surf, sym, list_span)?;
    let class = format!("{}$holon-record", surface_kw.trim_start_matches(':'));
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let hologram = build_holon_hologram(&class, &field_names, &field_values, ctx, list_span)?;
    Ok(Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class,
        field_names,
        Arc::new(field_values),
        hologram,
    ))))
}


/// `(:wat::holon::Map items)` -> a classifier-wrapped `Bind(Atom("Map"),
/// Bundle(...))` HolonAST composition encoding wat `Map` `items` — the
/// counterpart `from-holon` recognizes and decodes back to a
/// `:wat::core::Map`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs, alone
/// @ret     :wat::holon::HolonAST the classifier-wrapped composition
/// @example (:wat::holon::Map (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::Bind (:wat::holon::leaf "k") (:wat::holon::leaf "v")))) #=> (:wat::holon::Map (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::Bind (:wat::holon::leaf "k") (:wat::holon::leaf "v"))))
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::Map")]
pub(crate) fn eval_algebra_map(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `items`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Map";
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children: Vec<HolonAST> = list
        .iter()
        .map(|v| require_holon(OP, &v.clone()).map(|h| (*h).clone()))
        .collect::<Result<Vec<HolonAST>, _>>()?;
    let inner_bundle = HolonAST::bundle(children);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("Map"))),
        inner_bundle,
    );
    Ok(Value::holon__HolonAST(Arc::new(classified)))
}


/// `(:wat::holon::Set items)` -> a classifier-wrapped `Bind(Atom("Set"),
/// Bundle(...))` HolonAST composition encoding wat `Set` `items` — the
/// counterpart `from-holon` recognizes and decodes back to a
/// `:wat::core::Set`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs, alone
/// @ret     :wat::holon::HolonAST the classifier-wrapped composition
/// @example (:wat::holon::Set (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role"))) #=> (:wat::holon::Set (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::Set")]
pub(crate) fn eval_algebra_set(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `items`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Set";
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children: Vec<HolonAST> = list
        .iter()
        .map(|v| require_holon(OP, &v.clone()).map(|h| (*h).clone()))
        .collect::<Result<Vec<HolonAST>, _>>()?;
    let inner_bundle = HolonAST::bundle(children);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("Set"))),
        inner_bundle,
    );
    Ok(Value::holon__HolonAST(Arc::new(classified)))
}


/// `(:wat::holon::Vector items)` -> a classifier-wrapped `Bind(Atom("Vector"),
/// Bundle(...))` HolonAST composition encoding wat `Vector` `items` — the
/// counterpart `from-holon` recognizes and decodes back to a
/// `:wat::core::Vector`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs, alone
/// @ret     :wat::holon::HolonAST the classifier-wrapped composition
/// @example (:wat::holon::Vector (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role"))) #=> (:wat::holon::Vector (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::Vector")]
pub(crate) fn eval_algebra_vector(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `items`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Vector";
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let positional: Vec<HolonAST> = list
        .iter()
        .enumerate()
        .map(|(i, v)| {
            require_holon(OP, &v.clone())
                .map(|h| HolonAST::bind(HolonAST::i64(i as i64), (*h).clone()))
        })
        .collect::<Result<Vec<HolonAST>, _>>()?;
    let inner_bundle = HolonAST::bundle(positional);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("Vector"))),
        inner_bundle,
    );
    Ok(Value::holon__HolonAST(Arc::new(classified)))
}


/// `(:wat::holon::List items)` -> a classifier-wrapped `Bind(Atom("List"),
/// Bundle(...))` HolonAST composition encoding wat `List` `items` — the
/// counterpart `from-holon` recognizes and decodes back to a
/// `:wat::core::List`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs, alone
/// @ret     :wat::holon::HolonAST the classifier-wrapped composition
/// @example (:wat::holon::List (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role"))) #=> (:wat::holon::List (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::List")]
pub(crate) fn eval_algebra_list(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `items`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::List";
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children: Vec<HolonAST> = list
        .iter()
        .map(|v| require_holon(OP, &v.clone()).map(|h| (*h).clone()))
        .collect::<Result<Vec<HolonAST>, _>>()?;
    let inner_bundle = HolonAST::bundle(children);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("List"))),
        inner_bundle,
    );
    Ok(Value::holon__HolonAST(Arc::new(classified)))
}


/// `(:wat::holon::Tuple items)` -> a classifier-wrapped `Bind(Atom("Tuple"),
/// Bundle(...))` HolonAST composition encoding wat `Tuple` `items` — the
/// counterpart `from-holon` recognizes and decodes back to a
/// `:wat::core::Tuple`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs, alone
/// @ret     :wat::holon::HolonAST the classifier-wrapped composition
/// @example (:wat::holon::Tuple (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role"))) #=> (:wat::holon::Tuple (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))
/// @see     :wat::holon::from-holon
#[wat_intrinsic(":wat::holon::Tuple")]
pub(crate) fn eval_algebra_tuple(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `items`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Tuple";
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let positional: Vec<HolonAST> = list
        .iter()
        .enumerate()
        .map(|(i, v)| {
            require_holon(OP, &v.clone())
                .map(|h| HolonAST::bind(HolonAST::i64(i as i64), (*h).clone()))
        })
        .collect::<Result<Vec<HolonAST>, _>>()?;
    let inner_bundle = HolonAST::bundle(positional);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("Tuple"))),
        inner_bundle,
    );
    Ok(Value::holon__HolonAST(Arc::new(classified)))
}


/// `(:wat::holon::Bind a b)` -> the HolonAST `Bind(a, b)` composition — the
/// role-filler binding primitive FOUNDATION 1718 is built on. Constructs
/// the tree; the self-inverse property (`Bind(Bind(x,y),x) ≈ y`) is a
/// VECTOR-level identity observable via `cosine`/`presence?`, not an
/// AST-level rewrite this constructor performs.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     a :wat::holon::HolonAST the two operands bound together, in order
/// @arg     b :wat::holon::HolonAST the two operands bound together, in order
/// @ret     :wat::holon::HolonAST the `Bind(a, b)` composition
/// @example (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler")) #=> (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))
/// @see     :wat::holon::Bind/left
#[wat_intrinsic(":wat::holon::Bind")]
pub(crate) fn eval_algebra_bind(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only errors (from `coerce_to_holon_ast`) locate at `a`'s/`b`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    // Arc 234 Stone 234.5 — D3: thread coerce_to_holon_ast for both args.
    // Accepts Value::holon__HolonAST (existing) OR Value::Aggregate(HolonRecord).
    // Records flow through natively; auto-dispatch extracts the hologram at the boundary.
    let a = coerce_to_holon_ast(
        ":wat::holon::Bind",
        eval_inner(a, env, sym)?.value_owned(),
        a.span(),
    )?;
    let b = coerce_to_holon_ast(
        ":wat::holon::Bind",
        eval_inner(b, env, sym)?.value_owned(),
        b.span(),
    )?;

    // No AST-level simplification. MAP's bind self-inverse — Bind(Bind(x,y),x) →
    // y — is a VECTOR-level identity (and per 058-024's rejection text, holds
    // only on non-zero positions of the key; zero positions drop to 0).
    // Lifting it to the AST as a rewrite rule would overclaim exact recovery
    // where the algebra acknowledges quantized noise. Bind always constructs
    // the Bind tree; the self-inverse is observable via vector-level presence
    // measurement. FOUNDATION 1718: the retrieval primitive is cosine.
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::bind(a, b))))
}


/// `(:wat::holon::Bundle items)` -> `(:Result :- [wat::holon::HolonAST
/// wat::holon::CapacityExceeded])`. The HolonAST `Bundle(items)`
/// superposition, `Ok` under the router-picked dimension's Kanerva
/// capacity budget (`floor(sqrt(d))`); over budget, `Err` under
/// `:error` capacity mode or a panic under `:panic` mode.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     items (:wat::core::Vector :- [:wat::holon::HolonAST]) the `:wat::core::Vector` of child HolonASTs bundled, alone
/// @ret     (:wat::core::Result :- [:wat::holon::HolonAST :wat::holon::CapacityExceeded]) `Ok` the Bundle composition, or `Err` a `CapacityExceeded`
/// @example (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) #=> (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler")))
#[wat_intrinsic(":wat::holon::Bundle")]
pub(crate) fn eval_algebra_bundle(
    items: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let list = match eval_inner(items, env, sym)?.value_owned() {
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::new(
                items.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Bundle".into(),
                    expected: "(List :- [wat::holon::HolonAST]) from (:wat::core::Vector ...)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // Arc 234 Stone 234.5 — D3: thread coerce_to_holon_ast for each child.
    // Accepts Value::holon__HolonAST (existing) OR Value::Aggregate(HolonRecord).
    // Records auto-extract their hologram at the coerce boundary.
    // Per-element WatAST span is gone by this point (we have Value, not
    // WatAST — arc 138 discipline); fall back to the Bundle call's own
    // span, which is real user source rather than a Rust line.
    let children: Vec<HolonAST> = list
        .iter()
        .map(|v| coerce_to_holon_ast(":wat::holon::Bundle list element", v.clone(), list_span))
        .collect::<Result<Vec<HolonAST>, _>>()?;

    // Arc 037 slice 1 layer 3: the dim the Bundle encodes at is
    // chosen by the ambient router given THIS Bundle's AST — not by
    // `ctx.config.dims`. The router's verdict drives the capacity
    // budget: `budget = floor(sqrt(d))` at the picked d. `None`
    // means no tier fits; treated identically to cost > budget
    // overflow.
    let cost = children.len();
    // Build the Bundle AST up front so the router (and any
    // downstream failure paths) can see the full shape. Under
    // Clone-on-use this is cheap; the AST is Arc-shared through
    // HolonAST's internal structure.
    let bundle_ast = HolonAST::bundle(children);
    let ctx = require_encoding_ctx(":wat::holon::Bundle", sym, list_span)?;
    let mode = ctx.config.capacity_mode;
    let d = ctx.dim_count;

    // Arc 077 / 294.c.2a: capacity check via the shared one-guard
    // `bundle_capacity_verdict`. Budget = ctx.capacity = floor(sqrt(d)).
    if let Some((cost_i, budget_i)) = bundle_capacity_verdict(cost, ctx) {
        match mode {
            crate::config::CapacityMode::Error => {
                let err = Value::Aggregate(Arc::new(AggregateValue::struct_(
                    "wat::holon::CapacityExceeded".into(),
                    capacity_exceeded_names(),
                    vec![Value::i64(cost_i), Value::i64(budget_i)],
                )));
                return Ok(Value::Result(Arc::new(Err(err))));
            }
            crate::config::CapacityMode::Panic => {
                panic!(
                    ":wat::holon::Bundle: capacity exceeded under :panic — cost {} > budget {} (d={})",
                    cost_i, budget_i, d
                );
            }
        }
    }

    let ok = Value::holon__HolonAST(Arc::new(bundle_ast));
    Ok(Value::Result(Arc::new(Ok(ok))))
}


/// `(:wat::holon::Permute h k)` -> the HolonAST `Permute(h, k)` composition
/// — `h` cyclically shifted by `k` positions at the vector level, used to
/// encode sequence position / depth.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     h :wat::holon::HolonAST the HolonAST permuted and the integer shift amount, in order
/// @arg     k :wat::core::i64 the HolonAST permuted and the integer shift amount, in order
/// @ret     :wat::holon::HolonAST the `Permute(h, k)` composition
/// @example (:wat::holon::Permute (:wat::holon::leaf "role") 1) #=> (:wat::holon::Permute (:wat::holon::leaf "role") 1)
#[wat_intrinsic(":wat::holon::Permute")]
pub(crate) fn eval_algebra_permute(
    h: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only errors (TypeMismatch, non-i32 step) locate at `k`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let child = require_holon(
        ":wat::holon::Permute",
        &eval_inner(h, env, sym)?.value_owned(),
    )?;
    let k = match eval_inner(k, env, sym)?.value_owned() {
        Value::i64(n) => i32::try_from(n).map_err(|_| {
            RuntimeError::new(
                k.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Permute".into(),
                    expected: "i32 step (integer fitting in i32)",
                    got: Box::new(ValueSnapshot::unavailable("i64 out of range")),
                },
            )
        })?,
        other => {
            return Err(RuntimeError::new(
                k.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Permute".into(),
                    expected: "i32 step",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::permute(
        (*child).clone(),
        k,
    ))))
}


/// `(:wat::holon::Thermometer v min max)` -> a HolonAST `Thermometer` leaf
/// encoding scalar `v`, clamped to `[min, max]`, via a thermometer code —
/// the raw form `therm-form` also builds, taking already-`f64` args.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     v :wat::core::f64 the value and its `[min, max]` range, in order
/// @arg     min :wat::core::f64 the value and its `[min, max]` range, in order
/// @arg     max :wat::core::f64 the value and its `[min, max]` range, in order
/// @ret     :wat::holon::HolonAST the Thermometer leaf
/// @example (:wat::holon::Thermometer 5.0 0.0 10.0) #=> (:wat::holon::Thermometer 5.0 0.0 10.0)
/// @see     :wat::holon::therm-form
#[wat_intrinsic(":wat::holon::Thermometer")]
pub(crate) fn algebra_thermometer(
    v: &Value,
    min: &Value,
    max: &Value,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let v = require_numeric(":wat::holon::Thermometer", v, span)?;
    let mn = require_numeric(":wat::holon::Thermometer", min, span)?;
    let mx = require_numeric(":wat::holon::Thermometer", max, span)?;
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::thermometer(
        v, mn, mx,
    ))))
}


/// `(:wat::holon::Blend a b w1 w2)` -> the HolonAST `Blend(a, b, w1, w2)`
/// composition — a weighted vector-level average of `a` and `b`, unlike
/// `Bundle`'s unweighted superposition.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     a :wat::holon::HolonAST the two HolonAST operands and their two weights, in order
/// @arg     b :wat::holon::HolonAST the two HolonAST operands and their two weights, in order
/// @arg     w1 :wat::core::f64 the two HolonAST operands and their two weights, in order
/// @arg     w2 :wat::core::f64 the two HolonAST operands and their two weights, in order
/// @ret     :wat::holon::HolonAST the `Blend(a, b, w1, w2)` composition
/// @example (:wat::holon::Blend (:wat::holon::leaf "role") (:wat::holon::leaf "filler") 0.7 0.3) #=> (:wat::holon::Blend (:wat::holon::leaf "role") (:wat::holon::leaf "filler") 0.7 0.3)
#[wat_intrinsic(":wat::holon::Blend")]
pub(crate) fn algebra_blend(
    a: &Value,
    b: &Value,
    w1: &Value,
    w2: &Value,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let a = require_holon(":wat::holon::Blend", a)?;
    let b = require_holon(":wat::holon::Blend", b)?;
    let w1 = require_numeric(":wat::holon::Blend", w1, span)?;
    let w2 = require_numeric(":wat::holon::Blend", w2, span)?;
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::blend(
        (*a).clone(),
        (*b).clone(),
        w1,
        w2,
    ))))
}


/// `(:wat::holon::extract-classifier x)` -> the classifier name of `x`, if
/// any. A `wat::core::Record` Aggregate's classifier (its `class`) is
/// always present, returned as a bare `String`; a HolonAST's classifier
/// (from a `Map`/`Set`/`Vector`/`List`/`Tuple`-style `Bind(Atom(name),
/// ...)` composition) may be absent, returned as an `Option`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     x :wat::holon::HolonAST the HolonAST or Record probed, alone
/// @ret     (:wat::core::Option :- [:wat::core::String]) the classifier name — a bare `String` (Record) or an `Option` (HolonAST)
/// @example (:wat::holon::extract-classifier (:wat::holon::Vector (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role")))) #=> (:wat::holon::extract-classifier (:wat::holon::Vector (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role"))))
#[wat_intrinsic(":wat::holon::extract-classifier")]
pub(crate) fn eval_extract_classifier(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `x`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::extract-classifier";
    let arg_val = eval_inner(x, env, sym)?.value_owned();
    // Arc 234 Stone 234.5 — D3: auto-dispatch on wat::core::Record.
    // Records always have a class_fqdn; return String directly (not Option).
    // This is honest: a record's classifier is NEVER absent (mandatory at construction).
    // HolonAST path returns Option<String> as before (classifier may be absent for
    // structural HolonASTs that aren't typed-entity Binds).
    match arg_val {
        // Arc 293.R2.1 — Aggregate carries class (colon-free); return as String.
        Value::Aggregate(a) => Ok(Value::String(Arc::new(a.class.to_string()))),
        Value::holon__HolonAST(h) => {
            let result = extract_classifier(&h).map(|s| Value::String(Arc::new(s)));
            Ok(Value::Option(Arc::new(result)))
        }
        other => Err(RuntimeError::new(
            x.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::holon::HolonAST or wat::core::Record",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// `(:wat::holon::Bind/left h)` -> `(:Option :- [wat::holon::HolonAST])`,
/// the left child of `h` if `h` is a `Bind` composition, else `None`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the HolonAST probed, alone
/// @ret     (:wat::core::Option :- [:wat::holon::HolonAST]) `h`'s left child, or `None`
/// @example (:wat::holon::Bind/left (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) #=> (:wat::holon::Bind/left (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler")))
/// @see     :wat::holon::Bind/right
#[wat_intrinsic(":wat::holon::Bind/left")]
pub(crate) fn eval_bind_left(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Bind/left";
    let arg_val = eval_inner(h, env, sym)?.value_owned();
    let holon_arc = match arg_val {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let result = bind_left(&holon_arc).map(|h| Value::holon__HolonAST(Arc::new(h)));
    Ok(Value::Option(Arc::new(result)))
}


/// `(:wat::holon::Bind/right h)` -> `(:Option :- [wat::holon::HolonAST])`,
/// the right child of `h` if `h` is a `Bind` composition, else `None`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the HolonAST probed, alone
/// @ret     (:wat::core::Option :- [:wat::holon::HolonAST]) `h`'s right child, or `None`
/// @example (:wat::holon::Bind/right (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) #=> (:wat::holon::Bind/right (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler")))
/// @see     :wat::holon::Bind/left
#[wat_intrinsic(":wat::holon::Bind/right")]
pub(crate) fn eval_bind_right(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Bind/right";
    let arg_val = eval_inner(h, env, sym)?.value_owned();
    let holon_arc = match arg_val {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let result = bind_right(&holon_arc).map(|h| Value::holon__HolonAST(Arc::new(h)));
    Ok(Value::Option(Arc::new(result)))
}


/// `(:wat::holon::Bundle/children h)` -> `(:Vector :- [wat::holon::HolonAST])`,
/// the children of `h`, which must be a `Bundle` composition. Raises on
/// any other shape.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the Bundle HolonAST probed, alone
/// @ret     (:wat::core::Vector :- [:wat::holon::HolonAST]) `h`'s children, in order
/// @example (:wat::core::match (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) ((:wat::core::Ok h) (:wat::holon::Bundle/children h)) (_ (:wat::holon::Bundle/children (:wat::holon::leaf "unreachable")))) #=> (:wat::core::match (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) ((:wat::core::Ok h) (:wat::holon::Bundle/children h)) (_ (:wat::holon::Bundle/children (:wat::holon::leaf "unreachable"))))
/// @see     :wat::holon::Bundle/first
#[wat_intrinsic(":wat::holon::Bundle/children")]
pub(crate) fn eval_bundle_children(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Bundle/children";
    let arg_val = eval_inner(h, env, sym)?.value_owned();
    let holon_arc = match arg_val {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST (Bundle)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children = require_bundle(OP, &holon_arc, h.span())?;
    let out: Vec<Value> = children
        .iter()
        .map(|child| Value::holon__HolonAST(Arc::new(child.clone())))
        .collect();
    Ok(Value::Vec(Arc::new(out)))
}


/// `(:wat::holon::Bundle/first h)` -> the first child of `h`, which must
/// be a non-empty `Bundle` composition. Raises on any other shape, or an
/// empty Bundle.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the Bundle HolonAST probed, alone
/// @ret     :wat::holon::HolonAST `h`'s first child
/// @example (:wat::core::match (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) ((:wat::core::Ok h) (:wat::holon::Bundle/first h)) (_ (:wat::holon::Bundle/first (:wat::holon::leaf "unreachable")))) #=> (:wat::core::match (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) ((:wat::core::Ok h) (:wat::holon::Bundle/first h)) (_ (:wat::holon::Bundle/first (:wat::holon::leaf "unreachable"))))
/// @see     :wat::holon::Bundle/children
#[wat_intrinsic(":wat::holon::Bundle/first")]
pub(crate) fn eval_bundle_first(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only errors (TypeMismatch, empty Bundle) locate at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Bundle/first";
    let arg_val = eval_inner(h, env, sym)?.value_owned();
    let holon_arc = match arg_val {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST (Bundle)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children = require_bundle(OP, &holon_arc, h.span())?;
    let first = children.first().ok_or_else(|| {
        RuntimeError::new(
            h.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Bundle with at least one child",
                got: Box::new(ValueSnapshot::unavailable("empty Bundle")),
            },
        )
    })?;
    Ok(Value::holon__HolonAST(Arc::new(first.clone())))
}


/// `(:wat::holon::is-Map? x)` -> whether `x` is a HolonAST classified
/// `"Map"` (i.e. `(:wat::holon::is? x "Map")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Map`-classified HolonAST
/// @example (:wat::holon::is-Map? (:wat::holon::Map (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::Bind (:wat::holon::leaf "k") (:wat::holon::leaf "v"))))) #=> (:wat::holon::is-Map? (:wat::holon::Map (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::Bind (:wat::holon::leaf "k") (:wat::holon::leaf "v")))))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Map?")]
pub(crate) fn holon_is_map_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Map"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Set? x)` -> whether `x` is a HolonAST classified
/// `"Set"` (i.e. `(:wat::holon::is? x "Set")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Set`-classified HolonAST
/// @example (:wat::holon::is-Set? (:wat::holon::leaf "role")) #=> (:wat::holon::is-Set? (:wat::holon::leaf "role"))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Set?")]
pub(crate) fn holon_is_set_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Set"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Vector? x)` -> whether `x` is a HolonAST classified
/// `"Vector"` (i.e. `(:wat::holon::is? x "Vector")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Vector`-classified HolonAST
/// @example (:wat::holon::is-Vector? (:wat::holon::leaf "role")) #=> (:wat::holon::is-Vector? (:wat::holon::leaf "role"))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Vector?")]
pub(crate) fn holon_is_vector_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Vector"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-List? x)` -> whether `x` is a HolonAST classified
/// `"List"` (i.e. `(:wat::holon::is? x "List")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `List`-classified HolonAST
/// @example (:wat::holon::is-List? (:wat::holon::leaf "role")) #=> (:wat::holon::is-List? (:wat::holon::leaf "role"))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-List?")]
pub(crate) fn holon_is_list_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("List"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Tuple? x)` -> whether `x` is a HolonAST classified
/// `"Tuple"` (i.e. `(:wat::holon::is? x "Tuple")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Tuple`-classified HolonAST
/// @example (:wat::holon::is-Tuple? (:wat::holon::leaf "role")) #=> (:wat::holon::is-Tuple? (:wat::holon::leaf "role"))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Tuple?")]
pub(crate) fn holon_is_tuple_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Tuple"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Symbol? x)` -> whether `x` is a HolonAST classified
/// `"Symbol"` (i.e. `(:wat::holon::is? x "Symbol")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Symbol`-classified HolonAST
/// @example (:wat::holon::is-Symbol? (:wat::holon::from-wat (:wat::core::quote x))) #=> (:wat::holon::is-Symbol? (:wat::holon::from-wat (:wat::core::quote x)))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Symbol?")]
pub(crate) fn holon_is_symbol_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Symbol"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Keyword? x)` -> whether `x` is a HolonAST classified
/// `"Keyword"` (i.e. `(:wat::holon::is? x "Keyword")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Keyword`-classified HolonAST
/// @example (:wat::holon::is-Keyword? (:wat::holon::from-wat (:wat::core::quote :k))) #=> (:wat::holon::is-Keyword? (:wat::holon::from-wat (:wat::core::quote :k)))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Keyword?")]
pub(crate) fn holon_is_keyword_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Keyword"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Tag? x)` -> whether `x` is a HolonAST classified
/// `"Tag"` (i.e. `(:wat::holon::is? x "Tag")`). `false` for any
/// non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a `Tag`-classified HolonAST
/// @example (:wat::holon::is-Tag? (:wat::holon::leaf "role")) #=> (:wat::holon::is-Tag? (:wat::holon::leaf "role"))
/// @see     :wat::holon::is?
#[wat_intrinsic(":wat::holon::is-Tag?")]
pub(crate) fn holon_is_tag_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => extract_classifier(h).as_deref() == Some("Tag"),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is-Nil? x)` -> whether `x` is a HolonAST `nil`
/// composition. `false` for any non-HolonAST value.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed, alone
/// @ret     :wat::core::bool true iff `x` is a nil-composition HolonAST
/// @example (:wat::holon::is-Nil? (:wat::holon::to-holon nil)) #=> (:wat::holon::is-Nil? (:wat::holon::to-holon nil))
#[wat_intrinsic(":wat::holon::is-Nil?")]
pub(crate) fn holon_is_nil_q(x: &Value) -> Result<Value, EvalBreak> {
    let matches = match x {
        Value::holon__HolonAST(h) => h.is_nil(),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::is? x class)` -> whether `x` is a HolonAST whose
/// classifier equals `class`. The general form the eight `is-*?` shortcuts
/// (`is-Map?` etc.) delegate the same check to.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     x :wat::holon::HolonAST the value probed and the classifier name, in order
/// @arg     class :wat::core::String the value probed and the classifier name, in order
/// @ret     :wat::core::bool true iff `x` is a HolonAST classified `class`
/// @example (:wat::holon::is? (:wat::holon::leaf "role") "Vector") #=> (:wat::holon::is? (:wat::holon::leaf "role") "Vector")
/// @see     :wat::holon::extract-classifier
#[wat_intrinsic(":wat::holon::is?")]
pub(crate) fn eval_holon_is_predicate(
    x: &WatAST,
    class: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch, non-String classifier) locates at `class`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::is?";
    let value_val = eval_inner(x, env, sym)?.value_owned();
    let class_val = eval_inner(class, env, sym)?.value_owned();
    let class_name = match class_val {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError::new(
                class.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String (classifier name)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let matches = match value_val {
        Value::holon__HolonAST(h) => extract_classifier(&h).as_deref() == Some(class_name.as_str()),
        _ => false,
    };
    Ok(Value::bool(matches))
}


/// `(:wat::holon::term::template h)` -> `h` with every Thermometer leaf's
/// scalar value erased — the shape two HolonASTs must share to be
/// `term::matches?`-comparable.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the HolonAST templated, alone
/// @ret     :wat::holon::HolonAST `h` with scalar leaves erased
/// @example (:wat::holon::term::template (:wat::holon::Thermometer 5.0 0.0 10.0)) #=> (:wat::holon::term::template (:wat::holon::Thermometer 5.0 0.0 10.0))
/// @see     :wat::holon::term::matches?
#[wat_intrinsic(":wat::holon::term::template")]
pub(crate) fn eval_term_template(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::term::template";
    let h = match eval_inner(h, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(h.template())))
}


/// `(:wat::holon::term::slots h)` -> `(:Vector :- [f64])`, `h`'s
/// Thermometer leaf values in pre-order — the per-slot magnitudes
/// `term::matches?` compares against another HolonAST's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the HolonAST probed, alone
/// @ret     (:wat::core::Vector :- [:wat::core::f64]) `h`'s Thermometer leaf values, in pre-order
/// @example (:wat::holon::term::slots (:wat::holon::Thermometer 5.0 0.0 10.0)) #=> (:wat::holon::term::slots (:wat::holon::Thermometer 5.0 0.0 10.0))
/// @see     :wat::holon::term::ranges
#[wat_intrinsic(":wat::holon::term::slots")]
pub(crate) fn eval_term_slots(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::term::slots";
    let h = match eval_inner(h, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let items: Vec<Value> = h.slots().into_iter().map(Value::f64).collect();
    Ok(Value::Vec(Arc::new(items)))
}


/// `(:wat::holon::term::ranges h)` -> `(:Vector :- [(:Tuple :- [f64
/// f64])])`, the `[min, max]` range of each of `h`'s Thermometer leaves in
/// pre-order, paired with `term::slots`'s values.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     h :wat::holon::HolonAST the HolonAST probed, alone
/// @ret     (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::f64 :wat::core::f64])]) `h`'s Thermometer leaf `[min, max]` ranges, in pre-order
/// @example (:wat::holon::term::ranges (:wat::holon::Thermometer 5.0 0.0 10.0)) #=> (:wat::holon::term::ranges (:wat::holon::Thermometer 5.0 0.0 10.0))
/// @see     :wat::holon::term::slots
#[wat_intrinsic(":wat::holon::term::ranges")]
pub(crate) fn eval_term_ranges(
    h: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `h`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::term::ranges";
    let h = match eval_inner(h, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                h.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let items: Vec<Value> = h
        .ranges()
        .into_iter()
        .map(|(lo, hi)| Value::Tuple(Arc::new(vec![Value::f64(lo), Value::f64(hi)])))
        .collect();
    Ok(Value::Vec(Arc::new(items)))
}


/// `(:wat::holon::term::matches? q s)` -> whether `q` and `s` share the
/// same template AND every Thermometer slot agrees within the program's
/// coincident floor (scaled by each slot's own range).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     q :wat::holon::HolonAST the query and subject HolonASTs, in order
/// @arg     s :wat::holon::HolonAST the query and subject HolonASTs, in order
/// @ret     :wat::core::bool true iff same template and every slot is within floor
/// @example (:wat::holon::term::matches? (:wat::holon::Thermometer 5.0 0.0 10.0) (:wat::holon::Thermometer 5.0 0.0 10.0)) #=> (:wat::holon::term::matches? (:wat::holon::Thermometer 5.0 0.0 10.0) (:wat::holon::Thermometer 5.0 0.0 10.0))
#[wat_intrinsic(":wat::holon::term::matches?")]
pub(crate) fn eval_term_matches_q(
    q: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::term::matches?";
    let q = match eval_inner(q, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                q.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let s = match eval_inner(s, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                s.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    if q.template() != s.template() {
        return Ok(Value::bool(false));
    }
    let q_slots = q.slots();
    let s_slots = s.slots();
    let q_ranges = q.ranges();
    // Same template guarantees same slot/range arity by construction —
    // the template decomposition reads the form's shape and slots/
    // ranges read the same Thermometer leaves in the same pre-order
    // sequence.
    let d = program_dim(OP, sym, list_span)?;
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let floor = ctx.encoders.get(d).coincident_floor(sym);
    for i in 0..q_slots.len() {
        let (lo, hi) = q_ranges[i];
        let range = hi - lo;
        // Zero-range slot (min == max) is degenerate; require exact
        // value equality rather than dividing by zero. Two thoughts
        // with a degenerate Thermometer still match if and only if
        // their values agree bit-for-bit.
        if range == 0.0 {
            if q_slots[i] != s_slots[i] {
                return Ok(Value::bool(false));
            }
        } else if (q_slots[i] - s_slots[i]).abs() / range >= floor {
            return Ok(Value::bool(false));
        }
    }
    Ok(Value::bool(true))
}


/// `(:wat::holon::therm-form low high value)` -> a HolonAST `Thermometer`
/// leaf for `value` clamped to `[low, high]` — the raw-`f64` counterpart
/// of `Thermometer` (which takes wat scalar args in `(value, min, max)`
/// order; this one takes already-evaluated `f64`s in `(low, high, value)`
/// order).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     low :wat::core::f64 the low bound, high bound, and value, in order
/// @arg     high :wat::core::f64 the low bound, high bound, and value, in order
/// @arg     value :wat::core::f64 the low bound, high bound, and value, in order
/// @ret     :wat::holon::HolonAST the Thermometer leaf
/// @example (:wat::holon::therm-form 0.0 10.0 5.0) #=> (:wat::holon::therm-form 0.0 10.0 5.0)
/// @see     :wat::holon::Thermometer
#[wat_intrinsic(":wat::holon::therm-form")]
pub(crate) fn eval_therm_form(
    low: &WatAST,
    high: &WatAST,
    value: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::therm-form";
    let low = match eval_inner(low, env, sym)?.value_owned() {
        Value::f64(x) => x,
        other => {
            return Err(RuntimeError::new(
                low.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let high = match eval_inner(high, env, sym)?.value_owned() {
        Value::f64(x) => x,
        other => {
            return Err(RuntimeError::new(
                high.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let value = match eval_inner(value, env, sym)?.value_owned() {
        Value::f64(x) => x,
        other => {
            return Err(RuntimeError::new(
                value.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "f64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    if !low.is_finite() || !high.is_finite() || low >= high {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("require finite low < high; got low={}, high={}", low, high),
            },
        )
        .into());
    }
    let clamped = if !value.is_finite() {
        low
    } else {
        value.clamp(low, high)
    };
    let ast = HolonAST::Thermometer {
        value: clamped,
        min: low,
        max: high,
    };
    Ok(Value::holon__HolonAST(Arc::new(ast)))
}


/// `(:wat::holon::presence-floor d)` -> `:f64`, the presence-detection
/// noise floor at dimension `d` — the threshold `presence?` compares a
/// cosine against.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     d :wat::core::i64 the vector dimension, alone
/// @ret     :wat::core::f64 the presence-detection noise floor at dimension `d`
/// @example (:wat::holon::presence-floor 4096) #=> (:wat::holon::presence-floor 4096)
/// @see     :wat::holon::presence?
#[wat_intrinsic(":wat::holon::presence-floor")]
pub(crate) fn eval_presence_floor(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::presence-floor";
    let dval = require_i64(OP, eval_inner(d, env, sym)?.value_owned())?;
    if dval <= 0 {
        return Err(RuntimeError::new(
            d.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("d must be positive; got {}", dval),
            },
        )
        .into());
    }
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    Ok(Value::f64(ctx.encoders.get(dval as usize).presence_floor(sym)))
}


/// `(:wat::holon::coincident-floor d)` -> `:f64`, the coincident-detection
/// noise floor at dimension `d` — the threshold `coincident?` compares a
/// cosine against.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     d :wat::core::i64 the vector dimension, alone
/// @ret     :wat::core::f64 the coincident-detection noise floor at dimension `d`
/// @example (:wat::holon::coincident-floor 4096) #=> (:wat::holon::coincident-floor 4096)
/// @see     :wat::holon::coincident?
#[wat_intrinsic(":wat::holon::coincident-floor")]
pub(crate) fn eval_coincident_floor(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::coincident-floor";
    let dval = require_i64(OP, eval_inner(d, env, sym)?.value_owned())?;
    if dval <= 0 {
        return Err(RuntimeError::new(
            d.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("d must be positive; got {}", dval),
            },
        )
        .into());
    }
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    Ok(Value::f64(
        ctx.encoders.get(dval as usize).coincident_floor(sym),
    ))
}


/// `(:wat::holon::cosine a b)` -> `:wat::holon::CosineOutcome`, the cosine
/// similarity between `a` and `b` (each a HolonAST, encoded at the
/// router-picked dimension, or a raw `Vector`) — degenerate (all-zero)
/// sides and dimension mismatches are matchable outcome variants, not
/// raises. FOUNDATION 1718: cosine is the retrieval primitive.
///
/// **Totality ground —** BRIEF-cosine-outcome-wall.md: `cosine` is a MEASUREMENT, so per the
/// where-corpus ruling it may NOT absorb its own undefined case into a value drawn from its
/// own range — a dimension mismatch and a zero-magnitude operand (the guarded `0.0`
/// `Similarity::cosine` used to return, which read as "orthogonal, unrelated" — a
/// fabrication, proven reachable by `probe-zero-magnitude-reachable.wat`) both become named
/// `:wat::holon::CosineOutcome` variants (`Similarity`/`Degenerate`/`DimensionMismatch`)
/// instead. An enum construction never raises and is always a well-typed value — total. From
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a), grouped with `dot`/`coincident?`/
/// `presence?`; the verdict is that list's, made by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @Category      Probe
/// @arg     a :wat::core::Value the two operands compared, in order
/// @arg     b :wat::core::Value the two operands compared, in order
/// @ret     :wat::holon::CosineOutcome the matchable cosine-similarity outcome
/// @example (:wat::holon::cosine (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::holon::cosine (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::cosine")]
pub(crate) fn eval_algebra_cosine(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let a = eval_inner(a, env, sym)?.value_owned();
    let b = eval_inner(b, env, sym)?.value_owned();
    cosine_outcome_from_values(a, b, list_span, sym)
}


/// `(:wat::holon::presence? target reference)` -> `:bool`, whether
/// `target`'s cosine to `reference` clears the presence floor — the
/// looser of the two similarity thresholds (`coincident?` is the tighter
/// one).
///
/// **Totality ground —** BRIEF-total-column-honest.md Direction 2: `eval_algebra_presence_q`
/// takes both args through `require_holon` (a raw `Vector` is rejected as a `TypeMismatch`,
/// the ordinary "type checker's concern" exclusion this axis already uses elsewhere), then
/// encodes BOTH at the same ambient `d` — so there is no code path by which its two vectors
/// can disagree in dimension; unlike `cosine`/`dot`/`coincident?` it never reaches
/// `pair_values_to_vectors`. Its only float op is `cosine > enc.presence_floor(sym)` — a
/// comparison, total for the same reason `f64::>` is (returns `bool`, never raises). From
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a), grouped with `cosine`/`dot`/
/// `coincident?`; the verdict is that list's, made by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @Category      Probe
/// @arg     target :wat::holon::HolonAST the target and reference operands, in order
/// @arg     reference :wat::holon::HolonAST the target and reference operands, in order
/// @ret     :wat::core::bool true iff `target` clears the presence floor against `reference`
/// @example (:wat::holon::presence? (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::holon::presence? (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
/// @see     :wat::holon::coincident?
#[wat_intrinsic(":wat::holon::presence?")]
pub(crate) fn eval_algebra_presence_q(
    target: &WatAST,
    reference: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let target = eval_inner(target, env, sym)?.value_owned();
    let reference = eval_inner(reference, env, sym)?.value_owned();
    presence_q_from_values(target, reference, list_span, sym)
}


/// `(:wat::holon::coincident? a b)` -> `:bool`, whether `a`'s cosine to
/// `b` clears the coincident floor — the tighter of the two similarity
/// thresholds (`presence?` is the looser one).
///
/// **Totality ground —** BRIEF-cosine-outcome-wall.md: `coincident?` routes through the
/// shared `pair_values_to_vectors` guard, which used to RAISE `TypeMismatch` on a
/// dimension-mismatched Vector pair — not total by this axis's definition. The cosine
/// outcome wall retired that raise: a dimension mismatch now answers `Value::bool(false)`
/// (a predicate absorbs its own undefined case — a documented total contract, not an IEEE
/// accident); every other path returns an ordinary `bool` as before. No raise remains on any
/// input. From `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a), grouped with
/// `cosine`/`dot`/`presence?`; the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @Category      Probe
/// @arg     a :wat::core::Value the two operands compared, in order
/// @arg     b :wat::core::Value the two operands compared, in order
/// @ret     :wat::core::bool true iff `a` clears the coincident floor against `b`
/// @example (:wat::holon::coincident? (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::holon::coincident? (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
/// @see     :wat::holon::presence?
#[wat_intrinsic(":wat::holon::coincident?")]
pub(crate) fn eval_algebra_coincident_q(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let a = eval_inner(a, env, sym)?.value_owned();
    let b = eval_inner(b, env, sym)?.value_owned();
    coincident_q_from_values(a, b, list_span, sym)
}


/// `(:wat::holon::coincident-explain a b)` -> a `wat::holon::CoincidentExplanation`
/// Aggregate: `a` and `b`'s cosine, the coincident floor, the compared
/// dimension, the ambient sigma, whether they're coincident, and the
/// minimum sigma that would make them pass. Raises on a dimension
/// mismatch (the one hole its fixed struct shape cannot honestly carry).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     a :wat::core::Value the two operands compared, in order
/// @arg     b :wat::core::Value the two operands compared, in order
/// @ret     :wat::core::Value a `wat::holon::CoincidentExplanation`-classed Aggregate
/// @example (:wat::holon::coincident-explain (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::holon::coincident-explain (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::coincident-explain")]
pub(crate) fn eval_algebra_coincident_explain(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::coincident-explain";
    let a = eval_inner(a, env, sym)?.value_owned();
    let b = eval_inner(b, env, sym)?.value_owned();
    let (va, vb) = match pair_values_to_vectors(OP, a, b, sym, list_span)? {
        // `coincident-explain`'s return shape is a fixed `CoincidentExplanation`
        // struct (STOP-5: do not touch it) with no field able to honestly
        // carry "these can't be compared" — unlike `cosine`/`dot`/`coincident?`,
        // it keeps the guard's former behavior for this one hole: raise,
        // exactly as `pair_values_to_vectors` itself used to raise before
        // this wall (arc 278 the cosine outcome wall).
        PairedVectors::DimensionMismatch { .. } => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "Vector pair with matching dimensions",
                    got: Box::new(ValueSnapshot::unavailable("mismatched-dim Vector pair")),
                    // arc 138: no per-arg AST span (takes a Value pair) — list_span (the call site) used instead
                },
            )
            .into());
        }
        PairedVectors::Paired(va, vb) => (va, vb),
    };
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let dim = va.dimensions();
    let enc = ctx.encoders.get(dim);
    let cosine = Similarity::cosine(&va, &vb);
    let floor = enc.coincident_floor(sym);
    let sigma = sym
        .coincident_sigma_fn()
        .map(|f| f.sigma_at(dim, sym))
        .unwrap_or(1);
    let coincident = (1.0 - cosine) < floor;
    let sqrt_d = (dim as f64).sqrt();
    let min_sigma_raw = ((1.0 - cosine) * sqrt_d).ceil() as i64;
    let min_sigma_to_pass = min_sigma_raw.max(1);
    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::holon::CoincidentExplanation".into(),
        coincident_explanation_names(),
        vec![
            Value::f64(cosine),
            Value::f64(floor),
            Value::i64(dim as i64),
            Value::i64(sigma),
            Value::bool(coincident),
            Value::i64(min_sigma_to_pass),
        ],
    ))))
}


/// `(:wat::holon::eval-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: runs two already-quoted forms, then checks whether the two
/// resulting values are `coincident?`. Any evaluation failure surfaces
/// through the wrapped `EvalResult`, matching the load pipeline's
/// `eval-ast!`/`eval-edn!` discipline rather than raising directly.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-coincident?")]
pub(crate) fn eval_form_ast_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    // Structural pre-check — matches eval-ast! pattern. Arity errors
    // fire before the EvalError wrap; they're caller-syntactic issues.
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::holon::eval-coincident?".into(),
                reason: format!(
                "(:wat::holon::eval-coincident? <ast-a> <ast-b>) takes exactly 2 arguments; got {}",
                args.len()
            ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let op = ":wat::holon::eval-coincident?";
        let value_a = run_ast_arg_for_eval_coincident(&args[0], env, sym, op)?;
        let value_b = run_ast_arg_for_eval_coincident(&args[1], env, sym, op)?;
        coincident_of_two_values(value_a, value_b, sym, op, list_span)
    })())
}


/// `(:wat::holon::eval-edn-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: runs two EDN source strings, then checks whether the two
/// resulting values are `coincident?`. Any evaluation failure surfaces
/// through the wrapped `EvalResult`, matching the load pipeline's
/// `eval-ast!`/`eval-edn!` discipline rather than raising directly.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-edn-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-edn-coincident?")]
pub(crate) fn eval_form_edn_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::holon::eval-edn-coincident?".into(),
            reason: format!(
                "(:wat::holon::eval-edn-coincident? <source-a> <source-b>) takes exactly 2 arguments; got {}",
                args.len()
            )
        }).into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        let op = ":wat::holon::eval-edn-coincident?";
        let src_a = expect_string_value(op, &args[0], env, sym)?;
        let src_b = expect_string_value(op, &args[1], env, sym)?;
        let value_a = parse_and_run(&src_a, env, sym)?;
        let value_b = parse_and_run(&src_b, env, sym)?;
        coincident_of_two_values(value_a, value_b, sym, op, list_span)
    })())
}


/// `(:wat::holon::eval-digest-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: evaluates two forms, comparing their content digests, then checks `coincident?`.
/// Shares its shared-implementation sibling's `_shared` helper with a
/// `String`-vs-forms flag (the `-string` suffix selects the source-string
/// entry point).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-digest-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-digest-coincident?")]
pub(crate) fn eval_form_digest_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_digest_coincident_shared(args, list_span, env, sym, false)
}


/// `(:wat::holon::eval-digest-string-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: evaluates two source strings, comparing their content digests, then checks `coincident?`.
/// Shares its shared-implementation sibling's `_shared` helper with a
/// `String`-vs-forms flag (the `-string` suffix selects the source-string
/// entry point).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-digest-string-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-digest-string-coincident?")]
pub(crate) fn eval_form_digest_string_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_digest_coincident_shared(args, list_span, env, sym, true)
}


/// `(:wat::holon::eval-signed-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: evaluates two forms, comparing their signed digests, then checks `coincident?`.
/// Shares its shared-implementation sibling's `_shared` helper with a
/// `String`-vs-forms flag (the `-string` suffix selects the source-string
/// entry point).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-signed-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-signed-coincident?")]
pub(crate) fn eval_form_signed_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_signed_coincident_shared(args, list_span, env, sym, false)
}


/// `(:wat::holon::eval-signed-string-coincident? <a> <b>)` -> a `wat::eval::EvalResult`-shaped
/// value wrapping `:bool`: evaluates two source strings, comparing their signed digests, then checks `coincident?`.
/// Shares its shared-implementation sibling's `_shared` helper with a
/// `String`-vs-forms flag (the `-string` suffix selects the source-string
/// entry point).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     args… :wat::core::Value the two sources compared, in order
/// @ret     (:wat::core::Result :- [:wat::core::bool :wat::core::EvalError]) an `EvalResult`-wrapped `:bool`
/// @example-norun (eval-signed-string-coincident? a b) #=> true
#[wat_intrinsic(":wat::holon::eval-signed-string-coincident?")]
pub(crate) fn eval_form_signed_string_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    eval_form_signed_coincident_shared(args, list_span, env, sym, true)
}


/// `(:wat::holon::dot a b)` -> `:wat::holon::DotOutcome`, the raw dot
/// product between `a` and `b` (each a HolonAST or a raw `Vector`) —
/// unlike `cosine`, not normalized by magnitude. Dimension mismatches are
/// a matchable outcome variant.
///
/// **Totality ground —** BRIEF-cosine-outcome-wall.md: its arithmetic cannot overflow
/// (`Vector.data: Vec<i8>`, bounded by `d × 127²`, unreachable at real dimensions — `d ≈
/// 10³⁰⁴` would be needed to reach ±Inf) and it needs no `Degenerate` case (a zero-magnitude
/// operand dots to an honest `0.0` — no division happens). The one thing that made it
/// partial was the same shared-guard raise `coincident?` had; with that retired, `dot`
/// returns `:wat::holon::DotOutcome` (`Computed`/`DimensionMismatch`) on every input. From
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a), grouped with `cosine`/
/// `coincident?`/`presence?`; the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
/// @Category      Probe
/// @arg     a :wat::core::Value the two operands compared, in order
/// @arg     b :wat::core::Value the two operands compared, in order
/// @ret     :wat::holon::DotOutcome the matchable dot-product outcome
/// @example (:wat::holon::dot (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::holon::dot (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::dot")]
pub(crate) fn eval_algebra_dot(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    // Arc 052 — polymorphic input: HolonAST or Vector in either
    // position. Same dim-resolution rule as cosine.
    let a = eval_inner(a, env, sym)?.value_owned();
    let b = eval_inner(b, env, sym)?.value_owned();
    dot_outcome_from_values(a, b, list_span, sym)
}


/// `(:wat::holon::simhash target)` -> `:i64`, a 64-bit locality-sensitive
/// hash of `target` (a HolonAST, encoded at the program's dimension, or a
/// raw `Vector`): each bit is the sign of `target`'s dot product against
/// one of 64 canonical basis atoms.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     target :wat::core::Value the value hashed, alone
/// @ret     :wat::core::i64 a 64-bit locality-sensitive hash
/// @example (:wat::holon::simhash (:wat::holon::leaf "role")) #=> (:wat::holon::simhash (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::simhash")]
pub(crate) fn eval_algebra_simhash(
    target: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let val = eval_inner(target, env, sym)?.value_owned();
    let ctx = require_encoding_ctx(":wat::holon::simhash", sym, list_span)?;
    // Arc 052 — polymorphic input: HolonAST encodes at router-picked d;
    // Vector uses its native dim directly.
    let (v, enc) = match val {
        Value::Vector(vec) => {
            let d = vec.dimensions();
            (vec.as_ref().clone(), ctx.encoders.get(d))
        }
        Value::holon__HolonAST(ast) => {
            let enc = ctx.encoders.get(ctx.dim_count);
            let v = encode(&ast, &enc.vm, &enc.scalar);
            (v, enc)
        }
        other => {
            return Err(RuntimeError::new(
                target.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::simhash".into(),
                    expected: "wat::holon::HolonAST or wat::holon::Vector",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };

    // Project onto I64(0)..I64(63) via the canonical LSH basis.
    let mut key: u64 = 0;
    for i in 0..64u32 {
        let atom_ast = HolonAST::i64(i as i64);
        let atom_vec = encode(&atom_ast, &enc.vm, &enc.scalar);
        let dot = Similarity::dot(&v, &atom_vec);
        if dot > 0.0 {
            key |= 1u64 << i;
        }
        // else: bit i stays 0 (sign-of-zero rule: dot == 0 → bit off)
    }
    Ok(Value::i64(key as i64))
}


/// `(:wat::holon::encode target)` -> `:wat::holon::Vector`, `target` (a
/// HolonAST) encoded to a raw ternary vector at the program's ambient
/// dimension — the bridge from AST-level algebra to the raw `Vector`
/// mirror ops (`vector-bind` etc.).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     target :wat::holon::HolonAST the HolonAST encoded, alone
/// @ret     :wat::holon::Vector the encoded raw ternary vector
/// @example (:wat::holon::encode (:wat::holon::leaf "role")) #=> (:wat::holon::encode (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::encode")]
pub(crate) fn eval_holon_encode(
    target: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let target = require_holon(
        ":wat::holon::encode",
        &eval_inner(target, env, sym)?.value_owned(),
    )?;
    let ctx = require_encoding_ctx(":wat::holon::encode", sym, list_span)?;
    let enc = ctx.encoders.get(ctx.dim_count);
    let v = encode(&target, &enc.vm, &enc.scalar);
    Ok(Value::Vector(Arc::new(v)))
}


/// `(:wat::holon::vector-bytes v)` -> `(:Vector :- [u8])`, `v`'s ternary
/// cells packed 4-per-byte behind a 4-byte little-endian dimension header —
/// the wire format `bytes-vector` decodes back.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     v :wat::holon::Vector the vector encoded, alone
/// @ret     :wat::core::Bytes the packed byte representation
/// @example (:wat::holon::vector-bytes (:wat::holon::encode (:wat::holon::leaf "role"))) #=> (:wat::holon::vector-bytes (:wat::holon::encode (:wat::holon::leaf "role")))
/// @see     :wat::holon::bytes-vector
#[wat_intrinsic(":wat::holon::vector-bytes")]
pub(crate) fn holon_vector_bytes(v: &Value, span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::vector-bytes";
    let v = require_vector(OP, v)?;
    // Codec logic (dim -> u32 header + 4-cells-per-byte packing) lives in
    // src/holon/codec.rs::encode_vector — this delegate only converts the
    // wat Value in, and adapts the domain Vec<u8> / VectorEncodeError back.
    let bytes = encode_vector(&v).map_err(|e| match e {
        VectorEncodeError::DimTooLarge => RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vector with dim representable as u32",
                got: Box::new(ValueSnapshot::unavailable("oversized Vector dim")),
                // arc 138: no per-value AST span — dim comes from Vector value, not AST; the call span is used instead
            },
        ),
        VectorEncodeError::InvalidCell { value, .. } => RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vector cell in {-1, 0, +1}",
                got: Box::new(ValueSnapshot::described(
                    "wat::core::i64",
                    format!("cell value out of ternary range ({})", value),
                )),
                // arc 138: no per-value AST span — cell value from Vector data, not AST; the call span is used instead
            },
        ),
    })?;
    Ok(Value::Vec(Arc::new(
        bytes.into_iter().map(Value::u8).collect(),
    )))
}


/// `(:wat::holon::bytes-vector bs)` -> `:wat::holon::VectorDecodeOutcome`,
/// decoding `bs` (as produced by `vector-bytes`) back to a raw `Vector` —
/// a truncated header, a length mismatch, a foreign dimension, and an
/// invalid ternary cell are all matchable outcome variants, never raises.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     bs :wat::core::Bytes the packed byte vector decoded, alone
/// @ret     :wat::holon::VectorDecodeOutcome the matchable decode outcome
/// @example (:wat::holon::bytes-vector (:wat::holon::vector-bytes (:wat::holon::encode (:wat::holon::leaf "role")))) #=> (:wat::holon::bytes-vector (:wat::holon::vector-bytes (:wat::holon::encode (:wat::holon::leaf "role"))))
/// @see     :wat::holon::vector-bytes
#[wat_intrinsic(":wat::holon::bytes-vector")]
pub(crate) fn eval_holon_bytes_vector(
    bs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::bytes-vector";
    // Pull the byte vector contents out as Vec<u8>.
    let xs = match eval_inner(bs, env, sym)?.value_owned() {
        Value::Vec(xs) => xs,
        other => {
            return Err(RuntimeError::new(
                bs.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Vector :- [u8])",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let mut bytes: Vec<u8> = Vec::with_capacity(xs.len());
    for v in xs.iter() {
        match v {
            Value::u8(b) => bytes.push(*b),
            other => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "(Vector :- [u8])",
                        got: Box::new(ValueSnapshot::of(other)),
                        // arc 138: no per-value AST span — element from Vec value, not AST; list_span (call site) used instead
                    },
                )
                .into());
            }
        }
    }
    // Header + length: structural checks that don't need the program's
    // dim-count — src/holon/codec.rs::parse_vector_header stays sym-free.
    match parse_vector_header(&bytes) {
        VectorHeader::TruncatedHeader { got } => {
            Ok(vector_decode_outcome_truncated_header(got as i64))
        }
        VectorHeader::LengthMismatch { expected, got } => Ok(
            vector_decode_outcome_length_mismatch(expected as i64, got as i64),
        ),
        VectorHeader::Ok { dim } => {
            // Cross-dim validation: this program's dim-count is a static,
            // once-only constant (`config::collect_entry_file`); a vector
            // whose wire header names a different d is a foreign-dimension
            // value, not a structural parse failure — matchable, not fatal
            // (BRIEF-dimension-heresy-screams.md). Fetched here — after the
            // header/length checks, same as before the codec split — so a
            // symbol table with no EncodingCtx attached still resolves a
            // truncated/malformed header without ever needing one.
            let expected_dim = program_dim(OP, sym, list_span)?;
            if dim != expected_dim {
                return Ok(vector_decode_outcome_dimension_mismatch(
                    expected_dim as i64,
                    dim as i64,
                ));
            }
            // Codec logic (4-cells-per-byte unpacking) lives in
            // src/holon/codec.rs::decode_vector_cells.
            Ok(match decode_vector_cells(&bytes, dim) {
                VectorCells::Decoded(v) => vector_decode_outcome_decoded(v),
                VectorCells::InvalidCell { at } => vector_decode_outcome_invalid_cell(at as i64),
            })
        }
    }
}


/// `(:wat::holon::vector-bind a b)` -> `:wat::holon::CombineOutcome`, the
/// raw-`Vector` mirror of `Bind`: binds `a` and `b` at the ternary-vector
/// level directly. Dimension mismatch is a matchable outcome variant.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     a :wat::holon::Vector the two raw vectors bound together, in order
/// @arg     b :wat::holon::Vector the two raw vectors bound together, in order
/// @ret     :wat::holon::CombineOutcome the matchable combine outcome
/// @example (:wat::holon::vector-bind (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler"))) #=> (:wat::holon::vector-bind (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler")))
#[wat_intrinsic(":wat::holon::vector-bind")]
pub(crate) fn holon_vector_bind(a: &Value, b: &Value) -> Result<Value, EvalBreak> {
    let va = require_vector(":wat::holon::vector-bind", a)?;
    let vb = require_vector(":wat::holon::vector-bind", b)?;
    if va.dimensions() != vb.dimensions() {
        return Ok(combine_outcome_dimension_mismatch(
            va.dimensions() as i64,
            vb.dimensions() as i64,
        ));
    }
    let result = holon::primitives::Primitives::bind(&va, &vb);
    Ok(combine_outcome_combined(result))
}


/// `(:wat::holon::vector-bundle vs)` -> `:wat::holon::CombineOutcome`, the
/// raw-`Vector` mirror of `Bundle`: superposes every vector in non-empty
/// `vs` directly. Dimension mismatch across `vs` is a matchable outcome
/// variant; an empty `vs` raises (no capacity budget to check against).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     vs (:wat::core::Vector :- [:wat::holon::Vector]) the `:wat::core::Vector` of raw vectors bundled, alone
/// @ret     :wat::holon::CombineOutcome the matchable combine outcome
/// @example (:wat::holon::vector-bundle (:wat::core::Vector :- [:wat::holon::Vector] (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler")))) #=> (:wat::holon::vector-bundle (:wat::core::Vector :- [:wat::holon::Vector] (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler"))))
#[wat_intrinsic(":wat::holon::vector-bundle")]
pub(crate) fn eval_holon_vector_bundle(
    vs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only errors (TypeMismatch) locate at `vs`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let vec_value = eval_inner(vs, env, sym)?.value_owned();
    let elements = match vec_value {
        Value::Vec(v) => v,
        other => {
            return Err(RuntimeError::new(
                vs.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::vector-bundle".into(),
                    expected: "Vec of wat::holon::Vector",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    if elements.is_empty() {
        return Err(RuntimeError::new(
            vs.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::holon::vector-bundle".into(),
                expected: "non-empty Vec of Vector",
                got: Box::new(ValueSnapshot::unavailable("empty Vec")),
            },
        )
        .into());
    }
    let mut owned: Vec<Arc<holon::Vector>> = Vec::with_capacity(elements.len());
    for elem in elements.iter() {
        owned.push(require_vector(":wat::holon::vector-bundle", elem)?);
    }
    // Verify dim match.
    let d = owned[0].dimensions();
    for v in &owned[1..] {
        if v.dimensions() != d {
            return Ok(combine_outcome_dimension_mismatch(
                d as i64,
                v.dimensions() as i64,
            ));
        }
    }
    let refs: Vec<&holon::Vector> = owned.iter().map(|v| v.as_ref()).collect();
    let result = holon::primitives::Primitives::bundle(&refs);
    Ok(combine_outcome_combined(result))
}


/// `(:wat::holon::vector-blend a b w1 w2)` -> `:wat::holon::CombineOutcome`,
/// the raw-`Vector` mirror of `Blend`: a weighted average of `a` and `b`
/// directly. Dimension mismatch is a matchable outcome variant.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     a :wat::holon::Vector the two raw vectors and their two weights, in order
/// @arg     b :wat::holon::Vector the two raw vectors and their two weights, in order
/// @arg     w1 :wat::core::f64 the two raw vectors and their two weights, in order
/// @arg     w2 :wat::core::f64 the two raw vectors and their two weights, in order
/// @ret     :wat::holon::CombineOutcome the matchable combine outcome
/// @example (:wat::holon::vector-blend (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler")) 0.7 0.3) #=> (:wat::holon::vector-blend (:wat::holon::encode (:wat::holon::leaf "role")) (:wat::holon::encode (:wat::holon::leaf "filler")) 0.7 0.3)
#[wat_intrinsic(":wat::holon::vector-blend")]
pub(crate) fn holon_vector_blend(
    a: &Value,
    b: &Value,
    w1: &Value,
    w2: &Value,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let va = require_vector(":wat::holon::vector-blend", a)?;
    let vb = require_vector(":wat::holon::vector-blend", b)?;
    let w1 = require_numeric(":wat::holon::vector-blend", w1, span)?;
    let w2 = require_numeric(":wat::holon::vector-blend", w2, span)?;
    if va.dimensions() != vb.dimensions() {
        return Ok(combine_outcome_dimension_mismatch(
            va.dimensions() as i64,
            vb.dimensions() as i64,
        ));
    }
    let result = holon::primitives::Primitives::blend_weighted(&va, &vb, w1, w2);
    Ok(combine_outcome_combined(result))
}


/// `(:wat::holon::vector-permute v k)` -> `:wat::holon::Vector`, the
/// raw-`Vector` mirror of `Permute`: `v` cyclically shifted by `k`
/// positions directly. Total — no dimension to mismatch against.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Combine
/// @arg     v :wat::holon::Vector the raw vector permuted and the integer shift amount, in order
/// @arg     k :wat::core::i64 the raw vector permuted and the integer shift amount, in order
/// @ret     :wat::holon::Vector the shifted vector
/// @example (:wat::holon::vector-permute (:wat::holon::encode (:wat::holon::leaf "role")) 1) #=> (:wat::holon::vector-permute (:wat::holon::encode (:wat::holon::leaf "role")) 1)
#[wat_intrinsic(":wat::holon::vector-permute")]
pub(crate) fn eval_holon_vector_permute(
    v: &WatAST,
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch, non-i32 step) locates at `k`'s own span, more precise than the coarse list span
) -> Result<Value, EvalBreak> {
    let v = require_vector(
        ":wat::holon::vector-permute",
        &eval_inner(v, env, sym)?.value_owned(),
    )?;
    let k_val = eval_inner(k, env, sym)?.value_owned();
    let k = match k_val {
        Value::i64(n) => n as i32,
        other => {
            return Err(RuntimeError::new(
                k.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::vector-permute".into(),
                    expected: "i64 shift amount",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    let result = holon::primitives::Primitives::permute(&v, k);
    Ok(Value::Vector(Arc::new(result)))
}


/// `(:wat::holon::statement-length ast)` -> `:i64`, the structural size of
/// `ast`'s top-level form for statement-length accounting: 1 for any
/// primitive leaf (including Symbol/Keyword/Tag/Nil compositions), 2 for
/// `Bind`/`Blend`, and the child count for `Bundle`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     ast :wat::holon::HolonAST the HolonAST measured, alone
/// @ret     :wat::core::i64 the top-level form's structural size
/// @example (:wat::holon::statement-length (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler"))) #=> (:wat::holon::statement-length (:wat::holon::Bind (:wat::holon::leaf "role") (:wat::holon::leaf "filler")))
#[wat_intrinsic(":wat::holon::statement-length")]
pub(crate) fn holon_statement_length(ast: &Value) -> Result<Value, EvalBreak> {
    let ast = require_holon(":wat::holon::statement-length", ast)?;
    // Arc 230 — Symbol/Keyword/Tag/Nil are now Bind compositions; intercept before
    // the generic match so they return 1 (conceptual leaf) not 2 (Bind structural count).
    let n = if ast.as_symbol().is_some() || ast.as_keyword().is_some() || ast.as_tag().is_some() {
        1
    } else {
        match &*ast {
            HolonAST::String(_)
            | HolonAST::I64(_)
            | HolonAST::F64(_)
            | HolonAST::Bool(_)
            // Arc 221 Stone 221.2 — Char is a primitive leaf; statement-length = 1.
            | HolonAST::Char(_)
            | HolonAST::Atom(_)
            | HolonAST::Permute(_, _)
            | HolonAST::Thermometer { .. }
            | HolonAST::SlotMarker { .. } => 1,
            HolonAST::Bind(_, _) | HolonAST::Blend(_, _, _, _) => 2,
            HolonAST::Bundle(children) => children.len(),
        }
    };
    Ok(Value::i64(n as i64))
}


