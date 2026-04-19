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
use crate::config::Config;
use holon::{encode, AtomTypeRegistry, HolonAST, ScalarEncoder, Similarity, VectorManager};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Kernel-owned stop flag read by `(:wat::kernel::stopped)`.
///
/// The wat-vm binary installs OS signal handlers for SIGINT and
/// SIGTERM; both set this flag to `true`. User programs poll via the
/// `:wat::kernel::stopped` form to decide whether to continue their
/// main loops — whenever `true`, they drop their output senders
/// and return, which cascades clean shutdown through the channel
/// disconnects.
///
/// Lives under `:wat::kernel::` (not `:wat::config::`) because
/// config is user-set and frozen after startup; the stop flag
/// mutates at runtime under kernel control.
pub static KERNEL_STOPPED: AtomicBool = AtomicBool::new(false);

/// Set the kernel stop flag to `true`. Called by the wat-vm CLI's
/// signal handler. After `true` is set, any user program polling
/// `(:wat::kernel::stopped)` will observe it and can begin clean
/// shutdown.
pub fn request_kernel_stop() {
    KERNEL_STOPPED.store(true, Ordering::SeqCst);
}

/// Reset the kernel stop flag. Used only by test harnesses that
/// exercise the flag within a single process — the flag is a
/// process-lifetime static and test ordering can otherwise leak
/// state between tests.
#[cfg(test)]
pub fn reset_kernel_stop() {
    KERNEL_STOPPED.store(false, Ordering::SeqCst);
}

/// Runtime value.
///
/// **Variant names encode their Rust or conceptual origin path via
/// `__` as the namespace separator.** `crossbeam_channel::Sender`
/// becomes `crossbeam_channel__Sender`; only internal `::` is encoded
/// (leading `::` is never written in Rust paths and not encoded here).
/// Prelude types (`bool`, `i64`, `f64`, `String`, `Vec`, `()`) stay
/// short because that's what Rust users write — wat follows Rust's
/// prelude convention.
///
/// `type_name()` returns the full `::`-separated path users write in
/// wat source. Every Value carries its honest identity; error messages
/// say what the user would recognize.
#[derive(Debug, Clone)]
#[allow(non_camel_case_types, non_snake_case)]
pub enum Value {
    bool(bool),
    i64(i64),
    f64(f64),
    String(Arc<String>),
    /// A `Vec<Value>` — constructed by `:wat::core::vec`.
    Vec(Arc<Vec<Value>>),
    /// The empty tuple / Rust unit `()`. Named `Unit` since `()` isn't
    /// a legal identifier.
    Unit,
    /// Keyword literal — leading `:` included. Wat-source type
    /// `:wat::core::keyword`.
    wat__core__keyword(Arc<String>),
    /// A callable — `define`-registered function or `lambda` closure.
    /// Per 058-029: `define` = `lambda` + startup-time symbol-table
    /// registration. Static type is `:fn(A,B,...)->R`; the variant
    /// records HOW it was produced.
    wat__core__lambda(Arc<Function>),
    /// A composed `holon::HolonAST` — the algebra AST tier carried
    /// at runtime.
    holon__HolonAST(Arc<HolonAST>),
    /// A parsed wat AST carried as a first-class runtime value. Used
    /// by `:wat::core::eval-ast!` and adjacent forms. Distinct from
    /// [`Value::String`] (raw EDN text that still needs parsing) and
    /// from [`Value::holon__HolonAST`] (algebra AST).
    wat__WatAST(Arc<WatAST>),
    /// A channel sender handle. String-typed for the MVP wat-vm; the
    /// variant encodes the full `crossbeam_channel::Sender` path —
    /// wat takes a direct dep on `crossbeam-channel` and does not
    /// hide it.
    crossbeam_channel__Sender(Arc<crossbeam_channel::Sender<String>>),
    /// A channel receiver handle. String-typed for the MVP.
    crossbeam_channel__Receiver(Arc<crossbeam_channel::Receiver<String>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::bool(_) => "bool",
            Value::i64(_) => "i64",
            Value::f64(_) => "f64",
            Value::String(_) => "String",
            Value::Vec(_) => "Vec",
            Value::Unit => "()",
            Value::wat__core__keyword(_) => "wat::core::keyword",
            Value::wat__core__lambda(_) => "wat::core::lambda",
            Value::holon__HolonAST(_) => "holon::HolonAST",
            Value::wat__WatAST(_) => "wat::WatAST",
            Value::crossbeam_channel__Sender(_) => "crossbeam_channel::Sender",
            Value::crossbeam_channel__Receiver(_) => "crossbeam_channel::Receiver",
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

/// Runtime encoding context — the machinery needed to project a
/// `HolonAST` into its `Vector` at eval time.
///
/// Constructed once from [`Config`] at freeze and attached to the
/// frozen [`SymbolTable`]. Used by vector-level primitives like
/// `:wat::core::presence` (FOUNDATION 1718), which measure cosine
/// similarity between encoded holons against the substrate noise floor.
///
/// Holds `Arc`s so it can be cloned cheaply by the runtime when a
/// primitive needs encoding access; the underlying `VectorManager` and
/// `ScalarEncoder` are pure caches that can be shared across threads.
#[derive(Clone)]
pub struct EncodingCtx {
    pub vm: Arc<VectorManager>,
    pub scalar: Arc<ScalarEncoder>,
    pub registry: Arc<AtomTypeRegistry>,
    pub config: Config,
}

impl EncodingCtx {
    /// Build an encoding context from the frozen [`Config`]. `dims` and
    /// `global_seed` drive deterministic atom vectors; the registry
    /// is seeded with the built-in atom payload types (i64, f64, bool,
    /// String, keyword, HolonAST) plus [`WatAST`] for programs-as-atoms
    /// — a program captured via `:wat::core::quote` and wrapped in an
    /// `:wat::algebra::Atom` needs a stable canonical form so it can
    /// encode to a deterministic vector.
    pub fn from_config(cfg: &Config) -> Self {
        let mut registry = AtomTypeRegistry::with_builtins();
        registry.register::<WatAST>("wat/WatAST", |ast, _reg| canonical_wat_ast(ast));
        EncodingCtx {
            vm: Arc::new(VectorManager::with_seed(cfg.dims, cfg.global_seed)),
            scalar: Arc::new(ScalarEncoder::with_seed(cfg.dims, cfg.global_seed)),
            registry: Arc::new(registry),
            config: *cfg,
        }
    }
}

/// Canonical byte encoding of a [`WatAST`] for atom-payload hashing.
///
/// Deterministic per spec: same AST ⇒ same bytes ⇒ same vector seed.
/// Format is a simple tagged recursive serialization — variant tag
/// (1 byte) followed by variant-specific body. Used only for atom
/// canonicalization inside the registry; not a wire format.
fn canonical_wat_ast(ast: &WatAST) -> Vec<u8> {
    let mut out = Vec::new();
    write_wat_ast(ast, &mut out);
    out
}

fn write_wat_ast(ast: &WatAST, out: &mut Vec<u8>) {
    match ast {
        WatAST::IntLit(n) => {
            out.push(0);
            out.extend_from_slice(&n.to_le_bytes());
        }
        WatAST::FloatLit(x) => {
            out.push(1);
            out.extend_from_slice(&x.to_le_bytes());
        }
        WatAST::BoolLit(b) => {
            out.push(2);
            out.push(if *b { 1 } else { 0 });
        }
        WatAST::StringLit(s) => {
            out.push(3);
            write_bytes(s.as_bytes(), out);
        }
        WatAST::Keyword(k) => {
            out.push(4);
            write_bytes(k.as_bytes(), out);
        }
        WatAST::Symbol(ident) => {
            out.push(5);
            write_bytes(ident.name.as_bytes(), out);
            // Scope IDs — sorted (BTreeSet already provides order).
            out.extend_from_slice(&(ident.scopes.len() as u64).to_le_bytes());
            for sid in ident.scopes.iter() {
                out.extend_from_slice(&sid.0.to_le_bytes());
            }
        }
        WatAST::List(items) => {
            out.push(6);
            out.extend_from_slice(&(items.len() as u64).to_le_bytes());
            for child in items {
                write_wat_ast(child, out);
            }
        }
    }
}

fn write_bytes(bytes: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
}

impl fmt::Debug for EncodingCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncodingCtx")
            .field("dims", &self.config.dims)
            .field("global_seed", &self.config.global_seed)
            .field("noise_floor", &self.config.noise_floor)
            .finish()
    }
}

