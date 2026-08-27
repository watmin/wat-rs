//! `:wat::holon::OnlineSubspace/*` intrinsics — arc 255 Stone HOME-8,
//! registry half.
//!
//! `OnlineSubspace` is a CCIPCA (candid incremental PCA) tracker: it learns
//! a low-rank basis for "normal" from a stream of raw `f64` vectors, and
//! scores new vectors against that basis (`residual`, `project`,
//! `reconstruct`). Native `ThreadOwnedCell`-backed handle, same framing as
//! `Hologram` (`hologram.rs`, this home) — `@Category Resource` uniformly.
//!
//! `@Purity`: `new` (mints the handle) and `update` (the only mutator,
//! absorbing one observation via `with_mut`) are `Effectful`; the eight
//! readers (`dim`, `k`, `n`, `threshold`, `eigenvalues`, `residual`,
//! `project`, `reconstruct`) go through `with_ref` and are `Pure`.
//!
//! None of these ten are among the four rete-classified holon verbs
//! (`src/rete/purity.rs:647`) — STOP-7 forbids adding to that builder-ruled
//! set, and this carve does not.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::holon::*;
use crate::runtime::{eval_inner, require_i64};
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

/// `(:wat::holon::OnlineSubspace/new dim k)` -> a fresh `OnlineSubspace`
/// tracking a rank-`k` basis over `dim`-dimensional raw vectors.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the ambient vector dimension and the tracked rank, in order
/// @ret     :wat::holon::OnlineSubspace a fresh, untrained subspace tracker
/// @example-norun (:wat::holon::OnlineSubspace/new 4096 8) #=> #wat.holon/OnlineSubspace{}
#[wat_intrinsic(":wat::holon::OnlineSubspace/new")]
pub(crate) fn eval_subspace_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/new".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let dim = require_i64(
        ":wat::holon::OnlineSubspace/new",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let k = require_i64(
        ":wat::holon::OnlineSubspace/new",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let s = holon::OnlineSubspace::new(dim as usize, k as usize);
    Ok(Value::OnlineSubspace(Arc::new(
        crate::rust_deps::ThreadOwnedCell::new(s),
    )))
}


/// `(:wat::holon::OnlineSubspace/dim s)` -> `:i64`. The raw vector
/// dimension `s` was constructed with.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace, alone
/// @ret     :wat::core::i64 the raw vector dimension
/// @example (:wat::holon::OnlineSubspace/dim (:wat::holon::OnlineSubspace/new 10000 8)) #=> (:wat::holon::OnlineSubspace/dim (:wat::holon::OnlineSubspace/new 10000 8))
#[wat_intrinsic(":wat::holon::OnlineSubspace/dim")]
pub(crate) fn eval_subspace_dim(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/dim".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/dim",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let n = s.with_ref(":wat::holon::OnlineSubspace/dim", |s| s.dim())?;
    Ok(Value::i64(n as i64))
}


/// `(:wat::holon::OnlineSubspace/k s)` -> `:i64`. The tracked rank `s` was
/// constructed with.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace, alone
/// @ret     :wat::core::i64 the tracked rank
/// @example (:wat::holon::OnlineSubspace/k (:wat::holon::OnlineSubspace/new 10000 8)) #=> (:wat::holon::OnlineSubspace/k (:wat::holon::OnlineSubspace/new 10000 8))
#[wat_intrinsic(":wat::holon::OnlineSubspace/k")]
pub(crate) fn eval_subspace_k(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/k".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/k",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let n = s.with_ref(":wat::holon::OnlineSubspace/k", |s| s.k())?;
    Ok(Value::i64(n as i64))
}


/// `(:wat::holon::OnlineSubspace/n s)` -> `:i64`. The number of
/// observations `s` has absorbed so far.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace, alone
/// @ret     :wat::core::i64 the number of observations absorbed so far
/// @example (:wat::holon::OnlineSubspace/n (:wat::holon::OnlineSubspace/new 10000 8)) #=> (:wat::holon::OnlineSubspace/n (:wat::holon::OnlineSubspace/new 10000 8))
#[wat_intrinsic(":wat::holon::OnlineSubspace/n")]
pub(crate) fn eval_subspace_n(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/n".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/n",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let n = s.with_ref(":wat::holon::OnlineSubspace/n", |s| s.n())?;
    Ok(Value::i64(n as i64))
}


/// `(:wat::holon::OnlineSubspace/threshold s)` -> `:f64`. The current
/// anomaly-residual threshold `s` has settled on.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace, alone
/// @ret     :wat::core::f64 the current residual threshold
/// @example (:wat::holon::OnlineSubspace/threshold (:wat::holon::OnlineSubspace/new 10000 8)) #=> (:wat::holon::OnlineSubspace/threshold (:wat::holon::OnlineSubspace/new 10000 8))
#[wat_intrinsic(":wat::holon::OnlineSubspace/threshold")]
pub(crate) fn eval_subspace_threshold(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/threshold".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/threshold",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let t = s.with_ref(":wat::holon::OnlineSubspace/threshold", |s| s.threshold())?;
    Ok(Value::f64(t))
}


