//! `:rust::lru::LruCache<K,V>` — wat binding for the `lru` crate.
//!
//! Surfaced as an opaque handle to an `lru::LruCache<String, Value>`
//! (the `<K,V>` type parameters are enforced by the type checker;
//! runtime storage collapses to canonical-string-keyed `Value`,
//! same shape as HashMap/HashSet).
//!
//! # Scope discipline — `thread_owned`
//!
//! An `LruCache` is SINGLE-THREAD-OWNED by construction — the `Arc`
//! that wraps it carries a `thread::ThreadId` guard that every op
//! asserts before touching the `UnsafeCell`. A value that crosses
//! a thread boundary (e.g., via channel send) errors at the first
//! subsequent op with a clear `MalformedForm` message. Zero Mutex,
//! zero RwLock — the guard is structural, not contended.
//!
//! This is the SHIM implementation for wat-rs's default registry.
//! When `#[wat_dispatch]` lands (design in
//! `docs/wat-dispatch-macro-design-2026-04-19.md`), this file
//! becomes a macro-annotated newtype and the hand-written dispatch/
//! scheme/register below go away.

use std::cell::UnsafeCell;
use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::rust_deps::{RustDepsBuilder, RustSymbol, RustTypeDecl, SchemeCtx};
use crate::runtime::{eval, hashmap_key, Environment, RuntimeError, SymbolTable, Value};
use crate::types::TypeExpr;

/// The per-instance state behind a `:rust::lru::LruCache` Value. Tied
/// to its creating thread by a `ThreadId` guard.
pub struct LruCacheCell {
    owner: std::thread::ThreadId,
    cache: UnsafeCell<lru::LruCache<String, Value>>,
}

impl std::fmt::Debug for LruCacheCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LruCacheCell {{ owner: {:?} }}", self.owner)
    }
}

// Safety: every access routes through `with_mut`, which asserts
// `thread::current().id() == self.owner` before dereferencing the
// UnsafeCell. Only one thread can ever reach the UnsafeCell; the
// interpreter is single-threaded within that thread and closures
// passed to `with_mut` do not recurse into wat evaluation against
// the same cell.
unsafe impl Send for LruCacheCell {}
unsafe impl Sync for LruCacheCell {}

impl LruCacheCell {
    pub fn new(capacity: usize) -> Self {
        let nz = NonZeroUsize::new(capacity).expect("LruCache capacity must be non-zero");
        Self {
            owner: std::thread::current().id(),
            cache: UnsafeCell::new(lru::LruCache::new(nz)),
        }
    }

    fn ensure_owner(&self, op: &'static str) -> Result<(), RuntimeError> {
        if std::thread::current().id() != self.owner {
            return Err(RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "LruCache crossed thread boundary (owner: {:?}, current: {:?})",
                    self.owner,
                    std::thread::current().id()
                ),
            });
        }
        Ok(())
    }

    pub fn with_mut<R>(
        &self,
        op: &'static str,
        f: impl FnOnce(&mut lru::LruCache<String, Value>) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op)?;
        // Safety: thread-owner invariant checked above. Closure is
        // called once, does not recurse into Value evaluation against
        // this cell.
        Ok(unsafe { f(&mut *self.cache.get()) })
    }
}

/// Extract the `Arc<LruCacheCell>` from a wat `Value`. The Value
/// variant `Value::rust__lru__LruCache` is the wat-visible form.
fn require_lru(op: &'static str, v: Value) -> Result<Arc<LruCacheCell>, RuntimeError> {
    match v {
        Value::rust__lru__LruCache(c) => Ok(c),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "rust::lru::LruCache",
            got: other.type_name(),
        }),
    }
}

// ─── Dispatch fns ────────────────────────────────────────────────────

fn dispatch_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":rust::lru::LruCache::new".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let capacity = match eval(&args[0], env, sym)? {
        Value::i64(n) if n > 0 => n as usize,
        Value::i64(n) => {
            return Err(RuntimeError::MalformedForm {
                head: ":rust::lru::LruCache::new".into(),
                reason: format!("capacity must be positive; got {}", n),
            });
        }
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":rust::lru::LruCache::new".into(),
                expected: "i64",
                got: other.type_name(),
            });
        }
    };
    Ok(Value::rust__lru__LruCache(Arc::new(LruCacheCell::new(
        capacity,
    ))))
}

fn dispatch_put(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: ":rust::lru::LruCache::put".into(),
            expected: 3,
            got: args.len(),
        });
    }
    let cell = require_lru(":rust::lru::LruCache::put", eval(&args[0], env, sym)?)?;
    let k = eval(&args[1], env, sym)?;
    let v = eval(&args[2], env, sym)?;
    let key = hashmap_key(":rust::lru::LruCache::put", &k)?;
    cell.with_mut(":rust::lru::LruCache::put", |c| {
        c.put(key, v);
    })?;
    Ok(Value::Unit)
}

fn dispatch_get(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":rust::lru::LruCache::get".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let cell = require_lru(":rust::lru::LruCache::get", eval(&args[0], env, sym)?)?;
    let k = eval(&args[1], env, sym)?;
    let key = hashmap_key(":rust::lru::LruCache::get", &k)?;
    let opt = cell.with_mut(":rust::lru::LruCache::get", |c| c.get(&key).cloned())?;
    Ok(Value::Option(Arc::new(opt)))
}