/// Keyword-path ↦ Function registry + optional encoding context.
///
/// The `encoding_ctx` field is populated at freeze time by the startup
/// pipeline. Test harnesses (`SymbolTable::new()`) leave it `None`;
/// primitives that require encoding (presence, encode) error cleanly if
/// invoked without a frozen context.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub functions: HashMap<String, Arc<Function>>,
    pub encoding_ctx: Option<Arc<EncodingCtx>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, path: &str) -> Option<&Arc<Function>> {
        self.functions.get(path)
    }

    /// Attach an encoding context. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`].
    pub fn set_encoding_ctx(&mut self, ctx: Arc<EncodingCtx>) {
        self.encoding_ctx = Some(ctx);
    }

    /// Borrow the encoding context, if one is attached. Runtime
    /// primitives that require encoding (`:wat::core::presence`) call
    /// this and raise [`RuntimeError::NoEncodingCtx`] on `None`.
    pub fn encoding_ctx(&self) -> Option<&Arc<EncodingCtx>> {
        self.encoding_ctx.as_ref()
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
    /// Verification failed for a `:wat::core::eval-digest!` /
    /// `:wat::core::eval-signed!` call. The wrapped [`HashError`]
    /// names the specific failure (mismatched digest, invalid
    /// signature, unsupported algorithm, malformed payload).
    EvalVerificationFailed { err: crate::hash::HashError },
    /// A `:wat::kernel::recv` call on a channel whose sender has been
    /// dropped, or a `:wat::kernel::send` on a channel whose receiver
    /// has been dropped. MVP `recv` returns `:String` and errors on
    /// disconnect; when `:Option<T>` + match lands, recv disconnect
    /// becomes a first-class `:None` instead of an error.
    ChannelDisconnected { op: String },
    /// A vector-level primitive (`:wat::core::presence`,
    /// `:wat::config::noise-floor`, etc.) was invoked but the
    /// [`SymbolTable`] has no attached [`EncodingCtx`]. Reachable from
    /// test harnesses that don't go through freeze; the frozen startup
    /// pipeline always installs one.
    NoEncodingCtx { op: String },
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
            RuntimeError::EvalVerificationFailed { err } => {
                write!(f, "eval verification failed: {}", err)
            }
            RuntimeError::ChannelDisconnected { op } => write!(
                f,
                "{}: channel disconnected (counterparty dropped); MVP recv returns :String and errors on disconnect — :Option<T> lands with match",
                op
            ),
            RuntimeError::NoEncodingCtx { op } => write!(
                f,
                "{}: no encoding context attached to SymbolTable; presence / config accessors need a frozen EncodingCtx. Call via the freeze pipeline rather than a bare SymbolTable::new().",
                op
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
        WatAST::IntLit(n) => Ok(Value::i64(*n)),
        WatAST::FloatLit(x) => Ok(Value::f64(*x)),
        WatAST::BoolLit(b) => Ok(Value::bool(*b)),
        WatAST::StringLit(s) => Ok(Value::String(Arc::new(s.clone()))),
        WatAST::Keyword(k) => Ok(Value::wat__core__keyword(Arc::new(k.clone()))),
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
        ":wat::core::let*" => eval_let_star(args, env, sym),
        ":wat::core::if" => eval_if(args, env, sym),
        ":wat::core::quote" => eval_quote(args),
        ":wat::core::atom-value" => eval_atom_value(args, env, sym),

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

        // Presence — the retrieval primitive per FOUNDATION 1718.
        // Cosine between encoded target and encoded reference. Returns
        // scalar :f64; the caller binarizes at the noise floor.
        ":wat::core::presence" => eval_presence(args, env, sym),

        // Constrained runtime eval — four forms, matching the load
        // pipeline's discipline on source interface and verification.
        ":wat::core::eval-ast!" => eval_form_ast(args, env, sym),
        ":wat::core::eval-edn!" => eval_form_edn(args, env, sym),
        ":wat::core::eval-digest!" => eval_form_digest(args, env, sym),
        ":wat::core::eval-signed!" => eval_form_signed(args, env, sym),

        // Kernel primitives — channel IO + stop flag.
        ":wat::kernel::stopped" => eval_kernel_stopped(args),
        ":wat::kernel::send" => eval_kernel_send(args, env, sym),
        ":wat::kernel::recv" => eval_kernel_recv(args, env, sym),

        // Config accessors — read committed config fields at runtime.
        ":wat::config::dims" => eval_config_dims(args, sym),
        ":wat::config::global-seed" => eval_config_global_seed(args, sym),
        ":wat::config::noise-floor" => eval_config_noise_floor(args, sym),

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
    Ok(Value::wat__core__lambda(Arc::new(Function {
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

/// `(:wat::core::let* (((n1 :T1) e1) ((n2 :T2) e2) ...) body)` —
/// sequential let.
///
/// Same surface grammar as `:wat::core::let` but each RHS is evaluated
/// in an environment that includes the PRIOR bindings. `n2`'s RHS can
/// refer to `n1`; `n3`'s RHS can refer to both.
///
/// Rust-level semantics: cumulative `Environment` chain. Parallel `let`
/// evaluates all RHSes in the outer env then commits; `let*` commits
/// each binding before evaluating the next.
fn eval_let_star(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::let*".into(),
            reason: format!(
                "expected (:wat::core::let* (((n1 :T1) e1) ...) body); got {} args",
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
                head: ":wat::core::let*".into(),
                reason: "bindings must be a list of ((name :Type) expr) pairs".into(),
            })
        }
    };

    // Sequential: each binding commits to the env chain before the next
    // RHS evaluates, so subsequent bindings can reference earlier ones.
    let mut scope = env.clone();
    for pair in binding_pairs {
        let (name, _declared_type, rhs) = parse_let_binding(pair)?;
        let value = eval(rhs, &scope, sym)?;
        scope = scope.child().bind(name, value).build();
    }
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
        Value::bool(true) => eval(&args[1], env, sym),
        Value::bool(false) => eval(&args[2], env, sym),
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
        (Value::i64(x), Value::i64(y)) => Ok(Value::i64(int_op(x, y)?)),
        (Value::f64(x), Value::f64(y)) => Ok(Value::f64(float_op(x, y)?)),
        (Value::i64(x), Value::f64(y)) => Ok(Value::f64(float_op(x as f64, y)?)),
        (Value::f64(x), Value::i64(y)) => Ok(Value::f64(float_op(x, y as f64)?)),
        (a, b) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "numeric (i64 or f64)",
            got: if !matches!(a, Value::i64(_) | Value::f64(_)) {
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
        (Value::i64(x), Value::i64(y)) => x.cmp(y),
        (Value::f64(x), Value::f64(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::i64(x), Value::f64(y)) => (*x as f64)
            .partial_cmp(y)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::f64(x), Value::i64(y)) => x
            .partial_cmp(&(*y as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(x), Value::String(y)) => x.cmp(y),
        (Value::bool(x), Value::bool(y)) => x.cmp(y),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => x.cmp(y),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: head.into(),
                expected: "matching comparable pair",
                got: a.type_name(),
            });
        }
    };
    Ok(Value::bool(pred(order)))
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
        Value::bool(b) => Ok(Value::bool(!b)),
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
            Value::bool(false) => return Ok(Value::bool(false)),
            Value::bool(true) => continue,
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::core::and".into(),
                    expected: "bool",
                    got: other.type_name(),
                })
            }
        }
    }
    Ok(Value::bool(true))
}

