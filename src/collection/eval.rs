//! Runtime per-Type dispatch impls for the collection dispatch home.
//!
//! Contains the ~30 container-polymorphic eval functions + the 3 constructors.
//! Each function is a standalone substrate primitive; the dispatch routing
//! arms in `dispatch_keyword_head_value` (src/runtime.rs) redirect here.
//! `dispatch_substrate_impl` also calls the `*_inner` helpers directly
//! (pre-evaluated values path — no double side-effects).
//!
//! See `src/collection/mod.rs` and `docs/DISPATCH.md` for the full doctrine.

use crate::ast::WatAST;
use crate::runtime::{
    eval_inner, value_is_key_hashable, value_is_set_hashable, EvalBreak, Environment,
    RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ─── Arc 146 slice 2 — per-Type length impls ────────────────────────────────

// rune:conformare(spanless-by-domain) — the _inner helpers operate on pre-evaluated &Value with
// no originating AST in scope on any call path (eval wrappers and dispatch_substrate_impl both
// arrive value-level); crate::rust_caller_span!() in this family is the API contract, not a discipline gap.

/// Returns the length of a `Value::Vec` as `Value::i64`; pre-evaluated value path.
pub(crate) fn vector_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/length".into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/length` inner helper.
pub(crate) fn list_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__List(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/length".into(),
            expected: "(List :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Returns the length of a `Value::wat__std__HashMap` as `Value::i64`; pre-evaluated value path.
pub(crate) fn hashmap_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashMap(m) => Ok(Value::i64(m.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/length".into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Returns the length of a `Value::wat__std__HashSet` as `Value::i64`; pre-evaluated value path.
pub(crate) fn hashset_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashSet(s) => Ok(Value::i64(s.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/length".into(),
            expected: "(HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── Arc 146 slice 3 — per-Type empty? / contains? / get / conj impls ────────

pub(crate) fn vector_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok(Value::bool(xs.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/empty?".into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashmap_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashMap(m) => Ok(Value::bool(m.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/empty?".into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashset_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashSet(s) => Ok(Value::bool(s.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/empty?".into(),
            expected: "(HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/empty?` inner helper.
pub(crate) fn list_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__List(xs) => Ok(Value::bool(xs.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/empty?".into(),
            expected: "(List :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── contains? — Vector/List/HashSet use `contains?`; HashMap dispatches `contains-key?` ─────

pub(crate) fn vector_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            // Stone 216.5d — native Value::PartialEq (impl PartialEq for Value, arc 216.5a).
            // hashmap_key canonical-key crutch removed; Value: PartialEq + Eq is the contract.
            // This corrects the pre-arc-146 Vec×i64 valid-index check.
            let found = xs.iter().any(|x| x == item);
            Ok(Value::bool(found))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/contains?".into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/contains?` inner helper.
/// O(N) linear scan (LinkedList has no indexing).
pub(crate) fn list_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__List(xs) => {
            let found = xs.iter().any(|x| x == item);
            Ok(Value::bool(found))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/contains?".into(),
            expected: "(List :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashmap_contains_key_q_inner(container: &Value, key: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — native HashMap::contains_key via Value: Hash + Eq.
            // Guard: opaque-handle keys can't be in the map (rejected at insert);
            // contains-key? on an unhashable key is always false (never inserted).
            if !value_is_key_hashable(key) {
                return Ok(Value::bool(false));
            }
            Ok(Value::bool(m.contains_key(key)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/contains-key?".into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashset_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__std__HashSet(s) => {
            // Stone 216.5b — native HashSet::contains via Value: Hash + Eq.
            // hashmap_key canonical-key crutch removed.
            // Guard: opaque-handle items can't be in a HashSet (set rejects them at insert);
            // contains? on an unhashable item is always false (never inserted).
            if !value_is_set_hashable(item) {
                return Ok(Value::bool(false));
            }
            Ok(Value::bool(s.contains(item)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/contains?".into(),
            expected: "(HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── get — return type varies per arm (Option<T> vs Option<V>) ──────────────

pub(crate) fn vector_get_inner(container: &Value, index: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            let i = match index {
                Value::i64(n) => *n,
                other => {
                    return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::Vector/get".into(),
                        expected: "i64 index",
                        got: Box::new(ValueSnapshot::of(other))
                    }).into());
                }
            };
            if i < 0 || (i as usize) >= xs.len() {
                Ok(Value::Option(Arc::new(None)))
            } else {
                Ok(Value::Option(Arc::new(Some(xs[i as usize].clone()))))
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/get".into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/get` inner helper.
/// O(N) index walk (LinkedList has no random access). Returns `Option<T>`.
pub(crate) fn list_get_inner(container: &Value, index: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__List(xs) => {
            let i = match index {
                Value::i64(n) => *n,
                other => {
                    return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::List/get".into(),
                        expected: "i64 index",
                        got: Box::new(ValueSnapshot::of(other))
                    }).into());
                }
            };
            if i < 0 || (i as usize) >= xs.len() {
                Ok(Value::Option(Arc::new(None)))
            } else {
                Ok(Value::Option(Arc::new(
                    xs.iter().nth(i as usize).cloned(),
                )))
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/get".into(),
            expected: "(List :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashmap_get_inner(container: &Value, key: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — native HashMap::get via Value: Hash + Eq.
            // Guard: opaque-handle keys return None (they can never be inserted).
            if !value_is_key_hashable(key) {
                return Ok(Value::Option(Arc::new(None)));
            }
            match m.get(key) {
                Some(v) => Ok(Value::Option(Arc::new(Some(v.clone())))),
                None => Ok(Value::Option(Arc::new(None))),
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/get".into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── Arc 220 Stone 220.4 — List eval wrappers ────────────────────────────────

// ─── conj inner helpers ──────────────────────────────────────────────────────

pub(crate) fn vector_conj_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            let mut out = (**xs).clone();
            out.push(item.clone());
            Ok(Value::Vec(Arc::new(out)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/conj".into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/conj` inner helper.
/// **PREPEND** semantic per Clojure precedent (distinct from Vector/conj = APPEND).
/// `conj` on a List adds the item to the FRONT, matching `cons` behavior.
pub(crate) fn list_conj_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__List(xs) => {
            let mut out = (**xs).clone();
            out.push_front(item.clone());
            Ok(Value::wat__core__List(Arc::new(out)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/conj".into(),
            expected: "(List :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// Stone 216.5b — suppress `mutable_key_type` for `HashSet<Value>`.
// `Value` contains `Arc`-wrapped types with interior mutability (Sender, AtomicBool, etc.)
// which triggers the lint. The interior-mutability variants are opaque handles that never
// appear as set elements (guarded by `value_is_set_hashable`). The lint is a false positive
// for the Value variants actually used as HashSet elements (all structurally pure).
#[allow(clippy::mutable_key_type)]
pub(crate) fn hashset_conj_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__std__HashSet(s) => {
            // Stone 216.5b — native HashSet insert via Value: Hash + Eq.
            // Arc strategy: clone-then-new-Arc (functional; no aliased mutation).
            // hashmap_key canonical-key crutch removed.
            // Guard: reject opaque-handle variants before they reach Hash.
            if !value_is_set_hashable(item) {
                return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::HashSet/conj".into(),
                    expected: "hashable value (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                    got: Box::new(ValueSnapshot::of(item))
                }).into());
            }
            let mut out: HashSet<Value> = (**s).clone();
            out.insert(item.clone());
            Ok(Value::wat__std__HashSet(Arc::new(out)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/conj".into(),
            expected: "(HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── Arc 146 slice 4 — per-Type assoc / dissoc / keys / values / concat impls ─

// Stone 216.5c — suppress `mutable_key_type` for `HashMap<Value, Value>`.
// `Value` contains `Arc`-wrapped types with interior mutability (Sender, AtomicBool, etc.)
// which triggers the lint. The interior-mutability variants are opaque handles that never
// appear as map keys (guarded by `value_is_key_hashable`). The lint is a false positive
// for the Value variants actually used as HashMap keys (all structurally pure).
#[allow(clippy::mutable_key_type)]
pub(crate) fn hashmap_assoc_inner(container: &Value, k: &Value, v: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::HashMap/assoc";
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — native HashMap insert via Value: Hash + Eq.
            // Arc strategy: clone-then-new-Arc (functional; no aliased mutation; mirrors 216.5b).
            // Guard: reject opaque-handle keys before they reach Hash::hash.
            if !value_is_key_hashable(k) {
                return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                    got: Box::new(ValueSnapshot::of(k))
                }).into());
            }
            let mut new_map: std::collections::HashMap<Value, Value> = (**m).clone();
            new_map.insert(k.clone(), v.clone());
            Ok(Value::wat__std__HashMap(Arc::new(new_map)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// Stone 216.5c — suppress `mutable_key_type` for `HashMap<Value, Value>`.
// See comment on `hashmap_assoc_inner` for rationale.
#[allow(clippy::mutable_key_type)]
pub(crate) fn hashmap_dissoc_inner(container: &Value, k: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::HashMap/dissoc";
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — native HashMap remove via Value: Hash + Eq.
            // Arc strategy: clone-then-new-Arc (functional; mirrors hashmap_assoc_inner).
            // Guard: opaque-handle keys can't be in the map; dissoc is a no-op.
            if !value_is_key_hashable(k) {
                // Nothing to remove — return the map unchanged.
                return Ok(Value::wat__std__HashMap(m.clone()));
            }
            let mut new_map: std::collections::HashMap<Value, Value> = (**m).clone();
            new_map.remove(k);
            Ok(Value::wat__std__HashMap(Arc::new(new_map)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashmap_keys_inner(container: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::HashMap/keys";
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — SEMANTIC CORRECTION: returns actual K Values (not canonical String keys).
            // Previously: m.values().map(|(k, _v)| k.clone()) — still returned original K Values
            // (from the (canonical_key, (original_k, v)) tuple), which was correct by accident.
            // Now: m.keys().cloned() — K is the direct HashMap key; no tuple indirection.
            let ks: Vec<Value> = m.keys().cloned().collect();
            Ok(Value::Vec(Arc::new(ks)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn hashmap_values_inner(container: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::HashMap/values";
    match container {
        Value::wat__std__HashMap(m) => {
            // Stone 216.5c — native HashMap<Value, Value>; V is the direct map value.
            let vs: Vec<Value> = m.values().cloned().collect();
            Ok(Value::Vec(Arc::new(vs)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(HashMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn vector_concat_inner(left: &Value, right: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Vector/concat";
    use crate::collection::seq_container::StreamContainer;
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + ordered()).
    // Same-kind constraint preserved: Vec+Vec or PersistentVector+PersistentVector only.
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    match StreamContainer::of_value(left) {
        Some(left_container) if left_container.ordered() => {
            // Right side must be the same container kind.
            match StreamContainer::of_value(right) {
                Some(right_container) if right_container == left_container => {
                    // Dispatch over the closed enum — exhaustive, no `_`.
                    // left_container == right_container is guaranteed by the guard above.
                    match left_container {
                        StreamContainer::Vector => {
                            let Value::Vec(l) = left else { unreachable!("of_value⇒Vector") };
                            let Value::Vec(r) = right else { unreachable!("of_value⇒Vector") };
                            let mut out: Vec<Value> = Vec::with_capacity(l.len() + r.len());
                            out.extend((*l).iter().cloned());
                            out.extend((*r).iter().cloned());
                            Ok(Value::Vec(Arc::new(out)))
                        }
                        StreamContainer::PersistentVector => {
                            let Value::wat__core__PersistentVector(l) = left else { unreachable!("of_value⇒PersistentVector") };
                            let Value::wat__core__PersistentVector(r) = right else { unreachable!("of_value⇒PersistentVector") };
                            // empty ++ x = x (`DESIGN-STONE-insert-all-empty-identity`).
                            // Clone-left then unique-mut append: Array COW-copies the Vec;
                            // Tree shares the RRB spine. The old rebuild-from-empty copied
                            // both sides into a new rpds Vector.
                            if l.is_empty() {
                                return Ok(right.clone());
                            }
                            if r.is_empty() {
                                return Ok(left.clone());
                            }
                            let mut out = l.clone();
                            for elem in r.iter() {
                                out.push_back_mut(elem.clone());
                            }
                            Ok(Value::wat__core__PersistentVector(out))
                        }
                        StreamContainer::List => {
                            let Value::wat__core__List(l) = left else { unreachable!("of_value⇒List") };
                            let Value::wat__core__List(r) = right else { unreachable!("of_value⇒List") };
                            let mut out = std::collections::LinkedList::new();
                            for elem in l.iter() {
                                out.push_back(elem.clone());
                            }
                            for elem in r.iter() {
                                out.push_back(elem.clone());
                            }
                            Ok(Value::wat__core__List(Arc::new(out)))
                        }
                        // ordered() gate excludes these — named arms, genuinely dead, compiler-forced:
                        StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet | StreamContainer::Stream =>
                            unreachable!("ordered() gate excludes Tuple/WatAstList/HashSet/Stream"),
                    }
                }
                // Right side is a different (or non-ordered) container kind.
                _ => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Vector :- [T]), (PersistentVector :- [T]), or (List :- [T]) (same kind as left)",
                    got: Box::new(ValueSnapshot::of(right))
                }).into()),
            }
        }
        // Left side is not an ordered container.
        _ => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (PersistentVector :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(left))
        }).into()),
    }
}

// ─── Arc-278-0a — PersistentMap ops (mirror hashmap_* family) ────────────────
//
// `PMap` (DESIGN-STONE-promoting-map) is persistent: every mutating operation returns a NEW
// map, the original UNCHANGED. Below the promotion threshold it is an array (structural-share
// via `Arc`, cheap clone-on-write); above it, `rpds::HashTrieMapSync` (O(log n), no clone of
// contents). The arm is an implementation detail — callers here never branch on it.

/// Returns the length of a `Value::wat__core__PersistentMap` as `Value::i64`.
pub(crate) fn persistentmap_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__PersistentMap(m) => Ok(Value::i64(m.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentMap/length".into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentmap_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__PersistentMap(m) => Ok(Value::bool(m.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentMap/empty?".into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentmap_contains_key_q_inner(container: &Value, key: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__PersistentMap(m) => {
            if !value_is_key_hashable(key) {
                return Ok(Value::bool(false));
            }
            Ok(Value::bool(m.contains_key(key)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentMap/contains-key?".into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentmap_get_inner(container: &Value, key: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__PersistentMap(m) => {
            if !value_is_key_hashable(key) {
                return Ok(Value::Option(Arc::new(None)));
            }
            match m.get(key) {
                Some(v) => Ok(Value::Option(Arc::new(Some(v.clone())))),
                None => Ok(Value::Option(Arc::new(None))),
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentMap/get".into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::PersistentMap/assoc pm k v)` — persistent insert.
/// Returns a NEW PersistentMap with (k → v) added; the original `pm` is UNCHANGED.
/// Array copies the pair slice; Trie shares (`PMap::assoc`).
pub(crate) fn persistentmap_assoc_inner(container: &Value, k: &Value, v: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::PersistentMap/assoc";
    match container {
        Value::wat__core__PersistentMap(m) => {
            if !value_is_key_hashable(k) {
                return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                    got: Box::new(ValueSnapshot::of(k))
                }).into());
            }
            // PMap::assoc returns a NEW map — Array copies the pair slice; Trie shares.
            Ok(Value::wat__core__PersistentMap(m.assoc(k.clone(), v.clone())))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::PersistentMap/dissoc pm k)` — persistent remove.
/// Returns a NEW PersistentMap with key `k` removed; the original `pm` is UNCHANGED.
pub(crate) fn persistentmap_dissoc_inner(container: &Value, k: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::PersistentMap/dissoc";
    match container {
        Value::wat__core__PersistentMap(m) => {
            if !value_is_key_hashable(k) {
                // Nothing to remove — return the map unchanged (same as HashMap arm).
                return Ok(Value::wat__core__PersistentMap(m.clone()));
            }
            Ok(Value::wat__core__PersistentMap(m.dissoc(k)))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentmap_keys_inner(container: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::PersistentMap/keys";
    match container {
        Value::wat__core__PersistentMap(m) => {
            Ok(Value::Vec(Arc::new(m.keys())))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentmap_values_inner(container: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::PersistentMap/values";
    match container {
        Value::wat__core__PersistentMap(m) => {
            Ok(Value::Vec(Arc::new(m.values())))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(PersistentMap :- [K V])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::PersistentMap k1 v1 k2 v2 ...)` — constructor.
/// Takes alternating key/value pairs directly (NO leading K/V type keywords).
/// Types are inferred from the actual key/value values (checked at check-time by
/// `infer_persistentmap_constructor`). `from_pairs` chooses Array (≤8) or Trie (>8).
pub(crate) fn eval_persistentmap_ctor(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if !args.len().is_multiple_of(2) {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::PersistentMap".into(),
            reason: format!(
                "arity must be even (alternating key/value pairs); got {}",
                args.len()
            )
        }).into());
    }
    let mut pairs: Vec<(Value, Value)> = Vec::with_capacity(args.len() / 2);
    for pair in args.chunks(2) {
        let k = eval_inner(&pair[0], env, sym)?.value_owned();
        let v = eval_inner(&pair[1], env, sym)?.value_owned();
        if !value_is_key_hashable(&k) {
            return Err(RuntimeError::new(pair[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::PersistentMap".into(),
                expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                got: Box::new(ValueSnapshot::of(&k))
            }).into());
        }
        pairs.push((k, v));
    }
    Ok(Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs)))
}

// ─── Arc-278-A2 — Record ops (get/contains?/length/empty?) ─────────────────

/// Arc-278-A2 — `record_get_inner`: keyword-keyed lookup on a Record.
///
/// Resolves the keyword key to a field index via `RecordDef.field_names`;
/// returns `Some(fields[idx])` if the field exists, `None` if the keyword
/// is not a declared field (not an error — same Option<V> contract as HashMap/get).
/// Accepts `Value::Aggregate` (Record and HolonRecord natures).
pub(crate) fn record_get_inner(
    record: &Value,
    key: &Value,
    span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/get";
    // Arc 293.R2.1 — Aggregate (Record/HolonRecord).
    let agg = match record {
        Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => a,
        other => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record instance",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    };
    // Extract the bare field name from the keyword (strip leading colon).
    let key_name = match key {
        Value::wat__core__keyword(k) => {
            let s = k.as_ref().as_str();
            s.strip_prefix(':').unwrap_or(s).to_string()
        }
        other => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::keyword field name",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    };
    // Resolve field index via RecordDef.
    let type_key = format!(":{}", agg.class);
    let types = sym.types().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "record get requires the type registry".into()
    }))?;
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => a,
        _ => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("record class :{} is not registered in the TypeEnv", agg.class)
        }).into()),
    };
    match record_def.field_names().position(|n| n == key_name.as_str()) {
        Some(idx) => Ok(Value::Option(std::sync::Arc::new(Some(agg.fields[idx].clone())))),
        None => Ok(Value::Option(std::sync::Arc::new(None))),
    }
}

/// Arc-278-A2 — `record_contains_field_q_inner`: field existence test on a Record.
///
/// Returns `true` iff `key` (a keyword) names a declared field of the record's class.
/// Missing-from-fields is impossible (schema == fields shape by construction);
/// the check is purely "is this keyword a declared field name?".
pub(crate) fn record_contains_field_q_inner(
    record: &Value,
    key: &Value,
    span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/contains?";
    let agg = match record {
        Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => a,
        other => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record instance",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    };
    let key_name = match key {
        Value::wat__core__keyword(k) => {
            let s = k.as_ref().as_str();
            s.strip_prefix(':').unwrap_or(s).to_string()
        }
        other => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::keyword field name",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    };
    let type_key = format!(":{}", agg.class);
    let types = sym.types().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "record contains? requires the type registry".into()
    }))?;
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => a,
        _ => return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("record class :{} is not registered in the TypeEnv", agg.class)
        }).into()),
    };
    Ok(Value::bool(record_def.field_names().any(|n| n == key_name.as_str())))
}

/// Arc-278-A2 — `record_length_inner`: field count of a Record.
///
/// Returns the number of declared fields (= `fields.len()` = `RecordDef.field_names.len()`).
/// Does NOT need the type registry — `fields` length IS the field count.
pub(crate) fn record_length_inner(record: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/length";
    match record {
        Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => Ok(Value::i64(a.fields.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record instance",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// Arc-278-A2 — `record_empty_q_inner`: zero-field check on a Record.
///
/// Returns `true` iff the record has no declared fields.
/// Does NOT need the type registry — `fields.is_empty()` is the source of truth.
pub(crate) fn record_empty_q_inner(record: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/empty?";
    match record {
        Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => Ok(Value::bool(a.fields.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::core::Record instance",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── Arc-278-0b — PersistentVector ops (mirror vector_* family) ──────────────
//
// crate::value::pvec::PVec: persistent `push_back` returns a NEW vector; the
// original is UNCHANGED. Array copies the Vec below the promotion threshold and
// promotes at it; Tree shares the RRB spine (O(log n)). Unique `push_back_mut`
// stays Array at any length.

/// Returns the length of a `Value::wat__core__PersistentVector` as `Value::i64`.
pub(crate) fn persistentvector_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__PersistentVector(pv) => Ok(Value::i64(pv.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentVector/length".into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentvector_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__PersistentVector(pv) => Ok(Value::bool(pv.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentVector/empty?".into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

pub(crate) fn persistentvector_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__PersistentVector(pv) => {
            let found = pv.iter().any(|x| x == item);
            Ok(Value::bool(found))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentVector/contains?".into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::PersistentVector/get pv i)` — index lookup, returns `Option<T>`.
/// Arc-278-0b: mirrors std `Vector/get` — returns `Some(elem)` on hit, `None` on out-of-bounds.
/// Safe: never raises on OOB (use `(:wat::core::PersistentVector/contains? pv i)` to guard
/// before unwrapping if needed, but `None` is the preferred signal).
pub(crate) fn persistentvector_get_inner(container: &Value, index: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__PersistentVector(pv) => {
            let i = match index {
                Value::i64(n) => *n,
                other => {
                    return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::PersistentVector/get".into(),
                        expected: "i64 index",
                        got: Box::new(ValueSnapshot::of(other))
                    }).into());
                }
            };
            if i < 0 || (i as usize) >= pv.len() {
                Ok(Value::Option(Arc::new(None)))
            } else {
                Ok(Value::Option(Arc::new(Some(pv.get(i as usize).cloned().unwrap()))))
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentVector/get".into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::PersistentVector/conj pv elem)` — persistent append.
/// Returns a NEW PersistentVector with `elem` appended; the original `pv` is UNCHANGED.
/// Array copies the Vec (below threshold) or promotes; Tree shares the RRB spine.
pub(crate) fn persistentvector_conj_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__core__PersistentVector(pv) => {
            Ok(Value::wat__core__PersistentVector(pv.push_back(item.clone())))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::PersistentVector/conj".into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// `(:wat::core::Vector/extend to from)` — Arc 278: a Vector extended by every element of a
/// Vector OR a PersistentVector, in ONE build.
///
/// NOT a `concat` variant, deliberately. `concat` is same-kind by a documented invariant
/// (`vector_concat_inner`'s gate + `infer_concat`) because concatenating two different container
/// kinds leaves the RESULT kind ambiguous. `into` has no such ambiguity — its contract is that the
/// DESTINATION decides — so the mixed-kind case belongs to a verb named for extension, not
/// concatenation, and `concat`'s invariant is left untouched.
///
/// WHY IT EXISTS (measured, `probe-into-is-quadratic.wat`): `stream->vec` drained a Stream with one
/// `conj` per element, and `vector_conj_inner` copies the whole accumulator each time — so
/// `(into [] (map f coll))`, the language's standard materializer, was QUADRATIC: 8,112 ms at
/// n=40,000 against 113 ms for the identical drain into an rpds accumulator (structural sharing),
/// and 0.8 ms for a single native build. This is the one-shot conversion that lets `stream->vec`
/// drain linearly into a PersistentVector and materialize ONCE.
pub(crate) fn vector_extend_inner(to: &Value, from: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Vector/extend";
    let Value::Vec(l) = to else {
        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T])",
            got: Box::new(ValueSnapshot::of(to))
        }).into());
    };
    // Size for the FINAL length up front — the whole point is one allocation, not N.
    let extra = match from {
        Value::Vec(r) => r.len(),
        Value::wat__core__PersistentVector(r) => r.len(),
        other => {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]) or (PersistentVector :- [T])",
                got: Box::new(ValueSnapshot::of(other))
            }).into());
        }
    };
    let mut out: Vec<Value> = Vec::with_capacity(l.len() + extra);
    out.extend(l.iter().cloned());
    match from {
        Value::Vec(r) => out.extend(r.iter().cloned()),
        Value::wat__core__PersistentVector(r) => out.extend(r.iter().cloned()),
        _ => unreachable!("the arity/kind check above already rejected every other shape"),
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::PersistentVector/concat to from)` — DESIGN-STONE-into-pv-from-vector.md.
///
/// The per-Type sibling of `Vector/concat`: appends every element of `from` onto `to`,
/// returning a NEW PersistentVector (`to`/`from` unchanged). Clone-left then unique
/// `push_back_mut`: Array stays Array (COW copy of the Vec); Tree shares the RRB spine.
///
/// Deliberately its OWN function rather than a new arm inside `vector_concat_inner` above.
/// That function's same-kind-only gate (Vec+Vec / PersistentVector+PersistentVector /
/// List+List) is a load-bearing, DOCUMENTED invariant ("Same-kind constraint preserved",
/// arc-278 strike 3) shared with `insert-all'` (`rete/kernel.rs:3628`, always PV×PV) and the
/// general `concat`/`Vector/concat` surface (`infer_concat`, which explicitly rejects
/// Vector+PersistentVector as a TypeMismatch). Widening `vector_concat_inner` itself to
/// accept a mismatched-kind pair would be exactly the "widen the polymorphic surface" move
/// DESIGN-STONE-into-pv-from-vector.md rejects for `Vector/concat` — the same reasoning
/// extends one level down to its native backing fn. `to` MUST be a PersistentVector (the
/// receiver, whose kind the result preserves — DESIGN row 2); `from` may be EITHER a Vector
/// or a PersistentVector (the two schemes `infer_persistentvector_concat` type-checks).
pub(crate) fn persistentvector_concat_inner(to: &Value, from: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::PersistentVector/concat";
    let Value::wat__core__PersistentVector(l) = to else {
        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(PersistentVector :- [T])",
            got: Box::new(ValueSnapshot::of(to))
        }).into());
    };
    if l.is_empty() {
        return match from {
            Value::wat__core__PersistentVector(_) => Ok(from.clone()),
            Value::Vec(r) => {
                let mut out = crate::value::pvec::PVec::new();
                for elem in r.iter() {
                    out.push_back_mut(elem.clone());
                }
                Ok(Value::wat__core__PersistentVector(out))
            }
            other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]) or (PersistentVector :- [T])",
                got: Box::new(ValueSnapshot::of(other))
            }).into()),
        };
    }
    let mut out = l.clone();
    match from {
        Value::wat__core__PersistentVector(r) => {
            for elem in r.iter() {
                out.push_back_mut(elem.clone());
            }
        }
        Value::Vec(r) => {
            for elem in r.iter() {
                out.push_back_mut(elem.clone());
            }
        }
        other => {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]) or (PersistentVector :- [T])",
                got: Box::new(ValueSnapshot::of(other))
            }).into());
        }
    }
    Ok(Value::wat__core__PersistentVector(out))
}

/// `(:wat::core::PersistentVector e1 e2 ...)` — constructor.
/// Takes bare elements in order (NO leading type keyword).
/// Types are inferred from the actual elements (checked at check-time by
/// `infer_persistentvector_constructor`). Unique `push_back_mut` — stays Array.
pub(crate) fn eval_persistentvector_ctor(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let _ = call_span; // arity is any (0+ elements)
    let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
    for arg in args {
        let v = eval_inner(arg, env, sym)?.value_owned();
        pv.push_back_mut(v);
    }
    Ok(Value::wat__core__PersistentVector(pv))
}

// ─── Container-polymorphic rest — Vec/List/WatAST-form/PersistentVector ──────

/// `(:wat::core::rest xs)` — everything after the first element. Four dispatch arms:
///
/// - `Value::Vec` — returns a new `Vec<T>` of the tail (mirrors `slice[1..]`).
/// - `Value::wat__core__List` — returns a new `List<T>` of the tail; preserves List type identity.
/// - `Value::wat__WatAST(WatAST::List)` — form-value decomposition: returns a new `WatAST::List`
///   of the tail forms, preserving the surrounding span (arc 249 Stone 249.3a-ii).
///   This arm is reachable only in macro-expansion contexts where checker discipline is
///   relaxed; type-checked user code calling `rest` on a form-value is rejected at check time
///   (checker's `rest` arm at `src/check.rs::infer_list` rejects non-Vec/non-List types).
/// - `Value::wat__core__PersistentVector` — rebuild-from-empty via unique `push_back_mut`
///   (stays Array). Preserves PersistentVector type identity.
///
/// Runtime error if the Vec/List/form is empty. Lives here beside the per-Type impls rather than
/// `transform.rs` (which holds Vector/List-SPECIFIC seq-HOFs, not container-polymorphic dispatch).
pub(crate) fn eval_rest(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::rest".into(),
            expected: 1,
            got: args.len()
        }).into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc-278 strike 2 — classify via the registry (StreamContainer::of_value + has_tail()).
    // The registry is the single source of truth; dispatch arms below are per-container
    // implementation only — no classification logic lives here.
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.has_tail() => {
            // Dispatch: each has_tail container computes its tail.
            // Identity-preserving: rest(Container<T>) → Container<T>.
            match container {
                StreamContainer::Vector => {
                    let Value::Vec(xs) = v else { unreachable!("of_value⇒Vector") };
                    if xs.is_empty() {
                        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty Vec".into()
                        }).into());
                    }
                    let out: Vec<Value> = xs.iter().skip(1).cloned().collect();
                    Ok(Value::Vec(Arc::new(out)))
                }
                // Arc 220 Stone 220.4 — List: rest returns a new List (tail after first element).
                // Maintains type identity: List/rest → List (not Vec).
                StreamContainer::List => {
                    let Value::wat__core__List(xs) = v else { unreachable!("of_value⇒List") };
                    if xs.is_empty() {
                        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty List".into()
                        }).into());
                    }
                    let out: std::collections::LinkedList<Value> = xs.iter().skip(1).cloned().collect();
                    Ok(Value::wat__core__List(Arc::new(out)))
                }
                // Arc 249 Stone 249.3a-ii — form-value decomposition: WatAST::List/rest →
                // a new WatAST::List of the tail. Maintains form identity (List/rest → List),
                // mirroring the wat__core__List arm above. Empty form → MalformedForm;
                // of_value guarantees this is a WatAST::List so non-List branch is unreachable.
                StreamContainer::WatAstList => {
                    let Value::wat__WatAST(ast) = v else { unreachable!("of_value⇒WatAstList") };
                    match &*ast {
                        WatAST::List(children, span) => {
                            if children.is_empty() {
                                return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                                    head: ":wat::core::rest".into(),
                                    reason: "cannot take rest of empty form".into()
                                }).into());
                            }
                            let tail: Vec<WatAST> = children.iter().skip(1).cloned().collect();
                            Ok(Value::wat__WatAST(Arc::new(WatAST::List(tail, span.clone()))))
                        }
                        // Unreachable: of_value only returns WatAstList for List forms.
                        _ => unreachable!("StreamContainer::of_value guarantees WatAST::List for WatAstList"),
                    }
                }
                // Arc-278-0b — PersistentVector: rest returns a new PersistentVector (tail after first element).
                // Maintains type identity: PersistentVector/rest → PersistentVector (not Vec).
                StreamContainer::PersistentVector => {
                    let Value::wat__core__PersistentVector(pv) = v else { unreachable!("of_value⇒PersistentVector") };
                    if pv.is_empty() {
                        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty PersistentVector".into()
                        }).into());
                    }
                    // Rebuild-from-empty via unique mut — stays Array.
                    let mut out: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
                    for elem in pv.iter().skip(1) {
                        out.push_back_mut(elem.clone());
                    }
                    Ok(Value::wat__core__PersistentVector(out))
                }
                // Stone 118.B4-iii — THE WALL: `has_tail()` is FALSE for Stream now, so this
                // arm is dead — no `container` value can reach it as `Stream`. Named, not `_`,
                // so a future capability change that reopens Stream here is a compile error, not
                // a silent revival. `rest` forced one cell to discard it — the same cost as
                // `next`, but the name hid the force; the wall closes that.
                StreamContainer::Stream => unreachable!(
                    "has_tail() gate excludes Stream (Stone 118.B4-iii — THE WALL: use :wat::stream::next)"
                ),
                // has_tail() gate excludes these — named arms, genuinely dead, compiler-forced:
                StreamContainer::Tuple | StreamContainer::HashSet =>
                    unreachable!("has_tail() gate excludes Tuple/HashSet"),
            }
        }
        // ∅ N/A: container has no tail (Tuple, HashSet — nature forbids it). Stone 118.B4-iii —
        // THE WALL: Stream lands here too now (has_tail()==false) — a lazy seq has no `rest`;
        // advance it with `:wat::stream::next`, whose `NextOutcome<T> = Item(value, rest) |
        // Exhausted` is the only door a Stream yields through.
        Some(StreamContainer::Stream) => Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "(Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), or WatAST — a lazy (Stream :- [T]) has no rest; advance it with :wat::stream::next ((NextOutcome :- [T]) = Item(value, rest) | Exhausted)",
            got: Box::new(ValueSnapshot::of(&v))
        }).into()),
        Some(_) => Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "Vec, List, PersistentVector, or list form",
            got: Box::new(ValueSnapshot::of(&v))
        }).into()),
        // Not a sequence container (or WatAST non-List form — preserve that specific error).
        None => match v {
            Value::wat__WatAST(ast) => Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::rest".into(),
                expected: "Vec, List, or list form",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(ast)))
            }).into()),
            other => Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::rest".into(),
                expected: "Vec, List, or PersistentVector",
                got: Box::new(ValueSnapshot::of(&other))
            }).into()),
        }
    }
}

