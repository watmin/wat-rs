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

pub(crate) fn vector_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/length".into(),
            expected: "Vec<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/length` inner helper.
pub(crate) fn list_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__List(xs) => Ok(Value::i64(xs.len() as i64)),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/length".into(),
            expected: "List<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn hashmap_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashMap(m) => Ok(Value::i64(m.len() as i64)),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/length".into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn hashset_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashSet(s) => Ok(Value::i64(s.len() as i64)),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/length".into(),
            expected: "HashSet<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn eval_vector_length(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/length".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    vector_length_inner(&v)
}

pub(crate) fn eval_hashmap_length(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/length".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    hashmap_length_inner(&v)
}

pub(crate) fn eval_hashset_length(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet/length".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    hashset_length_inner(&v)
}

// ─── Arc 146 slice 3 — per-Type empty? / contains? / get / conj impls ────────

pub(crate) fn vector_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok(Value::bool(xs.is_empty())),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/empty?".into(),
            expected: "Vec<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn hashmap_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashMap(m) => Ok(Value::bool(m.is_empty())),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/empty?".into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn hashset_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__std__HashSet(s) => Ok(Value::bool(s.is_empty())),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/empty?".into(),
            expected: "HashSet<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

/// Arc 220 Stone 220.4 — `:wat::core::List/empty?` inner helper.
pub(crate) fn list_empty_q_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v {
        Value::wat__core__List(xs) => Ok(Value::bool(xs.is_empty())),
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/empty?".into(),
            expected: "List<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn eval_vector_empty_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/empty?".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    vector_empty_q_inner(&v)
}

pub(crate) fn eval_hashmap_empty_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/empty?".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    hashmap_empty_q_inner(&v)
}

pub(crate) fn eval_hashset_empty_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet/empty?".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    hashset_empty_q_inner(&v)
}

// ─── contains? — MIXED VERBS ────────────────────────────────────────────────

pub(crate) fn vector_contains_q_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            // Stone 216.5d — native Value::PartialEq (impl PartialEq for Value, arc 216.5a).
            // hashmap_key canonical-key crutch removed; Value: PartialEq + Eq is the contract.
            // This corrects the pre-arc-146 Vec×i64 valid-index check.
            let found = xs.iter().any(|x| x == item);
            Ok(Value::bool(found))
        }
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/contains?".into(),
            expected: "Vec<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/contains?".into(),
            expected: "List<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/contains-key?".into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/contains?".into(),
            expected: "HashSet<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn eval_vector_contains_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/contains?".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    vector_contains_q_inner(&container, &item)
}

pub(crate) fn eval_hashmap_contains_key_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/contains-key?".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let key = eval_inner(&args[1], env, sym)?.value_owned();
    hashmap_contains_key_q_inner(&container, &key)
}

pub(crate) fn eval_hashset_contains_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet/contains?".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    hashset_contains_q_inner(&container, &item)
}

// ─── get — return type varies per arm (Option<T> vs Option<V>) ──────────────

pub(crate) fn vector_get_inner(container: &Value, index: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            let i = match index {
                Value::i64(n) => *n,
                other => {
                    return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::Vector/get".into(),
                        expected: "i64 index",
                        got: Box::new(ValueSnapshot::of(other))
                    } }.into());
                }
            };
            if i < 0 || (i as usize) >= xs.len() {
                Ok(Value::Option(Arc::new(None)))
            } else {
                Ok(Value::Option(Arc::new(Some(xs[i as usize].clone()))))
            }
        }
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/get".into(),
            expected: "Vec<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
                    return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                        op: ":wat::core::List/get".into(),
                        expected: "i64 index",
                        got: Box::new(ValueSnapshot::of(other))
                    } }.into());
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/get".into(),
            expected: "List<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashMap/get".into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn eval_vector_get(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/get".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let index = eval_inner(&args[1], env, sym)?.value_owned();
    vector_get_inner(&container, &index)
}

pub(crate) fn eval_hashmap_get(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/get".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let key = eval_inner(&args[1], env, sym)?.value_owned();
    hashmap_get_inner(&container, &key)
}

