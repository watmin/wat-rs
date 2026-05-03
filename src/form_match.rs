//! `:wat::form::matches?` — Clara-style single-item pattern matcher.
//!
//! Arc 098. Pattern grammar + shared classifier consumed by both the
//! type checker (`check.rs::infer_form_matches`) and the runtime
//! (`runtime.rs::eval_form_matches`). Macros expand before type-
//! checking and can't query the struct registry, so the matcher
//! ships as a substrate-recognized special form rather than a user
//! defmacro — same shape as `:wat::core::let*` / `match` / `if`.
//!
//! The classifier is intentionally structural-only. It decides what
//! KIND of clause an AST node is; semantic validation (does the
//! field exist, is the variable bound) belongs to the walker that
//! owns the local scope. That keeps the classifier free of the
//! check/runtime split and lets the two walkers differ on what
//! "valid" means without forking the grammar.

use crate::ast::WatAST;
use crate::span::Span;

/// Six-way comparison on bound `?var`s and literals.
///
/// `=` and `not=` are the Clara-flavored equality variants; `<`,
/// `>`, `<=`, `>=` are the Clara/Clojure-traditional ordering
/// comparisons. The string form matches the wat keyword head
/// directly so error messages can reference the source name without
/// translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl CompareOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompareOp::Eq => "=",
            CompareOp::NotEq => "not=",
            CompareOp::Lt => "<",
            CompareOp::Gt => ">",
            CompareOp::Le => "<=",
            CompareOp::Ge => ">=",
        }
    }
}

/// Structural classification of a single clause inside a pattern.
///
/// `Eq` is reported as its own variant rather than folded into
/// `Compare` — the walker disambiguates between binding `(= ?var
/// :field)` and equality comparison `(= ?var "Grace")` based on
/// whether `?var` is already in scope, so it needs to inspect both
/// sides directly. The other comparisons can never be bindings.
#[derive(Debug)]
pub enum RawClause<'a> {
    /// `(= L R)` — could be a binding or an equality comparison.
    /// The walker decides based on scope.
    Eq { left: &'a WatAST, right: &'a WatAST },
    /// `(<op> L R)` for any op other than `=`. Always a comparison.
    Compare {
        op: CompareOp,
        left: &'a WatAST,
        right: &'a WatAST,
    },
    /// `(and clause ...)` — every sub-clause must hold.
    And(&'a [WatAST]),
    /// `(or clause ...)` — at least one sub-clause must hold.
    Or(&'a [WatAST]),
    /// `(not clause)` — sub-clause must NOT hold. Exactly one sub.
    Not(&'a WatAST),
    /// `(where <wat-expr>)` — escape hatch. Sub-expr must type to
    /// `:bool`; runtime evaluates it in the binding scope.
    Where(&'a WatAST),
}

/// Why a clause failed structural classification. Each variant
/// carries enough information for either side to surface a
/// diagnostic naming the offending shape.
#[derive(Debug, Clone)]
pub enum ClauseGrammarError {
    /// The clause wasn't a list — e.g. a bare literal or symbol
    /// where a `(head ...)` form was expected.
    NotAList(Span),
    /// The clause was the empty list `()`. Pattern clauses must
    /// have a head.
    EmptyList(Span),
    /// The head wasn't a keyword. Clauses always start with a
    /// keyword head (`=`, `<`, `and`, `where`, ...).
    NonKeywordHead(Span),
    /// Head keyword wasn't in the recognized vocabulary. Carries
    /// the exact head string so the walker can render it.
    UnknownHead(String, Span),
    /// `(not clause)` got a different number of args.
    NotArity { got: usize, span: Span },
    /// `(where expr)` got a different number of args.
    WhereArity { got: usize, span: Span },
    /// `(<op> L R)` got a different number of args.
    BinaryArity { op: CompareOp, got: usize, span: Span },
}

/// Prefix `"<file>:<line>:<col>: "` when span is known; empty string
/// when unknown. Mirrors `src/check.rs::span_prefix` and
/// `src/types.rs::span_prefix` exactly.
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() {
        String::new()
    } else {
        format!("{}: ", span)
    }
}

impl std::fmt::Display for ClauseGrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClauseGrammarError::NotAList(span) => {
                write!(f, "{}clause must be a list `(head ...)`", span_prefix(span))
            }
            ClauseGrammarError::EmptyList(span) => {
                write!(f, "{}empty clause `()` — clauses need a head", span_prefix(span))
            }
            ClauseGrammarError::NonKeywordHead(span) => {
                write!(f, "{}clause head must be a keyword (=, <, and, where, ...)", span_prefix(span))
            }
            ClauseGrammarError::UnknownHead(h, span) => write!(
                f,
                "{}unknown matcher head: {}; recognized: =, <, >, <=, >=, not=, and, or, not, where",
                span_prefix(span),
                h
            ),
            ClauseGrammarError::NotArity { got, span } => write!(
                f,
                "{}`not` takes exactly 1 sub-clause; got {}",
                span_prefix(span),
                got
            ),
            ClauseGrammarError::WhereArity { got, span } => write!(
                f,
                "{}`where` takes exactly 1 expression; got {}",
                span_prefix(span),
                got
            ),
            ClauseGrammarError::BinaryArity { op, got, span } => write!(
                f,
                "{}`{}` takes exactly 2 args; got {}",
                span_prefix(span),
                op.as_str(),
                got
            ),
        }
    }
}

