//! Type-check pass — rank-1 Hindley-Milner.
//!
//! This slice implements real parametric polymorphism per 058-030:
//!
//! - [`TypeScheme`] carries `type_params` — the list of names that are
//!   universally quantified (e.g., `["T"]` for `∀T. T -> :holon::HolonAST`).
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
//!   `∀T. T -> :holon::HolonAST`.
//! - `:Any` is banned everywhere — the type universe is closed
//!   ([058-030](https://…/058-030-types/PROPOSAL.md), §591). User
//!   source containing `:Any` halts at parse (see
//!   [`crate::types::parse_type_expr`]).
//!
//! # What this catches today
//!
//! - Arity mismatches in user-function and built-in calls at startup.
//! - Type mismatches: `(:wat::core::i64::+ "hello" 3)`, `(:wat::core::< 1 "x")`
//!   — `<` requires matching operand types.
//! - Polymorphic failures: `(:wat::core::vec 1 "two" 3)` — list
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
    /// A built-in form (e.g., `:wat::core::match`) is structurally
    /// malformed in a way the syntax-level grammar doesn't catch —
    /// e.g., a match arm that isn't `(pattern body)`, or a match
    /// whose pattern coverage is non-exhaustive.
    MalformedForm {
        head: String,
        reason: String,
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
            CheckError::MalformedForm { head, reason } => {
                write!(f, "malformed {} form: {}", head, reason)
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

    /// Build an env with built-in schemes for `:wat::core::*` and
    /// `:wat::algebra::*` forms, then overlay user-define signatures
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
        // `:None` — nullary constructor of the built-in :Option<T> enum.
        // Infers as `:Option<T>` with a fresh T; unification against the
        // expected type sharpens T at the use site.
        WatAST::Keyword(k) if k == ":None" => Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![fresh.fresh()],
        }),
        WatAST::Keyword(_) => Some(TypeExpr::Path(":wat::core::keyword".into())),
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
    // `()` — empty list — is the unit value. Type :() per the
    // existing TypeExpr::Tuple([]) encoding.
    let head = match items.first() {
        Some(h) => h,
        None => return Some(TypeExpr::Tuple(vec![])),
    };

    if let WatAST::Keyword(k) = head {
        let args = &items[1..];
        match k.as_str() {
            ":wat::core::if" => return infer_if(args, env, locals, fresh, subst, errors),
            ":wat::core::let" => return infer_let(args, env, locals, fresh, subst, errors),
            ":wat::core::let*" => return infer_let_star(args, env, locals, fresh, subst, errors),
            ":wat::core::vec" => return infer_list_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::list" => return infer_list_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::tuple" => return infer_tuple_constructor(args, env, locals, fresh, subst, errors),
            ":wat::std::HashMap" => return infer_hashmap_constructor(args, env, locals, fresh, subst, errors),
            ":wat::std::HashSet" => return infer_hashset_constructor(args, env, locals, fresh, subst, errors),
            ":wat::std::get" => return infer_get(args, env, locals, fresh, subst, errors),
            ":wat::core::quote" => {
                // Quote captures an unevaluated AST. The argument is
                // DATA, not an expression — the type checker does not
                // recurse into it. Return type is `:wat::WatAST`.
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: ":wat::core::quote".into(),
                        expected: 1,
                        got: args.len(),
                    });
                }
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            ":wat::core::match" => {
                return infer_match(args, env, locals, fresh, subst, errors);
            }
            ":wat::kernel::make-bounded-queue" => {
                return infer_make_queue(
                    args,
                    env,
                    locals,
                    fresh,
                    subst,
                    errors,
                    ":wat::kernel::make-bounded-queue",
                    /*with_capacity=*/ true,
                );
            }
            ":wat::kernel::make-unbounded-queue" => {
                return infer_make_queue(
                    args,
                    env,
                    locals,
                    fresh,
                    subst,
                    errors,
                    ":wat::kernel::make-unbounded-queue",
                    /*with_capacity=*/ false,
                );
            }
            ":wat::kernel::drop" => {
                return infer_drop(args, env, locals, fresh, subst, errors);
            }
            ":wat::kernel::spawn" => {
                return infer_spawn(args, env, locals, fresh, subst, errors);
            }
            ":wat::core::first" => {
                return infer_positional_accessor(args, env, locals, fresh, subst, errors, ":wat::core::first", 0);
            }
            ":wat::core::second" => {
                return infer_positional_accessor(args, env, locals, fresh, subst, errors, ":wat::core::second", 1);
            }
            ":wat::core::third" => {
                return infer_positional_accessor(args, env, locals, fresh, subst, errors, ":wat::core::third", 2);
            }
            ":wat::core::and" | ":wat::core::or" => {
                return infer_boolean_shortcircuit(args, env, locals, fresh, subst, errors);
            }
            ":wat::core::lambda" => return infer_lambda(args, env, locals, fresh, subst, errors),
            ":wat::core::use!" => {
                // use! is a resolve-pass declaration. It validates at
                // resolve time; the type checker treats it as a no-op
                // returning :(). The argument is a keyword path; we
                // don't recurse into it.
                return Some(TypeExpr::Tuple(vec![]));
            }
            _ if k.starts_with(":rust::") => {
                return dispatch_rust_scheme(k, args, env, locals, fresh, subst, errors);
            }
            ":wat::core::define"
            | ":wat::core::struct"
            | ":wat::core::enum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::core::defmacro"
            | ":wat::core::load!"
            | ":wat::core::digest-load!"
            | ":wat::core::signed-load!"
            | ":wat::core::quasiquote"
            | ":wat::core::unquote"
            | ":wat::core::unquote-splicing" => {
                // Top-level forms / reader-macro heads don't participate
                // in expression-level inference.
                return None;
            }
            _ if k.starts_with(":wat::config::set-") => return None,
            _ if (k.starts_with(":wat::kernel::") || k.starts_with(":wat::std::"))
                && !k.starts_with(":wat::std::math::")
                && env.get(k).is_none() =>
            {
                // Unknown kernel / std path with no registered scheme —
                // accept and recurse. Math lives at `:wat::std::math::*`
                // and has registered schemes; exclude it so the normal
                // scheme lookup below kicks in.
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

    // Bare `Some` as call head — built-in tagged constructor of
    // `:Option<T>`. `(Some expr)` infers as `:Option<T>` where T is the
    // argument's type.
    if let WatAST::Symbol(ident) = head {
        if ident.as_str() == "Some" {
            let args = &items[1..];
            if args.len() != 1 {
                errors.push(CheckError::ArityMismatch {
                    callee: "Some".into(),
                    expected: 1,
                    got: args.len(),
                });
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![fresh.fresh()],
                });
            }
            let inner_ty = infer(&args[0], env, locals, fresh, subst, errors)
                .unwrap_or_else(|| fresh.fresh());
            return Some(TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![inner_ty],
            });
        }
    }

    // Non-keyword head (bare symbol or inline expression). Not typed
    // at this layer pending your call on explicit let-binding type
    // annotations. Recurse into args so nested keyword-headed calls
    // still get checked.
    for item in items {
        let _ = infer(item, env, locals, fresh, subst, errors);
    }
    None
}

