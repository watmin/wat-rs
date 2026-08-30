//! The one expression core — `DESIGN-STONE-the-one-expression-core.md`.
//!
//! Nested `Expr` DAG. No `Interp` arm. `lower()` is total or it refuses.
//! First consumer: `where` (rule-compile refuse + native filter exec).
//! Oracle remains `eval_test_core`.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::rete::matcher::{compare_values, Bindings, FieldNames};
use crate::rete::vocabulary::{resolve_core_name, OpClass, RETE_OPS};
use crate::runtime::{
    coincident_q_from_values, cosine_outcome_from_values, dot_outcome_from_values,
    presence_q_from_values, EvalBreak, FunctionBody,
    RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use crate::types::TypeDef;
use crate::value::value::{AggregateValue, EnumValue};

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
        names: FieldNames,
        fields: Box<[Expr]>,
    },
    /// `(:ns::Type::Variant field…)` — tagged or unit enum constructor.
    Variant {
        type_path: String,
        variant_name: String,
        names: FieldNames,
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
        payload: PatPayload,
    },
}

pub(crate) type PatPayload = Option<Box<Pat>>;
pub(crate) type SlotName = Option<Arc<str>>;
pub(crate) type SlotNames = Box<[SlotName]>;
type ExecArena = Vec<Option<Value>>;

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
    pub(crate) names: SlotNames,
    /// Source span of the original expr — exec errors name this, not rust-caller.
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct LowerError {
    pub(crate) span: Span,
    pub(crate) kind: LowerErrorKind,
}

#[derive(Debug)]
pub(crate) enum LowerErrorKind {
    Unsupported { reason: String },
    NonLexicalCallee,
    Unbound { name: String },
}

impl LowerError {
    pub(crate) fn unsupported(span: Span, reason: String) -> Self {
        Self {
            span,
            kind: LowerErrorKind::Unsupported { reason },
        }
    }

    pub(crate) fn non_lexical(span: Span) -> Self {
        Self {
            span,
            kind: LowerErrorKind::NonLexicalCallee,
        }
    }

    pub(crate) fn unbound(span: Span, name: String) -> Self {
        Self {
            span,
            kind: LowerErrorKind::Unbound { name },
        }
    }