// ─── Constructors ────────────────────────────────────────────────────────────

pub(crate) fn eval_vector_ctor(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector".into(),
            expected: 1,
            got: 0
        }).into());
    }
    // Arc 109 ②-iii — widen to accept the `:-` reference FORM `(Head :- [T …])`
    // too, routed through `parse_type_node` (the substrate's one door reading
    // all four type node shapes, src/types/surface.rs:345), same as γ-i's
    // src/function/parse.rs:178. Additive only: the `Keyword` arm is
    // untouched (no new validation on it) so the keyword path stays
    // byte-identical; a `List` is now actually parsed as a type form — not
    // merely shape-matched — so a malformed list is still rejected.
    match &args[0] {
        WatAST::Keyword(_, _) => {}
        list @ WatAST::List(_, _) => {
            crate::types::parse_type_node(list).map_err(|e| RuntimeError::new(
                e.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::Vector".into(),
                    reason: e.to_string(),
                },
            ))?;
        }
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                head: ":wat::core::Vector".into(),
                reason: "first argument must be a `(Head :- [T …])` type form".into()
            }).into());
        }
    }
    // rune:perspicere(mumble-alias) — Result<Vec<_>, _> turbofish reads better than a
    // single-home alias would; the pattern is substrate-wide convention with no existing
    // typealias to reuse.
    let items = args[1..]
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Vec(Arc::new(items)))
}

