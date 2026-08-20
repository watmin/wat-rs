//! `#wat.rete/Export` — the compiled program as one EDN value.
//!
//! Not a Session. No facts, no memories, no source forms. Native fire only.
//! One tag; interior is packed vectors (kind + integers + literals).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use crate::ast::WatAST;
use crate::rete::alpha_tree::AlphaTree;
use crate::rete::compiled_cond::{CompiledCond, Op};
use crate::rete::compiled_rhs::{CompiledRhs, RhsOp};
use crate::rete::expr_ir::{Expr, Pat, Program};
use crate::rete::kernel::{
    class_field_names, get_node, kind_of, network_identity, node_children, node_record,
    invert_feeding_alpha, kind_id_lists, rete_arm_get_or_build, rete_arm_intern,
    rule_deps_from_rules, session_names, sorted_node_ids, AccFold, CondDriver, ReteArm, RuleDep,
};
use crate::rete::matcher::{alpha_pattern, CmpKind};
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

fn names(fields: &'static [&'static str]) -> Arc<Vec<String>> {
    crate::value::value::names_arc_from_static(fields)
}

fn export_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| names(EXPORT_FIELDS)).clone()
}

fn kw(name: &str) -> Value {
    Value::wat__core__keyword(Arc::new(name.to_string()))
}

fn pv(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Vec(Arc::new(items.into_iter().collect()))
}

fn empty_pv() -> Value {
    Value::wat__core__PersistentVector(rpds::VectorSync::new_sync())
}

fn empty_pm() -> Value {
    Value::wat__core__PersistentMap(PMap::new())
}

fn dummy_ast() -> Value {
    Value::wat__WatAST(Arc::new(WatAST::List(Vec::new(), crate::rust_caller_span!())))
}

fn malformed(op: &str, reason: impl Into<String>) -> EvalBreak {
    RuntimeError::new(
        crate::rust_caller_span!(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: reason.into(),
        },
    )
    .into()
}