/// Decide what kind of clause `ast` is, structurally. Pure syntax —
/// the walker handles whether bindings are fresh, whether fields
/// exist, whether `?var`s are in scope.
///
/// Notes on the bare-symbol heads (`and`, `or`, `not`, `where`) and
/// keyword heads (`:and`, etc.): the matcher accepts BOTH spellings.
/// The classifier inspects either form. This matches how Clara
/// reads — `(and ...)` is the common shape, but bare keyword paths
/// are stylistically natural in some wat sources.
pub fn classify_clause(ast: &WatAST) -> Result<RawClause<'_>, ClauseGrammarError> {
    let (items, list_span) = match ast {
        WatAST::List(items, span) => (items, span),
        _ => return Err(ClauseGrammarError::NotAList(ast.span().clone())), // Pattern A
    };
    let head = items.first().ok_or_else(|| ClauseGrammarError::EmptyList(list_span.clone()))?; // Pattern B
    let head_str = match head {
        WatAST::Keyword(k, _) => k.as_str(),
        WatAST::Symbol(ident, _) => ident.as_str(),
        _ => return Err(ClauseGrammarError::NonKeywordHead(head.span().clone())), // Pattern D
    };
    let rest = &items[1..];

    // Strip leading colon for keyword heads so `:=` matches `=`.
    // The recognized vocabulary (= < > <= >= not= and or not where)
    // is a fixed set.
    let head_norm = head_str.strip_prefix(':').unwrap_or(head_str);

    match head_norm {
        "=" => binary(rest, CompareOp::Eq, list_span.clone()).map(|(l, r)| RawClause::Eq { left: l, right: r }),
        "not=" => binary(rest, CompareOp::NotEq, list_span.clone())
            .map(|(l, r)| RawClause::Compare { op: CompareOp::NotEq, left: l, right: r }),
        "<" => binary(rest, CompareOp::Lt, list_span.clone())
            .map(|(l, r)| RawClause::Compare { op: CompareOp::Lt, left: l, right: r }),
        ">" => binary(rest, CompareOp::Gt, list_span.clone())
            .map(|(l, r)| RawClause::Compare { op: CompareOp::Gt, left: l, right: r }),
        "<=" => binary(rest, CompareOp::Le, list_span.clone())
            .map(|(l, r)| RawClause::Compare { op: CompareOp::Le, left: l, right: r }),
        ">=" => binary(rest, CompareOp::Ge, list_span.clone())
            .map(|(l, r)| RawClause::Compare { op: CompareOp::Ge, left: l, right: r }),
        "and" => Ok(RawClause::And(rest)),
        "or" => Ok(RawClause::Or(rest)),
        "not" => {
            if rest.len() != 1 {
                Err(ClauseGrammarError::NotArity { got: rest.len(), span: list_span.clone() }) // Pattern B
            } else {
                Ok(RawClause::Not(&rest[0]))
            }
        }
        "where" => {
            if rest.len() != 1 {
                Err(ClauseGrammarError::WhereArity { got: rest.len(), span: list_span.clone() }) // Pattern B
            } else {
                Ok(RawClause::Where(&rest[0]))
            }
        }
        _ => Err(ClauseGrammarError::UnknownHead(head_str.to_string(), head.span().clone())), // Pattern D
    }
}

fn binary(rest: &[WatAST], op: CompareOp, list_span: Span) -> Result<(&WatAST, &WatAST), ClauseGrammarError> {
    if rest.len() != 2 {
        return Err(ClauseGrammarError::BinaryArity { op, got: rest.len(), span: list_span }); // Pattern F: caller-propagated list span
    }
    Ok((&rest[0], &rest[1]))
}

/// True if `ast` is a bare symbol whose name starts with `?`.
/// Returns the name (with the `?` prefix included) so the caller
/// can use it as a scope key directly.
///
/// Per Q12 research (arc 098 design): wat's lexer accepts
/// `?`-prefixed symbols as bare symbols. No special tokenizer
/// support is needed; the matcher just notices the convention.
pub fn logic_var_name(ast: &WatAST) -> Option<&str> {
    match ast {
        WatAST::Symbol(ident, _) if ident.as_str().starts_with('?') => Some(ident.as_str()),
        _ => None,
    }
}

