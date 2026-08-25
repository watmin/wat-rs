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
//! - `(:wat::holon::Bundle (:wat::core::Vector ...))` — list form required;
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
//!   a fn parameter) — requires resolution, which isn't in this
//!   slice.
//! - Stdlib macros (`:wat::holon::Subtract`, `:wat::holon::Log`, etc.) — require
//!   macro expansion before lowering reaches them.
//! - Language forms (`define`, `fn`, `let`, `if`, etc.) — require
//!   an evaluator, not just a lowering pass.
//!
//! Anything unsupported returns a [`LowerError`] naming the form.

use crate::ast::WatAST;
use crate::span::{span_prefix, Span};
use holon::HolonAST;
use std::fmt;

/// Lower error — the parsed form isn't an algebra-core expression this
/// MVP lowering can handle. Pattern A (Stone 243.7d): span at the outer
/// struct level; variant data in `LowerErrorKind`.
#[derive(Debug, Clone, PartialEq)]
pub struct LowerError {
    pub span: Span,
    pub kind: LowerErrorKind,
}

/// Variant data for [`LowerError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug, Clone, PartialEq)]
pub enum LowerErrorKind {
    /// An `Atom` expected one literal argument; got zero or more than one.
    AtomArity(usize),
    /// An `Atom` argument wasn't a literal (it was a list, a symbol, etc.).
    AtomNonLiteral,
    /// A `Bind` expected two arguments; got some other count.
    BindArity(usize),
    /// A `Bundle` expected exactly one list argument `(:wat::core::Vector ...)`.
    BundleShape,
    /// A `Permute` expected two arguments (child, integer step).
    PermuteArity(usize),
    /// A `Permute` step wasn't an integer literal.
    PermuteStepNotInt,
    /// A `Permute` integer step didn't fit in `i32`.
    PermuteStepOverflow(i64),
    /// A `Thermometer` expected three numeric literal arguments.
    ThermometerShape,
    /// A `Blend` expected two holons + two numeric weights.
    BlendShape,
    /// An UpperCall head wasn't a supported algebra-core keyword.
    UnsupportedUpperCall(String),
    /// A form isn't an algebra-core call or a literal — the MVP lowering
    /// can't handle it (e.g., a bare Symbol, a `define`, a `let`).
    UnsupportedForm(String),
    /// An algebra-core call must be a List starting with a Keyword.
    MalformedCall,
}

impl fmt::Display for LowerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerErrorKind::AtomArity(n) => write!(
                f,
                "(:wat::holon::Atom ...) expects exactly one literal argument; got {}",
                n
            ),
            LowerErrorKind::AtomNonLiteral => write!(
                f,
                "(:wat::holon::Atom ...) argument must be a literal (int/float/bool/string/keyword)"
            ),
            LowerErrorKind::BindArity(n) => write!(
                f,
                "(:wat::holon::Bind ...) expects exactly two arguments; got {}",
                n
            ),
            LowerErrorKind::BundleShape => write!(
                f,
                "(:wat::holon::Bundle ...) expects (:wat::core::Vector ...) as its single argument"
            ),
            LowerErrorKind::PermuteArity(n) => write!(
                f,
                "(:wat::holon::Permute ...) expects two arguments (child, integer step); got {}",
                n
            ),
            LowerErrorKind::PermuteStepNotInt => write!(
                f,
                "(:wat::holon::Permute ...) step must be an integer literal"
            ),
            LowerErrorKind::PermuteStepOverflow(n) => write!(
                f,
                "(:wat::holon::Permute ...) integer step {} does not fit in i32",
                n
            ),
            LowerErrorKind::ThermometerShape => write!(
                f,
                "(:wat::holon::Thermometer ...) expects three numeric literal arguments: value, min, max"
            ),
            LowerErrorKind::BlendShape => write!(
                f,
                "(:wat::holon::Blend ...) expects two holons and two numeric weights (a b w1 w2)"
            ),
            LowerErrorKind::UnsupportedUpperCall(head) => write!(
                f,
                "unsupported algebra-core form: {} — MVP handles only Atom, Bind, Bundle, Permute, Thermometer, Blend",
                head
            ),
            LowerErrorKind::UnsupportedForm(kind) => write!(
                f,
                "MVP lowering does not handle {} — macro expansion, name resolution, and type checking land in later slices",
                kind
            ),
            LowerErrorKind::MalformedCall => write!(
                f,
                "algebra-core call must be a list whose first element is a keyword"
            ),
        }
    }
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
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
        // Arc 300 stone B — RationalLit joins the bare-literal group (no
        // algebra-core lowering, same as its IntLit/FloatLit siblings).
        // Arc 300 stone C1 — BigIntLit joins it too.
        // Arc 300 stone D — CharLit joins it too.
        WatAST::IntLit(_, span) | WatAST::FloatLit(_, span) | WatAST::RationalLit(_, span)
        | WatAST::BigIntLit(_, span)
        | WatAST::CharLit(_, span)
        | WatAST::BoolLit(_, span)
        | WatAST::StringLit(_, span) | WatAST::Keyword(_, span) => Err(LowerError { span: span.clone(), kind: LowerErrorKind::UnsupportedForm("bare literal outside of an (:wat::holon::...) call".into()) }),
        // Arc 244 — NilLit: bare nil literal has no algebra-core lowering (like its literal siblings).
        WatAST::NilLit(span) => Err(LowerError { span: span.clone(), kind: LowerErrorKind::UnsupportedForm("bare `nil` literal outside of an (:wat::holon::...) call".into()) }),
        // Pattern B — ast.span() is the form's own span
        WatAST::Symbol(ident, span) => Err(LowerError { span: span.clone(), kind: LowerErrorKind::UnsupportedForm(format!("bare symbol '{}' (requires name resolution)", ident.as_str())) }),
        // Arc 167 slice 1 — vectors aren't lowered to HolonAST.
        // The lower path only admits algebra-core UpperCalls; a
        // bracketed form has no algebra-core meaning here.
        WatAST::Vector(_, span) => Err(LowerError { span: span.clone(), kind: LowerErrorKind::UnsupportedForm("vector literal in lower() (algebra-core does not admit vector literals)".into()) }),
        // Arc 257 slice 1 — Map/Set literals have no algebra-core lowering.
        WatAST::Map(_, span) | WatAST::Set(_, span) => Err(LowerError { span: span.clone(), kind: LowerErrorKind::UnsupportedForm("map/set literal in lower() (algebra-core does not admit map/set literals)".into()) }),
    }
}

