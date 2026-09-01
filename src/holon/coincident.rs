//! The `presence?`/`coincident?` predicate family: value-in measurement
//! (`presence_q_from_values`, `coincident_q_from_values`) and the
//! `eval-*-coincident?` embedded-program family (`run_ast_arg_for_eval_coincident`,
//! `coincident_of_two_values`, `eval_form_digest_coincident_shared`,
//! `eval_form_signed_coincident_shared`), which verifies an embedded program's
//! source (by digest or signature) before running it and reducing both sides
//! to the same coincidence measurement. Functions lifted out of `runtime.rs`
//! — see `src/holon/mod.rs` for the doctrine.

// `eval_inner`/`expect_string_value`/`parse_and_run`/`program_dim`/
// `require_encoding_ctx`/`wrap_as_eval_result` and the eval-verification
// helpers below are genuinely defined in `crate::runtime` (not facade
// re-exports of `crate::value` types — see STOP-2): `eval_inner` is the
// evaluator's own entry point; the rest are ambient program config and the
// `eval-file!`/`eval-digest!`/`eval-signed!` family's shared machinery,
// bumped to `pub(crate)` here (a visibility change forced by this module
// boundary, not a signature change) because this file is now a second
// caller outside `runtime.rs`.
use crate::runtime::{
    eval_inner, expect_string_value, parse_and_run, parse_program, parse_verify_algo_keyword,
    program_dim, read_source_via_loader, require_encoding_ctx, resolve_verify_payload,
    run_constrained, run_program, wrap_as_eval_result,
};

// `require_holon`/`to_holon_inner` are `crate::holon::ast::{require_holon, to_holon_inner}`,
// re-exported at `crate::holon` (the `ast` submodule itself is private) —
// the canonical path, not a facade.
use crate::holon::{require_holon, to_holon_inner};

// `PairedVectors`/`pair_values_to_vectors` live in this home's sibling
// `outcome.rs`, re-exported at `crate::holon` the same way.
use crate::holon::{pair_values_to_vectors, PairedVectors};

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use holon::{encode, Similarity};

/// Value-in `presence?`. Shared by AST eval and native `apply_op`.
pub(crate) fn presence_q_from_values(
    target: Value,
    reference: Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let target = require_holon(":wat::holon::presence?", &target)?;
    let reference = require_holon(":wat::holon::presence?", &reference)?;
    let ctx = require_encoding_ctx(":wat::holon::presence?", sym, list_span)?;

    // Arc 037 slice 3: normalize UP via ambient router. Presence
    // sigma is computed at the ACTUAL encoding d via arc 024's
    // formula `sqrt(d)/2 - 1` — it adapts by design (Ch 28
    // slack-lemma). Using config.presence_sigma directly would
    // over-threshold at smaller d (sigma was calibrated at
    // config.dims).
    let d = program_dim(":wat::holon::presence?", sym, list_span)?;
    let enc = ctx.encoders.get(d);
    let vt = encode(&target, &enc.vm, &enc.scalar);
    let vr = encode(&reference, &enc.vm, &enc.scalar);
    let cosine = Similarity::cosine(&vt, &vr);
    Ok(Value::bool(cosine > enc.presence_floor(sym)))
}

/// Value-in `coincident?`. Shared by AST eval and native `apply_op`.
pub(crate) fn coincident_q_from_values(
    a: Value,
    b: Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::coincident?";
    // Arc 061 — polymorphic over (HolonAST, Vector) pairs in any
    // combination, mirroring arc 052's `cosine` shape. Pre-encoded
    // Vector inputs short-circuit the encoding step; mixed inputs
    // promote the AST side at the Vector side's d. Coincident
    // sigma stays at 1 (the 1σ native-granularity floor — Ch 28),
    // applied at the actual encoding d.
    let (va, vb) = match pair_values_to_vectors(OP, a, b, sym, list_span)? {
        // Arc 278 the cosine outcome wall — `coincident?` is a PREDICATE
        // (THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT): an undefined
        // comparison is not below the floor, so the honest answer to the
        // question actually asked ("are these the same point?") is `false`,
        // by documented total contract — never a raise.
        PairedVectors::DimensionMismatch { .. } => return Ok(Value::bool(false)),
        PairedVectors::Paired(va, vb) => (va, vb),
    };
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let enc = ctx.encoders.get(va.dimensions());
    let cosine = Similarity::cosine(&va, &vb);
    Ok(Value::bool((1.0 - cosine) < enc.coincident_floor(sym)))
}