fn eval_or(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    for arg in args {
        match eval(arg, env, sym)? {
            Value::bool(true) => return Ok(Value::bool(true)),
            Value::bool(false) => continue,
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::core::or".into(),
                    expected: "bool",
                    got: other.type_name(),
                })
            }
        }
    }
    Ok(Value::bool(false))
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
    Ok(Value::Vec(Arc::new(items)))
}

/// `(:wat::core::quote <expr>)` — capture an unevaluated AST.
///
/// This is the mechanism that places a wat program into the algebra as
/// data. The inner form is NOT evaluated at quote time — no side effects
/// fire, no functions are called. The AST is wrapped as a
/// `Value::wat__WatAST` and can be passed to `:wat::algebra::Atom`,
/// `:wat::core::eval-ast!`, stored in environments, etc.
///
/// Quote is how programs become holons without running.
fn eval_quote(args: &[WatAST]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::quote".into(),
            expected: 1,
            got: args.len(),
        });
    }
    Ok(Value::wat__WatAST(Arc::new(args[0].clone())))
}

/// `(:wat::core::atom-value <holon>)` — extract the payload from an Atom.
///
/// Dual of `:wat::algebra::Atom`. Given an `:Atom<T>` holon, returns the
/// `:T` payload. The payload's Rust type determines which `Value`
/// variant is returned; callers annotate the expected `T` at let-binding
/// sites, and the checker unifies through `atom-value`'s
/// `∀T. :holon::HolonAST -> :T` scheme.
///
/// Errors if the holon is not an `Atom` variant (Bind/Bundle/Permute/
/// Thermometer/Blend) or if the payload type isn't one of the dispatchable
/// atom payload types (String/i64/f64/bool/HolonAST/WatAST/keyword).
fn eval_atom_value(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::atom-value".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    let holon = match v {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::atom-value".into(),
                expected: "holon::HolonAST",
                got: other.type_name(),
            });
        }
    };
    match &*holon {
        HolonAST::Atom(payload) => {
            // Dispatch on the payload's concrete Rust type. Order
            // matters only for `String` vs keyword: HolonAST::keyword
            // stores keywords as `String` with a leading `:`. We
            // distinguish by inspecting that byte.
            if let Some(s) = payload.downcast_ref::<String>() {
                if s.starts_with(':') {
                    return Ok(Value::wat__core__keyword(Arc::new(s.clone())));
                }
                return Ok(Value::String(Arc::new(s.clone())));
            }
            if let Some(n) = payload.downcast_ref::<i64>() {
                return Ok(Value::i64(*n));
            }
            if let Some(x) = payload.downcast_ref::<f64>() {
                return Ok(Value::f64(*x));
            }
            if let Some(b) = payload.downcast_ref::<bool>() {
                return Ok(Value::bool(*b));
            }
            if let Some(w) = payload.downcast_ref::<WatAST>() {
                return Ok(Value::wat__WatAST(Arc::new(w.clone())));
            }
            if let Some(h) = payload.downcast_ref::<HolonAST>() {
                return Ok(Value::holon__HolonAST(Arc::new(h.clone())));
            }
            Err(RuntimeError::TypeMismatch {
                op: ":wat::core::atom-value".into(),
                expected: "atom payload of known type (String/i64/f64/bool/HolonAST/WatAST/keyword)",
                got: "atom payload type not registered in atom-value dispatch",
            })
        }
        _ => Err(RuntimeError::TypeMismatch {
            op: ":wat::core::atom-value".into(),
            expected: "Atom holon",
            got: "non-Atom HolonAST variant (Bind/Bundle/Permute/Thermometer/Blend)",
        }),
    }
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
        Value::i64(n) => HolonAST::atom(n),
        Value::f64(x) => HolonAST::atom(x),
        Value::bool(b) => HolonAST::atom(b),
        Value::String(s) => HolonAST::atom((*s).clone()),
        Value::wat__core__keyword(k) => HolonAST::keyword(&k),
        Value::holon__HolonAST(h) => HolonAST::atom((*h).clone()),
        // Programs-as-atoms: a quoted wat program (captured via
        // `:wat::core::quote`) becomes an Atom whose payload IS the
        // WatAST. Retrieved later via `:wat::core::atom-value` and
        // executed via `:wat::core::eval-ast!`. See VISION.md.
        Value::wat__WatAST(a) => HolonAST::atom((*a).clone()),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::algebra::Atom".into(),
                expected: "atomizable value (Int/Float/Bool/String/Keyword/Holon/WatAST)",
                got: other.type_name(),
            });
        }
    };
    Ok(Value::holon__HolonAST(Arc::new(holon)))
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

    // No AST-level simplification. MAP's bind self-inverse — Bind(Bind(x,y),x) →
    // y — is a VECTOR-level identity (and per 058-024's rejection text, holds
    // only on non-zero positions of the key; zero positions drop to 0).
    // Lifting it to the AST as a rewrite rule would overclaim exact recovery
    // where the algebra acknowledges quantized noise. Bind always constructs
    // the Bind tree; the self-inverse is observable via vector-level presence
    // measurement. FOUNDATION 1718: the retrieval primitive is cosine.
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::bind((*a).clone(), (*b).clone()))))
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
        Value::Vec(l) => l,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::algebra::Bundle".into(),
                expected: "List<holon::HolonAST> from (:wat::core::vec ...)",
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
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::bundle(children?))))
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
        Value::i64(n) => i32::try_from(n).map_err(|_| RuntimeError::TypeMismatch {
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
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::permute((*child).clone(), k))))
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
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::thermometer(v, mn, mx))))
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
    Ok(Value::holon__HolonAST(Arc::new(HolonAST::blend((*a).clone(), (*b).clone(), w1, w2))))
}

