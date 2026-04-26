//! Type-check pass — rank-1 Hindley-Milner.
//!
//! This slice implements real parametric polymorphism per 058-030:
//!
//! - [`TypeScheme`] carries `type_params` — the list of names that are
//!   universally quantified (e.g., `["T"]` for `∀T. T -> :wat::holon::HolonAST`).
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
//!   `∀T. T -> :wat::holon::HolonAST`.
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
use std::sync::Arc;

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

/// Cross-cutting context threaded through every `infer_*` helper.
/// Owns two concerns that need global scope during a single
/// `check_program` run:
///
/// 1. **Fresh type-variable ids.** A monotonic counter that hands out
///    unique `TypeExpr::Var(n)` ids so distinct unification variables
///    never collide across call sites or function bodies.
/// 2. **Enclosing return-type stack.** Pushed on entry to every
///    function body and lambda body, popped on exit, consulted by
///    `infer_try` to unify the propagated `E` with the enclosing
///    function/lambda's own `Err` variant. LIFO so the innermost
///    enclosing scope wins — matches Rust's `?`-operator scoping.
///
/// The parameter name in most call sites is still `fresh` by
/// convention — the ctx's primary role was originally just fresh-var
/// generation, and the shorter name reads naturally for that case.
/// New concerns added here (scoped flags, effect rows, whatever) land
/// as additional fields without further renames.
#[derive(Debug, Default)]
struct InferCtx {
    next: u64,
    enclosing_rets: Vec<TypeExpr>,
}

impl InferCtx {
    fn fresh(&mut self) -> TypeExpr {
        let v = TypeExpr::Var(self.next);
        self.next += 1;
        v
    }

    /// Push the declared return type of a function/lambda we are about
    /// to check. Paired with [`pop_enclosing_ret`].
    fn push_enclosing_ret(&mut self, ret: TypeExpr) {
        self.enclosing_rets.push(ret);
    }

    /// Pop the most recently pushed return type. Caller is responsible
    /// for pairing pushes and pops at scope boundaries.
    fn pop_enclosing_ret(&mut self) {
        self.enclosing_rets.pop();
    }

    /// Innermost enclosing return type, if any. `None` outside any
    /// function/lambda body (top-level `check_form` invocations).
    fn enclosing_ret(&self) -> Option<&TypeExpr> {
        self.enclosing_rets.last()
    }
}

/// Substitution map: unification variable id → its concrete type.
/// Updated as unification succeeds; applied via [`apply_subst`] to
/// resolve a type to its canonical form.
type Subst = HashMap<u64, TypeExpr>;

/// The type-check environment: built-in + user function schemes plus
/// a shared handle to the [`TypeEnv`] (user type declarations).
/// Unification consults the type-env to expand typealiases to their
/// structural definitions before the structural match.
#[derive(Debug)]
pub struct CheckEnv {
    schemes: HashMap<String, TypeScheme>,
    /// Arc 048 — keyword paths for user-enum unit variants mapped to
    /// the enum's type. When `infer` sees one of these as a value-
    /// position keyword (e.g. `:trading::types::PhaseLabel::Valley`),
    /// it returns the enum's type instead of the generic
    /// `:wat::core::keyword`. Mirrors the runtime's
    /// `SymbolTable.unit_variants`. Populated at construction by
    /// walking every `:wat::core::enum` declaration in `types`.
    unit_variant_types: HashMap<String, TypeExpr>,
    types: Arc<TypeEnv>,
}

impl CheckEnv {
    pub fn new() -> Self {
        Self::with_types(Arc::new(TypeEnv::with_builtins()))
    }

    /// Build an env with built-in schemes for `:wat::core::*` and
    /// `:wat::holon::*` forms, then overlay user-define signatures
    /// from `sym`. `types` carries the registered user type
    /// declarations (struct/enum/newtype/typealias) — unification uses
    /// it to expand aliases.
    pub fn from_symbols(sym: &SymbolTable, types: Arc<TypeEnv>) -> Self {
        let mut env = Self::with_builtins_and_types(types);
        for (path, func) in &sym.functions {
            if let Some(scheme) = derive_scheme_from_function(func) {
                env.register(path.clone(), scheme);
            }
        }
        env
    }

    pub fn with_builtins() -> Self {
        Self::with_builtins_and_types(Arc::new(TypeEnv::with_builtins()))
    }

    pub fn with_builtins_and_types(types: Arc<TypeEnv>) -> Self {
        let mut env = Self::with_types(types);
        register_builtins(&mut env);
        env
    }

    fn with_types(types: Arc<TypeEnv>) -> Self {
        // Arc 048 — pre-populate unit-variant keyword types from the
        // declared enums. Walks every TypeDef::Enum and registers each
        // unit variant's full keyword path (`:enum::Variant`) → enum
        // type, so `infer` can return the enum type when the bare
        // keyword appears in expression position.
        let mut unit_variant_types = HashMap::new();
        for (name, def) in types.iter() {
            if let crate::types::TypeDef::Enum(e) = def {
                for variant in &e.variants {
                    if let crate::types::EnumVariant::Unit(variant_name) = variant {
                        let key = format!("{}::{}", name, variant_name);
                        unit_variant_types.insert(key, TypeExpr::Path(name.clone()));
                    }
                }
            }
        }
        CheckEnv {
            schemes: HashMap::new(),
            unit_variant_types,
            types,
        }
    }

    /// Arc 048 — look up the enum type for a unit-variant keyword
    /// path. Returns `None` for non-variant keywords.
    pub fn unit_variant_type(&self, key: &str) -> Option<&TypeExpr> {
        self.unit_variant_types.get(key)
    }

    pub fn register(&mut self, name: String, scheme: TypeScheme) {
        self.schemes.insert(name, scheme);
    }

    pub fn get(&self, name: &str) -> Option<&TypeScheme> {
        self.schemes.get(name)
    }

    /// Handle to the user/builtin type declarations. Used by `unify`
    /// to expand typealiases to their structural form before the
    /// structural match.
    pub fn types(&self) -> &TypeEnv {
        &self.types
    }
}

impl Default for CheckEnv {
    fn default() -> Self {
        Self::new()
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
    types: &TypeEnv,
) -> Result<(), CheckErrors> {
    let env = CheckEnv::from_symbols(sym, Arc::new(types.clone()));
    let mut errors = Vec::new();
    let mut fresh = InferCtx::default();

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
    fresh: &mut InferCtx,
    errors: &mut Vec<CheckError>,
) {
    // Declared type parameters are RIGID inside the body — rigid
    // meaning they unify only with themselves. Represented as
    // `Path(":T")` where T is the declared name; the checker
    // distinguishes rigid names from fresh unification Vars.
    let locals = build_locals(&func.params, &scheme.params);
    let mut subst = Subst::new();
    // Push this function's declared return type so `infer_try`, if it
    // recurses into the body, can unify its propagated `Err` with this
    // function's own `Result<_, E>` shape.
    fresh.push_enclosing_ret(scheme.ret.clone());
    let body_ty = infer(&func.body, env, &locals, fresh, &mut subst, errors);
    fresh.pop_enclosing_ret();
    // Unify body type with declared return type. If unification fails,
    // produce a ReturnTypeMismatch.
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &scheme.ret, &mut subst, env.types()).is_err() {
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
    fresh: &mut InferCtx,
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
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    match ast {
        WatAST::IntLit(_, _) => Some(TypeExpr::Path(":i64".into())),
        WatAST::FloatLit(_, _) => Some(TypeExpr::Path(":f64".into())),
        WatAST::BoolLit(_, _) => Some(TypeExpr::Path(":bool".into())),
        WatAST::StringLit(_, _) => Some(TypeExpr::Path(":String".into())),
        // `:None` — nullary constructor of the built-in :Option<T> enum.
        // Infers as `:Option<T>` with a fresh T; unification against the
        // expected type sharpens T at the use site.
        WatAST::Keyword(k, _) if k == ":None" => Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![fresh.fresh()],
        }),
        // Arc 048 — user-enum unit variant. The bare keyword resolves
        // to the enum's type (e.g. `:trading::types::PhaseLabel::Valley`
        // → `:trading::types::PhaseLabel`).
        WatAST::Keyword(k, _) if env.unit_variant_type(k).is_some() => {
            Some(env.unit_variant_type(k).expect("guard").clone())
        }
        // Arc 009 — names are values. If the keyword is a registered
        // function (user define, stdlib define, or builtin primitive),
        // instantiate its scheme and return a `:fn(...)->Ret` type so
        // the keyword can be passed to any `:fn(...)`-typed parameter.
        // Mirrors `infer_spawn`'s long-standing keyword-path path,
        // generalized to every expression position.
        WatAST::Keyword(k, _) if env.get(k).is_some() => {
            let scheme = env.get(k).expect("guard").clone();
            let (params, ret) = instantiate(&scheme, fresh);
            Some(TypeExpr::Fn {
                args: params,
                ret: Box::new(ret),
            })
        }
        WatAST::Keyword(_, _) => Some(TypeExpr::Path(":wat::core::keyword".into())),
        WatAST::Symbol(ident, _) => locals.get(&ident.name).cloned(),
        WatAST::List(items, _) => infer_list(items, env, locals, fresh, subst, errors),
    }
}

