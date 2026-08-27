//! `:wat::holon::Engram/*` and `:wat::holon::EngramLibrary/*` intrinsics —
//! arc 255 Stone HOME-8, registry half.
//!
//! An `Engram` is a learned pattern snapshot (a fitted `OnlineSubspace`
//! frozen as a named reference point); an `EngramLibrary` holds many,
//! indexed for nearest-match lookup (`match-vec`, eigenvalue-prefiltered).
//! Both are native `ThreadOwnedCell`-backed handles, same framing as
//! `Hologram` (`hologram.rs`, this home) and `HandlePool`
//! (`intrinsic/kernel/resource.rs`) — `@Category Resource` uniformly.
//!
//! `@Purity`: readers going through `with_ref` are `Pure`
//! (`Engram/name`, `n`, `eigenvalue-signature`; `EngramLibrary/len`,
//! `contains`, `names`); constructors and anything reaching `with_mut`
//! are `Effectful` — including `Engram/residual` and
//! `EngramLibrary/match-vec`, both of which cache state on read
//! (mirroring `Reckoner/curve` in `reckoner.rs`, this home).
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

/// `(:wat::holon::Engram/name e)` -> `:String`. The engram's name, as
/// given at `EngramLibrary/add` time.
///
/// ⚠ UNREACHABLE FROM WAT TODAY, disclosed rather than hidden: nothing in
/// this crate ever constructs a bare `Value::Engram` (`EngramLibrary/add`
/// freezes one internally but never hands it back; `match-vec` returns
/// `(name, residual)` tuples, not the engram itself). `@Purity Pure` +
/// `@Determinism Deterministic` is still the honest claim about this
/// handler's own body (a `with_ref` read, no side effect); the mandatory
/// runnable `@example` below cannot actually be evaluated by any legal wat
/// program until a future stone adds an accessor producing `Value::Engram`
/// (e.g. an `EngramLibrary/get`). Same shape as the already-disclosed,
/// `#[ignore]`d gap in `probe_arc255_ivb2b_verify_examples.rs` for
/// `type-equal?`/`type-params-used-in` — a documented purpose current
/// syntax cannot reach, not a wrong example.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the engram, alone
/// @ret     :wat::core::String the engram's name
/// @example (:wat::holon::Engram/name e) #=> "anomaly-a"
#[wat_intrinsic(":wat::holon::Engram/name")]
pub(crate) fn eval_engram_name(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::Engram/name".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let e = require_engram(
        ":wat::holon::Engram/name",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let s = e.with_ref(":wat::holon::Engram/name", |e| e.name().to_string())?;
    Ok(Value::String(Arc::new(s)))
}


/// `(:wat::holon::Engram/eigenvalue-signature e)` -> `(:Vector :- [f64])`.
/// The frozen subspace's eigenvalues at snapshot time — the shape used to
/// eigenvalue-prefilter `EngramLibrary/match-vec` candidates.
///
/// ⚠ UNREACHABLE FROM WAT TODAY — see `Engram/name`'s doc for why (no
/// constructor anywhere in this crate ever hands a wat program a bare
/// `Value::Engram`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the engram, alone
/// @ret     (:wat::core::Vector :- [:wat::core::f64]) the engram's frozen eigenvalue signature
/// @example (:wat::holon::Engram/eigenvalue-signature e) #=> (:wat::core::Vector 0.9 0.4)
#[wat_intrinsic(":wat::holon::Engram/eigenvalue-signature")]
pub(crate) fn eval_engram_eigenvalue_signature(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::Engram/eigenvalue-signature".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let e = require_engram(
        ":wat::holon::Engram/eigenvalue-signature",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let xs = e.with_ref(":wat::holon::Engram/eigenvalue-signature", |e| {
        e.eigenvalue_signature().to_vec()
    })?;
    Ok(vec_f64_to_value(xs))
}


