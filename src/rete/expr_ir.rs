//! The one expression core — `DESIGN-STONE-the-one-expression-core.md`.
//!
//! Nested `Expr` DAG. No `Interp` arm. `lower()` is total or it refuses.
//! First consumer: `where` (rule-compile refuse + native filter exec).
//! Oracle remains `eval_test_core`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::rete::matcher::{compare_values, Bindings};
use crate::rete::vocabulary::{resolve_core_name, rete_op_for, OpClass, RETE_OPS};
use crate::runtime::{
    EvalBreak, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use crate::types::TypeDef;
use crate::value::value::AggregateValue;

/// A lowered rete expression. Children are nested (builder: "matches the precedent").
#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Lit(Value),
    Slot(u16),
    /// Strict rete-vocabulary call. `op` indexes `RETE_OPS`.
    Call {
        op: u16,
        args: Box<[Expr]>,
    },
    CallFallback {
        op: u16,
        args: Box<[Expr]>,
        fallback: Box<Expr>,
    },
    /// Named rete-defn (or a literal `fn` compiled as its own program).
    CallUser {
        program: Arc<Program>,
        args: Box<[Expr]>,
    },
    Field {
        recv: Box<Expr>,
        idx: usize,
    },
    /// `(:Type field…)` / kwargs — a fact constructor inside a rete-defn body
    /// (fn-headed `:then` fallbacks, e.g. `(:tf::Rate :count 0)`).
    Construct {
        class: String,
        names: Arc<Vec<String>>,
        fields: Box<[Expr]>,
    },
    If {
        cond: Box<Expr>,
        then_: Box<Expr>,
        else_: Box<Expr>,
    },
    And(Box<[Expr]>),
    Or(Box<[Expr]>),
    Let {
        binds: Box<[(u16, Expr)]>,
        body: Box<Expr>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Box<[(Pat, Expr)]>,
    },
}

/// Closed match patterns. Map-destructure is a `LowerError`, not an arm.
#[derive(Clone, Debug)]
pub(crate) enum Pat {
    Lit(Value),
    Wild,
    Bind(u16),
    /// `(Some p)` / `(Ok p)` / `(Err p)` / unit `:None`.
    Variant {
        name: String,
        payload: Option<Box<Pat>>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Program {
    pub(crate) frame_len: u16,
    pub(crate) root: Expr,
    /// Token-binding key (`Value::String "?x"`) → slot. Prologue for `where`.
    pub(crate) reads: Arc<[(Value, u16)]>,
    /// Parameter slots in declaration order. Empty for a `where` program.
    /// A literal `fn` compiled inside a `where` shares the parent's slot
    /// numbering; foldl writes `[acc, x]` here rather than at 0..n.
    pub(crate) params: Box<[u16]>,
    /// Slot → binder name (`?x`, `acc`, …). Unbound-slot errors use this so
    /// `exec` and `eval_inner` raise the same `UnboundSymbol` kind (flip 4
    /// RHS differential). Missing entries render as `slot N`.
    pub(crate) names: Box<[Option<Arc<str>>]>,
    /// Source span of the original expr — exec errors name this, not rust-caller.
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) enum LowerError {
    Unsupported { span: Span, reason: String },
    NonLexicalCallee { span: Span },
    Unbound { span: Span, name: String },
}

impl LowerError {
    pub(crate) fn into_eval(self) -> EvalBreak {
        match self {
            LowerError::Unsupported { span, reason } => RuntimeError::new(
                span,
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::lower".into(),
                    reason,
                },
            )
            .into(),
            LowerError::NonLexicalCallee { span } => RuntimeError::new(
                span,
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::lower".into(),
                    reason: "HOF fn-arg must be a literal fn or a named rete-defn (callee visible in the AST)".into(),
                },
            )
            .into(),
            LowerError::Unbound { span, name } => RuntimeError::new(
                span,
                RuntimeErrorKind::UnboundSymbol(name),
            )
            .into(),
        }
    }
}

struct LowerCx<'a> {
    sym: &'a SymbolTable,
    /// `?var` / let / fn param name → slot.
    slots: HashMap<String, u16>,
    next: u16,
    /// When true, a call-position that is not literal `fn` / named rete-defn is NonLexicalCallee.
    hof_fn_pos: bool,
}

impl<'a> LowerCx<'a> {
    fn slot(&mut self, name: &str) -> u16 {
        if let Some(&s) = self.slots.get(name) {
            return s;
        }
        let s = self.next;
        self.next += 1;
        self.slots.insert(name.to_string(), s);
        s
    }
}

pub(crate) fn lower(expr: &WatAST, sym: &SymbolTable) -> Result<Program, LowerError> {
    let mut cx = LowerCx {
        sym,
        slots: HashMap::new(),
        next: 0,
        hof_fn_pos: false,
    };
    let root = lower_expr(expr, &mut cx)?;
    let mut reads: Vec<(Value, u16)> = cx
        .slots
        .iter()
        .filter(|(n, _)| n.starts_with('?'))
        .map(|(n, &s)| (Value::String(Arc::new(n.clone())), s))
        .collect();
    reads.sort_by(|a, b| match (&a.0, &b.0) {
        (Value::String(x), Value::String(y)) => x.as_ref().cmp(y.as_ref()),
        _ => std::cmp::Ordering::Equal,
    });
    Ok(Program {
        frame_len: cx.next,
        root,
        reads: Arc::from(reads),
        params: Box::from([]),
        names: slot_names(&cx),
        span: expr.span().clone(),
    })
}