pub(crate) fn eval_hashmap_ctor(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() < 2 {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap".into(),
            expected: 2,
            got: args.len()
        }).into());
    }
    // Arc 109 ③ — widen to accept the `:-` reference FORM `(Head :- [args])` too (a
    // `WatAST::List`), matching `:wat::program::self-peer`'s identical widening
    // (`crate::runtime::is_type_arg_shaped`) — this shape check validates and discards;
    // neither arg's content is otherwise consumed here.
    if !crate::runtime::is_type_arg_shaped(&args[0]) {
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "first two arguments must be type keywords or `(Head :- [args])` type forms (K, V); first argument is not one".into()
        }).into());
    }
    if !crate::runtime::is_type_arg_shaped(&args[1]) {
        return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "first two arguments must be type keywords or `(Head :- [args])` type forms (K, V); second argument is not one".into()
        }).into());
    }
    let pairs = &args[2..];
    if !pairs.len().is_multiple_of(2) {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: format!(
                "arity after :K :V type args must be even (alternating key/value pairs); got {}",
                pairs.len()
            )
        }).into());
    }
    // Stone 216.5c — HashMap<Value, Value> native storage; hashmap_key crutch removed.
    // Guard: reject opaque-handle keys before they reach Hash::hash (unreachable!()).
    // Arc strategy: build map locally, wrap in Arc once.
    #[allow(clippy::mutable_key_type)]
    let mut map: HashMap<Value, Value> =
        HashMap::with_capacity(pairs.len() / 2);
    for pair in pairs.chunks(2) {
        let k = eval_inner(&pair[0], env, sym)?.value_owned();
        let v = eval_inner(&pair[1], env, sym)?.value_owned();
        if !value_is_key_hashable(&k) {
            return Err(RuntimeError::new(pair[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::HashMap".into(),
                expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                got: Box::new(ValueSnapshot::of(&k))
            }).into());
        }
        map.insert(k, v);
    }
    Ok(Value::wat__std__HashMap(Arc::new(map)))
}

