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
//! - `(:wat::algebra::Atom <literal>)` — lowers to `HolonAST::atom(...)`
//!   for the Rust primitive the literal represents, or
//!   `HolonAST::keyword(...)` for a keyword literal.
//! - `(:wat::algebra::Bind a b)` — both args recursively lowered.
//! - `(:wat::algebra::Bundle (:wat::core::vec ...))` — list form required;
//!   children recursively lowered.
//! - `(:wat::algebra::Permute child k)` — `k` must be an integer literal
//!   (fits in `i32`).
//! - `(:wat::algebra::Thermometer value min max)` — three float literals.
//! - `(:wat::algebra::Blend a b w1 w2)` — two holons and two float/int
//!   literal weights.
//!
//! # What's NOT handled (yet)
//!
//! - Symbol references (a let-bound name, a define-registered function,
//!   a lambda parameter) — requires resolution, which isn't in this
//!   slice.
//! - Stdlib macros (`:wat::std::Subtract`, `:wat::std::Log`, etc.) — require
//!   macro expansion before lowering reaches them.
//! - Language forms (`define`, `lambda`, `let`, `if`, etc.) — require
//!   an evaluator, not just a lowering pass.
//!
//! Anything unsupported returns a [`LowerError`] naming the form.

use crate::ast::WatAST;
use holon::HolonAST;
use std::fmt;

/// Lower error — the parsed form isn't an algebra-core expression this
/// MVP lowering can handle.
#[derive(Debug, Clone, PartialEq)]
pub enum LowerError {
    /// An `Atom` expected one literal argument; got zero or more than one.
    AtomArity(usize),
    /// An `Atom` argument wasn't a literal (it was a list, a symbol, etc.).
    AtomNonLiteral,
    /// A `Bind` expected two arguments; got some other count.
    BindArity(usize),
    /// A `Bundle` expected exactly one list argument `(:wat::core::vec ...)`.
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

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::AtomArity(n) => write!(
                f,
                "(:wat::algebra::Atom ...) expects exactly one literal argument; got {}",
                n
            ),
            LowerError::AtomNonLiteral => write!(
                f,
                "(:wat::algebra::Atom ...) argument must be a literal (int/float/bool/string/keyword)"
            ),
            LowerError::BindArity(n) => write!(
                f,
                "(:wat::algebra::Bind ...) expects exactly two arguments; got {}",
                n
            ),
            LowerError::BundleShape => write!(
                f,
                "(:wat::algebra::Bundle ...) expects (:wat::core::vec ...) as its single argument"
            ),
            LowerError::PermuteArity(n) => write!(
                f,
                "(:wat::algebra::Permute ...) expects two arguments (child, integer step); got {}",
                n
            ),
            LowerError::PermuteStepNotInt => write!(
                f,
                "(:wat::algebra::Permute ...) step must be an integer literal"
            ),
            LowerError::PermuteStepOverflow(n) => write!(
                f,
                "(:wat::algebra::Permute ...) integer step {} does not fit in i32",
                n
            ),
            LowerError::ThermometerShape => write!(
                f,
                "(:wat::algebra::Thermometer ...) expects three numeric literal arguments: value, min, max"
            ),
            LowerError::BlendShape => write!(
                f,
                "(:wat::algebra::Blend ...) expects two holons and two numeric weights (a b w1 w2)"
            ),
            LowerError::UnsupportedUpperCall(head) => write!(
                f,
                "unsupported algebra-core form: {} — MVP handles only Atom, Bind, Bundle, Permute, Thermometer, Blend",
                head
            ),
            LowerError::UnsupportedForm(kind) => write!(
                f,
                "MVP lowering does not handle {} — macro expansion, name resolution, and type checking land in later slices",
                kind
            ),
            LowerError::MalformedCall => write!(
                f,
                "algebra-core call must be a list whose first element is a keyword"
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
        WatAST::List(items) => lower_call(items),
        WatAST::IntLit(_) | WatAST::FloatLit(_) | WatAST::BoolLit(_)
        | WatAST::StringLit(_) | WatAST::Keyword(_) => Err(LowerError::UnsupportedForm(
            "bare literal outside of an (:wat::algebra::...) call".into(),
        )),
        WatAST::Symbol(ident) => Err(LowerError::UnsupportedForm(format!(
            "bare symbol '{}' (requires name resolution)",
            ident.as_str()
        ))),
    }
}

