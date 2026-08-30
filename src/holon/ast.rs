//! HolonAST <-> `Value`/`WatAST` conversion algebra, plus the shared
//! `Bundle` capacity guard. Pure functions lifted out of `runtime.rs`
//! per Stone HOME-8 — see `src/holon/mod.rs` for the two-layer doctrine
//! this module is half of.

use crate::runtime::{EvalBreak, HolonForm, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::EncodingCtx;
use holon::HolonAST;
use std::collections::HashSet;
use std::sync::Arc;

/// Arc 216 Stone 1 + 2 — Reverse one HolonAST item back to a `Value`.
///
/// Used by `from-holon` Bundle extraction path. Handles the six
/// primitive leaf variants and recursively handles nested `Bundle`
/// (dispatching on shape: bare-atom set-shape → HashSet; positional-Bind
/// vector-shape → Vec). Returns `Err` for other composite shapes
/// (`Permute`/`Thermometer`/`Blend`/`SlotMarker`) that have no
/// unambiguous Value reconstruction without consumer-declared T.
///
/// Arc 225 Stone 225.1 — renamed from `holon_item_to_value`. `op: &str`
/// parameter threaded through to close arc 224 L1-runtime-3 latent lie
/// (hardcoded op name in error arm).
///
/// Arc 228 Stone 228.1 — updated decode dispatch. Previously dispatched by Bundle
/// child-shape heuristic (three-way: bare-atom → HashSet, positional-Bind → Vec,
/// arbitrary-Bind → HashMap). Now dispatches by classifier-atom first: if the outermost
/// form is `Bind(Atom(String(name)), inner)`, dispatches by name
/// ("Map" → HashMap, "Set" → HashSet, "Vector" → Vec, "List" → List, "Tuple" → Tuple).
/// Bare Bundle (no classifier) errors with helpful diagnostic per HARD CUT discipline —
/// the substrate refuses to decode unclassified collections. Callers must use
/// `to_holon_inner` (or a Pascal-Case constructor) which always produces classifier-wrapped
/// forms.
///
/// Polymorphic decode — the full HolonAST-to-Value materializer:
///
/// - Primitive leaf (Symbol/Keyword/Nil/Char/String/I64/F64/Bool) → corresponding
///   runtime `Value`.
/// - `Atom(inner)` → inner HolonAST as a `Value::holon__HolonAST`.
/// - `Bind(Atom(String(name)), Bundle(items))` → classifier-dispatch by name.
/// - `Bundle(items)` → TypeMismatch (unclassified Bundle; HARD CUT per arc 228 doctrine).
// Stone 216.5b — suppress `mutable_key_type` for `HashSet<Value>`.
// See comment on `hashset_conj_inner` for rationale.
#[allow(clippy::mutable_key_type)]
pub(crate) fn from_holon_item(
    item: &HolonAST,
    op: &str,
    op_span: &Span,
) -> Result<Value, EvalBreak> {
    // Arc 230: Symbol/Keyword/Nil/Tag variants retired. Recognise via accessors.
    // Symbol composition → keyword Value (Symbol carried colon-prefixed keywords).
    if let Some(s) = item.as_symbol() {
        // nil composition (symbol("nil")) → Value::Unit.
        if s == "nil" {
            return Ok(Value::Unit);
        }
        return Ok(Value::wat__core__keyword(Arc::new(s.to_string())));
    }
    // Keyword composition → keyword Value with leading colon restored.
    if let Some(s) = item.as_keyword() {
        return Ok(Value::wat__core__keyword(Arc::new(format!(":{}", s))));
    }
    match item {
        // Arc 221 Stone 221.2 — HolonAST::Char leaf → Value::wat__core__Char.
        HolonAST::Char(c) => Ok(Value::wat__core__Char(*c)),
        HolonAST::String(s) => Ok(Value::String(Arc::new(s.to_string()))),
        HolonAST::I64(n) => Ok(Value::i64(*n)),
        HolonAST::F64(x) => Ok(Value::f64(*x)),
        HolonAST::Bool(b) => Ok(Value::bool(*b)),
        // Arc 228 Stone 228.1 — classifier-dispatch for nested collection items.
        // Recognizes Bind(Atom(String(name)), Bundle(items)) produced by to_holon_inner.
        // Falls through to the bare-Bundle error path for unclassified Bundles.
        item if extract_classifier(item).is_some() => {
            let classifier = extract_classifier(item).unwrap();
            let inner_items = extract_classifier_inner_bundle(item).ok_or_else(|| {
                RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "classifier-wrapped Bundle (Bind(Atom(name), Bundle(...)))",
                    got: Box::new(ValueSnapshot::unavailable("classifier-wrapped non-Bundle inner"))
                })
            })?;
            match classifier.as_str() {
                "Map" => {
                    let n = inner_items.len();
                    #[allow(clippy::mutable_key_type)]
                    let mut map: std::collections::HashMap<Value, Value> =
                        std::collections::HashMap::with_capacity(n);
                    for child in inner_items.iter() {
                        match child {
                            HolonAST::Bind(k_holon, v_holon) => {
                                let k_val = from_holon_item(k_holon, op, op_span)?;
                                let v_val = from_holon_item(v_holon, op, op_span)?;
                                map.insert(k_val, v_val);
                            }
                            _ => {
                                return Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                                    op: op.into(),
                                    expected: "Bind(K, V) child in nested Map classifier-Bundle",
                                    got: Box::new(ValueSnapshot::unavailable("non-Bind child in Map classifier-Bundle inner items"))
                                }).into());
                            }
                        }
                    }
                    Ok(Value::wat__std__HashMap(Arc::new(map)))
                }
                "Set" => {
                    let mut set: HashSet<Value> = HashSet::with_capacity(inner_items.len());
                    for child in inner_items.iter() {
                        let v = from_holon_item(child, op, op_span)?;
                        set.insert(v);
                    }
                    Ok(Value::wat__std__HashSet(Arc::new(set)))
                }
                "Vector" => {
                    let n = inner_items.len();
                    let mut pairs: Vec<(i64, Value)> = Vec::with_capacity(n);
                    for child in inner_items.iter() {
                        match child {
                            HolonAST::Bind(k, v) => {
                                let idx = match k.as_ref() {
                                    HolonAST::I64(i) => *i,
                                    _ => {
                                        return Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                                            op: op.into(),
                                            expected: "I64 positional key in nested Vector classifier-Bundle",
                                            got: Box::new(ValueSnapshot::unavailable("non-I64 Bind key in Vector classifier-Bundle"))
                                        }).into());
                                    }
                                };
                                let elem = from_holon_item(v, op, op_span)?;
                                pairs.push((idx, elem));
                            }
                            _ => {
                                return Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                                    op: op.into(),
                                    expected: "Bind(I64, _) in nested Vector classifier-Bundle",
                                    got: Box::new(ValueSnapshot::unavailable("non-Bind child in Vector classifier-Bundle inner items"))
                                }).into());
                            }
                        }
                    }
                    pairs.sort_by_key(|(k, _)| *k);
                    let elems: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
                    Ok(Value::Vec(Arc::new(elems)))
                }
                "List" => {
                    let mut list = std::collections::LinkedList::new();
                    for child in inner_items.iter() {
                        let v = from_holon_item(child, op, op_span)?;
                        list.push_back(v);
                    }
                    Ok(Value::wat__core__List(Arc::new(list)))
                }
                "Tuple" => {
                    let n = inner_items.len();
                    let mut pairs: Vec<(i64, Value)> = Vec::with_capacity(n);
                    for child in inner_items.iter() {
                        match child {
                            HolonAST::Bind(k, v) => {
                                let idx = match k.as_ref() {
                                    HolonAST::I64(i) => *i,
                                    _ => {
                                        return Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                                            op: op.into(),
                                            expected: "I64 positional key in nested Tuple classifier-Bundle",
                                            got: Box::new(ValueSnapshot::unavailable("non-I64 Bind key in Tuple classifier-Bundle"))
                                        }).into());
                                    }
                                };
                                let elem = from_holon_item(v, op, op_span)?;
                                pairs.push((idx, elem));
                            }
                            _ => {
                                return Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                                    op: op.into(),
                                    expected: "Bind(I64, _) in nested Tuple classifier-Bundle",
                                    got: Box::new(ValueSnapshot::unavailable("non-Bind child in Tuple classifier-Bundle inner items"))
                                }).into());
                            }
                        }
                    }
                    pairs.sort_by_key(|(k, _)| *k);
                    let elems: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
                    Ok(Value::Tuple(Arc::new(elems)))
                }
                _ => Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "known classifier: Map, Set, Vector, List, or Tuple",
                    got: Box::new(ValueSnapshot::unavailable("unknown classifier name in nested collection item"))
                }).into()),
            }
        }
        _ => Err(RuntimeError::new(op_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "primitive leaf or classifier-wrapped collection (Bind(Atom(name), Bundle(...))) as produced by to-holon",
            got: Box::new(ValueSnapshot::unavailable("unclassified HolonAST (bare Bundle, non-classifier Bind, Permute, Thermometer, Blend, or other composite)"))
        }).into()),
    }
}


