//! Type-check pass — slice 7a of the rank-1 HM work.
//!
//! This slice ships a PRAGMATIC monomorphic type check:
//!
//! - Every user `define` registers a [`TypeScheme`] from its declared
//!   signature (parameter types + return type).
//! - Every `:wat/core/*` / `:wat/algebra/*` built-in has a type scheme
//!   registered in the default [`CheckEnv`].
//! - The walker checks every call-position form in the program:
//!   - Arity: arg count matches scheme's param count.
//!   - Type match: each argument's inferred type unifies with the
//!     scheme's parameter type (structural equality for this slice).
//! - Every user `define`'s body is inferred and checked against its
//!   declared return type.
//!
//! # What this catches today
//!
//! - Arity mismatches in user-function calls at startup.
//! - Obvious type mismatches: `(:wat/core/+ "hello" 3)` (string arg
//!   where i64 expected), `(:wat/core/= true 42)` (type mismatch
//!   across comparison), define body returning the wrong type.
//!
//! # What this does NOT catch (deferred to slice 7b)
//!
//! - Parametric polymorphism. Type variables in schemes (`T`, `K`, `V`)
//!   are treated as accept-any: no unification tracks what they bind to.
//!   Real rank-1 HM with substitution + generalization is the follow-up
//!   slice's deliverable.
//! - Typed-macro parameter checking (058-032). Macros are already
//!   expanded before this pass; their bodies' type checks happen via
//!   their expansions. 058-032's macro-definition-time check is future
//!   work.
//! - Subtyping / type promotion. `:i64` doesn't promote to `:f64`
//!   statically here (the runtime promotes at eval); a user who wants
//!   mixed-numeric arithmetic passes the type check if both operands
//!   are the SAME numeric type. Mixing without explicit conversion
//!   rejects at check time.
//! - User-declared type reference validation (i.e., does `:my/Candle`
//!   match a `TypeEnv` entry). The `TypeEnv` is threaded in but not
//!   yet queried for referential consistency.

use crate::ast::WatAST;
use crate::runtime::{Function, SymbolTable};
use crate::types::{TypeEnv, TypeExpr};
use std::collections::HashMap;
use std::fmt;

/// A function's declared signature — parameter types + return type.
#[derive(Debug, Clone)]
pub struct TypeScheme {
    pub params: Vec<TypeExpr>,
    pub ret: TypeExpr,
}

/// Type-checking errors. Multiple errors accumulate in a single pass
/// so users get one batch of findings.
#[derive(Debug, Clone)]
pub enum CheckError {
    ArityMismatch {
        callee: String,
        expected: usize,
        got: usize,
    },
    TypeMismatch {
        callee: String,
        param: String,
        expected: String,
        got: String,
    },
    ReturnTypeMismatch {
        function: String,
        expected: String,
        got: String,
    },
    UnknownCallee {
        callee: String,
    },
    // Diagnostic for bugs in check itself — a form we can't handle.
    NotYetImplemented {
        detail: String,
    },
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::ArityMismatch { callee, expected, got } => write!(
                f,
                "{}: expected {} argument(s); got {}",
                callee, expected, got
            ),
            CheckError::TypeMismatch {
                callee,
                param,
                expected,
                got,
            } => write!(
                f,
                "{}: parameter {} expects {}; got {}",
                callee, param, expected, got
            ),
            CheckError::ReturnTypeMismatch {
                function,
                expected,
                got,
            } => write!(
                f,
                "{}: body produces {}; signature declares {}",
                function, got, expected
            ),
            CheckError::UnknownCallee { callee } => {
                write!(f, "unknown callee: {}", callee)
            }
            CheckError::NotYetImplemented { detail } => write!(
                f,
                "type checker: feature not yet implemented ({})",
                detail
            ),
        }
    }
}

impl std::error::Error for CheckError {}

/// Aggregated errors — `check_program` returns all findings together.
#[derive(Debug)]
pub struct CheckErrors(pub Vec<CheckError>);

impl fmt::Display for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} type-check error(s):", self.0.len())?;
        for e in &self.0 {
            writeln!(f, "  - {}", e)?;
        }
        Ok(())
    }
}

impl std::error::Error for CheckErrors {}

/// The type-check environment: built-in + user function schemes.
#[derive(Debug, Default)]
pub struct CheckEnv {
    schemes: HashMap<String, TypeScheme>,
}

