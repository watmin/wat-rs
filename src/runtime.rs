//! Runtime — AST walker for `define` / `lambda` / `let` / `if` +
//! a small set of `:wat::core::*` built-in primitives + algebra-core
//! UpperCall construction.
//!
//! This is the first slice where a multi-form wat program runs
//! end-to-end. Not yet: kernel primitives (queue/spawn/select),
//! stdio handles, `:user/main`, or the measurements tier (cosine/dot).
//! Those live in later slices.
//!
//! # Value surface
//!
//! [`Value`] covers what a runtime expression can evaluate to:
//! `Bool`, `Int`, `Float`, `String`, `Keyword`, `Holon`, `Function`,
//! `Unit`, and `List` for the small set of list-shaped runtime values
//! (currently only used as return values from explicit `:wat::core::vec`
//! calls). No `Null`. No `Any`.
//!
//! # Environment model
//!
//! - [`Environment`] is a lexical-scope chain via `Arc`. Each `let` /
//!   function application creates a child env; lookups walk outward.
//! - [`SymbolTable`] holds keyword-path ↦ `Arc<Function>` entries
//!   registered by `:wat::core::define`. Functions are looked up directly
//!   by their full path.
//!
//! # Functions
//!
//! `define` registers at call to [`register_defines`]; the body is
//! stored as an AST and evaluated on each invocation. `lambda` at
//! evaluation time captures the enclosing [`Environment`] and produces
//! a `Value::Function` that can be passed, stored, and invoked.
//!
//! # Types
//!
//! Parameter type annotations are PARSED but IGNORED in this slice.
//! The type checker lands in task #137 and will walk the same AST.

use crate::ast::WatAST;
use holon::HolonAST;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Runtime value.
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Arc<String>),
    /// Keyword literal — leading ':' included.
    Keyword(Arc<String>),
    /// A composed HolonAST (the wat algebra's AST tier, carried at runtime).
    Holon(Arc<HolonAST>),
    /// A callable — either a `define`-registered function or a `lambda`
    /// closure.
    Function(Arc<Function>),
    /// A list of values — used by `:wat::core::vec`.
    List(Arc<Vec<Value>>),
    /// `:()` — unit. Returned by expressions with no meaningful value
    /// (not used widely in this slice).
    Unit,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "i64",
            Value::Float(_) => "f64",
            Value::String(_) => "String",
            Value::Keyword(_) => "Keyword",
            Value::Holon(_) => "Holon",
            Value::Function(_) => "Function",
            Value::List(_) => "Vec",
            Value::Unit => "()",
        }
    }
}

/// A callable. `define`-registered functions have `name = Some(path)`
/// and `closed_env = None` (they resolve symbols via the global
/// [`SymbolTable`] at call time). `lambda` values have `name = None`
/// and carry their `closed_env` from the creation site.
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    /// Declared type-parameter list from the function name keyword
    /// (e.g., `<T,U>` on `:my::ns::foo<T,U>`). Empty for monomorphic
    /// functions. Names appearing in `param_types` / `ret_type` that
    /// match an entry here are treated as type variables at check
    /// time.
    pub type_params: Vec<String>,
    /// Declared parameter types, parallel to `params`. Populated from
    /// the `(:wat::core::define (sig ...) body)` signature by
    /// `parse_define_form`. Used by the type checker for call-site
    /// unification and body-vs-signature checks. Empty only for
    /// lambda values (type-untracked).
    pub param_types: Vec<crate::types::TypeExpr>,
    /// Declared return type. `:()` (unit) if the signature omitted a
    /// return type. For lambdas, `:()` — the checker treats lambda
    /// values as opaque function values in slice 7b.
    pub ret_type: crate::types::TypeExpr,
    pub body: Arc<WatAST>,
    pub closed_env: Option<Environment>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("params", &self.params)
            .field(
                "closed_env",
                &if self.closed_env.is_some() { "<env>" } else { "<none>" },
            )
            .finish()
    }
}

/// Lexical-scope chain.
#[derive(Clone)]
pub struct Environment {
    inner: Arc<EnvCell>,
}

struct EnvCell {
    bindings: HashMap<String, Value>,
    parent: Option<Environment>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            inner: Arc::new(EnvCell {
                bindings: HashMap::new(),
                parent: None,
            }),
        }
    }

    pub fn child(&self) -> EnvBuilder {
        EnvBuilder {
            bindings: HashMap::new(),
            parent: Some(self.clone()),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.inner.bindings.get(name) {
            return Some(v.clone());
        }
        self.inner.parent.as_ref().and_then(|p| p.lookup(name))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder that accumulates bindings, then freezes into an [`Environment`].
pub struct EnvBuilder {
    bindings: HashMap<String, Value>,
    parent: Option<Environment>,
}

impl EnvBuilder {
    pub fn bind(mut self, name: impl Into<String>, value: Value) -> Self {
        self.bindings.insert(name.into(), value);
        self
    }

    pub fn build(self) -> Environment {
        Environment {
            inner: Arc::new(EnvCell {
                bindings: self.bindings,
                parent: self.parent,
            }),
        }
    }
}

/// Keyword-path ↦ Function registry.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub functions: HashMap<String, Arc<Function>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, path: &str) -> Option<&Arc<Function>> {
        self.functions.get(path)
    }
}