fn expect_kw<'a>(v: &'a Value, op: &str) -> Result<&'a str, EvalBreak> {
    match v {
        Value::wat__core__keyword(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "keyword tag",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_i64(v: &Value, op: &str) -> Result<i64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(*n),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_str<'a>(v: &'a Value, op: &str) -> Result<&'a str, EvalBreak> {
    match v {
        Value::String(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "string",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_seq(v: &Value, op: &str) -> Result<Vec<Value>, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok((**xs).clone()),
        Value::wat__core__PersistentVector(pv) => Ok(pv.iter().cloned().collect()),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
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

fn unpack_cmp(v: &Value) -> Result<CmpKind, EvalBreak> {
    match expect_kw(v, IMPORT_OP)? {
        ":eq" => Ok(CmpKind::Eq),
        ":neq" => Ok(CmpKind::NotEq),
        ":lt" => Ok(CmpKind::Lt),
        ":gt" => Ok(CmpKind::Gt),
        ":le" => Ok(CmpKind::Le),
        ":ge" => Ok(CmpKind::Ge),
        other => Err(malformed(IMPORT_OP, format!("unknown cmp {other}"))),
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

fn unpack_pat(v: &Value) -> Result<Pat, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    let tag = items.first().ok_or_else(|| malformed(IMPORT_OP, "empty pat"))?;
    match expect_kw(tag, IMPORT_OP)? {
        ":plit" => {
            let lit = items
                .get(1)
                .ok_or_else(|| malformed(IMPORT_OP, "plit missing value"))?
                .clone();
            Ok(Pat::Lit(lit))
        }
        ":wild" => Ok(Pat::Wild),
        ":pbind" => {
            let n = expect_i64(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "pbind missing slot"))?,
                IMPORT_OP,
            )?;
            Ok(Pat::Bind(n as u16))
        }
        ":pvar" => {
            let name = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "pvar missing name"))?,
                IMPORT_OP,
            )?
            .to_string();
            let payload = match items.get(2) {
                Some(inner) => Some(Box::new(unpack_pat(inner)?)),
                None => None,
            };
            Ok(Pat::Variant { name, payload })
        }
        other => Err(malformed(IMPORT_OP, format!("unknown pat {other}"))),
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

fn unpack_expr(v: &Value) -> Result<Expr, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    let tag = items.first().ok_or_else(|| malformed(IMPORT_OP, "empty expr"))?;
    match expect_kw(tag, IMPORT_OP)? {
        ":lit" => Ok(Expr::Lit(
            items
                .get(1)
                .ok_or_else(|| malformed(IMPORT_OP, "lit missing value"))?
                .clone(),
        )),
        ":slot" => {
            let n = expect_i64(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "slot missing n"))?,
                IMPORT_OP,
            )?;
            Ok(Expr::Slot(n as u16))
        }
        ":call" => {
            let op = expect_i64(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "call missing op"))?,
                IMPORT_OP,
            )? as u16;
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x)?);
            }
            Ok(Expr::Call {
                op,
                args: args.into_boxed_slice(),
            })
        }
        ":call-fb" => {
            let op = expect_i64(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "call-fb missing op"))?,
                IMPORT_OP,
            )? as u16;
            let fallback = Box::new(unpack_expr(items.get(2).ok_or_else(|| {
                malformed(IMPORT_OP, "call-fb missing fallback")
            })?)?);
            let mut args = Vec::new();
            for x in items.iter().skip(3) {
                args.push(unpack_expr(x)?);
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
                    .ok_or_else(|| malformed(IMPORT_OP, "user missing prog"))?,
            )?);
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x)?);
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
                    .ok_or_else(|| malformed(IMPORT_OP, "field missing recv"))?,
            )?),
            idx: expect_i64(
                items
                    .get(2)
                    .ok_or_else(|| malformed(IMPORT_OP, "field missing idx"))?,
                IMPORT_OP,
            )? as usize,
        }),
        ":ctor" => {
            let class = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "ctor missing class"))?,
                IMPORT_OP,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(IMPORT_OP, "ctor missing names"))?,
                IMPORT_OP,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(3) {
                fields.push(unpack_expr(x)?);
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
                    .ok_or_else(|| malformed(IMPORT_OP, "variant missing type"))?,
                IMPORT_OP,
            )?
            .to_string();
            let variant_name = expect_str(
                items
                    .get(2)
                    .ok_or_else(|| malformed(IMPORT_OP, "variant missing name"))?,
                IMPORT_OP,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(3)
                    .ok_or_else(|| malformed(IMPORT_OP, "variant missing names"))?,
                IMPORT_OP,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(4) {
                fields.push(unpack_expr(x)?);
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
                items.get(1).ok_or_else(|| malformed(IMPORT_OP, "if"))?,
            )?),
            then_: Box::new(unpack_expr(
                items.get(2).ok_or_else(|| malformed(IMPORT_OP, "if"))?,
            )?),
            else_: Box::new(unpack_expr(
                items.get(3).ok_or_else(|| malformed(IMPORT_OP, "if"))?,
            )?),
        }),
        ":and" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x)?);
            }
            Ok(Expr::And(xs.into_boxed_slice()))
        }
        ":or" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x)?);
            }
            Ok(Expr::Or(xs.into_boxed_slice()))
        }
        ":let" => {
            let binds_pv = expect_seq(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "let missing binds"))?,
                IMPORT_OP,
            )?;
            let mut binds = Vec::new();
            for pair in binds_pv.iter() {
                let p = expect_seq(pair, IMPORT_OP)?;
                let slot = expect_i64(
                    p.first().ok_or_else(|| malformed(IMPORT_OP, "let bind"))?,
                    IMPORT_OP,
                )? as u16;
                let e = unpack_expr(p.get(1).ok_or_else(|| malformed(IMPORT_OP, "let bind"))?)?;
                binds.push((slot, e));
            }
            Ok(Expr::Let {
                binds: binds.into_boxed_slice(),
                body: Box::new(unpack_expr(
                    items
                        .get(2)
                        .ok_or_else(|| malformed(IMPORT_OP, "let missing body"))?,
                )?),
            })
        }
        ":match" => {
            let scrutinee = Box::new(unpack_expr(
                items
                    .get(1)
                    .ok_or_else(|| malformed(IMPORT_OP, "match missing scrut"))?,
            )?);
            let arms_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(IMPORT_OP, "match missing arms"))?,
                IMPORT_OP,
            )?;
            let mut arms = Vec::new();
            for a in arms_pv.iter() {
                let p = expect_seq(a, IMPORT_OP)?;
                arms.push((
                    unpack_pat(p.first().ok_or_else(|| malformed(IMPORT_OP, "arm"))?)?,
                    unpack_expr(p.get(1).ok_or_else(|| malformed(IMPORT_OP, "arm"))?)?,
                ));
            }
            Ok(Expr::Match {
                scrutinee,
                arms: arms.into_boxed_slice(),
            })
        }
        other => Err(malformed(IMPORT_OP, format!("unknown expr {other}"))),
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

