//! Type-check pass — rank-1 Hindley-Milner.
//!
//! This slice implements real parametric polymorphism per 058-030:
//!
//! - [`TypeScheme`] carries `type_params` — the list of names that are
//!   universally quantified (e.g., `["T"]` for `∀T. T -> :Holon`).
//! - Every call site **instantiates** the scheme with fresh unification
//!   variables ([`TypeExpr::Var`]), accumulates a substitution by
//!   unifying each argument type with its (instantiated) parameter
//!   type, and produces a monomorphic return type after applying the
//!   final substitution.
//! - Within a user define's body, declared type parameters are
//!   **rigid** — they unify only with themselves, not with concrete
//!   types. The body must type-check for any choice of T.
//! - Built-in schemes use real polymorphism: `list` is `∀T. T* ->
//!   List<T>`; `= < > <= >=` are `∀T. T -> T -> :bool`; `Atom` is
//!   `∀T. T -> :Holon`.
//! - `:Any` is banned everywhere — the type universe is closed
//!   ([058-030](https://…/058-030-types/PROPOSAL.md), §591). User
//!   source containing `:Any` halts at parse (see
//!   [`crate::types::parse_type_expr`]).
//!
//! # What this catches today
//!
//! - Arity mismatches in user-function and built-in calls at startup.
//! - Type mismatches: `(:wat/core/+ "hello" 3)`, `(:wat/core/< 1 "x")`
//!   — `<` requires matching operand types.
//! - Polymorphic failures: `(:wat/core/list 1 "two" 3)` — list
//!   elements must unify to a common element type.
//! - User-define body vs signature mismatches. Rigid type params
//!   mean a body of `:i64` in a `∀T. T -> T` signature is rejected.
//!
//! # What this does NOT catch (explicitly deferred)
//!
//! - **Lambda-value call-site typing.** Lambda values don't carry
//!   structured signatures through [`crate::runtime::Function`] yet,
//!   so calling a lambda stays Unknown at the check layer.
//! - **`:Union<T,U,V>` coproduct discipline.** `:Union` is a
//!   first-class type form in the grammar; full subtype / variant
//!   checks land when stdlib needs demand them.
//! - **Typed-macro parameter checks (058-032).** Macros expand before
//!   check; macro-definition-time checks (`:AST<T>` against body
//!   positions) are future work.
//! - **Numeric promotion.** `:i64` does not promote to `:f64` statically;
//!   mixing numeric types in arithmetic is rejected.

use crate::ast::WatAST;
use crate::runtime::{Function, SymbolTable};
use crate::types::{TypeEnv, TypeExpr};
use std::collections::HashMap;
use std::fmt;

/// A function's declared signature: universally-quantified type
/// parameters, then parameter types and return type.
///
/// `type_params` names the variables that are `∀`-bound over the
/// scheme. At every use site, [`instantiate`] freshens them to unique
/// [`TypeExpr::Var`]s so multiple independent call sites don't alias.
#[derive(Debug, Clone)]
pub struct TypeScheme {
    pub type_params: Vec<String>,
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

/// Source of fresh [`TypeExpr::Var`] ids. Shared across the whole
/// `check_program` run so ids never collide across call sites or
/// function bodies.
#[derive(Debug, Default)]
struct FreshGen {
    next: u64,
}

impl FreshGen {
    fn fresh(&mut self) -> TypeExpr {
        let v = TypeExpr::Var(self.next);
        self.next += 1;
        v
    }
}

/// Substitution map: unification variable id → its concrete type.
/// Updated as unification succeeds; applied via [`apply_subst`] to
/// resolve a type to its canonical form.
type Subst = HashMap<u64, TypeExpr>;

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
    let mut fresh = FreshGen::default();

    // Check each user define's body against its declared return type.
    for (path, func) in &sym.functions {
        if let Some(scheme) = env.get(path) {
            check_function_body(path, func, scheme, &env, &mut fresh, &mut errors);
        }
    }