/// Runtime error.
#[derive(Debug)]
pub enum RuntimeError {
    UnboundSymbol(String),
    UnknownFunction(String),
    NotCallable { got: &'static str },
    TypeMismatch {
        op: String,
        expected: &'static str,
        got: &'static str,
    },
    ArityMismatch {
        op: String,
        expected: usize,
        got: usize,
    },
    BadCondition { got: &'static str },
    MalformedForm { head: String, reason: String },
    ParamShadowsBuiltin(String),
    DivisionByZero,
    DuplicateDefine(String),
    ReservedPrefix(String),
    /// `:wat::core::define` / `:wat::core::lambda` found in expression
    /// position at runtime. Define is a top-level registration form;
    /// lambda is fine in expression position. A caught-in-eval define
    /// means the caller confused the two phases.
    DefineInExpressionPosition,
    /// A constrained `eval` (`eval_in_frozen`) found a mutation-inducing
    /// form inside the AST it was asked to evaluate. Per FOUNDATION
    /// (§ constrained eval, line 663): "If the submitted AST contains a
    /// `define`, `defmacro`, `struct`, `enum`, `newtype`, `typealias`,
    /// or `load` form — eval refuses. This is not a mode; it is an
    /// invariant." Also covers `set-*!` config setters.
    EvalForbidsMutationForm { head: String },
    /// `:user::main` was not registered at startup. FOUNDATION requires
    /// exactly one `:user::main` declaration; zero halts.
    UserMainMissing,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UnboundSymbol(s) => write!(f, "unbound symbol: {}", s),
            RuntimeError::UnknownFunction(p) => write!(f, "unknown function: {}", p),
            RuntimeError::NotCallable { got } => {
                write!(f, "not callable: expected Function, got {}", got)
            }
            RuntimeError::TypeMismatch { op, expected, got } => {
                write!(f, "{}: expected {}, got {}", op, expected, got)
            }
            RuntimeError::ArityMismatch { op, expected, got } => {
                write!(f, "{}: expected {} arguments, got {}", op, expected, got)
            }
            RuntimeError::BadCondition { got } => {
                write!(f, "if / when condition must be :bool; got {}", got)
            }
            RuntimeError::MalformedForm { head, reason } => {
                write!(f, "malformed {} form: {}", head, reason)
            }
            RuntimeError::ParamShadowsBuiltin(s) => {
                write!(f, "parameter name {} shadows a :wat::core builtin; pick another name", s)
            }
            RuntimeError::DivisionByZero => write!(f, "division by zero"),
            RuntimeError::DuplicateDefine(p) => {
                write!(f, "duplicate define: {} already registered", p)
            }
            RuntimeError::ReservedPrefix(p) => write!(
                f,
                "cannot define {} — reserved prefix ({}); user defines must use their own prefix",
                p,
                crate::resolve::reserved_prefix_list()
            ),
            RuntimeError::DefineInExpressionPosition => write!(
                f,
                ":wat::core::define is a top-level registration form, not an expression"
            ),
            RuntimeError::EvalForbidsMutationForm { head } => write!(
                f,
                "constrained eval refuses mutation form {}; eval evaluates against the frozen symbol table and cannot register / replace / load definitions",
                head
            ),
            RuntimeError::UserMainMissing => write!(
                f,
                ":user::main not defined — a wat program needs an entry point"
            ),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Walk `forms`, register every `(:wat::core::define ...)` into `sym`,
/// and return the remaining (non-define) forms in order. Dupe
/// registration halts with [`RuntimeError::DuplicateDefine`].
pub fn register_defines(
    forms: Vec<WatAST>,
    sym: &mut SymbolTable,
) -> Result<Vec<WatAST>, RuntimeError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_define_form(&form) {
            let (path, func) = parse_define_form(form)?;
            if crate::resolve::is_reserved_prefix(&path) {
                return Err(RuntimeError::ReservedPrefix(path));
            }
            if sym.functions.contains_key(&path) {
                return Err(RuntimeError::DuplicateDefine(path));
            }
            sym.functions.insert(path, func);
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

fn is_define_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items)
            if matches!(items.first(), Some(WatAST::Keyword(k)) if k == ":wat::core::define")
    )
}

/// Parsed pieces of a define signature.
struct ParsedDefineSignature {
    canonical_name: String,
    type_params: Vec<String>,
    params: Vec<String>,
    param_types: Vec<crate::types::TypeExpr>,
    ret_type: crate::types::TypeExpr,
}

/// Parse `(:wat::core::define (:name::path<T,U> (p1 :T1) ... -> :R) body)`
/// into `(path, Arc<Function>)`. Captures the declared name (with
/// type-parameter list stripped from the keyword), parameter names
/// and types, and return type so the type checker can run real
/// signature checks.
fn parse_define_form(form: WatAST) -> Result<(String, Arc<Function>), RuntimeError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => return Err(RuntimeError::MalformedForm {
            head: ":wat::core::define".into(),
            reason: "expected list".into(),
        }),
    };
    if items.len() != 3 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::define".into(),
            reason: format!(
                "expected (:wat::core::define signature body); got {} elements",
                items.len()
            ),
        });
    }
    let mut iter = items.into_iter();
    let _define_kw = iter.next(); // already matched
    let signature = iter.next().expect("length checked above");
    let body = iter.next().expect("length checked above");

    let ParsedDefineSignature {
        canonical_name,
        type_params,
        params,
        param_types,
        ret_type,
    } = parse_define_signature(signature)?;
    Ok((
        canonical_name.clone(),
        Arc::new(Function {
            name: Some(canonical_name),
            params,
            type_params,
            param_types,
            ret_type,
            body: Arc::new(body),
            closed_env: None,
        }),
    ))
}