/// Lower a parenthesized form whose head is expected to be an algebra-core
/// keyword.
fn lower_call(items: &[WatAST]) -> Result<HolonAST, LowerError> {
    let head = items.first().ok_or(LowerError::MalformedCall)?;
    let head_name = match head {
        WatAST::Keyword(k) => k.as_str(),
        _ => return Err(LowerError::MalformedCall),
    };
    let args = &items[1..];

    match head_name {
        ":wat::algebra::Atom" => lower_atom(args),
        ":wat::algebra::Bind" => lower_bind(args),
        ":wat::algebra::Bundle" => lower_bundle(args),
        ":wat::algebra::Permute" => lower_permute(args),
        ":wat::algebra::Thermometer" => lower_thermometer(args),
        ":wat::algebra::Blend" => lower_blend(args),
        other => Err(LowerError::UnsupportedUpperCall(other.to_string())),
    }
}

fn lower_atom(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    if args.len() != 1 {
        return Err(LowerError::AtomArity(args.len()));
    }
    atom_from_literal(&args[0])
}

fn atom_from_literal(lit: &WatAST) -> Result<HolonAST, LowerError> {
    match lit {
        WatAST::IntLit(n) => Ok(HolonAST::atom(*n)),
        WatAST::FloatLit(x) => Ok(HolonAST::atom(*x)),
        WatAST::BoolLit(b) => Ok(HolonAST::atom(*b)),
        WatAST::StringLit(s) => Ok(HolonAST::atom(s.clone())),
        WatAST::Keyword(k) => {
            // Stored as-is — the leading `:` is part of the canonical bytes,
            // so keywords and strings never collide (per holon-rs's Keyword
            // convention).
            Ok(HolonAST::keyword(k))
        }
        _ => Err(LowerError::AtomNonLiteral),
    }
}

fn lower_bind(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        return Err(LowerError::BindArity(args.len()));
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    Ok(HolonAST::bind(a, b))
}

fn lower_bundle(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    // Expect exactly one argument: a (:wat::core::vec ...) form.
    if args.len() != 1 {
        return Err(LowerError::BundleShape);
    }
    let list_items = match &args[0] {
        WatAST::List(items) => {
            let head = items.first().ok_or(LowerError::BundleShape)?;
            match head {
                WatAST::Keyword(k) if k == ":wat::core::vec" => &items[1..],
                _ => return Err(LowerError::BundleShape),
            }
        }
        _ => return Err(LowerError::BundleShape),
    };
    let children: Result<Vec<_>, _> = list_items.iter().map(lower).collect();
    Ok(HolonAST::bundle(children?))
}

fn lower_permute(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    if args.len() != 2 {
        return Err(LowerError::PermuteArity(args.len()));
    }
    let child = lower(&args[0])?;
    let k: i32 = match &args[1] {
        WatAST::IntLit(n) => {
            i32::try_from(*n).map_err(|_| LowerError::PermuteStepOverflow(*n))?
        }
        _ => return Err(LowerError::PermuteStepNotInt),
    };
    Ok(HolonAST::permute(child, k))
}

fn lower_thermometer(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    if args.len() != 3 {
        return Err(LowerError::ThermometerShape);
    }
    let value = numeric(&args[0]).ok_or(LowerError::ThermometerShape)?;
    let min = numeric(&args[1]).ok_or(LowerError::ThermometerShape)?;
    let max = numeric(&args[2]).ok_or(LowerError::ThermometerShape)?;
    Ok(HolonAST::thermometer(value, min, max))
}

fn lower_blend(args: &[WatAST]) -> Result<HolonAST, LowerError> {
    if args.len() != 4 {
        return Err(LowerError::BlendShape);
    }
    let a = lower(&args[0])?;
    let b = lower(&args[1])?;
    let w1 = numeric(&args[2]).ok_or(LowerError::BlendShape)?;
    let w2 = numeric(&args[3]).ok_or(LowerError::BlendShape)?;
    Ok(HolonAST::blend(a, b, w1, w2))
}