fn unpack_prog(v: &Value) -> Result<Program, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    if expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(IMPORT_OP, "empty prog"))?,
        IMPORT_OP,
    )? != ":prog"
    {
        return Err(malformed(IMPORT_OP, "expected :prog"));
    }
    let frame_len = expect_i64(
        items
            .get(1)
            .ok_or_else(|| malformed(IMPORT_OP, "prog frame"))?,
        IMPORT_OP,
    )? as u16;
    let params_pv = expect_seq(
        items
            .get(2)
            .ok_or_else(|| malformed(IMPORT_OP, "prog params"))?,
        IMPORT_OP,
    )?;
    let mut params = Vec::new();
    for x in params_pv.iter() {
        params.push(expect_i64(x, IMPORT_OP)? as u16);
    }
    let names_pv = expect_seq(
        items
            .get(3)
            .ok_or_else(|| malformed(IMPORT_OP, "prog names"))?,
        IMPORT_OP,
    )?;
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
            .ok_or_else(|| malformed(IMPORT_OP, "prog reads"))?,
        IMPORT_OP,
    )?;
    let mut reads = Vec::new();
    for x in reads_pv.iter() {
        let p = expect_seq(x, IMPORT_OP)?;
        reads.push((
            p.first()
                .ok_or_else(|| malformed(IMPORT_OP, "read key"))?
                .clone(),
            expect_i64(
                p.get(1).ok_or_else(|| malformed(IMPORT_OP, "read slot"))?,
                IMPORT_OP,
            )? as u16,
        ));
    }
    let root = unpack_expr(
        items
            .get(5)
            .ok_or_else(|| malformed(IMPORT_OP, "prog root"))?,
    )?;
    Ok(Program {
        frame_len,
        root,
        reads: reads.into(),
        params: params.into_boxed_slice(),
        names: names.into_boxed_slice(),
        span: crate::rust_caller_span!(),
    })
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