    // Check every call in the program body (the post-define residue).
    for form in forms {
        check_form(form, &env, &mut fresh, &mut errors);
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
    fresh: &mut FreshGen,
    errors: &mut Vec<CheckError>,
) {
    // Declared type parameters are RIGID inside the body — rigid
    // meaning they unify only with themselves. Represented as
    // `Path(":T")` where T is the declared name; the checker
    // distinguishes rigid names from fresh unification Vars.
    let locals = build_locals(&func.params, &scheme.params);
    let mut subst = Subst::new();
    let body_ty = infer(&func.body, env, &locals, fresh, &mut subst, errors);
    // Unify body type with declared return type. If unification fails,
    // produce a ReturnTypeMismatch.
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &scheme.ret, &mut subst).is_err() {
            errors.push(CheckError::ReturnTypeMismatch {
                function: path.to_string(),
                expected: format_type(&apply_subst(&scheme.ret, &subst)),
                got: format_type(&apply_subst(&body_ty, &subst)),
            });
        }
    }
}

fn check_form(
    form: &WatAST,
    env: &CheckEnv,
    fresh: &mut FreshGen,
    errors: &mut Vec<CheckError>,
) {
    let mut subst = Subst::new();
    let _ = infer(form, env, &HashMap::new(), fresh, &mut subst, errors);
}

// ─── Inference ──────────────────────────────────────────────────────────

/// Infer the type of an expression, recording errors along the way.
///
/// Returns `Some(type)` when a type can be assigned, `None` when the
/// expression's type is opaque at this layer (e.g., lambda
/// application, user symbol that isn't a known local). Errors from
/// nested calls are pushed to `errors`.
fn infer(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    match ast {
        WatAST::IntLit(_) => Some(TypeExpr::Path(":i64".into())),
        WatAST::FloatLit(_) => Some(TypeExpr::Path(":f64".into())),
        WatAST::BoolLit(_) => Some(TypeExpr::Path(":bool".into())),
        WatAST::StringLit(_) => Some(TypeExpr::Path(":String".into())),
        WatAST::Keyword(_) => Some(TypeExpr::Path(":Keyword".into())),
        WatAST::Symbol(ident) => locals.get(&ident.name).cloned(),
        WatAST::List(items) => infer_list(items, env, locals, fresh, subst, errors),
    }
}

fn infer_list(
    items: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let head = items.first()?;

    if let WatAST::Keyword(k) = head {
        let args = &items[1..];
        match k.as_str() {
            ":wat/core/if" => return infer_if(args, env, locals, fresh, subst, errors),
            ":wat/core/let" => return infer_let(args, env, locals, fresh, subst, errors),
            ":wat/core/list" => return infer_list_constructor(args, env, locals, fresh, subst, errors),
            ":wat/core/and" | ":wat/core/or" => {
                return infer_boolean_shortcircuit(args, env, locals, fresh, subst, errors);
            }
            ":wat/core/lambda" => return None,
            ":wat/core/define"
            | ":wat/core/struct"
            | ":wat/core/enum"
            | ":wat/core/newtype"
            | ":wat/core/typealias"
            | ":wat/core/defmacro"
            | ":wat/core/load!"
            | ":wat/core/digest-load!"
            | ":wat/core/signed-load!"
            | ":wat/core/quasiquote"
            | ":wat/core/unquote"
            | ":wat/core/unquote-splicing" => {
                // Top-level forms / reader-macro heads don't participate
                // in expression-level inference.
                return None;
            }
            _ if k.starts_with(":wat/config/set-") => return None,
            _ if k.starts_with(":wat/kernel/") || k.starts_with(":wat/std/") => {
                // Kernel / std paths don't have type schemes in this
                // slice; still recurse into their args so inner calls
                // get checked.
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return None;
            }
            _ => {}
        }

        // Normal call: look up scheme, instantiate, unify args.
        let scheme = match env.get(k) {
            Some(s) => s,
            None => {
                // Resolve pass validated the name; we just don't have
                // a scheme for it (e.g., user function not registered
                // in this run). Still recurse into args for nested
                // checks.
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return None;
            }
        };

        let (param_types, ret_type) = instantiate(scheme, fresh);

        if args.len() != param_types.len() {
            errors.push(CheckError::ArityMismatch {
                callee: k.clone(),
                expected: param_types.len(),
                got: args.len(),
            });
            for arg in args {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(apply_subst(&ret_type, subst));
        }

        for (i, (arg, expected)) in args.iter().zip(&param_types).enumerate() {
            let arg_ty = infer(arg, env, locals, fresh, subst, errors);
            if let Some(arg_ty) = arg_ty {
                if unify(&arg_ty, expected, subst).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: k.clone(),
                        param: format!("#{}", i + 1),
                        expected: format_type(&apply_subst(expected, subst)),
                        got: format_type(&apply_subst(&arg_ty, subst)),
                    });
                }
            }
        }
        return Some(apply_subst(&ret_type, subst));
    }

    // Non-keyword head: bare-symbol application or inline lambda. Not
    // typed at this layer; still recurse for nested call checks.
    for item in items {
        let _ = infer(item, env, locals, fresh, subst, errors);
    }
    None
}