/// Post-splice runtime shape: `args[0]` is the element type (keyword or
/// nested type form), already unwrapped from any `:- [...]` marker by the
/// dispatch call site's `unwrap_type_param_bracket`; remaining args are
/// elements. Duplicate elements collapse (HashSet semantics; Value: Hash + Eq).
// Stone 216.5b — suppress `mutable_key_type` for `HashSet<Value>`.
// See comment on `hashset_conj_inner` for rationale.
#[allow(clippy::mutable_key_type)]
pub(crate) fn eval_hashset_ctor(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError::new(call_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet".into(),
            expected: 1,
            got: 0
        }).into());
    }
    // Arc 109 ②-iii — widen to accept the `:-` reference FORM `(Head :- [T …])`
    // too, routed through `parse_type_node` (the substrate's one door reading
    // all four type node shapes, src/types/surface.rs:345), same as γ-i's
    // src/function/parse.rs:178. Additive only: the `Keyword` arm is
    // untouched (no new validation on it) so the keyword path stays
    // byte-identical; a `List` is now actually parsed as a type form — not
    // merely shape-matched — so a malformed list is still rejected.
    match &args[0] {
        WatAST::Keyword(_, _) => {}
        list @ WatAST::List(_, _) => {
            crate::types::parse_type_node(list).map_err(|e| RuntimeError::new(
                e.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::HashSet".into(),
                    reason: e.to_string(),
                },
            ))?;
        }
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                head: ":wat::core::HashSet".into(),
                reason: "first argument must be a `(Head :- [T …])` type form".into()
            }).into());
        }
    }
    // Stone 216.5b — native HashSet<Value> insert. Value implements Hash + Eq
    // (Stone 216.5a); dedupe is handled natively by HashSet::insert semantics.
    // hashmap_key canonical-key crutch removed.
    // Guard: reject opaque-handle variants (would hit unreachable!() in Hash).
    let mut set: HashSet<Value> = HashSet::with_capacity(args.len() - 1);
    for a in &args[1..] {
        let v = eval_inner(a, env, sym)?.value_owned();
        if !value_is_set_hashable(&v) {
            return Err(RuntimeError::new(a.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::HashSet".into(),
                expected: "hashable value (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                got: Box::new(ValueSnapshot::of(&v))
            }).into());
        }
        set.insert(v);
    }
    Ok(Value::wat__std__HashSet(Arc::new(set)))
}

