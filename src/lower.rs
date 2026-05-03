//! WatAST → HolonAST lowering for the algebra-core subset.
//!
//! This module handles exactly the six algebra-core forms — `Atom`,
//! `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend` — plus the literal
//! and keyword forms they accept as leaves. Anything else (a `define`, a
//! `let`, a user-defined call) is rejected at this stage; the eventual
//! full runtime will walk those through macro-expansion, name-resolution,
//! and type-checking before lowering is reached.
//!
//! # What's handled
//!
//! - `(:wat::holon::Atom <literal>)` — lowers to the matching typed leaf
//!   (`HolonAST::i64/f64/bool_/string`) per arc 057, or `HolonAST::keyword`
//!   for a keyword literal.
//! - `(:wat::holon::Bind a b)` — both args recursively lowered.
//! - `(:wat::holon::Bundle (:wat::core::vec ...))` — list form required;
//!   children recursively lowered.
//! - `(:wat::holon::Permute child k)` — `k` must be an integer literal
//!   (fits in `i32`).
//! - `(:wat::holon::Thermometer value min max)` — three float literals.
//! - `(:wat::holon::Blend a b w1 w2)` — two holons and two float/int
//!   literal weights.
//!
//! # What's NOT handled (yet)
//!
//! - Symbol references (a let-bound name, a define-registered function,
//!   a lambda parameter) — requires resolution, which isn't in this
//!   slice.
//! - Stdlib macros (`:wat::holon::Subtract`, `:wat::holon::Log`, etc.) — require
//!   macro expansion before lowering reaches them.
//! - Language forms (`define`, `lambda`, `let`, `if`, etc.) — require
//!   an evaluator, not just a lowering pass.
//!
//! Anything unsupported returns a [`LowerError`] naming the form.

use crate::ast::WatAST;
use crate::span::Span;
use holon::HolonAST;
use std::fmt;

/// Prefix `"<file>:<line>:<col>: "` when span is known; empty string
/// when unknown. Mirrors `src/macros.rs::span_prefix` and
/// `src/types.rs::span_prefix` exactly.
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() {
        String::new()
    } else {
        format!("{}: ", span)
    }
}

/// Lower error — the parsed form isn't an algebra-core expression this
/// MVP lowering can handle.
#[derive(Debug, Clone, PartialEq)]
pub enum LowerError {
    /// An `Atom` expected one literal argument; got zero or more than one.
    AtomArity(usize, Span),
    /// An `Atom` argument wasn't a literal (it was a list, a symbol, etc.).
    AtomNonLiteral(Span),
    /// A `Bind` expected two arguments; got some other count.
    BindArity(usize, Span),
    /// A `Bundle` expected exactly one list argument `(:wat::core::vec ...)`.
    BundleShape(Span),
    /// A `Permute` expected two arguments (child, integer step).
    PermuteArity(usize, Span),
    /// A `Permute` step wasn't an integer literal.
    PermuteStepNotInt(Span),
    /// A `Permute` integer step didn't fit in `i32`.
    PermuteStepOverflow(i64, Span),
    /// A `Thermometer` expected three numeric literal arguments.
    ThermometerShape(Span),
    /// A `Blend` expected two holons + two numeric weights.
    BlendShape(Span),
    /// An UpperCall head wasn't a supported algebra-core keyword.
    UnsupportedUpperCall(String, Span),
    /// A form isn't an algebra-core call or a literal — the MVP lowering
    /// can't handle it (e.g., a bare Symbol, a `define`, a `let`).
    UnsupportedForm(String, Span),
    /// An algebra-core call must be a List starting with a Keyword.
    MalformedCall(Span),
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::AtomArity(n, span) => write!(
                f,
                "{}(:wat::holon::Atom ...) expects exactly one literal argument; got {}",
                span_prefix(span),
                n
            ),
            LowerError::AtomNonLiteral(span) => write!(
                f,
                "{}(:wat::holon::Atom ...) argument must be a literal (int/float/bool/string/keyword)",
                span_prefix(span)
            ),
            LowerError::BindArity(n, span) => write!(
                f,
                "{}(:wat::holon::Bind ...) expects exactly two arguments; got {}",
                span_prefix(span),
                n
            ),
            LowerError::BundleShape(span) => write!(
                f,
                "{}(:wat::holon::Bundle ...) expects (:wat::core::vec ...) as its single argument",
                span_prefix(span)
            ),
            LowerError::PermuteArity(n, span) => write!(
                f,
                "{}(:wat::holon::Permute ...) expects two arguments (child, integer step); got {}",
                span_prefix(span),
                n
            ),
            LowerError::PermuteStepNotInt(span) => write!(
                f,
                "{}(:wat::holon::Permute ...) step must be an integer literal",
                span_prefix(span)
            ),
            LowerError::PermuteStepOverflow(n, span) => write!(
                f,
                "{}(:wat::holon::Permute ...) integer step {} does not fit in i32",
                span_prefix(span),
                n
            ),
            LowerError::ThermometerShape(span) => write!(
                f,
                "{}(:wat::holon::Thermometer ...) expects three numeric literal arguments: value, min, max",
                span_prefix(span)
            ),
            LowerError::BlendShape(span) => write!(
                f,
                "{}(:wat::holon::Blend ...) expects two holons and two numeric weights (a b w1 w2)",
                span_prefix(span)
            ),
            LowerError::UnsupportedUpperCall(head, span) => write!(
                f,
                "{}unsupported algebra-core form: {} — MVP handles only Atom, Bind, Bundle, Permute, Thermometer, Blend",
                span_prefix(span),
                head
            ),
            LowerError::UnsupportedForm(kind, span) => write!(
                f,
                "{}MVP lowering does not handle {} — macro expansion, name resolution, and type checking land in later slices",
                span_prefix(span),
                kind
            ),
            LowerError::MalformedCall(span) => write!(
                f,
                "{}algebra-core call must be a list whose first element is a keyword",
                span_prefix(span)
            ),
        }
    }
}