fn infer_if(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 3 {
        return None;
    }
    // Condition must be :bool.
    let cond_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(c) = cond_ty {
        let _ = unify(&c, &TypeExpr::Path(":bool".into()), subst);
    }
    // Branches must agree.
    let then_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let else_ty = infer(&args[2], env, locals, fresh, subst, errors);
    match (then_ty, else_ty) {
        (Some(t), Some(e)) => {
            if unify(&t, &e, subst).is_ok() {
                Some(apply_subst(&t, subst))
            } else {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat/core/if".into(),
                    param: "branches".into(),
                    expected: format_type(&apply_subst(&t, subst)),
                    got: format_type(&apply_subst(&e, subst)),
                });
                None
            }
        }
        (Some(t), None) | (None, Some(t)) => Some(apply_subst(&t, subst)),
        (None, None) => None,
    }
}

fn infer_let(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let mut extended = locals.clone();
    if let WatAST::List(bindings) = &args[0] {
        for pair in bindings {
            if let WatAST::List(kv) = pair {
                if kv.len() == 2 {
                    if let WatAST::Symbol(name) = &kv[0] {
                        if let Some(t) = infer(&kv[1], env, locals, fresh, subst, errors) {
                            extended.insert(name.name.clone(), apply_subst(&t, subst));
                        }
                    }
                }
            }
        }
    }
    infer(&args[1], env, &extended, fresh, subst, errors)
}

fn infer_list_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // :wat/core/list — `∀T. T* -> List<T>`. All args must unify to a
    // common element type.
    let elem_var = fresh.fresh();
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &elem_var, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat/core/list".into(),
                    param: format!("#{}", i + 1),
                    expected: format_type(&apply_subst(&elem_var, subst)),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "List".into(),
        args: vec![apply_subst(&elem_var, subst)],
    })
}