// ─── Arc-278-seq-1b — Tuple/WatAstList/HashSet helpers ─────────────────────────────────────

/// seq-1b — `Tuple/length`: returns the element count of a `Value::Tuple`.
pub(crate) fn tuple_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Tuple(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Tuple/length".into(),
            expected: "Tuple",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `Tuple/empty?`: returns true iff the tuple has zero elements.
pub(crate) fn tuple_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Tuple(xs) => Ok(Value::bool(xs.is_empty())),
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Tuple/empty?".into(),
            expected: "Tuple",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `WatAstList/length`: returns the child-form count of a `WatAST::List`.
pub(crate) fn watastlist_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__WatAST(ast) => match &**ast {
            WatAST::List(children, _) => Ok(Value::i64(children.len() as i64)),
            other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::WatAST::List/length".into(),
                expected: "WatAST::List",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(other.clone()))))
            }).into()),
        },
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::WatAST::List/length".into(),
            expected: "WatAST",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `WatAstList/empty?`: returns true iff the WatAST::List has zero children.
pub(crate) fn watastlist_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__WatAST(ast) => match &**ast {
            WatAST::List(children, _) => Ok(Value::bool(children.is_empty())),
            other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::WatAST::List/empty?".into(),
                expected: "WatAST::List",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(other.clone()))))
            }).into()),
        },
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::WatAST::List/empty?".into(),
            expected: "WatAST",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `Tuple/contains?`: linear scan over a Tuple's elements using PartialEq.