/// Signature is a List: `(:name::path<T,U> (p1 :T1) ... -> :R)`.
/// Extracts:
/// - canonical_name (the keyword path with any `<T,U>` stripped, re-
///   prefixed with ':' — matches the form used for symbol-table keys)
/// - type_params (names from the `<...>` suffix, or empty)
/// - params (parameter names)
/// - param_types (parallel type expressions parsed via
///   [`crate::types::parse_type_expr`])
/// - ret_type (parsed type after `->`; defaults to `:()` if omitted)
fn parse_define_signature(sig: WatAST) -> Result<ParsedDefineSignature, RuntimeError> {
    let items = match sig {
        WatAST::List(items) => items,
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::define".into(),
                reason: "signature must be a list".into(),
            })
        }
    };
    let mut iter = items.into_iter();
    let name_kw = match iter.next() {
        Some(WatAST::Keyword(k)) => k,
        Some(other) => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::define".into(),
                reason: format!(
                    "function name must be a keyword-path; got {}",
                    ast_variant_name(&other)
                ),
            });
        }
        None => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::define".into(),
                reason: "signature is empty".into(),
            });
        }
    };

    let (canonical_name, type_params) = split_name_and_type_params(&name_kw)?;

    let mut params = Vec::new();
    let mut param_types = Vec::new();
    let mut ret_type: Option<crate::types::TypeExpr> = None;
    let mut saw_arrow = false;
    for item in iter {
        if saw_arrow {
            // Only one form may follow `->` — the return type keyword.
            if ret_type.is_some() {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::define".into(),
                    reason: "signature has more than one return type after '->'".into(),
                });
            }
            match item {
                WatAST::Keyword(k) => {
                    ret_type = Some(parse_type_keyword(&k)?);
                }
                other => {
                    return Err(RuntimeError::MalformedForm {
                        head: ":wat::core::define".into(),
                        reason: format!(
                            "return type after '->' must be a type keyword; got {}",
                            ast_variant_name(&other)
                        ),
                    });
                }
            }
            continue;
        }
        match item {
            WatAST::Symbol(ref s) if s.as_str() == "->" => {
                saw_arrow = true;
            }
            WatAST::List(pair) => {
                let (pname, ptype) = parse_param_pair(pair)?;
                params.push(pname);
                param_types.push(ptype);
            }
            other => {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::define".into(),
                    reason: format!(
                        "unexpected signature element: {}",
                        ast_variant_name(&other)
                    ),
                });
            }
        }
    }

    Ok(ParsedDefineSignature {
        canonical_name,
        type_params,
        params,
        param_types,
        ret_type: ret_type.unwrap_or_else(|| crate::types::TypeExpr::Tuple(Vec::new())),
    })
}

/// `(p1 :T1)` → (`"p1"`, `TypeExpr`). Refuses malformed shapes.
fn parse_param_pair(
    pair: Vec<WatAST>,
) -> Result<(String, crate::types::TypeExpr), RuntimeError> {
    if pair.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::define".into(),
            reason: format!(
                "parameter must be (name :Type); got {}-element list",
                pair.len()
            ),
        });
    }
    let mut it = pair.into_iter();
    let name = match it.next() {
        Some(WatAST::Symbol(ident)) => ident.name,
        Some(other) => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::define".into(),
                reason: format!(
                    "parameter name must be a bare symbol; got {}",
                    ast_variant_name(&other)
                ),
            });
        }
        None => unreachable!("length checked above"),
    };
    let type_kw = match it.next() {
        Some(WatAST::Keyword(k)) => k,
        Some(other) => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::define".into(),
                reason: format!(
                    "parameter type must be a type keyword; got {}",
                    ast_variant_name(&other)
                ),
            });
        }
        None => unreachable!("length checked above"),
    };
    let ty = parse_type_keyword(&type_kw)?;
    Ok((name, ty))
}

fn parse_type_keyword(kw: &str) -> Result<crate::types::TypeExpr, RuntimeError> {
    crate::types::parse_type_expr(kw).map_err(|e| RuntimeError::MalformedForm {
        head: ":wat::core::define".into(),
        reason: e.to_string(),
    })
}

/// Split a keyword like `:ns/foo<T,U>` into (`":ns/foo"`, `vec!["T","U"]`).
/// A keyword with no `<` returns `(kw.to_string(), vec![])`.
fn split_name_and_type_params(kw: &str) -> Result<(String, Vec<String>), RuntimeError> {
    let lt_index = match kw.find('<') {
        Some(i) => i,
        None => return Ok((kw.to_string(), Vec::new())),
    };
    if !kw.ends_with('>') {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::define".into(),
            reason: format!("name keyword {:?} opens '<' but does not close '>'", kw),
        });
    }
    let head = kw[..lt_index].to_string();
    let inside = &kw[lt_index + 1..kw.len() - 1];
    let params: Vec<String> = inside
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok((head, params))
}

/// Evaluate a single form in the given scope.
pub fn eval(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    match ast {
        WatAST::IntLit(n) => Ok(Value::Int(*n)),
        WatAST::FloatLit(x) => Ok(Value::Float(*x)),
        WatAST::BoolLit(b) => Ok(Value::Bool(*b)),
        WatAST::StringLit(s) => Ok(Value::String(Arc::new(s.clone()))),
        WatAST::Keyword(k) => Ok(Value::Keyword(Arc::new(k.clone()))),
        WatAST::Symbol(ident) => env
            .lookup(ident.as_str())
            .ok_or_else(|| RuntimeError::UnboundSymbol(ident.name.clone())),
        WatAST::List(items) => eval_list(items, env, sym),
    }
}

