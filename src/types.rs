//! Type declarations + the type environment.
//!
//! Four declaration forms per 058-030, each with a distinct head keyword:
//!
//! - `(:wat/core/struct :name (field :Type) ...)` — product type.
//! - `(:wat/core/enum :name :unit-variant (tagged-variant (field :Type)) ...)` —
//!   coproduct type.
//! - `(:wat/core/newtype :name :Inner)` — nominal wrapper.
//! - `(:wat/core/typealias :name :Expr)` — structural alias (same type,
//!   alternative name).
//!
//! Parametric polymorphism (058-030 Q1 resolved YES): the name keyword
//! may carry a `<T,U,V>` suffix declaring type parameters. Example:
//! `:my/Wrapper<T>` declares a type with one type variable `T`.
//!
//! # What this slice does
//!
//! - Classifies each declaration form at startup.
//! - Extracts the name, type parameters, and structural shape (field
//!   name/type pairs, enum variants).
//! - Parses type expressions (`:f64`, `:List<T>`, `:fn(T,U)->R`,
//!   `:my/ns/MyType`) into structured [`TypeExpr`] values.
//! - Stores the result in a [`TypeEnv`], keyed by the bare declaration
//!   name (no `<T>` in the key — parametric types are registered once;
//!   call-site instantiation is the type checker's concern, task #137).
//! - Rejects duplicate declarations and reserved-prefix names
//!   (`:wat/core/`, `:wat/kernel/`, `:wat/algebra/`, `:wat/std/`,
//!   `:wat/config/`).
//!
//! # What's deferred
//!
//! - Validation that field-type references resolve to declared types.
//!   Requires a second pass (name resolution, task #136).
//! - Parametric type instantiation at call sites (type checker,
//!   task #137).
//! - Code generation for Rust-backed compiled binaries (wat-to-rust,
//!   Track 2 of the 058 backlog).

use crate::ast::WatAST;
use std::collections::HashMap;
use std::fmt;

/// A type expression — the shape that appears after `:` in a keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// A bare type path: `:f64`, `:Holon`, `:my/ns/Candle`, `:T` (type var).
    /// The stored string DOES include a leading ':' when the expression
    /// came from a keyword token; type-variable strings from inside a
    /// `<>` parametric list do NOT have one.
    Path(String),
    /// `:List<T>`, `:HashMap<K,V>`, `:my/ns/Container<Holon,f64>`.
    Parametric {
        head: String,
        args: Vec<TypeExpr>,
    },
    /// `:fn(T,U)->R`. Function type — arguments and return.
    Fn {
        args: Vec<TypeExpr>,
        ret: Box<TypeExpr>,
    },
}

/// Struct declaration — named product type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<(String, TypeExpr)>,
}

/// Enum declaration — coproduct type. Variants are either unit
/// (payload-free) or tagged (with named typed fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariant {
    Unit(String),
    Tagged {
        name: String,
        fields: Vec<(String, TypeExpr)>,
    },
}

/// Newtype declaration — nominal wrapper distinct from its inner type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewtypeDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub inner: TypeExpr,
}

/// Typealias — structural alias for an existing type expression.
/// `:A` and its expansion are THE SAME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub expr: TypeExpr,
}

/// One of the four declaration variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
}

impl TypeDef {
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Struct(s) => &s.name,
            TypeDef::Enum(e) => &e.name,
            TypeDef::Newtype(n) => &n.name,
            TypeDef::Alias(a) => &a.name,
        }
    }
}

/// Keyword-path ↦ `TypeDef` registry.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    types: HashMap<String, TypeDef>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &TypeDef)> {
        self.types.iter()
    }

    pub fn register(&mut self, def: TypeDef) -> Result<(), TypeError> {
        let name = def.name().to_string();
        if crate::resolve::is_reserved_prefix(&name) {
            return Err(TypeError::ReservedPrefix { name });
        }
        if self.types.contains_key(&name) {
            return Err(TypeError::DuplicateType { name });
        }
        self.types.insert(name, def);
        Ok(())
    }
}

