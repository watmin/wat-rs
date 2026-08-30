//! The one expression core — `DESIGN-STONE-the-one-expression-core.md`.
//!
//! Every rete expression in the substrate compiles and runs through this file: an inline
//! constraint, a `where` fence, a `:then` value, a user `defn` body reached from any of them. There
//! is no second path and no `Interp` arm — that is what "the ONE expression core" names.
//!
//! ── THE TWO PHASES, AND THE LINE BETWEEN THEM ────────────────────────────────────────────────
//!
//! **[`lower`] : `WatAST` → [`Program`]** happens once, at rule-compile time. It resolves every
//! binder to a numbered SLOT, resolves every head to a `RETE_OPS` row index, and refuses anything
//! it cannot represent. **[`exec`] : `Program` × frame → `Value`** happens per row, per fire, and
//! does no resolution at all.
//!
//! ⛔ **THE INVARIANT THAT MAKES THAT SPLIT WORTH IT: `lower` IS TOTAL OR IT REFUSES.** A
//! `Program` that exists is one `exec` can run — every name resolved, every arity checked, every
//! head known. `exec` therefore raises only on VALUES (an unbound slot, a wrong type, a partial
//! primitive), never on shape. **A refusal that belongs at compile time and lands at fire time is
//! a defect in this file**, because it moves a diagnostic from the rule the author is writing to
//! the millionth row of someone's data.
//!
//! That invariant is also why the lowering half is the bigger half: `lower_list` alone is the
//! dispatcher for every form the language admits, and every "cannot lower" it emits is a promise
//! that `exec` will not meet that shape.
//!
//! ── THE FRAME MODEL ──────────────────────────────────────────────────────────────────────────
//!
//! A [`Program`] carries `frame_len` slots. The caller allocates `[Option<Value>; frame_len]`,
//! writes the bound values in, and hands it to [`exec`]. Three fields describe how it is filled:
//!
//! - `reads` — token-binding key (`"?x"`) → slot. The prologue a `where` runs to seed the frame
//!   from a row's bindings; sorted, so two structurally identical programs compare equal.
//! - `params` — parameter slots in declaration order, EMPTY for a `where` program. A literal `fn`
//!   lowered inside a `where` shares the parent's slot numbering rather than restarting at zero,
//!   which is why `foldl` writes `[acc, x]` at these slots and not at `0..n`.
//! - `names` — slot → binder name, read ONLY to name an unbound slot in a diagnostic. Nothing on a
//!   successful path touches it.
//!
//! ── WHAT LIVES ON TOP ────────────────────────────────────────────────────────────────────────
//!
//! `compiled_cond.rs` and `compiled_rhs.rs` sit on this; `where_tree.rs`'s `exec_dim` is its
//! sibling for the compiled-dimension path. The wat oracle keeps its own interpreter
//! (`eval_test_core`) by the dual-impl contract — the two are compared by the differential
//! fuzzers, so a divergence here is caught as a divergence rather than as a wrong answer.

// ⛔ SPLIT AT THE AUTHOR'S OWN SEAM (`// ── exec`), 2026-08-30 — `partire`'s named cut, made.
//
// The file was 2_458 lines with `// ── exec` drawn across the middle by whoever wrote it, and the
// two halves have DIFFERENT REASONS TO CHANGE: this one moves when the language admits a new form
// (a head to lower, a pattern to bind); `exec` moves when a lowered program must run differently
// (a new opcode, a frame change). That is the partire test, and it was already answered here in a
// comment nobody acted on for a week.
//
// The IR types stay HERE, with lowering, because they ARE the thing lowering builds. `exec` reads
// them and is the consumer, which is why the dependency points one way.
// ⚠ THE FILE IS `eval.rs`, NOT `exec.rs`, and the reason is a name collision worth stating: the
// half's principal function IS `exec`, and `mod exec` would occupy `expr_ir::exec` — the path
// every caller outside this module already uses for the FUNCTION. Naming the module `eval` keeps
// `expr_ir::exec` meaning what it has always meant, so the split moves no caller.
mod eval;
pub(crate) use eval::{apply_op, eval_lower, exec, exec_call, exec_value, exec_where};


use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

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
    /// Clojure-shape hash-destructure arm — `{var :field  var2 :field2}` — binding each
    /// `var` to the subject's `field`. Carries `(bare field name, slot)` pairs.
    ///
    /// ── WHY THE FIELD NAME AND NOT A COMPILED INDEX ───────────────────────────────────────
    ///
    /// Its settled sibling `(:ns::Type/field ?x)` compiles to `Expr::Field { idx }` because the
    /// CLASS is in the accessor head, so `field_index` resolves at lower time. Here the field is
    /// in the pattern and the class is the SUBJECT's — and `LowerCx` carries no type for a slot,
    /// only names. So the index is resolved in `pat_matches` against the value's own `class`,
    /// which is the same lookup `field_index` performs, just later.
    ///
    /// That is a deliberate, stated cost rather than an oversight: rete DOES know `?p`'s declared
    /// type at validate time (`validate.rs`'s `collect_rule_bind_types`), so the index is
    /// compilable in principle — it would take threading those bind types into `LowerCx`, which is
    /// a wider change than making the form work. **If this arm ever shows on a profile, that is
    /// the fix, and it is a pure win with no semantic change.**
    Fields(Box<[(Arc<str>, u16)]>),
}