/// Type-check `(:wat::core::match scrutinee arm...)`. Scrutinee must
/// be `:Option<T>` (the only built-in enum in this slice). Each arm's
/// pattern introduces bindings visible in its body; every arm body's
/// type unifies to a common result type. Exhaustiveness: at least one
/// arm matches `:None` (either the `:None` pattern or a wildcard) and
/// at least one arm matches `(Some _)` (either the `Some` pattern or
/// a wildcard).
fn infer_match(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() < 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::match".into(),
            expected: 2,
            got: args.len(),
        });
        return None;
    }
    // Scrutinee must be :Option<T>.
    let scrutinee_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let inner_ty = fresh.fresh();
    let expected_scrutinee = TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![inner_ty.clone()],
    };
    if let Some(sty) = &scrutinee_ty {
        if unify(sty, &expected_scrutinee, subst).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::match".into(),
                param: "scrutinee".into(),
                expected: "Option<T>".into(),
                got: format_type(&apply_subst(sty, subst)),
            });
        }
    }

    let mut covers_none = false;
    let mut covers_some = false;
    let mut result_ty: Option<TypeExpr> = None;

    for (idx, arm) in args[1..].iter().enumerate() {
        let arm_items = match arm {
            WatAST::List(items) if items.len() == 2 => items,
            _ => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("arm #{} must be `(pattern body)`", idx + 1),
                });
                continue;
            }
        };
        let pattern = &arm_items[0];
        let body = &arm_items[1];

        let mut arm_locals = locals.clone();
        match pattern_coverage(pattern, &inner_ty, &mut arm_locals, errors) {
            Some(Coverage::None) => covers_none = true,
            Some(Coverage::Some) => covers_some = true,
            Some(Coverage::Wildcard) => {
                covers_none = true;
                covers_some = true;
            }
            None => continue,
        }

        let arm_ty = infer(body, env, &arm_locals, fresh, subst, errors);
        match (&result_ty, arm_ty) {
            (None, Some(t)) => result_ty = Some(t),
            (Some(rt), Some(t)) => {
                if unify(rt, &t, subst).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::core::match".into(),
                        param: format!("arm #{}", idx + 1),
                        expected: format_type(&apply_subst(rt, subst)),
                        got: format_type(&apply_subst(&t, subst)),
                    });
                }
            }
            _ => {}
        }
    }

    if !(covers_none && covers_some) {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: "non-exhaustive: :Option<T> needs arms for both :None and (Some _), or a wildcard".into(),
        });
    }

    result_ty.map(|t| apply_subst(&t, subst))
}

/// Coverage class for a match pattern under `:Option<T>`.
enum Coverage {
    None,
    Some,
    Wildcard,
}

/// Validate `pattern` against the inner type `T` (the argument of
/// `:Option<T>`), push bindings into `bindings`, and report its
/// coverage class.
fn pattern_coverage(
    pattern: &WatAST,
    inner_ty: &TypeExpr,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Option<Coverage> {
    match pattern {
        WatAST::Keyword(k) if k == ":None" => Some(Coverage::None),
        WatAST::Keyword(k) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!("keyword pattern {} not recognized (only `:None` in this slice)", k),
            });
            None
        }
        WatAST::Symbol(ident) if ident.as_str() == "_" => Some(Coverage::Wildcard),
        WatAST::Symbol(ident) => {
            // Bare name binds the whole scrutinee.
            bindings.insert(
                ident.as_str().to_string(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![inner_ty.clone()],
                },
            );
            Some(Coverage::Wildcard)
        }
        WatAST::List(items) => {
            let (head, rest) = match items.split_first() {
                Some(pair) => pair,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "empty list pattern".into(),
                    });
                    return None;
                }
            };
            match head {
                WatAST::Symbol(ident) if ident.as_str() == "Some" => {
                    if rest.len() != 1 {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Some _) takes exactly one field, got {}",
                                rest.len()
                            ),
                        });
                        return None;
                    }
                    match &rest[0] {
                        WatAST::Symbol(b) => {
                            bindings.insert(b.as_str().to_string(), inner_ty.clone());
                            Some(Coverage::Some)
                        }
                        other => {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Some _): binder must be a bare symbol, got {}",
                                    ast_variant_name_check(other)
                                ),
                            });
                            None
                        }
                    }
                }
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "list pattern head must be a variant constructor; got {}",
                            ast_variant_name_check(other)
                        ),
                    });
                    None
                }
            }
        }
        other => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!("pattern must be keyword, symbol, or list; got {}", ast_variant_name_check(other)),
            });
            None
        }
    }
}

fn ast_variant_name_check(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_) => "int",
        WatAST::FloatLit(_) => "float",
        WatAST::BoolLit(_) => "bool",
        WatAST::StringLit(_) => "string",
        WatAST::Keyword(_) => "keyword",
        WatAST::Symbol(_) => "symbol",
        WatAST::List(_) => "list",
    }
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
                    callee: ":wat::core::if".into(),
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
    let bindings = match &args[0] {
        WatAST::List(items) => items,
        _ => return None,
    };
    // Each binding is either typed-single `((name :Type) rhs)` or
    // untyped destructure `((a b ...) rhs)`. Parallel let — all RHSs
    // see the OUTER locals, not each other.
    let mut extended = locals.clone();
    for pair in bindings {
        process_let_binding(pair, env, locals, &mut extended, fresh, subst, errors, ":wat::core::let");
    }
    infer(&args[1], env, &extended, fresh, subst, errors)
}

/// Sequential let — same binding shapes as parallel `let`, but each
/// RHS is checked with the cumulatively extended locals so later
/// bindings may reference earlier ones.
fn infer_let_star(
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
    let bindings = match &args[0] {
        WatAST::List(items) => items,
        _ => return None,
    };
    let mut extended = locals.clone();
    for pair in bindings {
        // let* threads the cumulative extended locals through each RHS.
        // We pass `extended` as BOTH the RHS-inference scope and the
        // mutable target; the parallel variant passes the outer
        // `locals` as the RHS scope.
        let cumulative = extended.clone();
        process_let_binding(pair, env, &cumulative, &mut extended, fresh, subst, errors, ":wat::core::let*");
    }
    infer(&args[1], env, &extended, fresh, subst, errors)
}