/// Coerce an int or float literal to `f64`.
fn numeric(ast: &WatAST) -> Option<f64> {
    match ast {
        WatAST::IntLit(n) => Some(*n as f64),
        WatAST::FloatLit(x) => Some(*x),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_one;
    use holon::{atom_value, encode, AtomTypeRegistry, ScalarEncoder, VectorManager};

    const D: usize = 1024;

    fn env() -> (VectorManager, ScalarEncoder, AtomTypeRegistry) {
        (
            VectorManager::with_seed(D, 42),
            ScalarEncoder::with_seed(D, 42),
            AtomTypeRegistry::with_builtins(),
        )
    }

    #[test]
    fn lower_atom_string() {
        let ast = parse_one(r#"(:wat::algebra::Atom "role")"#).unwrap();
        let holon = lower(&ast).unwrap();
        // Verify payload type by downcast through atom_value.
        let recovered: Option<&String> = atom_value(&holon);
        assert_eq!(recovered, Some(&"role".to_string()));
    }

    #[test]
    fn lower_atom_int() {
        let ast = parse_one("(:wat::algebra::Atom 42)").unwrap();
        let holon = lower(&ast).unwrap();
        let recovered: Option<&i64> = atom_value(&holon);
        assert_eq!(recovered, Some(&42_i64));
    }

    #[test]
    fn lower_atom_float() {
        let ast = parse_one("(:wat::algebra::Atom 3.14)").unwrap();
        let holon = lower(&ast).unwrap();
        let recovered: Option<&f64> = atom_value(&holon);
        assert_eq!(recovered, Some(&3.14_f64));
    }

    #[test]
    fn lower_atom_bool() {
        let ast = parse_one("(:wat::algebra::Atom true)").unwrap();
        let holon = lower(&ast).unwrap();
        let recovered: Option<&bool> = atom_value(&holon);
        assert_eq!(recovered, Some(&true));
    }

    #[test]
    fn lower_atom_keyword() {
        let ast = parse_one("(:wat::algebra::Atom :foo::bar)").unwrap();
        let holon = lower(&ast).unwrap();
        // Keyword payloads are stored as Strings with leading `:`.
        let recovered: Option<&String> = atom_value(&holon);
        assert_eq!(recovered, Some(&":foo::bar".to_string()));
    }

    #[test]
    fn lower_bind() {
        let ast = parse_one(
            r#"(:wat::algebra::Bind (:wat::algebra::Atom "role") (:wat::algebra::Atom "filler"))"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        // Shape check: the lowered value encodes to a ternary vector.
        let (vm, se, reg) = env();
        let v = encode(&holon, &vm, &se, &reg);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_bundle() {
        let ast = parse_one(
            r#"(:wat::algebra::Bundle (:wat::core::vec (:wat::algebra::Atom "a") (:wat::algebra::Atom "b") (:wat::algebra::Atom "c")))"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se, reg) = env();
        let v = encode(&holon, &vm, &se, &reg);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_permute() {
        let ast = parse_one(
            r#"(:wat::algebra::Permute (:wat::algebra::Atom "x") 3)"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se, reg) = env();
        let v = encode(&holon, &vm, &se, &reg);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_thermometer() {
        let ast = parse_one("(:wat::algebra::Thermometer 0.5 0.0 1.0)").unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se, reg) = env();
        let v = encode(&holon, &vm, &se, &reg);
        assert_eq!(v.dimensions(), D);
    }

    #[test]
    fn lower_blend_subtract() {
        let ast = parse_one(
            r#"(:wat::algebra::Blend (:wat::algebra::Atom "x") (:wat::algebra::Atom "y") 1 -1)"#,
        )
        .unwrap();
        let holon = lower(&ast).unwrap();
        let (vm, se, reg) = env();
        let v = encode(&holon, &vm, &se, &reg);
        assert_eq!(v.dimensions(), D);
    }

    // ─── Error cases ────────────────────────────────────────────────────

    #[test]
    fn atom_wrong_arity() {
        let ast = parse_one(r#"(:wat::algebra::Atom "a" "b")"#).unwrap();
        assert!(matches!(lower(&ast), Err(LowerError::AtomArity(2))));
    }

    #[test]
    fn atom_non_literal_rejected() {
        // An argument that's a list, not a literal.
        let ast = parse_one(
            r#"(:wat::algebra::Atom (:wat::algebra::Atom "inner"))"#,
        )
        .unwrap();
        assert!(matches!(lower(&ast), Err(LowerError::AtomNonLiteral)));
    }

    #[test]
    fn permute_step_must_be_int() {
        let ast = parse_one(
            r#"(:wat::algebra::Permute (:wat::algebra::Atom "x") 1.5)"#,
        )
        .unwrap();
        assert!(matches!(lower(&ast), Err(LowerError::PermuteStepNotInt)));
    }

    #[test]
    fn bundle_must_take_list_form() {
        // Bundle directly with args, not (:wat::core::vec ...).
        let ast = parse_one(
            r#"(:wat::algebra::Bundle (:wat::algebra::Atom "a") (:wat::algebra::Atom "b"))"#,
        )
        .unwrap();
        assert!(matches!(lower(&ast), Err(LowerError::BundleShape)));
    }

    #[test]
    fn unsupported_upper_call() {
        let ast = parse_one(r#"(:wat::algebra::MadeUp "a")"#).unwrap();
        assert!(matches!(
            lower(&ast),
            Err(LowerError::UnsupportedUpperCall(_))
        ));
    }

    #[test]
    fn bare_symbol_rejected() {
        let ast = parse_one("x").unwrap();
        assert!(matches!(lower(&ast), Err(LowerError::UnsupportedForm(_))));
    }
}