fn infer_boolean_shortcircuit(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // `and` / `or` take any number of :bool args, return :bool.
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &TypeExpr::Path(":bool".into()), subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat/core/and/or".into(),
                    param: format!("#{}", i + 1),
                    expected: ":bool".into(),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

// ─── Unification ────────────────────────────────────────────────────────

#[derive(Debug)]
struct UnifyError;

/// Attempt to unify two type expressions under the given substitution.
/// Extends `subst` on success; leaves it untouched on failure.
fn unify(a: &TypeExpr, b: &TypeExpr, subst: &mut Subst) -> Result<(), UnifyError> {
    let a = walk(a, subst);
    let b = walk(b, subst);
    match (&a, &b) {
        (TypeExpr::Var(x), TypeExpr::Var(y)) if x == y => Ok(()),
        (TypeExpr::Var(x), other) | (other, TypeExpr::Var(x)) => {
            if occurs(*x, other, subst) {
                return Err(UnifyError);
            }
            subst.insert(*x, other.clone());
            Ok(())
        }
        (TypeExpr::Path(p1), TypeExpr::Path(p2)) => {
            if p1 == p2 {
                Ok(())
            } else {
                Err(UnifyError)
            }
        }
        (
            TypeExpr::Parametric { head: h1, args: a1 },
            TypeExpr::Parametric { head: h2, args: a2 },
        ) => {
            if h1 != h2 || a1.len() != a2.len() {
                return Err(UnifyError);
            }
            for (x, y) in a1.iter().zip(a2.iter()) {
                unify(x, y, subst)?;
            }
            Ok(())
        }
        (TypeExpr::Fn { args: a1, ret: r1 }, TypeExpr::Fn { args: a2, ret: r2 }) => {
            if a1.len() != a2.len() {
                return Err(UnifyError);
            }
            for (x, y) in a1.iter().zip(a2.iter()) {
                unify(x, y, subst)?;
            }
            unify(r1, r2, subst)
        }
        _ => Err(UnifyError),
    }
}

/// Chase a type through the substitution map until a non-bound root
/// is reached. Does not mutate the subst — callers use this to peek
/// at the current binding without path-compressing.
fn walk(ty: &TypeExpr, subst: &Subst) -> TypeExpr {
    let mut current = ty.clone();
    loop {
        match &current {
            TypeExpr::Var(id) => match subst.get(id) {
                Some(next) => current = next.clone(),
                None => return current,
            },
            _ => return current,
        }
    }
}

/// Apply a substitution deeply — rewrites every `Var(id)` in `ty` to
/// its bound target (transitively).
fn apply_subst(ty: &TypeExpr, subst: &Subst) -> TypeExpr {
    match ty {
        TypeExpr::Var(id) => match subst.get(id) {
            Some(inner) => apply_subst(inner, subst),
            None => TypeExpr::Var(*id),
        },
        TypeExpr::Path(_) => ty.clone(),
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head: head.clone(),
            args: args.iter().map(|a| apply_subst(a, subst)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| apply_subst(a, subst)).collect(),
            ret: Box::new(apply_subst(ret, subst)),
        },
    }
}

/// Occurs check — prevents binding `α := foo(α)`.
fn occurs(id: u64, ty: &TypeExpr, subst: &Subst) -> bool {
    let ty = walk(ty, subst);
    match &ty {
        TypeExpr::Var(other) => *other == id,
        TypeExpr::Path(_) => false,
        TypeExpr::Parametric { args, .. } => args.iter().any(|a| occurs(id, a, subst)),
        TypeExpr::Fn { args, ret } => {
            args.iter().any(|a| occurs(id, a, subst)) || occurs(id, ret, subst)
        }
    }
}

/// Instantiate a scheme's universally-quantified type parameters with
/// fresh unification variables. Produces monomorphic `(params, ret)`.
fn instantiate(scheme: &TypeScheme, fresh: &mut FreshGen) -> (Vec<TypeExpr>, TypeExpr) {
    if scheme.type_params.is_empty() {
        return (scheme.params.clone(), scheme.ret.clone());
    }
    let mut mapping: HashMap<String, TypeExpr> = HashMap::new();
    for tp in &scheme.type_params {
        mapping.insert(tp.clone(), fresh.fresh());
    }
    let params = scheme
        .params
        .iter()
        .map(|p| rename(p, &mapping))
        .collect();
    let ret = rename(&scheme.ret, &mapping);
    (params, ret)
}

/// Replace `Path(":T")` occurrences where T is a key in `mapping`
/// with the mapping's value. Used by [`instantiate`] to convert a
/// rigid type variable name into a fresh unification var.
fn rename(ty: &TypeExpr, mapping: &HashMap<String, TypeExpr>) -> TypeExpr {
    match ty {
        TypeExpr::Path(p) => {
            let key = p.strip_prefix(':').unwrap_or(p);
            if let Some(replacement) = mapping.get(key) {
                replacement.clone()
            } else {
                ty.clone()
            }
        }
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head: head.clone(),
            args: args.iter().map(|a| rename(a, mapping)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| rename(a, mapping)).collect(),
            ret: Box::new(rename(ret, mapping)),
        },
        TypeExpr::Var(_) => ty.clone(),
    }
}