/// Type-check `(:wat::kernel::spawn :fn::path arg1 arg2 ...)`.
/// Variadic in the args (one per function parameter) — rank-1 HM
/// can't express variadic schemes, so spawn is special-cased. First
/// argument must be a keyword-path; remaining args are checked
/// against the named function's parameter types looked up in the
/// CheckEnv. Return type is `:ProgramHandle<R>` where R is the
/// function's declared return type.
fn infer_spawn(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::kernel::spawn".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "wat::kernel::ProgramHandle".into(),
            args: vec![fresh.fresh()],
        });
    }
    let fn_path = match &args[0] {
        WatAST::Keyword(k) => k.clone(),
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::kernel::spawn".into(),
                reason: "first argument must be a function keyword path".into(),
            });
            return Some(TypeExpr::Parametric {
                head: "wat::kernel::ProgramHandle".into(),
                args: vec![fresh.fresh()],
            });
        }
    };
    let scheme = match env.get(&fn_path) {
        Some(s) => s.clone(),
        None => {
            // Function not registered — may be a primitive / future
            // slice / driver. Produce a ProgramHandle<?> so the call
            // site keeps checking.
            for arg in &args[1..] {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(TypeExpr::Parametric {
                head: "wat::kernel::ProgramHandle".into(),
                args: vec![fresh.fresh()],
            });
        }
    };
    let (param_types, ret_type) = instantiate(&scheme, fresh);
    let spawn_args = &args[1..];
    if spawn_args.len() != param_types.len() {
        errors.push(CheckError::ArityMismatch {
            callee: format!(":wat::kernel::spawn {}", fn_path),
            expected: param_types.len(),
            got: spawn_args.len(),
        });
    }
    for (i, (arg, expected)) in spawn_args.iter().zip(&param_types).enumerate() {
        if let Some(arg_ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&arg_ty, expected, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: format!(":wat::kernel::spawn {}", fn_path),
                    param: format!("#{}", i + 1),
                    expected: format_type(&apply_subst(expected, subst)),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "wat::kernel::ProgramHandle".into(),
        args: vec![apply_subst(&ret_type, subst)],
    })
}

/// Type-check `(:wat::core::first xs)` / `second` / `third`.
/// Polymorphic over Vec<T> and tuple — both are index-addressed.
/// Rank-1 HM can't express the union, so this is special-cased:
/// inspect the argument's type after substitution and return the
/// element at `index` from whichever container shape matches.
#[allow(clippy::too_many_arguments)]
fn infer_positional_accessor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    op: &str,
    index: usize,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 1,
            got: args.len(),
        });
        return Some(fresh.fresh());
    }
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ty) = arg_ty {
        let resolved = apply_subst(&ty, subst);
        match &resolved {
            // Tuple: return element at `index`.
            TypeExpr::Tuple(elements) => {
                if let Some(elem) = elements.get(index) {
                    return Some(apply_subst(elem, subst));
                } else {
                    errors.push(CheckError::TypeMismatch {
                        callee: op.into(),
                        param: "#1".into(),
                        expected: format!("tuple with ≥ {} element(s)", index + 1),
                        got: format_type(&resolved),
                    });
                    return Some(fresh.fresh());
                }
            }
            // Vec<T>: return T.
            TypeExpr::Parametric { head, args: targs } if head == "Vec" => {
                if let Some(inner) = targs.first() {
                    return Some(apply_subst(inner, subst));
                } else {
                    return Some(fresh.fresh());
                }
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: op.into(),
                    param: "#1".into(),
                    expected: "tuple or Vec<T>".into(),
                    got: format_type(&resolved),
                });
            }
        }
    }
    Some(fresh.fresh())
}

/// Type-check `(:wat::kernel::drop handle)`. The handle is either a
/// `Sender<T>` or `Receiver<T>` — rank-1 HM can't express a union, so
/// this is special-cased: accept either parametric head, produce `:()`.
fn infer_drop(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::kernel::drop".into(),
            expected: 1,
            got: args.len(),
        });
        return Some(TypeExpr::Tuple(vec![]));
    }
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ty) = arg_ty {
        let resolved = apply_subst(&ty, subst);
        let is_channel_handle = matches!(
            &resolved,
            TypeExpr::Parametric { head, .. }
                if head == "crossbeam_channel::Sender"
                    || head == "crossbeam_channel::Receiver"
        );
        if !is_channel_handle {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::kernel::drop".into(),
                param: "#1".into(),
                expected: "crossbeam_channel::Sender<T> | crossbeam_channel::Receiver<T>".into(),
                got: format_type(&resolved),
            });
        }
    }
    Some(TypeExpr::Tuple(vec![]))
}

/// Type-check `(make-bounded-queue :T N)` / `(make-unbounded-queue :T)`.
/// First argument is a type keyword (introspected directly, not
/// inferred as a value); optional second argument is the capacity,
/// which must unify to `:i64`. Return type is
/// `:(Sender<T>, Receiver<T>)`.
///
/// Written as a special form because the `∀T. ...` shape expresses T
/// through a type-keyword argument — the value-level checker can't
/// extract T from `infer(args[0])` the way rank-1 HM would want.
#[allow(clippy::too_many_arguments)]
fn infer_make_queue(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
    with_capacity: bool,
) -> Option<TypeExpr> {
    let expected_arity = if with_capacity { 2 } else { 1 };
    if args.len() != expected_arity {
        errors.push(CheckError::ArityMismatch {
            callee: form.into(),
            expected: expected_arity,
            got: args.len(),
        });
        // Still recurse into any extra args for nested checks.
        for arg in args.iter().skip(1) {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        // Return a best-effort tuple with a fresh inner so the call
        // site can continue checking.
        let t = fresh.fresh();
        return Some(TypeExpr::Tuple(vec![
            TypeExpr::Parametric {
                head: "crossbeam_channel::Sender".into(),
                args: vec![t.clone()],
            },
            TypeExpr::Parametric {
                head: "crossbeam_channel::Receiver".into(),
                args: vec![t],
            },
        ]));
    }
    // Extract T from the type-keyword argument.
    let t_ty = match &args[0] {
        WatAST::Keyword(k) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: form.into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                fresh.fresh()
            }
        },
        other => {
            errors.push(CheckError::MalformedForm {
                head: form.into(),
                reason: format!(
                    "first argument must be a type keyword; got {}",
                    match other {
                        WatAST::IntLit(_) => "int",
                        WatAST::FloatLit(_) => "float",
                        WatAST::BoolLit(_) => "bool",
                        WatAST::StringLit(_) => "string",
                        WatAST::Symbol(_) => "symbol",
                        WatAST::List(_) => "list",
                        WatAST::Keyword(_) => unreachable!(),
                    }
                ),
            });
            fresh.fresh()
        }
    };
    // If bounded, check capacity unifies to :i64.
    if with_capacity {
        let cap_ty = infer(&args[1], env, locals, fresh, subst, errors);
        if let Some(cap_ty) = cap_ty {
            let i64_ty = TypeExpr::Path(":i64".into());
            if unify(&cap_ty, &i64_ty, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: form.into(),
                    param: "capacity".into(),
                    expected: "i64".into(),
                    got: format_type(&apply_subst(&cap_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Tuple(vec![
        TypeExpr::Parametric {
            head: "crossbeam_channel::Sender".into(),
            args: vec![t_ty.clone()],
        },
        TypeExpr::Parametric {
            head: "crossbeam_channel::Receiver".into(),
            args: vec![t_ty],
        },
    ]))
}

/// Process one binding — single-typed or destructure. Infers the RHS
/// in `rhs_scope` and adds the binding(s) to `out_scope`.
#[allow(clippy::too_many_arguments)]
fn process_let_binding(
    pair: &WatAST,
    env: &CheckEnv,
    rhs_scope: &HashMap<String, TypeExpr>,
    out_scope: &mut HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
) {
    let kv = match pair {
        WatAST::List(items) if items.len() == 2 => items,
        _ => return, // runtime parser surfaces the shape error
    };
    let binder = match &kv[0] {
        WatAST::List(inner) => inner,
        _ => return, // bare `(name rhs)` refused at runtime; check silently skips
    };
    let rhs = &kv[1];

    let is_typed_single = binder.len() == 2
        && matches!(&binder[0], WatAST::Symbol(_))
        && matches!(&binder[1], WatAST::Keyword(_));

    if is_typed_single {
        let name = match &binder[0] {
            WatAST::Symbol(ident) => ident.name.clone(),
            _ => return,
        };
        let declared = match &binder[1] {
            WatAST::Keyword(k) => match crate::types::parse_type_expr(k) {
                Ok(t) => t,
                Err(_) => return,
            },
            _ => return,
        };
        let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
        if let Some(rhs_ty) = rhs_ty {
            if unify(&rhs_ty, &declared, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: form.into(),
                    param: format!("binding '{}'", name),
                    expected: format_type(&apply_subst(&declared, subst)),
                    got: format_type(&apply_subst(&rhs_ty, subst)),
                });
            }
        }
        out_scope.insert(name, declared);
        return;
    }

    // Destructure: each element is a bare symbol. Generate one fresh
    // type variable per name; unify the RHS against a tuple of those
    // vars; bind each name to its (post-substitution) element type.
    let mut names = Vec::with_capacity(binder.len());
    for item in binder {
        match item {
            WatAST::Symbol(ident) => names.push(ident.name.clone()),
            _ => return, // runtime parser surfaces the shape error
        }
    }
    if names.is_empty() {
        return;
    }
    let elem_vars: Vec<TypeExpr> = (0..names.len()).map(|_| fresh.fresh()).collect();
    let tuple_ty = TypeExpr::Tuple(elem_vars.clone());
    let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
    if let Some(rhs_ty) = rhs_ty {
        if unify(&rhs_ty, &tuple_ty, subst).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: form.into(),
                param: format!("destructure ({})", names.join(" ")),
                expected: format_type(&apply_subst(&tuple_ty, subst)),
                got: format_type(&apply_subst(&rhs_ty, subst)),
            });
        }
    }
    for (name, ev) in names.into_iter().zip(elem_vars.into_iter()) {
        out_scope.insert(name, apply_subst(&ev, subst));
    }
}