fn slot_names(cx: &LowerCx<'_>) -> Box<[Option<Arc<str>>]> {
    let mut names = vec![None; cx.next as usize];
    for (name, &slot) in &cx.slots {
        if let Some(slot_name) = names.get_mut(slot as usize) {
            *slot_name = Some(Arc::from(name.as_str()));
        }
    }
    names.into_boxed_slice()
}

fn lower_expr(ast: &WatAST, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    // Consume the HOF-callee flag at THIS node only. A literal `fn` body's
    // symbols (`acc` in `(and acc …)`) are ordinary binders, not callees.
    let hof = cx.hof_fn_pos;
    cx.hof_fn_pos = false;
    if hof {
        return lower_hof_callee(ast, cx);
    }
    match ast {
        WatAST::IntLit(n, _) => Ok(Expr::Lit(Value::i64(*n))),
        WatAST::FloatLit(n, _) => Ok(Expr::Lit(Value::f64(*n))),
        WatAST::BoolLit(b, _) => Ok(Expr::Lit(Value::bool(*b))),
        WatAST::StringLit(s, _) => Ok(Expr::Lit(Value::String(Arc::new(s.clone())))),
        WatAST::Keyword(k, _) => Ok(Expr::Lit(keyword_value(k, cx.sym))),
        WatAST::NilLit(_) => Ok(Expr::Lit(Value::Unit)),
        WatAST::Symbol(id, span) => {
            let name = id.as_str();
            if name.starts_with('?') || cx.slots.contains_key(name) {
                return Ok(Expr::Slot(cx.slot(name)));
            }
            Err(LowerError::Unbound {
                span: span.clone(),
                name: name.to_string(),
            })
        }
        WatAST::List(items, span) => lower_list(items, span, cx),
        WatAST::Vector(elems, _) => {
            let mut out = Vec::with_capacity(elems.len());
            for e in elems {
                match lower_expr(e, cx)? {
                    Expr::Lit(v) => out.push(v),
                    _ => {
                        return Err(LowerError::Unsupported {
                            span: e.span().clone(),
                            reason: "vector literal in a where must be constant".into(),
                        });
                    }
                }
            }
            Ok(Expr::Lit(Value::wat__core__PersistentVector(
                out.into_iter().collect(),
            )))
        }
        other => Err(LowerError::Unsupported {
            span: other.span().clone(),
            reason: format!("cannot lower {}", other.span()),
        }),
    }
}

fn keyword_value(k: &str, _sym: &SymbolTable) -> Value {
    Value::wat__core__keyword(Arc::new(k.to_string()))
}

fn lower_hof_callee(ast: &WatAST, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    match ast {
        WatAST::List(items, span) => {
            let head = match items.first() {
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                _ => {
                    return Err(LowerError::NonLexicalCallee {
                        span: ast.span().clone(),
                    });
                }
            };
            if resolve_core_name(head) == ":wat::core::fn" {
                return lower_fn(items, span, cx);
            }
            Err(LowerError::NonLexicalCallee { span: span.clone() })
        }
        WatAST::Keyword(k, span) => {
            if let Some(func) = cx.sym.get(k) {
                if func.rete.is_some() {
                    if let FunctionBody::Wat(body) = &func.body {
                        let program = lower_rete_defn(func.as_ref(), body, cx.sym)?;
                        return Ok(Expr::CallUser {
                            program,
                            args: Box::from([]),
                        });
                    }
                }
            }
            Err(LowerError::NonLexicalCallee { span: span.clone() })
        }
        other => Err(LowerError::NonLexicalCallee {
            span: other.span().clone(),
        }),
    }
}