/// `(:wat::holon::OnlineSubspace/eigenvalues s)` -> `(:Vector :- [f64])`.
/// The current eigenvalues of `s`'s tracked basis, largest first.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace, alone
/// @ret     (:wat::core::Vector :- [:wat::core::f64]) the tracked basis's current eigenvalues
/// @example (:wat::holon::OnlineSubspace/eigenvalues (:wat::holon::OnlineSubspace/new 10000 8)) #=> (:wat::holon::OnlineSubspace/eigenvalues (:wat::holon::OnlineSubspace/new 10000 8))
#[wat_intrinsic(":wat::holon::OnlineSubspace/eigenvalues")]
pub(crate) fn eval_subspace_eigenvalues(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/eigenvalues".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/eigenvalues",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let xs = s.with_ref(":wat::holon::OnlineSubspace/eigenvalues", |s| {
        s.eigenvalues()
    })?;
    Ok(vec_f64_to_value(xs))
}


/// `(:wat::holon::OnlineSubspace/update s v)` -> `:f64`. Absorbs raw
/// vector `v` as one more observation, mutating `s`'s tracked basis, and
/// returns `v`'s residual against the basis as it stood BEFORE absorbing it.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace and the raw `f64` vector observed, in order
/// @ret     :wat::core::f64 `v`'s pre-update residual
/// @example-norun (:wat::holon::OnlineSubspace/update s v) #=> 0.31
#[wat_intrinsic(":wat::holon::OnlineSubspace/update")]
pub(crate) fn eval_subspace_update(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/update".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/update",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let v = require_vector(
        ":wat::holon::OnlineSubspace/update",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let xs = v.to_f64();
    let residual = s.with_mut(
        ":wat::holon::OnlineSubspace/update",
        list_span.clone(),
        |s| s.update(&xs),
    )?;
    Ok(Value::f64(residual))
}


/// `(:wat::holon::OnlineSubspace/residual s v)` -> `:f64`. `v`'s
/// reconstruction residual against `s`'s current basis, without mutating
/// `s`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace and the raw `f64` vector scored, in order
/// @ret     :wat::core::f64 `v`'s residual against `s`'s current basis
/// @example (:wat::holon::OnlineSubspace/residual (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role"))) #=> (:wat::holon::OnlineSubspace/residual (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role")))
#[wat_intrinsic(":wat::holon::OnlineSubspace/residual")]
pub(crate) fn eval_subspace_residual(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/residual".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/residual",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let v = require_vector(
        ":wat::holon::OnlineSubspace/residual",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let xs = v.to_f64();
    let r = s.with_ref(":wat::holon::OnlineSubspace/residual", |s| s.residual(&xs))?;
    Ok(Value::f64(r))
}


/// `(:wat::holon::OnlineSubspace/project s v)` -> `(:Vector :- [f64])`.
/// `v`'s coordinates in `s`'s current rank-`k` basis.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace and the raw `f64` vector projected, in order
/// @ret     (:wat::core::Vector :- [:wat::core::f64]) `v`'s coordinates in the rank-`k` basis
/// @example (:wat::holon::OnlineSubspace/project (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role"))) #=> (:wat::holon::OnlineSubspace/project (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role")))
#[wat_intrinsic(":wat::holon::OnlineSubspace/project")]
pub(crate) fn eval_subspace_project(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/project".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/project",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let v = require_vector(
        ":wat::holon::OnlineSubspace/project",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let xs = v.to_f64();
    let projected = s.with_ref(":wat::holon::OnlineSubspace/project", |s| s.project(&xs))?;
    Ok(vec_f64_to_value(projected))
}


/// `(:wat::holon::OnlineSubspace/reconstruct s v)` -> `(:Vector :- [f64])`.
/// `v` projected onto `s`'s basis and back — the "normal" approximation of
/// `v` the residual is measured against.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the subspace and the raw `f64` vector reconstructed, in order
/// @ret     (:wat::core::Vector :- [:wat::core::f64]) `v` projected onto the basis and back
/// @example (:wat::holon::OnlineSubspace/reconstruct (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role"))) #=> (:wat::holon::OnlineSubspace/reconstruct (:wat::holon::OnlineSubspace/new 10000 8) (:wat::holon::encode (:wat::holon::leaf "role")))
#[wat_intrinsic(":wat::holon::OnlineSubspace/reconstruct")]
pub(crate) fn eval_subspace_reconstruct(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::OnlineSubspace/reconstruct".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let s = require_subspace(
        ":wat::holon::OnlineSubspace/reconstruct",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let v = require_vector(
        ":wat::holon::OnlineSubspace/reconstruct",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let xs = v.to_f64();
    let r = s.with_ref(":wat::holon::OnlineSubspace/reconstruct", |s| {
        s.reconstruct(&xs)
    })?;
    Ok(vec_f64_to_value(r))
}