fn eval_list(
    items: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let head = items
        .first()
        .ok_or_else(|| RuntimeError::MalformedForm {
            head: "<empty>".into(),
            reason: "empty list in expression position".into(),
        })?;
    let rest = &items[1..];

    match head {
        WatAST::Keyword(k) => dispatch_keyword_head(k, rest, env, sym),
        WatAST::Symbol(ident) => {
            // Bare symbol as head — look up a callable in the env.
            let callee = env
                .lookup(ident.as_str())
                .ok_or_else(|| RuntimeError::UnboundSymbol(ident.name.clone()))?;
            apply_value(&callee, rest, env, sym)
        }
        WatAST::List(_) => {
            // Inline lambda call: ((lambda ...) arg1 arg2)
            let callee = eval(head, env, sym)?;
            apply_value(&callee, rest, env, sym)
        }
        other => Err(RuntimeError::MalformedForm {
            head: ast_variant_name(other).into(),
            reason: "call head must be a keyword, symbol, or list".into(),
        }),
    }
}

fn dispatch_keyword_head(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    match head {
        // Language forms
        ":wat::core::define" => Err(RuntimeError::DefineInExpressionPosition),
        ":wat::core::lambda" => eval_lambda(args, env),
        ":wat::core::let" => eval_let(args, env, sym),
        ":wat::core::if" => eval_if(args, env, sym),

        // Arithmetic
        ":wat::core::+" => eval_arith(head, args, env, sym, |a, b| Ok(a + b), |a, b| Ok(a + b)),
        ":wat::core::-" => eval_arith(head, args, env, sym, |a, b| Ok(a - b), |a, b| Ok(a - b)),
        ":wat::core::*" => eval_arith(head, args, env, sym, |a, b| Ok(a * b), |a, b| Ok(a * b)),
        ":wat::core::/" => eval_arith(
            head,
            args,
            env,
            sym,
            |a, b| if b == 0 { Err(RuntimeError::DivisionByZero) } else { Ok(a / b) },
            |a, b| if b == 0.0 { Err(RuntimeError::DivisionByZero) } else { Ok(a / b) },
        ),

        // Comparison — return :bool
        ":wat::core::=" => eval_compare(head, args, env, sym, |o| o == std::cmp::Ordering::Equal),
        ":wat::core::<" => eval_compare(head, args, env, sym, |o| o == std::cmp::Ordering::Less),
        ":wat::core::>" => eval_compare(head, args, env, sym, |o| o == std::cmp::Ordering::Greater),
        ":wat::core::<=" => eval_compare(head, args, env, sym, |o| {
            o != std::cmp::Ordering::Greater
        }),
        ":wat::core::>=" => eval_compare(head, args, env, sym, |o| o != std::cmp::Ordering::Less),

        // Boolean
        ":wat::core::not" => eval_not(args, env, sym),
        ":wat::core::and" => eval_and(args, env, sym),
        ":wat::core::or" => eval_or(args, env, sym),

        // List construction
        ":wat::core::vec" => eval_list_ctor(args, env, sym),

        // Algebra-core UpperCalls — construct HolonAST values at runtime.
        ":wat::algebra::Atom" => eval_algebra_atom(args, env, sym),
        ":wat::algebra::Bind" => eval_algebra_bind(args, env, sym),
        ":wat::algebra::Bundle" => eval_algebra_bundle(args, env, sym),
        ":wat::algebra::Permute" => eval_algebra_permute(args, env, sym),
        ":wat::algebra::Thermometer" => eval_algebra_thermometer(args, env, sym),
        ":wat::algebra::Blend" => eval_algebra_blend(args, env, sym),

        // Anything else: user-defined function lookup.
        other => {
            let func = sym
                .get(other)
                .ok_or_else(|| RuntimeError::UnknownFunction(other.to_string()))?
                .clone();
            let vals = args
                .iter()
                .map(|a| eval(a, env, sym))
                .collect::<Result<Vec<_>, _>>()?;
            apply_function(&func, vals, sym)
        }
    }
}

// ─── Language forms ─────────────────────────────────────────────────────

fn eval_lambda(args: &[WatAST], env: &Environment) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::lambda".into(),
            reason: format!(
                "expected (:wat::core::lambda signature body); got {} args",
                args.len()
            ),
        });
    }
    let sig = &args[0];
    let body = &args[1];
    let (params, param_types, ret_type) = parse_lambda_signature(sig)?;
    Ok(Value::Function(Arc::new(Function {
        name: None,
        params,
        type_params: Vec::new(),
        param_types,
        ret_type,
        body: Arc::new(body.clone()),
        closed_env: Some(env.clone()),
    })))
}