/// `(:wat::holon::Engram/n e)` -> `:i64`. The number of observations the
/// underlying subspace had absorbed when `e` was snapshotted.
///
/// ⚠ UNREACHABLE FROM WAT TODAY — see `Engram/name`'s doc for why (no
/// constructor anywhere in this crate ever hands a wat program a bare
/// `Value::Engram`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the engram, alone
/// @ret     :wat::core::i64 the observation count at snapshot time
/// @example (:wat::holon::Engram/n e) #=> 512
#[wat_intrinsic(":wat::holon::Engram/n")]
pub(crate) fn eval_engram_n(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::Engram/n".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let e = require_engram(
        ":wat::holon::Engram/n",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let n = e.with_ref(":wat::holon::Engram/n", |e| e.n())?;
    Ok(Value::i64(n as i64))
}


/// `(:wat::holon::Engram/residual e v)` -> `:f64`. The reconstruction
/// residual of raw vector `v` against `e`'s frozen subspace — small when
/// `v` looks like what `e` was fit on.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the engram and the raw `f64` vector to score, in order
/// @ret     :wat::core::f64 the residual of `v` against `e`'s frozen subspace
/// @example-norun (:wat::holon::Engram/residual e v) #=> 0.03
#[wat_intrinsic(":wat::holon::Engram/residual")]
pub(crate) fn eval_engram_residual(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::Engram/residual".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let e = require_engram(
        ":wat::holon::Engram/residual",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let v = require_vector(
        ":wat::holon::Engram/residual",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let xs = v.to_f64();
    let r = e.with_mut(":wat::holon::Engram/residual", list_span.clone(), |e| {
        e.residual(&xs)
    })?;
    Ok(Value::f64(r))
}


/// `(:wat::holon::EngramLibrary/new dim)` -> a fresh, empty `EngramLibrary`
/// sized to `dim`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library's vector dimension
/// @ret     :wat::holon::EngramLibrary a fresh, empty library
/// @example-norun (:wat::holon::EngramLibrary/new 4096) #=> #wat.holon/EngramLibrary{}
#[wat_intrinsic(":wat::holon::EngramLibrary/new")]
pub(crate) fn eval_library_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/new".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let dim = require_i64(
        ":wat::holon::EngramLibrary/new",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let lib = holon::EngramLibrary::new(dim as usize);
    Ok(Value::EngramLibrary(Arc::new(
        crate::rust_deps::ThreadOwnedCell::new(lib),
    )))
}


/// `(:wat::holon::EngramLibrary/add lib name subspace)` -> `:Unit`. Freezes
/// `subspace` into a new named `Engram` and adds it to `lib`, mutating it
/// in place.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library, the new engram's name, and the subspace to freeze, in order
/// @ret     :wat::core::nil always `Unit`
/// @example-norun (:wat::holon::EngramLibrary/add lib "anomaly-a" subspace) #=> nil
#[wat_intrinsic(":wat::holon::EngramLibrary/add")]
pub(crate) fn eval_library_add(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/add".into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    let lib = require_engram_library(
        ":wat::holon::EngramLibrary/add",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let name = require_string(
        ":wat::holon::EngramLibrary/add",
        eval_inner(&args[1], env, sym)?.value_owned(),
        list_span,
    )?;
    let subspace = require_subspace(
        ":wat::holon::EngramLibrary/add",
        eval_inner(&args[2], env, sym)?.value_owned(),
        list_span,
    )?;
    // EngramLibrary::add takes &OnlineSubspace by reference; we have
    // ThreadOwnedCell. Borrow immutably to get the reference.
    lib.with_mut(":wat::holon::EngramLibrary/add", list_span.clone(), |lib| {
        subspace.with_ref(":wat::holon::EngramLibrary/add", |s| {
            lib.add(&name, s, None, std::collections::HashMap::new());
        })
    })??;
    Ok(Value::Unit)
}


