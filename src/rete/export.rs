//! `#wat.rete/Export` — the compiled program as one EDN value.
//!
//! Not a Session. No facts, no memories, no source forms. Native fire only.
//! One tag; interior is packed vectors (kind + integers + literals).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::WatAST;
use crate::rete::alpha_tree::AlphaTree;
use crate::rete::compiled_cond::{CompiledCond, Op};
use crate::rete::compiled_rhs::{CompiledRhs, CompiledRhsByRule, RhsOp};
use crate::rete::expr_ir::{Expr, Pat, Program};
use crate::rete::kernel::{
    alpha_cond_from_node, class_field_names, get_node, kind_of, network_identity, node_children,
    node_named_field, node_named_i64, node_named_string, invert_feeding_alpha,
    kind_id_lists, rete_arm_get_or_build, rete_arm_intern,
    agg_named_field, session_named_field, session_network, session_names, sorted_node_ids, AccFold, AlphasByType,
    CondDriver, InternedNetwork, NodeKind, RuleDep,
};
use crate::rete::clause::CmpKind;
use crate::rete::matcher::alpha_pattern;
use crate::rete::vocabulary::RETE_OPS;
use crate::runtime::{
    EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use crate::types::Nature;
use crate::value::pmap::PMap;
use crate::value::value::AggregateValue;

const OP: &str = ":wat::rete::export";
const IMPORT_OP: &str = ":wat::rete::import";
const FORMAT_V: i64 = 1;

::wat_source_derive::wat_field_names_from!(EXPORT_FIELDS, "wat/rete.wat", ":wat::rete::Export");
::wat_source_derive::wat_field_names_from!(ALPHA_FIELDS, "wat/rete.wat", ":wat::rete::AlphaNode");
::wat_source_derive::wat_field_names_from!(
    ROOT_FIELDS,
    "wat/rete.wat",
    ":wat::rete::RootJoinNode"
);
::wat_source_derive::wat_field_names_from!(
    HASH_FIELDS,
    "wat/rete.wat",
    ":wat::rete::HashJoinNode"
);
::wat_source_derive::wat_field_names_from!(
    PROD_FIELDS,
    "wat/rete.wat",
    ":wat::rete::ProductionNode"
);
::wat_source_derive::wat_field_names_from!(TEST_FIELDS, "wat/rete.wat", ":wat::rete::TestNode");
::wat_source_derive::wat_field_names_from!(
    NEG_FIELDS,
    "wat/rete.wat",
    ":wat::rete::NegationNode"
);
::wat_source_derive::wat_field_names_from!(
    EXISTS_FIELDS,
    "wat/rete.wat",
    ":wat::rete::ExistsNode"
);
::wat_source_derive::wat_field_names_from!(
    ACC_FIELDS,
    "wat/rete.wat",
    ":wat::rete::AccumulateNode"
);
::wat_source_derive::wat_field_names_from!(QUERY_FIELDS, "wat/rete.wat", ":wat::rete::QueryNode");

fn names(fields: &'static [&'static str]) -> crate::rete::kernel::FieldNames {
    crate::value::value::names_arc_from_static(fields)
}

fn export_names() -> crate::rete::kernel::FieldNames {
    static N: OnceLock<crate::rete::kernel::FieldNames> = OnceLock::new();
    N.get_or_init(|| names(EXPORT_FIELDS)).clone()
}

fn kw(name: &str) -> Value {
    Value::wat__core__keyword(Arc::new(name.to_string()))
}

fn pv(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Vec(Arc::new(items.into_iter().collect()))
}

fn empty_pv() -> Value {
    Value::wat__core__PersistentVector(crate::value::pvec::PVec::new())
}

fn empty_pm() -> Value {
    Value::wat__core__PersistentMap(PMap::new())
}

fn dummy_ast(span: &Span) -> Value {
    Value::wat__WatAST(Arc::new(WatAST::List(Vec::new(), span.clone())))
}

fn malformed(span: &Span, op: &str, reason: impl Into<String>) -> EvalBreak {
    RuntimeError::new(
        span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: reason.into(),
        },
    )
    .into()
}

fn expect_at<'a>(items: &'a [Value], i: usize, span: &Span, what: &str) -> Result<&'a Value, EvalBreak> {
    items
        .get(i)
        .ok_or_else(|| malformed(span, IMPORT_OP, format!("{what} missing slot {i}")))
}

fn export_named<'a>(export: &'a Value, name: &'static str, span: &Span) -> Result<&'a Value, EvalBreak> {
    agg_named_field(export, name)
        .ok_or_else(|| malformed(span, IMPORT_OP, format!("Export missing field `{name}`")))
}

fn expect_kw<'a>(v: &'a Value, op: &str, span: &Span) -> Result<&'a str, EvalBreak> {
    match v {
        Value::wat__core__keyword(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "keyword tag",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_i64(v: &Value, op: &str, span: &Span) -> Result<i64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(*n),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Slot / frame / param: refuse wrap-into-range (`n as u16`).
fn expect_u16(v: &Value, span: &Span, what: &str) -> Result<u16, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    u16::try_from(n).map_err(|_| {
        malformed(span, IMPORT_OP, format!("{what} {n} does not fit u16"))
    })
}

/// Opcode: refuse wrap-into-range AND refuse `>= RETE_OPS.len()` (apply_op would
/// either dispatch the wrong OpExec or MalformedForm after a wrap).
fn expect_op(v: &Value, span: &Span) -> Result<u16, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    if n < 0 || (n as u64) >= RETE_OPS.len() as u64 {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("op index {n} is outside RETE_OPS"),
        ));
    }
    Ok(n as u16)
}

/// Compiled-cond slot / n_slots / field_idx: non-negative and ≤ u16::MAX.
fn expect_idx(v: &Value, span: &Span, what: &str) -> Result<usize, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    if n < 0 || n > i64::from(u16::MAX) {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("{what} {n} does not fit a slot index"),
        ));
    }
    Ok(n as usize)
}

fn check_slot(slot: u16, frame_len: u16, span: &Span, what: &str) -> Result<(), EvalBreak> {
    if slot >= frame_len {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("{what} slot {slot} >= frame_len {frame_len}"),
        ));
    }
    Ok(())
}

fn check_pat_slots(pat: &Pat, frame_len: u16, span: &Span) -> Result<(), EvalBreak> {
    match pat {
        Pat::Lit(_) | Pat::Wild => Ok(()),
        Pat::Bind(s) => check_slot(*s, frame_len, span, "pbind"),
        Pat::Variant { payload, .. } => match payload {
            Some(inner) => check_pat_slots(inner, frame_len, span),
            None => Ok(()),
        },
    }
}