/// Parse a lambda signature list `((p1 :T1) (p2 :T2) ... -> :R)`.
///
/// Per 058-029, lambdas carry the SAME typing discipline as `define`:
/// every parameter is `(name :Type)` and the return type is required.
/// No "untyped lambda" exists in wat — the language is strongly typed
/// at every function boundary. This parser rejects a signature that
/// omits a type annotation or the `-> :Return` tail.
fn parse_lambda_signature(
    sig: &WatAST,
) -> Result<(Vec<String>, Vec<crate::types::TypeExpr>, crate::types::TypeExpr), RuntimeError> {
    let items = match sig {
        WatAST::List(items) => items,
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::lambda".into(),
                reason: "signature must be a list".into(),
            });
        }
    };
    let mut params = Vec::new();
    let mut param_types = Vec::new();
    let mut ret_type: Option<crate::types::TypeExpr> = None;
    let mut saw_arrow = false;
    for item in items {
        if saw_arrow {
            if ret_type.is_some() {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::lambda".into(),
                    reason: "signature has more than one return type after '->'".into(),
                });
            }
            match item {
                WatAST::Keyword(k) => {
                    ret_type = Some(parse_type_keyword(k)?);
                }
                other => {
                    return Err(RuntimeError::MalformedForm {
                        head: ":wat::core::lambda".into(),
                        reason: format!(
                            "return type after '->' must be a type keyword; got {}",
                            ast_variant_name(other)
                        ),
                    });
                }
            }
            continue;
        }
        match item {
            WatAST::Symbol(s) if s.as_str() == "->" => {
                saw_arrow = true;
            }
            WatAST::List(pair) => {
                let (pname, ptype) = parse_param_pair(pair.clone())?;
                params.push(pname);
                param_types.push(ptype);
            }
            other => {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::lambda".into(),
                    reason: format!(
                        "unexpected signature element: {}",
                        ast_variant_name(other)
                    ),
                });
            }
        }
    }
    let ret_type = ret_type.ok_or_else(|| RuntimeError::MalformedForm {
        head: ":wat::core::lambda".into(),
        reason:
            "lambda signature must end with '-> :Type' (typed return is required per 058-029)"
                .into(),
    })?;
    Ok((params, param_types, ret_type))
}

fn eval_let(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::let".into(),
            reason: format!(
                "expected (:wat::core::let (((n1 :T1) e1) ...) body); got {} args",
                args.len()
            ),
        });
    }
    let bindings_form = &args[0];
    let body = &args[1];

    let binding_pairs = match bindings_form {
        WatAST::List(items) => items,
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: "bindings must be a list of ((name :Type) expr) pairs".into(),
            })
        }
    };

    let mut builder = env.child();
    for pair in binding_pairs {
        let (name, _declared_type, rhs) = parse_let_binding(pair)?;
        // Runtime ignores the declared type — type checking ran before
        // eval. Parsing it here enforces the grammar: an untyped
        // binding like `(x 42)` halts here rather than being allowed.
        let value = eval(rhs, env, sym)?; // eval in OUTER env, not cumulative let*
        builder = builder.bind(name, value);
    }
    let scope = builder.build();
    eval(body, &scope, sym)
}

/// Parse a single let binding. Per the typed-let discipline, every
/// binding is `((name :Type) rhs)` — a 2-list whose first element is
/// itself a 2-list `(name :Type)` and whose second is the RHS
/// expression. Untyped `(name rhs)` is refused.
///
/// Returns `(name, declared_type, rhs)`. Declared type is validated
/// via [`crate::types::parse_type_expr`] so `:Any` and malformed
/// type expressions are caught at this layer.
fn parse_let_binding(
    pair: &WatAST,
) -> Result<(String, crate::types::TypeExpr, &WatAST), RuntimeError> {
    let kv = match pair {
        WatAST::List(items) if items.len() == 2 => items,
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "each binding must be ((name :Type) rhs); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let typed_name = match &kv[0] {
        WatAST::List(inner) if inner.len() == 2 => inner,
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "binding's name-and-type must be (name :Type); got {}. Every let binding declares its type — no untyped form.",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let name = match &typed_name[0] {
        WatAST::Symbol(ident) => ident.name.clone(),
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "binding name must be a bare symbol; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let declared_type = match &typed_name[1] {
        WatAST::Keyword(k) => parse_type_keyword(k)?,
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "binding type must be a type keyword; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    Ok((name, declared_type, &kv[1]))
}

fn eval_if(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond then else); got {} args",
                args.len()
            ),
        });
    }
    let cond_val = eval(&args[0], env, sym)?;
    match cond_val {
        Value::Bool(true) => eval(&args[1], env, sym),
        Value::Bool(false) => eval(&args[2], env, sym),
        other => Err(RuntimeError::BadCondition {
            got: other.type_name(),
        }),
    }
}

// ─── Built-ins ──────────────────────────────────────────────────────────

fn eval_arith<IF, FF>(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    int_op: IF,
    float_op: FF,
) -> Result<Value, RuntimeError>
where
    IF: Fn(i64, i64) -> Result<i64, RuntimeError>,
    FF: Fn(f64, f64) -> Result<f64, RuntimeError>,
{
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: head.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(int_op(x, y)?)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(float_op(x, y)?)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float(float_op(x as f64, y)?)),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(float_op(x, y as f64)?)),
        (a, b) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "numeric (i64 or f64)",
            got: if !matches!(a, Value::Int(_) | Value::Float(_)) {
                a.type_name()
            } else {
                b.type_name()
            },
        }),
    }
}

fn eval_compare<F: Fn(std::cmp::Ordering) -> bool>(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    pred: F,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: head.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?;
    let order = match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(x), Value::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(x), Value::Int(y)) => x
            .partial_cmp(&(*y as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(x), Value::String(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (Value::Keyword(x), Value::Keyword(y)) => x.cmp(y),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: head.into(),
                expected: "matching comparable pair",
                got: a.type_name(),
            });
        }
    };
    Ok(Value::Bool(pred(order)))
}

fn eval_not(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::not".into(),
            expected: 1,
            got: args.len(),
        });
    }
    match eval(&args[0], env, sym)? {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::core::not".into(),
            expected: "bool",
            got: other.type_name(),
        }),
    }
}

fn eval_and(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // Short-circuit: false wins.
    for arg in args {
        match eval(arg, env, sym)? {
            Value::Bool(false) => return Ok(Value::Bool(false)),
            Value::Bool(true) => continue,
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::core::and".into(),
                    expected: "bool",
                    got: other.type_name(),
                })
            }
        }
    }
    Ok(Value::Bool(true))
}