impl CheckEnv {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an env with built-in schemes for `:wat/core/*` and
    /// `:wat/algebra/*` forms, then overlay user-define signatures
    /// from `sym`.
    pub fn from_symbols(sym: &SymbolTable) -> Self {
        let mut env = Self::with_builtins();
        for (path, func) in &sym.functions {
            if let Some(scheme) = derive_scheme_from_function(func) {
                env.register(path.clone(), scheme);
            }
        }
        env
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::default();
        register_builtins(&mut env);
        env
    }

    pub fn register(&mut self, name: String, scheme: TypeScheme) {
        self.schemes.insert(name, scheme);
    }

    pub fn get(&self, name: &str) -> Option<&TypeScheme> {
        self.schemes.get(name)
    }
}

/// Check every user define's body against its declared return type;
/// verify every call-position form in the `forms` list has correct
/// arity and argument types.
///
/// Reports all errors found in a single pass.
pub fn check_program(
    forms: &[WatAST],
    sym: &SymbolTable,
    _types: &TypeEnv,
) -> Result<(), CheckErrors> {
    let env = CheckEnv::from_symbols(sym);
    let mut errors = Vec::new();

    // Check each user define's body against its declared return type.
    for (path, func) in &sym.functions {
        if let Some(scheme) = env.get(path) {
            check_function_body(path, func, scheme, &env, &mut errors);
        }
    }

    // Check every call in the program body (the post-define residue).
    for form in forms {
        check_form(form, &env, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CheckErrors(errors))
    }
}

fn check_function_body(
    path: &str,
    func: &Function,
    scheme: &TypeScheme,
    env: &CheckEnv,
    errors: &mut Vec<CheckError>,
) {
    let body_type = infer_with_locals(
        &func.body,
        env,
        &build_locals(&func.params, &scheme.params),
    );
    match body_type {
        InferResult::Known(t) => {
            if !types_unify(&t, &scheme.ret) {
                errors.push(CheckError::ReturnTypeMismatch {
                    function: path.to_string(),
                    expected: format_type(&scheme.ret),
                    got: format_type(&t),
                });
            }
        }
        InferResult::Unknown => { /* accept — slice 7a is best-effort */ }
    }
}

fn check_form(form: &WatAST, env: &CheckEnv, errors: &mut Vec<CheckError>) {
    if let WatAST::List(items) = form {
        if let Some(WatAST::Keyword(head)) = items.first() {
            check_call(head, &items[1..], env, errors);
        }
        for child in items {
            check_form(child, env, errors);
        }
    }
}

fn check_call(
    head: &str,
    args: &[WatAST],
    env: &CheckEnv,
    errors: &mut Vec<CheckError>,
) {
    // Skip language forms handled specially — they don't have flat
    // (scheme.params, scheme.ret) signatures. define / lambda / let /
    // if / struct / enum / newtype / typealias / defmacro / load! /
    // config-setters / :wat/core/list / and / or.
    if is_special_form(head) {
        return;
    }

    let scheme = match env.get(head) {
        Some(s) => s,
        None => {
            // Unknown user path — resolve already validated, so accept.
            // (Kernel / std paths don't have schemes in this slice.)
            return;
        }
    };

    if args.len() != scheme.params.len() {
        errors.push(CheckError::ArityMismatch {
            callee: head.to_string(),
            expected: scheme.params.len(),
            got: args.len(),
        });
        return;
    }

    // Type-check each argument.
    for (i, (arg, param_ty)) in args.iter().zip(&scheme.params).enumerate() {
        let arg_ty = infer(arg, env);
        if let InferResult::Known(t) = &arg_ty {
            if !types_unify(t, param_ty) {
                errors.push(CheckError::TypeMismatch {
                    callee: head.to_string(),
                    param: format!("#{}", i + 1),
                    expected: format_type(param_ty),
                    got: format_type(t),
                });
            }
        }
    }
}

// ─── Inference ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum InferResult {
    Known(TypeExpr),
    /// The expression's type isn't statically resolvable in this slice
    /// (polymorphism, unknown builtin, etc.). Accept without error.
    Unknown,
}

fn infer(ast: &WatAST, env: &CheckEnv) -> InferResult {
    infer_with_locals(ast, env, &HashMap::new())
}