fn check_expr_slots(e: &Expr, frame_len: u16, span: &Span) -> Result<(), EvalBreak> {
    match e {
        Expr::Lit(_) => Ok(()),
        Expr::Slot(s) => check_slot(*s, frame_len, span, "expr"),
        Expr::Call { args, .. } | Expr::And(args) | Expr::Or(args) => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            Ok(())
        }
        Expr::CallFallback {
            args, fallback, ..
        } => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            check_expr_slots(fallback, frame_len, span)
        }
        Expr::CallUser { args, .. } => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            Ok(())
        }
        Expr::Field { recv, .. } => check_expr_slots(recv, frame_len, span),
        Expr::Construct { fields, .. } | Expr::Variant { fields, .. } => {
            for f in fields.iter() {
                check_expr_slots(f, frame_len, span)?;
            }
            Ok(())
        }
        Expr::If {
            cond,
            then_,
            else_,
        } => {
            check_expr_slots(cond, frame_len, span)?;
            check_expr_slots(then_, frame_len, span)?;
            check_expr_slots(else_, frame_len, span)
        }
        Expr::Let { binds, body } => {
            for (s, e) in binds.iter() {
                check_slot(*s, frame_len, span, "let")?;
                check_expr_slots(e, frame_len, span)?;
            }
            check_expr_slots(body, frame_len, span)
        }
        Expr::Match { scrutinee, arms } => {
            check_expr_slots(scrutinee, frame_len, span)?;
            for (pat, body) in arms.iter() {
                check_pat_slots(pat, frame_len, span)?;
                check_expr_slots(body, frame_len, span)?;
            }
            Ok(())
        }
    }
}

fn check_program_slots(p: &Program, span: &Span) -> Result<(), EvalBreak> {
    for s in p.params.iter() {
        check_slot(*s, p.frame_len, span, "param")?;
    }
    for (_, s) in p.reads.iter() {
        check_slot(*s, p.frame_len, span, "read")?;
    }
    check_expr_slots(&p.root, p.frame_len, span)
}

fn check_cond_ops(ops: &[Op], n_slots: usize, span: &Span) -> Result<(), EvalBreak> {
    let frame_len = n_slots as u16;
    for op in ops {
        match op {
            Op::Bind { slot, .. } | Op::BindCheck { slot, .. } => {
                if *slot >= n_slots {
                    return Err(malformed(
                        span,
                        IMPORT_OP,
                        format!("cond slot {slot} >= n_slots {n_slots}"),
                    ));
                }
            }
            Op::Cmp { lhs, rhs, .. } | Op::SeedCmp { lhs, rhs, .. } => {
                check_expr_slots(lhs, frame_len, span)?;
                check_expr_slots(rhs, frame_len, span)?;
            }
            Op::Or(branches) => {
                for b in branches.iter() {
                    check_cond_ops(b, n_slots, span)?;
                }
            }
            Op::Not(inner) => check_cond_ops(inner, n_slots, span)?,
            Op::Fail => {}
        }
    }
    Ok(())
}

fn expect_str<'a>(v: &'a Value, op: &str, span: &Span) -> Result<&'a str, EvalBreak> {
    match v {
        Value::String(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "string",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_seq(v: &Value, op: &str, span: &Span) -> Result<Vec<Value>, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok((**xs).clone()),
        Value::wat__core__PersistentVector(pv) => Ok(pv.iter().cloned().collect()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "vector",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn record(class: &str, field_names: &'static [&'static str], fields: Vec<Value>) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        class.into(),
        names(field_names),
        Arc::new(fields),
    )))
}

// ── ABI ──────────────────────────────────────────────────────────────────────

fn fnv1a(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn abi_of(classes: &[String], fields: &[Vec<String>]) -> String {
    let mut s = format!("v{FORMAT_V}");
    for (c, fs) in classes.iter().zip(fields.iter()) {
        s.push(';');
        s.push_str(c);
        s.push('[');
        s.push_str(&fs.join(","));
        s.push(']');
    }
    s.push_str(";ops:");
    for (i, op) in RETE_OPS.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(op.rete_name);
    }
    format!("v{FORMAT_V}:{:016x}", fnv1a(&s))
}

// ── Expr / Program ───────────────────────────────────────────────────────────

fn pack_cmp(k: CmpKind) -> Value {
    kw(match k {
        CmpKind::Eq => ":eq",
        CmpKind::NotEq => ":neq",
        CmpKind::Lt => ":lt",
        CmpKind::Gt => ":gt",
        CmpKind::Le => ":le",
        CmpKind::Ge => ":ge",
    })
}

fn unpack_cmp(v: &Value, span: &Span) -> Result<CmpKind, EvalBreak> {
    match expect_kw(v, IMPORT_OP, span)? {
        ":eq" => Ok(CmpKind::Eq),
        ":neq" => Ok(CmpKind::NotEq),
        ":lt" => Ok(CmpKind::Lt),
        ":gt" => Ok(CmpKind::Gt),
        ":le" => Ok(CmpKind::Le),
        ":ge" => Ok(CmpKind::Ge),
        other => Err(malformed(span, IMPORT_OP, format!("unknown cmp {other}"))),
    }
}

fn pack_pat(p: &Pat) -> Value {
    match p {
        Pat::Lit(v) => pv([kw(":plit"), v.clone()]),
        Pat::Wild => pv([kw(":wild")]),
        Pat::Bind(s) => pv([kw(":pbind"), Value::i64(*s as i64)]),
        Pat::Variant { name, payload } => {
            let mut xs = vec![kw(":pvar"), Value::String(Arc::new(name.clone()))];
            if let Some(inner) = payload {
                xs.push(pack_pat(inner));
            }
            pv(xs)
        }
    }
}

fn unpack_pat(v: &Value, span: &Span) -> Result<Pat, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = items.first().ok_or_else(|| malformed(span, IMPORT_OP, "empty pat"))?;
    match expect_kw(tag, IMPORT_OP, span)? {
        ":plit" => {
            let lit = items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "plit missing value"))?
                .clone();
            Ok(Pat::Lit(lit))
        }
        ":wild" => Ok(Pat::Wild),
        ":pbind" => {
            let n = expect_u16(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "pbind missing slot"))?,
                span,
                "pbind",
            )?;
            Ok(Pat::Bind(n))
        }
        ":pvar" => {
            let name = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "pvar missing name"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let payload = match items.get(2) {
                Some(inner) => Some(Box::new(unpack_pat(inner, span)?)),
                None => None,
            };
            Ok(Pat::Variant { name, payload })
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown pat {other}"))),
    }
}