    pub(crate) fn into_eval(self) -> EvalBreak {
        match self.kind {
            LowerErrorKind::Unsupported { reason } => RuntimeError::new(
                self.span,
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::lower".into(),
                    reason,
                },
            )
            .into(),
            LowerErrorKind::NonLexicalCallee => RuntimeError::new(
                self.span,
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::lower".into(),
                    reason: "HOF fn-arg must be a literal fn or a named rete-defn (callee visible in the AST)".into(),
                },
            )
            .into(),
            LowerErrorKind::Unbound { name } => RuntimeError::new(
                self.span,
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

/// Lower an expression into a slot frame the CALLER owns — the entry `compiled_cond` needs to
/// finish flip 3.
///
/// [`lower`] builds a self-contained `Program` with its own frame numbered from zero. An inline
/// alpha constraint cannot use that: its operands must read the SAME scratch the `Op::Bind`
/// prologue writes fields into, so the slot numbering has to be the alpha's, not ours. This lowers
/// into a caller-supplied name->slot map and next-slot counter, and hands back the bare `Expr`.
///
/// **Why this exists at all, and it is the whole of fix-list entry F.** `compiled_cond` already
/// imports this module's `Expr`, and `Op::Cmp { lhs: Expr, rhs: Expr }` could always hold an
/// `Expr::Call`. What never landed with flip 3 was the LOWERING: `compile_operand_expr` had its
/// own three-case mini-lowering that stopped at literals, so a nested operand produced `None`, the
/// whole condition failed to compile, and the interpreted fallback answered "no match" for every
/// fact — silently. The builder's framing settles the design question it looked like: *"we made it
/// such that every rete form can be compiled to a jump table... why is this any exception?"* It is
/// not one. Same `Expr::Call`, same opcode, same `RETE_OPS` table.
pub(crate) fn lower_in_frame(
    expr: &WatAST,
    sym: &SymbolTable,
    slots: &mut HashMap<String, u16>,
    next: &mut u16,
) -> Result<Expr, LowerError> {
    let mut cx = LowerCx {
        sym,
        slots: std::mem::take(slots),
        next: *next,
        hof_fn_pos: false,
    };
    let out = lower_expr(expr, &mut cx);
    // Write the counter and map back WHATEVER the outcome: a failed lowering may still have
    // allocated slots, and leaving the caller's counter behind would alias them to later ones.
    *slots = std::mem::take(&mut cx.slots);
    *next = cx.next;
    out
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

fn slot_names(cx: &LowerCx<'_>) -> SlotNames {
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
        ast if crate::rete::matcher::ast_literal_value(ast).is_some() => {
            Ok(Expr::Lit(crate::rete::matcher::ast_literal_value(ast).unwrap()))
        }
        WatAST::Keyword(k, _) => Ok(Expr::Lit(keyword_value(k, cx.sym))),
        WatAST::NilLit(_) => Ok(Expr::Lit(Value::Unit)),
        WatAST::Symbol(id, span) => {
            let name = id.as_str();
            if name.starts_with('?') || cx.slots.contains_key(name) {
                return Ok(Expr::Slot(cx.slot(name)));
            }
            Err(LowerError::unbound(span.clone(), name.to_string()))
        }
        WatAST::List(items, span) => lower_list(items, span, cx),
        WatAST::Vector(elems, _) => {
            let mut out = Vec::with_capacity(elems.len());
            for e in elems {
                match lower_expr(e, cx)? {
                    Expr::Lit(v) => out.push(v),
                    _ => {
                        return Err(LowerError::unsupported(e.span().clone(), "vector literal in a where must be constant".into()));
                    }
                }
            }
            Ok(Expr::Lit(Value::wat__core__PersistentVector(
                out.into_iter().collect(),
            )))
        }
        other => Err(LowerError::unsupported(other.span().clone(), format!("cannot lower {}", other.span()))),
    }
}

/// A bare keyword's VALUE: an enum unit variant if the symbol table knows one, else a plain
/// keyword. `pub(crate)` since 2026-08-28 — `compiled_cond` and `matcher` need the identical
/// resolution for a keyword in DIRECT operand position, which was refused as an unknown field
/// while this exact function was already resolving it correctly one level down, inside a nested
/// operand. One question, one answer, one function.
pub(crate) fn keyword_value(k: &str, sym: &SymbolTable) -> Value {
    if let Some(ev) = sym.unit_variant(k) {
        return Value::Enum(Arc::new(ev.clone()));
    }
    Value::wat__core__keyword(Arc::new(k.to_string()))
}

fn lower_hof_callee(ast: &WatAST, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    match ast {
        WatAST::List(items, span) => {
            let head = match items.first() {
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                _ => {
                    return Err(LowerError::non_lexical(ast.span().clone()));
                }
            };
            if resolve_core_name(head) == ":wat::core::fn" {
                return lower_fn(items, span, cx);
            }
            Err(LowerError::non_lexical(span.clone()))
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
            Err(LowerError::non_lexical(span.clone()))
        }
        other => Err(LowerError::non_lexical(other.span().clone())),
    }
}

fn lower_list(items: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        Some(other) => {
            return Err(LowerError::unsupported(other.span().clone(), "call head must be a keyword".into()));
        }
        None => {
            return Err(LowerError::unsupported(span.clone(), "empty list".into()));
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
            return Err(LowerError::unsupported(span.clone(), "if takes cond then else".into()));
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
        return Err(LowerError::unsupported(span.clone(), "quote is data, not a where expression".into()));
    }
    if core == ":wat::core::kwargs-construct" || core == ":wat::core::aggregate-new" {
        let type_kw = match items.get(1) {
            Some(WatAST::Keyword(k, _)) => k.as_str(),
            _ => {
                return Err(LowerError::unsupported(span.clone(), "constructor needs a type keyword".into()));
            }
        };
        return lower_construct(type_kw, &items[1..], span, cx)?
            .ok_or_else(|| LowerError::unsupported(span.clone(), format!("unknown aggregate {type_kw}")));
    }

    // Vocabulary rows win over `:Type/field` — `PersistentVector/length` contains `/`
    // but is a rete op, not a record accessor.
    if let Some(op) = crate::rete::vocabulary::rete_op_index(head) {
        let row = &RETE_OPS[op];
        let op = op as u16;
        let hof = matches!(
            row.core_name,
            ":wat::core::foldl"
                | ":wat::core::mapv"
                | ":wat::core::filterv"
                | ":wat::core::reduce"
        );
        if row.class == OpClass::Fallback {
            return lower_fallback(op, &items[1..], span, cx, hof);
        }
        // ⛔ THE RETE SURFACE ADMITS ONLY `reduce`'s TOTAL ARITY, and refuses the other at COMPILE
        // time rather than raising at fire time.
        //
        // `wat/seq.wat:317-329` defines both clauses: 3-arity is literally `(foldl f init coll)`,
        // and 2-arity seeds from the first element and RAISES BY NAME on an empty collection. The
        // row declares `total: true` — which every row must, by `every_rete_row_is_total`, because
        // "a jump table over a partial op is not a thing". So the 2-arity form and that declaration
        // cannot both stand.
        //
        // The table's own comment already ruled how a partial core op earns a rete surface: NOT by
        // weakening the wall, but by BUYING totality with a mandatory `:undefined` (`OpClass::Fallback`,
        // which is exactly why partial `i64::/` is `total: true` here). `Fallback` is a property of
        // the ROW though, so taking it would force the ceremony onto the 3-arity form, which is
        // already total and needs nothing. Refusing the partial arity is the narrower reading of
        // the same doctrine, and it keeps rete's surface narrower than core's for the reason it
        // always is — per-type comparators, eager materializers, and now total arities only.
        //
        // Found 2026-08-28 by the § 4.1 ledger. The partiality is not new; it was UNREACHABLE
        // until `exec_reduce` landed hours earlier, which is what turned a latent false
        // declaration into a live one.
        if row.core_name == ":wat::core::reduce" && items.len() == 3 {
            return Err(LowerError::unsupported(
                span.clone(),
                "the rete surface admits only the 3-arity `reduce` — `(reduce f init coll)`. The \
                 2-arity form seeds the fold from the first element and raises on an empty \
                 collection, and every rete row must be TOTAL. Supply an explicit init."
                    .to_string(),
            ));
        }
        let args = lower_call_args(&items[1..], cx, hof)?;
        return Ok(Expr::Call { op, args });
    }

    // Accessor `:ns::Type/field` — class and field are in the head.
    if let Some((cls, field)) = split_accessor(head) {
        if items.len() != 2 {
            return Err(LowerError::unsupported(span.clone(), "accessor takes one receiver".into()));
        }
        let idx = field_index(cx.sym, cls, field).ok_or_else(|| LowerError::unsupported(span.clone(), format!("unknown accessor {head}")))?;
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

    Err(LowerError::unsupported(span.clone(), format!("cannot lower head {head}")))
}

type ExprArgs = Box<[Expr]>;

fn lower_call_args(
    args: &[WatAST],
    cx: &mut LowerCx,
    hof: bool,
) -> Result<ExprArgs, LowerError> {
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

fn lower_args(args: &[WatAST], cx: &mut LowerCx) -> Result<ExprArgs, LowerError> {
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
        return Err(LowerError::unsupported(span.clone(), format!("{} wants {total} args", row.rete_name)));
    }
    let marker = total.saturating_sub(2);
    match &args.get(marker) {
        Some(WatAST::Keyword(k, _)) if k == ":undefined" => {}
        _ => {
            return Err(LowerError::unsupported(span.clone(), "fallback op requires literal :undefined".into()));
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
            return Err(LowerError::unsupported(span.clone(), "let takes [binds] body".into()));
        }
    };
    let pairs = match binds_ast {
        WatAST::Vector(v, _) => v.as_slice(),
        _ => {
            return Err(LowerError::unsupported(binds_ast.span().clone(), "let binds must be a vector".into()));
        }
    };
    let mut binds = Vec::new();
    let mut i = 0;
    while i + 1 < pairs.len() {
        let name = match &pairs[i] {
            WatAST::Symbol(id, _) => id.as_str().to_string(),
            other => {
                return Err(LowerError::unsupported(other.span().clone(), "let binder must be a symbol".into()));
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
        return Err(LowerError::unsupported(span.clone(), "match needs a scrutinee".into()));
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
                return Err(LowerError::unsupported(other.span().clone(), "malformed match arm".into()));
            }
        }
    }
    Ok(Expr::Match {
        scrutinee,
        arms: arms.into_boxed_slice(),
    })
}

fn lower_pat(ast: &WatAST, cx: &mut LowerCx) -> Result<Pat, LowerError> {
    if let Some(v) = crate::rete::matcher::ast_literal_value(ast) {
        return Ok(Pat::Lit(v));
    }
    match ast {
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
                    return Err(LowerError::unsupported(span.clone(), "match list pattern head must be a keyword".into()));
                }
            };
            if tag.contains('{') || matches!(items[0], WatAST::Map(_, _)) {
                return Err(LowerError::unsupported(span.clone(), "match map-destructure is not lowered in v1".into()));
            }
            let name = option_result_tag(tag).unwrap_or_else(|| tag.to_string());
            let payload = if items.len() > 1 {
                Some(Box::new(lower_pat(&items[1], cx)?))
            } else {
                None
            };
            Ok(Pat::Variant { name, payload })
        }
        WatAST::Map(_, span) => Err(LowerError::unsupported(span.clone(), "match map-destructure is not lowered in v1".into())),
        other => Err(LowerError::unsupported(other.span().clone(), "unsupported match pattern".into())),
    }
}

