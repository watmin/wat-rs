//! `:wat::holon::Reckoner/*` intrinsics — arc 255 Stone HOME-8, registry
//! half.
//!
//! A `Reckoner` accumulates directional evidence from raw `f64` vectors
//! against a discrete label set or a continuous scale, then predicts a
//! direction/conviction and calibrates itself against outcomes
//! (`resolve`). Native `ThreadOwnedCell`-backed handle, same framing as
//! `Hologram`/`OnlineSubspace` (this home) — `@Category Resource`
//! uniformly.
//!
//! `@Purity`: the two constructors, `observe`, and `resolve` mutate via
//! `with_mut` and are `Effectful`; `labels` and `dims` read via `with_ref`
//! and are `Pure`. `predict` and `curve` both go through `with_ref`
//! /`with_mut` respectively in the pre-carve body — `predict` is `Pure`
//! (a `with_ref` read), `curve` is `Effectful` (its `with_mut` caches the
//! fitted calibration curve on first read, same "mutates on read" shape as
//! `Engram/residual`, `engram.rs` this home).
//!
//! None of these eight are among the four rete-classified holon verbs
//! (`src/rete/purity.rs:647`) — STOP-7 forbids adding to that builder-ruled
//! set, and this carve does not.
//!
//! arc 255 Stone H-1a — each handler declares its real fixed arity
//! (`&WatAST` per parameter) instead of `args: &[WatAST]`; the
//! hand-rolled arity checks are gone, replaced by the check
//! `#[wat_intrinsic]` now generates from the declared parameter count. See
//! `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-H-holon-adopts-the-kernels-interface.md`.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::holon::*;
use crate::runtime::{eval_inner, require_i64};
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use holon::HolonAST;