fn infer_list(
    items: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // `()` — empty list — is the unit value. Type :() per the
    // existing TypeExpr::Tuple([]) encoding.
    let head = match items.first() {
        Some(h) => h,
        None => return Some(TypeExpr::Tuple(vec![])),
    };

    if let WatAST::Keyword(k, _) = head {
        let args = &items[1..];
        match k.as_str() {
            ":wat::core::if" => return infer_if(args, env, locals, fresh, subst, errors),
            ":wat::core::cond" => return infer_cond(args, env, locals, fresh, subst, errors),
            ":wat::core::let" => return infer_let(args, env, locals, fresh, subst, errors),
            ":wat::core::let*" => return infer_let_star(args, env, locals, fresh, subst, errors),
            ":wat::core::try" => return infer_try(args, env, locals, fresh, subst, errors),
            ":wat::core::vec" => return infer_list_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::list" => return infer_list_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::tuple" => return infer_tuple_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::string::concat" => return infer_string_concat(args, env, locals, fresh, subst, errors),
            ":wat::core::HashMap" => return infer_hashmap_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::assoc" => return infer_assoc(args, env, locals, fresh, subst, errors),
            ":wat::core::concat" => return infer_concat(args, env, locals, fresh, subst, errors),
            ":wat::core::dissoc" => return infer_dissoc(args, env, locals, fresh, subst, errors),
            ":wat::core::keys" => return infer_keys(args, env, locals, fresh, subst, errors),
            ":wat::core::values" => return infer_values(args, env, locals, fresh, subst, errors),
            ":wat::core::empty?" => return infer_empty_q(args, env, locals, fresh, subst, errors),
            ":wat::core::conj" => return infer_conj(args, env, locals, fresh, subst, errors),
            ":wat::core::contains?" => return infer_contains_q(args, env, locals, fresh, subst, errors),
            ":wat::core::length" => return infer_length(args, env, locals, fresh, subst, errors),
            ":wat::core::HashSet" => return infer_hashset_constructor(args, env, locals, fresh, subst, errors),
            ":wat::core::get" => return infer_get(args, env, locals, fresh, subst, errors),
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
            ":wat::core::forms" => {
                // Variadic sibling of quote. Every positional arg is
                // DATA, captured as `:wat::WatAST`. The checker does
                // not recurse into any of them. Return type is
                // `:Vec<wat::WatAST>` regardless of arity (including
                // zero, which produces an empty Vec).
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::WatAST".into())],
                });
            }
            ":wat::core::macroexpand-1" | ":wat::core::macroexpand" => {
                // Arc 030: macro debugging primitives.
                // (:wat::core::macroexpand{-1}? <wat::WatAST>) -> :wat::WatAST
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: k.clone(),
                        expected: 1,
                        got: args.len(),
                    });
                    return Some(TypeExpr::Path(":wat::WatAST".into()));
                }
                if let Some(arg_ty) = infer(&args[0], env, locals, fresh, subst, errors) {
                    let expected = TypeExpr::Path(":wat::WatAST".into());
                    if unify(&arg_ty, &expected, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: k.clone(),
                            param: "#1".into(),
                            expected: format_type(&apply_subst(&expected, subst)),
                            got: format_type(&apply_subst(&arg_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            ":wat::core::match" => {
                return infer_match(args, env, locals, fresh, subst, errors);
            }
            // Arc 050 — polymorphic comparison/equality. Same-type
            // for non-numeric, cross-numeric promotion for (i64, f64)
            // pairs. Always returns :bool. `not=` (Clojure tradition)
            // shares the inference path with `=` since the rules are
            // identical; only the runtime differs.
            ":wat::core::="
            | ":wat::core::not="
            | ":wat::core::<"
            | ":wat::core::>"
            | ":wat::core::<="
            | ":wat::core::>=" => {
                return infer_polymorphic_compare(k, args, env, locals, fresh, subst, errors);
            }
            // Arc 050 — polymorphic arithmetic. Both args must be
            // numeric (i64 or f64); result type is f64 if either is
            // f64, else i64.
            ":wat::core::+"
            | ":wat::core::-"
            | ":wat::core::*"
            | ":wat::core::/" => {
                return infer_polymorphic_arith(k, args, env, locals, fresh, subst, errors);
            }
            // Arc 052 — polymorphic algebra ops. Cosine and dot accept
            // HolonAST or Vector in either position; simhash accepts
            // HolonAST or Vector as its single argument.
            ":wat::holon::cosine" | ":wat::holon::dot" => {
                return infer_polymorphic_holon_pair_to_f64(
                    k, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::holon::simhash" => {
                return infer_polymorphic_holon_to_i64(
                    k, args, env, locals, fresh, subst, errors,
                );
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
            | ":wat::load-file!"
            | ":wat::digest-load!"
            | ":wat::signed-load!"
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
                if unify(&arg_ty, expected, subst, env.types()).is_err() {
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
    if let WatAST::Symbol(ident, _) = head {
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
        // `(Ok expr)` — built-in tagged constructor of `:Result<T,E>`.
        // Infers T from the argument; E is a fresh var for later
        // unification against the match arms or declared type.
        if ident.as_str() == "Ok" {
            let args = &items[1..];
            if args.len() != 1 {
                errors.push(CheckError::ArityMismatch {
                    callee: "Ok".into(),
                    expected: 1,
                    got: args.len(),
                });
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "Result".into(),
                    args: vec![fresh.fresh(), fresh.fresh()],
                });
            }
            let t_ty = infer(&args[0], env, locals, fresh, subst, errors)
                .unwrap_or_else(|| fresh.fresh());
            let e_var = fresh.fresh();
            return Some(TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![t_ty, e_var],
            });
        }
        // `(Err expr)` — dual. Infers E from the argument; T is fresh.
        if ident.as_str() == "Err" {
            let args = &items[1..];
            if args.len() != 1 {
                errors.push(CheckError::ArityMismatch {
                    callee: "Err".into(),
                    expected: 1,
                    got: args.len(),
                });
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "Result".into(),
                    args: vec![fresh.fresh(), fresh.fresh()],
                });
            }
            let e_ty = infer(&args[0], env, locals, fresh, subst, errors)
                .unwrap_or_else(|| fresh.fresh());
            let t_var = fresh.fresh();
            return Some(TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![t_var, e_ty],
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
/// `(:wat::core::match scrutinee -> :T arm1 arm2 ...)` — typed match.
///
/// Per the 2026-04-20 INSCRIPTION, match now requires an explicit
/// `-> :T` declaration between the scrutinee and the arms. Every
/// arm body is checked against `:T` independently so divergent
/// arms produce a per-arm TypeMismatch naming the declared type.
/// The old no-annotation form is refused with a migration-hint
/// MalformedForm.
fn infer_match(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // Pre-inscription shape detection: if args[1] isn't `->`, this
    // is the old form. Surface a migration-hint error before the
    // standard arity check so authors see the right guidance.
    if args.len() >= 2
        && !matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->")
    {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: "`:wat::core::match` now requires `-> :T` between scrutinee and arms; write (:wat::core::match scrut -> :T (pat body) ...)".into(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    if args.len() < 4 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "expected (:wat::core::match scrut -> :T arm1 arm2 ...) — at least 4 args; got {}",
                args.len()
            ),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    // Parse the declared `:T`.
    let declared_ty = match &args[2] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "expected type keyword after `->`".into(),
            });
            return None;
        }
    };

    // Detect shape from the arms (arms begin at args[3..]).
    let arm_refs: Vec<&WatAST> = args[3..].iter().collect();
    let shape = detect_match_shape(&arm_refs, env, fresh);

    // Scrutinee must unify with the detected shape.
    let scrutinee_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let expected_scrutinee = shape.as_type();
    if let Some(sty) = &scrutinee_ty {
        if unify(sty, &expected_scrutinee, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::match".into(),
                param: "scrutinee".into(),
                expected: format_type(&expected_scrutinee),
                got: format_type(&apply_subst(sty, subst)),
            });
        }
    }

    // Arc 055 — resolve the shape's inner types via the substitution
    // *now* so recursive sub-pattern checking sees concrete types
    // (e.g. `Option<fresh>` → `Option<(i64,i64,i64)>` once the
    // scrutinee unifies with a let-bound variable).
    let shape = match &shape {
        MatchShape::Option(t) => MatchShape::Option(apply_subst(t, subst)),
        MatchShape::Result(t, e) => {
            MatchShape::Result(apply_subst(t, subst), apply_subst(e, subst))
        }
        MatchShape::Enum(p) => MatchShape::Enum(p.clone()),
    };

    let mut covers_option_none = false;
    let mut covers_option_some = false;
    let mut covers_result_ok = false;
    let mut covers_result_err = false;
    let mut wildcard_seen = false;
    // Arc 048 — track which user-enum variant names have arms.
    let mut covered_enum_variants: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (idx, arm) in args[3..].iter().enumerate() {
        let arm_items = match arm {
            WatAST::List(items, _) if items.len() == 2 => items,
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
        match pattern_coverage(pattern, &shape, env, &mut arm_locals, errors) {
            Some(Coverage::OptionNone) => covers_option_none = true,
            // Arc 055 — partial Some (e.g. `(Some (1 _))`) does not
            // satisfy Some-coverage; needs a fallback arm.
            Some(Coverage::OptionSome { full: true }) => covers_option_some = true,
            Some(Coverage::OptionSome { full: false }) => {}
            Some(Coverage::ResultOk { full: true }) => covers_result_ok = true,
            Some(Coverage::ResultOk { full: false }) => {}
            Some(Coverage::ResultErr { full: true }) => covers_result_err = true,
            Some(Coverage::ResultErr { full: false }) => {}
            Some(Coverage::EnumVariant { name, full: true }) => {
                covered_enum_variants.insert(name);
            }
            Some(Coverage::EnumVariant { full: false, .. }) => {}
            Some(Coverage::Wildcard) => {
                wildcard_seen = true;
                covers_option_none = true;
                covers_option_some = true;
                covers_result_ok = true;
                covers_result_err = true;
            }
            None => continue,
        }

        // Each arm body checked against the declared `:T` independently.
        let arm_ty = infer(body, env, &arm_locals, fresh, subst, errors);
        if let Some(t) = arm_ty {
            if unify(&t, &declared_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::match".into(),
                    param: format!("arm #{}", idx + 1),
                    expected: format_type(&apply_subst(&declared_ty, subst)),
                    got: format_type(&apply_subst(&t, subst)),
                });
            }
        }
    }

    let exhaustive = match &shape {
        MatchShape::Option(_) => covers_option_none && covers_option_some,
        MatchShape::Result(_, _) => covers_result_ok && covers_result_err,
        MatchShape::Enum(enum_path) => {
            if wildcard_seen {
                true
            } else if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                e.variants.iter().all(|v| {
                    let name = match v {
                        crate::types::EnumVariant::Unit(n) => n,
                        crate::types::EnumVariant::Tagged { name, .. } => name,
                    };
                    covered_enum_variants.contains(name)
                })
            } else {
                false
            }
        }
    };
    if !exhaustive {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: match &shape {
                MatchShape::Option(_) => "non-exhaustive: :Option<T> needs arms for both :None and (Some _), or a wildcard. (Arc 055 — narrowing patterns like `(Some (1 _))` are partial; add a fallback `_` arm.)".into(),
                MatchShape::Result(_, _) => "non-exhaustive: :Result<T,E> needs arms for both (Ok _) and (Err _), or a wildcard. (Arc 055 — narrowing patterns like `(Ok 200)` are partial; add a fallback `_` arm.)".into(),
                MatchShape::Enum(enum_path) => {
                    if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                        let missing: Vec<String> = e.variants.iter().filter_map(|v| {
                            let name = match v {
                                crate::types::EnumVariant::Unit(n) => n,
                                crate::types::EnumVariant::Tagged { name, .. } => name,
                            };
                            if covered_enum_variants.contains(name) {
                                None
                            } else {
                                Some(name.clone())
                            }
                        }).collect();
                        format!(
                            "non-exhaustive: enum {} missing arm(s) for variant(s): {} (or include `_` wildcard)",
                            enum_path,
                            missing.join(", ")
                        )
                    } else {
                        format!("non-exhaustive: enum {} missing arms (or include `_` wildcard)", enum_path)
                    }
                }
            },
        });
    }

    Some(apply_subst(&declared_ty, subst))
}

/// Coverage class for a match pattern. Spans built-in `:Option<T>`,
/// `:Result<T,E>`, and (arc 048) user-defined enums. Wildcard covers
/// any shape.
///
/// Arc 055 — variant-carrying coverage classes carry a `full` flag.
/// `full=true` means the variant arm's inner sub-pattern is fully
/// general (bare symbol or `_` recursively); `full=false` means the
/// arm narrows the variant's space (a literal or nested variant
/// somewhere inside) and a fallback wildcard arm is required to
/// remain exhaustive.
enum Coverage {
    OptionNone,
    OptionSome { full: bool },
    ResultOk { full: bool },
    ResultErr { full: bool },
    /// Arc 048 — user-enum variant covered. Carries the variant's
    /// bare name (e.g. "Valley") for exhaustiveness checking against
    /// the enum's declared variant set. Arc 055 — `full` flag tracks
    /// whether the inner sub-pattern is fully general.
    EnumVariant {
        name: String,
        full: bool,
    },
    Wildcard,
}

/// Which shape the match dispatches on. Determined by inspecting the
/// first variant-constructor arm.
#[derive(Clone, Debug)]
enum MatchShape {
    /// :Option<T> — inner_ty is T.
    Option(TypeExpr),
    /// :Result<T,E> — t_ty is T (Ok-inner), e_ty is E (Err-inner).
    Result(TypeExpr, TypeExpr),
    /// Arc 048 — user-defined enum. Carries the enum's full type path
    /// (e.g. `:trading::types::PhaseLabel`); the checker looks up the
    /// declared variant set in `CheckEnv.types` for exhaustiveness +
    /// per-variant arity.
    Enum(String),
}

impl MatchShape {
    fn as_type(&self) -> TypeExpr {
        match self {
            MatchShape::Option(t) => TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![t.clone()],
            },
            MatchShape::Result(t, e) => TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![t.clone(), e.clone()],
            },
            MatchShape::Enum(path) => TypeExpr::Path(path.clone()),
        }
    }
}