/// Type-declaration errors.
#[derive(Debug)]
pub enum TypeError {
    DuplicateType { name: String },
    ReservedPrefix { name: String },
    MalformedDecl { head: String, reason: String },
    MalformedName { raw: String, reason: String },
    MalformedField { reason: String },
    MalformedVariant { reason: String },
    MalformedTypeExpr { raw: String, reason: String },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::DuplicateType { name } => {
                write!(f, "duplicate type declaration: {}", name)
            }
            TypeError::ReservedPrefix { name } => write!(
                f,
                "type name {} uses a reserved prefix (:wat/core/, :wat/kernel/, :wat/algebra/, :wat/std/, :wat/config/); user types must use their own prefix",
                name
            ),
            TypeError::MalformedDecl { head, reason } => {
                write!(f, "malformed {} declaration: {}", head, reason)
            }
            TypeError::MalformedName { raw, reason } => {
                write!(f, "malformed type name {:?}: {}", raw, reason)
            }
            TypeError::MalformedField { reason } => {
                write!(f, "malformed field: {}", reason)
            }
            TypeError::MalformedVariant { reason } => {
                write!(f, "malformed enum variant: {}", reason)
            }
            TypeError::MalformedTypeExpr { raw, reason } => {
                write!(f, "malformed type expression {:?}: {}", raw, reason)
            }
        }
    }
}

impl std::error::Error for TypeError {}

/// Walk `forms`, register every type declaration, return the remaining
/// forms in order.
pub fn register_types(
    forms: Vec<WatAST>,
    env: &mut TypeEnv,
) -> Result<Vec<WatAST>, TypeError> {
    let mut rest = Vec::with_capacity(forms.len());
    for form in forms {
        match classify_type_decl(&form) {
            Some(head) => {
                let def = parse_type_decl(head, form)?;
                env.register(def)?;
            }
            None => rest.push(form),
        }
    }
    Ok(rest)
}

fn classify_type_decl(form: &WatAST) -> Option<&'static str> {
    if let WatAST::List(items) = form {
        if let Some(WatAST::Keyword(k)) = items.first() {
            match k.as_str() {
                ":wat/core/struct" => return Some("struct"),
                ":wat/core/enum" => return Some("enum"),
                ":wat/core/newtype" => return Some("newtype"),
                ":wat/core/typealias" => return Some("typealias"),
                _ => {}
            }
        }
    }
    None
}

fn parse_type_decl(head: &str, form: WatAST) -> Result<TypeDef, TypeError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => {
            return Err(TypeError::MalformedDecl {
                head: head.into(),
                reason: "expected list form".into(),
            })
        }
    };
    let mut iter = items.into_iter();
    let _head_kw = iter.next();
    match head {
        "struct" => parse_struct(iter.collect()),
        "enum" => parse_enum(iter.collect()),
        "newtype" => parse_newtype(iter.collect()),
        "typealias" => parse_typealias(iter.collect()),
        _ => unreachable!(),
    }
}

fn parse_struct(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    let mut iter = args.into_iter();
    let name_kw = iter.next().ok_or_else(|| TypeError::MalformedDecl {
        head: "struct".into(),
        reason: "missing name".into(),
    })?;
    let (name, type_params) = parse_declared_name("struct", &name_kw)?;
    let mut fields = Vec::new();
    for item in iter {
        fields.push(parse_field(item)?);
    }
    Ok(TypeDef::Struct(StructDef {
        name,
        type_params,
        fields,
    }))
}

fn parse_enum(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    let mut iter = args.into_iter();
    let name_kw = iter.next().ok_or_else(|| TypeError::MalformedDecl {
        head: "enum".into(),
        reason: "missing name".into(),
    })?;
    let (name, type_params) = parse_declared_name("enum", &name_kw)?;
    let mut variants = Vec::new();
    for item in iter {
        variants.push(parse_enum_variant(item)?);
    }
    if variants.is_empty() {
        return Err(TypeError::MalformedDecl {
            head: "enum".into(),
            reason: "enum must have at least one variant".into(),
        });
    }
    Ok(TypeDef::Enum(EnumDef {
        name,
        type_params,
        variants,
    }))
}