fn unpack_cond_op(v: &Value) -> Result<Op, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    match expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(IMPORT_OP, "empty cond-op"))?,
        IMPORT_OP,
    )? {
        ":bind" => Ok(Op::Bind {
            field_idx: expect_i64(items.get(1).unwrap(), IMPORT_OP)? as usize,
            slot: expect_i64(items.get(2).unwrap(), IMPORT_OP)? as usize,
        }),
        ":bchk" => Ok(Op::BindCheck {
            field_idx: expect_i64(items.get(1).unwrap(), IMPORT_OP)? as usize,
            slot: expect_i64(items.get(2).unwrap(), IMPORT_OP)? as usize,
        }),
        ":cmp" => Ok(Op::Cmp {
            op: unpack_cmp(items.get(1).unwrap())?,
            lhs: unpack_expr(items.get(2).unwrap())?,
            rhs: unpack_expr(items.get(3).unwrap())?,
        }),
        ":scmp" => Ok(Op::SeedCmp {
            op: unpack_cmp(items.get(1).unwrap())?,
            lhs: unpack_expr(items.get(2).unwrap())?,
            rhs: unpack_expr(items.get(3).unwrap())?,
        }),
        ":or-c" => {
            let mut branches = Vec::new();
            for b in items.iter().skip(1) {
                let bp = expect_seq(b, IMPORT_OP)?;
                let mut ops = Vec::new();
                for x in bp.iter() {
                    ops.push(unpack_cond_op(x)?);
                }
                branches.push(ops);
            }
            Ok(Op::Or(branches))
        }
        ":not-c" => {
            let mut inner = Vec::new();
            for x in items.iter().skip(1) {
                inner.push(unpack_cond_op(x)?);
            }
            Ok(Op::Not(inner))
        }
        ":fail" => Ok(Op::Fail),
        other => Err(malformed(IMPORT_OP, format!("unknown cond-op {other}"))),
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

fn unpack_compiled_cond(v: &Value) -> Result<CompiledCond, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    if expect_kw(items.first().unwrap(), IMPORT_OP)? != ":cond" {
        return Err(malformed(IMPORT_OP, "expected :cond"));
    }
    let n_slots = expect_i64(items.get(1).unwrap(), IMPORT_OP)? as usize;
    let fact_bind = match items.get(2) {
        Some(Value::String(_)) => Some(items.get(2).unwrap().clone()),
        _ => None,
    };
    let keys_pv = expect_seq(items.get(3).unwrap(), IMPORT_OP)?;
    let slot_keys: Arc<[Value]> = keys_pv.into();
    let slots_pv = expect_seq(items.get(4).unwrap(), IMPORT_OP)?;
    let output_slots: Arc<[usize]> = slots_pv
        .iter()
        .map(|x| expect_i64(x, IMPORT_OP).map(|n| n as usize))
        .collect::<Result<Vec<_>, _>>()?
        .into();
    let seeds_pv = expect_seq(items.get(5).unwrap(), IMPORT_OP)?;
    let mut seed_reads = Vec::new();
    for x in seeds_pv.iter() {
        let p = expect_seq(x, IMPORT_OP)?;
        seed_reads.push((
            p.first().unwrap().clone(),
            expect_i64(p.get(1).unwrap(), IMPORT_OP)? as usize,
        ));
    }
    let ops_pv = expect_seq(items.get(6).unwrap(), IMPORT_OP)?;
    let mut ops = Vec::new();
    for x in ops_pv.iter() {
        ops.push(unpack_cond_op(x)?);
    }
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

fn unpack_driver(v: &Value) -> Result<CondDriver, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    match expect_kw(items.first().unwrap(), IMPORT_OP)? {
        ":leaf" => Ok(CondDriver::Leaf(expect_i64(items.get(1).unwrap(), IMPORT_OP)?)),
        ":and" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x)?);
            }
            Ok(CondDriver::And(ks))
        }
        ":or" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x)?);
            }
            Ok(CondDriver::Or(ks))
        }
        ":not" => Ok(CondDriver::Not(Box::new(unpack_driver(items.get(1).unwrap())?))),
        ":exists" => Ok(CondDriver::Exists(Box::new(unpack_driver(
            items.get(1).unwrap(),
        )?))),
        ":where" => Ok(CondDriver::Where(Arc::new(unpack_prog(items.get(1).unwrap())?))),
        other => Err(malformed(IMPORT_OP, format!("unknown driver {other}"))),
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

fn unpack_fold(v: &Value) -> Result<AccFold, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    match expect_kw(items.first().unwrap(), IMPORT_OP)? {
        ":count" => Ok(AccFold::Count),
        ":sum" => Ok(AccFold::Sum(items.get(1).unwrap().clone())),
        ":min" => Ok(AccFold::Min(items.get(1).unwrap().clone())),
        ":max" => Ok(AccFold::Max(items.get(1).unwrap().clone())),
        ":mean" => Ok(AccFold::Mean(items.get(1).unwrap().clone())),
        ":distinct" => Ok(AccFold::Distinct(items.get(1).unwrap().clone())),
        ":all" => Ok(AccFold::All),
        ":group" => Ok(AccFold::GroupBy(items.get(1).unwrap().clone())),
        ":ufold" => Ok(AccFold::User {
            var: items.get(1).unwrap().clone(),
            program: Arc::new(unpack_prog(items.get(2).unwrap())?),
        }),
        other => Err(malformed(IMPORT_OP, format!("unknown fold {other}"))),
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

fn unpack_rhs_op(v: &Value) -> Result<RhsOp, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    match expect_kw(items.first().unwrap(), IMPORT_OP)? {
        ":rbind" => {
            let k = items
                .get(1)
                .ok_or_else(|| malformed(IMPORT_OP, "rbind missing key"))?
                .clone();
            let dbg = match items.get(2) {
                Some(v) => expect_str(v, IMPORT_OP)?.to_string(),
                None => match &k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                },
            };
            Ok(RhsOp::Bind(k, dbg))
        }
        ":rlit" => Ok(RhsOp::Lit(items.get(1).unwrap().clone())),
        ":rexpr" => Ok(RhsOp::Expr(Arc::new(unpack_prog(items.get(1).unwrap())?))),
        other => Err(malformed(IMPORT_OP, format!("unknown rhs-op {other}"))),
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

fn unpack_rhs(v: &Value) -> Result<CompiledRhs, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    match expect_kw(items.first().unwrap(), IMPORT_OP)? {
        ":rec" => {
            let class: Arc<str> = expect_str(items.get(1).unwrap(), IMPORT_OP)?.into();
            let names_pv = expect_seq(items.get(2).unwrap(), IMPORT_OP)?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP)?.to_string());
            }
            let mut ops = Vec::new();
            for x in items.iter().skip(3) {
                ops.push(unpack_rhs_op(x)?);
            }
            Ok(CompiledRhs::Record {
                class,
                names: Arc::new(ns),
                ops,
            })
        }
        ":rcall" => Ok(CompiledRhs::Call(Arc::new(unpack_prog(items.get(1).unwrap())?))),
        other => Err(malformed(IMPORT_OP, format!("unknown rhs {other}"))),
    }
}