fn require_holon(op: &str, v: Value) -> Result<Arc<HolonAST>, RuntimeError> {
    match v {
        Value::holon__HolonAST(h) => Ok(h),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Holon",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::core::presence target reference) -> :f64` — the retrieval
/// primitive per FOUNDATION 1718.
///
/// Encodes both holons via the frozen [`EncodingCtx`] and returns the
/// cosine similarity in `[-1, +1]`. The algebra does NOT binarize — the
/// caller compares against `(:wat::config::noise-floor)` (or any
/// threshold of its own choosing) to derive a verdict.
///
/// Use cases: membership (`member?`), engram matching, discriminant
/// similarity, "is this atom present in this composite holon?"
fn eval_presence(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::presence".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let target = require_holon(":wat::core::presence", eval(&args[0], env, sym)?)?;
    let reference = require_holon(":wat::core::presence", eval(&args[1], env, sym)?)?;
    let ctx = require_encoding_ctx(":wat::core::presence", sym)?;

    let vt = encode(&target, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vr = encode(&reference, &ctx.vm, &ctx.scalar, &ctx.registry);
    Ok(Value::f64(Similarity::cosine(&vt, &vr)))
}

fn require_numeric(op: &str, v: Value) -> Result<f64, RuntimeError> {
    match v {
        Value::i64(n) => Ok(n as f64),
        Value::f64(x) => Ok(x),
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
        Value::wat__core__lambda(f) => f.clone(),
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

// ─── Four eval forms ────────────────────────────────────────────────────
//
// Mirror of the three load forms, with one extra on the AST side:
//
//   (:wat::core::eval-ast! <Value::Ast>)
//   (:wat::core::eval-edn! :wat::eval::<iface> <locator>)
//   (:wat::core::eval-digest! :wat::eval::<iface> <locator>
//                              :wat::verify::digest-<algo>
//                              :wat::verify::<iface> <payload>)
//   (:wat::core::eval-signed! :wat::eval::<iface> <locator>
//                              :wat::verify::signed-<algo>
//                              :wat::verify::<iface> <sig>
//                              :wat::verify::<iface> <pubkey>)
//
// `eval-ast!` takes a value that IS a parsed AST (already past any trust
// boundary); the other three take EDN source text with optional
// byte-level (digest) or meaning-level (signed) verification.
//
// The mutation-form refusal (FOUNDATION line 663) runs inside every
// path: an AST that contains a `define` / `defmacro` / `struct` / etc.
// is rejected before anything executes.

// ─── Kernel primitives: stopped / send / recv ───────────────────────────

/// `(:wat::kernel::stopped)` — nullary accessor; returns the kernel
/// stop flag as a `:bool`. The wat-vm's signal handler sets the flag
/// on SIGINT / SIGTERM; user programs poll it in their loops.
fn eval_kernel_stopped(args: &[WatAST]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::stopped".into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::bool(KERNEL_STOPPED.load(Ordering::SeqCst)))
}

// ─── Config accessors ─────────────────────────────────────────────────
//
// Every setter in `:wat::config::set-*!` commits exactly once during
// the startup's config pass. After freeze, the committed value is read
// by its nullary accessor. These have the same discipline as other
// substrate constants — no arguments, deterministic, safe to call from
// any context as long as the SymbolTable carries an EncodingCtx (which
// it does after freeze).

fn require_encoding_ctx<'a>(
    op: &'static str,
    sym: &'a SymbolTable,
) -> Result<&'a EncodingCtx, RuntimeError> {
    sym.encoding_ctx()
        .map(|arc| arc.as_ref())
        .ok_or_else(|| RuntimeError::NoEncodingCtx { op: op.into() })
}

fn check_nullary(op: &'static str, args: &[WatAST]) -> Result<(), RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(())
}

/// `(:wat::config::dims)` — committed vector dimensionality as `:i64`.
fn eval_config_dims(args: &[WatAST], sym: &SymbolTable) -> Result<Value, RuntimeError> {
    check_nullary(":wat::config::dims", args)?;
    let ctx = require_encoding_ctx(":wat::config::dims", sym)?;
    Ok(Value::i64(ctx.config.dims as i64))
}

/// `(:wat::config::global-seed)` — committed atom-seeding seed as `:i64`.
fn eval_config_global_seed(
    args: &[WatAST],
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    check_nullary(":wat::config::global-seed", args)?;
    let ctx = require_encoding_ctx(":wat::config::global-seed", sym)?;
    Ok(Value::i64(ctx.config.global_seed as i64))
}

/// `(:wat::config::noise-floor)` — committed substrate noise floor as
/// `:f64`. Per FOUNDATION 1718, defaults to `5.0 / sqrt(dims)` — the
/// 5-sigma threshold below which a presence measurement is
/// indistinguishable from noise. Applications that need tighter
/// confidence (10σ engram-recognition) or looser (rough prefiltering)
/// override via `(:wat::config::set-noise-floor! <f64>)` at startup.
fn eval_config_noise_floor(
    args: &[WatAST],
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    check_nullary(":wat::config::noise-floor", args)?;
    let ctx = require_encoding_ctx(":wat::config::noise-floor", sym)?;
    Ok(Value::f64(ctx.config.noise_floor))
}

/// `(:wat::kernel::send sender value)` — blocks until the value is
/// accepted by the channel; returns `:()`. Per 058-029 / FOUNDATION
/// the type scheme is `∀T. crossbeam_channel::Sender<T> -> T -> :()`;
/// the MVP wat-vm wires String-typed channels for stdio so the
/// concrete call shape is `Sender<String> -> String -> :()`.
fn eval_kernel_send(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::send".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let sender = match eval(&args[0], env, sym)? {
        Value::crossbeam_channel__Sender(s) => s,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::send".into(),
                expected: "crossbeam_channel::Sender",
                got: other.type_name(),
            });
        }
    };
    let msg = match eval(&args[1], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::send".into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };
    sender
        .send(msg)
        .map_err(|_| RuntimeError::ChannelDisconnected {
            op: ":wat::kernel::send".into(),
        })?;
    Ok(Value::Unit)
}

/// `(:wat::kernel::recv receiver)` — blocks until the receiver
/// produces a value or its sender is dropped. MVP returns `:String`
/// directly and raises `ChannelDisconnected` on sender drop; the
/// target shape is `∀T. Receiver<T> -> Option<T>` which requires
/// runtime `Option<T>` + pattern-matching support (future slice).
fn eval_kernel_recv(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::recv".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let receiver = match eval(&args[0], env, sym)? {
        Value::crossbeam_channel__Receiver(r) => r,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::recv".into(),
                expected: "crossbeam_channel::Receiver",
                got: other.type_name(),
            });
        }
    };
    match receiver.recv() {
        Ok(s) => Ok(Value::String(Arc::new(s))),
        Err(_) => Err(RuntimeError::ChannelDisconnected {
            op: ":wat::kernel::recv".into(),
        }),
    }
}

fn eval_form_ast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-ast!".into(),
            reason: format!(
                "(:wat::core::eval-ast! <ast-value>) takes exactly 1 argument; got {}",
                args.len()
            ),
        });
    }
    let value = eval(&args[0], env, sym)?;
    let ast = match value {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::eval-ast!".into(),
                expected: "Ast",
                got: other.type_name(),
            });
        }
    };
    run_constrained(&ast, env, sym)
}

fn eval_form_edn(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // (:wat::core::eval-edn! :wat::eval::<iface> <locator>)
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-edn!".into(),
            reason: format!(
                "(:wat::core::eval-edn! :wat::eval::<iface> <locator>) takes exactly 2 arguments; got {}",
                args.len()
            ),
        });
    }
    let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
    parse_and_run(&source, env, sym)
}