/// Scan the match arms to decide which shape the scrutinee matches.
/// First arm with a recognized variant-constructor pattern wins:
/// - `:None` or `(Some _)` → Option<T>
/// - `(Ok _)` or `(Err _)` → Result<T,E>
/// - `:enum::Variant` (unit) or `(:enum::Variant ...)` (tagged) → Enum
///   (arc 048). The keyword is split on the last `::` to separate
///   enum path from variant name; the prefix is looked up in the type
///   env to confirm it's a registered enum.
///
/// If no arm is definitive (all wildcards), defaults to Option with
/// a fresh T.
fn detect_match_shape(arms: &[&WatAST], env: &CheckEnv, fresh: &mut InferCtx) -> MatchShape {
    for arm in arms {
        if let WatAST::List(items, _) = arm {
            if items.len() == 2 {
                let pat = &items[0];
                match pat {
                    WatAST::Keyword(k, _) if k == ":None" => {
                        return MatchShape::Option(fresh.fresh());
                    }
                    WatAST::Keyword(k, _) => {
                        // Arc 048 — user-enum variant pattern (unit
                        // shape). First try the registered unit-variant
                        // map; falling back to enum-prefix lookup so a
                        // misapplied keyword pattern (e.g. tagged-variant
                        // name used in unit position) still classifies
                        // as Enum and produces the right error in
                        // pattern_coverage.
                        if let Some(TypeExpr::Path(enum_path)) = env.unit_variant_type(k) {
                            return MatchShape::Enum(enum_path.clone());
                        }
                        if let Some((enum_path, _)) = k.rsplit_once("::") {
                            if matches!(
                                env.types().get(enum_path),
                                Some(crate::types::TypeDef::Enum(_))
                            ) {
                                return MatchShape::Enum(enum_path.to_string());
                            }
                        }
                    }
                    WatAST::List(pat_items, _) => {
                        if let Some(WatAST::Symbol(ident, _)) = pat_items.first() {
                            match ident.as_str() {
                                "Some" => return MatchShape::Option(fresh.fresh()),
                                "Ok" | "Err" => {
                                    return MatchShape::Result(fresh.fresh(), fresh.fresh());
                                }
                                _ => {}
                            }
                        }
                        // Arc 048 — user-enum tagged variant pattern
                        // `(:enum::Variant binders...)`. Split the
                        // head keyword on the last `::` to get the
                        // enum path; if the path resolves to a
                        // declared enum, that's the shape.
                        if let Some(WatAST::Keyword(head_path, _)) = pat_items.first() {
                            if let Some((enum_path, _variant)) = head_path.rsplit_once("::") {
                                let enum_path_owned = enum_path.to_string();
                                if matches!(
                                    env.types().get(&enum_path_owned),
                                    Some(crate::types::TypeDef::Enum(_))
                                ) {
                                    return MatchShape::Enum(enum_path_owned);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    MatchShape::Option(fresh.fresh())
}

/// Validate `pattern` against the match shape, push bindings into
/// `bindings`, and report its coverage class.
fn pattern_coverage(
    pattern: &WatAST,
    shape: &MatchShape,
    env: &CheckEnv,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Option<Coverage> {
    match pattern {
        WatAST::Keyword(k, _) if k == ":None" => match shape {
            MatchShape::Option(_) => Some(Coverage::OptionNone),
            MatchShape::Result(_, _) | MatchShape::Enum(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        ":None pattern on a {} scrutinee",
                        format_type(&shape.as_type())
                    ),
                });
                None
            }
        },
        // Arc 048 — user-enum unit variant pattern. The keyword
        // path must split as `<enum>::<Variant>` where `<enum>`
        // matches the scrutinee shape's Enum path AND `<Variant>`
        // is a unit variant of that enum.
        WatAST::Keyword(k, _) => match shape {
            MatchShape::Enum(enum_path) => {
                let (prefix, variant_name) = match k.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword pattern {} must be `<enum>::<Variant>`",
                                k
                            ),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant pattern {} doesn't belong to scrutinee enum {}",
                            k, enum_path
                        ),
                    });
                    return None;
                }
                // Verify Variant is declared (and is a unit variant).
                if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                    let is_unit = e.variants.iter().any(|v| {
                        matches!(v, crate::types::EnumVariant::Unit(n) if n == variant_name)
                    });
                    let is_tagged = e.variants.iter().any(|v| {
                        matches!(v, crate::types::EnumVariant::Tagged { name, .. } if name == variant_name)
                    });
                    if !is_unit && is_tagged {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is a tagged variant; pattern must be (`{}` binders...)",
                                k, k
                            ),
                        });
                        return None;
                    }
                    if !is_unit {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant {} is not declared on enum {}",
                                variant_name, enum_path
                            ),
                        });
                        return None;
                    }
                    // Unit variant — no fields, vacuously fully general.
                    Some(Coverage::EnumVariant {
                        name: variant_name.to_string(),
                        full: true,
                    })
                } else {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!("enum {} not declared", enum_path),
                    });
                    None
                }
            }
            _ => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "keyword pattern {} not valid on a {} scrutinee",
                        k,
                        format_type(&shape.as_type())
                    ),
                });
                None
            }
        },
        WatAST::Symbol(ident, _) if ident.as_str() == "_" => Some(Coverage::Wildcard),
        WatAST::Symbol(ident, _) => {
            // Bare name binds the whole scrutinee.
            bindings.insert(ident.as_str().to_string(), shape.as_type());
            Some(Coverage::Wildcard)
        }
        WatAST::List(items, _) => {
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
            // Arc 048 — user-enum tagged variant pattern: head is a
            // keyword path `:enum::Variant`. Split, validate, bind
            // fields by position.
            if let WatAST::Keyword(variant_path, _) = head {
                let enum_path = match shape {
                    MatchShape::Enum(p) => p,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword variant pattern {} on a {} scrutinee",
                                variant_path,
                                format_type(&shape.as_type())
                            ),
                        });
                        return None;
                    }
                };
                let (prefix, variant_name) = match variant_path.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant constructor pattern {} must be `<enum>::<Variant>`",
                                variant_path
                            ),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant constructor {} doesn't belong to scrutinee enum {}",
                            variant_path, enum_path
                        ),
                    });
                    return None;
                }
                let enum_def = match env.types().get(enum_path) {
                    Some(crate::types::TypeDef::Enum(e)) => e,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!("enum {} not declared", enum_path),
                        });
                        return None;
                    }
                };
                let fields = enum_def.variants.iter().find_map(|v| {
                    if let crate::types::EnumVariant::Tagged { name, fields } = v {
                        if name == variant_name {
                            return Some(fields);
                        }
                    }
                    None
                });
                let fields = match fields {
                    Some(f) => f,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is not a tagged variant of {}",
                                variant_path, enum_path
                            ),
                        });
                        return None;
                    }
                };
                if rest.len() != fields.len() {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "({} ...) takes {} field(s), got {} binder(s)",
                            variant_path,
                            fields.len(),
                            rest.len()
                        ),
                    });
                    return None;
                }
                // Arc 055 — recurse into each field's sub-pattern.
                let mut all_full = true;
                for (binder_ast, (_field_name, field_type)) in rest.iter().zip(fields.iter()) {
                    match check_subpattern(binder_ast, field_type, env, bindings, errors) {
                        Some(full) => all_full &= full,
                        None => return None,
                    }
                }
                return Some(Coverage::EnumVariant {
                    name: variant_name.to_string(),
                    full: all_full,
                });
            }
            let ident = match head {
                WatAST::Symbol(i, _) => i.as_str(),
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "list pattern head must be a variant constructor; got {}",
                            ast_variant_name_check(other)
                        ),
                    });
                    return None;
                }
            };
            // Arc 055 — variant arm dispatches on shape, then recurses
            // into the inner sub-pattern via `check_subpattern`. The
            // returned `full` flag tracks whether the sub-pattern is
            // fully general (bare symbol or `_` recursively); narrowing
            // sub-patterns produce `full: false` and require a fallback.
            let (ctor_name, mk_coverage, expected_bind_ty): (
                &str,
                fn(bool) -> Coverage,
                TypeExpr,
            ) = match (ident, shape) {
                ("Some", MatchShape::Option(t)) => (
                    "Some",
                    |full| Coverage::OptionSome { full },
                    t.clone(),
                ),
                ("Ok", MatchShape::Result(t, _)) => (
                    "Ok",
                    |full| Coverage::ResultOk { full },
                    t.clone(),
                ),
                ("Err", MatchShape::Result(_, e)) => (
                    "Err",
                    |full| Coverage::ResultErr { full },
                    e.clone(),
                ),
                (other, _) => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant constructor `{}` does not match scrutinee shape ({})",
                            other,
                            format_type(&shape.as_type())
                        ),
                    });
                    return None;
                }
            };
            if rest.len() != 1 {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "({} _) takes exactly one field, got {}",
                        ctor_name,
                        rest.len()
                    ),
                });
                return None;
            }
            check_subpattern(&rest[0], &expected_bind_ty, env, bindings, errors)
                .map(mk_coverage)
        }
        other => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "pattern must be keyword, symbol, or list; got {}",
                    ast_variant_name_check(other)
                ),
            });
            None
        }
    }
}

/// Arc 055 — recursive sub-pattern checker.
///
/// Validates a sub-pattern (anywhere inside a variant or tuple) against
/// the type expected at that position. Populates `bindings` with any
/// bare-symbol binders introduced. Returns `Some(full)` on success
/// (where `full` indicates the sub-pattern is bare-symbol-or-wildcard
/// at every level — a fully-general match), `None` on type/shape
/// mismatch (with errors pushed).
///
/// Disambiguation at list-position is by `expected_ty`:
/// - `Option<U>`: list head Symbol "Some" is the variant constructor.
/// - `Result<T,E>`: list head Symbol "Ok" / "Err" are constructors.
/// - Enum: list head Keyword `:enum::Variant` is the constructor.
/// - Tuple `(T1,...,Tn)`: list is positional destructure; recurse on
///   each element type. The head can be any sub-pattern (bare symbol,
///   variant, literal, nested tuple) — no special "constructor" status.
///
/// `full` is conservative: any literal, variant constructor, or
/// keyword-narrowed pattern at any depth makes the result `false`. The
/// v1 exhaustiveness rule then demands a fallback wildcard arm at the
/// top level. A more sophisticated literal-narrowing analyzer can ship
/// later without changing this helper's contract.
fn check_subpattern(
    pat: &WatAST,
    expected_ty: &TypeExpr,
    env: &CheckEnv,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Option<bool> {
    match pat {
        // Wildcard — fully general.
        WatAST::Symbol(s, _) if s.as_str() == "_" => Some(true),
        // Bare binder — fully general; binds the matched value.
        WatAST::Symbol(s, _) => {
            bindings.insert(s.as_str().to_string(), expected_ty.clone());
            Some(true)
        }
        // Literal sub-patterns — narrow the variant's space; partial.
        WatAST::IntLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":i64" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "int literal pattern in {} position",
                        format_type(other)
                    ),
                });
                None
            }
        },
        WatAST::FloatLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":f64" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "float literal pattern in {} position",
                        format_type(other)
                    ),
                });
                None
            }
        },
        WatAST::BoolLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":bool" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "bool literal pattern in {} position",
                        format_type(other)
                    ),
                });
                None
            }
        },
        WatAST::StringLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":String" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "string literal pattern in {} position",
                        format_type(other)
                    ),
                });
                None
            }
        },
        // Keyword sub-patterns:
        // - `:None` — only valid at Option<U> position; partial (only None).
        // - `:enum::Variant` (unit) — valid at enum position.
        // - bare keyword payload (rare in pattern position) — error.
        WatAST::Keyword(k, _) if k == ":None" => match expected_ty {
            TypeExpr::Parametric { head, .. } if head == "Option" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        ":None pattern in {} position",
                        format_type(other)
                    ),
                });
                None
            }
        },
        WatAST::Keyword(k, _) => {
            // User-enum unit variant pattern: `:enum::Variant` against
            // the matching enum type at this position.
            let (prefix, variant_name) = match k.rsplit_once("::") {
                Some(p) => p,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "keyword sub-pattern {} must be `<enum>::<Variant>` or `:None`",
                            k
                        ),
                    });
                    return None;
                }
            };
            let enum_path = match expected_ty {
                TypeExpr::Path(p) => p.as_str(),
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "keyword variant pattern {} in {} position",
                            k,
                            format_type(other)
                        ),
                    });
                    return None;
                }
            };
            if prefix != enum_path {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "variant pattern {} doesn't belong to expected enum {}",
                        k, enum_path
                    ),
                });
                return None;
            }
            if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                let is_unit = e.variants.iter().any(|v| {
                    matches!(v, crate::types::EnumVariant::Unit(n) if n == variant_name)
                });
                if !is_unit {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "{} is not a unit variant of {} (use `({} ...)` for tagged variants)",
                            k, enum_path, k
                        ),
                    });
                    return None;
                }
                Some(false)
            } else {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("enum {} not declared", enum_path),
                });
                None
            }
        }
        WatAST::List(items, _) => {
            let head = match items.first() {
                Some(h) => h,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "empty list sub-pattern".into(),
                    });
                    return None;
                }
            };
            // Variant-constructor list at this sub-position:
            // dispatch on expected_ty's shape.
            // Built-in Some/Ok/Err — head is Symbol.
            if let WatAST::Symbol(ident, _) = head {
                match (ident.as_str(), expected_ty) {
                    ("Some", TypeExpr::Parametric { head: h, args })
                        if h == "Option" && args.len() == 1 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Some _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[0], env, bindings, errors)?;
                        return Some(false);
                    }
                    ("Ok", TypeExpr::Parametric { head: h, args })
                        if h == "Result" && args.len() == 2 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Ok _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[0], env, bindings, errors)?;
                        return Some(false);
                    }
                    ("Err", TypeExpr::Parametric { head: h, args })
                        if h == "Result" && args.len() == 2 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Err _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[1], env, bindings, errors)?;
                        return Some(false);
                    }
                    _ => {
                        // Fall through to tuple destructure below.
                    }
                }
            }
            // User-enum tagged variant: head is Keyword `:enum::Variant`.
            if let WatAST::Keyword(variant_path, _) = head {
                let enum_path = match expected_ty {
                    TypeExpr::Path(p) => p.as_str(),
                    other => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword variant pattern {} in {} position",
                                variant_path,
                                format_type(other)
                            ),
                        });
                        return None;
                    }
                };
                let (prefix, variant_name) = match variant_path.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant constructor pattern {} must be `<enum>::<Variant>`",
                                variant_path
                            ),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant constructor {} doesn't belong to expected enum {}",
                            variant_path, enum_path
                        ),
                    });
                    return None;
                }
                let enum_def = match env.types().get(enum_path) {
                    Some(crate::types::TypeDef::Enum(e)) => e,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!("enum {} not declared", enum_path),
                        });
                        return None;
                    }
                };
                let fields = enum_def.variants.iter().find_map(|v| {
                    if let crate::types::EnumVariant::Tagged { name, fields } = v {
                        if name == variant_name {
                            return Some(fields);
                        }
                    }
                    None
                });
                let fields = match fields {
                    Some(f) => f,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is not a tagged variant of {}",
                                variant_path, enum_path
                            ),
                        });
                        return None;
                    }
                };
                let rest = &items[1..];
                if rest.len() != fields.len() {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "({} ...) takes {} field(s), got {}",
                            variant_path,
                            fields.len(),
                            rest.len()
                        ),
                    });
                    return None;
                }
                for (sub_pat, (_field_name, field_type)) in rest.iter().zip(fields.iter()) {
                    check_subpattern(sub_pat, field_type, env, bindings, errors)?;
                }
                return Some(false);
            }
            // Tuple destructure: expected_ty must be a tuple of matching arity.
            match expected_ty {
                TypeExpr::Tuple(elem_tys) => {
                    if items.len() != elem_tys.len() {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "tuple pattern arity {} mismatched with type arity {}",
                                items.len(),
                                elem_tys.len()
                            ),
                        });
                        return None;
                    }
                    let mut all_full = true;
                    for (sub_pat, sub_ty) in items.iter().zip(elem_tys.iter()) {
                        match check_subpattern(sub_pat, sub_ty, env, bindings, errors) {
                            Some(full) => all_full &= full,
                            None => return None,
                        }
                    }
                    Some(all_full)
                }
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "list sub-pattern in {} position (expected tuple, Option, Result, or enum)",
                            format_type(other)
                        ),
                    });
                    None
                }
            }
        }
    }
}