fn pack_expr(e: &Expr) -> Value {
    match e {
        Expr::Lit(v) => pv([kw(":lit"), v.clone()]),
        Expr::Slot(s) => pv([kw(":slot"), Value::i64(*s as i64)]),
        Expr::Call { op, args } => {
            let mut xs = vec![kw(":call"), Value::i64(*op as i64)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::CallFallback { op, args, fallback } => {
            let mut xs = vec![kw(":call-fb"), Value::i64(*op as i64), pack_expr(fallback)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::CallUser { program, args } => {
            let mut xs = vec![kw(":user"), pack_prog(program)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::Field { recv, idx } => {
            pv([kw(":field"), pack_expr(recv), Value::i64(*idx as i64)])
        }
        Expr::Construct {
            class,
            names,
            fields,
        } => {
            let mut xs = vec![
                kw(":ctor"),
                Value::String(Arc::new(class.clone())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(fields.iter().map(pack_expr));
            pv(xs)
        }
        Expr::Variant {
            type_path,
            variant_name,
            names,
            fields,
        } => {
            let mut xs = vec![
                kw(":variant"),
                Value::String(Arc::new(type_path.clone())),
                Value::String(Arc::new(variant_name.clone())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(fields.iter().map(pack_expr));
            pv(xs)
        }
        Expr::If { cond, then_, else_ } => {
            pv([kw(":if"), pack_expr(cond), pack_expr(then_), pack_expr(else_)])
        }
        Expr::And(xs) => {
            let mut out = vec![kw(":and")];
            out.extend(xs.iter().map(pack_expr));
            pv(out)
        }
        Expr::Or(xs) => {
            let mut out = vec![kw(":or")];
            out.extend(xs.iter().map(pack_expr));
            pv(out)
        }
        Expr::Let { binds, body } => {
            let pairs = binds.iter().map(|(s, e)| {
                pv([Value::i64(*s as i64), pack_expr(e)])
            });
            pv([kw(":let"), pv(pairs), pack_expr(body)])
        }
        Expr::Match { scrutinee, arms } => {
            let packed_arms = arms.iter().map(|(p, e)| pv([pack_pat(p), pack_expr(e)]));
            pv([kw(":match"), pack_expr(scrutinee), pv(packed_arms)])
        }
    }
}

/// Decode one EDN-packed expression back into an `Expr` — the exact inverse of
/// [`pack_expr`], which sits directly above it.
///
/// **The two are a pair and must be edited as one.** Every `Expr` variant needs
/// an arm in `pack_expr` that writes its keyword tag and an arm here that reads
/// it back; a variant added to only one side compiles cleanly and breaks the
/// export/import round-trip at runtime instead. `tests/rete/probe_arc278_export.wat`
/// and `probe_arc278_rete_edn.wat` are what catch that — a new variant belongs in
/// their corpus in the same change.
///
/// Shape: every packed expr is a sequence whose head is the variant's keyword
/// tag and whose tail is that variant's fields, in declaration order. Unknown
/// tags and missing fields raise `MalformedForm` under `IMPORT_OP` rather than
/// defaulting, because a silently-defaulted field would import a DIFFERENT rule
/// than the one exported.
fn unpack_expr(v: &Value, span: &Span) -> Result<Expr, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = items.first().ok_or_else(|| malformed(span, IMPORT_OP, "empty expr"))?;
    match expect_kw(tag, IMPORT_OP, span)? {
        ":lit" => Ok(Expr::Lit(
            items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "lit missing value"))?
                .clone(),
        )),
        ":slot" => {
            let n = expect_u16(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "slot missing n"))?,
                span,
                "slot",
            )?;
            Ok(Expr::Slot(n))
        }
        ":call" => {
            let op = expect_op(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "call missing op"))?,
                span,
            )?;
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x, span)?);
            }
            Ok(Expr::Call {
                op,
                args: args.into_boxed_slice(),
            })
        }
        ":call-fb" => {
            let op = expect_op(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "call-fb missing op"))?,
                span,
            )?;
            let fallback = Box::new(unpack_expr(items.get(2).ok_or_else(|| {
                malformed(span, IMPORT_OP, "call-fb missing fallback")
            })?, span)?);
            let mut args = Vec::new();
            for x in items.iter().skip(3) {
                args.push(unpack_expr(x, span)?);
            }
            Ok(Expr::CallFallback {
                op,
                args: args.into_boxed_slice(),
                fallback,
            })
        }
        ":user" => {
            let program = Arc::new(unpack_prog(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "user missing prog"))?,
                span,
            )?);
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x, span)?);
            }
            Ok(Expr::CallUser {
                program,
                args: args.into_boxed_slice(),
            })
        }
        ":field" => Ok(Expr::Field {
            recv: Box::new(unpack_expr(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "field missing recv"))?,
                span,
            )?),
            idx: expect_idx(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "field missing idx"))?,
                span,
                "field idx",
            )?,
        }),
        ":ctor" => {
            let class = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "ctor missing class"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "ctor missing names"))?,
                IMPORT_OP,
                span,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(3) {
                fields.push(unpack_expr(x, span)?);
            }
            if ns.len() != fields.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!("ctor names length {} != fields length {}", ns.len(), fields.len()),
                ));
            }
            Ok(Expr::Construct {
                class,
                names: Arc::new(ns),
                fields: fields.into_boxed_slice(),
            })
        }
        ":variant" => {
            let type_path = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing type"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let variant_name = expect_str(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing name"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(3)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing names"))?,
                IMPORT_OP,
                span,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(4) {
                fields.push(unpack_expr(x, span)?);
            }
            if ns.len() != fields.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!(
                        "variant names length {} != fields length {}",
                        ns.len(),
                        fields.len()
                    ),
                ));
            }
            Ok(Expr::Variant {
                type_path,
                variant_name,
                names: Arc::new(ns),
                fields: fields.into_boxed_slice(),
            })
        }
        ":if" => Ok(Expr::If {
            cond: Box::new(unpack_expr(
                items.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span,
            )?),
            then_: Box::new(unpack_expr(
                items.get(2).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span,
            )?),
            else_: Box::new(unpack_expr(
                items.get(3).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span,
            )?),
        }),
        ":and" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x, span)?);
            }
            Ok(Expr::And(xs.into_boxed_slice()))
        }
        ":or" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x, span)?);
            }
            Ok(Expr::Or(xs.into_boxed_slice()))
        }
        ":let" => {
            let binds_pv = expect_seq(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "let missing binds"))?,
                IMPORT_OP,
                span,
            )?;
            let mut binds = Vec::new();
            for pair in binds_pv.iter() {
                let p = expect_seq(pair, IMPORT_OP, span)?;
                let slot = expect_u16(
                    p.first()
                        .ok_or_else(|| malformed(span, IMPORT_OP, "let bind"))?,
                    span,
                    "let bind",
                )?;
                let e = unpack_expr(p.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "let bind"))?, span)?;
                binds.push((slot, e));
            }
            Ok(Expr::Let {
                binds: binds.into_boxed_slice(),
                body: Box::new(unpack_expr(
                    items
                        .get(2)
                        .ok_or_else(|| malformed(span, IMPORT_OP, "let missing body"))?,
                    span,
                )?),
            })
        }
        ":match" => {
            let scrutinee = Box::new(unpack_expr(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "match missing scrut"))?,
                span,
            )?);
            let arms_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "match missing arms"))?,
                IMPORT_OP,
                span,
            )?;
            let mut arms = Vec::new();
            for a in arms_pv.iter() {
                let p = expect_seq(a, IMPORT_OP, span)?;
                arms.push((
                    unpack_pat(p.first().ok_or_else(|| malformed(span, IMPORT_OP, "arm"))?, span)?,
                    unpack_expr(p.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "arm"))?, span)?,
                ));
            }
            Ok(Expr::Match {
                scrutinee,
                arms: arms.into_boxed_slice(),
            })
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown expr {other}"))),
    }
}