/// Type-check `(:wat::std::HashSet :T x1 x2 ...)`. First arg is a
/// type keyword; remaining args are elements, each unifying with T.
/// Explicit typing required (matches the vec/list / make-queue
/// resource-constructor discipline — shape never depends on context).
fn infer_hashset_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::std::HashSet".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "HashSet".into(),
            args: vec![fresh.fresh()],
        });
    }
    let t_ty = match &args[0] {
        WatAST::Keyword(k) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::std::HashSet".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                fresh.fresh()
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::std::HashSet".into(),
                reason: "first argument must be a type keyword (e.g., :i64)".into(),
            });
            fresh.fresh()
        }
    };
    for (i, arg) in args[1..].iter().enumerate() {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &t_ty, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::std::HashSet".into(),
                    param: format!("element #{}", i + 1),
                    expected: format_type(&apply_subst(&t_ty, subst)),
                    got: format_type(&apply_subst(&ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashSet".into(),
        args: vec![apply_subst(&t_ty, subst)],
    })
}

/// Type-check `(:wat::std::get container locator)`. Polymorphic over
/// HashMap and HashSet; dispatch by arg shape. Rank-1 HM can't
/// express the union at the SCHEME layer, so special-case: inspect
/// the first arg's type and produce the matching return type.
///   HashMap<K,V>, K → Option<V>
///   HashSet<T>,   T → Option<T>
fn infer_get(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::std::get".into(),
            expected: 2,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let resolved = apply_subst(&ct, subst);
        match &resolved {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                let v = apply_subst(&ta[1], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::std::get".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&v, subst)],
                });
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &t, subst).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::std::get".into(),
                            param: "element".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&t, subst)],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::std::get".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | HashSet<T>".into(),
                    got: format_type(&resolved),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![fresh.fresh()],
    })
}

/// Type-check `(:wat::std::HashMap :(K,V) k1 v1 k2 v2 ...)`. First arg
/// is a tuple-type keyword `:(K,V)` encoding both parameters; the
/// remaining args are alternating key/value pairs. Keys unify with K,
/// values with V. Explicit typing required (matches vec/list / make-queue
/// resource-constructor discipline).
fn infer_hashmap_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::std::HashMap".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let (k_ty, v_ty) = match &args[0] {
        WatAST::Keyword(k) => match crate::types::parse_type_expr(k) {
            Ok(TypeExpr::Tuple(ts)) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
            Ok(other) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::std::HashMap".into(),
                    reason: format!(
                        "first argument must be a tuple type :(K,V); got {}",
                        format_type(&other)
                    ),
                });
                (fresh.fresh(), fresh.fresh())
            }
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::std::HashMap".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                (fresh.fresh(), fresh.fresh())
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::std::HashMap".into(),
                reason: "first argument must be a tuple type keyword :(K,V)".into(),
            });
            (fresh.fresh(), fresh.fresh())
        }
    };
    let pairs = &args[1..];
    if !pairs.len().is_multiple_of(2) {
        errors.push(CheckError::MalformedForm {
            head: ":wat::std::HashMap".into(),
            reason: "arity after :(K,V) must be even (alternating key/value)".into(),
        });
    }
    for (i, chunk) in pairs.chunks(2).enumerate() {
        if let Some(k_arg_ty) = infer(&chunk[0], env, locals, fresh, subst, errors) {
            if unify(&k_arg_ty, &k_ty, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::std::HashMap".into(),
                    param: format!("key #{}", i + 1),
                    expected: format_type(&apply_subst(&k_ty, subst)),
                    got: format_type(&apply_subst(&k_arg_ty, subst)),
                });
            }
        }
        if let Some(v_arg_ty) = chunk
            .get(1)
            .and_then(|a| infer(a, env, locals, fresh, subst, errors))
        {
            if unify(&v_arg_ty, &v_ty, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::std::HashMap".into(),
                    param: format!("value #{}", i + 1),
                    expected: format_type(&apply_subst(&v_ty, subst)),
                    got: format_type(&apply_subst(&v_arg_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![apply_subst(&k_ty, subst), apply_subst(&v_ty, subst)],
    })
}

/// Type-check `(:wat::core::tuple a b c ...)`. Heterogeneous — each
/// arg contributes its own inferred type, and the return type is the
/// concrete tuple shape. Variadic; rank-1 HM can't express a
/// per-position scheme, so special-cased.
fn infer_tuple_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::tuple".into(),
            reason: "tuple must have at least one element".into(),
        });
        return Some(TypeExpr::Tuple(vec![fresh.fresh()]));
    }
    let mut elements = Vec::with_capacity(args.len());
    for arg in args {
        let ty = infer(arg, env, locals, fresh, subst, errors).unwrap_or_else(|| fresh.fresh());
        elements.push(apply_subst(&ty, subst));
    }
    Some(TypeExpr::Tuple(elements))
}