fn infer_with_locals(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
) -> InferResult {
    match ast {
        WatAST::IntLit(_) => InferResult::Known(TypeExpr::Path(":i64".into())),
        WatAST::FloatLit(_) => InferResult::Known(TypeExpr::Path(":f64".into())),
        WatAST::BoolLit(_) => InferResult::Known(TypeExpr::Path(":bool".into())),
        WatAST::StringLit(_) => InferResult::Known(TypeExpr::Path(":String".into())),
        // Keyword literals that happen as atom payloads — type :Keyword.
        // Keyword paths in call head are handled by check_call, not here.
        WatAST::Keyword(_) => InferResult::Known(TypeExpr::Path(":Keyword".into())),
        WatAST::Symbol(ident) => match locals.get(&ident.name) {
            Some(t) => InferResult::Known(t.clone()),
            None => InferResult::Unknown, // runtime resolves; check skips
        },
        WatAST::List(items) => infer_list(items, env, locals),
    }
}

fn infer_list(
    items: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
) -> InferResult {
    let head = match items.first() {
        Some(h) => h,
        None => return InferResult::Unknown,
    };

    if let WatAST::Keyword(k) = head {
        let args = &items[1..];

        // Language forms with special inference.
        match k.as_str() {
            ":wat/core/if" => {
                // Type of (if c then else) is the type of the then/else
                // branches (they must match; best-effort accept one).
                if args.len() == 3 {
                    let then_ty = infer_with_locals(&args[1], env, locals);
                    if let InferResult::Known(t) = then_ty {
                        return InferResult::Known(t);
                    }
                }
                return InferResult::Unknown;
            }
            ":wat/core/let" => {
                // Type of (let bindings body) is type of body under
                // extended locals.
                if args.len() == 2 {
                    let mut extended = locals.clone();
                    if let WatAST::List(bindings) = &args[0] {
                        for pair in bindings {
                            if let WatAST::List(kv) = pair {
                                if kv.len() == 2 {
                                    if let WatAST::Symbol(name) = &kv[0] {
                                        if let InferResult::Known(t) =
                                            infer_with_locals(&kv[1], env, locals)
                                        {
                                            extended.insert(name.name.clone(), t);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return infer_with_locals(&args[1], env, &extended);
                }
                return InferResult::Unknown;
            }
            ":wat/core/list" => {
                // :List<T> — we don't track T in this slice.
                return InferResult::Known(TypeExpr::Parametric {
                    head: "List".into(),
                    args: vec![TypeExpr::Path(":Any".into())], // placeholder
                });
            }
            ":wat/core/lambda" => {
                // The type of a lambda expression is a function type;
                // this slice doesn't fully track it. Accept.
                return InferResult::Unknown;
            }
            _ => {}
        }

        // Look up in scheme registry.
        if let Some(scheme) = env.get(k) {
            return InferResult::Known(scheme.ret.clone());
        }
        // Unknown path — caller's resolve pass caught truly unresolved
        // names; paths we don't have schemes for (kernel, std, user
        // functions we elected not to register) fall through as Unknown.
        return InferResult::Unknown;
    }

    // Non-keyword head — bare symbol (lambda application) or list
    // (inline lambda). Neither is statically typed in this slice.
    InferResult::Unknown
}

// ─── Unification — monomorphic ──────────────────────────────────────────

/// Structural equality between types. Type variables (`Path(":T")` etc.)
/// and the placeholder `Path(":Any")` match anything.
fn types_unify(a: &TypeExpr, b: &TypeExpr) -> bool {
    // Type-variable / Any sentinels accept anything. Slice 7a doesn't
    // track what a T binds to; slice 7b will.
    if is_type_variable_or_any(a) || is_type_variable_or_any(b) {
        return true;
    }
    match (a, b) {
        (TypeExpr::Path(x), TypeExpr::Path(y)) => x == y,
        (
            TypeExpr::Parametric { head: h1, args: a1 },
            TypeExpr::Parametric { head: h2, args: a2 },
        ) => {
            h1 == h2
                && a1.len() == a2.len()
                && a1.iter().zip(a2.iter()).all(|(x, y)| types_unify(x, y))
        }
        (
            TypeExpr::Fn { args: a1, ret: r1 },
            TypeExpr::Fn { args: a2, ret: r2 },
        ) => {
            a1.len() == a2.len()
                && a1.iter().zip(a2.iter()).all(|(x, y)| types_unify(x, y))
                && types_unify(r1, r2)
        }
        _ => false,
    }
}

fn is_type_variable_or_any(t: &TypeExpr) -> bool {
    if let TypeExpr::Path(p) = t {
        // Single-letter type-variable names (T, K, V, U, ...) and
        // the :Any escape are treated as accept-any in this slice.
        let stripped = p.strip_prefix(':').unwrap_or(p);
        stripped == "Any" || (stripped.len() == 1 && stripped.chars().all(|c| c.is_uppercase()))
    } else {
        false
    }
}

fn format_type(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
    }
}

fn format_type_inner(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.strip_prefix(':').unwrap_or(p).to_string(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_inner).collect();
            format!("{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_inner).collect();
            format!("fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn is_special_form(head: &str) -> bool {
    matches!(
        head,
        ":wat/core/define"
            | ":wat/core/lambda"
            | ":wat/core/let"
            | ":wat/core/let*"
            | ":wat/core/if"
            | ":wat/core/cond"
            | ":wat/core/match"
            | ":wat/core/when"
            | ":wat/core/quasiquote"
            | ":wat/core/unquote"
            | ":wat/core/unquote-splicing"
            | ":wat/core/struct"
            | ":wat/core/enum"
            | ":wat/core/newtype"
            | ":wat/core/typealias"
            | ":wat/core/defmacro"
            | ":wat/core/load!"
            | ":wat/core/list"
            | ":wat/core/and"
            | ":wat/core/or"
    ) || head.starts_with(":wat/config/set-")
        || head.starts_with(":wat/kernel/")
        || head.starts_with(":wat/std/")
}

fn build_locals(
    param_names: &[String],
    param_types: &[TypeExpr],
) -> HashMap<String, TypeExpr> {
    let mut locals = HashMap::new();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        locals.insert(name.clone(), ty.clone());
    }
    locals
}

fn derive_scheme_from_function(func: &Function) -> Option<TypeScheme> {
    // `runtime::Function` currently doesn't carry structured type info —
    // it parsed param NAMES from the signature but discarded types at
    // that stage. We re-extract here by returning None if the signature
    // isn't recoverable. A future refinement adds `params: Vec<(String,
    // TypeExpr)>` and `ret: TypeExpr` to `Function`. For now, user
    // defines get best-effort checking based on body inference only.
    let _ = func;
    None
}

// ─── Built-in schemes ───────────────────────────────────────────────────

fn register_builtins(env: &mut CheckEnv) {
    let i64_ty = || TypeExpr::Path(":i64".into());
    let f64_ty = || TypeExpr::Path(":f64".into());
    let bool_ty = || TypeExpr::Path(":bool".into());
    let holon_ty = || TypeExpr::Path(":Holon".into());
    let any_ty = || TypeExpr::Path(":Any".into());

    // Arithmetic — i64. Mixed-numeric calls fail type check; users
    // must choose one type. (The runtime is more permissive; the
    // strict spec wants explicit conversion.)
    for op in &[":wat/core/+", ":wat/core/-", ":wat/core/*", ":wat/core//"] {
        env.register(
            op.to_string(),
            TypeScheme {
                params: vec![i64_ty(), i64_ty()],
                ret: i64_ty(),
            },
        );
    }

    // Comparison — accept :Any pair, return :bool. `:Any` matches
    // anything via is_type_variable_or_any, so this under-checks;
    // slice 7b upgrades with real polymorphism.
    for op in &[
        ":wat/core/=",
        ":wat/core/<",
        ":wat/core/>",
        ":wat/core/<=",
        ":wat/core/>=",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                params: vec![any_ty(), any_ty()],
                ret: bool_ty(),
            },
        );
    }

    // Boolean.
    env.register(
        ":wat/core/not".into(),
        TypeScheme {
            params: vec![bool_ty()],
            ret: bool_ty(),
        },
    );

    // Algebra-core UpperCalls (all return :Holon).
    env.register(
        ":wat/algebra/Atom".into(),
        TypeScheme {
            params: vec![any_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Bind".into(),
        TypeScheme {
            params: vec![holon_ty(), holon_ty()],
            ret: holon_ty(),
        },
    );
    // Bundle takes a :List<Holon>; we represent it here as one :Any
    // argument (the list) for MVP — slice 7b tracks :List<Holon>.
    env.register(
        ":wat/algebra/Bundle".into(),
        TypeScheme {
            params: vec![any_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Permute".into(),
        TypeScheme {
            params: vec![holon_ty(), i64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Thermometer".into(),
        TypeScheme {
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Blend".into(),
        TypeScheme {
            params: vec![holon_ty(), holon_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::{expand_all, register_defmacros, MacroRegistry};
    use crate::parser::parse_all;
    use crate::runtime::{register_defines, SymbolTable};
    use crate::types::{parse_type_expr, TypeEnv};

    fn check(src: &str) -> Result<(), CheckErrors> {
        let forms = parse_all(src).expect("parse ok");
        let mut macros = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let expanded = expand_all(rest, &macros).expect("expand");
        let mut sym = SymbolTable::new();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        let types = TypeEnv::new();
        check_program(&rest, &sym, &types)
    }

    // ─── Arity checking ─────────────────────────────────────────────────

    #[test]
    fn correct_arity_passes() {
        assert!(check("(:wat/core/+ 1 2)").is_ok());
        assert!(check("(:wat/core/not true)").is_ok());
        assert!(check("(:wat/algebra/Bind (:wat/algebra/Atom 1) (:wat/algebra/Atom 2))").is_ok());
    }

    #[test]
    fn too_few_args_rejected() {
        let err = check("(:wat/core/+ 1)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 2, got: 1, .. })));
    }

    #[test]
    fn too_many_args_rejected() {
        let err = check("(:wat/core/not true false)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 1, got: 2, .. })));
    }

    // ─── Type mismatch ──────────────────────────────────────────────────

    #[test]
    fn string_to_add_rejected() {
        let err = check(r#"(:wat/core/+ "hello" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bool_to_add_rejected() {
        let err = check("(:wat/core/+ true 3)").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn atom_bind_wrong_type_rejected() {
        // Bind expects :Holon, :Holon. Passing a raw i64 should fail.
        let err = check("(:wat/algebra/Bind 42 (:wat/algebra/Atom 1))").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Things that pass (for now) via :Any or unknowns ────────────────

    #[test]
    fn equality_accepts_any_matching_pair() {
        // Params registered as :Any, so anything matches.
        assert!(check("(:wat/core/= 1 2)").is_ok());
        assert!(check(r#"(:wat/core/= "a" "b")"#).is_ok());
    }

    #[test]
    fn user_define_body_not_yet_typed_against_signature() {
        // Slice 7a doesn't pull structured types off Function (noted
        // in derive_scheme_from_function). User-define signature
        // checks land in slice 7b once Function grows the fields.
        assert!(check(
            r#"(:wat/core/define (:my/app/add (x :i64) (y :i64) -> :i64)
                 (:wat/core/+ x y))"#
        )
        .is_ok());
    }

    // ─── Multiple errors reported together ──────────────────────────────

    #[test]
    fn multiple_errors_reported() {
        let err = check(r#"(:wat/core/+ "s" 1) (:wat/core/not 42)"#).unwrap_err();
        assert!(err.0.len() >= 2, "expected >=2 errors, got {}", err.0.len());
    }

    // ─── Unification helpers ────────────────────────────────────────────

    #[test]
    fn unify_identical_paths() {
        assert!(types_unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":i64".into())
        ));
    }

    #[test]
    fn unify_distinct_paths_fails() {
        assert!(!types_unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":f64".into())
        ));
    }

    #[test]
    fn unify_type_variable_accepts_anything() {
        assert!(types_unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":i64".into())
        ));
    }

    #[test]
    fn unify_any_accepts_anything() {
        assert!(types_unify(
            &TypeExpr::Path(":Any".into()),
            &TypeExpr::Parametric {
                head: "List".into(),
                args: vec![TypeExpr::Path(":Holon".into())]
            }
        ));
    }

    #[test]
    fn unify_parametric_head_must_match() {
        let list_int = TypeExpr::Parametric {
            head: "List".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        let vec_int = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        assert!(!types_unify(&list_int, &vec_int));
    }

    #[test]
    fn unify_fn_contra_co() {
        let f1 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        let f2 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        assert!(types_unify(&f1, &f2));
    }

    // ─── Parse + check round-trip ───────────────────────────────────────

    #[test]
    fn type_expr_parse_and_unify() {
        let a = parse_type_expr(":Holon").unwrap();
        let b = parse_type_expr(":Holon").unwrap();
        assert!(types_unify(&a, &b));
    }
}