/// Build the hologram for a HolonRecord from scratch.
///
/// Shape (verified against `wat/Record.wat:157-191` + `runtime.rs:14017-14031`):
///   outer = `Bind(Atom(String(class)), Bundle(field_binds))`
///   each  = `Bind(Atom(String(name)), Atom(<to_holon(val)>))`
///
/// Capacity is checked via the shared `bundle_capacity_verdict` guard (Arc 294.c.2a).
/// Exceeded capacity → loud `RuntimeError` (construction cannot return a Result).
///
/// Called by `eval_aggregate_new` for `Nature::HolonRecord`; the caller already
/// holds `ctx` from `require_encoding_ctx`.
// Arc 294.g — `pub(crate)` (was private): the wire decode side (`edn/render.rs
// reconstruct_holon_record`) is the SECOND caller. A holon record's wire form no longer
// carries the hologram (294.g collapses the encode arms), so the receiver derives its own
// index from the decoded fields via this SAME function — no second implementation.
pub(crate) fn build_holon_hologram(
    class: &str,
    field_names: &[String],
    field_values: &[Value],
    ctx: &EncodingCtx,
    span: &Span,
) -> Result<Arc<HolonAST>, EvalBreak> {
    let field_binds: Vec<HolonAST> = field_names
        .iter()
        .zip(field_values.iter())
        .map(|(name, val)| -> Result<HolonAST, EvalBreak> {
            let val_holon = match to_holon_inner(val.clone(), span)? {
                Value::holon__HolonAST(h) => (*h).clone(),
                _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
            };
            Ok(HolonAST::Bind(
                Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(
                    name.as_str(),
                ))))),
                Arc::new(HolonAST::Atom(Arc::new(val_holon))),
            ))
        })
        .collect::<Result<_, _>>()?;

    // Capacity check via the shared guard — one guard, two callers.
    // For construction, exceeded capacity is always a loud RuntimeError
    // (mode-agnostic: the ctor cannot return a Result).
    if let Some((cost_i, budget_i)) = bundle_capacity_verdict(field_binds.len(), ctx) {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::aggregate-new".into(),
                reason: format!(
                    "holon record construction capacity exceeded: \
                     {} fields > budget {} (dim={})",
                    cost_i, budget_i, ctx.dim_count
                ),
            },
        )
        .into());
    }

    let bundle = HolonAST::bundle(field_binds);
    let class_atom = HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(class))));
    Ok(Arc::new(HolonAST::Bind(
        Arc::new(class_atom),
        Arc::new(bundle),
    )))
}