fn lower_fn(items: &[WatAST], span: &Span, cx: &mut LowerCx) -> Result<Expr, LowerError> {
    let arrow = items
        .iter()
        .position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->"));
    let Some(arrow) = arrow else {
        return Err(LowerError::unsupported(span.clone(), "fn needs ->".into()));
    };
    let params_ast = items.get(1).ok_or_else(|| LowerError::unsupported(span.clone(), "fn needs a param vector".into()))?;
    let body_forms = items.get(arrow + 2..).unwrap_or(&[]);
    let body = body_forms.last().ok_or_else(|| LowerError::unsupported(span.clone(), "fn needs a body".into()))?;
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
    span: &Span,
    sym: &SymbolTable,
) -> Result<Arc<Program>, LowerError> {
    let func = match sym.get(head) {
        Some(f) => f,
        None => {
            return Err(LowerError::unsupported(span.clone(), format!("unknown rete-defn {head}")));
        }
    };
    if func.rete.is_none() {
        return Err(LowerError::unsupported(span.clone(), format!("{head} is not a rete-defn")));
    }
    match &func.body {
        FunctionBody::Wat(body) => lower_rete_defn(func.as_ref(), body, sym),
        _ => Err(LowerError::unsupported(span.clone(), format!("{head} has no wat body"))),
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
    let last = wat_reader::identifier::leaf(tag).trim_start_matches(':');
    match last {
        "None" | "Some" | "Ok" | "Err" => Some(last.to_string()),
        _ => None,
    }
}

fn split_accessor(head: &str) -> Option<(&str, &str)> {
    let rest = head.strip_prefix(':')?;
    if !rest.contains('/') {
        return None;
    }
    let (cls, field) = (wat_reader::identifier::receiver(rest), wat_reader::identifier::method(rest));
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
    if let Some(TypeDef::Aggregate(a)) = types.get(head) {
        let names = a.names_arc();
        let class = head.strip_prefix(':').unwrap_or(head).to_string();
        let args = &items[1..];
        // BY NAME, against this type's declaration order — see `rete_kwargs_value_asts`. `None`
        // (undeclared / duplicate / missing field) falls to the same `Ok(None)` this fn already
        // uses for a construct it cannot lower.
        let Some(value_asts) = crate::rete::eval_insert::rete_kwargs_value_asts(args, &names) else {
            return Ok(None);
        };
        let mut fields = Vec::with_capacity(value_asts.len());
        for v in value_asts {
            fields.push(lower_expr(v, cx)?);
        }
        if fields.len() != names.len() {
            return Err(LowerError::unsupported(span.clone(), format!(
                    "constructor {head} wants {} fields, got {}",
                    names.len(),
                    fields.len()
                )));
        }
        return Ok(Some(Expr::Construct {
            class,
            names,
            fields: fields.into_boxed_slice(),
        }));
    }

    // Enum variant constructor `:ns::Type::Variant` (unit or tagged).
    // Resolution through `matcher::enum_variant_ctor` — the one registry read. The lowerer is
    // the caller that needs all three parts: the `EnumDef` to reach `variant_names_arc`, the
    // variant name to key it, and the arity to check the call against.
    if let Some((e, variant, arity)) = crate::rete::matcher::enum_variant_ctor(types, head) {
        let names = if arity == 0 {
            Arc::new(Vec::new())
        } else {
            e.variant_names_arc(variant)
                .unwrap_or_else(|| Arc::new(Vec::new()))
        };
        let args = &items[1..];
        if args.len() != arity {
            return Err(LowerError::unsupported(span.clone(), format!(
                    "constructor {head} wants {arity} fields, got {}",
                    args.len()
                )));
        }
        let mut fields = Vec::with_capacity(args.len());
        for a in args {
            fields.push(lower_expr(a, cx)?);
        }
        return Ok(Some(Expr::Variant {
            type_path: e.name.clone(),
            variant_name: variant.to_string(),
            names,
            fields: fields.into_boxed_slice(),
        }));
    }
    Ok(None)
}

// ── exec ─────────────────────────────────────────────────────────────────────

pub(crate) fn exec_where<B: Bindings + ?Sized>(
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
                op: ":wat::rete::where".into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

fn write_slot(
    frame: &mut [Option<Value>],
    slot: u16,
    v: Value,
    span: &Span,
) -> Result<(), EvalBreak> {
    match frame.get_mut(slot as usize) {
        Some(s) => {
            *s = Some(v);
            Ok(())
        }
        None => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::exec_value".into(),
                reason: format!("slot {slot} is outside frame_len {}", frame.len()),
            },
        )
        .into()),
    }
}

/// Prologue (token bindings → slots) + eval. `where` requires bool;
/// `compiled_rhs` takes the `Value` as a fact field.
pub(crate) fn exec_value<B: Bindings + ?Sized>(
    program: &Program,
    bindings: &B,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    with_exec_frame(program.frame_len as usize, |frame| {
        for (k, slot) in program.reads.iter() {
            if let Some(v) = bindings.get(k) {
                write_slot(frame, *slot, v.clone(), span)?;
            }
        }
        exec(&program.root, frame, &program.names, sym, span)
    })
}

// rune:sequi(ambient-context) — one thread, nested frames bump a high-water
// arena so exec_where / CallUser / foldl do not allocate per token after warmup.
thread_local! {
    static EXEC_ARENA: RefCell<ExecArena> = const { RefCell::new(Vec::new()) };
    static EXEC_SP: Cell<usize> = const { Cell::new(0) };
}

fn with_exec_frame<R>(len: usize, f: impl FnOnce(&mut [Option<Value>]) -> R) -> R {
    EXEC_ARENA.with(|arena| {
        match arena.try_borrow_mut() {
            Ok(mut g) => {
                let start = EXEC_SP.get();
                let end = start + len;
                if g.len() < end {
                    g.resize(end, None);
                }
                for slot in &mut g[start..end] {
                    *slot = None;
                }
                EXEC_SP.set(end);
                let out = f(&mut g[start..end]);
                EXEC_SP.set(start);
                out
            }
            // Nested exec_where / CallUser / fold while the outer frame is live.
            // Stack frame; the TLS arena stays with the outer caller.
            Err(_) => {
                let mut local = vec![None; len];
                f(&mut local)
            }
        }
    })
}