fn pack_prog(p: &Program) -> Value {
    let names = p.names.iter().map(|n| match n {
        Some(s) => Value::String(Arc::new(s.to_string())),
        None => Value::Unit,
    });
    let reads = p.reads.iter().map(|(k, s)| pv([k.clone(), Value::i64(*s as i64)]));
    let params = p.params.iter().map(|s| Value::i64(*s as i64));
    pv([
        kw(":prog"),
        Value::i64(p.frame_len as i64),
        pv(params),
        pv(names),
        pv(reads),
        pack_expr(&p.root),
    ])
}

fn unpack_prog(v: &Value, span: &Span) -> Result<Program, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    if expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(span, IMPORT_OP, "empty prog"))?,
        IMPORT_OP,
        span,
    )? != ":prog"
    {
        return Err(malformed(span, IMPORT_OP, "expected :prog"));
    }
    let frame_len = expect_u16(
        items
            .get(1)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog frame"))?,
        span,
        "prog frame",
    )?;
    let params_pv = expect_seq(
        items
            .get(2)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog params"))?,
        IMPORT_OP,
        span,
    )?;
    let mut params = Vec::new();
    for x in params_pv.iter() {
        params.push(expect_u16(x, span, "prog param")?);
    }
    let names_pv = expect_seq(
        items
            .get(3)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog names"))?, IMPORT_OP, span)?;
    let mut names = Vec::new();
    for x in names_pv.iter() {
        names.push(match x {
            Value::String(s) => Some(Arc::<str>::from(s.as_str())),
            _ => None,
        });
    }
    let reads_pv = expect_seq(
        items
            .get(4)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog reads"))?,
        IMPORT_OP,
        span,
    )?;
    let mut reads = Vec::new();
    for x in reads_pv.iter() {
        let p = expect_seq(x, IMPORT_OP, span)?;
        reads.push((
            p.first()
                .ok_or_else(|| malformed(span, IMPORT_OP, "read key"))?
                .clone(),
            expect_u16(
                p.get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "read slot"))?,
                span,
                "read slot",
            )?,
        ));
    }
    let root = unpack_expr(
        items
            .get(5)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog root"))?,
        span,
    )?;
    let program = Program {
        frame_len,
        root,
        reads: reads.into(),
        params: params.into_boxed_slice(),
        names: names.into_boxed_slice(),
        span: span.clone(),
    };
    check_program_slots(&program, span)?;
    Ok(program)
}

fn pack_cond_op(op: &Op) -> Value {
    match op {
        Op::Bind { field_idx, slot } => pv([
            kw(":bind"),
            Value::i64(*field_idx as i64),
            Value::i64(*slot as i64),
        ]),
        Op::BindCheck { field_idx, slot } => pv([
            kw(":bchk"),
            Value::i64(*field_idx as i64),
            Value::i64(*slot as i64),
        ]),
        Op::Cmp { op, lhs, rhs } => {
            pv([kw(":cmp"), pack_cmp(*op), pack_expr(lhs), pack_expr(rhs)])
        }
        Op::SeedCmp { op, lhs, rhs } => {
            pv([kw(":scmp"), pack_cmp(*op), pack_expr(lhs), pack_expr(rhs)])
        }
        Op::Or(branches) => {
            let mut xs = vec![kw(":or-c")];
            xs.extend(branches.iter().map(|b| pv(b.iter().map(pack_cond_op))));
            pv(xs)
        }
        Op::Not(inner) => {
            let mut xs = vec![kw(":not-c")];
            xs.extend(inner.iter().map(pack_cond_op));
            pv(xs)
        }
        Op::Fail => pv([kw(":fail")]),
    }
}

fn unpack_cond_op(v: &Value, span: &Span) -> Result<Op, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(span, IMPORT_OP, "empty cond-op"))?,
        IMPORT_OP,
        span,
    )? {
        ":bind" => Ok(Op::Bind {
            field_idx: expect_idx(expect_at(&items, 1, span, "bind field_idx")?, span, "bind field_idx")?,
            slot: expect_idx(expect_at(&items, 2, span, "bind slot")?, span, "bind slot")?,
        }),
        ":bchk" => Ok(Op::BindCheck {
            field_idx: expect_idx(expect_at(&items, 1, span, "bchk field_idx")?, span, "bchk field_idx")?,
            slot: expect_idx(expect_at(&items, 2, span, "bchk slot")?, span, "bchk slot")?,
        }),
        ":cmp" => Ok(Op::Cmp {
            op: unpack_cmp(expect_at(&items, 1, span, "cmp op")?, span)?,
            lhs: unpack_expr(expect_at(&items, 2, span, "cmp lhs")?, span)?,
            rhs: unpack_expr(expect_at(&items, 3, span, "cmp rhs")?, span)?,
        }),
        ":scmp" => Ok(Op::SeedCmp {
            op: unpack_cmp(expect_at(&items, 1, span, "scmp op")?, span)?,
            lhs: unpack_expr(expect_at(&items, 2, span, "scmp lhs")?, span)?,
            rhs: unpack_expr(expect_at(&items, 3, span, "scmp rhs")?, span)?,
        }),
        ":or-c" => {
            let mut branches = Vec::new();
            for b in items.iter().skip(1) {
                let bp = expect_seq(b, IMPORT_OP, span)?;
                let mut ops = Vec::new();
                for x in bp.iter() {
                    ops.push(unpack_cond_op(x, span)?);
                }
                branches.push(ops);
            }
            Ok(Op::Or(branches))
        }
        ":not-c" => {
            let mut inner = Vec::new();
            for x in items.iter().skip(1) {
                inner.push(unpack_cond_op(x, span)?);
            }
            Ok(Op::Not(inner))
        }
        ":fail" => Ok(Op::Fail),
        other => Err(malformed(span, IMPORT_OP, format!("unknown cond-op {other}"))),
    }
}