fn eval_form_digest(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // (:wat::core::eval-digest! :wat::eval::<iface> <locator>
    //                            :wat::verify::digest-<algo>
    //                            :wat::verify::<iface> <hex>)
    if args.len() != 5 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-digest!".into(),
            reason: format!(
                "(:wat::core::eval-digest! :wat::eval::<iface> <locator> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>) takes exactly 5 arguments; got {}",
                args.len()
            ),
        });
    }
    let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
    let algo = parse_verify_algo_keyword(&args[2], "digest-", ":wat::core::eval-digest!")?;
    let hex = resolve_verify_payload(&args[3], &args[4], env, sym)?;
    // Verify hash of raw source bytes BEFORE parse (mirrors digest-load!).
    crate::hash::verify_source_hash(source.as_bytes(), &algo, hex.trim()).map_err(
        |err| RuntimeError::EvalVerificationFailed { err },
    )?;
    parse_and_run(&source, env, sym)
}

fn eval_form_signed(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // (:wat::core::eval-signed! :wat::eval::<iface> <locator>
    //                            :wat::verify::signed-<algo>
    //                            :wat::verify::<iface> <sig>
    //                            :wat::verify::<iface> <pubkey>)
    if args.len() != 7 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-signed!".into(),
            reason: format!(
                "(:wat::core::eval-signed! :wat::eval::<iface> <locator> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>) takes exactly 7 arguments; got {}",
                args.len()
            ),
        });
    }
    let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
    let algo = parse_verify_algo_keyword(&args[2], "signed-", ":wat::core::eval-signed!")?;
    let sig_b64 = resolve_verify_payload(&args[3], &args[4], env, sym)?;
    let pk_b64 = resolve_verify_payload(&args[5], &args[6], env, sym)?;
    // Parse FIRST (sig is over canonical-EDN of parsed AST, which we
    // need the AST to compute — same discipline as signed-load!).
    let ast = parse_program(&source, ":wat::core::eval-signed!")?;
    crate::hash::verify_program_signature(&ast, &algo, sig_b64.trim(), pk_b64.trim())
        .map_err(|err| RuntimeError::EvalVerificationFailed { err })?;
    // After verify, run each form under the mutation-refusal guard.
    run_program(&ast, env, sym)
}

/// Resolve a `:wat::eval::<iface> <locator>` pair to a source string.
fn resolve_eval_source(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    let iface = match iface_ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::eval::<iface>".into(),
                reason: format!(
                    "eval source interface must be a :wat::eval::<iface> keyword; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    match iface {
        ":wat::eval::string" => match eval(locator_ast, env, sym)? {
            Value::String(s) => Ok((*s).clone()),
            other => Err(RuntimeError::TypeMismatch {
                op: ":wat::eval::string".into(),
                expected: "String",
                got: other.type_name(),
            }),
        },
        ":wat::eval::file-path" => match eval(locator_ast, env, sym)? {
            Value::String(s) => std::fs::read_to_string(&*s).map_err(|e| {
                RuntimeError::MalformedForm {
                    head: ":wat::eval::file-path".into(),
                    reason: format!("read {:?}: {}", s, e),
                }
            }),
            other => Err(RuntimeError::TypeMismatch {
                op: ":wat::eval::file-path".into(),
                expected: "String",
                got: other.type_name(),
            }),
        },
        ":wat::eval::http-path" | ":wat::eval::s3-path" => {
            Err(RuntimeError::MalformedForm {
                head: iface.to_string(),
                reason: format!(
                    "eval source interface {} is reserved but not implemented in this build",
                    iface
                ),
            })
        }
        other => Err(RuntimeError::MalformedForm {
            head: iface.to_string(),
            reason: format!(
                "unknown eval source interface {}; expected :wat::eval::string or :wat::eval::file-path",
                other
            ),
        }),
    }
}

/// Resolve a `:wat::verify::<iface> <locator>` pair to a payload string.
/// Parallels [`resolve_eval_source`] but in the verify namespace; used
/// for digest hex and signature / pubkey base64 payloads.
fn resolve_verify_payload(
    iface_ast: &WatAST,
    locator_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    let iface = match iface_ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::verify::<iface>".into(),
                reason: format!(
                    "verify payload interface must be a :wat::verify::<iface> keyword; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    match iface {
        ":wat::verify::string" => match eval(locator_ast, env, sym)? {
            Value::String(s) => Ok((*s).clone()),
            other => Err(RuntimeError::TypeMismatch {
                op: ":wat::verify::string".into(),
                expected: "String",
                got: other.type_name(),
            }),
        },
        ":wat::verify::file-path" => match eval(locator_ast, env, sym)? {
            Value::String(s) => std::fs::read_to_string(&*s).map_err(|e| {
                RuntimeError::MalformedForm {
                    head: ":wat::verify::file-path".into(),
                    reason: format!("read {:?}: {}", s, e),
                }
            }),
            other => Err(RuntimeError::TypeMismatch {
                op: ":wat::verify::file-path".into(),
                expected: "String",
                got: other.type_name(),
            }),
        },
        ":wat::verify::http-path" | ":wat::verify::s3-path" => {
            Err(RuntimeError::MalformedForm {
                head: iface.to_string(),
                reason: format!(
                    "verify payload interface {} is reserved but not implemented in this build",
                    iface
                ),
            })
        }
        other => Err(RuntimeError::MalformedForm {
            head: iface.to_string(),
            reason: format!(
                "unknown verify payload interface {}; expected :wat::verify::string or :wat::verify::file-path",
                other
            ),
        }),
    }
}

/// Parse a `:wat::verify::<kind>-<algo>` keyword and extract the algo.
/// `expected_kind` is `"digest-"` or `"signed-"` depending on which
/// form called this.
fn parse_verify_algo_keyword(
    ast: &WatAST,
    expected_kind: &str,
    form: &str,
) -> Result<String, RuntimeError> {
    let kw = match ast {
        WatAST::Keyword(k) => k.as_str(),
        other => {
            return Err(RuntimeError::MalformedForm {
                head: form.into(),
                reason: format!(
                    "verification algorithm must be a :wat::verify::<kind>-<algo> keyword; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let stripped = kw.strip_prefix(":wat::verify::").ok_or_else(|| {
        RuntimeError::MalformedForm {
            head: form.into(),
            reason: format!(
                "verification algorithm keyword must start with :wat::verify::; got {}",
                kw
            ),
        }
    })?;
    let algo = stripped.strip_prefix(expected_kind).ok_or_else(|| {
        RuntimeError::MalformedForm {
            head: form.into(),
            reason: format!(
                "this form expects a :wat::verify::{}<algo> keyword; got {}",
                expected_kind, kw
            ),
        }
    })?;
    if algo.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: form.into(),
            reason: format!("no algorithm named after {}", expected_kind),
        });
    }
    Ok(algo.to_string())
}

/// Parse a source string into one or more top-level forms.
fn parse_program(source: &str, form: &str) -> Result<Vec<WatAST>, RuntimeError> {
    crate::parser::parse_all(source).map_err(|e| RuntimeError::MalformedForm {
        head: form.into(),
        reason: format!("parse error: {}", e),
    })
}

/// Parse a source string and evaluate all forms in sequence under the
/// constrained-eval discipline. Returns the value of the last form
/// (or Unit if the program was empty).
fn parse_and_run(
    source: &str,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let forms = parse_program(source, ":wat::core::eval-edn!")?;
    run_program(&forms, env, sym)
}