fn ast_variant_name_check(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_, _) => "int",
        WatAST::FloatLit(_, _) => "float",
        WatAST::BoolLit(_, _) => "bool",
        WatAST::StringLit(_, _) => "string",
        WatAST::Keyword(_, _) => "keyword",
        WatAST::Symbol(_, _) => "symbol",
        WatAST::List(_, _) => "list",
    }
}

/// `(:wat::core::if cond -> :T then else)` — typed conditional per
/// the 2026-04-20 INSCRIPTION.
///
/// Arity: 5 args exactly. Positions: [cond, `->`, `:T`, then, else].
/// The declared `:T` is the expected type for BOTH branches; each
/// branch body is checked against it independently so the error
/// message names WHICH branch diverged (rather than "branches
/// didn't unify" which doesn't name the author's intent).
///
/// The old 3-arg form is refused with a migration-hint MalformedForm
/// at resolve time via the runtime's eval_if; by the time we reach
/// infer_if with the wrong arity, we emit MalformedForm and bail.
fn infer_if(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() == 3 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` now requires `-> :T` between cond and then-branch; write (:wat::core::if cond -> :T then else)".into(),
        });
        // Still recurse into the body so inner errors surface too.
        let _ = infer(&args[0], env, locals, fresh, subst, errors);
        let _ = infer(&args[1], env, locals, fresh, subst, errors);
        let _ = infer(&args[2], env, locals, fresh, subst, errors);
        return None;
    }
    if args.len() != 5 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond -> :T then else) — 5 args; got {}",
                args.len()
            ),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    // Validate the `->` marker and parse the declared type.
    match &args[1] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: "expected `->` between cond and type".into(),
            });
            return None;
        }
    }
    let declared_ty = match &args[2] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::if".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: "expected type keyword after `->`".into(),
            });
            return None;
        }
    };
    // Condition must be :bool.
    let cond_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(c) = cond_ty {
        if unify(&c, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "cond".into(),
                expected: ":bool".into(),
                got: format_type(&apply_subst(&c, subst)),
            });
        }
    }
    // Each branch body checked against the declared `:T` independently.
    // Errors name the branch so the author sees where the divergence is.
    let then_ty = infer(&args[3], env, locals, fresh, subst, errors);
    if let Some(t) = then_ty {
        if unify(&t, &declared_ty, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "then-branch".into(),
                expected: format_type(&apply_subst(&declared_ty, subst)),
                got: format_type(&apply_subst(&t, subst)),
            });
        }
    }
    let else_ty = infer(&args[4], env, locals, fresh, subst, errors);
    if let Some(e) = else_ty {
        if unify(&e, &declared_ty, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "else-branch".into(),
                expected: format_type(&apply_subst(&declared_ty, subst)),
                got: format_type(&apply_subst(&e, subst)),
            });
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

/// `(:wat::core::cond -> :T arm1 arm2 ... (:else default))`.
///
/// Multi-way conditional; sibling of [`infer_if`]. Typed once at the
/// head via `-> :T`; every arm's body type-unifies with `:T`. Each
/// arm is a 2-element list `(test body)`; tests type-unify with
/// `:bool`. The final arm must be `(:else body)` — enforced here
/// and at runtime.
///
/// Per-arm error messages name which arm diverged (arm #N test /
/// arm #N body / :else body), matching `infer_if`'s branch-specific
/// diagnostics.
fn infer_cond(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() < 3 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::cond".into(),
            reason: format!(
                "expected (:wat::core::cond -> :T (:else body)) — at least 3 args; got {}",
                args.len()
            ),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "expected `->` at position 1".into(),
            });
            return None;
        }
    }
    let declared_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "expected type keyword at position 2 (after `->`)".into(),
            });
            return None;
        }
    };

    let arms = &args[2..];
    // Validate last arm is `:else`. Report once at the checker layer
    // so users get the diagnostic before the runtime sees it.
    let last = &arms[arms.len() - 1];
    let last_items = match last {
        WatAST::List(xs, _) if xs.len() == 2 => xs,
        WatAST::List(xs, _) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: format!(
                    "last arm must be (:else body); got {}-element list",
                    xs.len()
                ),
            });
            return None;
        }
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "last arm must be a list (:else body)".into(),
            });
            return None;
        }
    };
    let last_is_else = matches!(&last_items[0], WatAST::Keyword(k, _) if k == ":else");
    if !last_is_else {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::cond".into(),
            reason: "last arm must be (:else body) — cond requires an explicit default".into(),
        });
    }

    for (i, arm) in arms.iter().enumerate() {
        let items = match arm {
            WatAST::List(xs, _) if xs.len() == 2 => xs,
            WatAST::List(xs, _) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!(
                        "arm #{} must be (test body); got {}-element list",
                        i + 1,
                        xs.len()
                    ),
                });
                continue;
            }
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!(
                        "arm #{} must be a list (test body); got {:?}",
                        i + 1,
                        other
                    ),
                });
                continue;
            }
        };
        let is_last = i + 1 == arms.len();
        let is_else_arm =
            is_last && matches!(&items[0], WatAST::Keyword(k, _) if k == ":else");

        if !is_else_arm {
            // Test must unify with :bool.
            let test_ty = infer(&items[0], env, locals, fresh, subst, errors);
            if let Some(t) = test_ty {
                if unify(&t, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::core::cond".into(),
                        param: format!("arm #{} test", i + 1),
                        expected: ":bool".into(),
                        got: format_type(&apply_subst(&t, subst)),
                    });
                }
            }
        }
        // Body must unify with declared_ty.
        let body_ty = infer(&items[1], env, locals, fresh, subst, errors);
        if let Some(b) = body_ty {
            if unify(&b, &declared_ty, subst, env.types()).is_err() {
                let param = if is_else_arm {
                    ":else body".to_string()
                } else {
                    format!("arm #{} body", i + 1)
                };
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::cond".into(),
                    param,
                    expected: format_type(&apply_subst(&declared_ty, subst)),
                    got: format_type(&apply_subst(&b, subst)),
                });
            }
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

fn infer_let(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let bindings = match &args[0] {
        WatAST::List(items, _) => items,
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

/// `(:wat::core::try <result-expr>)` — the error-propagation form.
///
/// Type rules:
/// 1. Exactly one argument. Otherwise `ArityMismatch`.
/// 2. The innermost enclosing function/lambda must declare its return
///    type as `:Result<_, E>`. Otherwise `MalformedForm` — `try` has
///    nowhere to propagate to.
/// 3. The argument's type must unify with `:Result<T, E>` where `E` is
///    the enclosing function's `Err` variant. Mismatched `E` surfaces
///    as `TypeMismatch` (strict equality per the 2026-04-19 stance —
///    no auto-conversion, no From-trait analogue). Polymorphic error
///    handling is expressed via explicit enum-wrap at the boundary.
/// 4. On success, the form's type is `T` — the `Ok`-inner of the
///    argument's `Result`.
///
/// Runtime behavior (see `crate::runtime::eval_try`):
/// - `Ok(v)` → evaluates to `v`.
/// - `Err(e)` → raises `RuntimeError::TryPropagate(e)`; the innermost
///   `apply_function` packages it as the function's own `Err(e)`
///   return value.
fn infer_try(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::try".into(),
            expected: 1,
            got: args.len(),
        });
        // Still infer the arg(s) so any internal errors surface.
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }

    // The enclosing function's return type must exist and must itself
    // be `Result<_, E>`. Otherwise `try` has no propagation target.
    let enclosing = match fresh.enclosing_ret().cloned() {
        Some(r) => r,
        None => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::try".into(),
                reason: "used outside any function or lambda body; `try` requires an enclosing Result-returning scope to propagate into".into(),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };
    // Reduce so a typealias over Result<T,E> is recognized as
    // Result<T,E> here. (`:my::Res<T> = Result<T,String>` would
    // otherwise be rejected as "not a Result" at this match.)
    let enclosing_reduced = reduce(&enclosing, subst, env.types());
    let enclosing_err_ty = match &enclosing_reduced {
        TypeExpr::Parametric { head, args: type_args }
            if head == "Result" && type_args.len() == 2 =>
        {
            type_args[1].clone()
        }
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::try".into(),
                reason: format!(
                    "enclosing function returns {}; `try` requires the enclosing function to return :Result<T,E>",
                    format_type(&enclosing)
                ),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };

    // Argument must unify with Result<fresh_T, enclosing_err_ty>.
    // Building the expected type this way enforces both that the arg
    // is a Result and that its Err variant matches the enclosing
    // function's Err variant in one unification.
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors)?;
    let fresh_t = fresh.fresh();
    let expected = TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![fresh_t.clone(), enclosing_err_ty],
    };
    if unify(&arg_ty, &expected, subst, env.types()).is_err() {
        errors.push(CheckError::TypeMismatch {
            callee: ":wat::core::try".into(),
            param: "arg".into(),
            expected: format_type(&apply_subst(&expected, subst)),
            got: format_type(&apply_subst(&arg_ty, subst)),
        });
        return None;
    }

    // The try expression's type is T — the Ok-inner of the argument's
    // Result, now refined by unification with the enclosing function's
    // shape.
    Some(apply_subst(&fresh_t, subst))
}

/// Sequential let — same binding shapes as parallel `let`, but each
/// RHS is checked with the cumulatively extended locals so later
/// bindings may reference earlier ones.
fn infer_let_star(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let bindings = match &args[0] {
        WatAST::List(items, _) => items,
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

/// Type-check `(:wat::kernel::spawn <fn> arg1 arg2 ...)`.
/// Variadic in the args (one per function parameter) — rank-1 HM
/// can't express variadic schemes, so spawn is special-cased.
///
/// The first argument may be either of two shapes, mirroring the
/// runtime dispatch (see `eval_kernel_spawn`):
///
/// - A keyword-path literal → the function's declared scheme is
///   looked up in `CheckEnv` and instantiated.
/// - Any expression whose inferred type is `:fn(T1,T2,...)->R` → the
///   parameter types and return type come from the inferred Fn type
///   directly.
///
/// Either way, the remaining args are unified against the parameter
/// types, and the spawn's return is `:ProgramHandle<R>`.
fn infer_spawn(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
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
    // Resolve the first arg's signature — keyword path path or
    // infer-and-extract-Fn path.
    let (param_types, ret_type, callee_label) = match &args[0] {
        WatAST::Keyword(fn_path, _) => match env.get(fn_path) {
            Some(scheme) => {
                let (ps, r) = instantiate(&scheme.clone(), fresh);
                (ps, r, format!(":wat::kernel::spawn {}", fn_path))
            }
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
        },
        _ => {
            // Non-keyword: infer as a value, expect `:fn(...)->R`. Use
            // reduce so a typealias over an fn type still matches.
            let inferred = infer(&args[0], env, locals, fresh, subst, errors);
            let surface_ty = match &inferred {
                Some(t) => apply_subst(t, subst),
                None => {
                    return Some(TypeExpr::Parametric {
                        head: "wat::kernel::ProgramHandle".into(),
                        args: vec![fresh.fresh()],
                    });
                }
            };
            let fn_ty = reduce(&surface_ty, subst, env.types());
            match fn_ty {
                TypeExpr::Fn { args: ps, ret } => (ps, *ret, ":wat::kernel::spawn <lambda>".to_string()),
                _ => {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::kernel::spawn".into(),
                        param: "#1".into(),
                        expected: "function keyword path or fn(...) value".into(),
                        got: format_type(&surface_ty),
                    });
                    for arg in &args[1..] {
                        let _ = infer(arg, env, locals, fresh, subst, errors);
                    }
                    return Some(TypeExpr::Parametric {
                        head: "wat::kernel::ProgramHandle".into(),
                        args: vec![fresh.fresh()],
                    });
                }
            }
        }
    };
    let spawn_args = &args[1..];
    if spawn_args.len() != param_types.len() {
        errors.push(CheckError::ArityMismatch {
            callee: callee_label.clone(),
            expected: param_types.len(),
            got: spawn_args.len(),
        });
    }
    for (i, (arg, expected)) in spawn_args.iter().zip(&param_types).enumerate() {
        if let Some(arg_ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&arg_ty, expected, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: callee_label.clone(),
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
    fresh: &mut InferCtx,
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
        // Reduce to canonical structural form for the match; keep the
        // surface-name form for error display.
        let reduced = reduce(&ty, subst, env.types());
        match &reduced {
            // Tuple: return element at `index`.
            TypeExpr::Tuple(elements) => {
                if let Some(elem) = elements.get(index) {
                    return Some(apply_subst(elem, subst));
                } else {
                    errors.push(CheckError::TypeMismatch {
                        callee: op.into(),
                        param: "#1".into(),
                        expected: format!("tuple with ≥ {} element(s)", index + 1),
                        got: format_type(&apply_subst(&ty, subst)),
                    });
                    return Some(fresh.fresh());
                }
            }
            // Vec<T>: return Option<T> (arc 047 — empty/short is a
            // runtime fact, signature surfaces it honestly).
            TypeExpr::Parametric { head, args: targs } if head == "Vec" => {
                if let Some(inner) = targs.first() {
                    return Some(TypeExpr::Parametric {
                        head: "Option".into(),
                        args: vec![apply_subst(inner, subst)],
                    });
                } else {
                    return Some(fresh.fresh());
                }
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: op.into(),
                    param: "#1".into(),
                    expected: "tuple or Vec<T>".into(),
                    got: format_type(&apply_subst(&ty, subst)),
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
    fresh: &mut InferCtx,
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
        // Reduce for the shape match; keep the surface-name form for
        // the error display.
        let reduced = reduce(&ty, subst, env.types());
        let is_channel_handle = matches!(
            &reduced,
            TypeExpr::Parametric { head, .. }
                if head == "rust::crossbeam_channel::Sender"
                    || head == "rust::crossbeam_channel::Receiver"
        );
        if !is_channel_handle {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::kernel::drop".into(),
                param: "#1".into(),
                expected: "rust::crossbeam_channel::Sender<T> | rust::crossbeam_channel::Receiver<T>".into(),
                got: format_type(&apply_subst(&ty, subst)),
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
    fresh: &mut InferCtx,
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
                head: "rust::crossbeam_channel::Sender".into(),
                args: vec![t.clone()],
            },
            TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
                args: vec![t],
            },
        ]));
    }
    // Extract T from the type-keyword argument.
    let t_ty = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
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
                        WatAST::IntLit(_, _) => "int",
                        WatAST::FloatLit(_, _) => "float",
                        WatAST::BoolLit(_, _) => "bool",
                        WatAST::StringLit(_, _) => "string",
                        WatAST::Symbol(_, _) => "symbol",
                        WatAST::List(_, _) => "list",
                        WatAST::Keyword(_, _) => unreachable!(),
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
            if unify(&cap_ty, &i64_ty, subst, env.types()).is_err() {
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
            head: "rust::crossbeam_channel::Sender".into(),
            args: vec![t_ty.clone()],
        },
        TypeExpr::Parametric {
            head: "rust::crossbeam_channel::Receiver".into(),
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
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
) {
    let kv = match pair {
        WatAST::List(items, _) if items.len() == 2 => items,
        _ => return, // runtime parser surfaces the shape error
    };
    let binder = match &kv[0] {
        WatAST::List(inner, _) => inner,
        _ => return, // bare `(name rhs)` refused at runtime; check silently skips
    };
    let rhs = &kv[1];

    let is_typed_single = binder.len() == 2
        && matches!(&binder[0], WatAST::Symbol(_, _))
        && matches!(&binder[1], WatAST::Keyword(_, _));

    if is_typed_single {
        let name = match &binder[0] {
            WatAST::Symbol(ident, _) => ident.name.clone(),
            _ => return,
        };
        let declared = match &binder[1] {
            WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
                Ok(t) => t,
                Err(_) => return,
            },
            _ => return,
        };
        let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
        if let Some(rhs_ty) = rhs_ty {
            if unify(&rhs_ty, &declared, subst, env.types()).is_err() {
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
            WatAST::Symbol(ident, _) => names.push(ident.name.clone()),
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
        if unify(&rhs_ty, &tuple_ty, subst, env.types()).is_err() {
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

/// Type-check `(:wat::core::HashSet :T x1 x2 ...)`. First arg is a
/// type keyword; remaining args are elements, each unifying with T.
/// Explicit typing required (matches the vec/list / make-queue
/// resource-constructor discipline — shape never depends on context).
fn infer_hashset_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::HashSet".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "HashSet".into(),
            args: vec![fresh.fresh()],
        });
    }
    let t_ty = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::HashSet".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                fresh.fresh()
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::HashSet".into(),
                reason: "first argument must be a type keyword (e.g., :i64)".into(),
            });
            fresh.fresh()
        }
    };
    for (i, arg) in args[1..].iter().enumerate() {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &t_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashSet".into(),
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

/// Arc 050 — polymorphic comparison/equality inference.
///
/// For `:wat::core::=`, `<`, `>`, `<=`, `>=`. Same-type-for-non-
/// numeric, cross-numeric-promotion-for-(i64,f64) pairs. Always
/// returns `:bool`.
///
/// The runtime path (`eval_compare`, `values_equal` post-arc-050)
/// already handles the cross-numeric case; this checker branch
/// makes the runtime path reachable.
fn infer_polymorphic_compare(
    op: &str,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let bool_ty = TypeExpr::Path(":bool".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(bool_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let (Some(a), Some(b)) = (a_ty, b_ty) {
        let a_resolved = apply_subst(&a, subst);
        let b_resolved = apply_subst(&b, subst);
        // Numeric cross-type allowed: (i64, f64) and (f64, i64) accepted.
        if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
            return Some(bool_ty);
        }
        // Non-numeric: same-type required (preserves prior
        // ∀T. T → T → :bool semantics for strings, bools, etc.).
        if unify(&a_resolved, &b_resolved, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: format_type(&apply_subst(&a_resolved, subst)),
                got: format_type(&apply_subst(&b_resolved, subst)),
            });
        }
    }
    Some(bool_ty)
}

/// Arc 050 — polymorphic arithmetic inference.
///
/// For `:wat::core::+`, `-`, `*`, `/`. Both args must be numeric
/// (`:i64` or `:f64`). Result type is `:f64` if either is `:f64`,
/// else `:i64`. Mixed inputs promote at runtime (i64 cast to f64).
fn infer_polymorphic_arith(
    op: &str,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let i64_ty = TypeExpr::Path(":i64".into());
    let f64_ty = TypeExpr::Path(":f64".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(f64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let a_resolved = a_ty.as_ref().map(|t| apply_subst(t, subst));
    let b_resolved = b_ty.as_ref().map(|t| apply_subst(t, subst));

    // Push diagnostic if either arg is non-numeric.
    if let Some(t) = &a_resolved {
        if !is_numeric(t) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":i64 or :f64".into(),
                got: format_type(t),
            });
        }
    }
    if let Some(t) = &b_resolved {
        if !is_numeric(t) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":i64 or :f64".into(),
                got: format_type(t),
            });
        }
    }

    match (&a_resolved, &b_resolved) {
        (Some(a), Some(b)) if is_i64(a) && is_i64(b) => Some(i64_ty),
        (Some(a), Some(b)) if is_numeric(a) && is_numeric(b) => Some(f64_ty),
        // Either non-numeric or unknown — fall back to f64 so downstream
        // inference doesn't cascade more errors.
        _ => Some(f64_ty),
    }
}

/// Arc 050 — predicate. Recognizes `:i64` and `:f64` paths.
fn is_numeric(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Path(p) if p == ":i64" || p == ":f64")
}

/// Arc 050 — predicate. Recognizes `:i64` path specifically.
fn is_i64(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Path(p) if p == ":i64")
}

/// Arc 052 — predicate. Recognizes `:wat::holon::HolonAST` and
/// `:wat::holon::Vector` — the two algebra-tier value types accepted
/// by polymorphic cosine / dot / simhash.
fn is_holon_or_vector(t: &TypeExpr) -> bool {
    matches!(
        t,
        TypeExpr::Path(p)
            if p == ":wat::holon::HolonAST" || p == ":wat::holon::Vector"
    )
}

/// Arc 052 — polymorphic two-arg holon-algebra inference.
///
/// For `:wat::holon::cosine` and `:wat::holon::dot`. Both args must be
/// HolonAST or Vector; result type is `:f64`. Mixed inputs are
/// permitted (the runtime promotes the AST side by encoding at the
/// Vector side's d).
fn infer_polymorphic_holon_pair_to_f64(
    op: &str,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let f64_ty = TypeExpr::Path(":f64".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(f64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
            });
        }
    }
    if let Some(t) = &b_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
            });
        }
    }
    Some(f64_ty)
}

/// Arc 052 — polymorphic one-arg holon-algebra inference returning
/// `:i64`. For `:wat::holon::simhash` — accepts HolonAST or Vector.
fn infer_polymorphic_holon_to_i64(
    op: &str,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let i64_ty = TypeExpr::Path(":i64".into());
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 1,
            got: args.len(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(i64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
            });
        }
    }
    Some(i64_ty)
}

/// Type-check `(:wat::core::get container locator)`. Polymorphic over
/// HashMap and HashSet; dispatch by arg shape. Rank-1 HM can't
/// express the union at the SCHEME layer, so special-case: inspect
/// the first arg's type and produce the matching return type.
///   HashMap<K,V>, K → Option<V>
///   HashSet<T>,   T → Option<T>
fn infer_get(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::get".into(),
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
        // Reduce for the shape match — a user typealias over HashMap
        // / HashSet (e.g., `(typealias :my::Row :HashMap<String,i64>)`)
        // must be recognized by its structural root here.
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                let v = apply_subst(&ta[1], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
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
            // Arc 025: Vec support. `(get xs i)` with :i64 index
            // returns `:Option<T>`. Unify key with i64; container's
            // element type is the Option's T. 058-026 INSCRIPTION.
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&t, subst)],
                });
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
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
                    callee: ":wat::core::get".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | HashSet<T> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 020 — `(:wat::core::assoc container key value)`. Clojure
/// `assoc`: associate key with value in a HashMap, return new map.
/// For `HashMap<K,V>`: unifies key-ty with K, value-ty with V;
/// returns the input HashMap type. Matches `infer_get`'s dispatch-
/// on-container shape; extends to other containers if demand
/// surfaces.
fn infer_assoc(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 3 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::assoc".into(),
            expected: 3,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let value_ty = infer(&args[2], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                let v = apply_subst(&ta[1], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &v, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&v, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                        });
                    }
                }
                return Some(reduced);
            }
            // Arc 025: Vec support. `(assoc xs i v)` replaces xs[i]
            // with v; i must unify with :i64, v must unify with T.
            // Returns Vec<T>. Out-of-range i is a runtime error, not
            // a type error.
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::assoc".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![fresh.fresh(), fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::dissoc m k)`. Returns a NEW HashMap
/// without `k`; original unchanged. Missing key is no-op
/// (returns clone of input). Mirrors Clojure's dissoc.
///   ∀K, V. HashMap<K,V> × K → HashMap<K,V>
fn infer_dissoc(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::dissoc".into(),
            expected: 2,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::dissoc".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::dissoc".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![fresh.fresh(), fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::keys m)`. Materializes the map's keys
/// as a Vec (order unspecified — Rust's HashMap iteration order
/// depends on hash randomization; sort the result if you need
/// determinism).
///   ∀K, V. HashMap<K,V> → Vec<K>
fn infer_keys(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::keys".into(),
            expected: 1,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![k],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::keys".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::values m)`. Materializes the map's
/// values as a Vec (order unspecified — same caveat as `keys`).
///   ∀K, V. HashMap<K,V> → Vec<V>
fn infer_values(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::values".into(),
            expected: 1,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let v = apply_subst(&ta[1], subst);
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![v],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::values".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::empty? container)`. Polymorphic empty-check;
/// mirrors `length`'s polymorphism shape:
///   ∀T.   Vec<T>       → bool
///   ∀K,V. HashMap<K,V> → bool
///   ∀T.   HashSet<T>   → bool
fn infer_empty_q(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::empty?".into(),
            expected: 1,
            got: args.len(),
        });
        return Some(TypeExpr::Path(":bool".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::empty?".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashMap<K,V> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

/// Arc 025 — `(:wat::core::conj container value)`. Polymorphic
/// over Vec and HashSet; HashMap illegal (no key-value pairing —
/// use assoc).
///   ∀T. Vec<T>     × T -> Vec<T>
///   ∀T. HashSet<T> × T -> HashSet<T>
fn infer_conj(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::conj".into(),
            expected: 2,
            got: args.len(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let value_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::conj".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                        });
                    }
                }
                return Some(reduced);
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::conj".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::conj".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 035 — `(:wat::core::length container)`. Polymorphic size:
///   ∀T.   Vec<T>       -> i64    (elements)
///   ∀K,V. HashMap<K,V> -> i64    (entries)
///   ∀T.   HashSet<T>   -> i64    (elements)
/// Tuple is deliberately excluded — arity is structural and known
/// at type-check time.
fn infer_length(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::length".into(),
            expected: 1,
            got: args.len(),
        });
        return Some(TypeExpr::Path(":i64".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::length".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashMap<K,V> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Path(":i64".into()))
}

/// Arc 025 — `(:wat::core::contains? container key)`. Polymorphic
/// membership/key predicate:
///   ∀K,V. HashMap<K,V> × K -> bool    (has key)
///   ∀T.   HashSet<T>   × T -> bool    (has element)
///   ∀T.   Vec<T>       × i64 -> bool  (has valid index)
/// Retires `:wat::std::member?` — contains? covers it now.
fn infer_contains_q(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::contains?".into(),
            expected: 2,
            got: args.len(),
        });
        return Some(TypeExpr::Path(":bool".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                        });
                    }
                }
                // suppress unused-arg warnings in this arm
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::contains?".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | HashSet<T> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

/// Type-check `(:wat::core::HashMap :(K,V) k1 v1 k2 v2 ...)`. First arg
/// is a tuple-type keyword `:(K,V)` encoding both parameters; the
/// remaining args are alternating key/value pairs. Keys unify with K,
/// values with V. Explicit typing required (matches vec/list / make-queue
/// resource-constructor discipline).
fn infer_hashmap_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::HashMap".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let (k_ty, v_ty) = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(TypeExpr::Tuple(ts)) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
            Ok(other) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::HashMap".into(),
                    reason: format!(
                        "first argument must be a tuple type :(K,V); got {}",
                        format_type(&other)
                    ),
                });
                (fresh.fresh(), fresh.fresh())
            }
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::HashMap".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                });
                (fresh.fresh(), fresh.fresh())
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::HashMap".into(),
                reason: "first argument must be a tuple type keyword :(K,V)".into(),
            });
            (fresh.fresh(), fresh.fresh())
        }
    };
    let pairs = &args[1..];
    if !pairs.len().is_multiple_of(2) {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "arity after :(K,V) must be even (alternating key/value)".into(),
        });
    }
    for (i, chunk) in pairs.chunks(2).enumerate() {
        if let Some(k_arg_ty) = infer(&chunk[0], env, locals, fresh, subst, errors) {
            if unify(&k_arg_ty, &k_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashMap".into(),
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
            if unify(&v_arg_ty, &v_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashMap".into(),
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
    fresh: &mut InferCtx,
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

/// `(:wat::core::string::concat s1 s2 ... sn) -> :String`.
///
/// Variadic; each arg must unify with :String. Special-cased here
/// rather than registered as a polymorphic scheme because the type
/// checker has no first-class variadic-arity scheme today (same
/// rationale as `vec` / `tuple`). Empty arg list errors at the
/// runtime; the checker accepts arity 0 and returns `:String` so the
/// runtime owns the diagnostic — this mirrors how `tuple` behaves.
/// Arc 059 — `(:wat::core::concat v1 v2 ...)`. Variadic Vec
/// concatenation; ≥1 arg required (zero-arg ambiguous on T, same
/// reasoning as `:wat::core::vec`'s rejection of zero-arg).
///   ∀T. (Vec<T>)+ → Vec<T>
/// All args must unify on the same `Vec<T>` — no implicit coercion
/// (a `Vec<i64>` and `Vec<f64>` don't concat). Mirrors the
/// `string::concat` shape but with a fresh element type variable
/// instead of a fixed `:String`.
fn infer_concat(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::concat".into(),
            expected: 1,
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let elem_ty = fresh.fresh();
    let vec_ty = TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![elem_ty],
    };
    for arg in args {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &vec_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::concat".into(),
                    param: "arg".into(),
                    expected: format_type(&apply_subst(&vec_ty, subst)),
                    got: format_type(&apply_subst(&ty, subst)),
                });
            }
        }
    }
    Some(apply_subst(&vec_ty, subst))
}