fn parse_newtype(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError::MalformedDecl {
            head: "newtype".into(),
            reason: format!(
                "expected (:wat/core/newtype :name :InnerType); got {} args",
                args.len()
            ),
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let inner_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("newtype", &name_kw)?;
    let inner = match inner_kw {
        WatAST::Keyword(k) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedDecl {
                head: "newtype".into(),
                reason: format!(
                    "inner type must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok(TypeDef::Newtype(NewtypeDef {
        name,
        type_params,
        inner,
    }))
}

fn parse_typealias(args: Vec<WatAST>) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError::MalformedDecl {
            head: "typealias".into(),
            reason: format!(
                "expected (:wat/core/typealias :name :Expr); got {} args",
                args.len()
            ),
        });
    }
    let mut iter = args.into_iter();
    let name_kw = iter.next().unwrap();
    let expr_kw = iter.next().unwrap();
    let (name, type_params) = parse_declared_name("typealias", &name_kw)?;
    let expr = match expr_kw {
        WatAST::Keyword(k) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedDecl {
                head: "typealias".into(),
                reason: format!(
                    "alias expression must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok(TypeDef::Alias(AliasDef {
        name,
        type_params,
        expr,
    }))
}

/// `(field-name :Type)` — typed field form used by structs + tagged enum variants.
fn parse_field(form: WatAST) -> Result<(String, TypeExpr), TypeError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => {
            return Err(TypeError::MalformedField {
                reason: "field must be a (name :Type) list".into(),
            })
        }
    };
    if items.len() != 2 {
        return Err(TypeError::MalformedField {
            reason: format!(
                "field must be exactly (name :Type); got {} elements",
                items.len()
            ),
        });
    }
    let mut iter = items.into_iter();
    let name = match iter.next().unwrap() {
        WatAST::Symbol(ident) => ident.name,
        other => {
            return Err(TypeError::MalformedField {
                reason: format!(
                    "field name must be a bare symbol; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    let ty = match iter.next().unwrap() {
        WatAST::Keyword(k) => parse_type_expr(&k)?,
        other => {
            return Err(TypeError::MalformedField {
                reason: format!(
                    "field type must be a keyword; got {}",
                    ast_variant_name(&other)
                ),
            })
        }
    };
    Ok((name, ty))
}

/// A variant is either a bare keyword (`:unit-variant`) or a list
/// `(tagged-variant (field :Type) ...)`.
fn parse_enum_variant(form: WatAST) -> Result<EnumVariant, TypeError> {
    match form {
        WatAST::Keyword(k) => {
            let name = k
                .strip_prefix(':')
                .ok_or_else(|| TypeError::MalformedVariant {
                    reason: format!("unit variant must be a keyword; got {:?}", k),
                })?
                .to_string();
            Ok(EnumVariant::Unit(name))
        }
        WatAST::List(items) => {
            let mut iter = items.into_iter();
            let name_sym = iter.next().ok_or_else(|| TypeError::MalformedVariant {
                reason: "tagged variant must have a name".into(),
            })?;
            let name = match name_sym {
                WatAST::Symbol(ident) => ident.name,
                WatAST::Keyword(k) => k
                    .strip_prefix(':')
                    .map(str::to_string)
                    .unwrap_or(k),
                other => {
                    return Err(TypeError::MalformedVariant {
                        reason: format!(
                            "variant name must be a symbol or keyword; got {}",
                            ast_variant_name(&other)
                        ),
                    })
                }
            };
            let mut fields = Vec::new();
            for item in iter {
                fields.push(parse_field(item)?);
            }
            Ok(EnumVariant::Tagged { name, fields })
        }
        other => Err(TypeError::MalformedVariant {
            reason: format!(
                "variant must be a keyword (unit) or list (tagged); got {}",
                ast_variant_name(&other)
            ),
        }),
    }
}

/// Parse a declared type name. Accepts:
/// - `:my/ns/MyType` → ("my/ns/MyType", [])
/// - `:my/ns/Wrapper<T>` → ("my/ns/Wrapper", ["T"])
/// - `:my/ns/Container<K,V>` → ("my/ns/Container", ["K", "V"])
fn parse_declared_name(
    head: &str,
    form: &WatAST,
) -> Result<(String, Vec<String>), TypeError> {
    let raw = match form {
        WatAST::Keyword(k) => k.clone(),
        other => {
            return Err(TypeError::MalformedDecl {
                head: head.into(),
                reason: format!(
                    "name must be a keyword; got {}",
                    ast_variant_name(other)
                ),
            })
        }
    };
    // Strip the colon but keep the rest as the key for TypeEnv.
    let stripped = raw.strip_prefix(':').ok_or_else(|| TypeError::MalformedName {
        raw: raw.clone(),
        reason: "keyword must begin with ':'".into(),
    })?;
    // Split at first '<' if present.
    match stripped.find('<') {
        None => Ok((raw, Vec::new())),
        Some(lt_index) => {
            let base = &stripped[..lt_index];
            let params_part = &stripped[lt_index..];
            if !params_part.ends_with('>') {
                return Err(TypeError::MalformedName {
                    raw: raw.clone(),
                    reason: "parametric name must close with '>'".into(),
                });
            }
            let inner = &params_part[1..params_part.len() - 1];
            let params: Vec<String> = inner
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for p in &params {
                if p.contains(char::is_whitespace) || p.contains('<') || p.contains(':') {
                    return Err(TypeError::MalformedName {
                        raw: raw.clone(),
                        reason: format!("type parameter {:?} has invalid chars", p),
                    });
                }
            }
            // Key the registry by the bare name (no <T> suffix), but
            // preserve the colon for the stored name field.
            let stored_name = format!(":{}", base);
            Ok((stored_name, params))
        }
    }
}

/// Parse a type-expression keyword into a structured [`TypeExpr`].
pub fn parse_type_expr(kw: &str) -> Result<TypeExpr, TypeError> {
    let stripped = kw.strip_prefix(':').ok_or_else(|| TypeError::MalformedTypeExpr {
        raw: kw.into(),
        reason: "type expression keyword must begin with ':'".into(),
    })?;
    parse_type_inner(stripped, kw)
}

/// Parse the content of a type keyword after the leading ':' has been
/// stripped. `original` is the full keyword string for error reporting.
fn parse_type_inner(s: &str, original: &str) -> Result<TypeExpr, TypeError> {
    // `fn(args)->ret` function type — detect at the start.
    if let Some(body) = s.strip_prefix("fn(") {
        return parse_fn_body(body, original);
    }
    // `Head<args>` parametric.
    if let Some(lt_index) = find_top_level_char(s, '<') {
        let head = s[..lt_index].to_string();
        let rest = &s[lt_index..];
        if !rest.ends_with('>') {
            return Err(TypeError::MalformedTypeExpr {
                raw: original.into(),
                reason: "parametric type must close with '>'".into(),
            });
        }
        let inside = &rest[1..rest.len() - 1];
        let args = parse_type_list(inside, original)?;
        return Ok(TypeExpr::Parametric { head, args });
    }
    // Plain path.
    Ok(TypeExpr::Path(format!(":{}", s)))
}

fn parse_fn_body(body: &str, original: &str) -> Result<TypeExpr, TypeError> {
    // body is `T,U)->R` — find the matching `)` at depth 0.
    let close = find_matching_close(body, '(', ')').ok_or_else(|| {
        TypeError::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing matching ')'".into(),
        }
    })?;
    let args_part = &body[..close];
    let tail = &body[close + 1..];
    let ret_part = tail
        .strip_prefix("->")
        .ok_or_else(|| TypeError::MalformedTypeExpr {
            raw: original.into(),
            reason: "fn type missing '->' before return".into(),
        })?;
    let args = if args_part.trim().is_empty() {
        Vec::new()
    } else {
        parse_type_list(args_part, original)?
    };
    let ret = parse_type_inner(ret_part, original)?;
    Ok(TypeExpr::Fn {
        args,
        ret: Box::new(ret),
    })
}

/// Parse a comma-separated list of types (respecting nested `<>` and `()`).
fn parse_type_list(s: &str, original: &str) -> Result<Vec<TypeExpr>, TypeError> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                let piece = &s[start..i];
                out.push(parse_type_inner(piece.trim(), original)?);
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = &s[start..];
    if !tail.trim().is_empty() {
        out.push(parse_type_inner(tail.trim(), original)?);
    }
    Ok(out)
}

/// Find the first occurrence of `c` at bracket-depth 0.
///
/// Checks the match BEFORE adjusting depth so that `c` itself being a
/// bracket (`<` or `(`) is correctly detected at the outermost level —
/// finding `<` in `List<T>` matches position 4, not None.
fn find_top_level_char(s: &str, c: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        if depth == 0 && ch == c {
            return Some(i);
        }
        match ch {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Given a string that has just consumed an `open` bracket, find the
/// byte index of the matching `close` (accounting for nesting).
fn find_matching_close(s: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1i32; // caller already consumed the opening `open`
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn ast_variant_name(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_) => "int literal",
        WatAST::FloatLit(_) => "float literal",
        WatAST::BoolLit(_) => "bool literal",
        WatAST::StringLit(_) => "string literal",
        WatAST::Keyword(_) => "keyword",
        WatAST::Symbol(_) => "symbol",
        WatAST::List(_) => "list",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_all;

    fn collect(src: &str) -> Result<(TypeEnv, Vec<WatAST>), TypeError> {
        let forms = parse_all(src).expect("parse ok");
        let mut env = TypeEnv::new();
        let rest = register_types(forms, &mut env)?;
        Ok((env, rest))
    }

    // ─── Struct ─────────────────────────────────────────────────────────

    #[test]
    fn simple_struct() {
        let (env, rest) = collect(
            r#"(:wat/core/struct :project/market/Candle
                  (open :f64)
                  (high :f64)
                  (low :f64)
                  (close :f64))"#,
        )
        .unwrap();
        assert!(rest.is_empty());
        let def = env.get(":project/market/Candle").expect("registered");
        match def {
            TypeDef::Struct(s) => {
                assert_eq!(s.name, ":project/market/Candle");
                assert!(s.type_params.is_empty());
                assert_eq!(s.fields.len(), 4);
                assert_eq!(s.fields[0].0, "open");
                assert_eq!(s.fields[0].1, TypeExpr::Path(":f64".into()));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn parametric_struct() {
        let (env, _) = collect(
            r#"(:wat/core/struct :my/Container<T>
                  (value :T)
                  (count :i64))"#,
        )
        .unwrap();
        let def = env.get(":my/Container").expect("registered");
        match def {
            TypeDef::Struct(s) => {
                assert_eq!(s.type_params, vec!["T".to_string()]);
                assert_eq!(s.fields[0].1, TypeExpr::Path(":T".into()));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn parametric_struct_multiple_params() {
        let (env, _) = collect(
            r#"(:wat/core/struct :my/Pair<K,V>
                  (key :K)
                  (value :V))"#,
        )
        .unwrap();
        let def = env.get(":my/Pair").expect("registered");
        if let TypeDef::Struct(s) = def {
            assert_eq!(s.type_params, vec!["K".to_string(), "V".to_string()]);
        } else {
            panic!("expected Struct");
        }
    }

    // ─── Enum ───────────────────────────────────────────────────────────

    #[test]
    fn unit_variant_enum() {
        let (env, _) = collect(r#"(:wat/core/enum :my/Direction :up :down :left :right)"#).unwrap();
        if let TypeDef::Enum(e) = env.get(":my/Direction").unwrap() {
            assert_eq!(e.variants.len(), 4);
            assert!(matches!(&e.variants[0], EnumVariant::Unit(n) if n == "up"));
        } else {
            panic!("expected Enum");
        }
    }

    #[test]
    fn tagged_variant_enum() {
        let (env, _) = collect(
            r#"(:wat/core/enum :my/Event
                  :empty
                  (candle (open :f64) (close :f64))
                  (deposit (amount :f64)))"#,
        )
        .unwrap();
        if let TypeDef::Enum(e) = env.get(":my/Event").unwrap() {
            assert_eq!(e.variants.len(), 3);
            assert!(matches!(&e.variants[0], EnumVariant::Unit(n) if n == "empty"));
            match &e.variants[1] {
                EnumVariant::Tagged { name, fields } => {
                    assert_eq!(name, "candle");
                    assert_eq!(fields.len(), 2);
                }
                _ => panic!(),
            }
        } else {
            panic!("expected Enum");
        }
    }

    #[test]
    fn parametric_enum() {
        let (env, _) = collect(
            r#"(:wat/core/enum :my/Option<T>
                  :none
                  (some (value :T)))"#,
        )
        .unwrap();
        if let TypeDef::Enum(e) = env.get(":my/Option").unwrap() {
            assert_eq!(e.type_params, vec!["T".to_string()]);
        } else {
            panic!();
        }
    }

    #[test]
    fn empty_enum_rejected() {
        let err = collect(r#"(:wat/core/enum :my/Empty)"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedDecl { .. }));
    }

    // ─── Newtype ────────────────────────────────────────────────────────

    #[test]
    fn simple_newtype() {
        let (env, _) = collect(r#"(:wat/core/newtype :my/trading/Price :f64)"#).unwrap();
        if let TypeDef::Newtype(n) = env.get(":my/trading/Price").unwrap() {
            assert_eq!(n.inner, TypeExpr::Path(":f64".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parametric_newtype() {
        let (env, _) = collect(r#"(:wat/core/newtype :my/Wrap<T> :T)"#).unwrap();
        if let TypeDef::Newtype(n) = env.get(":my/Wrap").unwrap() {
            assert_eq!(n.type_params, vec!["T".to_string()]);
            assert_eq!(n.inner, TypeExpr::Path(":T".into()));
        } else {
            panic!();
        }
    }

    // ─── Typealias ──────────────────────────────────────────────────────

    #[test]
    fn simple_typealias() {
        let (env, _) = collect(r#"(:wat/core/typealias :my/Amount :f64)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my/Amount").unwrap() {
            assert_eq!(a.expr, TypeExpr::Path(":f64".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parametric_typealias() {
        let (env, _) = collect(r#"(:wat/core/typealias :my/Series<T> :List<T>)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my/Series").unwrap() {
            assert_eq!(a.type_params, vec!["T".to_string()]);
            assert_eq!(
                a.expr,
                TypeExpr::Parametric {
                    head: "List".into(),
                    args: vec![TypeExpr::Path(":T".into())]
                }
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn typealias_function_type() {
        let (env, _) = collect(r#"(:wat/core/typealias :my/Predicate :fn(Holon)->bool)"#).unwrap();
        if let TypeDef::Alias(a) = env.get(":my/Predicate").unwrap() {
            match &a.expr {
                TypeExpr::Fn { args, ret } => {
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], TypeExpr::Path(":Holon".into()));
                    assert_eq!(**ret, TypeExpr::Path(":bool".into()));
                }
                other => panic!("expected Fn, got {:?}", other),
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn typealias_nested_parametric() {
        let (env, _) = collect(
            r#"(:wat/core/typealias :my/Scores :HashMap<Atom,f64>)"#,
        )
        .unwrap();
        if let TypeDef::Alias(a) = env.get(":my/Scores").unwrap() {
            match &a.expr {
                TypeExpr::Parametric { head, args } => {
                    assert_eq!(head, "HashMap");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], TypeExpr::Path(":Atom".into()));
                    assert_eq!(args[1], TypeExpr::Path(":f64".into()));
                }
                other => panic!("expected Parametric, got {:?}", other),
            }
        } else {
            panic!();
        }
    }

    // ─── Error paths ────────────────────────────────────────────────────

    #[test]
    fn duplicate_type_rejected() {
        let err = collect(
            r#"
            (:wat/core/struct :my/T (x :f64))
            (:wat/core/struct :my/T (y :i64))
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, TypeError::DuplicateType { .. }));
    }

    #[test]
    fn reserved_prefix_rejected() {
        let err = collect(r#"(:wat/core/struct :wat/core/MyStruct (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));

        let err = collect(r#"(:wat/core/struct :wat/algebra/Bad (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));

        let err = collect(r#"(:wat/core/struct :wat/std/Bad (x :f64))"#).unwrap_err();
        assert!(matches!(err, TypeError::ReservedPrefix { .. }));
    }

    #[test]
    fn malformed_newtype_arity_rejected() {
        let err = collect(r#"(:wat/core/newtype :my/T)"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedDecl { .. }));
    }

    #[test]
    fn malformed_field_rejected() {
        let err = collect(r#"(:wat/core/struct :my/T (oops))"#).unwrap_err();
        assert!(matches!(err, TypeError::MalformedField { .. }));
    }

    #[test]
    fn malformed_parametric_name_rejected() {
        let err = collect(r#"(:wat/core/struct :my/Bad<T (x :T))"#).unwrap_err();
        // `:my/Bad<T` (no closing `>`) — under the keyword-lexer rules
        // either the lexer errors out (unterminated) or the type
        // declaration complains. Either way, an error surfaces.
        assert!(matches!(err, TypeError::MalformedName { .. } | TypeError::MalformedDecl { .. }));
    }

    // ─── Non-type forms pass through ────────────────────────────────────

    #[test]
    fn non_type_forms_preserved() {
        let (_env, rest) = collect(
            r#"
            (:wat/core/struct :my/T (x :f64))
            (:wat/algebra/Atom "hello")
            42
            "#,
        )
        .unwrap();
        assert_eq!(rest.len(), 2);
    }

    // ─── TypeExpr standalone parser ─────────────────────────────────────

    #[test]
    fn type_expr_path() {
        assert_eq!(
            parse_type_expr(":f64").unwrap(),
            TypeExpr::Path(":f64".into())
        );
        assert_eq!(
            parse_type_expr(":my/ns/MyType").unwrap(),
            TypeExpr::Path(":my/ns/MyType".into())
        );
    }

    #[test]
    fn type_expr_parametric() {
        assert_eq!(
            parse_type_expr(":List<T>").unwrap(),
            TypeExpr::Parametric {
                head: "List".into(),
                args: vec![TypeExpr::Path(":T".into())]
            }
        );
    }

    #[test]
    fn type_expr_parametric_nested() {
        let t = parse_type_expr(":HashMap<String,fn(i32)->i32>").unwrap();
        match t {
            TypeExpr::Parametric { head, args } => {
                assert_eq!(head, "HashMap");
                assert_eq!(args.len(), 2);
                match &args[1] {
                    TypeExpr::Fn { args: fn_args, ret } => {
                        assert_eq!(fn_args.len(), 1);
                        assert_eq!(fn_args[0], TypeExpr::Path(":i32".into()));
                        assert_eq!(**ret, TypeExpr::Path(":i32".into()));
                    }
                    _ => panic!("expected inner fn"),
                }
            }
            _ => panic!("expected outer Parametric"),
        }
    }

    #[test]
    fn type_expr_fn_no_args() {
        let t = parse_type_expr(":fn()->Holon").unwrap();
        match t {
            TypeExpr::Fn { args, ret } => {
                assert!(args.is_empty());
                assert_eq!(*ret, TypeExpr::Path(":Holon".into()));
            }
            _ => panic!(),
        }
    }
}