fn infer_list_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // :wat::core::vec / :wat::core::list — `(vec :T x1 x2 ...) -> Vec<T>`.
    // First arg is a type keyword (read, not inferred); remaining args
    // must unify with T. Explicit typing is required even for non-empty
    // literals — the shape never depends on content or context.
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::vec".into(),
            expected: 1,
            got: 0,
        });
        let t = fresh.fresh();
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![t],
        });
    }
    let elem_ty = match &args[0] {
        WatAST::Keyword(k) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::vec".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                fresh.fresh()
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::vec".into(),
                reason: "first argument must be a type keyword (e.g., :i64)".into(),
            });
            fresh.fresh()
        }
    };
    for (i, arg) in args[1..].iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &elem_ty, subst).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::vec".into(),
                    param: format!("#{}", i + 2),
                    expected: format_type(&apply_subst(&elem_ty, subst)),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![apply_subst(&elem_ty, subst)],
    })
}

/// A lambda expression's type is `:fn(<param types>) -> <return type>`.
/// The signature is mandatory per 058-029 — every param and the
/// return are annotated. The body is checked against the declared
/// return type (same discipline as `check_function_body`).
fn infer_lambda(
    args: &[WatAST],
    env: &CheckEnv,
    outer_locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let sig = &args[0];
    let body = &args[1];
    let (param_names, param_types, ret_type) = parse_lambda_signature_for_check(sig).ok()?;

    // Check body against declared return type under extended locals.
    let mut body_locals = outer_locals.clone();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        body_locals.insert(name.clone(), ty.clone());
    }
    let body_ty = infer(body, env, &body_locals, fresh, subst, errors);
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &ret_type, subst).is_err() {
            errors.push(CheckError::ReturnTypeMismatch {
                function: "<lambda>".into(),
                expected: format_type(&apply_subst(&ret_type, subst)),
                got: format_type(&apply_subst(&body_ty, subst)),
            });
        }
    }

    Some(TypeExpr::Fn {
        args: param_types,
        ret: Box::new(ret_type),
    })
}