// ── topology ─────────────────────────────────────────────────────────────────

struct ClassIntern {
    names: Vec<String>,
    fields: Vec<Vec<String>>,
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

fn pack_node(node: &Value, classes: &mut ClassIntern, _network: &Value, sym: &SymbolTable) -> Value {
    let id = match node_record(node) {
        Some((_, sf)) => match &sf[0] {
            Value::i64(n) => *n,
            _ => -1,
        },
        None => -1,
    };
    match kind_of(node) {
        "AlphaNode" => {
            let class_idx = node_record(node)
                .and_then(|(_, sf)| match &sf[1] {
                    Value::wat__core__PersistentVector(pv) => {
                        pv.first().and_then(|v| match v {
                            Value::wat__WatAST(ast) => alpha_pattern(ast).map(|p| {
                                let ty = p.type_head.to_string();
                                let fs = class_field_names(sym, &ty);
                                classes.intern(&ty, fs)
                            }),
                            _ => None,
                        })
                    }
                    _ => None,
                })
                .unwrap_or(-1);
            let mut xs = vec![kw(":a"), Value::i64(id), Value::i64(class_idx)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "RootJoinNode" => {
            let mut xs = vec![kw(":j"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "HashJoinNode" => {
            let mut xs = vec![kw(":h"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "ProductionNode" => {
            let name = match node_record(node) {
                Some((_, sf)) => match &sf[1] {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                },
                None => String::new(),
            };
            pv([kw(":p"), Value::i64(id), Value::String(Arc::new(name))])
        }
        "TestNode" => {
            let mut xs = vec![kw(":t"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "NegationNode" => {
            let aid = match node_record(node) {
                Some((_, sf)) => match &sf[1] {
                    Value::i64(n) => *n,
                    _ => -1,
                },
                None => -1,
            };
            let mut xs = vec![kw(":n"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "ExistsNode" => {
            let aid = match node_record(node) {
                Some((_, sf)) => match &sf[1] {
                    Value::i64(n) => *n,
                    _ => -1,
                },
                None => -1,
            };
            let mut xs = vec![kw(":e"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "AccumulateNode" => {
            let (var, aid) = match node_record(node) {
                Some((_, sf)) => {
                    let var = match &sf[1] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => String::new(),
                    };
                    let aid = match &sf[3] {
                        Value::i64(n) => *n,
                        _ => -1,
                    };
                    (var, aid)
                }
                None => (String::new(), -1),
            };
            let mut xs = vec![
                kw(":acc"),
                Value::i64(id),
                Value::String(Arc::new(var)),
                Value::i64(aid),
            ];
            xs.extend(pack_children(node));
            pv(xs)
        }
        "QueryNode" => {
            let (name, params) = match node_record(node) {
                Some((_, sf)) => {
                    let name = match &sf[1] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => String::new(),
                    };
                    let params = match &sf[2] {
                        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
                        _ => vec![],
                    };
                    (name, params)
                }
                None => (String::new(), vec![]),
            };
            let mut xs = vec![kw(":q"), Value::i64(id), Value::String(Arc::new(name))];
            xs.extend(params);
            pv(xs)
        }
        _ => pv([kw(":x"), Value::i64(id)]),
    }
}

fn unpack_i64s(items: &[Value], skip: usize) -> Result<Vec<i64>, EvalBreak> {
    let mut out = Vec::new();
    for x in items.iter().skip(skip) {
        out.push(expect_i64(x, IMPORT_OP)?);
    }
    Ok(out)
}

fn i64_pv(ids: &[i64]) -> Value {
    let mut v = rpds::VectorSync::new_sync();
    for id in ids {
        v.push_back_mut(Value::i64(*id));
    }
    Value::wat__core__PersistentVector(v)
}

type UnpackedNode = (i64, Value, Option<(String, i64)>);

fn unpack_node(v: &Value) -> Result<UnpackedNode, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP)?;
    let tag = expect_kw(items.first().unwrap(), IMPORT_OP)?;
    match tag {
        ":a" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let class_idx = expect_i64(items.get(2).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 3)?;
            let rec = record(
                "wat::rete::AlphaNode",
                ALPHA_FIELDS,
                vec![Value::i64(id), empty_pv(), i64_pv(&kids)],
            );
            let class = if class_idx >= 0 {
                Some((String::new(), class_idx))
            } else {
                None
            };
            Ok((id, rec, class))
        }
        ":j" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 2)?;
            let rec = record(
                "wat::rete::RootJoinNode",
                ROOT_FIELDS,
                vec![Value::i64(id), i64_pv(&kids), empty_pv()],
            );
            Ok((id, rec, None))
        }
        ":h" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 2)?;
            let rec = record(
                "wat::rete::HashJoinNode",
                HASH_FIELDS,
                vec![Value::i64(id), i64_pv(&kids), empty_pv()],
            );
            Ok((id, rec, None))
        }
        ":p" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let name = expect_str(items.get(2).unwrap(), IMPORT_OP)?.to_string();
            let rec = record(
                "wat::rete::ProductionNode",
                PROD_FIELDS,
                vec![Value::i64(id), Value::String(Arc::new(name))],
            );
            Ok((id, rec, None))
        }
        ":t" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 2)?;
            let rec = record(
                "wat::rete::TestNode",
                TEST_FIELDS,
                vec![Value::i64(id), dummy_ast(), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":n" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let aid = expect_i64(items.get(2).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 3)?;
            let rec = record(
                "wat::rete::NegationNode",
                NEG_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":e" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let aid = expect_i64(items.get(2).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 3)?;
            let rec = record(
                "wat::rete::ExistsNode",
                EXISTS_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":acc" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let var = expect_str(items.get(2).unwrap(), IMPORT_OP)?.to_string();
            let aid = expect_i64(items.get(3).unwrap(), IMPORT_OP)?;
            let kids = unpack_i64s(&items, 4)?;
            let rec = record(
                "wat::rete::AccumulateNode",
                ACC_FIELDS,
                vec![
                    Value::i64(id),
                    Value::String(Arc::new(var)),
                    dummy_ast(),
                    Value::i64(aid),
                    i64_pv(&kids),
                ],
            );
            Ok((id, rec, None))
        }
        ":q" => {
            let id = expect_i64(items.get(1).unwrap(), IMPORT_OP)?;
            let name = expect_str(items.get(2).unwrap(), IMPORT_OP)?.to_string();
            let mut params = rpds::VectorSync::new_sync();
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
        other => Err(malformed(IMPORT_OP, format!("unknown node {other}"))),
    }
}

// ── Session field readers ────────────────────────────────────────────────────

fn session_network_rules(session: &Value) -> Result<(&Value, &Value), EvalBreak> {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct && a.class.as_ref() == "wat::rete::Session" => {
            let sf = a.fields.as_slice();
            Ok((&sf[0], &sf[1]))
        }
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn map_i64<V>(m: &HashMap<i64, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    pv(m.iter().map(|(k, v)| pv([Value::i64(*k), f(v)])))
}

fn map_str<V>(m: &HashMap<String, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    pv(m.iter()
        .map(|(k, v)| pv([Value::String(Arc::new(k.clone())), f(v)])))
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
    let (network, rules) = session_network_rules(&session)?;
    let arm = rete_arm_get_or_build(network, rules, sym)?;
    let mut classes = ClassIntern::new();
    let mut nodes = Vec::new();
    for id in sorted_node_ids(network) {
        if let Some(node) = get_node(network, id) {
            nodes.push(pack_node(node, &mut classes, network, sym));
        }
    }
    let conds = map_i64(&arm.compiled_conds, pack_compiled_cond);
    let drivers = map_i64(&arm.compiled_drivers, pack_driver);
    let progs = map_i64(&arm.compiled_wheres, pack_prog);
    let folds = map_i64(&arm.compiled_acc_folds, pack_fold);
    let rhs = map_str(&arm.compiled_rhs, |items| {
        pv(items.iter().map(pack_rhs))
    });
    let deps = pack_deps(&rule_deps_from_rules(rules, sym));
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
    pv(deps.iter().map(|(name, produced, negated, consumed, bag)| {
        pv([
            Value::String(Arc::new(name.clone())),
            pv(produced.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(negated.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(consumed.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(bag.iter().map(|s| Value::String(Arc::new(s.clone())))),
        ])
    }))
}

fn unpack_string_list(v: &Value) -> Result<Vec<String>, EvalBreak> {
    let xs = expect_seq(v, IMPORT_OP)?;
    let mut out = Vec::new();
    for x in xs {
        out.push(expect_str(&x, IMPORT_OP)?.to_string());
    }
    Ok(out)
}

fn unpack_deps(v: &Value) -> Result<Vec<RuleDep>, EvalBreak> {
    let mut out = Vec::new();
    for row in expect_seq(v, IMPORT_OP)? {
        let p = expect_seq(&row, IMPORT_OP)?;
        if p.len() < 4 {
            return Err(malformed(IMPORT_OP, "deps row needs name + 3 lists"));
        }
        let bag = if p.len() >= 5 {
            unpack_string_list(&p[4])?
        } else {
            Vec::new()
        };
        out.push((
            expect_str(&p[0], IMPORT_OP)?.to_string(),
            unpack_string_list(&p[1])?,
            unpack_string_list(&p[2])?,
            unpack_string_list(&p[3])?,
            bag,
        ));
    }
    Ok(out)
}

/// `(:wat::rete::import <export>) -> :wat::rete::Session`
///
/// Slim topology, interned arm, empty facts. Fire does not lower.
/// Stratify schedule is `:deps` (produced / negated / consumed class names).
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
    import_export(&export, sym)
}

fn import_export(export: &Value, _sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let agg = match export {
        Value::Aggregate(a) if a.nature != Nature::Struct && a.class.as_ref() == "wat::rete::Export" => a,
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: IMPORT_OP.into(),
                    expected: ":wat::rete::Export",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let sf = agg.fields.as_slice();
    if sf.len() < 10 {
        return Err(malformed(IMPORT_OP, "Export missing fields"));
    }
    let v = expect_i64(&sf[0], IMPORT_OP)?;
    if v != FORMAT_V {
        return Err(malformed(
            IMPORT_OP,
            format!("unsupported Export version {v}"),
        ));
    }
    let stored_abi = expect_str(&sf[1], IMPORT_OP)?;
    let classes_pv = expect_seq(&sf[2], IMPORT_OP)?;
    let mut classes = Vec::new();
    for c in classes_pv.iter() {
        classes.push(expect_str(c, IMPORT_OP)?.to_string());
    }
    let fields_pv = expect_seq(&sf[3], IMPORT_OP)?;
    let mut fields = Vec::new();
    for row in fields_pv.iter() {
        let rp = expect_seq(row, IMPORT_OP)?;
        let mut fs = Vec::new();
        for f in rp.iter() {
            fs.push(expect_str(f, IMPORT_OP)?.to_string());
        }
        fields.push(fs);
    }
    let expect_abi = abi_of(&classes, &fields);
    if stored_abi != expect_abi {
        return Err(malformed(
            IMPORT_OP,
            "ABI mismatch — export is from a different TypeEnv/RETE_OPS",
        ));
    }

    let nodes_pv = expect_seq(&sf[4], IMPORT_OP)?;
    let mut network_pairs = Vec::new();
    let mut alpha_by_type: HashMap<String, Vec<i64>> = HashMap::new();
    let mut alpha_class: HashMap<i64, String> = HashMap::new();
    let mut max_id = 0i64;
    for n in nodes_pv.iter() {
        let (id, rec, class_hint) = unpack_node(n)?;
        if id > max_id {
            max_id = id;
        }
        if let Some((_, class_idx)) = class_hint {
            if class_idx >= 0 {
                if let Some(name) = classes.get(class_idx as usize) {
                    alpha_by_type.entry(name.clone()).or_default().push(id);
                    alpha_class.insert(id, name.clone());
                }
            }
        }
        network_pairs.push((Value::i64(id), rec));
    }
    let network = Value::wat__core__PersistentMap(PMap::from_pairs(network_pairs));

    let mut compiled_conds = HashMap::new();
    for pair in expect_seq(&sf[5], IMPORT_OP)? {
        let p = expect_seq(&pair, IMPORT_OP)?;
        compiled_conds.insert(
            expect_i64(&p[0], IMPORT_OP)?,
            unpack_compiled_cond(&p[1])?,
        );
    }
    let mut compiled_drivers = HashMap::new();
    for pair in expect_seq(&sf[6], IMPORT_OP)? {
        let p = expect_seq(&pair, IMPORT_OP)?;
        compiled_drivers.insert(expect_i64(&p[0], IMPORT_OP)?, unpack_driver(&p[1])?);
    }
    let mut compiled_wheres = HashMap::new();
    for pair in expect_seq(&sf[7], IMPORT_OP)? {
        let p = expect_seq(&pair, IMPORT_OP)?;
        compiled_wheres.insert(expect_i64(&p[0], IMPORT_OP)?, unpack_prog(&p[1])?);
    }
    let mut compiled_acc_folds = HashMap::new();
    for pair in expect_seq(&sf[8], IMPORT_OP)? {
        let p = expect_seq(&pair, IMPORT_OP)?;
        compiled_acc_folds.insert(expect_i64(&p[0], IMPORT_OP)?, unpack_fold(&p[1])?);
    }
    let mut compiled_rhs = HashMap::new();
    for pair in expect_seq(&sf[9], IMPORT_OP)? {
        let p = expect_seq(&pair, IMPORT_OP)?;
        let name = expect_str(&p[0], IMPORT_OP)?.to_string();
        let items = expect_seq(&p[1], IMPORT_OP)?;
        let mut rs = Vec::new();
        for x in &items {
            rs.push(unpack_rhs(x)?);
        }
        compiled_rhs.insert(name, rs);
    }

    let rule_deps = if sf.len() > 10 {
        unpack_deps(&sf[10])?
    } else {
        Vec::new()
    };

    let node_ids = sorted_node_ids(&network);
    let mut feeding_alpha_of: HashMap<i64, i64> = HashMap::new();
    let mut parents_of: HashMap<i64, Vec<i64>> = HashMap::new();
    for node_id in &node_ids {
        let Some(node) = get_node(&network, *node_id) else {
            continue;
        };
        let is_alpha = kind_of(node) == "AlphaNode";
        for child in node_children(node) {
            if is_alpha {
                feeding_alpha_of.insert(child, *node_id);
            } else {
                parents_of.entry(child).or_default().push(*node_id);
            }
        }
    }
    let mut beta_readers = HashSet::new();
    for node_id in &node_ids {
        let Some(node) = get_node(&network, *node_id) else {
            continue;
        };
        for child in node_children(node) {
            let child_kind = get_node(&network, child).map(kind_of).unwrap_or("");
            if child_kind == "HashJoinNode" || child_kind == "QueryNode" {
                beta_readers.insert(*node_id);
                break;
            }
        }
    }
    let compiled_max_slots = compiled_conds.values().map(|c| c.n_slots()).max().unwrap_or(0);
    let alpha_tree = AlphaTree::unpruned(&alpha_by_type);
    let where_tree = crate::rete::where_tree::WhereTree::build(&compiled_wheres);
    let kind_ids = kind_id_lists(&network, &node_ids);
    let joins_fed_by = invert_feeding_alpha(&feeding_alpha_of);
    let arm = Arc::new(ReteArm {
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
        alpha_class,
    });
    if let Some(id) = network_identity(&network) {
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