// ─── Arc 220 Stone 220.4 — List eval wrappers ────────────────────────────────

pub(crate) fn eval_list_length(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::List/length".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    list_length_inner(&v)
}

pub(crate) fn eval_list_empty_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::List/empty?".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    list_empty_q_inner(&v)
}

pub(crate) fn eval_list_contains_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::List/contains?".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    list_contains_q_inner(&container, &item)
}

pub(crate) fn eval_list_get(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::List/get".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let index = eval_inner(&args[1], env, sym)?.value_owned();
    list_get_inner(&container, &index)
}

pub(crate) fn eval_list_conj(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::List/conj".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    list_conj_inner(&container, &item)
}

// ─── conj inner helpers ──────────────────────────────────────────────────────

pub(crate) fn vector_conj_inner(container: &Value, item: &Value) -> Result<Value, EvalBreak> {
    match container {
        Value::Vec(xs) => {
            let mut out = (**xs).clone();
            out.push(item.clone());
            Ok(Value::Vec(Arc::new(out)))
        }
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::Vector/conj".into(),
            expected: "Vec<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::List/conj".into(),
            expected: "List<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
                return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::HashSet/conj".into(),
                    expected: "hashable value (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)",
                    got: Box::new(ValueSnapshot::of(item))
                } }.into());
            }
            let mut out: HashSet<Value> = (**s).clone();
            out.insert(item.clone());
            Ok(Value::wat__std__HashSet(Arc::new(out)))
        }
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::HashSet/conj".into(),
            expected: "HashSet<T>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn eval_vector_conj(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/conj".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    vector_conj_inner(&container, &item)
}

pub(crate) fn eval_hashset_conj(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet/conj".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let item = eval_inner(&args[1], env, sym)?.value_owned();
    hashset_conj_inner(&container, &item)
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
                return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "hashable key (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)",
                    got: Box::new(ValueSnapshot::of(k))
                } }.into());
            }
            let mut new_map: std::collections::HashMap<Value, Value> = (**m).clone();
            new_map.insert(k.clone(), v.clone());
            Ok(Value::wat__std__HashMap(Arc::new(new_map)))
        }
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
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
        other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "HashMap<K,V>",
            got: Box::new(ValueSnapshot::of(other))
        } }.into()),
    }
}

pub(crate) fn vector_concat_inner(left: &Value, right: &Value) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Vector/concat";
    let l = match left {
        Value::Vec(xs) => xs.clone(),
        other => {
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vec<T>",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let r = match right {
        Value::Vec(xs) => xs.clone(),
        other => {
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vec<T>",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let mut out: Vec<Value> = Vec::with_capacity(l.len() + r.len());
    out.extend((*l).iter().cloned());
    out.extend((*r).iter().cloned());
    Ok(Value::Vec(Arc::new(out)))
}

pub(crate) fn eval_hashmap_assoc(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/assoc".into(),
            expected: 3,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let k = eval_inner(&args[1], env, sym)?.value_owned();
    let v = eval_inner(&args[2], env, sym)?.value_owned();
    hashmap_assoc_inner(&container, &k, &v)
}

pub(crate) fn eval_hashmap_dissoc(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/dissoc".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    let k = eval_inner(&args[1], env, sym)?.value_owned();
    hashmap_dissoc_inner(&container, &k)
}

pub(crate) fn eval_hashmap_keys(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/keys".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    hashmap_keys_inner(&container)
}

pub(crate) fn eval_hashmap_values(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap/values".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let container = eval_inner(&args[0], env, sym)?.value_owned();
    hashmap_values_inner(&container)
}

pub(crate) fn eval_vector_concat(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector/concat".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let left = eval_inner(&args[0], env, sym)?.value_owned();
    let right = eval_inner(&args[1], env, sym)?.value_owned();
    vector_concat_inner(&left, &right)
}

// ─── Container-polymorphic sequence ops ─────────────────────────────────────

/// `(:wat::core::rest xs)` — everything after the first element of a
/// Vec or List. Mirrors `slice[1..]` for Vec. Runtime error if `xs` is
/// empty. Branches on `Value::Vec` vs `Value::wat__core__List`
/// (container-polymorphic dispatch); lives here beside the per-Type
/// impls rather than `transform.rs` (which holds Vector/List-SPECIFIC
/// seq-HOFs, not container-polymorphic dispatch).
pub(crate) fn eval_vec_rest(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::rest".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    match v {
        Value::Vec(xs) => {
            if xs.is_empty() {
                return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::rest".into(),
                    reason: "cannot take rest of empty Vec".into()
                } }.into());
            }
            let out: Vec<Value> = xs.iter().skip(1).cloned().collect();
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc 220 Stone 220.4 — List: rest returns a new List (tail after first element).
        // Maintains type identity: List/rest → List (not Vec).
        Value::wat__core__List(xs) => {
            if xs.is_empty() {
                return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::rest".into(),
                    reason: "cannot take rest of empty List".into()
                } }.into());
            }
            let out: std::collections::LinkedList<Value> = xs.iter().skip(1).cloned().collect();
            Ok(Value::wat__core__List(Arc::new(out)))
        }
        other => Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "Vec or List",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