/// Lower a parenthesized form whose head is expected to be an algebra-core
/// keyword.
fn lower_call(items: &[WatAST]) -> Result<HolonAST, LowerError> {
    // arc 138: no span — empty list has no head element; no AST node to read span from
    let head = items.first().ok_or(LowerError { span: crate::rust_caller_span!(), kind: LowerErrorKind::MalformedCall })?;
    let head_name = match head {
        // Pattern D — head keyword span
        WatAST::Keyword(k, head_span) => {
            let _ = head_span; // span available below via head.span()
            k.as_str()
        }
        // Pattern B — non-keyword head's span
        _ => return Err(LowerError { span: head.span().clone(), kind: LowerErrorKind::MalformedCall }),
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
        other => Err(LowerError { span: head_span, kind: LowerErrorKind::UnsupportedUpperCall(other.to_string()) }),
    }
}

fn lower_atom(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 1 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError { span: head_span, kind: LowerErrorKind::AtomArity(args.len()) });
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
        _ => Err(LowerError { span: lit.span().clone(), kind: LowerErrorKind::AtomNonLiteral }),
    }
}

fn lower_bind(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError { span: head_span, kind: LowerErrorKind::BindArity(args.len()) });
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    Ok(HolonAST::bind(a, b))
}

fn lower_bundle(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    // Expect exactly one argument: a (:wat::core::Vector :T item ...) form.
    // Typed form per 2026-04-19: the :T arg after the keyword is skipped
    // at lower time (it's for the checker).
    if args.len() != 1 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError { span: head_span.clone(), kind: LowerErrorKind::BundleShape });
    }
    let list_items = match &args[0] {
        WatAST::List(items, list_span) => {
            // Pattern B — inner list's span for inner shape errors
            let head = items.first().ok_or_else(|| LowerError { span: list_span.clone(), kind: LowerErrorKind::BundleShape })?;
            match head {
                WatAST::Keyword(k, _)
                    if k == ":wat::core::Vector" =>
                {
                    if items.len() < 2 {
                        return Err(LowerError { span: list_span.clone(), kind: LowerErrorKind::BundleShape });
                    }
                    if !matches!(&items[1], WatAST::Keyword(_, _)) {
                        return Err(LowerError { span: list_span.clone(), kind: LowerErrorKind::BundleShape });
                    }
                    &items[2..]
                }
                // Pattern A — unexpected head's span
                _ => return Err(LowerError { span: head.span().clone(), kind: LowerErrorKind::BundleShape }),
            }
        }
        // Pattern A — non-list argument's span
        arg => return Err(LowerError { span: arg.span().clone(), kind: LowerErrorKind::BundleShape }),
    };
    let children: Result<Vec<_>, _> = list_items.iter().map(lower).collect();
    Ok(HolonAST::bundle(children?))
}