pub(crate) type PatPayload = Option<Box<Pat>>;
pub(crate) type SlotName = Option<Arc<str>>;
pub(crate) type SlotNames = Box<[SlotName]>;

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

/// Compile one expression AST into a runnable [`Program`] — the module's front door.
///
/// Allocates a slot per distinct binder as they are encountered, so slot numbers are an artifact
/// of traversal order and mean nothing outside the `Program` that owns them.
///
/// `reads` is built from the `?`-prefixed binders only — those are the ones a row supplies — and
/// is SORTED. That sort is not cosmetic: two structurally identical programs must compare equal
/// for the arm-intern table to reuse one build, and a `HashMap` iteration order would make that
/// depend on hashing.
///
/// Refuses rather than defers: see the module's totality invariant. Everything this returns `Ok`
/// for is something [`exec`] can run.
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

/// Lower any expression: literals and symbols directly, everything else through [`lower_list`].
///
/// ⛔ **THE `?` PREFIX IS A SCOPING RULE, not a naming convention.** A `?`-prefixed symbol is a
/// TOKEN BINDING supplied by the row, so it mints a slot on first sight and reuses it after —
/// which is how `?x` twice in one fence reads one value. Any other symbol must ALREADY be in scope
/// (a `let` or `fn` binder lowered earlier); if it is not, this refuses with `unbound` rather than
/// minting, because there is nothing to fill that slot from.
///
/// So the two spellings fail at different TIMES, and both do fail: a typo'd `tpyo` is refused
/// here, at rule-compile; a typo'd `?tpyo` lowers fine, mints a slot nothing writes, and raises
/// `UnboundSymbol` from [`exec`]'s `Slot` arm on the first row. Neither is silent — but only one
/// of them reaches the author while they are still looking at the rule.
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

/// Lower the FUNCTION operand of a higher-order op (`foldl`, `reduce`, `mapv`, `filterv`).
///
/// Distinct from [`lower_expr`] because a callee position admits shapes an ordinary operand does
/// not — a literal `fn`, a named rete `defn` — and refuses shapes an operand would accept. It sets
/// `cx.hof_fn_pos` so a nested call knows it is being lowered as a callee, not as a value.
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

/// THE DISPATCHER — every non-atomic form in the language arrives here and leaves as an [`Expr`]
/// or as a refusal.
///
/// Ordered deliberately: reader-desugared literals (`#holon`) fold to constants FIRST, then the
/// special forms that cannot be lowered as ordinary calls (`if`/`and`/`or`/`let`/`match`/`fn`,
/// because their operands are not all strictly evaluated), then record/variant construction, then
/// the generic `RETE_OPS` lookup. A head that reaches the end unmatched is refused by name.
///
/// ⛔ **EVERY REFUSAL HERE IS A PROMISE `exec` WILL NOT MEET THAT SHAPE** (the module's totality
/// invariant). Adding a form means adding it here, not adding an arm to `exec`.
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
    // ── `#holon <form>` IS A LITERAL, and is folded like one ──────────────────────────────────
    //
    // The reader desugars `#holon [1 2 3]` to `(:wat::holon::literal [1 2 3])`, so it arrives here
    // wearing a call's clothes and fell straight through to "cannot lower head
    // `:wat::holon::literal`". The reachability ledger recorded that outcome as
    // *"a holon has no literal spelling, so the second operand cannot be written as a constant"* —
    // **which is false.** A holon has exactly the spelling EDN has, because it holds the same data;
    // `#holon {:a 1}` is a constant in every sense that `42` and `"a"` are. The ledger measured a
    // MISSING ARM and wrote it down as an impossibility, which is the corpus fallacy this table's
    // own doctrine already refutes: absence of a caller is not evidence of absence of need.
    // Builder, 2026-08-28: *"holon is just another holder for data like edn is."*
    //
    // WHY A FOLD AND NOT A `RETE_OPS` ROW. There is nothing to dispatch. The enclosed form is DATA
    // captured without evaluation (`check.rs`'s arm says so — it deliberately does not recurse), so
    // the value is fully determined by the source text and needs no environment, no bindings and no
    // encoding context. Folding it to `Expr::Lit` at lower time is the same treatment every other
    // literal gets, and it keeps the op out of the jump table entirely — a row for a constant would
    // be a runtime dispatch that can only ever return the same value.
    //
    // TOTALITY IS NOT WEAKENED, it moves EARLIER. `to_holon_inner` is partial (a base record has no
    // lift), but here its input is a quoted literal form and any failure lands at RULE-COMPILE with
    // this span — a located diagnostic where a bad literal belongs — rather than at fire time. The
    // fence's `total?` conjunct is untouched because no row is added.
    if head == ":wat::holon::literal" {
        if items.len() != 2 {
            return Err(LowerError::unsupported(
                span.clone(),
                format!("`#holon` takes exactly one form; got {}", items.len().saturating_sub(1)),
            ));
        }
        let data = crate::runtime::eval_quote(&items[1..], span)
            .map_err(|e| LowerError::unsupported(span.clone(), format!("{e}")))?;
        let holon = crate::runtime::to_holon_inner(data, items[1].span())
            .map_err(|e| LowerError::unsupported(items[1].span().clone(), format!("{e}")))?;
        return Ok(Expr::Lit(holon));
    }

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