/// Mirror of [`crate::runtime::parse_lambda_signature`] shape for the
/// check pass — returns (names, types, ret). Errors are silenced; if
/// the lambda is malformed, runtime parsing catches it and the
/// checker simply returns None.
fn parse_lambda_signature_for_check(
    sig: &WatAST,
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()> {
    let items = match sig {
        WatAST::List(items) => items,
        _ => return Err(()),
    };
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut ret: Option<TypeExpr> = None;
    let mut saw_arrow = false;
    for item in items {
        if saw_arrow {
            if ret.is_some() {
                return Err(());
            }
            match item {
                WatAST::Keyword(k) => {
                    ret = Some(crate::types::parse_type_expr(k).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
            continue;
        }
        match item {
            WatAST::Symbol(s) if s.as_str() == "->" => saw_arrow = true,
            WatAST::List(pair) => {
                if pair.len() != 2 {
                    return Err(());
                }
                let name = match &pair[0] {
                    WatAST::Symbol(s) => s.name.clone(),
                    _ => return Err(()),
                };
                let ty = match &pair[1] {
                    WatAST::Keyword(k) => crate::types::parse_type_expr(k).map_err(|_| ())?,
                    _ => return Err(()),
                };
                names.push(name);
                types.push(ty);
            }
            _ => return Err(()),
        }
    }
    Ok((names, types, ret.ok_or(())?))
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
                    callee: ":wat::core::and/or".into(),
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
        (TypeExpr::Tuple(e1), TypeExpr::Tuple(e2)) => {
            if e1.len() != e2.len() {
                return Err(UnifyError);
            }
            for (x, y) in e1.iter().zip(e2.iter()) {
                unify(x, y, subst)?;
            }
            Ok(())
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

// ─── Rust-deps scheme dispatch ───────────────────────────────────────

/// Dispatch a `:rust::*` call to the shim's scheme function registered
/// in the rust-deps registry. Wraps the checker's internal state in a
/// [`CheckSchemeCtx`] that implements [`crate::rust_deps::SchemeCtx`],
/// giving the shim a narrow interface that doesn't depend on this
/// module's private types.
#[allow(clippy::too_many_arguments)]
fn dispatch_rust_scheme(
    keyword: &str,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut FreshGen,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let registry = crate::rust_deps::get();
    let sym_entry = match registry.get_symbol(keyword) {
        Some(s) => s,
        None => {
            errors.push(CheckError::UnknownCallee {
                callee: keyword.to_string(),
            });
            return None;
        }
    };
    let mut ctx = CheckSchemeCtx {
        env,
        locals,
        fresh,
        subst,
        errors,
    };
    (sym_entry.scheme)(args, &mut ctx)
}

/// Adapter that presents the checker's internal state (`env`, `locals`,
/// `fresh`, `subst`, `errors`) through the narrow
/// [`crate::rust_deps::SchemeCtx`] trait. Lets shim authors write their
/// scheme functions without depending on `check.rs`'s private types.
struct CheckSchemeCtx<'a> {
    env: &'a CheckEnv,
    locals: &'a HashMap<String, TypeExpr>,
    fresh: &'a mut FreshGen,
    subst: &'a mut Subst,
    errors: &'a mut Vec<CheckError>,
}

impl<'a> crate::rust_deps::SchemeCtx for CheckSchemeCtx<'a> {
    fn fresh_var(&mut self) -> TypeExpr {
        self.fresh.fresh()
    }

    fn infer(&mut self, ast: &WatAST) -> Option<TypeExpr> {
        infer(ast, self.env, self.locals, self.fresh, self.subst, self.errors)
    }

    fn unify_types(&mut self, a: &TypeExpr, b: &TypeExpr) -> bool {
        unify(a, b, self.subst).is_ok()
    }

    fn apply_subst(&self, t: &TypeExpr) -> TypeExpr {
        apply_subst(t, self.subst)
    }

    fn push_type_mismatch(&mut self, callee: &str, param: &str, expected: String, got: String) {
        self.errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: param.into(),
            expected,
            got,
        });
    }

    fn push_arity_mismatch(&mut self, callee: &str, expected: usize, got: usize) {
        self.errors.push(CheckError::ArityMismatch {
            callee: callee.into(),
            expected,
            got,
        });
    }

    fn push_malformed(&mut self, head: &str, reason: String) {
        self.errors.push(CheckError::MalformedForm {
            head: head.into(),
            reason,
        });
    }

    fn parse_type_keyword(&self, keyword: &str) -> Result<TypeExpr, crate::types::TypeError> {
        crate::types::parse_type_expr(keyword)
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
        TypeExpr::Tuple(elements) => TypeExpr::Tuple(
            elements.iter().map(|e| apply_subst(e, subst)).collect(),
        ),
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
        TypeExpr::Tuple(elements) => elements.iter().any(|e| occurs(id, e, subst)),
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
        TypeExpr::Tuple(elements) => {
            TypeExpr::Tuple(elements.iter().map(|e| rename(e, mapping)).collect())
        }
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
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_inner).collect();
            if elements.len() == 1 {
                // 1-tuple requires trailing comma to disambiguate
                // from parenthesization.
                format!(":({},)", inner[0])
            } else {
                format!(":({})", inner.join(","))
            }
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
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_inner).collect();
            if elements.len() == 1 {
                format!("({},)", inner[0])
            } else {
                format!("({})", inner.join(","))
            }
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
    func.name.as_ref()?;
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
    let holon_ty = || TypeExpr::Path(":holon::HolonAST".into());
    let t_var = || TypeExpr::Path(":T".into());

    // Integer arithmetic — strict i64 × i64 → i64 under the
    // `:wat::core::i64::*` namespace.
    for op in &[
        ":wat::core::i64::+",
        ":wat::core::i64::-",
        ":wat::core::i64::*",
        ":wat::core::i64::/",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty(), i64_ty()],
                ret: i64_ty(),
            },
        );
    }
    // Float arithmetic — strict f64 × f64 → f64 under the
    // `:wat::core::f64::*` namespace. Users commit to int or float at
    // the call site; no implicit promotion.
    for op in &[
        ":wat::core::f64::+",
        ":wat::core::f64::-",
        ":wat::core::f64::*",
        ":wat::core::f64::/",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: f64_ty(),
            },
        );
    }

    // Comparison — ∀T. T → T → :bool. Operands must agree.
    for op in &[
        ":wat::core::=",
        ":wat::core::<",
        ":wat::core::>",
        ":wat::core::<=",
        ":wat::core::>=",
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
        ":wat::core::not".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![bool_ty()],
            ret: bool_ty(),
        },
    );

    // Algebra-core UpperCalls.
    // Atom — ∀T. T → :holon::HolonAST. Accepts any payload type.
    env.register(
        ":wat::algebra::Atom".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: holon_ty(),
        },
    );
    // atom-value — ∀T. :holon::HolonAST → :T. Dual of Atom. The caller's
    // let-binding type ascription (or surrounding context) pins T; the
    // runtime downcasts the payload and errors on type mismatch.
    env.register(
        ":wat::core::atom-value".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![holon_ty()],
            ret: t_var(),
        },
    );
    env.register(
        ":wat::algebra::Bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: holon_ty(),
        },
    );
    // Bundle takes :Vec<holon::HolonAST> → :holon::HolonAST.
    env.register(
        ":wat::algebra::Bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![holon_ty()],
            }],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::algebra::Permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), i64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::algebra::Thermometer".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::algebra::Blend".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );

    // Cosine measurement — the retrieval scalar (FOUNDATION 1718 +
    // OPEN-QUESTIONS line 419). Algebra-substrate operation (input is
    // holons, not raw numbers).
    //   (:wat::algebra::cosine    target ref) -> :f64
    //   (:wat::algebra::presence? target ref) -> :bool (cosine > noise-floor)
    env.register(
        ":wat::algebra::cosine".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::algebra::presence?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: bool_ty(),
        },
    );

    // Config accessors — nullary, read committed startup values.
    env.register(
        ":wat::config::dims".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::config::global-seed".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::config::noise-floor".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: f64_ty(),
        },
    );

    // Kernel primitives.
    // (:wat::kernel::stopped) → :bool.
    env.register(
        ":wat::kernel::stopped?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: bool_ty(),
        },
    );
    // User-signal surface — 2026-04-19 stance: kernel measures, userland
    // owns transitions. Six nullary primitives: three pollers return
    // :bool; three resetters return :(). SIGINT / SIGTERM stay on the
    // `stopped` flag above.
    for path in [
        ":wat::kernel::sigusr1?",
        ":wat::kernel::sigusr2?",
        ":wat::kernel::sighup?",
    ] {
        env.register(
            path.into(),
            TypeScheme {
                type_params: vec![],
                params: vec![],
                ret: bool_ty(),
            },
        );
    }
    for path in [
        ":wat::kernel::reset-sigusr1!",
        ":wat::kernel::reset-sigusr2!",
        ":wat::kernel::reset-sighup!",
    ] {
        env.register(
            path.into(),
            TypeScheme {
                type_params: vec![],
                params: vec![],
                ret: TypeExpr::Tuple(vec![]),
            },
        );
    }
    // (:wat::kernel::send sender value) — ∀T. Sender<T> -> T -> :().
    env.register(
        ":wat::kernel::send".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "crossbeam_channel::Sender".into(),
                    args: vec![t_var()],
                },
                t_var(),
            ],
            ret: TypeExpr::Tuple(vec![]),
        },
    );
    // (:wat::kernel::try-recv receiver) — ∀T. Receiver<T> -> :Option<T>.
    // Non-blocking; `:None` covers both empty and disconnected.
    env.register(
        ":wat::kernel::try-recv".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "crossbeam_channel::Receiver".into(),
                args: vec![t_var()],
            }],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![t_var()],
            },
        },
    );
    // (:wat::kernel::recv receiver) — ∀T. Receiver<T> -> :Option<T>.
    // `:None` is the disconnect signal; `(Some v)` carries the payload.
    env.register(
        ":wat::kernel::recv".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "crossbeam_channel::Receiver".into(),
                args: vec![t_var()],
            }],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![t_var()],
            },
        },
    );
    // (:wat::kernel::join handle) — ∀R. ProgramHandle<R> -> R.
    let r_var = || TypeExpr::Path(":R".into());
    env.register(
        ":wat::kernel::join".into(),
        TypeScheme {
            type_params: vec!["R".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::ProgramHandle".into(),
                args: vec![r_var()],
            }],
            ret: r_var(),
        },
    );
    // HandlePool — claim-or-panic discipline.
    //   new    : ∀T. :String -> :Vec<T> -> :HandlePool<T>
    //   pop    : ∀T. :HandlePool<T> -> :T
    //   finish : ∀T. :HandlePool<T> -> :()
    env.register(
        ":wat::kernel::HandlePool::new".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Path(":String".into()),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![t_var()],
                },
            ],
            ret: TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            },
        },
    );
    env.register(
        ":wat::kernel::HandlePool::pop".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            }],
            ret: t_var(),
        },
    );
    env.register(
        ":wat::kernel::HandlePool::finish".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            }],
            ret: TypeExpr::Tuple(vec![]),
        },
    );
    // (:wat::kernel::select receivers) — ∀T. Vec<Receiver<T>> -> :(i64, Option<T>).
    // Spec calls for :usize on the index; wat-rs returns :i64 until
    // :usize lands as a value variant.
    env.register(
        ":wat::kernel::select".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Parametric {
                    head: "crossbeam_channel::Receiver".into(),
                    args: vec![t_var()],
                }],
            }],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Path(":i64".into()),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![t_var()],
                },
            ]),
        },
    );
    // Algebra measurement: dot product. Per 058-005 new measurement
    // primitive. Scalar-returning sibling of cosine; used by the
    // Gram-Schmidt stdlib macros (Reject, Project).
    env.register(
        ":wat::algebra::dot".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: f64_ty(),
        },
    );

    // IO primitives — honest surface to the real OS stdio. Writes
    // are polymorphic over :io::Stdout and :io::Stderr (dispatched
    // via runtime variant match; rank-1 HM can't express the union).
    // For the scheme, each handle type gets its own write entry —
    // but since both go to the same dispatch, we register just one
    // polymorphic shape using a special head that the runtime
    // accepts. Concretely: `:wat::io::write` types as either of the
    // two stdio handles, producing :(). We express this by letting
    // the scheme accept `:io::Stdout` as the canonical; the runtime
    // also accepts `:io::Stderr`, so the check-time error only fires
    // on genuinely-wrong types (user passes an int, say). Users who
    // want tight checking can register their own wrapping macros.
    env.register(
        ":wat::io::write".into(),
        TypeScheme {
            type_params: vec!["H".into()],
            params: vec![TypeExpr::Path(":H".into()), TypeExpr::Path(":String".into())],
            ret: TypeExpr::Tuple(vec![]),
        },
    );
    env.register(
        ":wat::io::read-line".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":io::Stdin".into())],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Path(":String".into())],
            },
        },
    );

    // Stdlib math — single-method Rust calls per FOUNDATION-CHANGELOG
    // 2026-04-18. All unary :f64 -> :f64 except pi which is :() -> :f64.
    // Packaged here so Log / Circular expansions get proper checking.
    for name in ["ln", "log", "sin", "cos"] {
        env.register(
            format!(":wat::std::math::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty()],
                ret: f64_ty(),
            },
        );
    }
    env.register(
        ":wat::std::math::pi".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: f64_ty(),
        },
    );
    // List/Vec primitives — Round 4a, per docs/058-backlog.md.
    //
    //   length   : ∀T. Vec<T> -> :i64
    //   empty?   : ∀T. Vec<T> -> :bool
    //   reverse  : ∀T. Vec<T> -> Vec<T>
    //   range    : :i64 × :i64 -> Vec<i64>   (two-arg; no overload)
    //   take     : ∀T. Vec<T> × :i64 -> Vec<T>
    //   drop     : ∀T. Vec<T> × :i64 -> Vec<T>
    //   map      : ∀T,U. Vec<T> × fn(T)->U -> Vec<U>
    //   foldl    : ∀T,Acc. Vec<T> × Acc × fn(Acc,T)->Acc -> Acc
    //   window   : ∀T. Vec<T> × :i64 -> Vec<Vec<T>>   (at :wat::std::list::)
    let u_var = || TypeExpr::Path(":U".into());
    let acc_var = || TypeExpr::Path(":Acc".into());
    let vec_of = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![inner],
    };
    env.register(
        ":wat::core::length".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::core::empty?".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::core::reverse".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::range".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty(), i64_ty()],
            ret: vec_of(i64_ty()),
        },
    );
    env.register(
        ":wat::core::take".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::drop".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::map".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(u_var()),
                },
            ],
            ret: vec_of(u_var()),
        },
    );
    env.register(
        ":wat::core::foldl".into(),
        TypeScheme {
            type_params: vec!["T".into(), "Acc".into()],
            params: vec![
                vec_of(t_var()),
                acc_var(),
                TypeExpr::Fn {
                    args: vec![acc_var(), t_var()],
                    ret: Box::new(acc_var()),
                },
            ],
            ret: acc_var(),
        },
    );
    env.register(
        ":wat::core::foldr".into(),
        TypeScheme {
            type_params: vec!["T".into(), "Acc".into()],
            params: vec![
                vec_of(t_var()),
                acc_var(),
                TypeExpr::Fn {
                    args: vec![t_var(), acc_var()],
                    ret: Box::new(acc_var()),
                },
            ],
            ret: acc_var(),
        },
    );
    env.register(
        ":wat::core::filter".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(bool_ty()),
                },
            ],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::std::list::zip".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![vec_of(t_var()), vec_of(u_var())],
            ret: vec_of(TypeExpr::Tuple(vec![t_var(), u_var()])),
        },
    );
    // get is special-cased in infer_list (polymorphic over HashMap
    // and HashSet). contains? (HashMap) and member? (HashSet) carry
    // their own narrow schemes.
    env.register(
        ":wat::std::contains?".into(),
        TypeScheme {
            type_params: vec!["K".into(), "V".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "HashMap".into(),
                    args: vec![TypeExpr::Path(":K".into()), TypeExpr::Path(":V".into())],
                },
                TypeExpr::Path(":K".into()),
            ],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::std::member?".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "HashSet".into(),
                    args: vec![t_var()],
                },
                t_var(),
            ],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::std::list::remove-at".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::std::list::window".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(vec_of(t_var())),
        },
    );

    // first/second/third are special-cased (polymorphic over Vec + tuple;
    // see infer_positional_accessor). rest is simple:
    env.register(
        ":wat::core::rest".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: vec_of(t_var()),
        },
    );
    // :wat::std::list::map-with-index — needed by Sequential for
    // indexed fold.
    env.register(
        ":wat::std::list::map-with-index".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var(), i64_ty()],
                    ret: Box::new(u_var()),
                },
            ],
            ret: vec_of(u_var()),
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
        assert!(check("(:wat::core::i64::+ 1 2)").is_ok());
        assert!(check("(:wat::core::not true)").is_ok());
        assert!(check("(:wat::algebra::Bind (:wat::algebra::Atom 1) (:wat::algebra::Atom 2))").is_ok());
    }

    #[test]
    fn too_few_args_rejected() {
        let err = check("(:wat::core::i64::+ 1)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 2, got: 1, .. })));
    }

    #[test]
    fn too_many_args_rejected() {
        let err = check("(:wat::core::not true false)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 1, got: 2, .. })));
    }

    // ─── Monomorphic type mismatch ──────────────────────────────────────

    #[test]
    fn string_to_add_rejected() {
        let err = check(r#"(:wat::core::i64::+ "hello" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bool_to_add_rejected() {
        let err = check("(:wat::core::i64::+ true 3)").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bind_non_holon_rejected() {
        let err = check("(:wat::algebra::Bind 42 (:wat::algebra::Atom 1))").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic comparison (T -> T -> bool) ────────────────────────

    #[test]
    fn equality_same_type_passes() {
        assert!(check("(:wat::core::= 1 2)").is_ok());
        assert!(check(r#"(:wat::core::= "a" "b")"#).is_ok());
        assert!(check("(:wat::core::= true false)").is_ok());
    }

    #[test]
    fn equality_mixed_types_rejected() {
        let err = check(r#"(:wat::core::= 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn less_than_mixed_types_rejected() {
        let err = check(r#"(:wat::core::< 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic list (T* -> List<T>) ───────────────────────────────

    #[test]
    fn list_same_type_passes() {
        assert!(check("(:wat::core::vec :i64 1 2 3)").is_ok());
        assert!(check(r#"(:wat::core::vec :String "a" "b")"#).is_ok());
    }

    #[test]
    fn list_mixed_types_rejected() {
        let err = check(r#"(:wat::core::vec :i64 1 "two" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bundle_of_list_of_holons_passes() {
        // Bundle takes :Vec<holon::HolonAST>. A list of (Atom ...) calls
        // returns :Vec<holon::HolonAST>, so Bundle(list(Atoms...)) type-checks.
        assert!(check(
            r#"(:wat::algebra::Bundle (:wat::core::vec :holon::HolonAST
                 (:wat::algebra::Atom 1)
                 (:wat::algebra::Atom 2)))"#
        )
        .is_ok());
    }

    #[test]
    fn bundle_of_list_of_ints_rejected() {
        // Bundle wants :Vec<holon::HolonAST>, but this is :Vec<i64>.
        let err = check(r#"(:wat::algebra::Bundle (:wat::core::vec :i64 1 2 3))"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── :rust::* dispatch via rust_deps registry ───────────────────────

    #[test]
    fn rust_lru_new_typechecks_via_let_annotation() {
        // No explicit :T on ::new — K,V flow from the let annotation
        // through the scheme's fresh vars via unification.
        let result = check(
            r#"(:wat::core::let*
                 (((cache :rust::lru::LruCache<String,i64>)
                   (:rust::lru::LruCache::new 16)))
                 cache)"#,
        );
        assert!(result.is_ok(), "expected ok, got {:?}", result.err());
    }

    #[test]
    fn rust_lru_put_typechecks_on_concrete_cache() {
        assert!(check(
            r#"(:wat::core::let*
                 (((cache :rust::lru::LruCache<String,i64>)
                   (:rust::lru::LruCache::new 16))
                  ((_ :()) (:rust::lru::LruCache::put cache "k" 42)))
                 cache)"#
        )
        .is_ok());
    }

    #[test]
    fn rust_lru_get_typechecks_returns_option_of_value_type() {
        assert!(check(
            r#"(:wat::core::let*
                 (((cache :rust::lru::LruCache<String,i64>)
                   (:rust::lru::LruCache::new 16)))
                 (:wat::core::match (:rust::lru::LruCache::get cache "k")
                   ((Some v) v)
                   (:None 0)))"#
        )
        .is_ok());
    }

    #[test]
    fn rust_lru_put_wrong_key_type_rejected() {
        // Cache is <String,i64>; putting i64 as key should fail.
        let err = check(
            r#"(:wat::core::let*
                 (((cache :rust::lru::LruCache<String,i64>)
                   (:rust::lru::LruCache::new 16))
                  ((_ :()) (:rust::lru::LruCache::put cache 42 1)))
                 cache)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn rust_unknown_symbol_rejected() {
        let err = check("(:rust::imaginary::Crate::method 1 2)").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::UnknownCallee { .. })));
    }

    // ─── User define signature checks ───────────────────────────────────

    #[test]
    fn user_define_body_matches_signature() {
        assert!(check(
            r#"(:wat::core::define (:my::app::add (x :i64) (y :i64) -> :i64)
                 (:wat::core::i64::+ x y))"#
        )
        .is_ok());
    }

    #[test]
    fn user_define_body_wrong_return_rejected() {
        let err = check(
            r#"(:wat::core::define (:my::app::add (x :i64) (y :i64) -> :bool)
                 (:wat::core::i64::+ x y))"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    #[test]
    fn user_parametric_define_passes() {
        // Identity: ∀T. T -> T. Body returns x, which has type T.
        // With rigid type variables, x: T unifies with ret: T.
        assert!(check(
            r#"(:wat::core::define (:my::app::id<T> (x :T) -> :T) x)"#
        )
        .is_ok());
    }

    #[test]
    fn user_parametric_wrong_return_rejected() {
        // Declared ret T; body returns an :i64 constant. Rigid T
        // doesn't unify with :i64.
        let err = check(
            r#"(:wat::core::define (:my::app::bad<T> (x :T) -> :T) 42)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    // ─── Typed-let discipline ───────────────────────────────────────────

    #[test]
    fn typed_let_binding_matches_rhs() {
        assert!(check(
            r#"(:wat::core::let (((x :i64) 42)) (:wat::core::i64::+ x 1))"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_wrong_type_rejected() {
        // Declared :i64 but RHS is :String — unification fails.
        let err = check(
            r#"(:wat::core::let (((x :i64) "hello")) x)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn typed_let_binding_multiple() {
        assert!(check(
            r#"(:wat::core::let
                 (((x :i64) 1)
                  ((y :i64) 2)
                  ((z :i64) 3))
                 (:wat::core::i64::+ (:wat::core::i64::+ x y) z))"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_with_lambda_value() {
        // A lambda bound to a let with :fn(i64)->i64 declaration.
        // Declared type matches lambda's own signature, so it passes.
        assert!(check(
            r#"(:wat::core::let
                 (((doubler :fn(i64)->i64)
                   (:wat::core::lambda ((x :i64) -> :i64)
                     (:wat::core::i64::+ x x))))
                 true)"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_lambda_declared_wrong_rejected() {
        // Declared :fn(i64)->bool but lambda produces :fn(i64)->i64.
        let err = check(
            r#"(:wat::core::let
                 (((f :fn(i64)->bool)
                   (:wat::core::lambda ((x :i64) -> :i64) x)))
                 true)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
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
        let err = parse_type_expr(":Vec<Any>").unwrap_err();
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
        let err = check(r#"(:wat::core::i64::+ "s" 1) (:wat::core::not 42)"#).unwrap_err();
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
        // Different parametric heads must NOT unify: Vec<i64> vs Option<i64>.
        let mut s = Subst::new();
        let vec_int = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        let option_int = TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        assert!(unify(&vec_int, &option_int, &mut s).is_err());
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
            head: "Vec".into(),
            args: vec![TypeExpr::Var(0)],
        };
        assert!(unify(&TypeExpr::Var(0), &cyclic, &mut s).is_err());
    }

    // ─── Parse + unify round-trip ───────────────────────────────────────

    #[test]
    fn type_expr_parse_and_unify() {
        let mut s = Subst::new();
        let a = parse_type_expr(":holon::HolonAST").unwrap();
        let b = parse_type_expr(":holon::HolonAST").unwrap();
        assert!(unify(&a, &b, &mut s).is_ok());
    }
}