fn pack_compiled_cond(c: &CompiledCond) -> Value {
    let ops = c.ops().iter().map(pack_cond_op);
    let keys = c.slot_keys().iter().cloned();
    let slots = c.output_slots().iter().map(|s| Value::i64(*s as i64));
    let seeds = c
        .seed_reads()
        .iter()
        .map(|(k, s)| pv([k.clone(), Value::i64(*s as i64)]));
    let bind = match c.fact_bind() {
        Some(v) => v.clone(),
        None => Value::Unit,
    };
    pv([
        kw(":cond"),
        Value::i64(c.n_slots() as i64),
        bind,
        pv(keys),
        pv(slots),
        pv(seeds),
        pv(ops),
    ])
}

fn unpack_compiled_cond(v: &Value, span: &Span) -> Result<CompiledCond, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    if expect_kw(expect_at(&items, 0, span, ":cond tag")?, IMPORT_OP, span)? != ":cond" {
        return Err(malformed(span, IMPORT_OP, "expected :cond"));
    }
    let n_slots = expect_idx(expect_at(&items, 1, span, "n_slots")?, span, "n_slots")?;
    let fact_bind = match expect_at(&items, 2, span, "fact_bind")? {
        Value::String(_) => Some(items[2].clone()),
        _ => None,
    };
    let keys_pv = expect_seq(expect_at(&items, 3, span, "slot_keys")?, IMPORT_OP, span)?;
    let slot_keys: Arc<[Value]> = keys_pv.into();
    let slots_pv = expect_seq(expect_at(&items, 4, span, "output_slots")?, IMPORT_OP, span)?;
    let output_slots: Arc<[usize]> = slots_pv
        .iter()
        .map(|x| expect_idx(x, span, "output slot"))
        .collect::<Result<Vec<_>, _>>()?
        .into();
    let seeds_pv = expect_seq(expect_at(&items, 5, span, "seed_reads")?, IMPORT_OP, span)?;
    let mut seed_reads = Vec::new();
    for x in seeds_pv.iter() {
        let p = expect_seq(x, IMPORT_OP, span)?;
        seed_reads.push((
            expect_at(&p, 0, span, "seed key")?.clone(),
            expect_idx(expect_at(&p, 1, span, "seed slot")?, span, "seed slot")?,
        ));
    }
    let ops_pv = expect_seq(expect_at(&items, 6, span, "ops")?, IMPORT_OP, span)?;
    let mut ops = Vec::new();
    for x in ops_pv.iter() {
        ops.push(unpack_cond_op(x, span)?);
    }
    for s in output_slots.iter() {
        if *s >= n_slots {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("output slot {s} >= n_slots {n_slots}"),
            ));
        }
    }
    for (_, s) in seed_reads.iter() {
        if *s >= n_slots {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("seed slot {s} >= n_slots {n_slots}"),
            ));
        }
    }
    if slot_keys.len() != output_slots.len() {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "slot_keys length {} != output_slots length {}",
                slot_keys.len(),
                output_slots.len()
            ),
        ));
    }
    check_cond_ops(&ops, n_slots, span)?;
    Ok(CompiledCond::from_parts(
        ops,
        slot_keys,
        output_slots,
        n_slots,
        seed_reads.into(),
        fact_bind,
    ))
}

fn pack_driver(d: &CondDriver) -> Value {
    match d {
        CondDriver::Leaf(id) => pv([kw(":leaf"), Value::i64(*id)]),
        CondDriver::And(ks) => {
            let mut xs = vec![kw(":and")];
            xs.extend(ks.iter().map(pack_driver));
            pv(xs)
        }
        CondDriver::Or(ks) => {
            let mut xs = vec![kw(":or")];
            xs.extend(ks.iter().map(pack_driver));
            pv(xs)
        }
        CondDriver::Not(inner) => pv([kw(":not"), pack_driver(inner)]),
        CondDriver::Exists(inner) => pv([kw(":exists"), pack_driver(inner)]),
        CondDriver::Where(p) => pv([kw(":where"), pack_prog(p)]),
    }
}

fn unpack_driver(v: &Value, span: &Span) -> Result<CondDriver, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "driver tag")?, IMPORT_OP, span)? {
        ":leaf" => Ok(CondDriver::Leaf(expect_i64(
            expect_at(&items, 1, span, "leaf id")?,
            IMPORT_OP,
            span,
        )?)),
        ":and" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x, span)?);
            }
            Ok(CondDriver::And(ks))
        }
        ":or" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x, span)?);
            }
            Ok(CondDriver::Or(ks))
        }
        ":not" => Ok(CondDriver::Not(Box::new(unpack_driver(
            expect_at(&items, 1, span, "not inner")?,
            span,
        )?))),
        ":exists" => Ok(CondDriver::Exists(Box::new(unpack_driver(
            expect_at(&items, 1, span, "exists inner")?,
            span,
        )?))),
        ":where" => Ok(CondDriver::Where(Arc::new(unpack_prog(
            expect_at(&items, 1, span, "where program")?,
            span,
        )?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown driver {other}"))),
    }
}

fn pack_fold(f: &AccFold) -> Value {
    match f {
        AccFold::Count => pv([kw(":count")]),
        AccFold::Sum(k) => pv([kw(":sum"), k.clone()]),
        AccFold::Min(k) => pv([kw(":min"), k.clone()]),
        AccFold::Max(k) => pv([kw(":max"), k.clone()]),
        AccFold::Mean(k) => pv([kw(":mean"), k.clone()]),
        AccFold::Distinct(k) => pv([kw(":distinct"), k.clone()]),
        AccFold::All => pv([kw(":all")]),
        AccFold::GroupBy(k) => pv([kw(":group"), k.clone()]),
        AccFold::User { var, program } => pv([kw(":ufold"), var.clone(), pack_prog(program)]),
    }
}

fn unpack_fold(v: &Value, span: &Span) -> Result<AccFold, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "fold tag")?, IMPORT_OP, span)? {
        ":count" => Ok(AccFold::Count),
        ":sum" => Ok(AccFold::Sum(expect_at(&items, 1, span, "sum key")?.clone())),
        ":min" => Ok(AccFold::Min(expect_at(&items, 1, span, "min key")?.clone())),
        ":max" => Ok(AccFold::Max(expect_at(&items, 1, span, "max key")?.clone())),
        ":mean" => Ok(AccFold::Mean(expect_at(&items, 1, span, "mean key")?.clone())),
        ":distinct" => Ok(AccFold::Distinct(expect_at(&items, 1, span, "distinct key")?.clone())),
        ":all" => Ok(AccFold::All),
        ":group" => Ok(AccFold::GroupBy(expect_at(&items, 1, span, "group key")?.clone())),
        ":ufold" => Ok(AccFold::User {
            var: expect_at(&items, 1, span, "ufold var")?.clone(),
            program: Arc::new(unpack_prog(expect_at(&items, 2, span, "ufold program")?, span)?),
        }),
        other => Err(malformed(span, IMPORT_OP, format!("unknown fold {other}"))),
    }
}