fn lower_list(items: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        Some(other) => {
            return Err(LowerError::Unsupported {
                span: other.span().clone(),
                reason: "call head must be a keyword".into(),
            });
        }
        None => {
            return Err(LowerError::Unsupported {
                span: span.clone(),
                reason: "empty list".into(),
            });
        }
    };
    let core = resolve_core_name(head);
    if core == ":wat::core::and" {
        return Ok(Expr::And(lower_args(&items[1..], cx)?));
    }
    if core == ":wat::core::or" {
        return Ok(Expr::Or(lower_args(&items[1..], cx)?));
    }
    if core == ":wat::core::if" {
        if items.len() != 4 {
            return Err(LowerError::Unsupported {
                span: span.clone(),
                reason: "if takes cond then else".into(),
            });
        }
        return Ok(Expr::If {
            cond: Box::new(lower_expr(&items[1], cx)?),
            then_: Box::new(lower_expr(&items[2], cx)?),
            else_: Box::new(lower_expr(&items[3], cx)?),
        });
    }
    if core == ":wat::core::let" {
        return lower_let(&items[1..], span, cx);
    }
    if core == ":wat::core::match" {
        return lower_match(&items[1..], span, cx);
    }
    if core == ":wat::core::fn" {
        return lower_fn(items, span, cx);
    }
    if core == ":wat::core::quote" || core == ":wat::core::quasiquote" {
        return Err(LowerError::Unsupported {
            span: span.clone(),
            reason: "quote is data, not a where expression".into(),
        });
    }
    if core == ":wat::core::kwargs-construct" || core == ":wat::core::aggregate-new" {
        let type_kw = match items.get(1) {
            Some(WatAST::Keyword(k, _)) => k.as_str(),
            _ => {
                return Err(LowerError::Unsupported {
                    span: span.clone(),
                    reason: "constructor needs a type keyword".into(),
                });
            }
        };
        return lower_construct(type_kw, &items[1..], span, cx)?
            .ok_or_else(|| LowerError::Unsupported {
                span: span.clone(),
                reason: format!("unknown aggregate {type_kw}"),
            });
    }

    // Vocabulary rows win over `:Type/field` — `PersistentVector/length` contains `/`
    // but is a rete op, not a record accessor.
    if let Some(row) = rete_op_for(head) {
        let op = RETE_OPS
            .iter()
            .position(|r| r.rete_name == row.rete_name)
            .expect("row is in RETE_OPS") as u16;
        let hof = matches!(
            row.core_name,
            ":wat::core::foldl"
                | ":wat::core::foldr"
                | ":wat::core::map"
                | ":wat::core::filter"
                | ":wat::core::reduce"
        );
        if row.class == OpClass::Fallback {
            return lower_fallback(op, &items[1..], span, cx, hof);
        }
        let args = lower_call_args(&items[1..], cx, hof)?;
        return Ok(Expr::Call { op, args });
    }

    // Accessor `:ns::Type/field` — class and field are in the head.
    if let Some((cls, field)) = split_accessor(head) {
        if items.len() != 2 {
            return Err(LowerError::Unsupported {
                span: span.clone(),
                reason: "accessor takes one receiver".into(),
            });
        }
        let idx = field_index(cx.sym, cls, field).ok_or_else(|| LowerError::Unsupported {
            span: span.clone(),
            reason: format!("unknown accessor {head}"),
        })?;
        return Ok(Expr::Field {
            recv: Box::new(lower_expr(&items[1], cx)?),
            idx,
        });
    }

    if let Some(e) = lower_construct(head, items, span, cx)? {
        return Ok(e);
    }

    // Named rete-defn.
    if let Some(func) = cx.sym.get(head) {
        if func.rete.is_some() {
            if let FunctionBody::Wat(body) = &func.body {
                let program = lower_rete_defn(func.as_ref(), body, cx.sym)?;
                let args = lower_args(&items[1..], cx)?;
                return Ok(Expr::CallUser { program, args });
            }
        }
    }

    Err(LowerError::Unsupported {
        span: span.clone(),
        reason: format!("cannot lower head {head}"),
    })
}

fn lower_call_args(
    args: &[WatAST],
    cx: &mut LowerCx,
    hof: bool,
) -> Result<Box<[Expr]>, LowerError> {
    let mut out = Vec::with_capacity(args.len());
    for (i, a) in args.iter().enumerate() {
        let prev = cx.hof_fn_pos;
        cx.hof_fn_pos = hof && i == 0;
        let e = lower_expr(a, cx);
        cx.hof_fn_pos = prev;
        out.push(e?);
    }
    Ok(out.into_boxed_slice())
}

fn lower_args(args: &[WatAST], cx: &mut LowerCx) -> Result<Box<[Expr]>, LowerError> {
    let mut out = Vec::with_capacity(args.len());
    for a in args {
        out.push(lower_expr(a, cx)?);
    }
    Ok(out.into_boxed_slice())
}

fn lower_fallback(
    op: u16,
    args: &[WatAST],
    span: &Span,
    cx: &mut LowerCx,
    hof: bool,
) -> Result<Expr, LowerError> {
    let row = &RETE_OPS[op as usize];
    let total = row.params.len();
    if args.len() != total {
        return Err(LowerError::Unsupported {
            span: span.clone(),
            reason: format!("{} wants {total} args", row.rete_name),
        });
    }
    let marker = total.saturating_sub(2);
    match &args.get(marker) {
        Some(WatAST::Keyword(k, _)) if k == ":undefined" => {}
        _ => {
            return Err(LowerError::Unsupported {
                span: span.clone(),
                reason: "fallback op requires literal :undefined".into(),
            });
        }
    }
    let real = lower_call_args(&args[..marker], cx, hof)?;
    let fallback = Box::new(lower_expr(&args[total - 1], cx)?);
    Ok(Expr::CallFallback {
        op,
        args: real,
        fallback,
    })
}

fn lower_let(args: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    let (binds_ast, body) = match args {
        [binds, body] => (binds, body),
        _ => {
            return Err(LowerError::Unsupported {
                span: span.clone(),
                reason: "let takes [binds] body".into(),
            });
        }
    };
    let pairs = match binds_ast {
        WatAST::Vector(v, _) => v.as_slice(),
        _ => {
            return Err(LowerError::Unsupported {
                span: binds_ast.span().clone(),
                reason: "let binds must be a vector".into(),
            });
        }
    };
    let mut binds = Vec::new();
    let mut i = 0;
    while i + 1 < pairs.len() {
        let name = match &pairs[i] {
            WatAST::Symbol(id, _) => id.as_str().to_string(),
            other => {
                return Err(LowerError::Unsupported {
                    span: other.span().clone(),
                    reason: "let binder must be a symbol".into(),
                });
            }
        };
        let val = lower_expr(&pairs[i + 1], cx)?;
        let slot = cx.slot(&name);
        binds.push((slot, val));
        i += 2;
    }
    Ok(Expr::Let {
        binds: binds.into_boxed_slice(),
        body: Box::new(lower_expr(body, cx)?),
    })
}