/// `(:wat::holon::EngramLibrary/match-vec lib probe top-k prefilter-k)` ->
/// `(:Vector :- [(:Tuple :- [String f64])])`. The `top-k` closest engrams
/// to raw vector `probe` by reconstruction residual, eigenvalue-prefiltered
/// down to `prefilter-k` candidates first.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library, the probe vector, top-k, and prefilter-k, in order
/// @ret     (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::f64])]) `(name, residual)` tuples for the closest matches, best first
/// @example-norun (:wat::holon::EngramLibrary/match-vec lib probe 3 16) #=> (:wat::core::Vector (:wat::core::Tuple "anomaly-a" 0.02))
#[wat_intrinsic(":wat::holon::EngramLibrary/match-vec")]
pub(crate) fn eval_library_match_vec(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 4 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/match-vec".into(),
                expected: 4,
                got: args.len(),
            },
        )
        .into());
    }
    let lib = require_engram_library(
        ":wat::holon::EngramLibrary/match-vec",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let probe = require_vector(
        ":wat::holon::EngramLibrary/match-vec",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
    let top_k = require_i64(
        ":wat::holon::EngramLibrary/match-vec",
        eval_inner(&args[2], env, sym)?.value_owned(),
    )?;
    let prefilter_k = require_i64(
        ":wat::holon::EngramLibrary/match-vec",
        eval_inner(&args[3], env, sym)?.value_owned(),
    )?;
    let xs = probe.to_f64();
    let matches = lib.with_mut(
        ":wat::holon::EngramLibrary/match-vec",
        list_span.clone(),
        |lib| lib.match_vec(&xs, top_k as usize, prefilter_k as usize),
    )?;
    let elems: Vec<Value> = matches
        .into_iter()
        .map(|(name, residual)| {
            Value::Tuple(Arc::new(vec![
                Value::String(Arc::new(name)),
                Value::f64(residual),
            ]))
        })
        .collect();
    Ok(Value::Vec(Arc::new(elems)))
}


/// `(:wat::holon::EngramLibrary/len lib)` -> `:i64`. The number of engrams
/// currently in `lib`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library, alone
/// @ret     :wat::core::i64 the number of engrams currently held
/// @example (:wat::holon::EngramLibrary/len (:wat::holon::EngramLibrary/new 4096)) #=> (:wat::holon::EngramLibrary/len (:wat::holon::EngramLibrary/new 4096))
#[wat_intrinsic(":wat::holon::EngramLibrary/len")]
pub(crate) fn eval_library_len(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/len".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let lib = require_engram_library(
        ":wat::holon::EngramLibrary/len",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let n = lib.with_ref(":wat::holon::EngramLibrary/len", |lib| lib.len())?;
    Ok(Value::i64(n as i64))
}


/// `(:wat::holon::EngramLibrary/contains lib name)` -> `:bool`. Whether
/// `lib` holds an engram named `name`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library and the name probed, in order
/// @ret     :wat::core::bool true iff `lib` holds an engram named `name`
/// @example (:wat::holon::EngramLibrary/contains (:wat::holon::EngramLibrary/new 4096) "anomaly-a") #=> (:wat::holon::EngramLibrary/contains (:wat::holon::EngramLibrary/new 4096) "anomaly-a")
#[wat_intrinsic(":wat::holon::EngramLibrary/contains")]
pub(crate) fn eval_library_contains(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/contains".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let lib = require_engram_library(
        ":wat::holon::EngramLibrary/contains",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let name = require_string(
        ":wat::holon::EngramLibrary/contains",
        eval_inner(&args[1], env, sym)?.value_owned(),
        list_span,
    )?;
    let b = lib.with_ref(":wat::holon::EngramLibrary/contains", |lib| {
        lib.contains(&name)
    })?;
    Ok(Value::bool(b))
}


/// `(:wat::holon::EngramLibrary/names lib)` -> `(:Vector :- [String])`. The
/// names of every engram `lib` holds.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     args… :wat::core::Value the library, alone
/// @ret     (:wat::core::Vector :- [:wat::core::String]) the names of every engram `lib` holds
/// @example (:wat::holon::EngramLibrary/names (:wat::holon::EngramLibrary/new 4096)) #=> (:wat::holon::EngramLibrary/names (:wat::holon::EngramLibrary/new 4096))
#[wat_intrinsic(":wat::holon::EngramLibrary/names")]
pub(crate) fn eval_library_names(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::holon::EngramLibrary/names".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let lib = require_engram_library(
        ":wat::holon::EngramLibrary/names",
        eval_inner(&args[0], env, sym)?.value_owned(),
        list_span,
    )?;
    let names = lib.with_ref(":wat::holon::EngramLibrary/names", |lib| {
        lib.names()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    })?;
    let elems: Vec<Value> = names
        .into_iter()
        .map(|s| Value::String(Arc::new(s)))
        .collect();
    Ok(Value::Vec(Arc::new(elems)))
}