pub(crate) fn exec(
    e: &Expr,
    frame: &mut [Option<Value>],
    names: &[SlotName],
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
                            record_class: a.class.to_string(),
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
        Expr::Variant {
            type_path,
            variant_name,
            names: field_names,
            fields,
        } => {
            let mut vs = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                vs.push(exec(f, frame, names, sym, span)?);
            }
            Ok(Value::Enum(Arc::new(EnumValue {
                type_path: type_path.clone(),
                variant_name: variant_name.clone(),
                names: Arc::clone(field_names),
                fields: vs,
            })))
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
                write_slot(frame, *slot, v, span)?;
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
            match RETE_OPS[*op as usize].core_name {
                ":wat::core::foldl" => return exec_foldl(args, frame, names, sym, span),
                ":wat::core::reduce" => return exec_reduce(args, frame, names, sym, span),
                ":wat::core::mapv" => return exec_mapv(args, frame, names, sym, span),
                ":wat::core::filterv" => return exec_filterv(args, frame, names, sym, span),
                _ => {}
            }
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            apply_op(*op, &vs, span, Some(sym))
        }
        Expr::CallFallback { op, args, fallback } => {
            let row = &RETE_OPS[*op as usize];
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            // ONE classification — see `where_tree.rs`'s twin and
            // `classify_fallback_outcome`. Only the recursion is this site's own.
            match crate::runtime::classify_fallback_outcome(
                apply_op(*op, &vs, span, Some(sym)),
                &row.ret,
                row.core_name,
                row.rete_name,
                span,
            )? {
                crate::runtime::FallbackVerdict::Value(v) => Ok(v),
                crate::runtime::FallbackVerdict::UseFallback => {
                    exec(fallback, frame, names, sym, span)
                }
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
    // rune:perspicere(intentional-structure) — SlotFrame row; alias body would hide the slot layout
    parent: Option<&[Option<Value>]>,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    let max_param = program
        .params
        .iter()
        .copied()
        .max()
        .map(|s| s as usize + 1)
        .unwrap_or(0);
    let n = (program.frame_len as usize)
        .max(parent.map(|p| p.len()).unwrap_or(0))
        .max(max_param);
    with_exec_frame(n, |inner| {
        if let Some(p) = parent {
            for (i, v) in p.iter().enumerate() {
                inner[i] = v.clone();
            }
        }
        for (i, v) in args.iter().enumerate() {
            if let Some(&slot) = program.params.get(i) {
                let idx = slot as usize;
                if idx < inner.len() {
                    inner[idx] = Some(v.clone());
                }
            } else if i < inner.len() {
                inner[i] = Some(v.clone());
            }
        }
        exec(&program.root, inner, &program.names, sym, span)
    })
}

fn exec_foldl(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
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
    let program = compiled_fn_arg(&args[0], ":wat::core::foldl", span)?;
    let mut acc = exec(&args[1], frame, names, sym, span)?;
    let coll = exec(&args[2], frame, names, sym, span)?;
    let items = eager_items(&coll, ":wat::core::foldl", span)?;
    for x in items {
        acc = exec_program_on(&program, &[acc.clone(), x], Some(frame), sym, span)?;
    }
    Ok(acc)
}

/// The fn operand of a HOF, as a compiled program. Shared so `reduce` cannot drift from `foldl`.
fn compiled_fn_arg(arg: &Expr, op: &str, span: &Span) -> Result<Arc<Program>, EvalBreak> {
    match arg {
        Expr::CallUser { program, .. } => Ok(Arc::clone(program)),
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "fn-arg must be a compiled fn".into(),
            },
        )
        .into()),
    }
}