// ─── Schemes ─────────────────────────────────────────────────────────

/// Scheme for `:rust::lru::LruCache::new`:
///   `∀K,V. (capacity: :i64) -> :rust::lru::LruCache<K,V>`
///
/// K,V are fresh type vars; the call-site's let-annotation or the
/// wrapper define's declared return-type constrains them.
fn scheme_new(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != 1 {
        ctx.push_arity_mismatch(":rust::lru::LruCache::new", 1, args.len());
        return Some(TypeExpr::Parametric {
            head: "rust::lru::LruCache".into(),
            args: vec![ctx.fresh_var(), ctx.fresh_var()],
        });
    }
    if let Some(cap_ty) = ctx.infer(&args[0]) {
        if !ctx.unify_types(&cap_ty, &TypeExpr::Path(":i64".into())) {
            ctx.push_type_mismatch(
                ":rust::lru::LruCache::new",
                "capacity",
                "i64".into(),
                format!("{:?}", ctx.apply_subst(&cap_ty)),
            );
        }
    }
    Some(TypeExpr::Parametric {
        head: "rust::lru::LruCache".into(),
        args: vec![ctx.fresh_var(), ctx.fresh_var()],
    })
}

/// Scheme for `:rust::lru::LruCache::put`:
///   `(cache: :rust::lru::LruCache<K,V>, k: K, v: V) -> :()`
fn scheme_put(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != 3 {
        ctx.push_arity_mismatch(":rust::lru::LruCache::put", 3, args.len());
        return Some(TypeExpr::Tuple(vec![]));
    }
    let cache_ty = ctx.infer(&args[0]);
    let k_ty = ctx.infer(&args[1]);
    let v_ty = ctx.infer(&args[2]);
    if let Some(ct) = cache_ty {
        let resolved = ctx.apply_subst(&ct);
        match &resolved {
            TypeExpr::Parametric { head, args: ta }
                if head == "rust::lru::LruCache" && ta.len() == 2 =>
            {
                let k = ctx.apply_subst(&ta[0]);
                let v = ctx.apply_subst(&ta[1]);
                if let Some(kt) = k_ty {
                    if !ctx.unify_types(&kt, &k) {
                        ctx.push_type_mismatch(
                            ":rust::lru::LruCache::put",
                            "key",
                            format!("{:?}", ctx.apply_subst(&k)),
                            format!("{:?}", ctx.apply_subst(&kt)),
                        );
                    }
                }
                if let Some(vt) = v_ty {
                    if !ctx.unify_types(&vt, &v) {
                        ctx.push_type_mismatch(
                            ":rust::lru::LruCache::put",
                            "value",
                            format!("{:?}", ctx.apply_subst(&v)),
                            format!("{:?}", ctx.apply_subst(&vt)),
                        );
                    }
                }
            }
            _ => {
                ctx.push_type_mismatch(
                    ":rust::lru::LruCache::put",
                    "cache",
                    "rust::lru::LruCache<K,V>".into(),
                    format!("{:?}", resolved),
                );
            }
        }
    }
    Some(TypeExpr::Tuple(vec![]))
}

/// Scheme for `:rust::lru::LruCache::get`:
///   `(cache: :rust::lru::LruCache<K,V>, k: K) -> :Option<V>`
fn scheme_get(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != 2 {
        ctx.push_arity_mismatch(":rust::lru::LruCache::get", 2, args.len());
        return Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![ctx.fresh_var()],
        });
    }
    let cache_ty = ctx.infer(&args[0]);
    let k_ty = ctx.infer(&args[1]);
    if let Some(ct) = cache_ty {
        let resolved = ctx.apply_subst(&ct);
        if let TypeExpr::Parametric { head, args: ta } = &resolved {
            if head == "rust::lru::LruCache" && ta.len() == 2 {
                let k = ctx.apply_subst(&ta[0]);
                let v = ctx.apply_subst(&ta[1]);
                if let Some(kt) = k_ty {
                    if !ctx.unify_types(&kt, &k) {
                        ctx.push_type_mismatch(
                            ":rust::lru::LruCache::get",
                            "key",
                            format!("{:?}", ctx.apply_subst(&k)),
                            format!("{:?}", ctx.apply_subst(&kt)),
                        );
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![ctx.apply_subst(&v)],
                });
            }
        }
        ctx.push_type_mismatch(
            ":rust::lru::LruCache::get",
            "cache",
            "rust::lru::LruCache<K,V>".into(),
            format!("{:?}", resolved),
        );
    }
    Some(TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![ctx.fresh_var()],
    })
}

// ─── Registration ────────────────────────────────────────────────────

pub fn register(builder: &mut RustDepsBuilder) {
    builder.register_type(RustTypeDecl {
        path: ":rust::lru::LruCache",
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::lru::LruCache::new",
        dispatch: dispatch_new,
        scheme: scheme_new,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::lru::LruCache::put",
        dispatch: dispatch_put,
        scheme: scheme_put,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::lru::LruCache::get",
        dispatch: dispatch_get,
        scheme: scheme_get,
    });
}