/// Tuple is heterogeneous so any Value is a valid element candidate.
pub(crate) fn tuple_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Tuple(xs) => {
            let found = xs.iter().any(|x| x == item);
            Ok(Value::bool(found))
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Tuple/contains?".into(),
            expected: "Tuple",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `WatAstList/contains?`: scan children of a WatAST::List; wraps each child as
/// `Value::wat__WatAST` for comparison with `item`.
pub(crate) fn watastlist_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__WatAST(ast) => match &**ast {
            WatAST::List(children, _) => {
                let found = children.iter().any(|c| {
                    Value::wat__WatAST(Arc::new(c.clone())) == *item
                });
                Ok(Value::bool(found))
            }
            other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::WatAST::List/contains?".into(),
                expected: "WatAST::List",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(other.clone()))))
            }).into()),
        },
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::WatAST::List/contains?".into(),
            expected: "WatAST",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `WatAstList/get`: index-based child-form lookup. Returns `Option<WatAST>`.
/// Out-of-bounds or negative index → `None`; in-bounds → `Some(child wrapped as Value::wat__WatAST)`.
pub(crate) fn watastlist_get_inner(container: &Value, index: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__WatAST(ast) => match &**ast {
            WatAST::List(children, _) => {
                let i = match index {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: ":wat::WatAST::List/get".into(),
                            expected: "i64 index",
                            got: Box::new(ValueSnapshot::of(other))
                        }).into());
                    }
                };
                if i < 0 || (i as usize) >= children.len() {
                    Ok(Value::Option(Arc::new(None)))
                } else {
                    Ok(Value::Option(Arc::new(Some(
                        Value::wat__WatAST(Arc::new(children[i as usize].clone()))
                    ))))
                }
            }
            other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::WatAST::List/get".into(),
                expected: "WatAST::List",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(Arc::new(other.clone()))))
            }).into()),
        },
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::WatAST::List/get".into(),
            expected: "WatAST",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