fn eval_or(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    for arg in args {
        match eval(arg, env, sym)? {
            Value::Bool(true) => return Ok(Value::Bool(true)),
            Value::Bool(false) => continue,
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::core::or".into(),
                    expected: "bool",
                    got: other.type_name(),
                })
            }
        }
    }
    Ok(Value::Bool(false))
}

fn eval_list_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let items = args
        .iter()
        .map(|a| eval(a, env, sym))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::List(Arc::new(items)))
}

// ─── Algebra-core UpperCall runtime construction ────────────────────────

fn eval_algebra_atom(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Atom".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    value_to_atom(v)
}

fn value_to_atom(v: Value) -> Result<Value, RuntimeError> {
    // Atomize a runtime value: wrap it in an Atom Holon whose payload
    // registry dispatches on the value's concrete Rust type.
    let holon = match v {
        Value::Int(n) => HolonAST::atom(n),
        Value::Float(x) => HolonAST::atom(x),
        Value::Bool(b) => HolonAST::atom(b),
        Value::String(s) => HolonAST::atom((*s).clone()),
        Value::Keyword(k) => HolonAST::keyword(&k),
        Value::Holon(h) => HolonAST::atom((*h).clone()),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::algebra::Atom".into(),
                expected: "atomizable value (Int/Float/Bool/String/Keyword/Holon)",
                got: other.type_name(),
            });
        }
    };
    Ok(Value::Holon(Arc::new(holon)))
}

fn eval_algebra_bind(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Bind".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = require_holon(":wat::algebra::Bind", eval(&args[0], env, sym)?)?;
    let b = require_holon(":wat::algebra::Bind", eval(&args[1], env, sym)?)?;
    Ok(Value::Holon(Arc::new(HolonAST::bind((*a).clone(), (*b).clone()))))
}

fn eval_algebra_bundle(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Bundle".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let list = match eval(&args[0], env, sym)? {
        Value::List(l) => l,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::algebra::Bundle".into(),
                expected: "List<Holon> from (:wat::core::vec ...)",
                got: other.type_name(),
            });
        }
    };
    let children: Result<Vec<HolonAST>, _> = list
        .iter()
        .map(|v| {
            require_holon(":wat::algebra::Bundle list element", v.clone())
                .map(|h| (*h).clone())
        })
        .collect();
    Ok(Value::Holon(Arc::new(HolonAST::bundle(children?))))
}

fn eval_algebra_permute(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Permute".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let child = require_holon(":wat::algebra::Permute", eval(&args[0], env, sym)?)?;
    let k = match eval(&args[1], env, sym)? {
        Value::Int(n) => i32::try_from(n).map_err(|_| RuntimeError::TypeMismatch {
            op: ":wat::algebra::Permute".into(),
            expected: "i32 step (integer fitting in i32)",
            got: "i64 out of range",
        })?,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::algebra::Permute".into(),
                expected: "i32 step",
                got: other.type_name(),
            });
        }
    };
    Ok(Value::Holon(Arc::new(HolonAST::permute((*child).clone(), k))))
}

fn eval_algebra_thermometer(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Thermometer".into(),
            expected: 3,
            got: args.len(),
        });
    }
    let v = require_numeric(":wat::algebra::Thermometer", eval(&args[0], env, sym)?)?;
    let mn = require_numeric(":wat::algebra::Thermometer", eval(&args[1], env, sym)?)?;
    let mx = require_numeric(":wat::algebra::Thermometer", eval(&args[2], env, sym)?)?;
    Ok(Value::Holon(Arc::new(HolonAST::thermometer(v, mn, mx))))
}

fn eval_algebra_blend(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::Blend".into(),
            expected: 4,
            got: args.len(),
        });
    }
    let a = require_holon(":wat::algebra::Blend", eval(&args[0], env, sym)?)?;
    let b = require_holon(":wat::algebra::Blend", eval(&args[1], env, sym)?)?;
    let w1 = require_numeric(":wat::algebra::Blend", eval(&args[2], env, sym)?)?;
    let w2 = require_numeric(":wat::algebra::Blend", eval(&args[3], env, sym)?)?;
    Ok(Value::Holon(Arc::new(HolonAST::blend((*a).clone(), (*b).clone(), w1, w2))))
}

fn require_holon(op: &str, v: Value) -> Result<Arc<HolonAST>, RuntimeError> {
    match v {
        Value::Holon(h) => Ok(h),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Holon",
            got: other.type_name(),
        }),
    }
}

fn require_numeric(op: &str, v: Value) -> Result<f64, RuntimeError> {
    match v {
        Value::Int(n) => Ok(n as f64),
        Value::Float(x) => Ok(x),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "numeric",
            got: other.type_name(),
        }),
    }
}

// ─── Function application ───────────────────────────────────────────────

fn apply_value(
    callee: &Value,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let func = match callee {
        Value::Function(f) => f.clone(),
        other => {
            return Err(RuntimeError::NotCallable {
                got: other.type_name(),
            })
        }
    };
    let vals = args
        .iter()
        .map(|a| eval(a, env, sym))
        .collect::<Result<Vec<_>, _>>()?;
    apply_function(&func, vals, sym)
}