impl std::error::Error for LowerError {}

/// Lower a WatAST expression to a HolonAST.
///
/// Only algebra-core UpperCalls and their literal leaves are supported.
pub fn lower(ast: &WatAST) -> Result<HolonAST, LowerError> {
    match ast {
        WatAST::List(items, _) => lower_call(items),
        // Pattern B — ast.span() is the form's own span
        WatAST::IntLit(_, span) | WatAST::FloatLit(_, span) | WatAST::BoolLit(_, span)
        | WatAST::StringLit(_, span) | WatAST::Keyword(_, span) => Err(LowerError::UnsupportedForm(
            "bare literal outside of an (:wat::holon::...) call".into(),
            span.clone(),
        )),
        // Pattern B — ast.span() is the form's own span
        WatAST::Symbol(ident, span) => Err(LowerError::UnsupportedForm(format!(
            "bare symbol '{}' (requires name resolution)",
            ident.as_str()
        ), span.clone())),
    }
}

/// Lower a parenthesized form whose head is expected to be an algebra-core
/// keyword.
fn lower_call(items: &[WatAST]) -> Result<HolonAST, LowerError> {
    // arc 138: no span — empty list has no head element; no AST node to read span from
    let head = items.first().ok_or(LowerError::MalformedCall(Span::unknown()))?;
    let head_name = match head {
        // Pattern D — head keyword span
        WatAST::Keyword(k, head_span) => {
            let _ = head_span; // span available below via head.span()
            k.as_str()
        }
        // Pattern B — non-keyword head's span
        _ => return Err(LowerError::MalformedCall(head.span().clone())),
    };
    let args = &items[1..];
    // Pattern D — head keyword span for all dispatch arms
    let head_span = head.span().clone();

    match head_name {
        ":wat::holon::Atom" => lower_atom(args, head_span),
        ":wat::holon::Bind" => lower_bind(args, head_span),
        ":wat::holon::Bundle" => lower_bundle(args, head_span),
        ":wat::holon::Permute" => lower_permute(args, head_span),
        ":wat::holon::Thermometer" => lower_thermometer(args, head_span),
        ":wat::holon::Blend" => lower_blend(args, head_span),
        // Pattern D — unsupported call: head keyword is the best span
        other => Err(LowerError::UnsupportedUpperCall(other.to_string(), head_span)),
    }
}

fn lower_atom(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 1 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError::AtomArity(args.len(), head_span));
    }
    atom_from_literal(&args[0])
}

fn atom_from_literal(lit: &WatAST) -> Result<HolonAST, LowerError> {
    // Per arc 057, primitives ARE HolonAST — atoms lower to the typed
    // leaf variant directly, not through a polymorphic dyn-Any wrapper.
    match lit {
        WatAST::IntLit(n, _) => Ok(HolonAST::i64(*n)),
        WatAST::FloatLit(x, _) => Ok(HolonAST::f64(*x)),
        WatAST::BoolLit(b, _) => Ok(HolonAST::bool_(*b)),
        WatAST::StringLit(s, _) => Ok(HolonAST::string(s.as_str())),
        WatAST::Keyword(k, _) => Ok(HolonAST::keyword(k)),
        // Pattern A — argument's own span
        _ => Err(LowerError::AtomNonLiteral(lit.span().clone())),
    }
}