/// The eager containers a compiled fence can walk. A `Stream` is deliberately absent: it is lazy,
/// and the compiled executor has no stream machinery — so it reports a type mismatch that NAMES
/// the containers it does accept rather than silently producing nothing.
fn eager_items(coll: &Value, op: &str, span: &Span) -> Result<Vec<Value>, EvalBreak> {
    match coll {
        Value::Vec(xs) => Ok(xs.iter().cloned().collect()),
        Value::wat__core__PersistentVector(pv) => Ok(pv.iter().cloned().collect()),
        Value::wat__core__List(xs) => Ok(xs.iter().cloned().collect()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::reduce f init coll)` / `(:wat::core::reduce f coll)`.
///
/// ⛔ **FOLDL IS REDUCE, and this is a MIRROR of `wat/seq.wat:317-329`, not a reimplementation.**
/// That `defclause` states both clauses outright: the 3-arity form is literally
/// `(:wat::core::foldl f init coll)`, and the 2-arity form seeds the fold with the first element
/// and RAISES BY NAME on an empty collection. Both are reproduced here and nowhere else, so the
/// compiled answer and the interpreted one cannot diverge.
///
/// Why a compiled arm exists at all: `reduce` is a wat-level `defclause`, so unlike its siblings
/// it has no Rust dispatch to re-enter, and a compiled `where` fence has no defclause machinery.
/// Found 2026-08-28 by the § 4.1 ledger — the row passed admission, totality, arity and type and
/// then raised `unbound symbol: acc`, because lowering treats all four HOFs alike while `exec`
/// routed only `foldl`.
///
/// ⚠ The 2-arity empty case RAISES, while `RETE_OPS` declares this row `total: true`. That
/// contradiction is inherited, not introduced — and it went unnoticed precisely because nothing
/// could execute the row to find it. Recorded in `RETE-OPEN-WORK` § 4.1; not silently papered over
/// here, because answering an empty reduce with some invented value would be the worse bug.
fn exec_reduce(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::reduce";
    if args.len() == 3 {
        return exec_foldl(args, frame, names, sym, span);
    }
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 3, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut it = items.into_iter();
    let Some(mut acc) = it.next() else {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "the 2-arity form needs at least one element to seed the fold; got an \
                         empty collection"
                    .into(),
            },
        )
        .into());
    };
    for x in it {
        acc = exec_program_on(&program, &[acc.clone(), x], Some(frame), sym, span)?;
    }
    Ok(acc)
}

/// `(:wat::core::mapv f coll)` — the EAGER map. Returns a `Vector`, matching `eval_mapv`
/// (`collection/transform.rs`), whose every exit is `Ok(Value::Vec(..))`.
///
/// ⛔ **THE RETE SURFACE TAKES `mapv`, NOT `map`, AND THAT IS THE WHOLE POINT.** `:wat::core::map`
/// returns a LAZY `Stream`; a compiled `where` fence has no stream machinery and nothing in a
/// fence can consume one, so the `map` row was unreachable in every position. Adding an eager arm
/// under the `map` name would have made `:wat::rete::core::map` mean something different from
/// `:wat::core::map` — silently — when the `Redispatch` contract is "the same routine as
/// `core_name`". wat already ships the eager materializer under its clojure name, so rete takes
/// that instead: no invented semantics and no divergence. See `wat/seq.wat`'s "the eager forms".
fn exec_mapv(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::mapv";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 2, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut out = Vec::with_capacity(items.len());
    for x in items {
        out.push(exec_program_on(&program, &[x], Some(frame), sym, span)?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::filterv pred coll)` — the EAGER filter. Returns a `Vector`, matching
/// `wat/seq.wat`'s `defclause`, which is `(:wat::core::into [] (:wat::core::filter pred coll))`
/// for both of its clauses.
///
/// The predicate must answer `bool`. A non-bool is refused BY NAME rather than coerced: a filter
/// that silently treats a non-boolean as truthy would drop or keep rows for a reason no user
/// wrote, which is the silent-wrong-answer class this arc exists to remove.
fn exec_filterv(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::filterv";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 2, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut out = Vec::with_capacity(items.len());
    for x in items {
        match exec_program_on(&program, std::slice::from_ref(&x), Some(frame), sym, span)? {
            Value::bool(true) => out.push(x),
            Value::bool(false) => {}
            other => {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::core::bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

fn pat_matches(pat: &Pat, v: &Value, frame: &mut [Option<Value>]) -> bool {
    match pat {
        Pat::Wild => true,
        Pat::Bind(s) => match frame.get_mut(*s as usize) {
            Some(slot) => {
                *slot = Some(v.clone());
                true
            }
            None => false,
        },
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
                let last = wat_reader::identifier::leaf(name).trim_start_matches(':');
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

#[derive(Clone, Copy, Debug)]
enum OpExec {
    Eq, NotEq, Gt, Lt, Ge, Le,
    I64Eq, I64NotEq, StrEq, StrNotEq,
    StrLen, StartsWith, EndsWith, Contains, Not,
    I64Add, I64Sub, I64Mul, I64Div, I64Rem, I64Mod, I64ToF64, I64ToStr,
    F64Gt, F64Lt, F64Ge, F64Le, F64Eq, F64NotEq, F64Add, F64Sub, F64Mul, F64Div, F64ToStr,
    BoolToStr, StrEmpty, StrConcat, StrTrim, StrLower, StrSubs,
    PvLen, PvContains, PvGet, VecGet, ListGet, First, PvNew, VecNew, ListNew,
    PmContainsKey, PmNew, Second, Third, TupleNew, KwToStr, KwFromStr,
    Cosine, Dot, Coincident, Presence,
    Unknown,
}

impl OpExec {
    fn of(core: &str) -> Self {
        // Arc 255 Stone C — `core` arrives as `row.core_name`, which for the per-type
        // numerics reads `:wat::i64::+` (B-i's home), not `:wat::core::i64::+`. Through
        // Stone B this table's arms were still keyed on the OLD spelling, folding the new
        // spelling back onto its old twin via `crate::runtime::fold_numeric_home` before
        // matching (mirroring `runtime.rs::dispatch_substrate_impl`'s fold). Stone C
        // retires the old spelling and deletes that fn, so the arms below are keyed on
        // the new spelling DIRECTLY — no fold left to perform.
        match core {
            ":wat::core::=" => Self::Eq,
            ":wat::core::not=" => Self::NotEq,
            ":wat::i64::>" | ":wat::core::>" => Self::Gt,
            ":wat::i64::<" | ":wat::core::<" => Self::Lt,
            ":wat::i64::>=" | ":wat::core::>=" => Self::Ge,
            ":wat::i64::<=" | ":wat::core::<=" => Self::Le,
            ":wat::i64::=" => Self::I64Eq,
            ":wat::i64::not=" => Self::I64NotEq,
            ":wat::string::=" => Self::StrEq,
            ":wat::string::not=" => Self::StrNotEq,
            ":wat::string::length" => Self::StrLen,
            ":wat::string::starts-with?" => Self::StartsWith,
            ":wat::string::ends-with?" => Self::EndsWith,
            ":wat::string::contains?" => Self::Contains,
            ":wat::core::not" => Self::Not,
            ":wat::i64::+" => Self::I64Add,
            ":wat::i64::-" => Self::I64Sub,
            ":wat::i64::*" => Self::I64Mul,
            ":wat::i64::/" | ":wat::i64::quot" => Self::I64Div,
            ":wat::i64::rem" => Self::I64Rem,
            ":wat::i64::mod" => Self::I64Mod,
            ":wat::i64::to-f64" => Self::I64ToF64,
            ":wat::i64::to-string" => Self::I64ToStr,
            ":wat::f64::>" => Self::F64Gt,
            ":wat::f64::<" => Self::F64Lt,
            ":wat::f64::>=" => Self::F64Ge,
            ":wat::f64::<=" => Self::F64Le,
            ":wat::f64::=" => Self::F64Eq,
            ":wat::f64::not=" => Self::F64NotEq,
            ":wat::f64::+" => Self::F64Add,
            ":wat::f64::-" => Self::F64Sub,
            ":wat::f64::*" => Self::F64Mul,
            ":wat::f64::/" => Self::F64Div,
            ":wat::f64::to-string" => Self::F64ToStr,
            ":wat::core::bool::to-string" => Self::BoolToStr,
            ":wat::string::empty?" => Self::StrEmpty,
            ":wat::string::concat" => Self::StrConcat,
            ":wat::string::trim" => Self::StrTrim,
            ":wat::string::to-lowercase" => Self::StrLower,
            ":wat::string::subs" => Self::StrSubs,
            // Arc 255 Stone E-ii — `core` arrives as `row.core_name`, which for the moved
            // PersistentVector/Vector verbs now reads `:wat::vector::*`/`:wat::vec::*` (E-ii's
            // homes), not `:wat::core::PersistentVector/*`/`:wat::core::Vector/*`. Mirrors Stone
            // C's numerics fold-removal note above: keyed on the new spelling directly.
            ":wat::vector::length" => Self::PvLen,
            ":wat::vector::contains?" => Self::PvContains,
            ":wat::vector::get" => Self::PvGet,
            ":wat::vec::get" => Self::VecGet,
            // Arc 255 Stone E-iii — `:wat::core::List/get` retired this stone;
            // `:wat::linkedlist::get` is its replacement. Mirrors the E-ii note above.
            ":wat::linkedlist::get" => Self::ListGet,
            // grok-rete (arc 278, "the keyword converters" + PersistentMap row), carried here
            // under main's post-255 spellings. The originals were written pre-rename as
            // `:wat::core::keyword/to-string`, `:wat::core::keyword/from-string` and
            // `:wat::core::PersistentMap/contains-key?`; each is mapped by the retirement
            // table (src/remedy/retirement.rs:307,308,253) to the homes used below. main has
            // no competing rows for these three — they are new rete surface, not a second
            // implementation.
            ":wat::keyword::to-string" => Self::KwToStr,
            ":wat::keyword::from-string" => Self::KwFromStr,
            ":wat::map::contains-key?" => Self::PmContainsKey,
            ":wat::core::first" => Self::First,
            ":wat::core::second" => Self::Second,
            ":wat::core::third" => Self::Third,
            ":wat::core::PersistentVector" => Self::PvNew,
            ":wat::core::Vector" => Self::VecNew,
            ":wat::core::List" => Self::ListNew,
            ":wat::core::Tuple" => Self::TupleNew,
            ":wat::core::PersistentMap" => Self::PmNew,
            ":wat::holon::cosine" => Self::Cosine,
            ":wat::holon::dot" => Self::Dot,
            ":wat::holon::coincident?" => Self::Coincident,
            ":wat::holon::presence?" => Self::Presence,
            _ => Self::Unknown,
        }
    }
}

/// Index `RETE_OPS` once; fire matches `OpExec`, never the FQDN string.
/// `sym` is required for holon rows (encoding ctx). The where-tree dim
/// walker may pass `None` and treat a holon miss as over-approx.
pub(crate) fn apply_op(
    op: u16,
    args: &[Value],
    span: &Span,
    sym: Option<&SymbolTable>,
) -> Result<Value, EvalBreak> {
    // rune:sequi(ambient-context) — opcode table interned once; not fire-domain state.
    static KINDS: OnceLock<Vec<OpExec>> = OnceLock::new();
    let kinds = KINDS.get_or_init(|| {
        RETE_OPS.iter().map(|r| OpExec::of(r.core_name)).collect()
    });
    let Some(&kind) = kinds.get(op as usize) else {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::apply_op".into(),
                reason: format!("op index {op} is outside RETE_OPS"),
            },
        )
        .into());
    };
    apply_core_kind(kind, args, span, sym)
}

/// Apply one compiled core op to its ALREADY-EVALUATED arguments.
///
/// This is the leaf of the compiled-expression interpreter: `expr_ir` lowers a
/// `:where` form to `Op`s, the walker evaluates the operands, and every actual
/// computation lands here. The 53 arms are a flat dispatch table — one per
/// `OpExec` — and are deliberately uncommented: each matches on `(kind, args)`
/// and its body IS its specification. Reach for the arm, not for prose.
///
/// Two things a reader cannot recover from the arms themselves:
///
/// **The pattern is the arity-and-type check.** An arm matches only when the
/// operand shapes match too (`[Value::i64(a), Value::i64(b)]`), so a wrong
/// arity or a wrong operand type does not reach a body — it falls through to
/// the catch-all. There is no separate validation pass; this table is it.
///
/// **The catch-all raises with head `"compiled-exec"`, and that head matters.**
/// `exec_dim`'s `CallFallback` swallows a `MalformedForm` only when its head
/// equals the op's own `core_name` (see `where_tree::exec_dim`). `"compiled-exec"`
/// never equals one, so a dispatch failure PROPAGATES rather than being
/// silently replaced by a fallback value. Do not retag this error to an op name.
///
/// `sym` is `None` off the encoding path; the holon arms that need it raise
/// `NoEncodingCtx` rather than assuming a context they were not given.
fn apply_core_kind(
    kind: OpExec,
    args: &[Value],
    span: &Span,
    sym: Option<&SymbolTable>,
) -> Result<Value, EvalBreak> {
    match (kind, args) {
        (OpExec::Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::Gt, [a, b]) => {
            ord(a, b, span, |o| o.is_gt())
        }
        (OpExec::Lt, [a, b]) => {
            ord(a, b, span, |o| o.is_lt())
        }
        (OpExec::Ge, [a, b]) => {
            ord(a, b, span, |o| !o.is_lt())
        }
        (OpExec::Le, [a, b]) => {
            ord(a, b, span, |o| !o.is_gt())
        }
        (OpExec::I64Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::I64NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrEq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::StrNotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrLen, [Value::String(s)]) => {
            Ok(Value::i64(s.chars().count() as i64))
        }
        (OpExec::StartsWith, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.starts_with(p.as_str())))
        }
        (OpExec::EndsWith, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.ends_with(p.as_str())))
        }
        (OpExec::Contains, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.contains(p.as_str())))
        }
        (OpExec::F64Gt, [a, b]) => ord(a, b, span, |o| o.is_gt()),
        (OpExec::F64Lt, [a, b]) => ord(a, b, span, |o| o.is_lt()),
        (OpExec::Not, [Value::bool(b)]) => Ok(Value::bool(!*b)),
        (OpExec::I64Add, [Value::i64(a), Value::i64(b)]) => match a.checked_add(*b) {
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
        (OpExec::I64Sub, [Value::i64(a), Value::i64(b)]) => match a.checked_sub(*b) {
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
        (OpExec::I64Mul, [Value::i64(a), Value::i64(b)]) => match a.checked_mul(*b) {
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
        (OpExec::I64Div, [Value::i64(a), Value::i64(b)]) => {
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
        (OpExec::I64Rem, [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            match a.checked_rem(*b) {
                Some(n) => Ok(Value::i64(n)),
                None => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::IntegerOverflow {
                        op: "rem".into(),
                        a: *a,
                        b: *b,
                    },
                )
                .into()),
            }
        }
        (OpExec::I64Mod, [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            let r = match a.checked_rem(*b) {
                Some(n) => n,
                None => {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::IntegerOverflow {
                            op: "mod".into(),
                            a: *a,
                            b: *b,
                        },
                    )
                    .into())
                }
            };
            Ok(Value::i64(if r != 0 && (r < 0) != (*b < 0) {
                r + *b
            } else {
                r
            }))
        }
        (OpExec::I64ToF64, [Value::i64(n)]) => Ok(Value::f64(*n as f64)),
        (OpExec::I64ToStr, [Value::i64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (OpExec::F64ToStr, [Value::f64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (OpExec::BoolToStr, [Value::bool(b)]) => {
            Ok(Value::String(Arc::new(b.to_string())))
        }
        (OpExec::F64Add, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a + *b)),
        (OpExec::F64Sub, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a - *b)),
        (OpExec::F64Mul, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a * *b)),
        (OpExec::F64Div, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a / *b)),
        (OpExec::F64Ge, [a, b]) => ord(a, b, span, |o| !o.is_lt()),
        (OpExec::F64Le, [a, b]) => ord(a, b, span, |o| !o.is_gt()),
        (OpExec::F64Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::F64NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrEmpty, [Value::String(s)]) => Ok(Value::bool(s.is_empty())),
        (OpExec::StrConcat, [Value::String(a), Value::String(b)]) => {
            Ok(Value::String(Arc::new(format!("{a}{b}"))))
        }
        (OpExec::StrTrim, [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.trim().to_string())))
        }
        (OpExec::StrLower, [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.to_lowercase())))
        }
        (OpExec::StrSubs, [Value::String(s), Value::i64(start), Value::i64(end)]) => {
            let char_len = s.chars().count() as i64;
            if *start < 0 || *end < 0 || *start > *end || *end > char_len {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::string::subs".into(),
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
        (OpExec::PvLen, [Value::wat__core__PersistentVector(pv)]) => {
            Ok(Value::i64(pv.len() as i64))
        }
        (OpExec::PvContains, [Value::wat__core__PersistentVector(pv), x]) => {
            Ok(Value::bool(pv.iter().any(|y| y == x)))
        }
        // Delegates to the SAME inner the interpreter calls (`runtime.rs`'s
        // `eval_persistentmap_contains_key_q` routes here too), rather than re-deriving map
        // membership — the sibling `PvGet`/`VecGet` arms below establish that shape. Its two
        // exits are audited in `vocabulary.rs`'s row comment: an unhashable key answers `false`
        // (the predicate ruling, not a sentinel), a wrong receiver raises `TypeMismatch` and is
        // refused by the checker before runtime because the row DECLARES its receiver.
        (OpExec::PmContainsKey, [m, k]) => {
            crate::collection::eval::persistentmap_contains_key_q_inner(m, k)
        }
        (OpExec::PvGet, [pv, i]) => {
            crate::collection::eval::persistentvector_get_inner(pv, i)
        }
        (OpExec::VecGet, [v, i]) => crate::collection::eval::vector_get_inner(v, i),
        (OpExec::ListGet, [v, i]) => crate::collection::eval::list_get_inner(v, i),
        (OpExec::First, [v]) => first_of(v, span),
        // The keyword converters, both delegating to the interpreter's own value-level routines so
        // an `Alias`/`Fallback` row cannot mean something different here than in core.
        (OpExec::KwToStr, [v]) => crate::runtime::keyword_to_string_value(v).ok_or_else(|| {
            EvalBreak::from(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::keyword::to-string".into(),
                    expected: "keyword",
                    got: Box::new(ValueSnapshot::of(v)),
                },
            ))
        }),
        // PARTIAL by design: a leading ':' or an angle-type head has no keyword. The row is
        // `Fallback`, so `CallFallback` substitutes the caller's mandatory `:undefined` value on
        // this Err — which is how the row is `total: true` without inventing an answer here.
        (OpExec::KwFromStr, [v]) => crate::runtime::keyword_from_string_value(v).ok_or_else(|| {
            EvalBreak::from(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::keyword::from-string".into(),
                    reason: "a keyword's text may not start with ':' or carry an angle-type head"
                        .into(),
                },
            ))
        }),
        // `second`/`third` call the interpreter's own `positional_at`, so every container it
        // supports — Tuple included — is supported here by construction rather than by a list
        // someone remembered to keep in step. Arity is enforced at CHECK time
        // (`third` on a 2-tuple is a TypeMismatch naming "expects tuple with >= 3 element(s)"),
        // which is what makes these rows honestly `total: true`.
        (OpExec::Second, [v]) => {
            crate::runtime::positional_at(v.clone(), 1, ":wat::core::second", span)
        }
        (OpExec::Third, [v]) => {
            crate::runtime::positional_at(v.clone(), 2, ":wat::core::third", span)
        }
        (OpExec::PvNew, args) => Ok(Value::wat__core__PersistentVector(
            args.iter().cloned().collect(),
        )),
        (OpExec::VecNew, args) => Ok(Value::Vec(Arc::new(args.to_vec()))),
        (OpExec::ListNew, args) => Ok(Value::wat__core__List(Arc::new(
            args.iter().cloned().collect(),
        ))),
        // Mirrors `eval_tuple_ctor` (`runtime.rs`), including its one rule: arity 1+, because the
        // 0-tuple is the Unit `:()` and not a Tuple. The three sibling constructors above have no
        // such floor, which is why this is spelled out rather than folded in with them.
        (OpExec::TupleNew, []) => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::Tuple".into(),
                reason: "tuple must have at least one element; the 0-tuple is :() (Unit)".into(),
            },
        )
        .into()),
        (OpExec::TupleNew, args) => Ok(Value::Tuple(Arc::new(args.to_vec()))),
        // The three sibling constructors above just collect; a map cannot, and the rules it must
        // follow are NOT invented here — every one is read off `eval_persistentmap_ctor`
        // (`collection/eval.rs`), which is what the interpreter runs: even arity, alternating
        // key/value, each key `value_is_key_hashable`, built with `PMap::from_pairs`. The two
        // semantic primitives are called directly rather than re-derived, so the compiled answer
        // and the interpreted one cannot drift; only argument EVALUATION differs, and the compiled
        // path has already done that.
        //
        // Found 2026-08-28 by the § 4.1 reachability ledger: this row passed admission, totality,
        // arity and type and then raised `cannot dispatch kind Unknown arity 2` at RUNTIME, exactly
        // like `PersistentMap/contains-key?` before it.
        (OpExec::PmNew, args) => {
            if !args.len().is_multiple_of(2) {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::PersistentMap".into(),
                        reason: format!(
                            "arity must be even (alternating key/value pairs); got {}",
                            args.len()
                        ),
                    },
                )
                .into());
            }
            let mut pairs: Vec<(Value, Value)> = Vec::with_capacity(args.len() / 2);
            for pair in args.chunks(2) {
                if !crate::runtime::value_is_key_hashable(&pair[0]) {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::PersistentMap".into(),
                            expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                            got: Box::new(ValueSnapshot::of(&pair[0])),
                        },
                    )
                    .into());
                }
                pairs.push((pair[0].clone(), pair[1].clone()));
            }
            Ok(Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs)))
        }
        (OpExec::Cosine, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::cosine".into(),
                    },
                )
                .into());
            };
            cosine_outcome_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Dot, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::dot".into(),
                    },
                )
                .into());
            };
            dot_outcome_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Coincident, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::coincident?".into(),
                    },
                )
                .into());
            };
            coincident_q_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Presence, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::presence?".into(),
                    },
                )
                .into());
            };
            presence_q_from_values(a.clone(), b.clone(), span, sym)
        }
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: "compiled-exec".into(),
                reason: format!("compiled apply cannot dispatch kind {kind:?} arity {}", args.len()),
            },
        )
        .into()),
    }
}