fn infer_string_concat(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let string_ty = TypeExpr::Path(":String".into());
    for arg in args {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &string_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::string::concat".into(),
                    param: "arg".into(),
                    expected: ":String".into(),
                    got: format_type(&apply_subst(&ty, subst)),
                });
            }
        }
    }
    Some(string_ty)
}

fn infer_list_constructor(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
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
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
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
            if unify(&arg_ty, &elem_ty, subst, env.types()).is_err() {
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
    fresh: &mut InferCtx,
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
    // Push this lambda's declared return type onto the enclosing-ret
    // stack so `try` inside the body propagates to the lambda's
    // boundary (matches Rust's `?`-operator scoping — short-circuits
    // the innermost fn or closure, not the outer function).
    fresh.push_enclosing_ret(ret_type.clone());
    let body_ty = infer(body, env, &body_locals, fresh, subst, errors);
    fresh.pop_enclosing_ret();
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &ret_type, subst, env.types()).is_err() {
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
        WatAST::List(items, _) => items,
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
                WatAST::Keyword(k, _) => {
                    ret = Some(crate::types::parse_type_expr(k).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
            continue;
        }
        match item {
            WatAST::Symbol(s, _) if s.as_str() == "->" => saw_arrow = true,
            WatAST::List(pair, _) => {
                if pair.len() != 2 {
                    return Err(());
                }
                let name = match &pair[0] {
                    WatAST::Symbol(s, _) => s.name.clone(),
                    _ => return Err(()),
                };
                let ty = match &pair[1] {
                    WatAST::Keyword(k, _) => crate::types::parse_type_expr(k).map_err(|_| ())?,
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
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // `and` / `or` take any number of :bool args, return :bool.
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
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
fn unify(
    a: &TypeExpr,
    b: &TypeExpr,
    subst: &mut Subst,
    types: &TypeEnv,
) -> Result<(), UnifyError> {
    // Reduce both sides to canonical shape before the structural
    // match — follow Var bindings AND expand typealiases at each
    // level. The recursive unify-on-children calls reduce at their
    // levels; combined, every position in both type trees is seen
    // post-alias. `:MyCache<K,V>` and its expansion
    // `:rust::lru::LruCache<K,V>` unify structurally as a result.
    let a = reduce(&walk(a, subst), subst, types);
    let b = reduce(&walk(b, subst), subst, types);
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
                unify(x, y, subst, types)?;
            }
            Ok(())
        }
        (TypeExpr::Fn { args: a1, ret: r1 }, TypeExpr::Fn { args: a2, ret: r2 }) => {
            if a1.len() != a2.len() {
                return Err(UnifyError);
            }
            for (x, y) in a1.iter().zip(a2.iter()) {
                unify(x, y, subst, types)?;
            }
            unify(r1, r2, subst, types)
        }
        (TypeExpr::Tuple(e1), TypeExpr::Tuple(e2)) => {
            if e1.len() != e2.len() {
                return Err(UnifyError);
            }
            for (x, y) in e1.iter().zip(e2.iter()) {
                unify(x, y, subst, types)?;
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
    fresh: &mut InferCtx,
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
    fresh: &'a mut InferCtx,
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
        unify(a, b, self.subst, self.env.types()).is_ok()
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

/// Apply the substitution map deeply — rewrites every `Var(id)` in
/// `ty` to its bound target (transitively). **Does NOT expand
/// typealiases.** `:MyAlias<i64>` stays `:MyAlias<i64>`.
///
/// Use this for **error display** — it preserves the surface name
/// the user wrote, so `TypeMismatch` reads "expected
/// `:wat::std::stream::Stream<i64>`", not the tuple expansion.
///
/// For **structural matching** against the canonical form of a type,
/// call [`reduce`] instead.
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

/// Fully reduce a type to its **canonical structural form** — follow
/// every Var substitution AND expand every typealias, at every level
/// of the tree. This is the single normalization pass: any
/// shape-inspection site (matching on `TypeExpr::Tuple`,
/// `TypeExpr::Parametric { head, ... }`, `TypeExpr::Fn`, etc.) should
/// call this before the match, so aliases never hide structure from
/// the check.
///
/// Relationship to the other passes:
///
/// - [`apply_subst`] is "walk Vars, preserve alias names." Right for
///   error messages (the surface name is what the user wrote).
/// - [`crate::types::expand_alias`] is "peel aliases at one level,
///   leave Vars." Right internally during unify to establish the
///   root shape before unifying children.
/// - `reduce` is both, recursively. Right for every shape-direct
///   inspection where the alias is incidental and the structural
///   root is what matters.
///
/// `unify`'s prologue also calls `reduce` — both sides see canonical
/// shapes before the structural match below runs, and the recursive
/// unify-on-children calls reduce at each level.
fn reduce(ty: &TypeExpr, subst: &Subst, types: &TypeEnv) -> TypeExpr {
    let expanded = crate::types::expand_alias(ty, types);
    match expanded {
        TypeExpr::Var(id) => match subst.get(&id) {
            Some(inner) => reduce(inner, subst, types),
            None => TypeExpr::Var(id),
        },
        TypeExpr::Path(_) => expanded,
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head,
            args: args.iter().map(|a| reduce(a, subst, types)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| reduce(a, subst, types)).collect(),
            ret: Box::new(reduce(&ret, subst, types)),
        },
        TypeExpr::Tuple(elements) => TypeExpr::Tuple(
            elements.iter().map(|e| reduce(e, subst, types)).collect(),
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
fn instantiate(scheme: &TypeScheme, fresh: &mut InferCtx) -> (Vec<TypeExpr>, TypeExpr) {
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
    let u8_ty = || TypeExpr::Path(":u8".into());
    let f64_ty = || TypeExpr::Path(":f64".into());
    let bool_ty = || TypeExpr::Path(":bool".into());
    let holon_ty = || TypeExpr::Path(":wat::holon::HolonAST".into());
    let t_var = || TypeExpr::Path(":T".into());

    // :u8 range-checked cast from :i64. Arc 008 slice 1. Runtime
    // rejects out-of-range values (0..=255) with a MalformedForm.
    env.register(
        ":wat::core::u8".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: u8_ty(),
        },
    );

    // :wat::io::IOReader + :wat::io::IOWriter abstract IO substrate.
    // Arc 008 slice 2. Two opaque wat types; multiple concrete
    // backings (real stdio, StringIo). Byte-oriented primitives with
    // char-level conveniences.
    let string_ty = || TypeExpr::Path(":String".into());
    let unit_ty = || TypeExpr::Tuple(vec![]);
    let vec_u8_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![u8_ty()],
    };
    let opt_vec_u8_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![vec_u8_ty()],
    };
    let opt_string_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![string_ty()],
    };
    let ioreader_ty = || TypeExpr::Path(":wat::io::IOReader".into());
    let iowriter_ty = || TypeExpr::Path(":wat::io::IOWriter".into());

    // IOReader — construction + ops.
    env.register(
        ":wat::io::IOReader/from-bytes".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_u8_ty()],
            ret: ioreader_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/from-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: ioreader_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty(), i64_ty()],
            ret: opt_vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read-all".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read-line".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: opt_string_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/rewind".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: unit_ty(),
        },
    );

    // IOWriter — construction + ops + snapshot.
    env.register(
        ":wat::io::IOWriter/new".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: iowriter_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/to-bytes".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: opt_string_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), vec_u8_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write-all".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), vec_u8_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/print".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/println".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/writeln".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/flush".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: unit_ty(),
        },
    );

    // :wat::kernel::run-sandboxed — arc 007 slice 2a.
    // (src: :String, stdin: :Vec<String>, scope: :Option<String>)
    //   -> :wat::kernel::RunResult
    //
    // Runs wat source in a fresh frozen world with captured stdio.
    // Scope None -> InMemoryLoader (no disk); Some path -> ScopedLoader
    // rooted at path. Result carries stdout / stderr Vec<String> and
    // an Option<Failure> (currently always :None on the happy path;
    // slice 2b populates via catch_unwind).
    env.register(
        ":wat::kernel::run-sandboxed".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![string_ty()],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: TypeExpr::Path(":wat::kernel::RunResult".into()),
        },
    );

    // :wat::kernel::run-sandboxed-hermetic (string-entry) — retired
    // in arc 012 slice 3. The AST-entry sibling
    // (:wat::kernel::run-sandboxed-hermetic-ast) lives in wat stdlib
    // on top of fork-with-forms; callers with raw source can parse
    // at the Rust boundary or (future) via a :wat::core::parse
    // primitive when a wat-level caller demands one.

    // :wat::kernel::run-sandboxed-ast — arc 007 slice 3b. Same
    // semantics as run-sandboxed but takes already-parsed forms as a
    // Vec<wat::WatAST> instead of source text. Typical caller: the
    // expansion of :wat::test::deftest, or any code that has AST in
    // hand. See sandbox.rs for the implementation.
    env.register(
        ":wat::kernel::run-sandboxed-ast".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::WatAST".into())],
                },
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![string_ty()],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: TypeExpr::Path(":wat::kernel::RunResult".into()),
        },
    );

    // :wat::kernel::run-sandboxed-hermetic-ast — retired as a Rust
    // primitive in arc 012 slice 3. Shipped as wat stdlib in
    // wat/std/hermetic.wat on top of fork-with-forms + wait-child
    // + struct-new. The keyword path + signature + return type are
    // identical; only the implementation layer moved. See
    // docs/arc/2026/04/012-fork-and-pipes/ for the arc's record.

    // :wat::kernel::assertion-failed! — arc 007 slice 3. Raises via
    // panic_any(AssertionPayload) so run-sandboxed's catch_unwind can
    // downcast and populate Failure.actual / Failure.expected. Declared
    // return type is :() since wat has no `!` / never type; the body
    // never returns.
    env.register(
        ":wat::kernel::assertion-failed!".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: unit_ty(),
        },
    );

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
    // Arc 019 — f64 rounding primitive. `(round v digits) -> f64`
    // rounds `v` to `digits` decimal places using round-half-away-
    // from-zero. `digits=0` rounds to the nearest integer;
    // `digits=2` rounds to two decimals. Negative `digits` rounds
    // to tens / hundreds / etc. NaN and ±∞ pass through unchanged.
    env.register(
        ":wat::core::f64::round".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), i64_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 046 — strict-f64 max / min / abs / clamp. Lab arc 015
    // surfaced these as substrate gaps while porting indicator
    // vocab; lifting them here means every wat consumer reaches
    // for the same names rather than reinventing in userland.
    for op in &[":wat::core::f64::max", ":wat::core::f64::min"] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: f64_ty(),
            },
        );
    }
    env.register(
        ":wat::core::f64::abs".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::core::f64::clamp".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 047 — Vec aggregates and the `last` accessor return
    // Option to honestly signal empty/no-match. Same reasoning as
    // the polymorphism shift on first/second/third for Vec inputs.
    let opt = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![inner],
    };
    let vec_of = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![inner],
    };
    env.register(
        ":wat::core::last".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: opt(t_var()),
        },
    );
    env.register(
        ":wat::core::find-last-index".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(bool_ty()),
                },
            ],
            ret: opt(i64_ty()),
        },
    );
    env.register(
        ":wat::core::f64::max-of".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_of(f64_ty())],
            ret: opt(f64_ty()),
        },
    );
    env.register(
        ":wat::core::f64::min-of".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_of(f64_ty())],
            ret: opt(f64_ty()),
        },
    );

    // Scalar conversions — arc 014. :wat::core::<source>::to-<target>
    // between the four scalar tiers (i64, f64, bool, String).
    // Infallible ones return the target directly; fallible ones return
    // :Option<T>. No implicit coercion — every conversion is an
    // explicit named call at the call site.
    let opt_i64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![i64_ty()],
    };
    let opt_f64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![f64_ty()],
    };
    let opt_bool_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![bool_ty()],
    };
    env.register(
        ":wat::core::i64::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::i64::to-f64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::core::f64::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::f64::to-i64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: opt_i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-i64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-f64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_f64_ty(),
        },
    );
    env.register(
        ":wat::core::bool::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![bool_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-bool".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_bool_ty(),
        },
    );

    // String basics — :wat::core::string::*. Per-type ops, char-
    // oriented (length counts unicode scalars, not bytes). See
    // src/string_ops.rs for the handlers.
    for op in &[
        ":wat::core::string::contains?",
        ":wat::core::string::starts-with?",
        ":wat::core::string::ends-with?",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![string_ty(), string_ty()],
                ret: bool_ty(),
            },
        );
    }
    env.register(
        ":wat::core::string::length".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::trim".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::string::split".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![string_ty()],
            },
        },
    );
    env.register(
        ":wat::core::string::join".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: string_ty(),
        },
    );

    // Regex — :wat::core::regex::*. matches? is unanchored (pattern
    // match anywhere in haystack); wrap with ^...$ for full-string.
    env.register(
        ":wat::core::regex::matches?".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: bool_ty(),
        },
    );

    // Comparison / equality — arc 050. The polymorphic forms
    // (`:wat::core::=`, `<`, `>`, `<=`, `>=`) are special-cased in
    // `infer_list` so they accept mixed numeric pairs (i64+f64) and
    // promote at runtime. For non-numeric types they still require
    // both operands to be the same type, same as the prior
    // `∀T. T → T → :bool` shape. No scheme registration here — the
    // special-case branch handles inference end-to-end.

    // Typed strict comparison/equality — arc 050. Power-user opt-in
    // for callers who want the type-guard behavior. Reject mixed
    // input at the checker; runtime delegates to the same eval_eq /
    // eval_compare paths.
    for op in &[
        ":wat::core::i64::=",
        ":wat::core::i64::<",
        ":wat::core::i64::>",
        ":wat::core::i64::<=",
        ":wat::core::i64::>=",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty(), i64_ty()],
                ret: bool_ty(),
            },
        );
    }
    for op in &[
        ":wat::core::f64::=",
        ":wat::core::f64::<",
        ":wat::core::f64::>",
        ":wat::core::f64::<=",
        ":wat::core::f64::>=",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: bool_ty(),
            },
        );
    }
    // Polymorphic arithmetic — arc 050. Special-cased in `infer_list`
    // for the cross-numeric promotion rule (i64+f64→f64). No scheme
    // registration here — the special-case branch handles inference
    // end-to-end.

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
    // Atom — ∀T. T → :wat::holon::HolonAST. Accepts any payload type.
    env.register(
        ":wat::holon::Atom".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: holon_ty(),
        },
    );
    // atom-value — ∀T. :wat::holon::HolonAST → :T. Dual of Atom. The caller's
    // let-binding type ascription (or surrounding context) pins T; the
    // runtime dispatches on the holon's variant and errors when the
    // variant doesn't match the expected return type.
    env.register(
        ":wat::core::atom-value".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![holon_ty()],
            ret: t_var(),
        },
    );

    // to-watast — :wat::holon::HolonAST → :wat::WatAST. Story-2 escape
    // hatch per arc 057: structural inverse of Atom's quote-lowering.
    // Pair with :wat::eval-ast! when you want the value, not the
    // coordinate.
    env.register(
        ":wat::holon::to-watast".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Path(":wat::WatAST".into()),
        },
    );

    // The eval-family forms — per the 2026-04-20 INSCRIPTION adding
    // :Result<wat::holon::HolonAST, :wat::core::EvalError> as the uniform
    // return type. Every dynamic evaluation failure (verification,
    // parse, mutation-form refused, unknown function, type mismatch,
    // etc.) becomes an Err value in the Result rather than an
    // unwinding RuntimeError. `:wat::core::try` inside eval'd code
    // continues to propagate as before — the TryPropagate signal
    // passes through the dispatcher's wrap.
    //
    // Arg types keep the pre-inscription looseness (the structural
    // keywords and payload strings aren't type-validated in fine
    // detail) — the purpose of adding these schemes is to enforce
    // the return shape at every call site, not to tighten arg
    // checking. A future pass may narrow arg types as real misuse
    // surfaces.
    let eval_result_ty = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            holon_ty(),
            TypeExpr::Path(":wat::core::EvalError".into()),
        ],
    };
    let wat_ast_ty = || TypeExpr::Path(":wat::WatAST".into());
    let keyword_ty = || TypeExpr::Path(":wat::core::keyword".into());
    let string_ty = || TypeExpr::Path(":String".into());

    // Arc 028 slice 3 — eval family iface drop. Each form takes its
    // source/path directly as the first arg; no interface keyword.
    // eval-edn! narrowed to string-only (one source shape per form,
    // like load! / load-string!).
    env.register(
        ":wat::eval-ast!".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![wat_ast_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-edn!".into(),
        TypeScheme {
            type_params: vec![],
            // <source-string>
            params: vec![string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-file!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>
            params: vec![string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-digest!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>, :wat::verify::digest-<algo>, :wat::verify::<iface>, <hex>
            params: vec![string_ty(), keyword_ty(), keyword_ty(), string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-digest-string!".into(),
        TypeScheme {
            type_params: vec![],
            // <source>, :wat::verify::digest-<algo>, :wat::verify::<iface>, <hex>
            params: vec![string_ty(), keyword_ty(), keyword_ty(), string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-signed!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>, :wat::verify::signed-<algo>,
            // :wat::verify::<iface>, <sig>, :wat::verify::<iface>, <pubkey>
            params: vec![
                string_ty(),
                keyword_ty(),
                keyword_ty(),
                string_ty(),
                keyword_ty(),
                string_ty(),
            ],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-signed-string!".into(),
        TypeScheme {
            type_params: vec![],
            // <source>, :wat::verify::signed-<algo>,
            // :wat::verify::<iface>, <sig>, :wat::verify::<iface>, <pubkey>
            params: vec![
                string_ty(),
                keyword_ty(),
                keyword_ty(),
                string_ty(),
                keyword_ty(),
                string_ty(),
            ],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::holon::Bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: holon_ty(),
        },
    );
    // Bundle takes :wat::holon::Holons and returns
    // :Result<wat::holon::HolonAST, :wat::holon::CapacityExceeded>.
    // The Result wrap is the forcing function for the capacity guard:
    // authors are required by the type system to acknowledge the
    // failure case — either matching explicitly or propagating via
    // `:wat::core::try`. Under `:error` the Err arm fires with the
    // cost/budget struct; under `:panic` the process panics before
    // returning.
    env.register(
        ":wat::holon::Bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![holon_ty()],
            }],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    holon_ty(),
                    TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
                ],
            },
        },
    );
    env.register(
        ":wat::holon::Permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), i64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::Thermometer".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::Blend".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );

    // Cosine measurement — the retrieval scalar (FOUNDATION 1718 +
    // OPEN-QUESTIONS line 419). Algebra-substrate operation (input is
    // holons, not raw numbers).
    //   (:wat::holon::cosine      target ref) -> :f64
    //   (:wat::holon::presence?   target ref) -> :bool (cosine > noise-floor)
    //   (:wat::holon::coincident? a      b  ) -> :bool ((1 - cosine) < noise-floor)
    //     dual to presence? — same threshold, equivalence direction. Arc 023.
    //
    // Arc 052: cosine and dot are special-cased in `infer_list` to
    // accept HolonAST OR Vector inputs (polymorphic). No scheme
    // registration here for those two — their inference branches in
    // infer_list cover both AST-AST and Vector-Vector and mixed cases.
    // presence? and coincident? remain HolonAST-only and keep their
    // scheme registrations.
    env.register(
        ":wat::holon::presence?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::holon::coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: bool_ty(),
        },
    );
    // eval-coincident? family — arc 026. Each variant mirrors its
    // eval-*! parent's arg shape, applied per-side (2 sides per
    // variant). Return is uniform Result<bool, EvalError> — any
    // failure on either side arrives as Err<EvalError>.
    let eval_coincident_ret = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            bool_ty(),
            TypeExpr::Path(":wat::core::EvalError".into()),
        ],
    };
    // slice 1 — base (AST). Takes two WatAST args (quote-captured).
    env.register(
        ":wat::holon::eval-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![wat_ast_ty(), wat_ast_ty()],
            ret: eval_coincident_ret(),
        },
    );
    // Arc 028 slice 3 — eval-coincident family arities updated to
    // match new eval-*! shapes (iface keyword dropped).
    // EDN variant — 2 source strings.
    env.register(
        ":wat::holon::eval-edn-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: eval_coincident_ret(),
        },
    );
    // digest variant — 2 × (path, algo, payload-iface, hex) = 8 args.
    env.register(
        ":wat::holon::eval-digest-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // digest-string variant — same arity, inline sources.
    env.register(
        ":wat::holon::eval-digest-string-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // signed variant — 2 × (path, algo, sig-iface, sig, pk-iface, pk) = 12 args.
    env.register(
        ":wat::holon::eval-signed-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // signed-string variant — same arity, inline sources.
    env.register(
        ":wat::holon::eval-signed-string-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );

    // Config accessors — nullary, read committed startup values.
    // Arc 037 slice 6: :wat::config::dims and :wat::config::noise-floor
    // are compatibility shims that return DEFAULT_TIERS[0]-derived
    // defaults. Semantically stale under multi-d but kept for
    // backward compat until callers migrate to per-AST primitives.
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
    // (:wat::kernel::pipe) → :(wat::io::IOWriter, wat::io::IOReader).
    // Arc 012 slice 1b. Writer first (producer), reader second.
    env.register(
        ":wat::kernel::pipe".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Path(":wat::io::IOWriter".into()),
                TypeExpr::Path(":wat::io::IOReader".into()),
            ]),
        },
    );
    // (:wat::kernel::fork-with-forms forms) → :wat::kernel::ForkedChild.
    // Arc 012 slice 2. Forks the current wat process (COW-inheriting
    // the loaded substrate), runs the caller's forms as a fresh
    // :user::main in the child, returns the ForkedChild struct
    // holding the child's handle + stdio pipe ends.
    env.register(
        ":wat::kernel::fork-with-forms".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Path(":wat::WatAST".into())],
            }],
            ret: TypeExpr::Path(":wat::kernel::ForkedChild".into()),
        },
    );
    // (:wat::kernel::wait-child handle) → :i64. Blocks on waitpid;
    // returns the child's exit code (WEXITSTATUS on normal exit,
    // 128+signum on signal termination). Idempotent — repeated
    // calls on the same handle return the cached code.
    env.register(
        ":wat::kernel::wait-child".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::kernel::ChildHandle".into())],
            ret: TypeExpr::Path(":i64".into()),
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
    // (:wat::kernel::send sender value) — ∀T. Sender<T> × T -> :Option<()>.
    // `(Some ())` on a successful send; `:None` when the receiver has
    // been dropped. Symmetric with `recv`'s Option-return; disconnect
    // on either endpoint is a value, not an error. See FOUNDATION-
    // CHANGELOG (wat-rs slice; pending) and runtime's `eval_kernel_send`.
    env.register(
        ":wat::kernel::send".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "rust::crossbeam_channel::Sender".into(),
                    args: vec![t_var()],
                },
                t_var(),
            ],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Tuple(vec![])],
            },
        },
    );
    // (:wat::kernel::try-recv receiver) — ∀T. Receiver<T> -> :Option<T>.
    // Non-blocking; `:None` covers both empty and disconnected.
    env.register(
        ":wat::kernel::try-recv".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
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
                head: "rust::crossbeam_channel::Receiver".into(),
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
                    head: "rust::crossbeam_channel::Receiver".into(),
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
    //
    // Arc 052: polymorphic via `infer_list` special-case branch (see
    // cosine note above); no scheme registration here.
    // Arc 052: Vector as first-class wat-tier value.
    // `:wat::holon::encode` materializes a HolonAST into a Vector at
    // the ambient d. The encoding context (vm/scalar/registry) is
    // ambient on the SymbolTable, same as cosine/dot/simhash; user
    // surface is one-arg.
    env.register(
        ":wat::holon::encode".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Path(":wat::holon::Vector".into()),
        },
    );
    // Arc 053: Vector-tier algebra primitives. Operate on raw
    // materialized Vectors without round-tripping through HolonAST.
    // Used by Phase 4 learning code that holds emergent vectors.
    let vector_ty = || TypeExpr::Path(":wat::holon::Vector".into());
    env.register(
        ":wat::holon::vector-bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), vector_ty()],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![vector_ty()],
            }],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-blend".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), vector_ty(), f64_ty(), f64_ty()],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), i64_ty()],
            ret: vector_ty(),
        },
    );
    // Arc 053: OnlineSubspace native value + 10 core methods.
    let subspace_ty = || TypeExpr::Path(":wat::holon::OnlineSubspace".into());
    let vec_f64_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![f64_ty()],
    };
    env.register(
        ":wat::holon::OnlineSubspace/new".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty(), i64_ty()],
            ret: subspace_ty(),
        },
    );
    for unary_to_i64 in &[
        ":wat::holon::OnlineSubspace/dim",
        ":wat::holon::OnlineSubspace/k",
        ":wat::holon::OnlineSubspace/n",
    ] {
        env.register(
            unary_to_i64.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![subspace_ty()],
                ret: i64_ty(),
            },
        );
    }
    env.register(
        ":wat::holon::OnlineSubspace/threshold".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/eigenvalues".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty()],
            ret: vec_f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/update".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/residual".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );
    for unary_to_vec in &[
        ":wat::holon::OnlineSubspace/project",
        ":wat::holon::OnlineSubspace/reconstruct",
    ] {
        env.register(
            unary_to_vec.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![subspace_ty(), vector_ty()],
                ret: vec_f64_ty(),
            },
        );
    }

    // Arc 053: Reckoner native value + 8 core methods. Label is :i64;
    // Prediction is a wat tuple :(Vec<(i64,f64)>, Option<i64>, f64,
    // f64). ReckConfig is encoded in the constructor name (Discrete
    // vs Continuous).
    let reckoner_ty = || TypeExpr::Path(":wat::holon::Reckoner".into());
    let unit_ty = || TypeExpr::Tuple(vec![]);
    env.register(
        ":wat::holon::Reckoner/new-discrete".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                i64_ty(),
                i64_ty(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
                },
            ],
            ret: reckoner_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/new-continuous".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), i64_ty(), i64_ty(), f64_ty(), i64_ty()],
            ret: reckoner_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/observe".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), vector_ty(), i64_ty(), f64_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/predict".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), vector_ty()],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Tuple(vec![i64_ty(), f64_ty()])],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![i64_ty()],
                },
                f64_ty(),
                f64_ty(),
            ]),
        },
    );
    env.register(
        ":wat::holon::Reckoner/resolve".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), f64_ty(), bool_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/curve".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Tuple(vec![f64_ty(), f64_ty()])],
            },
        },
    );
    env.register(
        ":wat::holon::Reckoner/labels".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![i64_ty()],
            },
        },
    );
    env.register(
        ":wat::holon::Reckoner/dims".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: i64_ty(),
        },
    );

    // Arc 053: Engram native value + 4 read methods.
    let engram_ty = || TypeExpr::Path(":wat::holon::Engram".into());
    env.register(
        ":wat::holon::Engram/name".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/eigenvalue-signature".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: vec_f64_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/n".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/residual".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 053: EngramLibrary native value + 6 core methods.
    let library_ty = || TypeExpr::Path(":wat::holon::EngramLibrary".into());
    env.register(
        ":wat::holon::EngramLibrary/new".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: library_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/add".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), string_ty(), subspace_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/match-vec".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), vector_ty(), i64_ty(), i64_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Tuple(vec![string_ty(), f64_ty()])],
            },
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/len".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/contains".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), string_ty()],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/names".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![string_ty()],
            },
        },
    );
    // Arc 051: SimHash — direction-space lattice position. Charikar's
    // hyperplane SimHash via the canonical Atom(0)..Atom(63) basis.
    // Maps an input holon to a 64-bit i64 key; cosine-similar inputs
    // share the same key (or near-same in hamming distance). Used as
    // the key-derivation function for bidirectional engram caches and
    // any content-addressed retrieval over the holon algebra.
    //
    // Arc 052: polymorphic via `infer_list` special-case branch —
    // accepts HolonAST or Vector input. No scheme registration here.
    // Arc 037 slice 4: HolonAST → immediate surface arity. The
    // natural introspection primitive for user dim-router bodies
    // ((:wat::config::set-dim-router! <fn>) where the fn signature
    // is `:fn(:wat::holon::HolonAST) -> :Option<i64>`). Returns the
    // top-level cardinality: 1 for Atom/Permute/Thermometer,
    // 2 for Bind/Blend, children.len() for Bundle.
    env.register(
        ":wat::holon::statement-length".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: i64_ty(),
        },
    );

    // IO primitives — see `:wat::io::IOReader/*` + `:wat::io::IOWriter/*`
    // registered above. Arc 008 retired the earlier `:wat::io::write`
    // and `:wat::io::read-line` primitives (which dispatched on
    // `Value::io__Stdin/Stdout/Stderr` directly) in favour of the
    // abstract IOReader/IOWriter surface.

    // Stdlib math — single-method Rust calls per FOUNDATION-CHANGELOG
    // 2026-04-18. All unary :f64 -> :f64 except pi which is :() -> :f64.
    // Packaged here so Log / Circular expansions get proper checking.
    for name in ["ln", "log", "exp", "sin", "cos", "sqrt"] {
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

    // Stat reductions over Vec<f64> — population variance/stddev
    // (matches numpy default ddof=0); all return :Option<f64> with
    // None on empty input (matches f64::min-of/max-of convention).
    let opt_f64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![f64_ty()],
    };
    let vec_f64_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![f64_ty()],
    };
    for name in ["mean", "variance", "stddev"] {
        env.register(
            format!(":wat::std::stat::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![vec_f64_ty()],
                ret: opt_f64_ty(),
            },
        );
    }

    // Arc 056 — :wat::time::* surface. Sibling of :wat::io::* at the
    // same nesting depth (world-observing primitives, not pure
    // stdlib). Single Instant value type backs all 9 primitives;
    // duration measurement is two `now` calls + integer-accessor
    // subtract (no separate Duration type).
    let instant_ty = || TypeExpr::Path(":wat::time::Instant".into());
    let string_ty = || TypeExpr::Path(":String".into());
    let opt_instant_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![instant_ty()],
    };
    env.register(
        ":wat::time::now".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: instant_ty(),
        },
    );
    for name in ["at", "at-millis", "at-nanos"] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty()],
                ret: instant_ty(),
            },
        );
    }
    env.register(
        ":wat::time::from-iso8601".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_instant_ty(),
        },
    );
    env.register(
        ":wat::time::to-iso8601".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![instant_ty(), i64_ty()],
            ret: string_ty(),
        },
    );
    for name in ["epoch-seconds", "epoch-millis", "epoch-nanos"] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![instant_ty()],
                ret: i64_ty(),
            },
        );
    }
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
    // :wat::core::length scheme retired; polymorphic under
    // `infer_length` (arc 035). Dispatched in `infer_list`.
    // :wat::core::empty? scheme retired (arc 058); polymorphic under
    // `infer_empty_q`. Same shape as `length` — Vec<T> | HashMap<K,V>
    // | HashSet<T> → bool.
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
    // Arc 056 — sort-by with user-supplied less-than predicate.
    // `(sort-by xs less?) -> Vec<T>` where `less? : :fn(T,T) -> :bool`.
    // The user owns asc vs desc via the predicate; key-extraction is
    // the predicate composing inner accessors. Common Lisp tradition.
    env.register(
        ":wat::core::sort-by".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var(), t_var()],
                    ret: Box::new(TypeExpr::Path(":bool".into())),
                },
            ],
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
    // get, assoc, conj, and contains? are all polymorphic over
    // container type — dispatched by the infer_* arms above. No
    // narrow schemes registered here.
    // :wat::std::member? RETIRED in arc 025. Use `:wat::core::contains?`
    // instead — now polymorphic over HashMap / HashSet / Vec.
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
    // :wat::core::conj — polymorphic add-to-growing-collection.
    //   ∀T. Vec<T>     × T -> Vec<T>
    //   ∀T. HashSet<T> × T -> HashSet<T>
    // Illegal on HashMap (use assoc instead — HashMap needs key+value
    // pairing). Dispatched by `infer_conj` at check.rs arm above.
    //
    // No narrow scheme registered; handled entirely by infer_conj.
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
    use crate::macros::{
        expand_all, register_defmacros, register_stdlib_defmacros, MacroRegistry,
    };
    use crate::parser::parse_all;
    use crate::runtime::{
        register_defines, register_stdlib_defines, register_struct_methods, SymbolTable,
    };
    use crate::types::{parse_type_expr, register_stdlib_types, TypeEnv};
    use std::sync::OnceLock;

    /// The stdlib is always part of the language. Test harnesses
    /// preload it once per process via `OnceLock`, clone the resulting
    /// state per test. This mirrors `startup_from_source`'s stdlib
    /// passes without running user-source phases, so every check()
    /// call sees `:wat::std::*` names, macros, and typealiases.
    fn stdlib_loaded() -> &'static (SymbolTable, MacroRegistry, TypeEnv) {
        static LOADED: OnceLock<(SymbolTable, MacroRegistry, TypeEnv)> = OnceLock::new();
        LOADED.get_or_init(|| {
            let stdlib = crate::stdlib::stdlib_forms().expect("stdlib parses");
            let mut macros = MacroRegistry::new();
            let stdlib_post_macros =
                register_stdlib_defmacros(stdlib, &mut macros).expect("stdlib defmacros");
            let expanded_stdlib =
                expand_all(stdlib_post_macros, &mut macros).expect("stdlib macro expansion");
            let mut types = TypeEnv::with_builtins();
            let stdlib_post_types =
                register_stdlib_types(expanded_stdlib, &mut types).expect("stdlib types");
            let mut symbols = SymbolTable::new();
            let _ = register_stdlib_defines(stdlib_post_types, &mut symbols)
                .expect("stdlib defines");
            register_struct_methods(&types, &mut symbols)
                .expect("built-in struct methods");
            (symbols, macros, types)
        })
    }

    fn check(src: &str) -> Result<(), CheckErrors> {
        let (stdlib_sym, stdlib_macros, types) = stdlib_loaded();
        let forms = parse_all(src).expect("parse ok");
        let mut macros = stdlib_macros.clone();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let expanded = expand_all(rest, &mut macros).expect("expand");
        let mut sym = stdlib_sym.clone();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        check_program(&rest, &sym, types)
    }

    // ─── Arity checking ─────────────────────────────────────────────────

    #[test]
    fn correct_arity_passes() {
        assert!(check("(:wat::core::i64::+ 1 2)").is_ok());
        assert!(check("(:wat::core::not true)").is_ok());
        assert!(check("(:wat::holon::Bind (:wat::holon::Atom 1) (:wat::holon::Atom 2))").is_ok());
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
        let err = check("(:wat::holon::Bind 42 (:wat::holon::Atom 1))").unwrap_err();
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
        // Bundle takes :wat::holon::Holons. A list of (Atom ...) calls
        // returns :wat::holon::Holons, so Bundle(list(Atoms...)) type-checks.
        assert!(check(
            r#"(:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST
                 (:wat::holon::Atom 1)
                 (:wat::holon::Atom 2)))"#
        )
        .is_ok());
    }

    #[test]
    fn bundle_of_list_of_ints_rejected() {
        // Bundle wants :wat::holon::Holons, but this is :Vec<i64>.
        let err = check(r#"(:wat::holon::Bundle (:wat::core::vec :i64 1 2 3))"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // LocalCache / :rust::lru::LruCache check tests retired in
    // arc 013 slice 4b — the wat-lru crate owns that surface now.
    // Equivalent check coverage lives in
    // crates/wat-lru/tests/wat_lru_tests.rs, exercised end-to-end
    // via wat::Harness::from_source_with_deps with the dep wiring.

    // Wrong-key-type rejection was enforced by the hand-written lru
    // shim's scheme via unification of call-site K with the cache's
    // declared K. The macro-regenerated shim's Rust signature uses
    // `Value` (not K) for the key arg — the scheme sees Value and
    // unifies trivially. Lands when the macro gets a per-arg type
    // hint (e.g. `#[wat_param = "K"]`). Tracked informally; not
    // blocking lru regeneration correctness because runtime
    // canonicalization still enforces primitive-key at dispatch time.

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
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_ok());
    }

    #[test]
    fn unify_distinct_paths_fails() {
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":f64".into()),
            &mut s,
            &TypeEnv::with_builtins(),
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
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_ok());
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":U".into()),
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_err());
    }

    #[test]
    fn unify_fresh_var_binds_to_concrete() {
        let mut s = Subst::new();
        let var = TypeExpr::Var(0);
        let concrete = TypeExpr::Path(":i64".into());
        unify(&var, &concrete, &mut s, &TypeEnv::with_builtins()).expect("unify");
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
        assert!(unify(&vec_int, &option_int, &mut s, &TypeEnv::with_builtins()).is_err());
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
        assert!(unify(&f1, &f2, &mut s, &TypeEnv::with_builtins()).is_ok());
    }

    #[test]
    fn occurs_check_rejects_cycle() {
        let mut s = Subst::new();
        // α = List<α>  — would produce an infinite type.
        let cyclic = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Var(0)],
        };
        assert!(unify(&TypeExpr::Var(0), &cyclic, &mut s, &TypeEnv::with_builtins()).is_err());
    }

    // ─── Parse + unify round-trip ───────────────────────────────────────

    #[test]
    fn type_expr_parse_and_unify() {
        let mut s = Subst::new();
        let a = parse_type_expr(":wat::holon::HolonAST").unwrap();
        let b = parse_type_expr(":wat::holon::HolonAST").unwrap();
        assert!(unify(&a, &b, &mut s, &TypeEnv::with_builtins()).is_ok());
    }
}