/// seq-1b — `HashSet/get`: membership-as-lookup. If `item` is in the set, returns `Some(item)`;
/// otherwise `None`. Unhashable items always return `None` (they can never be inserted).
pub(crate) fn hashset_get_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::wat__std__HashSet(s) => {
            if !value_is_set_hashable(item) {
                return Ok(Value::Option(Arc::new(None)));
            }
            if s.contains(item) {
                Ok(Value::Option(Arc::new(Some(item.clone()))))
            } else {
                Ok(Value::Option(Arc::new(None)))
            }
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/get".into(),
            expected: "(HashSet :- [T])",
            got: Box::new(ValueSnapshot::of(other))
        }).into()),
    }
}

// ─── Arc 109 ②-iii — acceptance rows 1-3 for the `Vector`/`HashSet` ctor guard widening ──
//
// `eval_vector_ctor` / `eval_hashset_ctor`'s first-arg guard now accepts the `:-`
// reference FORM `(Head :- [T …])` alongside the existing `Keyword`, routed through
// `crate::types::parse_type_node` (the substrate's one door, src/types/surface.rs:345).
//
// These are Rust-level (not `.wat` scratch-pad) probes DELIBERATELY: `startup_from_source`
// / `--check` runs `crate::check::infer_list_constructor` / `infer_hashset_constructor`
// BEFORE any eval, and those two check-time twins have the SAME keyword-only defect this
// stone fixes at the eval-time guards — but are OUT OF this stone's boundary (only
// `src/runtime.rs`'s one match arm and these two `collection/eval.rs` guards). A
// `wat-scripts/scratch-pad/*.wat` probe calling `(:wat::core::Vector (Head :- [T]) …)` or
// `(:wat::core::HashSet (Head :- [T]) …)` directly would be rejected by the checker before
// ever reaching the widened eval-time guard, and would go RED under
// `tests/lint/wat_scripts_fixes_load.rs` (which runs the identical `startup_from_source`
// gate) — a false failure of a fix that is correct at the layer it targets. `eval_in_frozen`
// evaluates a pre-parsed AST directly against a frozen world with NO re-check pass
// (`src/freeze.rs`: macro-expand then `crate::runtime::eval`), so it exercises exactly the
// two guards this stone widened, in isolation from the check-time twins' unrelated gap.
// Found and reported to the orchestrator, not fixed here (STOP-3: a third class surfaced,
// out of boundary).
#[cfg(test)]
mod arc109_two_iii_ctor_guard_widening {
    use super::{eval_hashset_ctor, eval_vector_ctor, EvalBreak, RuntimeErrorKind};
    use crate::runtime::{Environment, SymbolTable, Value};

    /// Calls `eval_vector_ctor` / `eval_hashset_ctor` DIRECTLY, on hand-built `WatAST`, with
    /// a bare `Environment`/`SymbolTable` — no `startup_from_source`/`--check`/`eval_in_frozen`
    /// world-build. Two reasons, both load-bearing:
    ///
    /// 1. `startup_from_source` (which BOTH `--check` and `eval_in_frozen`'s world-build run
    ///    through) currently fails to freeze the STDLIB ITSELF: the corpus migration this
    ///    stone ships (arc 109 ②-iii) rewrote `wat/fix.wat` / `cache.wat` / `lint.wat` /
    ///    `bracket.wat` / `spawn.wat`'s OWN `(:wat::core::Vector :(a,b,c) …)` tuple-keyword
    ///    constructor calls into the `(:wat::core::Vector (:wat::core::Tuple :- [...]) …)`
    ///    form — and `crate::check::infer_list_constructor` / `infer_hashset_constructor`
    ///    (check.rs's OWN, textually independent copy of this exact "first arg is a type
    ///    keyword" guard) has the SAME keyword-only defect this stone fixes in
    ///    `collection/eval.rs`, unfixed. 37 `CheckError`s, none reachable from this stone's
    ///    boundary (`src/runtime.rs`'s one match arm + these two guards) — reported to the
    ///    orchestrator as a found THIRD class (STOP-3), not fixed here.
    /// 2. Calling the two guards directly is ALSO the more precise instrument: it isolates
    ///    exactly the code this stone changed from macro-expansion, symbol resolution, and
    ///    check.rs's unrelated (and, per point 1, currently broken) parallel implementation.
    fn env_sym() -> (Environment, SymbolTable) {
        (Environment::new(), SymbolTable::new())
    }