fn lower_bind(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError::BindArity(args.len(), head_span));
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    Ok(HolonAST::bind(a, b))
}

fn lower_bundle(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    // Expect exactly one argument: a (:wat::core::vec :T item ...) form.
    // Typed form per 2026-04-19: the :T arg after the keyword is skipped
    // at lower time (it's for the checker).
    if args.len() != 1 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError::BundleShape(head_span.clone()));
    }
    let list_items = match &args[0] {
        WatAST::List(items, list_span) => {
            // Pattern B — inner list's span for inner shape errors
            let head = items.first().ok_or_else(|| LowerError::BundleShape(list_span.clone()))?;
            match head {
                WatAST::Keyword(k, _)
                    if k == ":wat::core::vec" || k == ":wat::core::Vector" =>
                {
                    if items.len() < 2 {
                        return Err(LowerError::BundleShape(list_span.clone()));
                    }
                    if !matches!(&items[1], WatAST::Keyword(_, _)) {
                        return Err(LowerError::BundleShape(list_span.clone()));
                    }
                    &items[2..]
                }
                // Pattern A — unexpected head's span
                _ => return Err(LowerError::BundleShape(head.span().clone())),
            }
        }
        // Pattern A — non-list argument's span
        arg => return Err(LowerError::BundleShape(arg.span().clone())),
    };
    let children: Result<Vec<_>, _> = list_items.iter().map(lower).collect();
    Ok(HolonAST::bundle(children?))
}

fn lower_permute(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError::PermuteArity(args.len(), head_span));
    }
    let child = lower(&args[0])?;
    let k: i32 = match &args[1] {
        // Pattern A — step argument's span
        WatAST::IntLit(n, step_span) => {
            i32::try_from(*n).map_err(|_| LowerError::PermuteStepOverflow(*n, step_span.clone()))?
        }
        // Pattern A — step argument's span (non-int)
        step_arg => return Err(LowerError::PermuteStepNotInt(step_arg.span().clone())),
    };
    Ok(HolonAST::permute(child, k))
}

fn lower_thermometer(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 3 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError::ThermometerShape(head_span.clone()));
    }
    // Pattern A — first bad argument's span; fall back to head_span if all ok_or
    let value = numeric(&args[0]).ok_or_else(|| LowerError::ThermometerShape(args[0].span().clone()))?;
    let min = numeric(&args[1]).ok_or_else(|| LowerError::ThermometerShape(args[1].span().clone()))?;
    let max = numeric(&args[2]).ok_or_else(|| LowerError::ThermometerShape(args[2].span().clone()))?;
    Ok(HolonAST::thermometer(value, min, max))
}

fn lower_blend(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 4 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError::BlendShape(head_span.clone()));
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    // Pattern A — first bad weight argument's span
    let w1 = numeric(&args[2]).ok_or_else(|| LowerError::BlendShape(args[2].span().clone()))?;
    let w2 = numeric(&args[3]).ok_or_else(|| LowerError::BlendShape(args[3].span().clone()))?;
    Ok(HolonAST::blend(a, b, w1, w2))
}