fn lower_match(args: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    if args.is_empty() {
        return Err(LowerError::Unsupported {
            span: span.clone(),
            reason: "match needs a scrutinee".into(),
        });
    }
    let scrutinee = Box::new(lower_expr(&args[0], cx)?);
    let mut arms = Vec::new();
    for arm in &args[1..] {
        match arm {
            WatAST::List(parts, _) if !parts.is_empty() => {
                let pat = lower_pat(&parts[0], cx)?;
                let body = if parts.len() == 1 {
                    Expr::Lit(Value::Unit)
                } else {
                    lower_expr(&parts[1], cx)?
                };
                arms.push((pat, body));
            }
            WatAST::Keyword(k, _) => {
                arms.push((Pat::Lit(keyword_value(k, cx.sym)), Expr::Lit(Value::Unit)));
            }
            other => {
                return Err(LowerError::Unsupported {
                    span: other.span().clone(),
                    reason: "malformed match arm".into(),
                });
            }
        }
    }
    Ok(Expr::Match {
        scrutinee,
        arms: arms.into_boxed_slice(),
    })
}

fn lower_pat(ast: &WatAST, cx: &mut LowerCx) -> Result<Pat, LowerError> {
    match ast {
        WatAST::IntLit(n, _) => Ok(Pat::Lit(Value::i64(*n))),
        WatAST::FloatLit(n, _) => Ok(Pat::Lit(Value::f64(*n))),
        WatAST::BoolLit(b, _) => Ok(Pat::Lit(Value::bool(*b))),
        WatAST::StringLit(s, _) => Ok(Pat::Lit(Value::String(Arc::new(s.clone())))),
        WatAST::Keyword(k, _) => {
            if let Some(name) = option_result_tag(k) {
                return Ok(Pat::Variant {
                    name,
                    payload: None,
                });
            }
            // Unit enum variant `:ns::Type::Variant` — `try_match_pattern`
            // composes `type_path::variant_name` against this keyword.
            if k.starts_with(':') && k.contains("::") {
                return Ok(Pat::Variant {
                    name: k.clone(),
                    payload: None,
                });
            }
            Ok(Pat::Lit(keyword_value(k, cx.sym)))
        }
        WatAST::Symbol(id, _) if id.as_str() == "_" => Ok(Pat::Wild),
        WatAST::Symbol(id, _) => Ok(Pat::Bind(cx.slot(id.as_str()))),
        WatAST::List(items, span) if !items.is_empty() => {
            let tag = match &items[0] {
                WatAST::Keyword(k, _) => k.as_str(),
                _ => {
                    return Err(LowerError::Unsupported {
                        span: span.clone(),
                        reason: "match list pattern head must be a keyword".into(),
                    });
                }
            };
            if tag.contains('{') || matches!(items[0], WatAST::Map(_, _)) {
                return Err(LowerError::Unsupported {
                    span: span.clone(),
                    reason: "match map-destructure is not lowered in v1".into(),
                });
            }
            let name = option_result_tag(tag).unwrap_or_else(|| tag.to_string());
            let payload = if items.len() > 1 {
                Some(Box::new(lower_pat(&items[1], cx)?))
            } else {
                None
            };
            Ok(Pat::Variant { name, payload })
        }
        WatAST::Map(_, span) => Err(LowerError::Unsupported {
            span: span.clone(),
            reason: "match map-destructure is not lowered in v1".into(),
        }),
        other => Err(LowerError::Unsupported {
            span: other.span().clone(),
            reason: "unsupported match pattern".into(),
        }),
    }
}

fn lower_fn(items: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    let arrow = items
        .iter()
        .position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->"));
    let Some(arrow) = arrow else {
        return Err(LowerError::Unsupported {
            span: span.clone(),
            reason: "fn needs ->".into(),
        });
    };
    let params_ast = items.get(1).ok_or_else(|| LowerError::Unsupported {
        span: span.clone(),
        reason: "fn needs a param vector".into(),
    })?;
    let body_forms = items.get(arrow + 2..).unwrap_or(&[]);
    let body = body_forms.last().ok_or_else(|| LowerError::Unsupported {
        span: span.clone(),
        reason: "fn needs a body".into(),
    })?;
    let mut param_slots = Vec::new();
    if let WatAST::Vector(ps, _) = params_ast {
        let mut i = 0;
        while i < ps.len() {
            match &ps[i] {
                WatAST::Symbol(id, _) => {
                    param_slots.push(cx.slot(id.as_str()));
                    i += 1;
                    if i < ps.len() && matches!(&ps[i], WatAST::Symbol(s, _) if s.as_str() == "<-")
                    {
                        i += 2; // skip <- type
                    }
                }
                _ => i += 1,
            }
        }
    }
    let body_e = lower_expr(body, cx)?;
    // A literal fn used as a value (HOF arg) is a CallUser of its own program
    // with no extra args at the construction site — foldl applies `params`.
    Ok(Expr::CallUser {
        program: Arc::new(Program {
            frame_len: cx.next,
            root: body_e,
            reads: Arc::from([]),
            params: param_slots.into_boxed_slice(),
            names: slot_names(cx),
            span: body.span().clone(),
        }),
        args: Box::from([]),
    })
}