/// `(:wat::holon::Reckoner/new-discrete name dims recalib labels)` -> a
/// fresh `Reckoner` predicting among a discrete `labels` set of HolonAST
/// leaves.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     name :wat::core::String the reckoner's name
/// @arg     dims :wat::core::i64 the raw vector dimension
/// @arg     recalib :wat::core::i64 the recalibration window
/// @arg     labels (:wat::core::Vector :- [:wat::holon::HolonAST]) the label HolonASTs
/// @ret     :wat::holon::Reckoner a fresh, untrained discrete reckoner
/// @example-norun (:wat::holon::Reckoner/new-discrete "direction" 4096 100 labels) #=> #wat.holon/Reckoner{}
#[wat_intrinsic(":wat::holon::Reckoner/new-discrete")]
pub(crate) fn eval_reckoner_new_discrete(
    name: &WatAST,
    dims: &WatAST,
    recalib: &WatAST,
    labels: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error (TypeMismatch) locates at its own arg's span (`name`'s or `labels`'s)
) -> Result<Value, EvalBreak> {
    let name_val = eval_inner(name, env, sym)?.value_owned();
    let name = match name_val {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(
                name.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Reckoner/new-discrete".into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    let dims = require_i64(
        ":wat::holon::Reckoner/new-discrete",
        eval_inner(dims, env, sym)?.value_owned(),
    )?;
    let recalib = require_i64(
        ":wat::holon::Reckoner/new-discrete",
        eval_inner(recalib, env, sym)?.value_owned(),
    )?;
    let labels_val = eval_inner(labels, env, sym)?.value_owned();
    let label_asts: Vec<HolonAST> = match labels_val {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                let h = require_holon(":wat::holon::Reckoner/new-discrete", &item.clone())?;
                out.push((*h).clone());
            }
            out
        }
        other => {
            return Err(RuntimeError::new(
                labels.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Reckoner/new-discrete".into(),
                    expected: "Vec of HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    let r = holon::Reckoner::new(
        &name,
        dims as usize,
        recalib as usize,
        holon::ReckConfig::Discrete(label_asts),
    );
    Ok(Value::Reckoner(Arc::new(
        crate::rust_deps::ThreadOwnedCell::new(r),
    )))
}


/// `(:wat::holon::Reckoner/new-continuous name dims recalib default-value
/// buckets)` -> a fresh `Reckoner` predicting a position on a continuous,
/// bucketed scale.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     name :wat::core::String the reckoner's name
/// @arg     dims :wat::core::i64 the raw vector dimension
/// @arg     recalib :wat::core::i64 the recalibration window
/// @arg     default_value :wat::core::f64 the default scale value
/// @arg     buckets :wat::core::i64 the bucket count
/// @ret     :wat::holon::Reckoner a fresh, untrained continuous reckoner
/// @example-norun (:wat::holon::Reckoner/new-continuous "level" 4096 100 0.0 16) #=> #wat.holon/Reckoner{}
#[wat_intrinsic(":wat::holon::Reckoner/new-continuous")]
// arc 255 Stone H-1a — five real wat args + the env/sym/span tail is 8, one over clippy's 7.
// The arity is the VERB's, not a design choice here: collapsing it back to `args: &[WatAST]`
// is what this stone exists to undo. `expect` rather than `allow` so it goes red if the
// signature ever shrinks under the limit — an exemption that self-audits.
#[expect(clippy::too_many_arguments, reason = "declared arity of a 5-arg verb + the env/sym/span tail")]
pub(crate) fn eval_reckoner_new_continuous(
    name: &WatAST,
    dims: &WatAST,
    recalib: &WatAST,
    default_value: &WatAST,
    buckets: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let name_val = eval_inner(name, env, sym)?.value_owned();
    let name = match name_val {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(
                name.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Reckoner/new-continuous".into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    let dims = require_i64(
        ":wat::holon::Reckoner/new-continuous",
        eval_inner(dims, env, sym)?.value_owned(),
    )?;
    let recalib = require_i64(
        ":wat::holon::Reckoner/new-continuous",
        eval_inner(recalib, env, sym)?.value_owned(),
    )?;
    let default_value = require_numeric(
        ":wat::holon::Reckoner/new-continuous",
        &eval_inner(default_value, env, sym)?.value_owned(),
        list_span,
    )?;
    let buckets = require_i64(
        ":wat::holon::Reckoner/new-continuous",
        eval_inner(buckets, env, sym)?.value_owned(),
    )?;
    let r = holon::Reckoner::new(
        &name,
        dims as usize,
        recalib as usize,
        holon::ReckConfig::Continuous {
            default_value,
            buckets: buckets as usize,
        },
    );
    Ok(Value::Reckoner(Arc::new(
        crate::rust_deps::ThreadOwnedCell::new(r),
    )))
}


/// `(:wat::holon::Reckoner/observe r v label-idx weight)` -> `:Unit`.
/// Absorbs one weighted observation of raw vector `v` toward label index
/// `label-idx`, mutating `r`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner mutated
/// @arg     v :wat::holon::Vector the observed vector
/// @arg     label_idx :wat::core::i64 the label index observed toward
/// @arg     weight :wat::core::f64 the observation's weight
/// @ret     :wat::core::nil always `Unit`
/// @example-norun (:wat::holon::Reckoner/observe r v 0 1.0) #=> nil
#[wat_intrinsic(":wat::holon::Reckoner/observe")]
pub(crate) fn reckoner_observe(
    r: &Value,
    v: &Value,
    label_idx: &Value,
    weight: &Value,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let r = require_reckoner(":wat::holon::Reckoner/observe", r, span)?;
    let v = require_vector(":wat::holon::Reckoner/observe", v)?;
    let label_idx = require_i64(":wat::holon::Reckoner/observe", label_idx.clone())?;
    let weight = require_numeric(":wat::holon::Reckoner/observe", weight, span)?;
    r.with_mut(":wat::holon::Reckoner/observe", span.clone(), |r| {
        r.observe(&v, holon::Label::from_index(label_idx as usize), weight)
    })?;
    Ok(Value::Unit)
}


/// `(:wat::holon::Reckoner/predict r v)` -> `(:Tuple :- [Vector (:Option
/// :- [i64]) f64 f64])`. Scores raw vector `v` against every label `r`
/// tracks, returning per-label `(index, cosine)` scores, the winning
/// direction (if any), a conviction, and the raw winning cosine.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner probed
/// @arg     v :wat::holon::Vector the vector to score
/// @ret     (:wat::core::Tuple :- [(:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::f64])]) (:wat::core::Option :- [:wat::core::i64]) :wat::core::f64 :wat::core::f64]) `(scores, direction, conviction, raw-cos)`
/// @example (:wat::holon::Reckoner/predict (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down"))) (:wat::holon::encode (:wat::holon::leaf "role"))) #=> (:wat::holon::Reckoner/predict (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down"))) (:wat::holon::encode (:wat::holon::leaf "role")))
#[wat_intrinsic(":wat::holon::Reckoner/predict")]
pub(crate) fn reckoner_predict(r: &Value, v: &Value, span: &Span) -> Result<Value, EvalBreak> {
    let r = require_reckoner(":wat::holon::Reckoner/predict", r, span)?;
    let v = require_vector(":wat::holon::Reckoner/predict", v)?;
    let pred = r.with_ref(":wat::holon::Reckoner/predict", |r| r.predict(&v))?;
    // Pack scores as Vec<(i64, f64)> tuples.
    let scores: Vec<Value> = pred
        .scores
        .into_iter()
        .map(|ls| {
            Value::Tuple(Arc::new(vec![
                Value::i64(ls.label.index() as i64),
                Value::f64(ls.cosine),
            ]))
        })
        .collect();
    let scores_value = Value::Vec(Arc::new(scores));
    let direction = match pred.direction {
        Some(label) => Value::Option(Arc::new(Some(Value::i64(label.index() as i64)))),
        None => Value::Option(Arc::new(None)),
    };
    let conviction = Value::f64(pred.conviction);
    let raw_cos = Value::f64(pred.raw_cos);
    Ok(Value::Tuple(Arc::new(vec![
        scores_value,
        direction,
        conviction,
        raw_cos,
    ])))
}


/// `(:wat::holon::Reckoner/resolve r conviction correct)` -> `:Unit`.
/// Feeds back whether a prediction made at `conviction` was `correct`,
/// mutating `r`'s calibration state.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner mutated
/// @arg     conviction :wat::core::f64 the prediction's conviction
/// @arg     correct :wat::core::bool whether the prediction was correct
/// @ret     :wat::core::nil always `Unit`
/// @example-norun (:wat::holon::Reckoner/resolve r 0.8 true) #=> nil
#[wat_intrinsic(":wat::holon::Reckoner/resolve")]
pub(crate) fn eval_reckoner_resolve(
    r: &WatAST,
    conviction: &WatAST,
    correct: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let r = require_reckoner(
        ":wat::holon::Reckoner/resolve",
        &eval_inner(r, env, sym)?.value_owned(),
        list_span,
    )?;
    let conviction = require_numeric(
        ":wat::holon::Reckoner/resolve",
        &eval_inner(conviction, env, sym)?.value_owned(),
        list_span,
    )?;
    let correct_val = eval_inner(correct, env, sym)?.value_owned();
    let correct = match correct_val {
        Value::bool(b) => b,
        other => {
            return Err(RuntimeError::new(
                correct.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::holon::Reckoner/resolve".into(),
                    expected: "bool",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into())
        }
    };
    r.with_mut(":wat::holon::Reckoner/resolve", list_span.clone(), |r| {
        r.resolve(conviction, correct)
    })?;
    Ok(Value::Unit)
}


/// `(:wat::holon::Reckoner/curve r)` -> `(:Option :- [(:Tuple :- [f64
/// f64])])`. The fitted `(slope, intercept)` calibration curve mapping raw
/// cosine to conviction, once enough `resolve` feedback has accumulated
/// (`None` before then).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner probed
/// @ret     (:wat::core::Option :- [(:wat::core::Tuple :- [:wat::core::f64 :wat::core::f64])]) the fitted `(slope, intercept)` curve, or `None`
/// @example-norun (:wat::holon::Reckoner/curve r) #=> (:wat::core::Option (:wat::core::Tuple 1.2 0.1))
#[wat_intrinsic(":wat::holon::Reckoner/curve")]
pub(crate) fn reckoner_curve(r: &Value, span: &Span) -> Result<Value, EvalBreak> {
    let r = require_reckoner(":wat::holon::Reckoner/curve", r, span)?;
    let curve = r.with_mut(":wat::holon::Reckoner/curve", span.clone(), |r| {
        r.curve()
    })?;
    Ok(match curve {
        Some((a, b)) => Value::Option(Arc::new(Some(Value::Tuple(Arc::new(vec![
            Value::f64(a),
            Value::f64(b),
        ]))))),
        None => Value::Option(Arc::new(None)),
    })
}


/// `(:wat::holon::Reckoner/labels r)` -> `(:Vector :- [i64])`. The label
/// indices `r` tracks.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner probed
/// @ret     (:wat::core::Vector :- [:wat::core::i64]) the label indices `r` tracks
/// @example (:wat::holon::Reckoner/labels (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down")))) #=> (:wat::holon::Reckoner/labels (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down"))))
#[wat_intrinsic(":wat::holon::Reckoner/labels")]
pub(crate) fn reckoner_labels(r: &Value, span: &Span) -> Result<Value, EvalBreak> {
    let r = require_reckoner(":wat::holon::Reckoner/labels", r, span)?;
    let labels = r.with_ref(":wat::holon::Reckoner/labels", |r| r.labels())?;
    let xs: Vec<Value> = labels
        .into_iter()
        .map(|l| Value::i64(l.index() as i64))
        .collect();
    Ok(Value::Vec(Arc::new(xs)))
}


/// `(:wat::holon::Reckoner/dims r)` -> `:i64`. The raw vector dimension
/// `r` was constructed with.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     r :wat::holon::Reckoner the reckoner probed
/// @ret     :wat::core::i64 the raw vector dimension
/// @example (:wat::holon::Reckoner/dims (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down")))) #=> (:wat::holon::Reckoner/dims (:wat::holon::Reckoner/new-discrete "direction" 10000 100 (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::leaf "up") (:wat::holon::leaf "down"))))
#[wat_intrinsic(":wat::holon::Reckoner/dims")]
pub(crate) fn reckoner_dims(r: &Value, span: &Span) -> Result<Value, EvalBreak> {
    let r = require_reckoner(":wat::holon::Reckoner/dims", r, span)?;
    let n = r.with_ref(":wat::holon::Reckoner/dims", |r| r.dims())?;
    Ok(Value::i64(n as i64))
}