/// Arc 228 Stone 228.1 — extract the classifier name from a classifier-wrapped HolonAST.
///
/// Returns `Some(name)` if the outermost form is `Bind(Atom(String(name)), _)` — i.e., a
/// classifier-wrapped collection as produced by `to_holon_inner` (arc 228) or the Pascal-Case
/// constructor verbs (`:wat::holon::Map`, `:wat::holon::Set`, etc.).
///
/// Returns `None` for any other form (bare primitives, bare Bundles, Atoms, Permute, etc.).
///
/// Callers use this to dispatch by classifier name on the decode path (`from-holon`).
pub(crate) fn extract_classifier(holon: &HolonAST) -> Option<String> {
    match holon {
        HolonAST::Bind(key, _) => match key.as_ref() {
            HolonAST::Atom(inner) => match inner.as_ref() {
                HolonAST::String(s) => Some(s.to_string()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}


/// `(:wat::holon::Bind/left h)` helper — structural left-position accessor.
///
/// Arc 232 Stone 232.0a. Returns `Some(left)` for `HolonAST::Bind(left, _)`;
/// `None` for any other HolonAST variant. Names the STRUCTURAL fact (left
/// position of a Bind primitive), not the doctrine-conventional reading.
/// Symmetric peer of `bind_right`.
pub(crate) fn bind_left(holon: &HolonAST) -> Option<HolonAST> {
    match holon {
        HolonAST::Bind(left, _) => Some(left.as_ref().clone()),
        _ => None,
    }
}


/// `(:wat::holon::Bind/right h)` helper — structural right-position accessor.
///
/// Arc 232 Stone 232.0a. Returns `Some(right)` for `HolonAST::Bind(_, right)`;
/// `None` for any other HolonAST variant. Names the STRUCTURAL fact (right
/// position of a Bind primitive), not the doctrine-conventional reading.
/// Symmetric peer of `bind_left`.
pub(crate) fn bind_right(holon: &HolonAST) -> Option<HolonAST> {
    match holon {
        HolonAST::Bind(_, right) => Some(right.as_ref().clone()),
        _ => None,
    }
}


/// Extract the inner Bundle items from a classifier-wrapped form.
///
/// Given `Bind(Atom(String(_)), Bundle(items))`, returns a reference to `items`.
/// Returns `None` if the form is not classifier-wrapped or the inner is not a Bundle.
///
/// Used by the decode dispatch in `from_holon_item` for nested classifier-wrapped collections.
pub(crate) fn extract_classifier_inner_bundle(holon: &HolonAST) -> Option<&Vec<HolonAST>> {
    match holon {
        HolonAST::Bind(key, inner) => match key.as_ref() {
            HolonAST::Atom(atom_inner) => match atom_inner.as_ref() {
                HolonAST::String(_) => match inner.as_ref() {
                    HolonAST::Bundle(items) => Some(items),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}


/// Wrap a HolonAST in an opaque-identity `Atom` node.
///
/// Arc 225 Stone 225.1 — renamed from `value_to_atom` (which was polymorphic).
/// This function now accepts ONLY `Value::holon__HolonAST`; the polymorphic UP
/// arms moved to `eval_holon_to_holon` / `to_holon_inner`.
pub(crate) fn wrap_holon_as_atom(v: Value, arg_span: &Span) -> Result<Value, EvalBreak> {
    match v {
        Value::holon__HolonAST(h) => Ok(Value::holon__HolonAST(Arc::new(HolonAST::Atom(h)))),
        other => Err(RuntimeError::new(
            arg_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::holon::Atom".into(),
                expected: ":wat::holon::HolonAST (use :wat::holon::to-holon for other types)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


// Arc 294.j RELAND — widened from private to `pub(crate)`: `edn::render`'s corrected HolonAST
// decoder (`edn_derive_holon`, DESIGN-STONE-294.j ⛔ CORRECTION) composes this with
// `edn_to_value` to derive a HolonAST from decoded data — the SAME holon-side lift
// `:wat::holon::literal` (`#holon <form>`, arc 294.b) already uses. Adopting an existing total
// function, not writing a second HolonAST-from-Value builder.
pub(crate) fn to_holon_inner(v: Value, arg_span: &Span) -> Result<Value, EvalBreak> {
    // Arc 225 Stone 225.1 — the polymorphic UP body, moved here from the
    // retired `value_to_atom`. All arms preserved in semantics; rename only.
    let holon = match v {
        // Primitive leaves ───────────────────────────────────────────
        Value::i64(n) => HolonAST::i64(n),
        Value::f64(x) => HolonAST::f64(x),
        Value::bool(b) => HolonAST::bool_(b),
        Value::String(s) => HolonAST::string(s.as_str()),
        // Arc 221 Stone 221.4 — Keyword primitive → HolonAST::Keyword leaf.
        // Stone 221.3 minted HolonAST::Keyword in holon-rs commit fa48b39.
        // Value::wat__core__keyword stores with leading colon (constructor at
        // src/runtime.rs:7111 formats ":{name}"). HolonAST::keyword(&k) strips
        // the colon at the boundary per Stone 221.3 doctrine (stored content has
        // no leading colon). Pre-arc-221 used HolonAST::symbol(k.as_str()) which
        // violated the honest-primitive discipline; retired here.
        Value::wat__core__keyword(k) => HolonAST::keyword(&k),
        // Arc 230: Value::Unit (wat's nil) → HolonAST::nil() composition.
        // Arc 221 minted HolonAST::Nil; arc 230 supersedes with Bind composition.
        // HolonAST::nil() = Bind(Atom(String("Symbol")), Atom(String("nil"))).
        Value::Unit => HolonAST::nil(),
        // Arc 221 Stone 221.4 — Uuid → HolonAST::Bind(Tag("uuid"), String(hex)).
        // Closes arc 207 false-flag (5-day-latent gap since 2026-05-17).
        // Uses tagged composition per arc 221 doctrine correction — bare-leaf
        // payload in Bind, NOT Atom-wrapped. HolonAST::tag("uuid") strips the '#'
        // if present; "uuid" has no '#'. The hex representation is the canonical
        // lowercase hyphenated UUID string.
        Value::wat__core__Uuid(u) => {
            HolonAST::bind(HolonAST::tag("uuid"), HolonAST::string(u.to_string()))
        }
        // Arc 221 Stone 221.2 — Char primitive → HolonAST::Char leaf.
        // Stone 221.1 minted HolonAST::Char + char_() constructor in holon-rs
        // commit 243eded. Char is a proper primitive (BMP-only Unicode scalar),
        // not a convention-based encoding inside an existing leaf.
        Value::wat__core__Char(c) => HolonAST::char_(c),
        // Opaque-identity wrap ───────────────────────────────────────
        // HolonAST input → Atom(inner) wrap; the to-holon verb is the general
        // lift, and for HolonAST inputs it behaves identically to narrow Atom.
        Value::holon__HolonAST(h) => HolonAST::Atom(h),
        // Structural lowering of a captured wat form ────────────────
        Value::wat__WatAST(a) => watast_to_holon(&a),
        // Arc 216 Stone 1 — (HashSet :- [T]) → classifier-wrapped Bundle of bare items.
        // Arc 228 Stone 228.1 supersedes arc 216 bare-Bundle encoding per the
        // typed-entities doctrine: every collection carries its classifier at substrate.
        // Output: Bind(Atom("Set"), Bundle(bare items)).
        // Stone 216.5b — iterate s.iter() (Values directly, not String keys).
        Value::wat__std__HashSet(s) => {
            let mut items: Vec<HolonAST> = Vec::with_capacity(s.len());
            for elem in s.iter() {
                let holon_val = to_holon_inner(elem.clone(), arg_span)?;
                match holon_val {
                    Value::holon__HolonAST(h) => items.push((*h).clone()),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                }
            }
            let inner_bundle = HolonAST::bundle(items);
            let classified = HolonAST::bind(
                HolonAST::Atom(Arc::new(HolonAST::string("Set"))),
                inner_bundle,
            );
            return Ok(Value::holon__HolonAST(Arc::new(classified)));
        }
        // Arc 216 Stone 2 — (Vector :- [T]) → classifier-wrapped positional-Bind Bundle.
        // Arc 228 Stone 228.1 supersedes arc 216 bare-Bundle encoding per the
        // typed-entities doctrine. Output: Bind(Atom("Vector"), Bundle(positional Binds)).
        // Each element's index i becomes the Bind key as HolonAST::I64(i).
        // Order is preserved — index 0 first.
        Value::Vec(v) => {
            let mut items: Vec<HolonAST> = Vec::with_capacity(v.len());
            for (i, elem) in v.iter().enumerate() {
                let holon_val = to_holon_inner(elem.clone(), arg_span)?;
                let elem_holon = match holon_val {
                    Value::holon__HolonAST(h) => (*h).clone(),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                };
                let key = HolonAST::i64(i as i64);
                items.push(HolonAST::bind(key, elem_holon));
            }
            let inner_bundle = HolonAST::bundle(items);
            let classified = HolonAST::bind(
                HolonAST::Atom(Arc::new(HolonAST::string("Vector"))),
                inner_bundle,
            );
            return Ok(Value::holon__HolonAST(Arc::new(classified)));
        }
        // Arc 216 Stone 7 — Tuple → classifier-wrapped positional-Bind Bundle.
        // Arc 228 Stone 228.1 supersedes arc 216 bare-Bundle encoding + resolves the
        // identical-encoding-as-Vec dishonesty. Output: Bind(Atom("Tuple"), Bundle(positional Binds)).
        // Bundle internals are identical to Vec (positional Binds); the OUTER Atom("Tuple")
        // vs Atom("Vector") classifier is the sole discriminator. NOW DISTINCT at substrate.
        Value::Tuple(t) => {
            let mut items: Vec<HolonAST> = Vec::with_capacity(t.len());
            for (i, elem) in t.iter().enumerate() {
                let holon_val = to_holon_inner(elem.clone(), arg_span)?;
                let elem_holon = match holon_val {
                    Value::holon__HolonAST(h) => (*h).clone(),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                };
                let key = HolonAST::i64(i as i64);
                items.push(HolonAST::bind(key, elem_holon));
            }
            let inner_bundle = HolonAST::bundle(items);
            let classified = HolonAST::bind(
                HolonAST::Atom(Arc::new(HolonAST::string("Tuple"))),
                inner_bundle,
            );
            return Ok(Value::holon__HolonAST(Arc::new(classified)));
        }
        // Arc 216 Stone 3 — (HashMap :- [K V]) → classifier-wrapped Bundle of arbitrary-K Binds.
        // Arc 228 Stone 228.1 supersedes arc 216 bare-Bundle encoding per the
        // typed-entities doctrine. Output: Bind(Atom("Map"), Bundle(K-V Binds)).
        // Iteration order is non-canonical (HashMap unordered); the produced Bundle's
        // Bind order is therefore non-deterministic. The reverse trip (from-holon)
        // reconstructs a HashMap which is also order-agnostic — round-trip is correct.
        // Stone 216.5c — iterate m.iter() for (k, v) directly (K is the native key).
        Value::wat__std__HashMap(m) => {
            let mut items: Vec<HolonAST> = Vec::with_capacity(m.len());
            for (k, v) in m.iter() {
                let k_holon_val = to_holon_inner(k.clone(), arg_span)?;
                let k_holon = match k_holon_val {
                    Value::holon__HolonAST(h) => (*h).clone(),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                };
                let v_holon_val = to_holon_inner(v.clone(), arg_span)?;
                let v_holon = match v_holon_val {
                    Value::holon__HolonAST(h) => (*h).clone(),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                };
                items.push(HolonAST::bind(k_holon, v_holon));
            }
            let inner_bundle = HolonAST::bundle(items);
            let classified = HolonAST::bind(
                HolonAST::Atom(Arc::new(HolonAST::string("Map"))),
                inner_bundle,
            );
            return Ok(Value::holon__HolonAST(Arc::new(classified)));
        }
        // Arc 228 Stone 228.1 — List (wat::core::List) → classifier-wrapped Bundle of
        // sequential bare items. Output: Bind(Atom("List"), Bundle(items)).
        // Items are sequential (like Set) but the outer Atom("List") vs Atom("Set")
        // classifier discriminates on the reverse trip. Order is preserved via LinkedList
        // iteration (front to back).
        Value::wat__core__List(l) => {
            let mut items: Vec<HolonAST> = Vec::with_capacity(l.len());
            for elem in l.iter() {
                let holon_val = to_holon_inner(elem.clone(), arg_span)?;
                match holon_val {
                    Value::holon__HolonAST(h) => items.push((*h).clone()),
                    _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
                }
            }
            let inner_bundle = HolonAST::bundle(items);
            let classified = HolonAST::bind(
                HolonAST::Atom(Arc::new(HolonAST::string("List"))),
                inner_bundle,
            );
            return Ok(Value::holon__HolonAST(Arc::new(classified)));
        }
        // Arc 293.R2.1 — Aggregate: HolonRecord exposes hologram; Record has no hologram.
        Value::Aggregate(a) => match &a.holon {
            HolonForm::Hologram(h) => {
                return Ok(Value::holon__HolonAST(Arc::new(h.as_ref().clone())));
            }
            HolonForm::Empty => {
                return Err(RuntimeError::new(
                    arg_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::holon::to-holon".into(),
                        reason: format!(
                            "base record `{}` has no holon flavor; construct a holonic record \
                         (`:wat::holon::defrecord`) to use holon operations",
                            a.class
                        ),
                    },
                )
                .into());
            }
        },
        other => {
            return Err(RuntimeError::new(arg_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::holon::to-holon".into(),
                expected: "primitive, HolonAST, quoted wat form, (HashSet :- [T]), (Vector :- [T]), Tuple, (HashMap :- [K V]), (List :- [T]), or wat::core::Record",
                got: Box::new(ValueSnapshot::of(&other))
            }).into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(holon)))
}


/// Lower a captured wat form into a HolonAST. Uniform structural
/// rule per arc 057's quote-all-the-way-down framing: every node is
/// a coordinate; lists nest as Bundle; literals collapse to their
/// matching primitive leaf; identifier scope is dropped (forms are
/// spelling, scope is resolution-time).
///
/// Arc 221 Stone 221.4b — WatAST::Keyword lowers to HolonAST::keyword()
/// (not HolonAST::symbol). HolonAST::keyword() strips the leading colon
/// per Stone 221.3 doctrine (holon-rs commit fa48b39). Pre-arc-221 used
/// HolonAST::symbol(k.as_str()) which violated the honest-primitive
/// discipline; retired here.
// Arc 294.j RELAND — briefly widened to `pub(crate)` for the first strike's `edn::render` encode
// arm; reverted here. The corrected design (DESIGN-STONE-294.j ⛔ CORRECTION) does not route
// HolonAST↔EDN through the wat-source bijection at all — `edn::render` now composes
// `from_holon_item` / `to_holon_inner` instead (both already `pub(crate)`). This fn's only
// callers are the 8 in this file (`from-wat`, macro support, etc.); private is correct again.
pub(crate) fn watast_to_holon(a: &WatAST) -> HolonAST {
    match a {
        WatAST::IntLit(n, _) => HolonAST::i64(*n),
        WatAST::FloatLit(x, _) => HolonAST::f64(*x),
        // Arc 300 stone B — SURPRISE (not in the brief's mapped rooms):
        // holon-rs's `HolonAST` has no native rational leaf (only
        // String/I64/F64/Bool/Char — see holon-rs/src/kernel/holon_ast.rs).
        // holon-rs is out of scope for this stone (it belongs to a
        // different crate, never named in the brief's room list). Lower to
        // its canonical rendered string ("n/d") — a lossy-but-honest leaf
        // encoding (same shape family as the String arm below), NOT a new
        // holon-rs primitive. Revisit if/when a holon-side Rational lands.
        WatAST::RationalLit(r, _) => HolonAST::string(format!("{}/{}", r.numer(), r.denom())),
        // Arc 300 stone C1 — same SURPRISE as Rational immediately above: no
        // native holon-rs bigint leaf. Lower to its canonical rendered string
        // ("<n>N"), same lossy-but-honest shape family as the Rational arm.
        WatAST::BigIntLit(n, _) => HolonAST::string(format!("{}N", n)),
        // Arc 300 stone D — unlike Rational/BigInt immediately above,
        // holon-rs's `HolonAST` DOES have a native `Char` leaf
        // (`holon-rs/src/kernel/holon_ast.rs:77`, `HolonAST::char_`
        // constructor) — no lossy string rendering needed.
        WatAST::CharLit(c, _) => HolonAST::char_(*c),
        WatAST::BoolLit(b, _) => HolonAST::bool_(*b),
        WatAST::StringLit(s, _) => HolonAST::string(s.as_str()),
        // Arc 244 — NilLit lowers to HolonAST::symbol("nil") — the HolonAST nil
        // representation (symmetric with the HolonAST→Value path at runtime.rs:15048).
        WatAST::NilLit(_) => HolonAST::symbol("nil"),
        // Arc 221 Stone 221.4b — Keyword lowers to HolonAST::Keyword, not Symbol.
        // HolonAST::keyword() strips the leading colon stored in WatAST::Keyword.
        WatAST::Keyword(k, _) => HolonAST::keyword(k.as_str()),
        WatAST::Symbol(ident, _) => HolonAST::symbol(ident.as_str()),
        WatAST::List(items, _) => HolonAST::bundle(items.iter().map(watast_to_holon).collect()),
        // Arc 167 slice 1 — collapses to the same Bundle shape
        // as List for the algebra-level lowering. The list /
        // vector distinction is a surface-syntax concern that
        // matters for parsing and binding-position dispatch; once
        // we cross the algebra boundary, the children form an
        // ordered structural composition either way. Honest delta:
        // a future arc that exposes vector-as-value at the
        // algebra level may re-tag these distinctly.
        WatAST::Vector(items, _) => HolonAST::bundle(items.iter().map(watast_to_holon).collect()),
        // Arc 257 slice 1 — `Map` lowers to `Bind(Atom(String("Map")), Bundle([Bind(k,v), …]))`
        // matching `from_holon_item`'s existing "Map" classifier arm (~11553).
        // The symmetric encoding ensures the holon round-trip is correct.
        WatAST::Map(pairs, _) => {
            let pair_holons: Vec<HolonAST> = pairs
                .iter()
                .map(|(k, v)| HolonAST::bind(watast_to_holon(k), watast_to_holon(v)))
                .collect();
            HolonAST::bind(HolonAST::string("Map"), HolonAST::bundle(pair_holons))
        }
        // Arc 257 slice 1 — `Set` lowers to `Bind(Atom(String("Set")), Bundle([…]))`
        // matching `from_holon_item`'s existing "Set" classifier arm.
        WatAST::Set(items, _) => {
            let elem_holons: Vec<HolonAST> = items.iter().map(watast_to_holon).collect();
            HolonAST::bind(HolonAST::string("Set"), HolonAST::bundle(elem_holons))
        }
    }
}


// Arc 294.j RELAND — briefly widened to `pub(crate)` for the first strike's `edn::render` encode
// arm; reverted here (DESIGN-STONE-294.j ⛔ CORRECTION: a HolonAST wire form is DATA, never the
// wat source form this fn renders — `edn::render` now composes `from_holon_item` /
// `to_holon_inner` instead). Its 8 `runtime.rs` callers are unaffected; private is correct again.
pub(crate) fn holon_to_watast(h: &HolonAST) -> WatAST {
    // Arc 230: Symbol/Keyword/Nil/Tag variants retired; check via accessors
    // before the generic match so the Bind arm handles generic compositions.
    // Symbol composition: bare identifier or colon-prefixed keyword.
    if let Some(s) = h.as_symbol() {
        // Arc 244 — nil is a Symbol composition; round-trip as NilLit (not the type keyword).
        if s == "nil" {
            return WatAST::NilLit(crate::rust_caller_span!());
        }
        // Colon-prefixed → keyword; bare → symbol identifier.
        if s.starts_with(':') {
            return WatAST::Keyword(s.to_string(), crate::rust_caller_span!());
        } else {
            return WatAST::Symbol(
                crate::scope::Identifier::bare(s.to_string()),
                crate::rust_caller_span!(),
            );
        }
    }
    // Keyword composition: restore leading colon for round-trip.
    if let Some(s) = h.as_keyword() {
        return WatAST::Keyword(format!(":{}", s), crate::rust_caller_span!());
    }
    // Tag composition: non-round-trip debug render (no :wat::holon::Tag constructor).
    if let Some(s) = h.as_tag() {
        return WatAST::List(
            vec![
                WatAST::Keyword(":wat::holon::Tag".into(), crate::rust_caller_span!()),
                WatAST::StringLit(s.to_string(), crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        );
    }
    match h {
        HolonAST::I64(n) => WatAST::IntLit(*n, crate::rust_caller_span!()),
        HolonAST::F64(x) => WatAST::FloatLit(*x, crate::rust_caller_span!()),
        HolonAST::Bool(b) => WatAST::BoolLit(*b, crate::rust_caller_span!()),
        HolonAST::String(s) => WatAST::StringLit(s.to_string(), crate::rust_caller_span!()),
        HolonAST::Bundle(items) => WatAST::List(
            items.iter().map(holon_to_watast).collect(),
            crate::rust_caller_span!(),
        ),
        HolonAST::Atom(inner) => WatAST::List(
            vec![
                // Arc 225 Stone 225.1 — `:wat::holon::Atom` is now the narrow constructor
                // (HolonAST → HolonAST::Atom). `to-wat` emits it here because the round-trip
                // `(to-wat h → eval-ast!)` must reconstruct the same HolonAST shape.
                WatAST::Keyword(":wat::holon::Atom".into(), crate::rust_caller_span!()),
                holon_to_watast(inner),
            ],
            crate::rust_caller_span!(),
        ),
        HolonAST::Bind(a, b) => WatAST::List(
            vec![
                WatAST::Keyword(":wat::holon::Bind".into(), crate::rust_caller_span!()),
                holon_to_watast(a),
                holon_to_watast(b),
            ],
            crate::rust_caller_span!(),
        ),
        HolonAST::Permute(child, k) => WatAST::List(
            vec![
                WatAST::Keyword(":wat::holon::Permute".into(), crate::rust_caller_span!()),
                holon_to_watast(child),
                WatAST::IntLit(*k as i64, crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        ),
        HolonAST::Thermometer { value, min, max } => WatAST::List(
            vec![
                WatAST::Keyword(
                    ":wat::holon::Thermometer".into(),
                    crate::rust_caller_span!(),
                ),
                WatAST::FloatLit(*value, crate::rust_caller_span!()),
                WatAST::FloatLit(*min, crate::rust_caller_span!()),
                WatAST::FloatLit(*max, crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        ),
        HolonAST::Blend(a, b, w1, w2) => WatAST::List(
            vec![
                WatAST::Keyword(":wat::holon::Blend".into(), crate::rust_caller_span!()),
                holon_to_watast(a),
                holon_to_watast(b),
                WatAST::FloatLit(*w1, crate::rust_caller_span!()),
                WatAST::FloatLit(*w2, crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        ),
        // Arc 300 stone D — Char primitive leaf now renders directly to
        // `WatAST::CharLit`; `(eval-ast! (to-wat char-holon))` round-trips
        // via the literal, not a `char/of` call. Was: `(:wat::core::char/of
        // "c")` (arc 221 Stone 221.2 / stone 242.1) — retired now that
        // WatAST can hold a char literal directly.
        HolonAST::Char(c) => WatAST::CharLit(*c, crate::rust_caller_span!()),
        // SlotMarker (arc 073) is a substrate-internal sentinel. Non-round-trippable.
        HolonAST::SlotMarker { min, max } => WatAST::List(
            vec![
                WatAST::Keyword(":wat::holon::SlotMarker".into(), crate::rust_caller_span!()),
                WatAST::FloatLit(*min, crate::rust_caller_span!()),
                WatAST::FloatLit(*max, crate::rust_caller_span!()),
            ],
            crate::rust_caller_span!(),
        ),
    }
}


/// `(:wat::holon::Bundle <list-of-holons>)` — superposition, with
/// Kanerva-capacity enforcement per the committed capacity-mode.
///
/// Return type is `(:Result :- [:wat::holon::HolonAST :wat::holon::CapacityExceeded])`.
/// Always. Under every mode. Callers are forced by the type system to
/// acknowledge the possibility of failure — either matching on the
/// Result explicitly or propagating with `:wat::core::try`.
///
/// Capacity math: `budget = floor(sqrt(dims))` per the lab's prior-art
/// trimming convention (`src/encoding/rhythm.rs` in holon-lab-trading).
/// At d=10_000 → budget 100; at d=4_096 → 64; at d=1_024 → 32. Matches
/// FOUNDATION's empirical "~100 at d=10k" statement exactly. There is
/// Arc 294.c.2a — Kanerva width-bound verdict for holon Bundle construction.
///
/// Returns `Some((cost_i, budget_i))` when `cost` exceeds `ctx.capacity`
/// (`floor(sqrt(ctx.dim_count))`); returns `None` when within capacity.
///
/// ONE guard, two callers — neither duplicates the cost/budget math:
///   * `eval_algebra_bundle` (the `:wat::holon::Bundle` verb) — on `Some`,
///     dispatches per `ctx.config.capacity_mode` (Panic → `panic!`;
///     Error → `Ok(Value::Result(Err(CapacityExceeded{…})))`).
///   * `build_holon_hologram` (`:wat::core::aggregate-new` for HolonRecord)
///     — on `Some`, returns `Err(RuntimeError { MalformedForm })` (loud,
///     mode-agnostic: construction cannot return a Result).
///
/// `pub(crate)` (BRIEF-construction-inside-a-fn.md, gap (b)) — a THIRD caller,
/// `freeze::validate_holon_record_capacity`, reuses this SAME guard at freeze time
/// (every registered `Nature::HolonRecord`'s OWN field count against the SAME budget),
/// closing the gap where a program could pass `--check` and freeze clean, then raise here
/// at the first construction. "ONE guard, {two->three} callers" — none duplicates the math.
pub(crate) fn bundle_capacity_verdict(cost: usize, ctx: &EncodingCtx) -> Option<(i64, i64)> {
    // ctx.capacity is floor(sqrt(ctx.dim_count)).max(1), cached at freeze.
    // For any realistic d (>= 1) this equals (d as f64).sqrt().floor() as usize.
    let budget = ctx.capacity;
    if cost > budget {
        Some((cost as i64, budget as i64))
    } else {
        None
    }
}


::wat_source_derive::wat_field_names_from!(
    CAPACITY_EXCEEDED_FIELDS,
    "wat/holon.wat",
    ":wat::holon::CapacityExceeded"
);
pub(crate) fn capacity_exceeded_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(CAPACITY_EXCEEDED_FIELDS))
        .clone()
}


/// ⚠ TAKES `&Value` (arc 255 Stone O-iv-c-0). It is the TENTH member of the `require_*` family
/// and the only one living outside `src/holon/require.rs` — which is why the stone's first pass,
/// whose blast radius was drawn around that FILE rather than around the ROLE, missed it. A
/// by-value `require_*` forces a `.clone()` at every ALGEBRA call site, which is exactly what
/// O-iv-c-0 exists to prevent; leaving this one by-value would have left `atom.rs`'s 10 sites
/// cloning while its siblings did not.
pub(crate) fn require_holon(op: &str, v: &Value) -> Result<Arc<HolonAST>, EvalBreak> {
    match v {
        Value::holon__HolonAST(h) => Ok(h.clone()),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "Holon",
                got: Box::new(ValueSnapshot::of(other)),
                // arc 138: no — takes Value, not WatAST; no source coords available
            },
        )
        .into()),
    }
}


/// Arc 234 Stone 234.5 — centralized "ensure HolonAST" helper (D1).
///
/// Accepts either a HolonAST value (existing case) OR a `Value::Aggregate(HolonRecord)`
/// (auto-extracts the pre-built hologram). Records flow through VSA
/// verbs natively without user-facing conversion calls; this helper is the
/// single site that normalises the two representations into a HolonAST.
///
/// Pattern mirrored from T1 trap-door: `hologram.as_ref().clone()` is
/// safe even when the Arc is shared (proven at Stone 234.2a eval_record_field_at).
pub(crate) fn coerce_to_holon_ast(op: &str, v: Value, arg_span: &Span) -> Result<HolonAST, EvalBreak> {
    match v {
        Value::holon__HolonAST(h) => Ok((*h).clone()),
        // Arc 293.R2.1 — Aggregate: HolonRecord exposes hologram; Record has no hologram.
        Value::Aggregate(a) => match &a.holon {
            HolonForm::Hologram(h) => Ok(h.as_ref().clone()),
            HolonForm::Empty => Err(RuntimeError::new(
                arg_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "base record `{}` has no holon flavor; construct a holonic record \
                     (`:wat::holon::defrecord`) to use holon operations",
                        a.class
                    ),
                },
            )
            .into()),
        },
        other => Err(RuntimeError::new(
            arg_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "HolonAST or wat::core::Record",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// Arc 066 — wrap a wat Value as a HolonAST Value. Used by
/// `eval-ast!` to honor its `(Result :- [HolonAST EvalError])` scheme;
/// returns TypeMismatch for Values that have no HolonAST
/// representation (channels, fns, ProgramHandles, etc.).
///
/// Reuses arc 065's named-verb conventions: primitives lift via the
/// matching HolonAST leaf constructor (same shape as
/// `:wat::holon::leaf` would produce); a Value::holon__HolonAST
/// passes through unchanged (the value IS already a HolonAST per
/// arc 057's closed algebra).
pub(crate) fn value_to_holon(op: &'static str, v: Value) -> Result<Value, EvalBreak> {
    let h = match v {
        // Primitives — same dispatch as :wat::holon::leaf.
        Value::i64(n) => HolonAST::i64(n),
        Value::f64(x) => HolonAST::f64(x),
        Value::bool(b) => HolonAST::bool_(b),
        Value::String(s) => HolonAST::string(s.as_str()),
        // Arc 221 Stone 221.4b — Keyword primitive → HolonAST::Keyword leaf.
        // Pre-arc-221 used HolonAST::symbol(k.as_str()); retired per arc 221 doctrine.
        // HolonAST::keyword() strips the leading colon (Stone 221.3 holon-rs fa48b39).
        Value::wat__core__keyword(k) => HolonAST::keyword(k.as_str()),
        // Arc 230 — Value::Unit (wat nil) → HolonAST::nil() composition.
        // nil() = Bind(Atom("Symbol"), Atom("nil")); supersedes HolonAST::Nil (retired).
        Value::Unit => HolonAST::nil(),
        // Already a HolonAST — pass through unchanged. Eval-ast!'s
        // contract is "return the form's value as a HolonAST." If
        // it's already a HolonAST, return it directly; wrapping
        // would force callers to unwrap a depth they didn't ask for.
        Value::holon__HolonAST(h) => return Ok(Value::holon__HolonAST(h)),
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "form whose terminal value has a HolonAST \
                           representation (primitive or HolonAST)",
                    got: Box::new(ValueSnapshot::of(&other)),
                    // arc 138: no — receives a Value, no originating AST in scope
                },
            )
            .into());
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(h)))
}


/// Arc 070 — try to recognize a WatAST as a holon-value shape. If
/// every node down the tree is a literal, a holon-constructor call
/// with value args, or a bare-list (Bundle-shape) of values, return
/// the corresponding HolonAST. Otherwise None.
///
/// This is what lets `eval-step!` distinguish "input was already a
/// value" (`AlreadyTerminal`) from "this step reduced a redex"
/// (`Terminal`). The substrate's accounting matters at the walker /
/// tracer / cache layer: chain length 0 vs ≥ 1.
///
/// Forms with reduction-shape (arithmetic, comparison, logical,
/// special forms, user fn calls) return None — they're β-redexes
/// and step normally.
pub(crate) fn try_recognize_holon_value(form: &WatAST) -> Option<HolonAST> {
    match form {
        WatAST::IntLit(n, _) => Some(HolonAST::i64(*n)),
        WatAST::FloatLit(x, _) => Some(HolonAST::f64(*x)),
        // Arc 300 stone B — SURPRISE (see `watast_to_holon`'s note): holon-rs
        // has no native rational leaf; lower to its canonical rendered string.
        WatAST::RationalLit(r, _) => Some(HolonAST::string(format!("{}/{}", r.numer(), r.denom()))),
        // Arc 300 stone C1 — same SURPRISE as Rational immediately above.
        WatAST::BigIntLit(n, _) => Some(HolonAST::string(format!("{}N", n))),
        // Arc 300 stone D — native holon-rs Char leaf (see `watast_to_holon`'s
        // note); no lossy string rendering needed.
        WatAST::CharLit(c, _) => Some(HolonAST::char_(*c)),
        WatAST::BoolLit(b, _) => Some(HolonAST::bool_(*b)),
        WatAST::StringLit(s, _) => Some(HolonAST::string(s.as_str())),
        // Arc 221 Stone 221.4b — Keyword value-shape recognition → HolonAST::Keyword leaf.
        // Pre-arc-221 used HolonAST::symbol(k.as_str()); retired per arc 221 doctrine.
        WatAST::Keyword(k, _) => Some(HolonAST::keyword(k.as_str())),
        // A bare Symbol could be either an unbound free variable
        // (NoStepRule territory) or a HolonAST::Symbol leaf (lifted
        // from a `holon::Symbol` per arc 057's `holon_to_watast`).
        // The substrate can't distinguish at this layer; we treat
        // it as a value-shape since the alternative (free var)
        // would still trigger NoStepRule via the existing path
        // when stepping fires. Conservative: don't recognize
        // bare symbols here; let them go to the symbol-ref error.
        WatAST::Symbol(_, _) => None,
        WatAST::List(items, _) => {
            if items.is_empty() {
                return None;
            }
            match &items[0] {
                WatAST::Keyword(k, _) => match k.as_str() {
                    ":wat::holon::Atom" if items.len() == 2 => {
                        // Arc 225 Stone 225.1 — `Atom` is now the NARROW constructor:
                        // accepts only a HolonAST value and wraps it as HolonAST::Atom(inner).
                        // At eval time, primitive literals would error (they evaluate to
                        // non-HolonAST Values). The recognizer only accepts nested holon
                        // constructor forms (whose evaluation produces HolonAST); primitive
                        // literals are rejected here so the stepper fires eval for them
                        // (which produces the honest TypeMismatch at runtime).
                        match &items[1] {
                            // Primitive literals are NOT recognized — they don't produce HolonAST.
                            // Callers passing primitives to Atom should use :wat::holon::to-holon.
                            WatAST::IntLit(_, _)
                            | WatAST::FloatLit(_, _)
                            // Arc 300 stone B — Rational joins the primitive-literal group.
                            | WatAST::RationalLit(_, _)
                            // Arc 300 stone C1 — BigInt joins it too.
                            | WatAST::BigIntLit(_, _)
                            // Arc 300 stone D — Char joins it too.
                            | WatAST::CharLit(_, _)
                            | WatAST::BoolLit(_, _)
                            | WatAST::StringLit(_, _)
                            | WatAST::Keyword(_, _) => None,
                            _ => {
                                let inner = try_recognize_holon_value(&items[1])?;
                                Some(HolonAST::Atom(std::sync::Arc::new(inner)))
                            }
                        }
                    }
                    ":wat::holon::leaf" if items.len() == 2 => {
                        // Arc 065's primitive-only constructor.
                        // Always emits a typed leaf — refuses non-
                        // primitive inputs at eval time. Recognize
                        // only when the arg is a primitive literal.
                        match &items[1] {
                            WatAST::IntLit(_, _)
                            | WatAST::FloatLit(_, _)
                            // Arc 300 stone B — Rational joins the primitive-literal group.
                            | WatAST::RationalLit(_, _)
                            // Arc 300 stone C1 — BigInt joins it too.
                            | WatAST::BigIntLit(_, _)
                            // Arc 300 stone D — Char joins it too.
                            | WatAST::CharLit(_, _)
                            | WatAST::BoolLit(_, _)
                            | WatAST::StringLit(_, _)
                            | WatAST::Keyword(_, _) => {
                                try_recognize_holon_value(&items[1])
                            }
                            _ => None,
                        }
                    }
                    ":wat::holon::Bind" if items.len() == 3 => {
                        let a = try_recognize_holon_value(&items[1])?;
                        let b = try_recognize_holon_value(&items[2])?;
                        Some(HolonAST::bind(a, b))
                    }
                    ":wat::holon::Permute" if items.len() == 3 => {
                        let inner = try_recognize_holon_value(&items[1])?;
                        let k = match &items[2] {
                            WatAST::IntLit(n, _) if *n >= 0 && *n <= i64::from(i32::MAX) => {
                                *n as i32
                            }
                            _ => return None,
                        };
                        Some(HolonAST::permute(inner, k))
                    }
                    ":wat::holon::Thermometer" if items.len() == 4 => {
                        let v = match &items[1] {
                            WatAST::FloatLit(x, _) => *x,
                            _ => return None,
                        };
                        let lo = match &items[2] {
                            WatAST::FloatLit(x, _) => *x,
                            _ => return None,
                        };
                        let hi = match &items[3] {
                            WatAST::FloatLit(x, _) => *x,
                            _ => return None,
                        };
                        Some(HolonAST::Thermometer {
                            value: v,
                            min: lo,
                            max: hi,
                        })
                    }
                    ":wat::holon::Blend" if items.len() == 5 => {
                        let a = try_recognize_holon_value(&items[1])?;
                        let b = try_recognize_holon_value(&items[2])?;
                        let w1 = match &items[3] {
                            WatAST::FloatLit(x, _) => *x,
                            _ => return None,
                        };
                        let w2 = match &items[4] {
                            WatAST::FloatLit(x, _) => *x,
                            _ => return None,
                        };
                        Some(HolonAST::blend(a, b, w1, w2))
                    }
                    // Source-form `:wat::holon::Bundle` is NOT a
                    // value-shape — it takes a `(:wat::core::Vector :T
                    // …)` arg and runs the encoder/capacity check
                    // when fired. The lifted Bundle (bare list, no
                    // keyword head) IS handled by the bare-list
                    // branch below.
                    //
                    // Any other keyword head → reduction-shape.
                    _ => None,
                },
                _ => {
                    // Bare-list head (List or Symbol). Structural
                    // Bundle lift per arc 057's
                    // `holon_to_watast(Bundle(items))` — the source
                    // shape `to-wat` produces. Recognize as a
                    // Bundle iff every child recognizes too.
                    let mut children = Vec::with_capacity(items.len());
                    for item in items {
                        children.push(try_recognize_holon_value(item)?);
                    }
                    Some(HolonAST::bundle(children))
                }
            }
        }
        // Arc 244 — NilLit is a value literal (evaluates to nil / Unit).
        // Recognized as a terminal value for the stepper.
        WatAST::NilLit(_) => Some(HolonAST::symbol("nil")),
        // Arc 167 slice 1 — vectors are not value-shape forms
        // for the stepper. They live in binding-position grammar
        // (slice 2's fn / defn signatures); the stepper sees an
        // expression tree. Refuse recognition so the caller falls
        // through to step_form, which surfaces NoStepRule.
        WatAST::Vector(_, _) => None,
        // Arc 257 slice 1 — Map/Set literals are not stepper value-shapes.
        // They evaluate to HashMap/HashSet values but the stepper path
        // doesn't reduce them; fall through to eval via NoStepRule.
        WatAST::Map(_, _) | WatAST::Set(_, _) => None,
    }
}


/// Holon-constructor argument canonicity. Admits primitives and
/// holon-constructor calls whose own args are recursively canonical.
/// This is what lets `(Bind (Atom "k") (Atom "v"))` fire as a single
/// step instead of trying to step `(Atom "k")` separately and lift
/// the typed leaf back through a primitive WatAST (where it'd lose
/// its HolonAST identity).
pub(crate) fn is_holon_arg_canonical(form: &WatAST) -> bool {
    match form {
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::Keyword(_, _) => true,
        WatAST::List(items, _) => match items.first() {
            Some(WatAST::Keyword(k, _)) => match k.as_str() {
                // Arc 225 Stone 225.1 — `to-holon` added (always returns HolonAST).
                ":wat::holon::Atom"
                | ":wat::holon::to-holon"
                | ":wat::holon::leaf"
                | ":wat::holon::Bind"
                | ":wat::holon::Permute"
                | ":wat::holon::Thermometer"
                | ":wat::holon::Blend" => items[1..].iter().all(is_holon_arg_canonical),
                // `(:wat::core::Vector :- [T] <elems>...)` — Bundle's
                // canonical input shape. The `:- [T]` param-spec is a
                // declaration (not evaluated, always canonical); the
                // elements after it are the holon elements that must
                // be recursively canonical for the parent
                // constructor to fire as a single step.
                //
                // Arc 163 slice 3d — retired `:wat::core::vec` /
                // `:wat::core::list` keywords removed; only the
                // canonical `:wat::core::Vector` arm remains.
                //
                // Arc 109 "THE LAST DOORS" door 3 — this arm used to require
                // a BARE type keyword at `items[1]` (`matches!(items[1],
                // WatAST::Keyword(_, _))`), with elements starting at
                // `items[2..]`. That is the retired spelling: the wall now
                // rejects it at parse/check time, so it can never appear
                // here, and it never learned the canonical `:- [T]` marker
                // in the first place — under it, `items[1]` is the `:-`
                // Keyword (still matches the old guard) but `items[2]` is
                // the bracket `Vector`, not an element, so `_ => false`
                // fired and Bundle's single-step path was dead for every
                // program a user could actually write
                // (`NOTE-bundle-is-coupled-to-the-retired-spelling.md`).
                // Fixed by peeling the param-spec the same way the checker
                // does — `peel_param_spec` — rather than assuming its
                // absence; the elements are whatever remains after the peel.
                ":wat::core::Vector" => {
                    let (peeled, rest) = crate::types::peel_param_spec(&items[1..]);
                    peeled.is_some() && rest.iter().all(is_holon_arg_canonical)
                }
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