fn lower_rete_defn(
    func: &crate::runtime::Function,
    body: &WatAST,
    sym: &SymbolTable,
) -> Result<Arc<Program>, LowerError> {
    let mut cx = LowerCx {
        sym,
        slots: HashMap::new(),
        next: 0,
        hof_fn_pos: false,
    };
    let mut params = Vec::with_capacity(func.params.len());
    for p in &func.params {
        params.push(cx.slot(p.as_str()));
    }
    let root = lower_expr(body, &mut cx)?;
    Ok(Arc::new(Program {
        frame_len: cx.next,
        root,
        reads: Arc::from([]),
        params: params.into_boxed_slice(),
        names: slot_names(&cx),
        span: body.span().clone(),
    }))
}

/// Flip 5 — lower a named rete-defn (user acc fold) onto the one core.
/// The callee is in the closed language; this is a call boundary, not a hatch.
pub(crate) fn lower_named_rete_fn(
    head: &str,
    sym: &SymbolTable,
) -> Result<Arc<Program>, LowerError> {
    let func = match sym.get(head) {
        Some(f) => f,
        None => {
            return Err(LowerError::Unsupported {
                span: crate::rust_caller_span!(),
                reason: format!("unknown rete-defn {head}"),
            });
        }
    };
    if func.rete.is_none() {
        return Err(LowerError::Unsupported {
            span: crate::rust_caller_span!(),
            reason: format!("{head} is not a rete-defn"),
        });
    }
    match &func.body {
        FunctionBody::Wat(body) => lower_rete_defn(func.as_ref(), body, sym),
        _ => Err(LowerError::Unsupported {
            span: crate::rust_caller_span!(),
            reason: format!("{head} has no wat body"),
        }),
    }
}

/// Apply a compiled rete-defn to concrete args (user acc fold: the gathered PV).
pub(crate) fn exec_call(
    program: &Program,
    args: &[Value],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    exec_program_on(program, args, None, sym, span)
}

fn option_result_tag(tag: &str) -> Option<String> {
    let last = tag
        .rsplit("::")
        .next()
        .unwrap_or(tag)
        .trim_start_matches(':');
    match last {
        "None" | "Some" | "Ok" | "Err" => Some(last.to_string()),
        _ => None,
    }
}

fn split_accessor(head: &str) -> Option<(&str, &str)> {
    let rest = head.strip_prefix(':')?;
    let (cls, field) = rest.rsplit_once('/')?;
    if field.is_empty() || cls.is_empty() {
        return None;
    }
    // Vocabulary ops use `::` not `/` (i64::>, PersistentVector/get).
    // `PersistentVector/get` is a rete row, not an accessor — rete_op_for wins first.
    Some((cls, field))
}

fn field_index(sym: &SymbolTable, class: &str, field: &str) -> Option<usize> {
    let types = sym.types()?;
    let key = if class.starts_with(':') {
        class.to_string()
    } else {
        format!(":{class}")
    };
    match types.get(&key) {
        Some(TypeDef::Aggregate(a)) => a.field_names().position(|n| n == field),
        _ => None,
    }
}

fn lower_construct(
    head: &str,
    items: &[WatAST],
    span: &Span,
    cx: &mut LowerCx<'_>,
) -> Result<Option<Expr>, LowerError> {
    let Some(types) = cx.sym.types() else {
        return Ok(None);
    };
    let Some(TypeDef::Aggregate(a)) = types.get(head) else {
        return Ok(None);
    };
    let names = a.names_arc();
    let class = head.strip_prefix(':').unwrap_or(head).to_string();
    let args = &items[1..];
    let is_kwargs = args.len() >= 2
        && args.len().is_multiple_of(2)
        && args
            .iter()
            .step_by(2)
            .all(|a| matches!(a, WatAST::Keyword(_, _)));
    let value_asts: Vec<&WatAST> = if is_kwargs {
        args.iter().skip(1).step_by(2).collect()
    } else {
        args.iter().collect()
    };
    let mut fields = Vec::with_capacity(value_asts.len());
    for v in value_asts {
        fields.push(lower_expr(v, cx)?);
    }
    if fields.len() != names.len() {
        return Err(LowerError::Unsupported {
            span: span.clone(),
            reason: format!(
                "constructor {head} wants {} fields, got {}",
                names.len(),
                fields.len()
            ),
        });
    }
    Ok(Some(Expr::Construct {
        class,
        names,
        fields: fields.into_boxed_slice(),
    }))
}

// ── exec ─────────────────────────────────────────────────────────────────────