/// Apply a function to a list of argument values, evaluated under the
/// given symbol table. Arity must match the function's declared
/// parameters; mismatch returns [`RuntimeError::ArityMismatch`].
///
/// Public so the freeze module's `:user::main` invocation and
/// constrained-eval paths can apply pre-registered functions from a
/// frozen world without duplicating the param-binding logic.
pub fn apply_function(
    func: &Function,
    args: Vec<Value>,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != func.params.len() {
        return Err(RuntimeError::ArityMismatch {
            op: func.name.clone().unwrap_or_else(|| "<lambda>".into()),
            expected: func.params.len(),
            got: args.len(),
        });
    }
    // Build the call env: parent is the closed env (lambda) or a fresh
    // root (define — the body resolves global names via sym).
    let parent = func.closed_env.clone().unwrap_or_else(Environment::new);
    let mut builder = parent.child();
    for (name, value) in func.params.iter().zip(args.into_iter()) {
        builder = builder.bind(name.clone(), value);
    }
    let call_env = builder.build();
    eval(&func.body, &call_env, sym)
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
    use crate::parser::{parse_all, parse_one};

    fn run(src: &str) -> Result<Value, RuntimeError> {
        let forms = parse_all(src).expect("parse ok");
        let mut sym = SymbolTable::new();
        let rest = register_defines(forms, &mut sym)?;
        let env = Environment::new();
        let mut last = Value::Unit;
        for form in &rest {
            last = eval(form, &env, &sym)?;
        }
        Ok(last)
    }

    fn eval_expr(src: &str) -> Result<Value, RuntimeError> {
        let ast = parse_one(src).expect("parse ok");
        eval(&ast, &Environment::new(), &SymbolTable::new())
    }

    // ─── Literals ───────────────────────────────────────────────────────

    #[test]
    fn int_literal() {
        assert!(matches!(eval_expr("42").unwrap(), Value::Int(42)));
    }

    #[test]
    fn float_literal() {
        match eval_expr("3.14").unwrap() {
            Value::Float(x) => assert_eq!(x, 3.14),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn bool_literals() {
        assert!(matches!(eval_expr("true").unwrap(), Value::Bool(true)));
        assert!(matches!(eval_expr("false").unwrap(), Value::Bool(false)));
    }

    #[test]
    fn string_literal() {
        match eval_expr(r#""hello""#).unwrap() {
            Value::String(s) => assert_eq!(&*s, "hello"),
            v => panic!("expected string, got {:?}", v),
        }
    }

    // ─── Arithmetic ─────────────────────────────────────────────────────

    #[test]
    fn add_ints() {
        assert!(matches!(
            eval_expr("(:wat::core::+ 2 3)").unwrap(),
            Value::Int(5)
        ));
    }

    #[test]
    fn subtract_ints() {
        assert!(matches!(
            eval_expr("(:wat::core::- 10 4)").unwrap(),
            Value::Int(6)
        ));
    }

    #[test]
    fn multiply_mixed_promotes_to_float() {
        match eval_expr("(:wat::core::* 3 2.0)").unwrap() {
            Value::Float(x) => assert_eq!(x, 6.0),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn divide_by_zero_errors() {
        assert!(matches!(
            eval_expr("(:wat::core::/ 5 0)"),
            Err(RuntimeError::DivisionByZero)
        ));
    }

    // ─── Comparison ─────────────────────────────────────────────────────

    #[test]
    fn equality() {
        assert!(matches!(
            eval_expr("(:wat::core::= 3 3)").unwrap(),
            Value::Bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::core::= 3 4)").unwrap(),
            Value::Bool(false)
        ));
    }

    #[test]
    fn less_than() {
        assert!(matches!(
            eval_expr("(:wat::core::< 2 3)").unwrap(),
            Value::Bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::core::< 3 2)").unwrap(),
            Value::Bool(false)
        ));
    }

    // ─── Boolean ────────────────────────────────────────────────────────

    #[test]
    fn and_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::and true false true)").unwrap(),
            Value::Bool(false)
        ));
    }

    #[test]
    fn or_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::or false false true false)").unwrap(),
            Value::Bool(true)
        ));
    }

    #[test]
    fn not_bool() {
        assert!(matches!(
            eval_expr("(:wat::core::not true)").unwrap(),
            Value::Bool(false)
        ));
    }

    // ─── Control flow ───────────────────────────────────────────────────

    #[test]
    fn if_true_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if true 1 2)").unwrap(),
            Value::Int(1)
        ));
    }

    #[test]
    fn if_false_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if false 1 2)").unwrap(),
            Value::Int(2)
        ));
    }

    #[test]
    fn if_non_bool_rejected() {
        assert!(matches!(
            eval_expr("(:wat::core::if 42 1 2)"),
            Err(RuntimeError::BadCondition { .. })
        ));
    }

    #[test]
    fn let_binds_parallel() {
        assert!(matches!(
            eval_expr(
                r#"(:wat::core::let (((x :i64) 2) ((y :i64) 3)) (:wat::core::+ x y))"#
            )
            .unwrap(),
            Value::Int(5)
        ));
    }

    #[test]
    fn let_shadows_outer() {
        // Inner let shadows the outer x.
        assert!(matches!(
            eval_expr(
                r#"(:wat::core::let (((x :i64) 1)) (:wat::core::let (((x :i64) 100)) x))"#
            )
            .unwrap(),
            Value::Int(100)
        ));
    }

    #[test]
    fn untyped_let_binding_rejected() {
        // The old `(name rhs)` shape is no longer legal; every binding
        // must declare its type via `((name :Type) rhs)`.
        let err = eval_expr(r#"(:wat::core::let ((x 1)) x)"#).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn let_binding_with_any_type_rejected() {
        // :Any is banned by parse_type_expr; a let binding declaring
        // :Any halts with a typed-form error.
        let err = eval_expr(r#"(:wat::core::let (((x :Any) 1)) x)"#).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    // ─── Define + function call ─────────────────────────────────────────

    #[test]
    fn define_and_call() {
        let result = run(
            r#"
            (:wat::core::define (:my::app::inc (x :i64) -> :i64)
              (:wat::core::+ x 1))
            (:my::app::inc 41)
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn define_recursive_factorial() {
        let result = run(
            r#"
            (:wat::core::define (:my::app::fact (n :i64) -> :i64)
              (:wat::core::if (:wat::core::= n 0)
                  1
                  (:wat::core::* n (:my::app::fact (:wat::core::- n 1)))))
            (:my::app::fact 5)
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::Int(120)));
    }

    #[test]
    fn reserved_prefix_define_rejected() {
        let err = run(
            r#"(:wat::core::define (:wat::algebra::Bogus (x :i64) -> :i64) x)"#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::ReservedPrefix(_)));
    }

    #[test]
    fn duplicate_define_rejected() {
        let err = run(
            r#"
            (:wat::core::define (:foo (x :i64) -> :i64) x)
            (:wat::core::define (:foo (x :i64) -> :i64) (:wat::core::+ x 1))
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::DuplicateDefine(_)));
    }

    #[test]
    fn undefined_function_errors() {
        assert!(matches!(
            eval_expr("(:my::app::missing 1)"),
            Err(RuntimeError::UnknownFunction(_))
        ));
    }

    // ─── Lambda + closures ──────────────────────────────────────────────

    #[test]
    fn lambda_as_value() {
        // The lambda produces a callable; invoking it inline.
        let result = eval_expr(
            r#"((:wat::core::lambda ((x :i64) (y :i64) -> :i64)
                  (:wat::core::+ x y))
                3 4)"#,
        )
        .unwrap();
        assert!(matches!(result, Value::Int(7)));
    }

    #[test]
    fn closure_captures_let_binding() {
        let result = eval_expr(
            r#"(:wat::core::let
                 (((adder :fn(i64)->i64)
                   (:wat::core::lambda ((x :i64) -> :i64)
                     (:wat::core::+ x 10))))
                 (adder 5))"#,
        )
        .unwrap();
        assert!(matches!(result, Value::Int(15)));
    }

    #[test]
    fn closure_captures_enclosing_variable() {
        // The lambda captures `n` from the outer let; even when invoked
        // from a deeper scope, it sees the captured value.
        let result = eval_expr(
            r#"(:wat::core::let (((n :i64) 100))
                 (:wat::core::let (((f :fn(i64)->i64)
                                  (:wat::core::lambda ((x :i64) -> :i64)
                                    (:wat::core::+ x n))))
                   (:wat::core::let (((n :i64) 999))
                     (f 1))))"#,
        )
        .unwrap();
        // Expected: f captured n=100, so f(1) = 1 + 100 = 101 regardless of inner rebind.
        assert!(matches!(result, Value::Int(101)));
    }

    // ─── Algebra-core runtime construction ──────────────────────────────

    #[test]
    fn algebra_atom_from_literal() {
        let v = eval_expr(r#"(:wat::algebra::Atom "role")"#).unwrap();
        assert!(matches!(v, Value::Holon(_)));
    }

    #[test]
    fn algebra_atom_from_bound_variable() {
        // (Atom x) where x is a let-bound integer — runtime construction.
        let v = eval_expr(
            r#"(:wat::core::let (((x :i64) 42)) (:wat::algebra::Atom x))"#,
        )
        .unwrap();
        match v {
            Value::Holon(h) => {
                let recovered: Option<&i64> = holon::atom_value(&h);
                assert_eq!(recovered, Some(&42_i64));
            }
            other => panic!("expected Holon, got {:?}", other),
        }
    }

    #[test]
    fn algebra_bind_composes_holons() {
        let v = eval_expr(
            r#"(:wat::algebra::Bind
                 (:wat::algebra::Atom "role")
                 (:wat::algebra::Atom "filler"))"#,
        )
        .unwrap();
        assert!(matches!(v, Value::Holon(_)));
    }

    #[test]
    fn algebra_bundle_via_list_ctor() {
        let v = eval_expr(
            r#"(:wat::algebra::Bundle
                 (:wat::core::vec
                   (:wat::algebra::Atom "a")
                   (:wat::algebra::Atom "b")
                   (:wat::algebra::Atom "c")))"#,
        )
        .unwrap();
        assert!(matches!(v, Value::Holon(_)));
    }

    #[test]
    fn algebra_blend_with_runtime_weight() {
        // Weight computed at runtime via arithmetic.
        let v = eval_expr(
            r#"(:wat::algebra::Blend
                 (:wat::algebra::Atom "x")
                 (:wat::algebra::Atom "y")
                 1
                 (:wat::core::- 0 1))"#,
        )
        .unwrap();
        assert!(matches!(v, Value::Holon(_)));
    }

    #[test]
    fn algebra_bundle_non_list_rejected() {
        let err = eval_expr(
            r#"(:wat::algebra::Bundle (:wat::algebra::Atom "a"))"#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── Program-level integration ──────────────────────────────────────

    #[test]
    fn program_with_defines_and_algebra() {
        // A small program that defines a helper and uses it to build a Holon.
        let result = run(
            r#"
            (:wat::core::define (:my::app::encode-pair (a :String) (b :String) -> :Holon)
              (:wat::algebra::Bind
                (:wat::algebra::Atom a)
                (:wat::algebra::Atom b)))
            (:my::app::encode-pair "role" "filler")
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::Holon(_)));
    }
}