/// `first` — delegates to the interpreter's `positional_at` at index 0.
///
/// ⛔ **THIS USED TO BE A SECOND IMPLEMENTATION, AND THAT WAS THE BUG.** It matched
/// PersistentVector / Vec / List and rejected everything else, so a `Tuple` built inside a `where`
/// fence could never be read — while core's `first` has always projected a Tuple. Two routines for
/// one verb, silently disagreeing about which containers exist. Now there is one.
fn first_of(v: &Value, span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::positional_at(v.clone(), 0, ":wat::core::first", span)
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
                op: "compiled-compare".into(),
                expected: "comparable pair",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

/// `(:wat::rete::lower expr) -> :wat::core::nil`
///
/// Rule-compile refuse: eval `expr` to a quoted `WatAST`, then run it through the compile pass
/// `lower()` (above) for validation only — the built `Program` is discarded (`Ok(Value::Unit)`)
/// and nothing about it outlives this call. Raises (via `LowerError::into_eval`) iff `lower`
/// refuses the form (an unsupported head, a non-lexical HOF callee, or an unbound symbol);
/// returns `nil` on success. `#49 — rule-compile refuse: lower the where expr or raise.`
///
/// Arc 255 Stone P6-c-W5c — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here.
///
/// **Purity ground:** `eval_inner` on `expr` is ordinary call-by-value argument evaluation (not
/// itself an effect — the same shape `alpha-match`'s wrapper is Pure for). `lower()` is a pure
/// static compile pass: it reads `sym: &SymbolTable` (never mutates it) and `lower_expr`/
/// `lower_rete_defn`/`lower_named_rete_fn` never call `eval_inner` or `apply_function` — no user
/// code is EXECUTED, only walked and translated to `Expr`/`Program` IR. The `LowerCx` (slot
/// table, next-slot counter) and the resulting `Program` are both freshly allocated per call and
/// dropped when `eval_lower` returns (the `Ok(Value::Unit)` discards the `Program` outright) —
/// nothing is cached, interned, or otherwise retained past the call.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      ControlFlow
/// @arg     expr :wat::WatAST the quoted expression to validate-and-lower (from `:wat::core::quote`)
/// @ret     :wat::core::nil `nil` on a successful lower; raises if `lower` refuses the form
/// @example (:wat::rete::lower (:wat::core::quote (:wat::rete::i64::> ?c 5))) #=> nil
#[wat_intrinsic(":wat::rete::lower")]
pub(crate) fn eval_lower(
    expr: &WatAST,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let v = crate::runtime::eval_inner(expr, env, sym)?.value_owned();
    let ast = match v {
        Value::wat__WatAST(a) => (*a).clone(),
        other => {
            return Err(RuntimeError::new(
                expr.span().clone(),
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

#[cfg(test)]
mod rete_ops_native_coverage {
    use super::*;

    /// BRIEF-native-where-vsa-ops: the four holon rows native-lower to
    /// Call / CallFallback and must have an `OpExec` arm.
    ///
    /// ⚠ **THIS DOC USED TO SAY "`PersistentMap/contains-key?` is still Unknown — do not widen
    /// this gate into that hole", AND THAT SENTENCE WAS THE WHOLE DEFECT.** The row was fully
    /// reasoned into `RETE_OPS` (its two exits audited in the table's own row comment) and then
    /// never wired here, so it passed admission, totality, arity and type — and raised
    /// `#wat.runtime/MalformedForm "compiled apply cannot dispatch kind Unknown arity 2"` at
    /// RUNTIME, inside a `where` fence, for any user who wrote it. A comment instructing a gate
    /// not to look is not a scope note; it is an unowned deferral with no re-read (FM 23), and
    /// nothing would ever have surfaced it.
    ///
    /// Found 2026-08-28 by the § 4.1 reachability ledger (`rete/reachability.rs`), which drives
    /// each row rather than reading about it. Four more of the same shape were found the same day:
    /// the `PersistentMap` CONSTRUCTOR (fixed, `PmNew`), `reduce` (fixed, `exec_reduce` — a mirror
    /// of `wat/seq.wat:317-329`, where 3-arity reduce IS `foldl`), and `map`/`filter`/`Tuple`,
    /// which need a ruling rather than an arm.
    ///
    /// ⛔ **THIS GATE IS NOW THE NARROW ONE, AND THAT IS FINE — DO NOT WIDEN IT HERE.** Not for the
    /// old reason (a hole nobody wanted to look at) but because the general question is answered
    /// STRICTLY BETTER next door: `reachability.rs` DRIVES every row and requires a verdict, where
    /// this can only ask whether an `OpExec` arm exists. Arm-existence is the wrong question — it
    /// is neither necessary (`foldl` maps to `Unknown` and reaches the executor by its own route)
    /// nor sufficient (an arm can exist and the row still be unwritable in every position). Keep
    /// this one as the cheap holon-specific check it has always been; the wall is the ledger.
    #[test]
    fn holon_rete_ops_have_opexec() {
        let mut missing = Vec::new();
        for row in RETE_OPS {
            if row.rete_name.starts_with(":wat::rete::holon::")
                && matches!(OpExec::of(row.core_name), OpExec::Unknown)
            {
                missing.push(row.rete_name);
            }
        }
        assert!(
            missing.is_empty(),
            "native apply_op has no OpExec for holon row {missing:?}"
        );
    }
}


/// DISCONFIRMING PROBE for fix-list entry **F** — can a lowered `Expr::Call` be evaluated against
/// an ALPHA slot frame?
///
/// Entry F is: an inline constraint whose operand is a nested call is accepted everywhere, runs,
/// and matches nothing — silently. Three places conspire, and the fix hinges on ONE assumption:
/// that `compiled_cond`'s slot frame and this module's `exec` are the same thing. If they are, the
/// fix is to finish flip 3 (lower the operand through the core); if they are not, the whole
/// approach dies here and a different one is needed.
///
/// `compiled_cond::SlotFrame` is `Vec<Option<Value>>`; `exec` takes `&mut [Option<Value>]`. This
/// probe asserts they compose in fact and not merely in type: lower a nested call whose operand is
/// a `?var`, put a value in that slot by hand the way an `Op::Bind` prologue would, and demand the
/// arithmetic.
///
/// ⚠ What it does NOT settle, deliberately: `exec` requires a `&SymbolTable` and NO alpha executor
/// signature carries one — the per-fact hot path is sym-free on purpose. That is the real obstacle
/// and it is a separate decision (thread it, or refuse the sym-needing ops at compile time). This
/// probe uses a bare world's symbols to isolate the frame question from the sym question.
#[cfg(test)]
mod entry_f_frame_composition {
    use super::*;

    #[test]
    fn a_lowered_call_evaluates_against_a_bare_alpha_style_slot_frame() {
        let world = crate::freeze::startup_bare().expect("bare world");
        let sym = world.symbols();

        // `(:wat::rete::i64::+ ?x 2 :undefined 0)` — the exact shape an inline constraint
        // operand takes: a Fallback row with the mandatory `:undefined` marker pair.
        let src = "(:wat::rete::i64::+ ?x 2 :undefined 0)";
        let forms = crate::parser::parse_all_with_file(src, "<entry-f-probe>")
            .expect("the probe expression must parse");
        let expr_ast = forms.first().expect("one form");

        let program = lower(expr_ast, sym).expect("a nested rete call must lower through the core");

        // The alpha prologue's job, done by hand: `?x`'s slot holds 10, exactly as `Op::Bind`
        // would have written a field value into `scratch`.
        let slot = program
            .reads
            .iter()
            .find_map(|(name, s)| match name {
                Value::String(n) if n.as_str() == "?x" => Some(*s),
                _ => None,
            })
            .expect("the lowered program must read `?x` from a slot");
        let mut frame: Vec<Option<Value>> = vec![None; program.frame_len as usize];
        frame[slot as usize] = Some(Value::i64(10));

        let got = exec(&program.root, &mut frame, &program.names, sym, &expr_ast.span().clone())
            .expect("exec must evaluate the call against the frame");

        assert_eq!(
            got,
            Value::i64(12),
            "10 + 2 = 12. If this fails, `compiled_cond`'s slot frame and this module's `exec` do \
             NOT compose, and entry F's fix cannot be 'finish flip 3' — it needs a different shape"
        );
    }
}