pub(crate) fn exec_where<B: Bindings>(
    program: &Program,
    bindings: &B,
    sym: &SymbolTable,
    span: &Span,
) -> Result<bool, EvalBreak> {
    match exec_value(program, bindings, sym, span)? {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::rete::eval-test".into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Prologue (token bindings → slots) + eval. `where` requires bool;
/// `compiled_rhs` takes the `Value` as a fact field.
pub(crate) fn exec_value<B: Bindings>(
    program: &Program,
    bindings: &B,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let mut frame: Vec<Option<Value>> = vec![None; program.frame_len as usize];
    for (k, slot) in program.reads.iter() {
        if let Some(v) = bindings.get(k) {
            frame[*slot as usize] = Some(v.clone());
        }
    }
    exec(&program.root, &mut frame, &program.names, sym, span)
}

fn exec(
    e: &Expr,
    frame: &mut [Option<Value>],
    names: &[Option<Arc<str>>],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    match e {
        Expr::Lit(v) => Ok(v.clone()),
        Expr::Slot(s) => frame
            .get(*s as usize)
            .and_then(|o| o.clone())
            .ok_or_else(|| {
                let name = names
                    .get(*s as usize)
                    .and_then(|n| n.as_ref().map(|a| a.to_string()))
                    .unwrap_or_else(|| format!("slot {s}"));
                RuntimeError::new(span.clone(), RuntimeErrorKind::UnboundSymbol(name)).into()
            }),
        Expr::Field { recv, idx } => {
            let v = exec(recv, frame, names, sym, span)?;
            match v {
                Value::Aggregate(a) => a.fields.get(*idx).cloned().ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::UnknownField {
                            record_class: a.class.clone(),
                            field: format!("{idx}"),
                            available: (*a.names).clone(),
                        },
                    )
                    .into()
                }),
                other => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::rete::lower".into(),
                        expected: "record",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into()),
            }
        }
        Expr::Construct {
            class,
            names: field_names,
            fields,
        } => {
            let mut vs = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                vs.push(exec(f, frame, names, sym, span)?);
            }
            Ok(Value::Aggregate(Arc::new(AggregateValue::record(
                class.clone(),
                Arc::clone(field_names),
                Arc::new(vs),
            ))))
        }
        Expr::If { cond, then_, else_ } => match exec(cond, frame, names, sym, span)? {
            Value::bool(true) => exec(then_, frame, names, sym, span),
            Value::bool(false) => exec(else_, frame, names, sym, span),
            other => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::BadCondition {
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into()),
        },
        Expr::And(xs) => {
            let mut acc = true;
            for x in xs.iter() {
                match exec(x, frame, names, sym, span)? {
                    Value::bool(b) => {
                        if !b {
                            return Ok(Value::bool(false));
                        }
                        acc = acc && b;
                    }
                    other => {
                        return Err(RuntimeError::new(
                            span.clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: ":wat::rete::core::and".into(),
                                expected: ":wat::core::bool",
                                got: Box::new(ValueSnapshot::of(&other)),
                            },
                        )
                        .into());
                    }
                }
            }
            Ok(Value::bool(acc))
        }
        Expr::Or(xs) => {
            for x in xs.iter() {
                match exec(x, frame, names, sym, span)? {
                    Value::bool(true) => return Ok(Value::bool(true)),
                    Value::bool(false) => {}
                    other => {
                        return Err(RuntimeError::new(
                            span.clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: ":wat::rete::core::or".into(),
                                expected: ":wat::core::bool",
                                got: Box::new(ValueSnapshot::of(&other)),
                            },
                        )
                        .into());
                    }
                }
            }
            Ok(Value::bool(false))
        }
        Expr::Let { binds, body } => {
            for (slot, e) in binds.iter() {
                let v = exec(e, frame, names, sym, span)?;
                frame[*slot as usize] = Some(v);
            }
            exec(body, frame, names, sym, span)
        }
        Expr::Match { scrutinee, arms } => {
            let v = exec(scrutinee, frame, names, sym, span)?;
            for (pat, body) in arms.iter() {
                if pat_matches(pat, &v, frame) {
                    return exec(body, frame, names, sym, span);
                }
            }
            Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::PatternMatchFailed {
                    value_type: v.type_name(),
                },
            )
            .into())
        }
        Expr::Call { op, args } => {
            let row = &RETE_OPS[*op as usize];
            if row.core_name == ":wat::core::foldl" {
                return exec_foldl(args, frame, names, sym, span);
            }
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            apply_core(row.core_name, &vs, span)
        }
        Expr::CallFallback { op, args, fallback } => {
            let row = &RETE_OPS[*op as usize];
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            match apply_core(row.core_name, &vs, span) {
                Ok(Value::f64(x)) if !x.is_finite() => exec(fallback, frame, names, sym, span),
                Ok(Value::Option(opt)) => match opt.as_ref() {
                    Some(v) => Ok(v.clone()),
                    None => exec(fallback, frame, names, sym, span),
                },
                Ok(v) => Ok(v),
                Err(EvalBreak::Diagnostic(e))
                    if matches!(
                        e.kind(),
                        RuntimeErrorKind::IntegerOverflow { .. } | RuntimeErrorKind::DivisionByZero
                    ) =>
                {
                    exec(fallback, frame, names, sym, span)
                }
                Err(EvalBreak::Diagnostic(e))
                    if matches!(
                        e.kind(),
                        RuntimeErrorKind::MalformedForm { head, .. } if head.as_str() == row.core_name
                    ) =>
                {
                    exec(fallback, frame, names, sym, span)
                }
                Err(e) => Err(e),
            }
        }
        Expr::CallUser { program, args } => {
            if args.is_empty() {
                // Literal fn value — foldl applies it via exec_foldl.
                return exec_program_on(program, &[], None, sym, span);
            }
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            exec_program_on(program, &vs, None, sym, span)
        }
    }
}

fn exec_program_on(
    program: &Program,
    args: &[Value],
    parent: Option<&[Option<Value>]>,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let n = (program.frame_len as usize).max(parent.map(|p| p.len()).unwrap_or(0));
    let mut inner: Vec<Option<Value>> = vec![None; n];
    if let Some(p) = parent {
        for (i, v) in p.iter().enumerate() {
            inner[i] = v.clone();
        }
    }
    for (i, v) in args.iter().enumerate() {
        if let Some(&slot) = program.params.get(i) {
            let idx = slot as usize;
            if idx >= inner.len() {
                inner.resize(idx + 1, None);
            }
            inner[idx] = Some(v.clone());
        } else if i < inner.len() {
            inner[i] = Some(v.clone());
        }
    }
    exec(&program.root, &mut inner, &program.names, sym, span)
}