fn pack_rhs_op(op: &RhsOp) -> Value {
    match op {
        // Slot name only. The second Bind field is a Debug rendering of
        // WatAST for fire-time unbound errors — source, not residual.
        RhsOp::Bind(k, _) => pv([kw(":rbind"), k.clone()]),
        RhsOp::Lit(v) => pv([kw(":rlit"), v.clone()]),
        RhsOp::Expr(p) => pv([kw(":rexpr"), pack_prog(p)]),
    }
}

fn unpack_rhs_op(v: &Value, span: &Span) -> Result<RhsOp, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "rhs-op tag")?, IMPORT_OP, span)? {
        ":rbind" => {
            let k = items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "rbind missing key"))?
                .clone();
            let dbg = match items.get(2) {
                Some(v) => expect_str(v, IMPORT_OP, span)?.to_string(),
                None => match &k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                },
            };
            Ok(RhsOp::Bind(k, dbg))
        }
        ":rlit" => Ok(RhsOp::Lit(expect_at(&items, 1, span, "rlit value")?.clone())),
        ":rexpr" => Ok(RhsOp::Expr(Arc::new(unpack_prog(
            expect_at(&items, 1, span, "rexpr prog")?,
            span,
        )?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown rhs-op {other}"))),
    }
}

fn pack_rhs(r: &CompiledRhs) -> Value {
    match r {
        CompiledRhs::Record { class, names, ops } => {
            let mut xs = vec![
                kw(":rec"),
                Value::String(Arc::new(class.to_string())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(ops.iter().map(pack_rhs_op));
            pv(xs)
        }
        CompiledRhs::Call(p) => pv([kw(":rcall"), pack_prog(p)]),
    }
}

fn unpack_rhs(v: &Value, span: &Span) -> Result<CompiledRhs, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "tag")?, IMPORT_OP, span)? {
        ":rec" => {
            let class: Arc<str> = expect_str(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?.into();
            let names_pv = expect_seq(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut ops = Vec::new();
            for x in items.iter().skip(3) {
                ops.push(unpack_rhs_op(x, span)?);
            }
            if ns.len() != ops.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!("rhs names length {} != ops length {}", ns.len(), ops.len()),
                ));
            }
            Ok(CompiledRhs::Record {
                class,
                names: Arc::new(ns),
                ops,
            })
        }
        ":rcall" => Ok(CompiledRhs::Call(Arc::new(unpack_prog(expect_at(&items, 1, span, "slot 1")?, span)?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown rhs {other}"))),
    }
}

// ── topology ─────────────────────────────────────────────────────────────────

type ClassFields = Vec<Vec<String>>;

struct ClassIntern {
    names: Vec<String>,
    fields: ClassFields,
    idx: HashMap<String, usize>,
}

impl ClassIntern {
    fn new() -> Self {
        ClassIntern {
            names: Vec::new(),
            fields: Vec::new(),
            idx: HashMap::new(),
        }
    }
    fn intern(&mut self, class: &str, field_names: Vec<String>) -> i64 {
        if let Some(i) = self.idx.get(class) {
            return *i as i64;
        }
        let i = self.names.len();
        self.idx.insert(class.to_string(), i);
        self.names.push(class.to_string());
        self.fields.push(field_names);
        i as i64
    }
}

fn pack_children(node: &Value) -> impl Iterator<Item = Value> {
    node_children(node).into_iter().map(Value::i64)
}

fn pack_node(
    node: &Value,
    classes: &mut ClassIntern,
    sym: &SymbolTable,
    tree: &AlphaTree,
) -> Value {
    let id = node_named_i64(node, "id").unwrap_or(-1);
    match kind_of(node) {
        NodeKind::Alpha => {
            let class_idx = alpha_cond_from_node(node)
                .and_then(|ast| alpha_pattern(&ast).map(|p| {
                    let ty = p.type_head.to_string();
                    let fs = class_field_names(sym, &ty);
                    classes.intern(&ty, fs)
                }))
                .or_else(|| {
                    tree.class_for_alpha(id).map(|ty| {
                        let fs = class_field_names(sym, ty);
                        classes.intern(ty, fs)
                    })
                })
                .unwrap_or(-1);
            let mut xs = vec![kw(":a"), Value::i64(id), Value::i64(class_idx)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::RootJoin => {
            let mut xs = vec![kw(":j"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::HashJoin => {
            let mut xs = vec![kw(":h"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Production => {
            let name = node_named_string(node, "rule-name")
                .unwrap_or("")
                .to_string();
            pv([kw(":p"), Value::i64(id), Value::String(Arc::new(name))])
        }
        NodeKind::Test => {
            let mut xs = vec![kw(":t"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Negation => {
            let aid = node_named_i64(node, "negated-alpha-id").unwrap_or(-1);
            let mut xs = vec![kw(":n"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Exists => {
            let aid = node_named_i64(node, "exists-alpha-id").unwrap_or(-1);
            let mut xs = vec![kw(":e"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Accumulate => {
            let var = node_named_string(node, "result-var")
                .unwrap_or("")
                .to_string();
            let aid = node_named_i64(node, "from-alpha-id").unwrap_or(-1);
            let mut xs = vec![
                kw(":acc"),
                Value::i64(id),
                Value::String(Arc::new(var)),
                Value::i64(aid),
            ];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Query => {
            let name = node_named_string(node, "query-name")
                .unwrap_or("")
                .to_string();
            let params = match node_named_field(node, "param-keys") {
                Some(Value::wat__core__PersistentVector(pv)) => pv.iter().cloned().collect(),
                _ => vec![],
            };
            let mut xs = vec![kw(":q"), Value::i64(id), Value::String(Arc::new(name))];
            xs.extend(params);
            pv(xs)
        }
    }
}

fn unpack_i64s(items: &[Value], skip: usize, span: &Span) -> Result<Vec<i64>, EvalBreak> {
    let mut out = Vec::new();
    for x in items.iter().skip(skip) {
        out.push(expect_i64(x, IMPORT_OP, span)?);
    }
    Ok(out)
}

fn i64_pv(ids: &[i64]) -> Value {
    let mut v = crate::value::pvec::PVec::new();
    for id in ids {
        v.push_back_mut(Value::i64(*id));
    }
    Value::wat__core__PersistentVector(v)
}

type UnpackedNode = (i64, Value, Option<i64>);

fn unpack_node(v: &Value, span: &Span) -> Result<UnpackedNode, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = expect_kw(expect_at(&items, 0, span, "tag")?, IMPORT_OP, span)?;
    match tag {
        ":a" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let class_idx = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::AlphaNode",
                ALPHA_FIELDS,
                vec![Value::i64(id), empty_pv(), i64_pv(&kids)],
            );
            let class = if class_idx >= 0 {
                Some(class_idx)
            } else {
                None
            };
            Ok((id, rec, class))
        }
        ":j" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::RootJoinNode",
                ROOT_FIELDS,
                vec![Value::i64(id), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":h" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::HashJoinNode",
                HASH_FIELDS,
                vec![Value::i64(id), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":p" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let name = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let rec = record(
                "wat::rete::ProductionNode",
                PROD_FIELDS,
                vec![Value::i64(id), Value::String(Arc::new(name))],
            );
            Ok((id, rec, None))
        }
        ":t" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::TestNode",
                TEST_FIELDS,
                vec![Value::i64(id), dummy_ast(span), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":n" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let aid = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::NegationNode",
                NEG_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":e" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let aid = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::ExistsNode",
                EXISTS_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":acc" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let var = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let aid = expect_i64(expect_at(&items, 3, span, "slot 3")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 4, span)?;
            let rec = record(
                "wat::rete::AccumulateNode",
                ACC_FIELDS,
                vec![
                    Value::i64(id),
                    Value::String(Arc::new(var)),
                    dummy_ast(span),
                    Value::i64(aid),
                    i64_pv(&kids),
                ],
            );
            Ok((id, rec, None))
        }
        ":q" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let name = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let mut params = crate::value::pvec::PVec::new();
            for x in items.iter().skip(3) {
                params.push_back_mut(x.clone());
            }
            let rec = record(
                "wat::rete::QueryNode",
                QUERY_FIELDS,
                vec![
                    Value::i64(id),
                    Value::String(Arc::new(name)),
                    Value::wat__core__PersistentVector(params),
                ],
            );
            Ok((id, rec, None))
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown node {other}"))),
    }
}

// ── Session field readers ────────────────────────────────────────────────────

fn session_network_rules<'a>(
    session: &'a Value,
    span: &Span,
) -> Result<(&'a Value, &'a Value), EvalBreak> {
    match (session_network(session), session_named_field(session, "rules")) {
        (Some(network), Some(rules)) => Ok((network, rules)),
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            },
        )
        .into()),
    }
}

fn map_i64<V>(m: &HashMap<i64, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    let mut keys: Vec<i64> = m.keys().copied().collect();
    keys.sort_unstable();
    let mut pairs = Vec::with_capacity(keys.len());
    for k in keys {
        pairs.push(pv([Value::i64(k), f(m.get(&k).expect("sorted key"))]));
    }
    pv(pairs)
}

fn map_str<V>(m: &HashMap<String, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let mut pairs = Vec::with_capacity(keys.len());
    for k in keys {
        pairs.push(pv([Value::String(Arc::new(k.clone())), f(m.get(k).expect("sorted key"))]));
    }
    pv(pairs)
}

// ── public mouths ────────────────────────────────────────────────────────────

/// `(:wat::rete::export <session>) -> :wat::rete::Export`
pub(crate) fn eval_export(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
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
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let (network, rules) = session_network_rules(&session, list_span)?;
    // Pack door: MISS intern's (`DESIGN-STONE-intern-eviction`); HIT reuses the compile lease.
    let arm = rete_arm_get_or_build(network, rules, sym)?;
    let mut classes = ClassIntern::new();
    let mut nodes = Vec::new();
    for id in sorted_node_ids(network) {
        if let Some(node) = get_node(network, id) {
            nodes.push(pack_node(node, &mut classes, sym, &arm.alpha_tree));
        }
    }
    let conds = map_i64(&arm.compiled_conds, pack_compiled_cond);
    let drivers = map_i64(&arm.compiled_drivers, pack_driver);
    let progs = map_i64(&arm.compiled_wheres, pack_prog);
    let folds = map_i64(&arm.compiled_acc_folds, pack_fold);
    let rhs = map_str(&arm.compiled_rhs, |items| {
        pv(items.iter().map(pack_rhs))
    });
    // Residual stratify schedule lives on the interned arm, not Session.rules
    // (`wat/rete.wat` Export/deps). Import drops source forms; packing from
    // `rule_deps_from_rules(session.rules)` wrote empty deps on re-export.
    let deps = pack_deps(&arm.rule_deps);
    let abi = abi_of(&classes.names, &classes.fields);
    let class_pv = pv(classes
        .names
        .iter()
        .map(|c| Value::String(Arc::new(c.clone()))));
    let fields_pv = pv(classes.fields.iter().map(|fs| {
        pv(fs.iter().map(|f| Value::String(Arc::new(f.clone()))))
    }));
    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Export".into(),
        export_names(),
        Arc::new(vec![
            Value::i64(FORMAT_V),
            Value::String(Arc::new(abi)),
            class_pv,
            fields_pv,
            pv(nodes),
            conds,
            drivers,
            progs,
            folds,
            rhs,
            deps,
        ]),
    ))))
}

fn pack_deps(deps: &[RuleDep]) -> Value {
    pv(deps.iter().map(|d| {
        pv([
            Value::String(Arc::new(d.name.clone())),
            pv(d.view.produced.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view.negated.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view.consumed.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view
                .exists_and_from_types
                .iter()
                .map(|s| Value::String(Arc::new(s.clone())))),
        ])
    }))
}

fn unpack_string_list(v: &Value, span: &Span) -> Result<Vec<String>, EvalBreak> {
    let xs = expect_seq(v, IMPORT_OP, span)?;
    let mut out = Vec::new();
    for x in xs {
        out.push(expect_str(&x, IMPORT_OP, span)?.to_string());
    }
    Ok(out)
}

fn unpack_deps(v: &Value, span: &Span) -> Result<Vec<RuleDep>, EvalBreak> {
    let mut out = Vec::new();
    for row in expect_seq(v, IMPORT_OP, span)? {
        let p = expect_seq(&row, IMPORT_OP, span)?;
        if p.len() < 4 {
            return Err(malformed(span, IMPORT_OP, "deps row needs name + 4 lists (5th optional for old rows)"));
        }
        let bag = if p.len() >= 5 {
            unpack_string_list(&p[4], span)?
        } else {
            Vec::new()
        };
        out.push(RuleDep {
            name: expect_str(&p[0], IMPORT_OP, span)?.to_string(),
            view: crate::rete::kernel::StratifyView {
                produced: unpack_string_list(&p[1], span)?,
                negated: unpack_string_list(&p[2], span)?,
                consumed: unpack_string_list(&p[3], span)?,
                exists_and_from_types: bag,
            },
        });
    }
    Ok(out)
}

/// `(:wat::rete::import <export>) -> :wat::rete::Session`
///
/// Slim topology, interned arm, empty facts. Fire does not lower.
/// Stratify schedule is `:deps` (produced / negated / consumed / exists-and-from class names).
pub(crate) fn eval_import(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: IMPORT_OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let export = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    import_export(&export, list_span, sym)
}

fn import_export(export: &Value, span: &Span, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let agg = match export {
        Value::Aggregate(a) if a.nature != Nature::Struct && a.class.as_ref() == "wat::rete::Export" => a,
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: IMPORT_OP.into(),
                    expected: ":wat::rete::Export",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let _ = agg;
    let v = expect_i64(export_named(export, "v", span)?, IMPORT_OP, span)?;
    if v != FORMAT_V {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("unsupported Export version {v}"),
        ));
    }
    let stored_abi = expect_str(export_named(export, "abi", span)?, IMPORT_OP, span)?;
    let classes_pv = expect_seq(export_named(export, "classes", span)?, IMPORT_OP, span)?;
    let mut classes = Vec::new();
    for c in classes_pv.iter() {
        classes.push(expect_str(c, IMPORT_OP, span)?.to_string());
    }
    let fields_pv = expect_seq(export_named(export, "fields", span)?, IMPORT_OP, span)?;
    let mut fields = Vec::new();
    for row in fields_pv.iter() {
        let rp = expect_seq(row, IMPORT_OP, span)?;
        let mut fs = Vec::new();
        for f in rp.iter() {
            fs.push(expect_str(f, IMPORT_OP, span)?.to_string());
        }
        fields.push(fs);
    }
    if classes.len() != fields.len() {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "classes length {} != fields length {}",
                classes.len(),
                fields.len()
            ),
        ));
    }
    let expect_abi = abi_of(&classes, &fields);
    if stored_abi != expect_abi {
        return Err(malformed(
            span,
            IMPORT_OP,
            "ABI mismatch — export is from a different packed-classes/RETE_OPS",
        ));
    }
    // Host TypeEnv field-order. Packed ABI can agree with itself and still
    // disagree with this process's declared records.
    for (c, packed) in classes.iter().zip(fields.iter()) {
        let host = class_field_names(sym, c);
        if !host.is_empty() && &host != packed {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("ABI mismatch — host TypeEnv field-order for {c} differs from export"),
            ));
        }
    }

    let nodes_pv = expect_seq(export_named(export, "nodes", span)?, IMPORT_OP, span)?;
    let mut network_pairs = Vec::new();
    let mut alpha_by_type: AlphasByType = HashMap::new();
    let mut max_id = 0i64;
    for n in nodes_pv.iter() {
        let (id, rec, class_hint) = unpack_node(n, span)?;
        if id > max_id {
            max_id = id;
        }
        if let Some(class_idx) = class_hint {
            if let Some(name) = classes.get(class_idx as usize) {
                alpha_by_type.entry(name.clone()).or_default().push(id);
            }
        }
        network_pairs.push((Value::i64(id), rec));
    }
    let network = Value::wat__core__PersistentMap(PMap::from_pairs(network_pairs));

    let mut compiled_conds = HashMap::new();
    for pair in expect_seq(export_named(export, "conds", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_conds.insert(
            expect_i64(expect_at(&p, 0, span, "cond id")?, IMPORT_OP, span)?,
            unpack_compiled_cond(expect_at(&p, 1, span, "cond")?, span)?,
        );
    }
    let mut compiled_drivers = HashMap::new();
    for pair in expect_seq(export_named(export, "drivers", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_drivers.insert(
            expect_i64(expect_at(&p, 0, span, "driver id")?, IMPORT_OP, span)?,
            unpack_driver(expect_at(&p, 1, span, "driver")?, span)?,
        );
    }
    let mut compiled_wheres = HashMap::new();
    for pair in expect_seq(export_named(export, "progs", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_wheres.insert(
            expect_i64(expect_at(&p, 0, span, "prog id")?, IMPORT_OP, span)?,
            unpack_prog(expect_at(&p, 1, span, "prog")?, span)?,
        );
    }
    let mut compiled_acc_folds = HashMap::new();
    for pair in expect_seq(export_named(export, "folds", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_acc_folds.insert(
            expect_i64(expect_at(&p, 0, span, "fold id")?, IMPORT_OP, span)?,
            unpack_fold(expect_at(&p, 1, span, "fold")?, span)?,
        );
    }
    let mut compiled_rhs: CompiledRhsByRule = HashMap::new();
    for pair in expect_seq(export_named(export, "rhs", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        let name = expect_str(expect_at(&p, 0, span, "rhs name")?, IMPORT_OP, span)?.to_string();
        let items = expect_seq(expect_at(&p, 1, span, "rhs items")?, IMPORT_OP, span)?;
        let mut rs = Vec::new();
        for x in &items {
            rs.push(unpack_rhs(x, span)?);
        }
        compiled_rhs.insert(name, rs);
    }

    let rule_deps = match agg_named_field(export, "deps") {
        Some(d) => unpack_deps(d, span)?,
        None => Vec::new(),
    };

    let node_ids = sorted_node_ids(&network);
    let crate::rete::kernel::NetworkEdges {
        feeding_alpha_of,
        parents_of,
        children_of,
        beta_readers,
    } = crate::rete::kernel::index_network_edges(&network, &node_ids);
    let compiled_max_slots = compiled_conds.values().map(|c| c.n_slots()).max().unwrap_or(0);
    let alpha_tree = AlphaTree::unpruned(&alpha_by_type);
    let where_tree = crate::rete::where_tree::WhereTree::build(&compiled_wheres);
    let kind_ids = kind_id_lists(&network, &node_ids);
    let joins_fed_by = invert_feeding_alpha(&feeding_alpha_of);
    let test_sibs = crate::rete::kernel::build_test_sibs(&network, &node_ids, &parents_of);
    let test_children = crate::rete::kernel::build_test_children(&network, &node_ids);
    let arm = Arc::new(InternedNetwork {
        node_ids,
        kind_ids,
        compiled_conds,
        compiled_drivers,
        compiled_wheres,
        compiled_acc_folds,
        compiled_rhs,
        alpha_tree,
        where_tree,
        feeding_alpha_of,
        joins_fed_by,
        parents_of,
        beta_readers,
        compiled_max_slots,
        rule_deps,
        test_sibs,
        test_children,
        children_of,
    });
    if let Some(id) = network_identity(&network) {
        // Import is an arm-session equivalent: MISS leases=1, HIT increments
        // (`DESIGN-STONE-intern-eviction`). Drop without release-session leaks until thread end.
        rete_arm_intern(id, &arm);
    }

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Session".into(),
        session_names(),
        Arc::new(vec![
            network,
            empty_pv(),
            empty_pm(),
            empty_pm(),
            empty_pm(),
            empty_pv(),
            Value::i64(max_id + 1),
            empty_pm(),
        ]),
    ))))
}