    fn i64_lit(n: i64) -> WatAST {
        WatAST::int(n)
    }

    fn kw(s: &str) -> WatAST {
        WatAST::Keyword(s.into(), crate::rust_caller_span!())
    }

    fn list(items: Vec<WatAST>) -> WatAST {
        WatAST::List(items, crate::rust_caller_span!())
    }

    fn vect(items: Vec<WatAST>) -> WatAST {
        WatAST::Vector(items, crate::rust_caller_span!())
    }

    use super::WatAST;

    /// Row 3 (the row that decides the stone) — the KEYWORD path is untouched. Same
    /// argument shape as before the widening, still accepted, still yields the SAME value.
    #[test]
    fn row3_vector_ctor_keyword_first_arg_unchanged() {
        let (env, sym) = env_sym();
        let args = vec![kw(":wat::core::i64"), i64_lit(1), i64_lit(2), i64_lit(3)];
        let v = eval_vector_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .unwrap_or_else(|e| panic!("keyword-typed Vector ctor must still eval: {e:?}"));
        assert_eq!(v, Value::Vec(std::sync::Arc::new(vec![Value::i64(1), Value::i64(2), Value::i64(3)])));
    }

    #[test]
    fn row3_hashset_ctor_keyword_first_arg_unchanged() {
        let (env, sym) = env_sym();
        let args = vec![kw(":wat::core::i64"), i64_lit(1), i64_lit(2), i64_lit(2), i64_lit(3)];
        let v = eval_hashset_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .unwrap_or_else(|e| panic!("keyword-typed HashSet ctor must still eval: {e:?}"));
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 3, "1,2,2,3 dedupes to 3 elements"),
            other => panic!("expected HashSet, got {other:?}"),
        }
    }

    /// Row 3 negative control — a first arg that was rejected BEFORE the widening (neither a
    /// `Keyword` nor now a `List` — an i64 literal) must still be rejected after it, with the
    /// SAME diagnostic text. Proves the widening did not become "accepts anything".
    #[test]
    fn row3_vector_ctor_still_rejects_non_type_first_arg() {
        let (env, sym) = env_sym();
        let args = vec![i64_lit(1), i64_lit(2), i64_lit(3)];
        let err = eval_vector_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .expect_err("a plain i64 first arg must still be rejected");
        // Structured, not string-matched (`no_loose_string_assert`'s own remedy — ask
        // through the door, whose argument is an enum): the pre-existing diagnostic
        // shape, byte-identical to before the widening.
        match err {
            EvalBreak::Diagnostic(e) => assert_eq!(
                format!("{:?}", e.kind()),
                format!(
                    "{:?}",
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::Vector".into(),
                        reason: "first argument must be a `(Head :- [T …])` type form".into()
                    }
                )
            ),
            other => panic!("expected Diagnostic, got {other:?}"),
        }
    }

    #[test]
    fn row3_hashset_ctor_still_rejects_non_type_first_arg() {
        let (env, sym) = env_sym();
        let args = vec![i64_lit(1), i64_lit(2), i64_lit(3)];
        let err = eval_hashset_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .expect_err("a plain i64 first arg must still be rejected");
        match err {
            EvalBreak::Diagnostic(e) => assert_eq!(
                format!("{:?}", e.kind()),
                format!(
                    "{:?}",
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::HashSet".into(),
                        reason: "first argument must be a `(Head :- [T …])` type form".into()
                    }
                )
            ),
            other => panic!("expected Diagnostic, got {other:?}"),
        }
    }

    /// Row 2 — `vec` / `HashSet` take a form first-arg: `(Head :- [T …])`, routed through
    /// `parse_type_node`. Element type is itself parametric (`Tuple :- [i64 i64]`) — the
    /// exact shape the corpus migration produces (`wat/fix.wat`, `cache.wat`, `lint.wat`,
    /// `bracket.wat`, `spawn.wat`).
    #[test]
    fn row2_vector_ctor_accepts_parametric_form_first_arg() {
        let (env, sym) = env_sym();
        let ty = list(vec![
            kw(":wat::core::Tuple"),
            kw(":-"),
            vect(vec![kw(":wat::core::i64"), kw(":wat::core::i64")]),
        ]);
        let args = vec![ty, i64_lit(1), i64_lit(2), i64_lit(3)];
        let v = eval_vector_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .unwrap_or_else(|e| panic!("form-typed Vector ctor must eval: {e:?}"));
        match v {
            Value::Vec(xs) => assert_eq!(xs.len(), 3),
            other => panic!("expected Vec, got {other:?}"),
        }
    }

    #[test]
    fn row2_hashset_ctor_accepts_parametric_form_first_arg() {
        let (env, sym) = env_sym();
        let ty = list(vec![
            kw(":wat::core::Tuple"),
            kw(":-"),
            vect(vec![kw(":wat::core::i64"), kw(":wat::core::i64")]),
        ]);
        let args = vec![ty, i64_lit(1), i64_lit(2), i64_lit(3)];
        let v = eval_hashset_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .unwrap_or_else(|e| panic!("form-typed HashSet ctor must eval: {e:?}"));
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 3),
            other => panic!("expected HashSet, got {other:?}"),
        }
    }

    /// Row 2 negative control — a MALFORMED form (the head is not a valid type constructor
    /// keyword — a bare i64-shaped List) must still be rejected. Proves the `List` arm
    /// actually parses via `parse_type_node`, rather than accepting any `List`
    /// unconditionally.
    #[test]
    fn row2_vector_ctor_rejects_malformed_form_first_arg() {
        let (env, sym) = env_sym();
        let malformed_ty = list(vec![i64_lit(1), i64_lit(2), i64_lit(3)]);
        let args = vec![malformed_ty, i64_lit(1)];
        let err = eval_vector_ctor(&args, &crate::rust_caller_span!(), &env, &sym)
            .expect_err("a List that is not a valid type form must still be rejected");
        eprintln!("row2_vector_ctor_rejects_malformed_form_first_arg: {err:?}");
    }
}