/// Run a sequence of pre-parsed forms under the constrained-eval
/// discipline: each form has mutation heads refused before execution.
fn run_program(
    forms: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let mut last = Value::Unit;
    for form in forms {
        last = run_constrained(form, env, sym)?;
    }
    Ok(last)
}

/// Refuse mutation forms in the given AST, then delegate to the
/// normal `eval` dispatcher against the (frozen) symbol table.
fn run_constrained(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    refuse_mutation_forms_in(ast)?;
    eval(ast, env, sym)
}

fn refuse_mutation_forms_in(ast: &WatAST) -> Result<(), RuntimeError> {
    if let WatAST::List(items) = ast {
        if let Some(WatAST::Keyword(head)) = items.first() {
            if is_mutation_head(head) {
                return Err(RuntimeError::EvalForbidsMutationForm {
                    head: head.clone(),
                });
            }
        }
        for child in items {
            refuse_mutation_forms_in(child)?;
        }
    }
    Ok(())
}

fn is_mutation_head(head: &str) -> bool {
    matches!(
        head,
        ":wat::core::define"
            | ":wat::core::defmacro"
            | ":wat::core::struct"
            | ":wat::core::enum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::core::load!"
            | ":wat::core::digest-load!"
            | ":wat::core::signed-load!"
    ) || head.starts_with(":wat::config::set-")
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
        assert!(matches!(eval_expr("42").unwrap(), Value::i64(42)));
    }

    #[test]
    fn float_literal() {
        match eval_expr("3.14").unwrap() {
            Value::f64(x) => assert_eq!(x, 3.14),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn bool_literals() {
        assert!(matches!(eval_expr("true").unwrap(), Value::bool(true)));
        assert!(matches!(eval_expr("false").unwrap(), Value::bool(false)));
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
            Value::i64(5)
        ));
    }

    #[test]
    fn subtract_ints() {
        assert!(matches!(
            eval_expr("(:wat::core::- 10 4)").unwrap(),
            Value::i64(6)
        ));
    }

    #[test]
    fn multiply_mixed_promotes_to_float() {
        match eval_expr("(:wat::core::* 3 2.0)").unwrap() {
            Value::f64(x) => assert_eq!(x, 6.0),
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
            Value::bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::core::= 3 4)").unwrap(),
            Value::bool(false)
        ));
    }

    #[test]
    fn less_than() {
        assert!(matches!(
            eval_expr("(:wat::core::< 2 3)").unwrap(),
            Value::bool(true)
        ));
        assert!(matches!(
            eval_expr("(:wat::core::< 3 2)").unwrap(),
            Value::bool(false)
        ));
    }

    // ─── Boolean ────────────────────────────────────────────────────────

    #[test]
    fn and_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::and true false true)").unwrap(),
            Value::bool(false)
        ));
    }

    #[test]
    fn or_short_circuits() {
        assert!(matches!(
            eval_expr("(:wat::core::or false false true false)").unwrap(),
            Value::bool(true)
        ));
    }

    #[test]
    fn not_bool() {
        assert!(matches!(
            eval_expr("(:wat::core::not true)").unwrap(),
            Value::bool(false)
        ));
    }

    // ─── Control flow ───────────────────────────────────────────────────

    #[test]
    fn if_true_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if true 1 2)").unwrap(),
            Value::i64(1)
        ));
    }

    #[test]
    fn if_false_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if false 1 2)").unwrap(),
            Value::i64(2)
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
            Value::i64(5)
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
            Value::i64(100)
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
        assert!(matches!(result, Value::i64(42)));
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
        assert!(matches!(result, Value::i64(120)));
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
        assert!(matches!(result, Value::i64(7)));
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
        assert!(matches!(result, Value::i64(15)));
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
        assert!(matches!(result, Value::i64(101)));
    }

    // ─── Algebra-core runtime construction ──────────────────────────────

    #[test]
    fn algebra_atom_from_literal() {
        let v = eval_expr(r#"(:wat::algebra::Atom "role")"#).unwrap();
        assert!(matches!(v, Value::holon__HolonAST(_)));
    }

    #[test]
    fn algebra_atom_from_bound_variable() {
        // (Atom x) where x is a let-bound integer — runtime construction.
        let v = eval_expr(
            r#"(:wat::core::let (((x :i64) 42)) (:wat::algebra::Atom x))"#,
        )
        .unwrap();
        match v {
            Value::holon__HolonAST(h) => {
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
        assert!(matches!(v, Value::holon__HolonAST(_)));
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
        assert!(matches!(v, Value::holon__HolonAST(_)));
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
        assert!(matches!(v, Value::holon__HolonAST(_)));
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
            (:wat::core::define (:my::app::encode-pair (a :String) (b :String) -> :holon::HolonAST)
              (:wat::algebra::Bind
                (:wat::algebra::Atom a)
                (:wat::algebra::Atom b)))
            (:my::app::encode-pair "role" "filler")
            "#,
        )
        .unwrap();
        assert!(matches!(result, Value::holon__HolonAST(_)));
    }

    // ─── Four eval forms (wat-source callable) ──────────────────────────

    /// Helper: run a program with a pre-bound `program` local holding
    /// a `Value::Ast` — simulates a caller that parsed or extracted
    /// the AST before passing it to `eval-ast!`.
    fn run_with_ast_local(
        body: &str,
        ast_to_bind: WatAST,
    ) -> Result<Value, RuntimeError> {
        let form = parse_one(body).expect("parse body");
        let env = Environment::new().child().bind(
            "program",
            Value::wat__WatAST(Arc::new(ast_to_bind)),
        ).build();
        eval(&form, &env, &SymbolTable::new())
    }

    #[test]
    fn eval_ast_bang_runs_a_parsed_program() {
        let program = parse_one("(:wat::core::+ 40 2)").unwrap();
        let result =
            run_with_ast_local("(:wat::core::eval-ast! program)", program).unwrap();
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn eval_ast_bang_refuses_mutation_form() {
        let program = parse_one(
            r#"(:wat::core::define (:evil (x :i64) -> :i64) x)"#,
        )
        .unwrap();
        let err = run_with_ast_local("(:wat::core::eval-ast! program)", program)
            .unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_ast_bang_rejects_non_ast_value() {
        // Binding a string as program; eval-ast! refuses because it
        // only accepts Value::Ast (not Value::String).
        let form = parse_one(r#"(:wat::core::eval-ast! "oops")"#).unwrap();
        let err = eval(&form, &Environment::new(), &SymbolTable::new()).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::TypeMismatch { op, expected: "Ast", got: "String" }
                if op == ":wat::core::eval-ast!"
        ));
    }

    // ─── Programs-as-atoms roundtrip ────────────────────────────────────
    //
    // quote + Atom + atom-value + Bind self-inverse — the substrate
    // claim made executable. A wat program is captured as data, atomized,
    // passed through Bind/unbind, extracted, and evaluated.

    #[test]
    fn quote_captures_unevaluated_ast() {
        // (quote (+ 1 2)) returns a WatAST; does NOT evaluate the +.
        let result =
            eval_expr("(:wat::core::quote (:wat::core::+ 1 2))").unwrap();
        match result {
            Value::wat__WatAST(ast) => {
                // The captured AST should be a List whose head is :wat::core::+
                match &*ast {
                    WatAST::List(items) => {
                        assert!(matches!(
                            items.first(),
                            Some(WatAST::Keyword(k)) if k == ":wat::core::+"
                        ));
                    }
                    other => panic!("expected List AST, got {:?}", other),
                }
            }
            other => panic!("expected Value::wat__WatAST, got {:?}", other),
        }
    }

    #[test]
    fn quote_arity_mismatch() {
        let err = eval_expr("(:wat::core::quote 1 2)").unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::ArityMismatch { op, expected: 1, got: 2 }
                if op == ":wat::core::quote"
        ));
    }

    #[test]
    fn atom_wraps_quoted_program() {
        // (Atom (quote (+ 1 2))) — program becomes a holon.
        let result = eval_expr(
            "(:wat::algebra::Atom (:wat::core::quote (:wat::core::+ 1 2)))",
        )
        .unwrap();
        assert!(matches!(result, Value::holon__HolonAST(_)));
    }

    #[test]
    fn atom_value_recovers_string() {
        let result = eval_expr(
            r#"(:wat::core::atom-value (:wat::algebra::Atom "hello"))"#,
        )
        .unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "hello"),
            other => panic!("expected Value::String, got {:?}", other),
        }
    }

    #[test]
    fn atom_value_recovers_quoted_program() {
        // Atom(quote X) → atom-value back to WatAST X.
        let result = eval_expr(
            "(:wat::core::atom-value (:wat::algebra::Atom (:wat::core::quote (:wat::core::+ 40 2))))",
        )
        .unwrap();
        match result {
            Value::wat__WatAST(ast) => match &*ast {
                WatAST::List(items) => {
                    assert!(matches!(
                        items.first(),
                        Some(WatAST::Keyword(k)) if k == ":wat::core::+"
                    ));
                }
                other => panic!("expected List AST, got {:?}", other),
            },
            other => panic!("expected Value::wat__WatAST, got {:?}", other),
        }
    }

    #[test]
    fn atom_value_refuses_non_atom_holon() {
        // Bind(Atom, Atom) is not an Atom — atom-value refuses.
        let err = eval_expr(
            r#"(:wat::core::atom-value
                 (:wat::algebra::Bind
                   (:wat::algebra::Atom "a")
                   (:wat::algebra::Atom "b")))"#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::TypeMismatch { op, .. } if op == ":wat::core::atom-value"
        ));
    }

    #[test]
    fn bind_always_constructs_tree() {
        // Bind never reduces at the AST level — even when the pattern would
        // be self-inverse at the vector level. The structure stays; the
        // vector is where the self-inverse shows up via cosine. Per 058-024
        // rejection text + FOUNDATION 1718 (presence is measurement).
        let result = eval_expr(
            r#"(:wat::algebra::Bind
                 (:wat::algebra::Bind
                   (:wat::algebra::Atom "key")
                   (:wat::algebra::Atom "program"))
                 (:wat::algebra::Atom "key"))"#,
        )
        .unwrap();
        match result {
            Value::holon__HolonAST(h) => {
                // Must be a Bind tree, NOT reduced to the "program" atom.
                assert!(matches!(&*h, HolonAST::Bind(_, _)));
            }
            other => panic!("expected Bind holon, got {:?}", other),
        }
    }

    #[test]
    fn programs_as_atoms_structural_roundtrip() {
        // The structural side of programs-as-atoms: quote captures a
        // WatAST; Atom wraps it; atom-value unwraps it; eval-ast! runs
        // it. No Bind / unbind in this path — that's the vector-side
        // proof, which needs presence (added separately).
        let result = eval_expr(
            r#"(:wat::core::let*
                 (((program :wat::WatAST)
                    (:wat::core::quote (:wat::core::+ 40 2)))
                  ((program-atom :holon::HolonAST)
                    (:wat::algebra::Atom program))
                  ((reveal :wat::WatAST)
                    (:wat::core::atom-value program-atom)))
                 (:wat::core::eval-ast! reveal))"#,
        )
        .unwrap();
        assert!(matches!(result, Value::i64(42)));
    }

    // ─── Presence measurement (FOUNDATION 1718) ─────────────────────────
    //
    // The vector-level proof that `Bind(k, p)` obscures `p` in the
    // composite vector, and that the self-inverse Bind-on-Bind recovers
    // it. The algebra's retrieval primitive: cosine between encoded
    // holons, scalar output, caller binarizes.

    /// Build a SymbolTable with an EncodingCtx attached — mirrors what
    /// `FrozenWorld::freeze` does. Needed for tests exercising presence
    /// or config accessors without running the full startup pipeline.
    fn test_sym_with_ctx(dims: usize) -> SymbolTable {
        let cfg = Config {
            dims,
            capacity_mode: crate::config::CapacityMode::Error,
            global_seed: 42,
            noise_floor: 5.0 / (dims as f64).sqrt(),
        };
        let mut sym = SymbolTable::new();
        sym.set_encoding_ctx(Arc::new(EncodingCtx::from_config(&cfg)));
        sym
    }

    fn eval_with_ctx(src: &str, dims: usize) -> Result<Value, RuntimeError> {
        let ast = parse_one(src).expect("parse ok");
        let sym = test_sym_with_ctx(dims);
        eval(&ast, &Environment::new(), &sym)
    }

    #[test]
    fn presence_of_atom_in_itself_is_one() {
        let result = eval_with_ctx(
            r#"(:wat::core::presence
                 (:wat::algebra::Atom "hello")
                 (:wat::algebra::Atom "hello"))"#,
            1024,
        )
        .unwrap();
        match result {
            Value::f64(x) => assert!((x - 1.0).abs() < 1e-9, "expected ≈1.0, got {}", x),
            other => panic!("expected f64, got {:?}", other),
        }
    }

    #[test]
    fn presence_requires_encoding_ctx() {
        // Without a frozen SymbolTable, presence must error — can't
        // reach into encoding machinery that doesn't exist.
        let ast = parse_one(
            r#"(:wat::core::presence
                 (:wat::algebra::Atom "a")
                 (:wat::algebra::Atom "b"))"#,
        )
        .unwrap();
        let err = eval(&ast, &Environment::new(), &SymbolTable::new()).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::NoEncodingCtx { op } if op == ":wat::core::presence"
        ));
    }

    #[test]
    fn bind_obscures_child_at_vector_level() {
        // Core claim: cosine(encode(p), encode(Bind(k, p))) is near zero —
        // MAP bind orthogonalizes. The presence of p in Bind(k,p) is
        // below the substrate noise floor.
        let result = eval_with_ctx(
            r#"(:wat::core::let*
                 (((program :holon::HolonAST) (:wat::algebra::Atom "the-program"))
                  ((key :holon::HolonAST) (:wat::algebra::Atom "the-key"))
                  ((bound :holon::HolonAST) (:wat::algebra::Bind key program)))
                 (:wat::core::presence program bound))"#,
            1024,
        )
        .unwrap();
        let noise_floor = 5.0 / (1024f64).sqrt(); // ≈ 0.156
        match result {
            Value::f64(x) => {
                // Cosine is ternary-vector small, well below the 5σ floor.
                assert!(
                    x < noise_floor,
                    "expected presence below noise floor {}, got {}",
                    noise_floor,
                    x
                );
            }
            other => panic!("expected f64, got {:?}", other),
        }
    }

    #[test]
    fn bind_on_bind_recovers_child_at_vector_level() {
        // Self-inverse: cosine(encode(p), encode(Bind(Bind(k,p), k))) is
        // well above the noise floor. MAP's bind(bind(k,p), k) ≈ p on
        // non-zero positions of k.
        let result = eval_with_ctx(
            r#"(:wat::core::let*
                 (((program :holon::HolonAST) (:wat::algebra::Atom "the-program"))
                  ((key :holon::HolonAST) (:wat::algebra::Atom "the-key"))
                  ((bound :holon::HolonAST) (:wat::algebra::Bind key program))
                  ((recovered :holon::HolonAST) (:wat::algebra::Bind bound key)))
                 (:wat::core::presence program recovered))"#,
            1024,
        )
        .unwrap();
        let noise_floor = 5.0 / (1024f64).sqrt();
        match result {
            Value::f64(x) => {
                assert!(
                    x > noise_floor,
                    "expected presence above noise floor {}, got {}",
                    noise_floor,
                    x
                );
            }
            other => panic!("expected f64, got {:?}", other),
        }
    }

    #[test]
    fn config_noise_floor_accessor_returns_derived_value() {
        let result = eval_with_ctx("(:wat::config::noise-floor)", 10000).unwrap();
        let expected = 5.0 / 100.0; // = 0.05
        match result {
            Value::f64(x) => assert!((x - expected).abs() < 1e-12),
            other => panic!("expected f64, got {:?}", other),
        }
    }

    #[test]
    fn config_dims_accessor_returns_committed_value() {
        let result = eval_with_ctx("(:wat::config::dims)", 4096).unwrap();
        assert!(matches!(result, Value::i64(4096)));
    }

    #[test]
    fn eval_edn_bang_inline_string_runs() {
        let result = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::string "(:wat::core::+ 40 2)")"#,
        )
        .unwrap();
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn eval_edn_bang_unknown_iface_refused() {
        let err = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::unknown "foo")"#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn eval_edn_bang_reserved_unimplemented_iface_refused() {
        let err = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::http-path "https://example.com/x")"#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn eval_edn_bang_refuses_mutation_inside_string() {
        // The parsed AST from the string still walks through the
        // mutation-form guard.
        let err = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::string "(:wat::core::define (:evil (x :i64) -> :i64) x)")"#,
        )
        .unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_digest_bang_valid_hex_runs() {
        use sha2::Digest as _;
        let source = r#"(:wat::core::+ 1 1)"#;
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let form = format!(
            r#"(:wat::core::eval-digest!
                :wat::eval::string "{}"
                :wat::verify::digest-sha256
                :wat::verify::string "{}")"#,
            source, hex
        );
        let result = eval_expr(&form).unwrap();
        assert!(matches!(result, Value::i64(2)));
    }

    #[test]
    fn eval_digest_bang_mismatch_refused() {
        let wrong =
            "0000000000000000000000000000000000000000000000000000000000000000";
        let form = format!(
            r#"(:wat::core::eval-digest!
                :wat::eval::string "(:wat::core::+ 1 1)"
                :wat::verify::digest-sha256
                :wat::verify::string "{}")"#,
            wrong
        );
        let err = eval_expr(&form).unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(err, crate::hash::HashError::Mismatch { .. }));
            }
            other => panic!("expected EvalVerificationFailed, got {:?}", other),
        }
    }

    #[test]
    fn eval_digest_bang_unknown_algo_refused() {
        let form = r#"(:wat::core::eval-digest!
            :wat::eval::string "(:wat::core::+ 1 1)"
            :wat::verify::signed-ed25519
            :wat::verify::string "abc")"#;
        let err = eval_expr(form).unwrap_err();
        // signed-ed25519 in a digest slot is a grammar error.
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn eval_signed_bang_valid_sig_runs() {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let source = r#"(:wat::core::+ 20 22)"#;
        let sk = SigningKey::from_bytes(&[17u8; 32]);
        let forms = parse_all(source).unwrap();
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = sk.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(sk.verifying_key().as_bytes());
        let form = format!(
            r#"(:wat::core::eval-signed!
                :wat::eval::string "{}"
                :wat::verify::signed-ed25519
                :wat::verify::string "{}"
                :wat::verify::string "{}")"#,
            source, sig_b64, pk_b64
        );
        let result = eval_expr(&form).unwrap();
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn eval_signed_bang_tampered_source_refused() {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let signed_source = r#"(:wat::core::+ 20 22)"#;
        let tampered_source = r#"(:wat::core::+ 99 99)"#;
        let sk = SigningKey::from_bytes(&[17u8; 32]);
        let forms = parse_all(signed_source).unwrap();
        let hash = crate::hash::hash_canonical_program(&forms);
        let sig = sk.sign(&hash);
        let sig_b64 = B64.encode(sig.to_bytes());
        let pk_b64 = B64.encode(sk.verifying_key().as_bytes());
        let form = format!(
            r#"(:wat::core::eval-signed!
                :wat::eval::string "{}"
                :wat::verify::signed-ed25519
                :wat::verify::string "{}"
                :wat::verify::string "{}")"#,
            tampered_source, sig_b64, pk_b64
        );
        let err = eval_expr(&form).unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(err, crate::hash::HashError::SignatureMismatch { .. }));
            }
            other => panic!("expected SignatureMismatch, got {:?}", other),
        }
    }

    #[test]
    fn eval_signed_bang_wrong_algo_kind_refused() {
        // digest-sha256 in a signed slot is a grammar error.
        let form = r#"(:wat::core::eval-signed!
            :wat::eval::string "(:wat::core::+ 1 1)"
            :wat::verify::digest-sha256
            :wat::verify::string "sig"
            :wat::verify::string "pk")"#;
        let err = eval_expr(form).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    // ─── File-path interface (real runtime I/O) ─────────────────────────

    fn write_temp(contents: &str, suffix: &str) -> std::path::PathBuf {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "wat-eval-test-{}-{}.{}",
            std::process::id(),
            // Unique per test via a nanos timestamp.
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            suffix
        ));
        let mut f = std::fs::File::create(&path).expect("create temp");
        f.write_all(contents.as_bytes()).expect("write");
        path
    }

    #[test]
    fn eval_edn_bang_file_path_runs() {
        let path = write_temp("(:wat::core::+ 10 11)", "wat");
        let form = format!(
            r#"(:wat::core::eval-edn! :wat::eval::file-path "{}")"#,
            path.display()
        );
        let result = eval_expr(&form).expect("eval");
        let _ = std::fs::remove_file(&path);
        assert!(matches!(result, Value::i64(21)));
    }

    #[test]
    fn eval_edn_bang_file_path_missing_errors() {
        let form = r#"(:wat::core::eval-edn! :wat::eval::file-path "/nonexistent/path/abc.xyz")"#;
        let err = eval_expr(form).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn eval_digest_bang_sidecar_file_runs() {
        use sha2::Digest as _;
        let source = "(:wat::core::* 6 7)";
        let source_path = write_temp(source, "wat");
        let mut hasher = sha2::Sha256::new();
        hasher.update(source.as_bytes());
        let hex = crate::hash::hex_encode(&hasher.finalize());
        let digest_path = write_temp(&hex, "sha256");
        let form = format!(
            r#"(:wat::core::eval-digest!
                :wat::eval::file-path "{}"
                :wat::verify::digest-sha256
                :wat::verify::file-path "{}")"#,
            source_path.display(),
            digest_path.display()
        );
        let result = eval_expr(&form).expect("eval");
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&digest_path);
        assert!(matches!(result, Value::i64(42)));
    }
}