fn exec_foldl(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[Option<Arc<str>>],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::foldl".into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    let program = match &args[0] {
        Expr::CallUser { program, .. } => Arc::clone(program),
        _ => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::foldl".into(),
                    reason: "fn-arg must be a compiled fn".into(),
                },
            )
            .into());
        }
    };
    let mut acc = exec(&args[1], frame, names, sym, span)?;
    let coll = exec(&args[2], frame, names, sym, span)?;
    let items: Vec<Value> = match &coll {
        Value::Vec(xs) => xs.iter().cloned().collect(),
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        Value::wat__core__List(xs) => xs.iter().cloned().collect(),
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::foldl".into(),
                    expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    for x in items {
        acc = exec_program_on(&program, &[acc.clone(), x], Some(frame), sym, span)?;
    }
    Ok(acc)
}

fn pat_matches(pat: &Pat, v: &Value, frame: &mut [Option<Value>]) -> bool {
    match pat {
        Pat::Wild => true,
        Pat::Bind(s) => {
            frame[*s as usize] = Some(v.clone());
            true
        }
        Pat::Lit(lit) => v == lit,
        Pat::Variant { name, payload } => match v {
            Value::Option(opt) => match (name.as_str(), opt.as_ref()) {
                ("None", None) => payload.is_none(),
                ("Some", Some(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame),
                    None => true,
                },
                _ => false,
            },
            Value::Result(r) => match (name.as_str(), r.as_ref()) {
                ("Ok", Ok(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame),
                    None => true,
                },
                ("Err", Err(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame),
                    None => true,
                },
                _ => false,
            },
            Value::Enum(e) => {
                let composed = format!("{}::{}", e.type_path, e.variant_name);
                let last = name
                    .rsplit("::")
                    .next()
                    .unwrap_or(name)
                    .trim_start_matches(':');
                if composed != *name && e.variant_name != *name && e.variant_name != last {
                    return false;
                }
                match payload {
                    None => e.fields.is_empty(),
                    Some(p) => e.fields.first().is_some_and(|f| pat_matches(p, f, frame)),
                }
            }
            _ => false,
        },
    }
}