/// Extract a `:keyword` payload from an AST node, if it is one.
/// Used by the binding walker — a binding's RHS is required to be
/// a keyword that names a struct field.
pub fn keyword_payload(ast: &WatAST) -> Option<&str> {
    match ast {
        WatAST::Keyword(k, _) => Some(k.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::Identifier;
    use crate::span::Span;

    fn kw(s: &str) -> WatAST {
        WatAST::Keyword(s.into(), Span::unknown())
    }
    fn sym(s: &str) -> WatAST {
        WatAST::Symbol(Identifier::bare(s), Span::unknown())
    }
    fn list(items: Vec<WatAST>) -> WatAST {
        WatAST::List(items, Span::unknown())
    }

    #[test]
    fn classifies_eq() {
        let ast = list(vec![kw(":="), sym("?x"), kw(":foo")]);
        match classify_clause(&ast).unwrap() {
            RawClause::Eq { left, right } => {
                assert!(matches!(left, WatAST::Symbol(_, _)));
                assert!(matches!(right, WatAST::Keyword(_, _)));
            }
            _ => panic!("expected Eq"),
        }
    }

    #[test]
    fn classifies_compare() {
        let ast = list(vec![kw(":>"), sym("?x"), WatAST::FloatLit(5.0, Span::unknown())]);
        match classify_clause(&ast).unwrap() {
            RawClause::Compare { op: CompareOp::Gt, .. } => {}
            _ => panic!("expected Gt"),
        }
    }

    #[test]
    fn classifies_and_or_not() {
        let inner = list(vec![kw(":="), sym("?x"), kw(":foo")]);
        let and = list(vec![kw(":and"), inner.clone()]);
        assert!(matches!(classify_clause(&and), Ok(RawClause::And(_))));
        let or = list(vec![kw(":or"), inner.clone()]);
        assert!(matches!(classify_clause(&or), Ok(RawClause::Or(_))));
        let not = list(vec![kw(":not"), inner]);
        assert!(matches!(classify_clause(&not), Ok(RawClause::Not(_))));
    }

    #[test]
    fn classifies_where() {
        let ast = list(vec![kw(":where"), WatAST::BoolLit(true, Span::unknown())]);
        assert!(matches!(classify_clause(&ast), Ok(RawClause::Where(_))));
    }

    #[test]
    fn rejects_unknown_head() {
        let ast = list(vec![kw(":foo"), sym("?x")]);
        match classify_clause(&ast) {
            Err(ClauseGrammarError::UnknownHead(h, _)) => assert_eq!(h, ":foo"),
            other => panic!("expected UnknownHead, got {:?}", other),
        }
    }

    #[test]
    fn rejects_non_list() {
        assert!(matches!(
            classify_clause(&WatAST::IntLit(5, Span::unknown())),
            Err(ClauseGrammarError::NotAList(_))
        ));
    }

    #[test]
    fn rejects_empty_list() {
        assert!(matches!(
            classify_clause(&list(vec![])),
            Err(ClauseGrammarError::EmptyList(_))
        ));
    }

    #[test]
    fn detects_logic_var() {
        assert_eq!(logic_var_name(&sym("?outcome")), Some("?outcome"));
        assert_eq!(logic_var_name(&sym("outcome")), None);
        assert_eq!(logic_var_name(&kw(":outcome")), None);
    }

    #[test]
    fn arity_errors() {
        let bad_eq = list(vec![kw(":="), sym("?x")]);
        assert!(matches!(
            classify_clause(&bad_eq),
            Err(ClauseGrammarError::BinaryArity { op: CompareOp::Eq, got: 1, .. })
        ));
        let bad_not = list(vec![kw(":not"), sym("?a"), sym("?b")]);
        assert!(matches!(
            classify_clause(&bad_not),
            Err(ClauseGrammarError::NotArity { got: 2, .. })
        ));
        let bad_where = list(vec![kw(":where")]);
        assert!(matches!(
            classify_clause(&bad_where),
            Err(ClauseGrammarError::WhereArity { got: 0, .. })
        ));
    }

    // ─── Arc 138 canary ─────────────────────────────────────────────────

    #[test]
    fn arc138_clause_grammar_error_message_carries_span() {
        // Trigger UnknownHead via a clause with an unrecognized keyword head.
        // Build the AST using parse_all! so spans carry the real call-site
        // Rust file:line. The Display arm prefixes the span via `span_prefix`,
        // so the rendered message must contain a real source coordinate
        // (not `<test>:`).
        let forms = crate::parse_all!("(:bogus-op ?x ?y)").expect("parse ok");
        let clause = &forms[0];
        let err = classify_clause(clause).unwrap_err();
        let rendered = format!("{}", err);
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "expected ClauseGrammarError Display to carry real source coordinates (file:line:col); got: {}",
            rendered
        );
        assert!(
            matches!(err, ClauseGrammarError::UnknownHead(_, _)),
            "expected UnknownHead, got: {:?}",
            err
        );
    }
}