/// Coerce an int or float literal to `f64`.
fn numeric(ast: &WatAST) -> Option<f64> {
    match ast {
        WatAST::IntLit(n, _) => Some(*n as f64),
        WatAST::FloatLit(x, _) => Some(*x),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holon::{encode, ScalarEncoder, VectorManager};

    const D: usize = 1024;

    fn env() -> (VectorManager, ScalarEncoder) {
        (
            VectorManager::with_seed(D, 42),
            ScalarEncoder::with_seed(D, 42),
        )
    }

    #[test]
    fn lower_atom_string() {
        let ast = crate::parse_one!(r#"(:wat::holon::Atom "role")"#).unwrap();
        let holon = lower(&ast).unwrap();
        assert_eq!(holon.as_string(), Some("role"));
    }

    #[test]
    fn lower_atom_int() {
        let ast = crate::parse_one!("(:wat::holon::Atom 42)").unwrap();
        let holon = lower(&ast).unwrap();
        assert_eq!(holon.as_i64(), Some(42));
    }

    #[test]
    fn lower_atom_float() {
        let ast = crate::parse_one!("(:wat::holon::Atom 2.5)").unwrap();
        let holon = lower(&ast).unwrap();
        assert_eq!(holon.as_f64(), Some(2.5));
    }

    #[test]
    fn lower_atom_bool() {
        let ast = crate::parse_one!("(:wat::holon::Atom true)").unwrap();
        let holon = lower(&ast).unwrap();
        assert_eq!(holon.as_bool(), Some(true));
    }

    #[test]
    fn lower_atom_keyword() {
        let ast = crate::parse_one!("(:wat::holon::Atom :foo::bar)").unwrap();
        let holon = lower(&ast).unwrap();
        // Keywords lower to Symbol leaves with the leading `:` preserved.
        assert_eq!(holon.as_symbol(), Some(":foo::bar"));
    }

    #[test]
    fn lower_bind() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Bind (:wat::holon::Atom "role") (:wat::holon::Atom "filler"))"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        // Shape check: the lowered value encodes to a ternary vector.
        let (vm, se) = env();
        let v = encode(&holon, &vm, &se);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_bundle() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST (:wat::holon::Atom "a") (:wat::holon::Atom "b") (:wat::holon::Atom "c")))"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se) = env();
        let v = encode(&holon, &vm, &se);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_permute() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Permute (:wat::holon::Atom "x") 3)"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se) = env();
        let v = encode(&holon, &vm, &se);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_thermometer() {
        let ast = crate::parse_one!("(:wat::holon::Thermometer 0.5 0.0 1.0)").unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se) = env();
        let v = encode(&holon, &vm, &se);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_blend_subtract() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Blend (:wat::holon::Atom "x") (:wat::holon::Atom "y") 1 -1)"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se) = env();
        let v = encode(&holon, &vm, &se);
        assert_eq!(v.dimensions(), D);
    }

    // ─── Error cases ────────────────────────────────────────────────────

    #[test]
    fn atom_wrong_arity() {
        let ast = crate::parse_one!(r#"(:wat::holon::Atom "a" "b")"#).unwrap();
        // mandatory compile fix: AtomArity now carries (usize, Span)
        assert!(matches!(lower(&ast), Err(LowerError::AtomArity(2, _))));
    }

    #[test]
    fn atom_non_literal_rejected() {
        // An argument that's a list, not a literal.
        let ast = crate::parse_one!(
            r#"(:wat::holon::Atom (:wat::holon::Atom "inner"))"#,
        )
        .unwrap();
        // mandatory compile fix: AtomNonLiteral now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError::AtomNonLiteral(_))));
    }

    #[test]
    fn permute_step_must_be_int() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Permute (:wat::holon::Atom "x") 1.5)"#,
        )
        .unwrap();
        // mandatory compile fix: PermuteStepNotInt now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError::PermuteStepNotInt(_))));
    }

    #[test]
    fn bundle_must_take_list_form() {
        // Bundle directly with args, not (:wat::core::vec ...).
        let ast = crate::parse_one!(
            r#"(:wat::holon::Bundle (:wat::holon::Atom "a") (:wat::holon::Atom "b"))"#,
        )
        .unwrap();
        // mandatory compile fix: BundleShape now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError::BundleShape(_))));
    }

    #[test]
    fn unsupported_upper_call() {
        let ast = crate::parse_one!(r#"(:wat::holon::MadeUp "a")"#).unwrap();
        // mandatory compile fix: UnsupportedUpperCall now carries (String, Span)
        assert!(matches!(
            lower(&ast),
            Err(LowerError::UnsupportedUpperCall(_, _))
        ));
    }

    #[test]
    fn bare_symbol_rejected() {
        let ast = crate::parse_one!("x").unwrap();
        // mandatory compile fix: UnsupportedForm now carries (String, Span)
        assert!(matches!(lower(&ast), Err(LowerError::UnsupportedForm(_, _))));
    }

    // ─── Arc 138 canary ──────────────────────────────────────────���──────

    #[test]
    fn arc138_lower_error_message_carries_span() {
        // Trigger MalformedCall — a list whose first element is not a keyword.
        // parse_one! labels spans with the real call-site Rust file:line.
        // The LowerError Display arm prefixes the span via `span_prefix`,
        // so the rendered message must contain a real source coordinate
        // (not `<test>:`).
        let ast = crate::parse_one!("(123)").unwrap(); // first element is IntLit, not Keyword
        let err = lower(&ast).unwrap_err();
        let rendered = format!("{}", err);
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "expected LowerError Display to carry real source coordinates (file:line:col); got: {}",
            rendered
        );
        assert!(
            matches!(err, LowerError::MalformedCall(_)),
            "expected MalformedCall, got: {:?}",
            err
        );
    }
}