/// Lower a call's operands, threading `hof` so a callee position is lowered as a callee.
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

/// Lower a call whose row declares a FALLBACK — an op that may answer "no value" rather than
/// raise (`Option`-shaped rows). The fallback expression is lowered too and `exec` picks between
/// them via `classify_fallback_outcome`, the one classifier `where_tree.rs` also uses.
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

/// Lower `(let [name expr …] body)`. Each binder mints a slot BEFORE the body is lowered, so the
/// body resolves the name; bindings are sequential, and a later one may read an earlier.
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

/// Lower `(match scrutinee (pattern body)…)`. Each arm's pattern is lowered first so its binders
/// exist as slots when its body is lowered.
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

/// Lower one `match` pattern into a [`Pat`], minting a slot for every binder it introduces.
///
/// Covers literals, wildcards, binders, enum variants and hash-destructure (`Type{a, b}`). A
/// pattern's binders are in scope for that arm's body ONLY — the slots are shared with the parent
/// frame, so an arm must not read a binder another arm introduced.
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
                return Err(LowerError::unsupported(
                    span.clone(),
                    "a map in match-arm position must be the hash-destructure `{var :field …}`;                      it cannot be the HEAD of a list pattern"
                        .to_string(),
                ));
            }
            let name = option_result_tag(tag).unwrap_or_else(|| tag.to_string());
            let payload = if items.len() > 1 {
                Some(Box::new(lower_pat(&items[1], cx)?))
            } else {
                None
            };
            Ok(Pat::Variant { name, payload })
        }
        // ── `{var :field  var2 :field2}` — the hash-destructure arm ────────────────────────────
        //
        // Refused until 2026-08-28 as "match map-destructure is not lowered in v1". That was a
        // STATUS, not a reason, and it was the last `v1` refusal left in this file. Core supports
        // the form (`try_match_pattern`'s `WatAST::Map` arm, receiver-polymorphic over
        // record / struct / HashMap) and drives `:md::Point{40,2}` -> 42 through it.
        //
        // The open design question was whether this arm is genuinely different from its settled
        // sibling `(:ns::Type/field ?x)`, which compiles its field index. It is not — and rete has
        // MORE static information than core here, not less: core must dispatch on the receiver at
        // runtime because nothing declares it, while a rete `?p` gets its class from the fact
        // pattern's declared field type. See `Pat::Fields` for why the index is nonetheless
        // resolved at match time today, and exactly what compiling it would take.
        //
        // ⛔ ACCEPTS ONLY the hash-destructure, matching core's own rule. `{:keys [a b]}` and a
        // plain map literal are refused BY NAME rather than by falling through to a generic
        // "unsupported pattern", so the diagnostic teaches the supported spelling.
        WatAST::Map(pairs, span) => {
            let mut binds: Vec<(Arc<str>, u16)> = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                let WatAST::Symbol(var, _) = k else {
                    return Err(LowerError::unsupported(
                        span.clone(),
                        "a match hash-destructure binds `{var :field …}` — the key must be a bare                          variable name. `{:keys […]}` and plain map literals are not match patterns"
                            .to_string(),
                    ));
                };
                let WatAST::Keyword(field, _) = v else {
                    return Err(LowerError::unsupported(
                        span.clone(),
                        format!(
                            "a match hash-destructure binds `{{var :field …}}` — `{}` must be                              followed by a `:field` keyword",
                            var.as_str()
                        ),
                    ));
                };
                let bare = field.trim_start_matches(':');
                if bare.is_empty() {
                    return Err(LowerError::unsupported(span.clone(), "empty field name in match hash-destructure".to_string()));
                }
                binds.push((Arc::from(bare), cx.slot(var.as_str())));
            }
            if binds.is_empty() {
                return Err(LowerError::unsupported(span.clone(), "an empty map is not a match pattern".to_string()));
            }
            Ok(Pat::Fields(binds.into_boxed_slice()))
        }
        other => Err(LowerError::unsupported(other.span().clone(), "unsupported match pattern".into())),
    }
}

/// Lower a literal `(fn [params] body)` into a nested [`Program`].
///
/// ⚠ It shares the PARENT's slot numbering rather than starting a fresh frame. That is what lets
/// `foldl` write `[acc, x]` into the enclosing frame instead of allocating one per element — see
/// the module's frame model.
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

/// Lower a NAMED rete `defn`'s body into a `Program`, so a user fn called from a fence runs on
/// this core rather than through the general interpreter. Cached by the caller.
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

/// Lower record / enum-variant construction, or return `Ok(None)` if `head` names neither.
///
/// `None` is "not a constructor, keep looking" — NOT a refusal. It is what lets [`lower_list`] try
/// construction before the generic op lookup without having to know the type table itself.
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