fn lower_permute(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        // Pattern D — head keyword span for arity errors
        return Err(LowerError { span: head_span, kind: LowerErrorKind::PermuteArity(args.len()) });
    }
    let child = lower(&args[0])?;
    let k: i32 = match &args[1] {
        // Pattern A — step argument's span
        WatAST::IntLit(n, step_span) => {
            i32::try_from(*n).map_err(|_| LowerError { span: step_span.clone(), kind: LowerErrorKind::PermuteStepOverflow(*n) })?
        }
        // Pattern A — step argument's span (non-int)
        step_arg => return Err(LowerError { span: step_arg.span().clone(), kind: LowerErrorKind::PermuteStepNotInt }),
    };
    Ok(HolonAST::permute(child, k))
}

fn lower_thermometer(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 3 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError { span: head_span.clone(), kind: LowerErrorKind::ThermometerShape });
    }
    // Pattern A — first bad argument's span; fall back to head_span if all ok_or
    let value = numeric(&args[0]).ok_or_else(|| LowerError { span: args[0].span().clone(), kind: LowerErrorKind::ThermometerShape })?;
    let min = numeric(&args[1]).ok_or_else(|| LowerError { span: args[1].span().clone(), kind: LowerErrorKind::ThermometerShape })?;
    let max = numeric(&args[2]).ok_or_else(|| LowerError { span: args[2].span().clone(), kind: LowerErrorKind::ThermometerShape })?;
    Ok(HolonAST::thermometer(value, min, max))
}

fn lower_blend(args: &[WatAST], head_span: Span) -> Result<HolonAST, LowerError> {
    if args.len() != 4 {
        // Pattern D — head keyword span for shape errors
        return Err(LowerError { span: head_span.clone(), kind: LowerErrorKind::BlendShape });
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    // Pattern A — first bad weight argument's span
    let w1 = numeric(&args[2]).ok_or_else(|| LowerError { span: args[2].span().clone(), kind: LowerErrorKind::BlendShape })?;
    let w2 = numeric(&args[3]).ok_or_else(|| LowerError { span: args[3].span().clone(), kind: LowerErrorKind::BlendShape })?;
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
        // Arc 221 Stone 221.3 (holon-rs fa48b39): HolonAST::keyword() now returns
        // HolonAST::Keyword (stripped of leading colon), not HolonAST::Symbol.
        // lower() calls HolonAST::keyword(k) at lower.rs:239 → Keyword variant.
        // as_keyword() returns content WITHOUT the leading colon; as_symbol() → None.
        assert_eq!(holon.as_keyword(), Some("foo::bar"));
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
            r#"(:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST (:wat::holon::Atom "a") (:wat::holon::Atom "b") (:wat::holon::Atom "c")))"#,
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
        assert!(matches!(lower(&ast), Err(LowerError { kind: LowerErrorKind::AtomArity(2), .. })));
    }

    #[test]
    fn atom_non_literal_rejected() {
        // An argument that's a list, not a literal.
        let ast = crate::parse_one!(
            r#"(:wat::holon::Atom (:wat::holon::Atom "inner"))"#,
        )
        .unwrap();
        // mandatory compile fix: AtomNonLiteral now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError { kind: LowerErrorKind::AtomNonLiteral, .. })));
    }

    #[test]
    fn permute_step_must_be_int() {
        let ast = crate::parse_one!(
            r#"(:wat::holon::Permute (:wat::holon::Atom "x") 1.5)"#,
        )
        .unwrap();
        // mandatory compile fix: PermuteStepNotInt now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError { kind: LowerErrorKind::PermuteStepNotInt, .. })));
    }

    #[test]
    fn bundle_must_take_list_form() {
        // Bundle directly with args, not (:wat::core::Vector ...).
        let ast = crate::parse_one!(
            r#"(:wat::holon::Bundle (:wat::holon::Atom "a") (:wat::holon::Atom "b"))"#,
        )
        .unwrap();
        // mandatory compile fix: BundleShape now carries (Span)
        assert!(matches!(lower(&ast), Err(LowerError { kind: LowerErrorKind::BundleShape, .. })));
    }

    #[test]
    fn unsupported_upper_call() {
        let ast = crate::parse_one!(r#"(:wat::holon::MadeUp "a")"#).unwrap();
        // mandatory compile fix: UnsupportedUpperCall now carries (String, Span)
        assert!(matches!(
            lower(&ast),
            Err(LowerError { kind: LowerErrorKind::UnsupportedUpperCall(_), .. })
        ));
    }

    #[test]
    fn bare_symbol_rejected() {
        let ast = crate::parse_one!("x").unwrap();
        // mandatory compile fix: UnsupportedForm now carries (String, Span)
        assert!(matches!(lower(&ast), Err(LowerError { kind: LowerErrorKind::UnsupportedForm(_), .. })));
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
        // rune:lint(loose-assert) — variable Rust source file path embedded in error Display output via parse_one! span (varies by build environment)
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "expected LowerError Display to carry real source coordinates (file:line:col); got: {}",
            rendered
        );
        assert!(
            matches!(err, LowerError { kind: LowerErrorKind::MalformedCall, .. }),
            "expected MalformedCall, got: {:?}",
            err
        );
    }
}