// ─── Constructors ────────────────────────────────────────────────────────────

pub(crate) fn eval_vector_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::Vector".into(),
            expected: 1,
            got: 0
        } }.into());
    }
    if !matches!(&args[0], WatAST::Keyword(_, _)) {
        return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::core::Vector".into(),
            reason: "first argument must be a type keyword (e.g., :i64)".into()
        } }.into());
    }
    let items = args[1..]
        .iter()
        .map(|a| eval_inner(a, env, sym).map(|tv| tv.value_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Vec(Arc::new(items)))
}

pub(crate) fn eval_hashmap_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() < 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashMap".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    if !matches!(&args[0], WatAST::Keyword(_, _)) {
        return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "first two arguments must be type keywords (K, V); first argument is not a keyword".into()
        } }.into());
    }
    if !matches!(&args[1], WatAST::Keyword(_, _)) {
        return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "first two arguments must be type keywords (K, V); second argument is not a keyword".into()
        } }.into());
    }
    let pairs = &args[2..];
    if !pairs.len().is_multiple_of(2) {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: format!(
                "arity after :K :V type args must be even (alternating key/value pairs); got {}",
                pairs.len()
            )
        } }.into());
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
            return Err(RuntimeError { span: pair[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::HashMap".into(),
                expected: "hashable key (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)",
                got: Box::new(ValueSnapshot::of(&k))
            } }.into());
        }
        map.insert(k, v);
    }
    Ok(Value::wat__std__HashMap(Arc::new(map)))
}

/// `(:wat::core::HashSet :T x1 x2 x3 ...)` — first arg is a type
/// keyword read by the checker; remaining args are elements. Duplicate
/// elements collapse (last stored wins on the exact canonical key).
// Stone 216.5b — suppress `mutable_key_type` for `HashSet<Value>`.
// See comment on `hashset_conj_inner` for rationale.
#[allow(clippy::mutable_key_type)]
pub(crate) fn eval_hashset_ctor(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.is_empty() {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::HashSet".into(),
            expected: 1,
            got: 0
        } }.into());
    }
    if !matches!(&args[0], WatAST::Keyword(_, _)) {
        return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
            head: ":wat::core::HashSet".into(),
            reason: "first argument must be a type keyword (e.g., :i64)".into()
        } }.into());
    }
    // Stone 216.5b — native HashSet<Value> insert. Value implements Hash + Eq
    // (Stone 216.5a); dedupe is handled natively by HashSet::insert semantics.
    // hashmap_key canonical-key crutch removed.
    // Guard: reject opaque-handle variants (would hit unreachable!() in Hash).
    let mut set: HashSet<Value> = HashSet::with_capacity(args.len() - 1);
    for a in &args[1..] {
        let v = eval_inner(a, env, sym)?.value_owned();
        if !value_is_set_hashable(&v) {
            return Err(RuntimeError { span: a.span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::HashSet".into(),
                expected: "hashable value (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)",
                got: Box::new(ValueSnapshot::of(&v))
            } }.into());
        }
        set.insert(v);
    }
    Ok(Value::wat__std__HashSet(Arc::new(set)))
}