/// Per-side helper for `eval-coincident?`: eval the arg to a
/// `Value::wat__WatAST`, then run that AST under the constrained
/// discipline (mutation forms refused) and return the inner Value.
/// Shared across the four eval-coincident-family variants for the
/// AST side of each (the AST variant's ENTIRE side; the edn/digest/
/// signed variants use different resolvers to obtain the AST).
pub(crate) fn run_ast_arg_for_eval_coincident(
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    op: &'static str,
) -> Result<Value, EvalBreak> {
    let ast = match eval_inner(arg, env, sym)?.value_owned() {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::new(
                arg.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "Ast",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    run_constrained(&ast, env, sym)
}

/// Shared finalizer: lift both sides via `to_holon_inner`, encode
/// both HolonASTs, cosine, compare against `coincident_floor`. Returns
/// `Value::bool`. Used by all four eval-coincident-family variants —
/// the per-variant resolver produces the two Values via its own
/// verification discipline, then hands them here for the coincidence
/// measurement.
pub(crate) fn coincident_of_two_values(
    value_a: Value,
    value_b: Value,
    sym: &SymbolTable,
    op: &'static str,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    // arc 138: no per-arg AST span — values produced by evaluation; fall
    // back to list_span (the call site), which is real user source.
    let atom_a = to_holon_inner(value_a, list_span)?;
    let atom_b = to_holon_inner(value_b, list_span)?;
    let holon_a = require_holon(op, &atom_a)?;
    let holon_b = require_holon(op, &atom_b)?;
    let ctx = require_encoding_ctx(op, sym, list_span)?;
    // Arc 037 slice 3: normalize UP via ambient router. Coincident
    // floor at actual encoding d.
    let d = program_dim(op, sym, list_span)?;
    let enc = ctx.encoders.get(d);
    let va = encode(&holon_a, &enc.vm, &enc.scalar);
    let vb = encode(&holon_b, &enc.vm, &enc.scalar);
    let cosine = Similarity::cosine(&va, &vb);
    Ok(Value::bool((1.0 - cosine) < enc.coincident_floor(sym)))
}

pub(crate) fn eval_form_digest_coincident_shared(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    is_string: bool,
) -> Result<Value, EvalBreak> {
    let op: &'static str = if is_string {
        ":wat::holon::eval-digest-string-coincident?"
    } else {
        ":wat::holon::eval-digest-coincident?"
    };
    if args.len() != 8 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "({} <4-arg side A> <4-arg side B>) takes exactly 8 arguments; got {}",
                    op,
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        // Side A — 4-arg block [0..4).
        let src_a = if is_string {
            expect_string_value(op, &args[0], env, sym)?
        } else {
            read_source_via_loader(op, &args[0], env, sym)?
        };
        let algo_a = parse_verify_algo_keyword(&args[1], "digest-", op)?;
        let hex_a = resolve_verify_payload(&args[2], &args[3], env, sym)?;
        crate::hash::verify_source_hash(src_a.as_bytes(), &algo_a, hex_a.trim()).map_err(
            |err| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalVerificationFailed { err },
                )
            },
        )?;
        let value_a = parse_and_run(&src_a, env, sym)?;

        // Side B — 4-arg block [4..8).
        let src_b = if is_string {
            expect_string_value(op, &args[4], env, sym)?
        } else {
            read_source_via_loader(op, &args[4], env, sym)?
        };
        let algo_b = parse_verify_algo_keyword(&args[5], "digest-", op)?;
        let hex_b = resolve_verify_payload(&args[6], &args[7], env, sym)?;
        crate::hash::verify_source_hash(src_b.as_bytes(), &algo_b, hex_b.trim()).map_err(
            |err| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalVerificationFailed { err },
                )
            },
        )?;
        let value_b = parse_and_run(&src_b, env, sym)?;

        coincident_of_two_values(value_a, value_b, sym, op, list_span)
    })())
}

pub(crate) fn eval_form_signed_coincident_shared(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
    is_string: bool,
) -> Result<Value, EvalBreak> {
    let op: &'static str = if is_string {
        ":wat::holon::eval-signed-string-coincident?"
    } else {
        ":wat::holon::eval-signed-coincident?"
    };
    if args.len() != 12 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "({} <6-arg side A> <6-arg side B>) takes exactly 12 arguments; got {}",
                    op,
                    args.len()
                ),
            },
        )
        .into());
    }
    wrap_as_eval_result((|| -> Result<Value, EvalBreak> {
        // Side A — 6-arg block [0..6).
        let src_a = if is_string {
            expect_string_value(op, &args[0], env, sym)?
        } else {
            read_source_via_loader(op, &args[0], env, sym)?
        };
        let algo_a = parse_verify_algo_keyword(&args[1], "signed-", op)?;
        let sig_a = resolve_verify_payload(&args[2], &args[3], env, sym)?;
        let pk_a = resolve_verify_payload(&args[4], &args[5], env, sym)?;
        let ast_a = parse_program(&src_a, op)?;
        crate::hash::verify_program_signature(&ast_a, &algo_a, sig_a.trim(), pk_a.trim()).map_err(
            |err| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalVerificationFailed { err },
                )
            },
        )?;
        let value_a = run_program(&ast_a, env, sym)?;

        // Side B — 6-arg block [6..12).
        let src_b = if is_string {
            expect_string_value(op, &args[6], env, sym)?
        } else {
            read_source_via_loader(op, &args[6], env, sym)?
        };
        let algo_b = parse_verify_algo_keyword(&args[7], "signed-", op)?;
        let sig_b = resolve_verify_payload(&args[8], &args[9], env, sym)?;
        let pk_b = resolve_verify_payload(&args[10], &args[11], env, sym)?;
        let ast_b = parse_program(&src_b, op)?;
        crate::hash::verify_program_signature(&ast_b, &algo_b, sig_b.trim(), pk_b.trim()).map_err(
            |err| {
                RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::EvalVerificationFailed { err },
                )
            },
        )?;
        let value_b = run_program(&ast_b, env, sym)?;

        coincident_of_two_values(value_a, value_b, sym, op, list_span)
    })())
}