// ─── Pretty printing ────────────────────────────────────────────────────

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
        TypeExpr::Var(id) => format!(":?{}", id),
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
        TypeExpr::Var(id) => format!("?{}", id),
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

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
    // `runtime::Function` carries declared type-parameters, parameter
    // types, and the return type since slice 7b. Lambdas (name = None)
    // leave param_types empty and aren't statically typed here.
    if func.name.is_none() {
        return None;
    }
    Some(TypeScheme {
        type_params: func.type_params.clone(),
        params: func.param_types.clone(),
        ret: func.ret_type.clone(),
    })
}

// ─── Built-in schemes ───────────────────────────────────────────────────

fn register_builtins(env: &mut CheckEnv) {
    let i64_ty = || TypeExpr::Path(":i64".into());
    let f64_ty = || TypeExpr::Path(":f64".into());
    let bool_ty = || TypeExpr::Path(":bool".into());
    let holon_ty = || TypeExpr::Path(":Holon".into());
    let t_var = || TypeExpr::Path(":T".into());

    // Arithmetic — i64 × i64 → i64. No implicit promotion.
    for op in &[":wat/core/+", ":wat/core/-", ":wat/core/*", ":wat/core//"] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty(), i64_ty()],
                ret: i64_ty(),
            },
        );
    }

    // Comparison — ∀T. T → T → :bool. Operands must agree.
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
                type_params: vec!["T".into()],
                params: vec![t_var(), t_var()],
                ret: bool_ty(),
            },
        );
    }

    // Boolean negation.
    env.register(
        ":wat/core/not".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![bool_ty()],
            ret: bool_ty(),
        },
    );

    // Algebra-core UpperCalls.
    // Atom — ∀T. T → :Holon. Accepts any payload type.
    env.register(
        ":wat/algebra/Atom".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: holon_ty(),
        },
    );
    // Bundle takes :List<Holon> → :Holon.
    env.register(
        ":wat/algebra/Bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "List".into(),
                args: vec![holon_ty()],
            }],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), i64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Thermometer".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat/algebra/Blend".into(),
        TypeScheme {
            type_params: vec![],
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

    // ─── Monomorphic type mismatch ──────────────────────────────────────

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
    fn bind_non_holon_rejected() {
        let err = check("(:wat/algebra/Bind 42 (:wat/algebra/Atom 1))").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic comparison (T -> T -> bool) ────────────────────────

    #[test]
    fn equality_same_type_passes() {
        assert!(check("(:wat/core/= 1 2)").is_ok());
        assert!(check(r#"(:wat/core/= "a" "b")"#).is_ok());
        assert!(check("(:wat/core/= true false)").is_ok());
    }

    #[test]
    fn equality_mixed_types_rejected() {
        let err = check(r#"(:wat/core/= 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn less_than_mixed_types_rejected() {
        let err = check(r#"(:wat/core/< 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic list (T* -> List<T>) ───────────────────────────────

    #[test]
    fn list_same_type_passes() {
        assert!(check("(:wat/core/list 1 2 3)").is_ok());
        assert!(check(r#"(:wat/core/list "a" "b")"#).is_ok());
    }

    #[test]
    fn list_mixed_types_rejected() {
        let err = check(r#"(:wat/core/list 1 "two" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bundle_of_list_of_holons_passes() {
        // Bundle takes :List<Holon>. A list of (Atom ...) calls
        // returns :List<Holon>, so Bundle(list(Atoms...)) type-checks.
        assert!(check(
            r#"(:wat/algebra/Bundle (:wat/core/list
                 (:wat/algebra/Atom 1)
                 (:wat/algebra/Atom 2)))"#
        )
        .is_ok());
    }

    #[test]
    fn bundle_of_list_of_ints_rejected() {
        // Bundle wants :List<Holon>, but this is :List<i64>.
        let err = check(r#"(:wat/algebra/Bundle (:wat/core/list 1 2 3))"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── User define signature checks ───────────────────────────────────

    #[test]
    fn user_define_body_matches_signature() {
        assert!(check(
            r#"(:wat/core/define (:my/app/add (x :i64) (y :i64) -> :i64)
                 (:wat/core/+ x y))"#
        )
        .is_ok());
    }

    #[test]
    fn user_define_body_wrong_return_rejected() {
        let err = check(
            r#"(:wat/core/define (:my/app/add (x :i64) (y :i64) -> :bool)
                 (:wat/core/+ x y))"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    #[test]
    fn user_parametric_define_passes() {
        // Identity: ∀T. T -> T. Body returns x, which has type T.
        // With rigid type variables, x: T unifies with ret: T.
        assert!(check(
            r#"(:wat/core/define (:my/app/id<T> (x :T) -> :T) x)"#
        )
        .is_ok());
    }

    #[test]
    fn user_parametric_wrong_return_rejected() {
        // Declared ret T; body returns an :i64 constant. Rigid T
        // doesn't unify with :i64.
        let err = check(
            r#"(:wat/core/define (:my/app/bad<T> (x :T) -> :T) 42)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    // ─── :Any ban ───────────────────────────────────────────────────────

    #[test]
    fn any_as_param_type_rejected_at_parse() {
        // Parsing `:Any` in any position is an error.
        let err = parse_type_expr(":Any").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_as_parametric_head_rejected_at_parse() {
        let err = parse_type_expr(":Any<i64>").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_as_nested_arg_rejected_at_parse() {
        let err = parse_type_expr(":List<Any>").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_in_fn_rejected_at_parse() {
        let err = parse_type_expr(":fn(Any)->i64").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    // ─── Multiple errors reported together ──────────────────────────────

    #[test]
    fn multiple_errors_reported() {
        let err = check(r#"(:wat/core/+ "s" 1) (:wat/core/not 42)"#).unwrap_err();
        assert!(err.0.len() >= 2, "expected >=2 errors, got {}", err.0.len());
    }

    // ─── Unification directly ───────────────────────────────────────────

    #[test]
    fn unify_identical_paths() {
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":i64".into()),
            &mut s
        )
        .is_ok());
    }

    #[test]
    fn unify_distinct_paths_fails() {
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":f64".into()),
            &mut s
        )
        .is_err());
    }

    #[test]
    fn unify_rigid_vars_require_same_name() {
        // Rigid Path(":T") only unifies with Path(":T").
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":T".into()),
            &mut s
        )
        .is_ok());
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":U".into()),
            &mut s
        )
        .is_err());
    }

    #[test]
    fn unify_fresh_var_binds_to_concrete() {
        let mut s = Subst::new();
        let var = TypeExpr::Var(0);
        let concrete = TypeExpr::Path(":i64".into());
        unify(&var, &concrete, &mut s).expect("unify");
        assert_eq!(apply_subst(&var, &s), concrete);
    }

    #[test]
    fn unify_parametric_head_must_match() {
        let mut s = Subst::new();
        let list_int = TypeExpr::Parametric {
            head: "List".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        let vec_int = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        assert!(unify(&list_int, &vec_int, &mut s).is_err());
    }

    #[test]
    fn unify_fn_types() {
        let mut s = Subst::new();
        let f1 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        let f2 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        assert!(unify(&f1, &f2, &mut s).is_ok());
    }

    #[test]
    fn occurs_check_rejects_cycle() {
        let mut s = Subst::new();
        // α = List<α>  — would produce an infinite type.
        let cyclic = TypeExpr::Parametric {
            head: "List".into(),
            args: vec![TypeExpr::Var(0)],
        };
        assert!(unify(&TypeExpr::Var(0), &cyclic, &mut s).is_err());
    }

    // ─── Parse + unify round-trip ───────────────────────────────────────

    #[test]
    fn type_expr_parse_and_unify() {
        let mut s = Subst::new();
        let a = parse_type_expr(":Holon").unwrap();
        let b = parse_type_expr(":Holon").unwrap();
        assert!(unify(&a, &b, &mut s).is_ok());
    }
}