fn apply_core(core: &str, args: &[Value], span: &Span) -> Result<Value, EvalBreak> {
    match (core, args) {
        (":wat::core::=", [a, b]) | (":wat::core::enum::=", [a, b]) => Ok(Value::bool(a == b)),
        (":wat::core::not=", [a, b]) => Ok(Value::bool(a != b)),
        (":wat::core::i64::>", [a, b]) | (":wat::core::>", [a, b]) => {
            ord(a, b, span, |o| o.is_gt())
        }
        (":wat::core::i64::<", [a, b]) | (":wat::core::<", [a, b]) => {
            ord(a, b, span, |o| o.is_lt())
        }
        (":wat::core::i64::>=", [a, b]) | (":wat::core::>=", [a, b]) => {
            ord(a, b, span, |o| !o.is_lt())
        }
        (":wat::core::i64::<=", [a, b]) | (":wat::core::<=", [a, b]) => {
            ord(a, b, span, |o| !o.is_gt())
        }
        (":wat::core::i64::=", [a, b]) => Ok(Value::bool(a == b)),
        (":wat::core::i64::not=", [a, b]) => Ok(Value::bool(a != b)),
        (":wat::core::string::=", [a, b]) => Ok(Value::bool(a == b)),
        (":wat::core::string::not=", [a, b]) => Ok(Value::bool(a != b)),
        (":wat::core::string::length", [Value::String(s)]) => {
            Ok(Value::i64(s.chars().count() as i64))
        }
        (":wat::core::string::starts-with?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.starts_with(p.as_str())))
        }
        (":wat::core::string::ends-with?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.ends_with(p.as_str())))
        }
        (":wat::core::string::contains?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.contains(p.as_str())))
        }
        (":wat::core::f64::>", [a, b]) => ord(a, b, span, |o| o.is_gt()),
        (":wat::core::f64::<", [a, b]) => ord(a, b, span, |o| o.is_lt()),
        (":wat::core::not", [Value::bool(b)]) => Ok(Value::bool(!*b)),
        (":wat::core::i64::+", [Value::i64(a), Value::i64(b)]) => match a.checked_add(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "+".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (":wat::core::i64::-", [Value::i64(a), Value::i64(b)]) => match a.checked_sub(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "-".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (":wat::core::i64::*", [Value::i64(a), Value::i64(b)]) => match a.checked_mul(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "*".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (":wat::core::i64::/", [Value::i64(a), Value::i64(b)])
        | (":wat::core::i64::quot", [Value::i64(a), Value::i64(b)])
        | (":wat::core::i64::div", [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            match a.checked_div(*b) {
                Some(n) => Ok(Value::i64(n)),
                None => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::IntegerOverflow {
                        op: "/".into(),
                        a: *a,
                        b: *b,
                    },
                )
                .into()),
            }
        }
        (":wat::core::i64::rem", [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            Ok(Value::i64(a.checked_rem(*b).unwrap_or(0)))
        }
        (":wat::core::i64::mod", [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            let r = a.checked_rem(*b).unwrap_or(0);
            Ok(Value::i64(if r != 0 && (r < 0) != (*b < 0) {
                r + *b
            } else {
                r
            }))
        }
        (":wat::core::i64::to-f64", [Value::i64(n)]) => Ok(Value::f64(*n as f64)),
        (":wat::core::i64::to-string", [Value::i64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (":wat::core::f64::to-string", [Value::f64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (":wat::core::bool::to-string", [Value::bool(b)]) => {
            Ok(Value::String(Arc::new(b.to_string())))
        }
        (":wat::core::f64::+", [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a + *b)),
        (":wat::core::f64::-", [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a - *b)),
        (":wat::core::f64::*", [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a * *b)),
        (":wat::core::f64::/", [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a / *b)),
        (":wat::core::f64::>=", [a, b]) => ord(a, b, span, |o| !o.is_lt()),
        (":wat::core::f64::<=", [a, b]) => ord(a, b, span, |o| !o.is_gt()),
        (":wat::core::f64::=", [a, b]) => Ok(Value::bool(a == b)),
        (":wat::core::f64::not=", [a, b]) => Ok(Value::bool(a != b)),
        (":wat::core::String/starts-with?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.starts_with(p.as_str())))
        }
        (":wat::core::String/ends-with?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.ends_with(p.as_str())))
        }
        (":wat::core::String/contains?", [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.contains(p.as_str())))
        }
        (":wat::core::String/empty?", [Value::String(s)]) => Ok(Value::bool(s.is_empty())),
        (":wat::core::String/concat", [Value::String(a), Value::String(b)]) => {
            Ok(Value::String(Arc::new(format!("{a}{b}"))))
        }
        (":wat::core::string::trim", [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.trim().to_string())))
        }
        (":wat::core::string::to-lowercase", [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.to_lowercase())))
        }
        (":wat::core::string::subs", [Value::String(s), Value::i64(start), Value::i64(end)]) => {
            let char_len = s.chars().count() as i64;
            if *start < 0 || *end < 0 || *start > *end || *end > char_len {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::string::subs".into(),
                        reason: format!(
                            "index out of range: start={start}, end={end}, char-length={char_len}; \
                             require 0 <= start <= end <= char-length"
                        ),
                    },
                )
                .into());
            }
            let result: String = s
                .chars()
                .skip(*start as usize)
                .take((*end - *start) as usize)
                .collect();
            Ok(Value::String(Arc::new(result)))
        }
        (":wat::core::PersistentVector/length", [Value::wat__core__PersistentVector(pv)]) => {
            Ok(Value::i64(pv.len() as i64))
        }
        (":wat::core::PersistentVector/contains?", [Value::wat__core__PersistentVector(pv), x]) => {
            Ok(Value::bool(pv.iter().any(|y| y == x)))
        }
        (":wat::core::PersistentVector/get", [pv, i]) => {
            crate::collection::eval::persistentvector_get_inner(pv, i)
        }
        (":wat::core::Vector/get", [v, i]) => crate::collection::eval::vector_get_inner(v, i),
        (":wat::core::List/get", [v, i]) => crate::collection::eval::list_get_inner(v, i),
        (":wat::core::first", [v]) => first_of(v, span),
        (":wat::core::PersistentVector", args) => Ok(Value::wat__core__PersistentVector(
            args.iter().cloned().collect(),
        )),
        (":wat::core::Vector", args) => Ok(Value::Vec(Arc::new(args.to_vec()))),
        (":wat::core::List", args) => Ok(Value::wat__core__List(Arc::new(
            args.iter().cloned().collect(),
        ))),
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: core.into(),
                reason: format!("compiled apply cannot dispatch {core} arity {}", args.len()),
            },
        )
        .into()),
    }
}

fn first_of(v: &Value, span: &Span) -> Result<Value, EvalBreak> {
    let empty = || {
        RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::first".into(),
                reason: ":wat::core::first: sequence has 0 element(s); no element at index 0"
                    .into(),
            },
        )
        .into()
    };
    match v {
        Value::wat__core__PersistentVector(pv) => pv.iter().next().cloned().ok_or_else(empty),
        Value::Vec(xs) => xs.first().cloned().ok_or_else(empty),
        Value::wat__core__List(xs) => xs.iter().next().cloned().ok_or_else(empty),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::first".into(),
                expected: "sequence",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn ord(
    a: &Value,
    b: &Value,
    span: &Span,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, EvalBreak> {
    match compare_values(a, b) {
        Some(o) => Ok(Value::bool(pred(o))),
        None => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::rete::core::cmp".into(),
                expected: "comparable pair",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

/// Rule-compile refuse: `(:wat::rete::lower <quoted-expr>) -> nil` or raise.
pub(crate) fn eval_lower(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::rete::lower".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match v {
        Value::wat__WatAST(a) => (*a).clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::rete::lower".into(),
                    expected: ":wat::WatAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    lower(&ast, sym).map_err(LowerError::into_eval)?;
    Ok(Value::Unit)
}

/// Fire-path: lower (already proved at rule-compile) and exec.
/// Oracle / differential helper. Native rematch uses a stashed `Program`.
#[allow(dead_code)]
pub(crate) fn exec_test<B: Bindings>(
    expr: &WatAST,
    bindings: &B,
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    let program = lower(expr, sym).map_err(LowerError::into_eval)?;
    exec_where(&program, bindings, sym, expr.span())
}
