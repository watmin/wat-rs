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

/// Kernel-owned stop flag read by `(:wat::kernel::stopped?)`.
///
/// The wat-vm binary installs OS signal handlers for SIGINT and
/// SIGTERM; both set this flag to `true`. User programs poll via the
/// `:wat::kernel::stopped?` form to decide whether to continue their
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
/// `(:wat::kernel::stopped?)` will observe it and can begin clean
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

/// Non-terminal user-signal flags — SIGUSR1, SIGUSR2, SIGHUP. Per the
/// 2026-04-19 signal-model stance: the kernel MEASURES; userland owns
/// the transitions. OS signal handlers set these true; wat programs
/// poll via `(:wat::kernel::sigusr1?)` / `(sigusr2?)` / `(sighup?)`
/// and clear via the matching `reset-*!` primitive.
///
/// Unlike [`KERNEL_STOPPED`] (terminal, set-once), these flags are
/// designed to be flipped back to `false` from userland. The boolean
/// is coalesced — five SIGHUPs in a burst read as one "yes" on the
/// next poll. Callers that need counter semantics build that in
/// userland.
pub static KERNEL_SIGUSR1: AtomicBool = AtomicBool::new(false);
pub static KERNEL_SIGUSR2: AtomicBool = AtomicBool::new(false);
pub static KERNEL_SIGHUP: AtomicBool = AtomicBool::new(false);

/// Set the SIGUSR1 flag. Called by the OS signal handler.
pub fn set_kernel_sigusr1() {
    KERNEL_SIGUSR1.store(true, Ordering::SeqCst);
}

/// Set the SIGUSR2 flag. Called by the OS signal handler.
pub fn set_kernel_sigusr2() {
    KERNEL_SIGUSR2.store(true, Ordering::SeqCst);
}

/// Set the SIGHUP flag. Called by the OS signal handler.
pub fn set_kernel_sighup() {
    KERNEL_SIGHUP.store(true, Ordering::SeqCst);
}

/// Reset all user-signal flags. Test-only — production uses the per-flag
/// `reset-*!` wat primitives.
#[cfg(test)]
pub fn reset_user_signals() {
    KERNEL_SIGUSR1.store(false, Ordering::SeqCst);
    KERNEL_SIGUSR2.store(false, Ordering::SeqCst);
    KERNEL_SIGHUP.store(false, Ordering::SeqCst);
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
    /// A channel sender handle. Carries `Value` — any wat runtime
    /// value can travel through a queue. The variant encodes the full
    /// `crossbeam_channel::Sender` path; wat takes a direct dep on
    /// `crossbeam-channel` and does not hide it. Type-level
    /// parameterization (`Sender<T>`) lives in the type checker; the
    /// runtime transports `Value` generically.
    crossbeam_channel__Sender(Arc<crossbeam_channel::Sender<Value>>),
    /// A channel receiver handle. Carries `Value`; see `Sender`.
    crossbeam_channel__Receiver(Arc<crossbeam_channel::Receiver<Value>>),
    /// An `:Option<T>` value — `:None` or `(Some v)`. Built-in
    /// parametric enum per 058-030; used as the return type of
    /// `:wat::kernel::recv` / `try-recv` / `select` and of structural
    /// retrieval (`get` on HashMap/Vec/HashSet). The `std::option::Option`
    /// here is the Rust host's own Option — wat's `:Option<T>`
    /// compiles to it directly.
    Option(Arc<std::option::Option<Value>>),
    /// An `n`-tuple — `:(T1,T2,...,Tn)`. Distinct from [`Value::Vec`]
    /// at the type level (heterogeneous vs homogeneous). Primarily
    /// produced by kernel primitives that return pairs
    /// (`make-bounded-queue`, `make-unbounded-queue`, `spawn`,
    /// `select`) and destructured in `let` / `let*` via the
    /// `((a b ...) rhs)` binder shape. The unit type `:()` stays on
    /// [`Value::Unit`] — tuples start at arity 1.
    Tuple(Arc<Vec<Value>>),
    /// A spawned program's handle — `:ProgramHandle<R>` per
    /// FOUNDATION. Returned by `:wat::kernel::spawn`; consumed by
    /// `:wat::kernel::join` which blocks until the program exits and
    /// yields its final `R` value. Structurally a one-shot result
    /// channel: the spawned thread sends its `Result<Value, _>` on
    /// the receiver end once; `join` does `recv`. No Mutex — the
    /// channel itself is the synchronization. If the thread panics
    /// before sending, the sender drops, and `join` reports the
    /// panic via `ChannelDisconnected`.
    wat__kernel__ProgramHandle(Arc<crossbeam_channel::Receiver<Result<Value, RuntimeError>>>),
    /// A claim-or-panic handle pool — `:HandlePool<T>` per FOUNDATION.
    /// Backing: a bounded crossbeam channel pre-filled with N handles
    /// and its sender dropped immediately, so `is_empty` means the
    /// pool has been fully drained. No Mutex — crossbeam's channel
    /// primitives handle the concurrent `pop` calls lock-free.
    /// `name` surfaces in error messages when a pop from empty or a
    /// finish with orphans fires.
    wat__kernel__HandlePool {
        name: Arc<String>,
        rx: Arc<crossbeam_channel::Receiver<Value>>,
    },
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
            Value::Option(_) => "Option",
            Value::Tuple(_) => "tuple",
            Value::wat__kernel__ProgramHandle(_) => "wat::kernel::ProgramHandle",
            Value::wat__kernel__HandlePool { .. } => "wat::kernel::HandlePool",
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
/// `:wat::algebra::cosine` (FOUNDATION 1718), which measure cosine
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
#[derive(Debug, Default, Clone)]
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
    /// primitives that require encoding (`:wat::algebra::cosine`) call
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
    /// A `:wat::kernel::send` on a channel whose receiver has been
    /// dropped. `recv` itself no longer errors on disconnect — it
    /// returns `:None` per FOUNDATION's `∀T. Receiver<T> -> Option<T>`
    /// shape — so the only surviving producer of this variant is
    /// send-after-disconnect.
    ChannelDisconnected { op: String },
    /// A vector-level primitive (`:wat::algebra::cosine`,
    /// `:wat::config::noise-floor`, etc.) was invoked but the
    /// [`SymbolTable`] has no attached [`EncodingCtx`]. Reachable from
    /// test harnesses that don't go through freeze; the frozen startup
    /// pipeline always installs one.
    NoEncodingCtx { op: String },
    /// A `(:wat::core::match scrutinee ...)` ran with no arm whose
    /// pattern matches the scrutinee's shape. Exhaustiveness is the
    /// type checker's job; this variant fires only when the check was
    /// bypassed or hasn't caught up with a new pattern form.
    PatternMatchFailed { value_type: &'static str },
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
                "{}: channel disconnected — receiver was dropped. `recv` is now Option-returning (disconnect yields :None); only `send` to a dropped receiver raises this error.",
                op
            ),
            RuntimeError::NoEncodingCtx { op } => write!(
                f,
                "{}: no encoding context attached to SymbolTable; presence / config accessors need a frozen EncodingCtx. Call via the freeze pipeline rather than a bare SymbolTable::new().",
                op
            ),
            RuntimeError::PatternMatchFailed { value_type } => write!(
                f,
                ":wat::core::match: no arm matched scrutinee of type {}; exhaustiveness should be caught at type-check time",
                value_type
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

/// Stdlib-registration variant of [`register_defines`] that bypasses
/// the reserved-prefix check. Called by the startup pipeline on the
/// baked stdlib sources; user source still goes through
/// [`register_defines`] where the prefix check blocks mis-namespaced
/// user defines.
pub fn register_stdlib_defines(
    forms: Vec<WatAST>,
    sym: &mut SymbolTable,
) -> Result<Vec<WatAST>, RuntimeError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_define_form(&form) {
            let (path, func) = parse_define_form(form)?;
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
        WatAST::Keyword(k) => {
            // `:None` is the nullary constructor of the built-in
            // `:Option<T>` enum (058-030). Special-cased here so users
            // can write `:None` in expression position to produce
            // `Value::Option(None)` without requiring a keyword-path
            // call form.
            if k == ":None" {
                return Ok(Value::Option(Arc::new(None)));
            }
            Ok(Value::wat__core__keyword(Arc::new(k.clone())))
        }
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
        WatAST::Symbol(ident) if ident.as_str() == "Some" => eval_some_ctor(rest, env, sym),
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
        ":wat::core::match" => eval_match(args, env, sym),
        ":wat::core::first" => {
            eval_positional_accessor(args, env, sym, ":wat::core::first", 0)
        }
        ":wat::core::second" => {
            eval_positional_accessor(args, env, sym, ":wat::core::second", 1)
        }
        ":wat::core::third" => {
            eval_positional_accessor(args, env, sym, ":wat::core::third", 2)
        }
        ":wat::core::rest" => eval_vec_rest(args, env, sym),
        ":wat::std::list::map-with-index" => eval_list_map_with_index(args, env, sym),

        // Integer arithmetic — strict i64. No promotion from f64.
        ":wat::core::i64::+" => eval_i64_arith(head, args, env, sym, |a, b| Ok(a + b)),
        ":wat::core::i64::-" => eval_i64_arith(head, args, env, sym, |a, b| Ok(a - b)),
        ":wat::core::i64::*" => eval_i64_arith(head, args, env, sym, |a, b| Ok(a * b)),
        ":wat::core::i64::/" => eval_i64_arith(head, args, env, sym, |a, b| {
            if b == 0 {
                Err(RuntimeError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }),
        // Float arithmetic — strict f64. No promotion from i64.
        ":wat::core::f64::+" => eval_f64_arith(head, args, env, sym, |a, b| Ok(a + b)),
        ":wat::core::f64::-" => eval_f64_arith(head, args, env, sym, |a, b| Ok(a - b)),
        ":wat::core::f64::*" => eval_f64_arith(head, args, env, sym, |a, b| Ok(a * b)),
        ":wat::core::f64::/" => eval_f64_arith(head, args, env, sym, |a, b| {
            if b == 0.0 {
                Err(RuntimeError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }),

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
        ":wat::core::list" => eval_list_ctor(args, env, sym),
        ":wat::core::length" => eval_vec_length(args, env, sym),
        ":wat::core::empty?" => eval_vec_empty(args, env, sym),
        ":wat::core::reverse" => eval_vec_reverse(args, env, sym),
        ":wat::core::range" => eval_vec_range(args, env, sym),
        ":wat::core::take" => eval_vec_take(args, env, sym),
        ":wat::core::drop" => eval_vec_drop(args, env, sym),
        ":wat::core::map" => eval_vec_map(args, env, sym),
        ":wat::core::foldl" => eval_vec_foldl(args, env, sym),
        ":wat::std::list::window" => eval_list_window(args, env, sym),

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
        ":wat::algebra::cosine" => eval_algebra_cosine(args, env, sym),
        ":wat::algebra::presence?" => eval_algebra_presence_q(args, env, sym),
        ":wat::algebra::dot" => eval_algebra_dot(args, env, sym),

        // Constrained runtime eval — four forms, matching the load
        // pipeline's discipline on source interface and verification.
        ":wat::core::eval-ast!" => eval_form_ast(args, env, sym),
        ":wat::core::eval-edn!" => eval_form_edn(args, env, sym),
        ":wat::core::eval-digest!" => eval_form_digest(args, env, sym),
        ":wat::core::eval-signed!" => eval_form_signed(args, env, sym),

        // Kernel primitives — channel IO + stop flag + user signals.
        ":wat::kernel::stopped?" => eval_kernel_stopped(args),
        ":wat::kernel::send" => eval_kernel_send(args, env, sym),
        ":wat::kernel::recv" => eval_kernel_recv(args, env, sym),
        ":wat::kernel::try-recv" => eval_kernel_try_recv(args, env, sym),
        ":wat::kernel::drop" => eval_kernel_drop(args, env, sym),
        ":wat::kernel::spawn" => eval_kernel_spawn(args, env, sym),
        ":wat::kernel::join" => eval_kernel_join(args, env, sym),
        ":wat::kernel::select" => eval_kernel_select(args, env, sym),
        ":wat::kernel::HandlePool::new" => eval_handle_pool_new(args, env, sym),
        ":wat::kernel::HandlePool::pop" => eval_handle_pool_pop(args, env, sym),
        ":wat::kernel::HandlePool::finish" => eval_handle_pool_finish(args, env, sym),
        ":wat::kernel::make-bounded-queue" => eval_make_bounded_queue(args, env, sym),
        ":wat::kernel::make-unbounded-queue" => eval_make_unbounded_queue(args),
        ":wat::kernel::sigusr1?" => {
            eval_user_signal_query(args, ":wat::kernel::sigusr1?", &KERNEL_SIGUSR1)
        }
        ":wat::kernel::sigusr2?" => {
            eval_user_signal_query(args, ":wat::kernel::sigusr2?", &KERNEL_SIGUSR2)
        }
        ":wat::kernel::sighup?" => {
            eval_user_signal_query(args, ":wat::kernel::sighup?", &KERNEL_SIGHUP)
        }
        ":wat::kernel::reset-sigusr1!" => {
            eval_user_signal_reset(args, ":wat::kernel::reset-sigusr1!", &KERNEL_SIGUSR1)
        }
        ":wat::kernel::reset-sigusr2!" => {
            eval_user_signal_reset(args, ":wat::kernel::reset-sigusr2!", &KERNEL_SIGUSR2)
        }
        ":wat::kernel::reset-sighup!" => {
            eval_user_signal_reset(args, ":wat::kernel::reset-sighup!", &KERNEL_SIGHUP)
        }

        // Config accessors — read committed config fields at runtime.
        ":wat::config::dims" => eval_config_dims(args, sym),
        ":wat::config::global-seed" => eval_config_global_seed(args, sym),
        ":wat::config::noise-floor" => eval_config_noise_floor(args, sym),

        // Stdlib math — single-method Rust calls packaged at
        // :wat::std::math::* per FOUNDATION-CHANGELOG 2026-04-18.
        // Not at :wat::core:: because they're numeric utilities, not
        // Lisp or algebra primitives; only stdlib macros (Log, Circular)
        // need them, and userland picks them up the same way.
        ":wat::std::math::ln" => eval_math_unary(args, env, sym, "ln", f64::ln),
        ":wat::std::math::log" => eval_math_unary(args, env, sym, "log", f64::ln),
        ":wat::std::math::sin" => eval_math_unary(args, env, sym, "sin", f64::sin),
        ":wat::std::math::cos" => eval_math_unary(args, env, sym, "cos", f64::cos),
        ":wat::std::math::pi" => eval_math_pi(args),

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
        let binding = parse_let_binding(pair)?;
        match binding {
            LetBinding::Single { name, rhs, .. } => {
                // Runtime ignores the declared type — the type checker
                // already validated it. Parsing it above enforced that
                // it exists (typed-let discipline).
                let value = eval(rhs, env, sym)?; // eval in OUTER env, not cumulative let*
                builder = builder.bind(name, value);
            }
            LetBinding::Destructure { names, rhs } => {
                let value = eval(rhs, env, sym)?;
                let elements = destructure_tuple(&value, names.len(), ":wat::core::let")?;
                for (name, elem) in names.into_iter().zip(elements.into_iter()) {
                    builder = builder.bind(name, elem);
                }
            }
        }
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
        let binding = parse_let_binding(pair)?;
        match binding {
            LetBinding::Single { name, rhs, .. } => {
                let value = eval(rhs, &scope, sym)?;
                scope = scope.child().bind(name, value).build();
            }
            LetBinding::Destructure { names, rhs } => {
                let value = eval(rhs, &scope, sym)?;
                let elements = destructure_tuple(&value, names.len(), ":wat::core::let*")?;
                let mut builder = scope.child();
                for (name, elem) in names.into_iter().zip(elements.into_iter()) {
                    builder = builder.bind(name, elem);
                }
                scope = builder.build();
            }
        }
    }
    eval(body, &scope, sym)
}

/// Verify `value` is a tuple of the expected arity and return its
/// elements cloned. Used by both `let` and `let*` destructure bindings.
fn destructure_tuple(
    value: &Value,
    expected_arity: usize,
    op: &str,
) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Tuple(items) => {
            if items.len() != expected_arity {
                Err(RuntimeError::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "destructure arity mismatch: binder has {} names, tuple has {} elements",
                        expected_arity,
                        items.len()
                    ),
                })
            } else {
                Ok((**items).clone())
            }
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "tuple",
            got: other.type_name(),
        }),
    }
}

/// Parse a single let binding. Per the typed-let discipline, every
/// binding is `((name :Type) rhs)` — a 2-list whose first element is
/// itself a 2-list `(name :Type)` and whose second is the RHS
/// expression. Untyped `(name rhs)` is refused.
///
/// Returns `(name, declared_type, rhs)`. Declared type is validated
/// via [`crate::types::parse_type_expr`] so `:Any` and malformed
/// type expressions are caught at this layer.
/// One let / let* binding form.
///
/// Two spec'd shapes — both honest about types. Bare-single
/// `(name rhs)` is NOT accepted: every bound name's type must be
/// derivable from a declaration somewhere in the program, not from
/// the checker guessing at a literal.
///
/// - **Single, typed** — `((name :Type) rhs)`. The canonical form.
///   Name's type is declared explicitly at the binding site.
/// - **Destructure** — `((a b c ...) rhs)`. RHS must have a declared
///   tuple return type (from a primitive or user function); each
///   binder name receives the matching tuple-element type from that
///   declaration. Structural destructure — types flow from the RHS's
///   declared shape through the pattern; no inference from literals.
enum LetBinding<'a> {
    Single {
        name: String,
        #[allow(dead_code)]
        declared_type: crate::types::TypeExpr,
        rhs: &'a WatAST,
    },
    Destructure {
        names: Vec<String>,
        rhs: &'a WatAST,
    },
}

fn parse_let_binding(pair: &WatAST) -> Result<LetBinding<'_>, RuntimeError> {
    let kv = match pair {
        WatAST::List(items) if items.len() == 2 => items,
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "each binding must be ((name :Type) rhs) or ((a b ...) rhs); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let binder = match &kv[0] {
        WatAST::List(inner) => inner,
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::let".into(),
                reason: format!(
                    "binding's binder must be a list — ((name :Type) rhs) or ((a b ...) rhs); got {}. Bare `(name rhs)` is refused: every name must have a declared type, not one inferred from a literal.",
                    ast_variant_name(other)
                ),
            });
        }
    };
    // Typed-single: `(symbol keyword)`.
    let is_typed_single = binder.len() == 2
        && matches!(&binder[0], WatAST::Symbol(_))
        && matches!(&binder[1], WatAST::Keyword(_));
    if is_typed_single {
        let name = match &binder[0] {
            WatAST::Symbol(ident) => ident.name.clone(),
            _ => unreachable!(),
        };
        let declared_type = match &binder[1] {
            WatAST::Keyword(k) => parse_type_keyword(k)?,
            _ => unreachable!(),
        };
        return Ok(LetBinding::Single {
            name,
            declared_type,
            rhs: &kv[1],
        });
    }
    // Destructure: every binder element must be a bare symbol.
    let mut names = Vec::with_capacity(binder.len());
    for item in binder {
        match item {
            WatAST::Symbol(ident) => names.push(ident.name.clone()),
            other => {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::let".into(),
                    reason: format!(
                        "destructure binder must be a list of bare symbols; got {}",
                        ast_variant_name(other)
                    ),
                });
            }
        }
    }
    if names.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::let".into(),
            reason: "destructure binder cannot be empty — at least one name is required".into(),
        });
    }
    Ok(LetBinding::Destructure {
        names,
        rhs: &kv[1],
    })
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

/// Integer arith: `:wat::core::i64::{+,-,*,/}`. Strictly i64 × i64 →
/// i64. No promotion; a f64 arg is a type error.
fn eval_i64_arith<F>(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, RuntimeError>
where
    F: Fn(i64, i64) -> Result<i64, RuntimeError>,
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
        (Value::i64(x), Value::i64(y)) => Ok(Value::i64(op(x, y)?)),
        (other, _) if !matches!(other, Value::i64(_)) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "i64",
            got: other.type_name(),
        }),
        (_, other) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "i64",
            got: other.type_name(),
        }),
    }
}

/// Float arith: `:wat::core::f64::{+,-,*,/}`. Strictly f64 × f64 →
/// f64. No promotion; an i64 arg is a type error.
fn eval_f64_arith<F>(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op: F,
) -> Result<Value, RuntimeError>
where
    F: Fn(f64, f64) -> Result<f64, RuntimeError>,
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
        (Value::f64(x), Value::f64(y)) => Ok(Value::f64(op(x, y)?)),
        (other, _) if !matches!(other, Value::f64(_)) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "f64",
            got: other.type_name(),
        }),
        (_, other) => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "f64",
            got: other.type_name(),
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

/// Require a `Vec` argument. Used by list primitives that take one
/// Vec as their sole / first arg.
fn require_vec(op: &'static str, v: Value) -> Result<Arc<Vec<Value>>, RuntimeError> {
    match v {
        Value::Vec(xs) => Ok(xs),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Vec",
            got: other.type_name(),
        }),
    }
}

/// Require an `i64` argument. Used by list primitives whose second
/// arg is a count / index.
fn require_i64(op: &'static str, v: Value) -> Result<i64, RuntimeError> {
    match v {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "i64",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::core::length xs)` → `:i64`. `xs.len() as i64`.
fn eval_vec_length(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::length".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::length", eval(&args[0], env, sym)?)?;
    Ok(Value::i64(xs.len() as i64))
}

/// `(:wat::core::empty? xs)` → `:bool`. Mirrors `slice::is_empty`.
/// Per FOUNDATION-CHANGELOG 2026-04-18: the wat replacement for
/// Scheme's `null?` (wat has no null).
fn eval_vec_empty(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::empty?".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::empty?", eval(&args[0], env, sym)?)?;
    Ok(Value::bool(xs.is_empty()))
}

/// `(:wat::core::reverse xs)` → `Vec<T>`. New Vec with elements
/// reversed; input unchanged.
fn eval_vec_reverse(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::reverse".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::reverse", eval(&args[0], env, sym)?)?;
    let mut out = (*xs).clone();
    out.reverse();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::range start end)` → `Vec<i64>`. Two-arg only; the
/// spec-frozen shape maps to Rust's `start..end` exactly. Callers
/// write `(range 0 n)` explicitly for 0..n.
fn eval_vec_range(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::range".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let start = require_i64(":wat::core::range", eval(&args[0], env, sym)?)?;
    let end = require_i64(":wat::core::range", eval(&args[1], env, sym)?)?;
    let items: Vec<Value> = if start <= end {
        (start..end).map(Value::i64).collect()
    } else {
        Vec::new()
    };
    Ok(Value::Vec(Arc::new(items)))
}

/// `(:wat::core::take xs n)` → `Vec<T>`. First `n` elements; if
/// `n >= xs.len()`, returns the full Vec. Negative `n` clamps to 0
/// (empty Vec).
fn eval_vec_take(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::take".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::take", eval(&args[0], env, sym)?)?;
    let n = require_i64(":wat::core::take", eval(&args[1], env, sym)?)?;
    let cap = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
    let out: Vec<Value> = xs.iter().take(cap).cloned().collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::drop xs n)` → `Vec<T>`. Skip first `n` elements. If
/// `n >= xs.len()`, returns an empty Vec. Negative `n` clamps to 0
/// (returns the full Vec).
fn eval_vec_drop(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::drop".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::drop", eval(&args[0], env, sym)?)?;
    let n = require_i64(":wat::core::drop", eval(&args[1], env, sym)?)?;
    let skip = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
    let out: Vec<Value> = xs.iter().skip(skip).cloned().collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::map xs f)` → `Vec<U>`. Calls `f` on each element.
/// `f` must be a callable Value (lambda or define-registered).
fn eval_vec_map(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::map".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::map", eval(&args[0], env, sym)?)?;
    let f = eval(&args[1], env, sym)?;
    let func = match &f {
        Value::wat__core__lambda(func) => func.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::map".into(),
                expected: "wat::core::lambda",
                got: other.type_name(),
            });
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for x in xs.iter() {
        out.push(apply_function(&func, vec![x.clone()], sym)?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::foldl xs init f)` → acc. `f : (acc, item) → acc`.
/// Left-associative: `f(f(f(init, x0), x1), x2)`. Sequential's
/// driver. `foldr` deferred until a caller needs it.
fn eval_vec_foldl(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::foldl".into(),
            expected: 3,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::foldl", eval(&args[0], env, sym)?)?;
    let mut acc = eval(&args[1], env, sym)?;
    let f = eval(&args[2], env, sym)?;
    let func = match &f {
        Value::wat__core__lambda(func) => func.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::foldl".into(),
                expected: "wat::core::lambda",
                got: other.type_name(),
            });
        }
    };
    for x in xs.iter() {
        acc = apply_function(&func, vec![acc, x.clone()], sym)?;
    }
    Ok(acc)
}

/// `(:wat::std::list::window xs n)` → `Vec<Vec<T>>`. Sliding window
/// of size `n`; maps to Rust's `slice.windows(n)`. `n <= 0` returns
/// an empty Vec. `n > xs.len()` returns an empty Vec (no full
/// window fits) — matches Rust's behavior.
fn eval_list_window(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::list::window".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::std::list::window", eval(&args[0], env, sym)?)?;
    let n = require_i64(":wat::std::list::window", eval(&args[1], env, sym)?)?;
    if n <= 0 {
        return Ok(Value::Vec(Arc::new(Vec::new())));
    }
    let n = n as usize;
    let out: Vec<Value> = xs
        .windows(n)
        .map(|w| Value::Vec(Arc::new(w.to_vec())))
        .collect();
    Ok(Value::Vec(Arc::new(out)))
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

/// `(:wat::core::first xs)` / `second` / `third` — positional
/// accessor polymorphic over `Vec<T>` and tuples. Both are
/// index-addressed sequences (user direction 2026-04-19: "both are
/// index-accessed data structs"). Returns the element at `index`,
/// cloned. Runtime error if the container is shorter than
/// `index + 1`.
///
/// `third` covers 3-tuples + Vecs-of-length-≥-3; higher indices go
/// through `:wat::std::get` (lands with HashMap in round 4b).
fn eval_positional_accessor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op: &'static str,
    index: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    match v {
        Value::Tuple(items) => items.get(index).cloned().ok_or_else(|| {
            RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "tuple has {} element(s); no element at index {}",
                    items.len(),
                    index
                ),
            }
        }),
        Value::Vec(items) => items.get(index).cloned().ok_or_else(|| {
            RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "Vec has {} element(s); no element at index {} (reach for :wat::std::get if empty is expected)",
                    items.len(),
                    index
                ),
            }
        }),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "tuple or Vec",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::core::rest xs)` — everything after the first element of a
/// Vec. Mirrors `slice[1..]`. Runtime error if `xs` is empty (there
/// is no `rest` of an empty sequence). Tuples do NOT support rest —
/// tuple arity is fixed at the type level.
fn eval_vec_rest(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::rest".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::rest", eval(&args[0], env, sym)?)?;
    if xs.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::rest".into(),
            reason: "cannot take rest of empty Vec".into(),
        });
    }
    let out: Vec<Value> = xs.iter().skip(1).cloned().collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::std::list::map-with-index xs f)` → `Vec<U>`. Per
/// FOUNDATION-CHANGELOG 2026-04-18 stdlib list surface. `f` takes
/// `(item, index)` and returns U. Used by Sequential's indexed fold.
fn eval_list_map_with_index(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::list::map-with-index".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::std::list::map-with-index", eval(&args[0], env, sym)?)?;
    let f = eval(&args[1], env, sym)?;
    let func = match &f {
        Value::wat__core__lambda(func) => func.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::std::list::map-with-index".into(),
                expected: "wat::core::lambda",
                got: other.type_name(),
            });
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for (i, x) in xs.iter().enumerate() {
        out.push(apply_function(
            &func,
            vec![x.clone(), Value::i64(i as i64)],
            sym,
        )?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(Some <expr>)` — tagged constructor of the built-in `:Option<T>`
/// enum (058-030). Reserved bare identifier; users cannot shadow it.
/// Arity 1. Evaluates `expr` and wraps it in `Value::Option(Some(_))`.
///
/// The dual is `:None` (keyword literal, nullary) handled directly in
/// [`eval`]. Together they are the only way to produce `Value::Option`;
/// callers consume via `(:wat::core::match ...)`.
fn eval_some_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: "Some".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    Ok(Value::Option(Arc::new(Some(v))))
}

/// `(:wat::core::match <scrutinee> <arm>...)` — pattern-match over
/// enum values. MVP-scoped to `:Option<T>` (the only built-in enum);
/// user-declared enums graduate in a later slice.
///
/// Each arm is `(pattern body)`. Pattern forms:
/// - `:None` — matches `Value::Option(None)`, no binding.
/// - `(Some binder)` — matches `Value::Option(Some(v))`, binds `binder`
///   to `v` in the body's scope. Exactly one binder; further pattern
///   nesting is a future slice.
/// - bare identifier — wildcard that binds the scrutinee as that name.
/// - `_` — wildcard, no binding.
///
/// Arms are tried in order; the first match fires. If no arm matches
/// the scrutinee, returns `PatternMatchFailed`. (Exhaustiveness is
/// enforced statically by the type checker; this runtime error fires
/// only when the type check hasn't run.)
fn eval_match(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::match".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let scrutinee = eval(&args[0], env, sym)?;
    for arm in &args[1..] {
        let arm_items = match arm {
            WatAST::List(items) => items,
            other => {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "each arm must be a list `(pattern body)`, got {}",
                        ast_variant_name(other)
                    ),
                });
            }
        };
        if arm_items.len() != 2 {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "each arm must have exactly (pattern body); got {} elements",
                    arm_items.len()
                ),
            });
        }
        let pattern = &arm_items[0];
        let body = &arm_items[1];
        if let Some(arm_env) = try_match_pattern(pattern, &scrutinee, env)? {
            return eval(body, &arm_env, sym);
        }
    }
    Err(RuntimeError::PatternMatchFailed {
        value_type: scrutinee.type_name(),
    })
}

/// Attempt to match `pattern` against `value`. Returns:
/// - `Ok(Some(env))` — pattern matches; `env` extends `outer` with any
///   pattern-introduced bindings.
/// - `Ok(None)` — pattern doesn't match this value; try the next arm.
/// - `Err(_)` — pattern is malformed.
fn try_match_pattern(
    pattern: &WatAST,
    value: &Value,
    outer: &Environment,
) -> Result<Option<Environment>, RuntimeError> {
    match pattern {
        // `:None` — matches Option(None) only.
        WatAST::Keyword(k) if k == ":None" => match value {
            Value::Option(opt) if opt.is_none() => Ok(Some(outer.clone())),
            _ => Ok(None),
        },
        // Keyword patterns other than `:None` are not yet spec'd;
        // user-enum variants graduate in a later slice.
        WatAST::Keyword(k) => Err(RuntimeError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "keyword pattern {} not supported (only `:None` is recognized in this slice)",
                k
            ),
        }),
        // `_` wildcard — matches any value, no binding.
        WatAST::Symbol(ident) if ident.as_str() == "_" => Ok(Some(outer.clone())),
        // Bare identifier — binds the scrutinee to that name.
        WatAST::Symbol(ident) => Ok(Some(
            outer.child().bind(ident.as_str().to_string(), value.clone()).build(),
        )),
        // `(Some binder)` — matches Option(Some(v)), binds `binder` to v.
        WatAST::List(items) => {
            let head = items.first().ok_or_else(|| RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "empty list pattern".into(),
            })?;
            match head {
                WatAST::Symbol(ident) if ident.as_str() == "Some" => {
                    if items.len() != 2 {
                        return Err(RuntimeError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Some binder) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        });
                    }
                    match value {
                        Value::Option(opt) => match &**opt {
                            Some(inner) => {
                                let binder = match &items[1] {
                                    WatAST::Symbol(b) => b.as_str().to_string(),
                                    other => {
                                        return Err(RuntimeError::MalformedForm {
                                            head: ":wat::core::match".into(),
                                            reason: format!(
                                                "(Some _): binder must be a bare symbol, got {}",
                                                ast_variant_name(other)
                                            ),
                                        });
                                    }
                                };
                                Ok(Some(outer.child().bind(binder, inner.clone()).build()))
                            }
                            None => Ok(None),
                        },
                        _ => Ok(None),
                    }
                }
                other => Err(RuntimeError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "list pattern head must be a variant constructor; got {}",
                        ast_variant_name(other)
                    ),
                }),
            }
        }
        other => Err(RuntimeError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "pattern must be a keyword, symbol, or list; got {}",
                ast_variant_name(other)
            ),
        }),
    }
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

/// `(:wat::algebra::cosine target reference) -> :f64` — raw cosine
/// measurement between two encoded holons. Per FOUNDATION 1718 +
/// OPEN-QUESTIONS line 419: algebra-substrate operation (input is
/// holons, not raw numbers). Sibling to `:wat::algebra::dot` — this
/// one normalizes.
///
/// Encodes both holons via the frozen [`EncodingCtx`] and returns a
/// value in `[-1, +1]`. The algebra does NOT binarize — callers that
/// want a verdict reach for [`eval_algebra_presence_q`] (alias
/// `presence?`), which compares against the committed noise floor.
fn eval_algebra_cosine(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::cosine".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let target = require_holon(":wat::algebra::cosine", eval(&args[0], env, sym)?)?;
    let reference = require_holon(":wat::algebra::cosine", eval(&args[1], env, sym)?)?;
    let ctx = require_encoding_ctx(":wat::algebra::cosine", sym)?;

    let vt = encode(&target, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vr = encode(&reference, &ctx.vm, &ctx.scalar, &ctx.registry);
    Ok(Value::f64(Similarity::cosine(&vt, &vr)))
}

/// `(:wat::algebra::presence? target reference) -> :bool` — boolean
/// verdict: is `target` present in `reference` above the 5σ noise
/// floor? Encodes both, computes cosine, returns
/// `cosine > :wat::config::noise-floor`.
///
/// The `?` suffix is the predicate convention (2026-04-19 naming
/// stance). Callers that want the raw scalar reach for
/// `:wat::algebra::cosine`.
fn eval_algebra_presence_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::presence?".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let target = require_holon(":wat::algebra::presence?", eval(&args[0], env, sym)?)?;
    let reference = require_holon(":wat::algebra::presence?", eval(&args[1], env, sym)?)?;
    let ctx = require_encoding_ctx(":wat::algebra::presence?", sym)?;

    let vt = encode(&target, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vr = encode(&reference, &ctx.vm, &ctx.scalar, &ctx.registry);
    let cosine = Similarity::cosine(&vt, &vr);
    Ok(Value::bool(cosine > ctx.config.noise_floor))
}

/// `(:wat::algebra::dot x y) -> :f64` — scalar dot product of two
/// encoded holons. Per 058-005: measurement primitive, not a HolonAST
/// variant (scalar-out, not vector-out). Sibling to `presence`:
/// presence returns cosine (dot normalized by magnitudes); dot is the
/// raw bilinear value, used by Gram-Schmidt macros (Reject, Project)
/// that need the unnormalized coefficient.
fn eval_algebra_dot(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::algebra::dot".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let x = require_holon(":wat::algebra::dot", eval(&args[0], env, sym)?)?;
    let y = require_holon(":wat::algebra::dot", eval(&args[1], env, sym)?)?;
    let ctx = require_encoding_ctx(":wat::algebra::dot", sym)?;
    let vx = encode(&x, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vy = encode(&y, &ctx.vm, &ctx.scalar, &ctx.registry);
    Ok(Value::f64(Similarity::dot(&vx, &vy)))
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

/// `(:wat::kernel::stopped?)` — nullary predicate; returns the kernel
/// stop flag as a `:bool`. The wat-vm's signal handler sets the flag
/// on SIGINT / SIGTERM; user programs poll it in their loops.
///
/// `?` suffix per the 2026-04-19 naming-convention stance —
/// predicates end in `?`.
fn eval_kernel_stopped(args: &[WatAST]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::stopped?".into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::bool(KERNEL_STOPPED.load(Ordering::SeqCst)))
}

/// Shared body for the three user-signal predicates — nullary, reads a
/// given atomic flag. `op` is the wat-facing keyword path for error
/// messages.
fn eval_user_signal_query(
    args: &[WatAST],
    op: &str,
    flag: &AtomicBool,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::bool(flag.load(Ordering::SeqCst)))
}

/// Shared body for the three user-signal resetters — nullary, flips a
/// given atomic flag back to `false`. Unlike the terminal stop flag
/// (set-once), user-signal flags are designed to be toggled by userland
/// after the signal's condition has been handled.
fn eval_user_signal_reset(
    args: &[WatAST],
    op: &str,
    flag: &AtomicBool,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 0,
            got: args.len(),
        });
    }
    flag.store(false, Ordering::SeqCst);
    Ok(Value::Unit)
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

/// `(:wat::kernel::make-bounded-queue :T capacity)` — creates a
/// bounded crossbeam channel carrying `:T` values with the given
/// capacity. Returns a `:(Sender<T>, Receiver<T>)` 2-tuple.
///
/// The first argument is a TYPE KEYWORD — not evaluated at runtime,
/// only read for the type checker's benefit. The runtime transports
/// any `Value`; `T` lives in the scheme only. Any non-keyword first
/// argument is a structural error.
///
/// `bounded(1)` is the spec'd default rendezvous shape (FOUNDATION's
/// Pipeline Discipline rule 4).
fn eval_make_bounded_queue(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::make-bounded-queue".into(),
            expected: 2,
            got: args.len(),
        });
    }
    if !matches!(&args[0], WatAST::Keyword(_)) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::kernel::make-bounded-queue".into(),
            reason: "first argument must be a type keyword (e.g., :Candle)".into(),
        });
    }
    let capacity = match eval(&args[1], env, sym)? {
        Value::i64(n) if n >= 0 => n as usize,
        Value::i64(n) => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::kernel::make-bounded-queue".into(),
                reason: format!("capacity must be non-negative; got {}", n),
            });
        }
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::make-bounded-queue".into(),
                expected: "i64",
                got: other.type_name(),
            });
        }
    };
    let (tx, rx) = crossbeam_channel::bounded::<Value>(capacity);
    Ok(Value::Tuple(Arc::new(vec![
        Value::crossbeam_channel__Sender(Arc::new(tx)),
        Value::crossbeam_channel__Receiver(Arc::new(rx)),
    ])))
}

/// `(:wat::kernel::make-unbounded-queue :T)` — creates an unbounded
/// crossbeam channel carrying `:T` values. Returns a
/// `:(Sender<T>, Receiver<T>)` 2-tuple.
///
/// Like `make-bounded-queue` the first argument is a type keyword for
/// the checker; the runtime transports any `Value`.
fn eval_make_unbounded_queue(args: &[WatAST]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::make-unbounded-queue".into(),
            expected: 1,
            got: args.len(),
        });
    }
    if !matches!(&args[0], WatAST::Keyword(_)) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::kernel::make-unbounded-queue".into(),
            reason: "argument must be a type keyword (e.g., :LearnSignal)".into(),
        });
    }
    let (tx, rx) = crossbeam_channel::unbounded::<Value>();
    Ok(Value::Tuple(Arc::new(vec![
        Value::crossbeam_channel__Sender(Arc::new(tx)),
        Value::crossbeam_channel__Receiver(Arc::new(rx)),
    ])))
}

/// `(:wat::kernel::send sender value)` — blocks until the value is
/// accepted by the channel; returns `:()`. Type scheme
/// `∀T. crossbeam_channel::Sender<T> -> T -> :()`. The runtime
/// transports any `Value` through the channel; the type checker
/// enforces that the declared `Sender<T>` matches the value's type.
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
    let msg = eval(&args[1], env, sym)?;
    sender
        .send(msg)
        .map_err(|_| RuntimeError::ChannelDisconnected {
            op: ":wat::kernel::send".into(),
        })?;
    Ok(Value::Unit)
}

/// `(:wat::kernel::recv receiver)` — blocks until the receiver
/// produces a value or its sender is dropped. Typed
/// `∀T. Receiver<T> -> Option<T>` per FOUNDATION: `(Some v)` on a
/// successful receive, `:None` when every sender has dropped
/// (disconnect becomes first-class absence rather than an error).
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
        Ok(v) => Ok(Value::Option(Arc::new(Some(v)))),
        Err(_) => Ok(Value::Option(Arc::new(None))),
    }
}

/// `(:wat::kernel::try-recv receiver)` — non-blocking receive. Typed
/// `∀T. Receiver<T> -> :Option<T>`. Returns `(Some v)` if a value is
/// ready, `:None` if the queue is empty OR the sender has dropped.
/// Per FOUNDATION: both cases collapse to `:None` — callers that need
/// to distinguish them wrap `try-recv` + `recv` differently, or use
/// `select`.
fn eval_kernel_try_recv(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::try-recv".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let receiver = match eval(&args[0], env, sym)? {
        Value::crossbeam_channel__Receiver(r) => r,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::try-recv".into(),
                expected: "crossbeam_channel::Receiver",
                got: other.type_name(),
            });
        }
    };
    match receiver.try_recv() {
        Ok(v) => Ok(Value::Option(Arc::new(Some(v)))),
        Err(_) => Ok(Value::Option(Arc::new(None))),
    }
}

/// `(:wat::kernel::drop handle)` — declares the caller is done with a
/// sender or receiver. Typed `∀T. Sender<T> -> :()` and
/// `∀T. Receiver<T> -> :()` (two registered schemes; runtime accepts
/// either). Returns `:()`.
///
/// **Close semantics are scope-based.** Following the lab's
/// single-owner discipline, a sender/receiver is held by exactly one
/// program's let-scope; when that scope ends, the underlying
/// crossbeam handle drops and the channel-end disconnects. This
/// primitive exists as a READABILITY MARKER at the call site — "the
/// program is done with this handle" — but it does not force the
/// channel to close while other references remain. The for-each-drop
/// idiom in FOUNDATION's shutdown cascade works because the
/// enclosing let-scope ends immediately after, releasing the Vec of
/// handles that the for-each iterated over.
///
/// A proper `consume` semantic (atomic take + underlying drop) is a
/// future refactor if userland programs need it before scope-end.
fn eval_kernel_drop(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::drop".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let handle = eval(&args[0], env, sym)?;
    match handle {
        Value::crossbeam_channel__Sender(_) | Value::crossbeam_channel__Receiver(_) => {
            // Intentional no-op. The Arc we just evaluated into
            // `handle` drops here at end-of-scope, decrementing the
            // refcount by one. Close happens when the caller's
            // enclosing scope releases its own binding.
            Ok(Value::Unit)
        }
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::kernel::drop".into(),
            expected: "crossbeam_channel::Sender | crossbeam_channel::Receiver",
            got: other.type_name(),
        }),
    }
}

/// Shared implementation for the unary stdlib math calls —
/// `:wat::std::math::ln`, `log`, `sin`, `cos`. Arity 1. Argument must
/// evaluate to `:f64` (or `:i64` auto-promoted). `op_name` is the
/// wat-facing short name for error messages.
fn eval_math_unary(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op_name: &str,
    f: fn(f64) -> f64,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: format!(":wat::std::math::{}", op_name),
            expected: 1,
            got: args.len(),
        });
    }
    let x = match eval(&args[0], env, sym)? {
        Value::f64(x) => x,
        Value::i64(n) => n as f64,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: format!(":wat::std::math::{}", op_name),
                expected: "f64",
                got: other.type_name(),
            });
        }
    };
    Ok(Value::f64(f(x)))
}

/// `(:wat::std::math::pi)` — the mathematical constant π as `:f64`.
/// Nullary. Backing: `std::f64::consts::PI`.
fn eval_math_pi(args: &[WatAST]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::math::pi".into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::f64(std::f64::consts::PI))
}

/// `(:wat::kernel::HandlePool::new name handles)` — build a pool of
/// N handles of the same type. `name` surfaces in error messages; the
/// pool drains as callers `pop` and asserts empty at `finish`.
///
/// Implementation: a bounded crossbeam channel of size N pre-filled
/// with the given handles, whose sender is dropped immediately so
/// further puts are impossible. Consumers `pop` via `try_recv`;
/// `finish` checks the channel is empty. No Mutex; the channel's
/// lock-free multi-consumer semantics are the synchronization.
fn eval_handle_pool_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::HandlePool::new".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let name = match eval(&args[0], env, sym)? {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::HandlePool::new".into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };
    let handles = match eval(&args[1], env, sym)? {
        Value::Vec(v) => v,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::HandlePool::new".into(),
                expected: "Vec",
                got: other.type_name(),
            });
        }
    };
    let n = handles.len();
    // Zero-handle pools are legal — a pool with zero handles whose
    // `finish` is called immediately asserts true vacuously. Callers
    // that pre-count capacity may hit N=0 for degenerate cases.
    let (tx, rx) = crossbeam_channel::bounded::<Value>(n.max(1));
    for v in handles.iter() {
        if tx.send(v.clone()).is_err() {
            // The rx is local to this scope; send cannot fail.
            unreachable!("newly-built channel receiver must be alive");
        }
    }
    // Drop tx so the channel's is_empty discipline reads "fully
    // drained" once every handle is popped.
    drop(tx);
    Ok(Value::wat__kernel__HandlePool {
        name,
        rx: Arc::new(rx),
    })
}

/// `(:wat::kernel::HandlePool::pop pool)` — claim one handle. Returns
/// the claimed value. If the pool is empty, returns a
/// MalformedForm error naming the pool — callers are expected to
/// pop exactly the count they committed to at construction.
fn eval_handle_pool_pop(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::HandlePool::pop".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let (name, rx) = match eval(&args[0], env, sym)? {
        Value::wat__kernel__HandlePool { name, rx } => (name, rx),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::HandlePool::pop".into(),
                expected: "wat::kernel::HandlePool",
                got: other.type_name(),
            });
        }
    };
    match rx.try_recv() {
        Ok(v) => Ok(v),
        Err(_) => Err(RuntimeError::MalformedForm {
            head: ":wat::kernel::HandlePool::pop".into(),
            reason: format!(
                "{}: no handles left to claim (pool drained or mis-counted at construction)",
                name
            ),
        }),
    }
}

/// `(:wat::kernel::HandlePool::finish pool)` — assert the pool is
/// empty and return `:()`. Callers call this at the end of wiring to
/// catch orphaned handles BEFORE any thread runs. If handles remain
/// (an orphan — typically a mis-counted handle budget at
/// construction), returns a MalformedForm error naming the pool and
/// the orphan count. This is the "claim or panic" discipline from
/// FOUNDATION's Pipeline Discipline rule 2.
fn eval_handle_pool_finish(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::HandlePool::finish".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let (name, rx) = match eval(&args[0], env, sym)? {
        Value::wat__kernel__HandlePool { name, rx } => (name, rx),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::HandlePool::finish".into(),
                expected: "wat::kernel::HandlePool",
                got: other.type_name(),
            });
        }
    };
    let remaining = rx.len();
    if remaining != 0 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::kernel::HandlePool::finish".into(),
            reason: format!(
                "{}: {} orphaned handle(s) — deadlock risk (every handle must be claimed before finish)",
                name, remaining
            ),
        });
    }
    Ok(Value::Unit)
}

/// `(:wat::kernel::select receivers)` — fan-in over multiple receivers.
/// Blocks until ANY of the given receivers produces a value or
/// disconnects. Returns a 2-tuple `(index, Option<T>)` — the position
/// of the ready receiver in the input Vec, and either `(Some v)` if
/// it produced or `:None` if it disconnected.
///
/// The caller typically loops over the result, dropping disconnected
/// receivers from the Vec on `(index, :None)` and exiting when the
/// Vec is empty. No Mailbox stdlib; the select loop IS the fan-in.
///
/// Spec index type is `:usize`; wat-rs currently has no `:usize`
/// value variant, so the index surfaces as `:i64`. This is the one
/// deviation from FOUNDATION here; a follow-up slice adds `:usize`
/// when the first caller demands it.
fn eval_kernel_select(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::select".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let items = match eval(&args[0], env, sym)? {
        Value::Vec(v) => v,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::select".into(),
                expected: "Vec",
                got: other.type_name(),
            });
        }
    };
    if items.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::kernel::select".into(),
            reason: "receivers vec cannot be empty — select would block forever".into(),
        });
    }
    // Extract Arc<Receiver<Value>> for each element; error on any
    // non-receiver Value so the typed-pipe contract is visible.
    let mut rxs: Vec<Arc<crossbeam_channel::Receiver<Value>>> = Vec::with_capacity(items.len());
    for v in items.iter() {
        match v {
            Value::crossbeam_channel__Receiver(r) => rxs.push(r.clone()),
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::kernel::select".into(),
                    expected: "crossbeam_channel::Receiver",
                    got: other.type_name(),
                });
            }
        }
    }
    let mut sel = crossbeam_channel::Select::new();
    for rx in &rxs {
        sel.recv(rx.as_ref());
    }
    let oper = sel.select();
    let idx = oper.index();
    let result = oper.recv(rxs[idx].as_ref());
    let inner = match result {
        Ok(v) => Value::Option(Arc::new(Some(v))),
        Err(_) => Value::Option(Arc::new(None)),
    };
    Ok(Value::Tuple(Arc::new(vec![Value::i64(idx as i64), inner])))
}

/// `(:wat::kernel::spawn :fn::path arg1 arg2 ...)` — spawn a function
/// on its own OS thread. First argument is a keyword-path naming a
/// registered `:wat::core::define`d function; remaining args are
/// evaluated in the caller's env and passed to the spawned thread.
///
/// Returns a `:ProgramHandle<R>` — structurally an Arc'd crossbeam
/// receiver over a one-shot channel. The spawned thread runs the
/// function and sends its `Result<Value, RuntimeError>` on that
/// channel; `join` blocks for the result. No Mutex; the channel is
/// the synchronization point.
///
/// The spawned thread gets its own clone of the `SymbolTable` — a
/// shallow HashMap clone whose values are `Arc<Function>` (cheap
/// refcount bumps) plus an `Arc<EncodingCtx>` clone. Thread-local
/// access to the frozen symbol table; no shared mutation.
fn eval_kernel_spawn(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::spawn".into(),
            expected: 1, // minimum — function-path keyword
            got: 0,
        });
    }
    let fn_path = match &args[0] {
        WatAST::Keyword(k) => k.clone(),
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::kernel::spawn".into(),
                reason: "first argument must be a function keyword path (e.g., :my::app::worker)".into(),
            });
        }
    };
    let func = match sym.get(&fn_path) {
        Some(f) => f.clone(),
        None => return Err(RuntimeError::UnknownFunction(fn_path)),
    };
    let mut arg_values = Vec::with_capacity(args.len() - 1);
    for a in &args[1..] {
        arg_values.push(eval(a, env, sym)?);
    }
    let thread_sym = sym.clone();
    let (tx, rx) = crossbeam_channel::bounded::<Result<Value, RuntimeError>>(1);
    std::thread::spawn(move || {
        let result = apply_function(&func, arg_values, &thread_sym);
        let _ = tx.send(result);
    });
    Ok(Value::wat__kernel__ProgramHandle(Arc::new(rx)))
}

/// `(:wat::kernel::join handle)` — block until the spawned program
/// exits and yield its final value. Typed
/// `∀R. ProgramHandle<R> -> R`.
///
/// If the spawned thread returned a `Value`, pass it through. If it
/// raised a `RuntimeError`, propagate as if it had been raised
/// locally. If the thread panicked, `rx.recv` fails
/// (sender dropped without sending) and we report
/// `ChannelDisconnected` — the OS-level panic has already printed to
/// stderr.
fn eval_kernel_join(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::kernel::join".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let handle = match eval(&args[0], env, sym)? {
        Value::wat__kernel__ProgramHandle(rx) => rx,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::kernel::join".into(),
                expected: "wat::kernel::ProgramHandle",
                got: other.type_name(),
            });
        }
    };
    match handle.recv() {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(RuntimeError::ChannelDisconnected {
            op: ":wat::kernel::join (spawned thread panicked before yielding a result)"
                .into(),
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
            eval_expr("(:wat::core::i64::+ 2 3)").unwrap(),
            Value::i64(5)
        ));
    }

    #[test]
    fn subtract_ints() {
        assert!(matches!(
            eval_expr("(:wat::core::i64::- 10 4)").unwrap(),
            Value::i64(6)
        ));
    }

    #[test]
    fn i64_mul_refuses_f64_arg() {
        // Post-split (2026-04-19): arith is strictly typed. i64::*
        // refuses any f64 argument — no silent promotion. Users
        // commit to the numeric tier at the call site; users who
        // want float math reach for :wat::core::f64::*.
        let err = eval_expr("(:wat::core::i64::* 3 2.0)").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn f64_mul_refuses_i64_arg() {
        let err = eval_expr("(:wat::core::f64::* 3.0 2)").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn f64_mul_float_times_float() {
        match eval_expr("(:wat::core::f64::* 3.0 2.0)").unwrap() {
            Value::f64(x) => assert_eq!(x, 6.0),
            v => panic!("expected float, got {:?}", v),
        }
    }

    #[test]
    fn divide_by_zero_errors() {
        assert!(matches!(
            eval_expr("(:wat::core::i64::/ 5 0)"),
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
                r#"(:wat::core::let (((x :i64) 2) ((y :i64) 3)) (:wat::core::i64::+ x y))"#
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
    fn bare_single_let_binding_rejected() {
        // `(name rhs)` is NOT accepted. Every bound name's type must
        // be declared at the binding site — the shape is
        // `((name :Type) rhs)` or destructure `((a b ...) rhs)`.
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
              (:wat::core::i64::+ x 1))
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
                  (:wat::core::i64::* n (:my::app::fact (:wat::core::i64::- n 1)))))
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
            (:wat::core::define (:foo (x :i64) -> :i64) (:wat::core::i64::+ x 1))
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
                  (:wat::core::i64::+ x y))
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
                     (:wat::core::i64::+ x 10))))
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
                                    (:wat::core::i64::+ x n))))
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
                 (:wat::core::i64::- 0 1))"#,
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
        let program = parse_one("(:wat::core::i64::+ 40 2)").unwrap();
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
            eval_expr("(:wat::core::quote (:wat::core::i64::+ 1 2))").unwrap();
        match result {
            Value::wat__WatAST(ast) => {
                // The captured AST should be a List whose head is :wat::core::i64::+
                match &*ast {
                    WatAST::List(items) => {
                        assert!(matches!(
                            items.first(),
                            Some(WatAST::Keyword(k)) if k == ":wat::core::i64::+"
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
            "(:wat::algebra::Atom (:wat::core::quote (:wat::core::i64::+ 1 2)))",
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
            "(:wat::core::atom-value (:wat::algebra::Atom (:wat::core::quote (:wat::core::i64::+ 40 2))))",
        )
        .unwrap();
        match result {
            Value::wat__WatAST(ast) => match &*ast {
                WatAST::List(items) => {
                    assert!(matches!(
                        items.first(),
                        Some(WatAST::Keyword(k)) if k == ":wat::core::i64::+"
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
                    (:wat::core::quote (:wat::core::i64::+ 40 2)))
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
            r#"(:wat::algebra::cosine
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
    fn dot_of_atom_with_itself_is_large_positive() {
        // dot(v, v) = |v|² — positive and equal to the number of
        // non-zero dimensions in v's encoding. The exact count
        // depends on the substrate's ternary content; we just
        // assert it's well above sqrt(d) (the noise scale).
        let result = eval_with_ctx(
            r#"(:wat::algebra::dot
                 (:wat::algebra::Atom "alice")
                 (:wat::algebra::Atom "alice"))"#,
            1024,
        )
        .unwrap();
        match result {
            Value::f64(x) => {
                // Expect |v|² > 5*sqrt(d) (~160 at d=1024).
                assert!(x > 5.0 * (1024f64).sqrt(), "got {}", x);
            }
            other => panic!("expected f64, got {:?}", other),
        }
    }

    #[test]
    fn dot_of_unrelated_atoms_vs_self_orders_correctly() {
        // dot(a, a) >> dot(a, b) for independent atoms. The exact
        // magnitudes are substrate-dependent; the ordering is the
        // load-bearing invariant for Gram-Schmidt (Reject / Project).
        let self_dot = match eval_with_ctx(
            r#"(:wat::algebra::dot
                 (:wat::algebra::Atom "alice")
                 (:wat::algebra::Atom "alice"))"#,
            1024,
        )
        .unwrap()
        {
            Value::f64(x) => x,
            other => panic!("expected f64, got {:?}", other),
        };
        let cross_dot = match eval_with_ctx(
            r#"(:wat::algebra::dot
                 (:wat::algebra::Atom "alice")
                 (:wat::algebra::Atom "charlie"))"#,
            1024,
        )
        .unwrap()
        {
            Value::f64(x) => x,
            other => panic!("expected f64, got {:?}", other),
        };
        assert!(
            self_dot > cross_dot.abs() * 3.0,
            "self dot {} should dwarf cross dot {}",
            self_dot,
            cross_dot
        );
    }

    #[test]
    fn dot_wrong_arity() {
        let ast = parse_one(r#"(:wat::algebra::dot (:wat::algebra::Atom "a"))"#).unwrap();
        let err = eval(&ast, &Environment::new(), &test_sym_with_ctx(1024)).unwrap_err();
        assert!(matches!(err, RuntimeError::ArityMismatch { .. }));
    }

    #[test]
    fn dot_refuses_non_holon() {
        let err = eval_with_ctx(r#"(:wat::algebra::dot 1 2)"#, 1024).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn presence_q_true_for_self() {
        // presence? is the boolean verdict — cosine > noise floor.
        // An atom against itself: cosine = 1.0, well above the floor.
        let result = eval_with_ctx(
            r#"(:wat::algebra::presence?
                 (:wat::algebra::Atom "alice")
                 (:wat::algebra::Atom "alice"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(true)));
    }

    #[test]
    fn presence_q_false_for_unrelated() {
        let result = eval_with_ctx(
            r#"(:wat::algebra::presence?
                 (:wat::algebra::Atom "alice")
                 (:wat::algebra::Atom "charlie"))"#,
            1024,
        )
        .unwrap();
        assert!(matches!(result, Value::bool(false)));
    }

    #[test]
    fn cosine_of_atom_with_itself_is_one() {
        // The renamed primitive (algebra::cosine) returns the same
        // scalar the old :wat::core::presence did.
        let result = eval_with_ctx(
            r#"(:wat::algebra::cosine
                 (:wat::algebra::Atom "self")
                 (:wat::algebra::Atom "self"))"#,
            1024,
        )
        .unwrap();
        match result {
            Value::f64(x) => assert!((x - 1.0).abs() < 1e-9, "got {}", x),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn stopped_q_reads_kernel_flag() {
        // The renamed primitive — stopped? per the `?` convention.
        reset_kernel_stop();
        assert!(matches!(
            eval_expr("(:wat::kernel::stopped?)").unwrap(),
            Value::bool(false)
        ));
        request_kernel_stop();
        assert!(matches!(
            eval_expr("(:wat::kernel::stopped?)").unwrap(),
            Value::bool(true)
        ));
        reset_kernel_stop();
    }

    #[test]
    fn presence_requires_encoding_ctx() {
        // Without a frozen SymbolTable, presence must error — can't
        // reach into encoding machinery that doesn't exist.
        let ast = parse_one(
            r#"(:wat::algebra::cosine
                 (:wat::algebra::Atom "a")
                 (:wat::algebra::Atom "b"))"#,
        )
        .unwrap();
        let err = eval(&ast, &Environment::new(), &SymbolTable::new()).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::NoEncodingCtx { op } if op == ":wat::algebra::cosine"
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
                 (:wat::algebra::cosine program bound))"#,
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
                 (:wat::algebra::cosine program recovered))"#,
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
            r#"(:wat::core::eval-edn! :wat::eval::string "(:wat::core::i64::+ 40 2)")"#,
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
        let source = r#"(:wat::core::i64::+ 1 1)"#;
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
                :wat::eval::string "(:wat::core::i64::+ 1 1)"
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
            :wat::eval::string "(:wat::core::i64::+ 1 1)"
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
        let source = r#"(:wat::core::i64::+ 20 22)"#;
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
        let signed_source = r#"(:wat::core::i64::+ 20 22)"#;
        let tampered_source = r#"(:wat::core::i64::+ 99 99)"#;
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
            :wat::eval::string "(:wat::core::i64::+ 1 1)"
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
        let path = write_temp("(:wat::core::i64::+ 10 11)", "wat");
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
        let source = "(:wat::core::i64::* 6 7)";
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

    // ─── User signals — kernel measures, userland owns transitions ─────
    //
    // The three user-signal flags are process-lifetime statics. Tests
    // reset them at entry so test ordering doesn't leak state.

    #[test]
    fn sigusr1_query_reflects_flag_state() {
        reset_user_signals();
        match eval_expr("(:wat::kernel::sigusr1?)").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false, got {:?}", v),
        }
        set_kernel_sigusr1();
        match eval_expr("(:wat::kernel::sigusr1?)").unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
        reset_user_signals();
    }

    #[test]
    fn sigusr2_and_sighup_independent() {
        reset_user_signals();
        set_kernel_sigusr2();
        // sighup? must remain false even though sigusr2? is true.
        match eval_expr("(:wat::kernel::sigusr2?)").unwrap() {
            Value::bool(true) => {}
            v => panic!("expected sigusr2 true, got {:?}", v),
        }
        match eval_expr("(:wat::kernel::sighup?)").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected sighup false, got {:?}", v),
        }
        reset_user_signals();
    }

    #[test]
    fn reset_sigusr1_flips_flag_false() {
        reset_user_signals();
        set_kernel_sigusr1();
        let _ = eval_expr("(:wat::kernel::reset-sigusr1!)").expect("reset");
        match eval_expr("(:wat::kernel::sigusr1?)").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false after reset, got {:?}", v),
        }
        reset_user_signals();
    }

    #[test]
    fn reset_sighup_returns_unit() {
        reset_user_signals();
        set_kernel_sighup();
        let v = eval_expr("(:wat::kernel::reset-sighup!)").expect("reset");
        assert!(matches!(v, Value::Unit));
        reset_user_signals();
    }

    #[test]
    fn user_signal_predicates_refuse_arguments() {
        reset_user_signals();
        assert!(matches!(
            eval_expr("(:wat::kernel::sigusr1? 1)"),
            Err(RuntimeError::ArityMismatch { .. })
        ));
        assert!(matches!(
            eval_expr("(:wat::kernel::reset-sigusr1! true)"),
            Err(RuntimeError::ArityMismatch { .. })
        ));
        reset_user_signals();
    }

    // ─── Tuples + destructure + first/second ───────────────────────────

    /// Helper: evaluate `src` in an env pre-bound with `name -> value`.
    fn eval_with_binding(src: &str, name: &str, value: Value) -> Result<Value, RuntimeError> {
        let ast = parse_one(src).expect("parse ok");
        let env = Environment::new().child().bind(name, value).build();
        eval(&ast, &env, &SymbolTable::new())
    }

    fn pair(a: Value, b: Value) -> Value {
        Value::Tuple(Arc::new(vec![a, b]))
    }

    #[test]
    fn first_extracts_zeroth_element() {
        let p = pair(Value::i64(10), Value::i64(20));
        match eval_with_binding("(:wat::core::first pair)", "pair", p).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn second_extracts_first_element() {
        let p = pair(Value::i64(10), Value::i64(20));
        match eval_with_binding("(:wat::core::second pair)", "pair", p).unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn first_refuses_non_tuple() {
        let err = eval_with_binding("(:wat::core::first v)", "v", Value::i64(42)).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn first_index_out_of_range_on_empty_tuple() {
        let t = Value::Tuple(Arc::new(vec![]));
        let err = eval_with_binding("(:wat::core::first t)", "t", t).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn let_star_destructures_a_pair() {
        let src = r#"
            (:wat::core::let* (((a b) p)) (:wat::core::i64::+ a b))
        "#;
        let p = pair(Value::i64(3), Value::i64(4));
        match eval_with_binding(src, "p", p).unwrap() {
            Value::i64(7) => {}
            v => panic!("expected 7, got {:?}", v),
        }
    }

    #[test]
    fn let_destructure_arity_mismatch_errors() {
        let src = r#"
            (:wat::core::let (((a b c) p)) a)
        "#;
        let p = pair(Value::i64(1), Value::i64(2));
        let err = eval_with_binding(src, "p", p).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn let_destructure_requires_tuple() {
        let src = r#"
            (:wat::core::let (((a b) v)) a)
        "#;
        let err = eval_with_binding(src, "v", Value::i64(42)).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── make-bounded-queue / make-unbounded-queue ─────────────────────

    #[test]
    fn make_bounded_queue_returns_sender_receiver_pair() {
        let src = "(:wat::kernel::make-bounded-queue :i64 1)";
        match eval_expr(src).unwrap() {
            Value::Tuple(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::crossbeam_channel__Sender(_)));
                assert!(matches!(&items[1], Value::crossbeam_channel__Receiver(_)));
            }
            v => panic!("expected tuple, got {:?}", v),
        }
    }

    #[test]
    fn make_unbounded_queue_returns_sender_receiver_pair() {
        let src = "(:wat::kernel::make-unbounded-queue :String)";
        match eval_expr(src).unwrap() {
            Value::Tuple(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::crossbeam_channel__Sender(_)));
                assert!(matches!(&items[1], Value::crossbeam_channel__Receiver(_)));
            }
            v => panic!("expected tuple, got {:?}", v),
        }
    }

    #[test]
    fn queue_roundtrip_via_destructure_and_send_recv() {
        // Make a queue, destructure the pair, send a value, recv it,
        // match to unwrap. End-to-end shape the real kernel primitives
        // expose.
        let src = r#"
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1))
               ((sent :()) (:wat::kernel::send tx 42)))
              (:wat::core::match (:wat::kernel::recv rx)
                ((Some v) v)
                (:None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(42) => {}
            v => panic!("expected 42, got {:?}", v),
        }
    }

    #[test]
    fn make_bounded_queue_refuses_non_keyword_type_arg() {
        let err = eval_expr("(:wat::kernel::make-bounded-queue 42 1)").unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn make_bounded_queue_refuses_negative_capacity() {
        let err = eval_expr("(:wat::kernel::make-bounded-queue :i64 -1)").unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn make_bounded_queue_wrong_arity() {
        let err = eval_expr("(:wat::kernel::make-bounded-queue :i64)").unwrap_err();
        assert!(matches!(err, RuntimeError::ArityMismatch { .. }));
    }

    // ─── Vec/list primitives (Round 4a) ───────────────────────────────

    #[test]
    fn list_constructor_is_alias_for_vec() {
        // Same runtime shape: both produce Value::Vec.
        let v1 = eval_expr("(:wat::core::list 1 2 3)").unwrap();
        let v2 = eval_expr("(:wat::core::vec 1 2 3)").unwrap();
        match (v1, v2) {
            (Value::Vec(a), Value::Vec(b)) => {
                assert_eq!(a.len(), b.len());
                for (x, y) in a.iter().zip(b.iter()) {
                    match (x, y) {
                        (Value::i64(xi), Value::i64(yi)) => assert_eq!(xi, yi),
                        _ => panic!("expected matching i64 items"),
                    }
                }
            }
            _ => panic!("expected Vec values"),
        }
    }

    #[test]
    fn length_of_three_element_vec() {
        match eval_expr("(:wat::core::length (:wat::core::list 1 2 3))").unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3, got {:?}", v),
        }
    }

    #[test]
    fn empty_true_on_empty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::list))").unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn empty_false_on_nonempty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::list 1))").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false, got {:?}", v),
        }
    }

    #[test]
    fn reverse_flips_order() {
        match eval_expr("(:wat::core::reverse (:wat::core::list 1 2 3))").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![3, 2, 1]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn range_start_end() {
        match eval_expr("(:wat::core::range 0 4)").unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![0, 1, 2, 3]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn range_start_geq_end_is_empty() {
        match eval_expr("(:wat::core::range 5 5)").unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn take_first_n() {
        match eval_expr("(:wat::core::take (:wat::core::list 1 2 3 4 5) 3)").unwrap() {
            Value::Vec(items) => assert_eq!(items.len(), 3),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn take_more_than_length_returns_full_vec() {
        match eval_expr("(:wat::core::take (:wat::core::list 1 2) 99)").unwrap() {
            Value::Vec(items) => assert_eq!(items.len(), 2),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn drop_skips_first_n() {
        match eval_expr("(:wat::core::drop (:wat::core::list 1 2 3 4 5) 2)").unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Value::i64(3) => {}
                    v => panic!("expected 3, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn map_doubles_every_element() {
        let src = r#"
            (:wat::core::map
              (:wat::core::list 1 2 3)
              (:wat::core::lambda ((x :i64) -> :i64) (:wat::core::i64::* x 2)))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                assert_eq!(ns, vec![2, 4, 6]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn foldl_sums_with_init() {
        let src = r#"
            (:wat::core::foldl
              (:wat::core::list 1 2 3 4)
              10
              (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
                (:wat::core::i64::+ acc x)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn list_window_builds_sliding_windows() {
        let src = r#"
            (:wat::std::list::window (:wat::core::list 1 2 3 4) 2)
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(outer) => {
                // Expect 3 windows of size 2.
                assert_eq!(outer.len(), 3);
                // First window = [1, 2].
                match &outer[0] {
                    Value::Vec(w) => {
                        assert_eq!(w.len(), 2);
                        match (&w[0], &w[1]) {
                            (Value::i64(1), Value::i64(2)) => {}
                            other => panic!("expected [1,2], got {:?}", other),
                        }
                    }
                    v => panic!("expected Vec window, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn first_polymorphic_on_vec() {
        match eval_expr("(:wat::core::first (:wat::core::list 10 20 30))").unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn second_polymorphic_on_vec() {
        match eval_expr("(:wat::core::second (:wat::core::list 10 20 30))").unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn third_on_vec() {
        match eval_expr("(:wat::core::third (:wat::core::list 10 20 30))").unwrap() {
            Value::i64(30) => {}
            v => panic!("expected 30, got {:?}", v),
        }
    }

    #[test]
    fn rest_drops_first() {
        match eval_expr("(:wat::core::rest (:wat::core::list 1 2 3))").unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 2);
                match (&items[0], &items[1]) {
                    (Value::i64(2), Value::i64(3)) => {}
                    other => panic!("expected [2,3]; got {:?}", other),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn rest_of_empty_errors() {
        let err = eval_expr("(:wat::core::rest (:wat::core::list))").unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn map_with_index_attaches_positions() {
        let src = r#"
            (:wat::std::list::map-with-index
              (:wat::core::list 10 20 30)
              (:wat::core::lambda ((x :i64) (i :i64) -> :i64)
                (:wat::core::i64::+ x i)))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                let ns: Vec<_> = items
                    .iter()
                    .map(|v| match v {
                        Value::i64(n) => *n,
                        _ => panic!("expected i64"),
                    })
                    .collect();
                // 10+0, 20+1, 30+2
                assert_eq!(ns, vec![10, 21, 32]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn list_window_bigger_than_length_is_empty() {
        match eval_expr("(:wat::std::list::window (:wat::core::list 1 2) 5)").unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected empty Vec, got {:?}", v),
        }
    }

    // ─── try-recv + drop ───────────────────────────────────────────────

    #[test]
    fn try_recv_on_empty_queue_returns_none() {
        let src = r#"
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1)))
              (:wat::core::match (:wat::kernel::try-recv rx)
                ((Some _) false)
                (:None true)))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn try_recv_on_ready_queue_returns_some() {
        let src = r#"
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1))
               ((_ :()) (:wat::kernel::send tx 7)))
              (:wat::core::match (:wat::kernel::try-recv rx)
                ((Some v) v)
                (:None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(7) => {}
            v => panic!("expected 7, got {:?}", v),
        }
    }

    #[test]
    fn drop_accepts_sender_returns_unit() {
        let src = r#"
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1)))
              (:wat::kernel::drop tx))
        "#;
        match eval_expr(src).unwrap() {
            Value::Unit => {}
            v => panic!("expected unit, got {:?}", v),
        }
    }

    #[test]
    fn drop_accepts_receiver_returns_unit() {
        let src = r#"
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1)))
              (:wat::kernel::drop rx))
        "#;
        match eval_expr(src).unwrap() {
            Value::Unit => {}
            v => panic!("expected unit, got {:?}", v),
        }
    }

    #[test]
    fn drop_refuses_non_handle() {
        let err = eval_expr("(:wat::kernel::drop 42)").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn try_recv_wrong_arity() {
        let err = eval_expr("(:wat::kernel::try-recv)").unwrap_err();
        assert!(matches!(err, RuntimeError::ArityMismatch { .. }));
    }

    // ─── spawn + join ──────────────────────────────────────────────────

    #[test]
    fn spawn_runs_function_on_new_thread_and_join_returns_its_value() {
        // Register a function, spawn it with args, join the handle,
        // confirm the function's return value surfaces.
        let src = r#"
            (:wat::core::define (:my::sum (a :i64) (b :i64) -> :i64)
              (:wat::core::i64::+ a b))
            (:wat::kernel::join (:wat::kernel::spawn :my::sum 3 4))
        "#;
        match run(src).unwrap() {
            Value::i64(7) => {}
            v => panic!("expected 7, got {:?}", v),
        }
    }

    #[test]
    fn spawn_refuses_unknown_function() {
        let err = eval_expr("(:wat::kernel::spawn :no::such::function)").unwrap_err();
        assert!(matches!(err, RuntimeError::UnknownFunction(_)));
    }

    #[test]
    fn spawn_refuses_non_keyword_head() {
        let err = eval_expr("(:wat::kernel::spawn 42)").unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn join_refuses_non_program_handle() {
        let err = eval_expr("(:wat::kernel::join 42)").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── select ────────────────────────────────────────────────────────

    #[test]
    fn select_returns_index_and_value_from_ready_receiver() {
        // Two queues; send only to the second; select returns index 1
        // with the value.
        let src = r#"
            (:wat::core::let*
              (((tx0 rx0) (:wat::kernel::make-bounded-queue :i64 1))
               (((tx1 rx1)) (:wat::kernel::make-bounded-queue :i64 1)))
              ;; (this shape won't parse — rewrite below)
              true)
        "#;
        let _ = src; // placeholder; inline the real test directly below.

        // Direct construction: two receivers, only rx1 gets a value,
        // select must pick index 1.
        let (tx0, rx0) = crossbeam_channel::bounded::<Value>(1);
        let (tx1, rx1) = crossbeam_channel::bounded::<Value>(1);
        drop(tx0); // rx0 disconnected
        tx1.send(Value::i64(7)).unwrap();
        let rxs = Value::Vec(Arc::new(vec![
            Value::crossbeam_channel__Receiver(Arc::new(rx0)),
            Value::crossbeam_channel__Receiver(Arc::new(rx1)),
        ]));
        let env = Environment::new().child().bind("rxs", rxs).build();
        let ast = parse_one("(:wat::kernel::select rxs)").expect("parse");
        let result = eval(&ast, &env, &SymbolTable::new()).expect("select");
        match result {
            Value::Tuple(items) => {
                assert_eq!(items.len(), 2);
                // select may pick index 0 (disconnected, :None) or
                // index 1 (Some 7). Both are valid because crossbeam's
                // select doesn't promise ordering. Accept either and
                // assert the OPTION is consistent with the index.
                match (&items[0], &items[1]) {
                    (Value::i64(0), Value::Option(opt)) if opt.is_none() => {}
                    (Value::i64(1), Value::Option(opt)) => match &**opt {
                        Some(Value::i64(7)) => {}
                        other => panic!("index 1 should carry Some 7; got {:?}", other),
                    },
                    other => panic!("unexpected select result {:?}", other),
                }
            }
            v => panic!("expected tuple, got {:?}", v),
        }
        drop(tx1);
    }

    #[test]
    fn select_refuses_empty_vec() {
        let src = r#"
            (:wat::kernel::select (:wat::core::vec))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn select_refuses_non_receiver_element() {
        let src = r#"
            (:wat::kernel::select (:wat::core::vec 1 2 3))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── HandlePool ────────────────────────────────────────────────────

    #[test]
    fn handle_pool_pop_all_then_finish() {
        let src = r#"
            (:wat::core::let*
              (((pool :wat::kernel::HandlePool<i64>)
                (:wat::kernel::HandlePool::new "test" (:wat::core::vec 1 2 3)))
               ((a :i64) (:wat::kernel::HandlePool::pop pool))
               ((b :i64) (:wat::kernel::HandlePool::pop pool))
               ((c :i64) (:wat::kernel::HandlePool::pop pool))
               ((_ :()) (:wat::kernel::HandlePool::finish pool)))
              (:wat::core::i64::+ (:wat::core::i64::+ a b) c))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(6) => {}
            v => panic!("expected 6, got {:?}", v),
        }
    }

    #[test]
    fn handle_pool_pop_from_empty_errors() {
        let src = r#"
            (:wat::core::let*
              (((pool :wat::kernel::HandlePool<i64>)
                (:wat::kernel::HandlePool::new "empty" (:wat::core::vec)))
               ((_ :i64) (:wat::kernel::HandlePool::pop pool)))
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn handle_pool_finish_with_orphans_errors() {
        let src = r#"
            (:wat::core::let*
              (((pool :wat::kernel::HandlePool<i64>)
                (:wat::kernel::HandlePool::new "orphaned" (:wat::core::vec 1 2 3)))
               ((_ :()) (:wat::kernel::HandlePool::finish pool)))
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn handle_pool_name_surfaces_in_error() {
        let src = r#"
            (:wat::core::let*
              (((pool :wat::kernel::HandlePool<i64>)
                (:wat::kernel::HandlePool::new "named-pool" (:wat::core::vec)))
               ((_ :i64) (:wat::kernel::HandlePool::pop pool)))
              0)
        "#;
        let err = eval_expr(src).unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("named-pool"),
            "error should name the pool; got: {}",
            msg
        );
    }

    // ─── Stdlib math ───────────────────────────────────────────────────

    #[test]
    fn math_ln_of_e_is_one() {
        // ln(e) = 1.
        let src = "(:wat::std::math::ln 2.718281828459045)";
        match eval_expr(src).unwrap() {
            Value::f64(x) => assert!((x - 1.0).abs() < 1e-10, "got {}", x),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_log_is_natural_log() {
        // `log` is the natural-log alias; matches ln for identical input.
        let a = match eval_expr("(:wat::std::math::log 2.718281828459045)").unwrap() {
            Value::f64(x) => x,
            v => panic!("expected f64, got {:?}", v),
        };
        let b = match eval_expr("(:wat::std::math::ln 2.718281828459045)").unwrap() {
            Value::f64(x) => x,
            v => panic!("expected f64, got {:?}", v),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn math_sin_pi_is_zero() {
        let src = "(:wat::std::math::sin (:wat::std::math::pi))";
        match eval_expr(src).unwrap() {
            Value::f64(x) => assert!(x.abs() < 1e-10, "got {}", x),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_cos_zero_is_one() {
        match eval_expr("(:wat::std::math::cos 0.0)").unwrap() {
            Value::f64(x) => assert_eq!(x, 1.0),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_pi_is_std_const() {
        match eval_expr("(:wat::std::math::pi)").unwrap() {
            Value::f64(x) => assert_eq!(x, std::f64::consts::PI),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_ln_accepts_i64_promotion() {
        // Integer arg gets promoted to f64 before the call.
        match eval_expr("(:wat::std::math::ln 1)").unwrap() {
            Value::f64(x) => assert_eq!(x, 0.0),
            v => panic!("expected f64, got {:?}", v),
        }
    }

    #[test]
    fn math_ln_wrong_arity() {
        let err = eval_expr("(:wat::std::math::ln 1.0 2.0)").unwrap_err();
        assert!(matches!(err, RuntimeError::ArityMismatch { .. }));
    }

    #[test]
    fn math_ln_refuses_non_number() {
        let err = eval_expr(r#"(:wat::std::math::ln "nope")"#).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn handle_pool_refuses_non_string_name() {
        let src = r#"
            (:wat::kernel::HandlePool::new 42 (:wat::core::vec))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn spawn_and_join_produce_queue_roundtrip_across_threads() {
        // Producer thread sends, consumer thread (the main) recv + match.
        // Proves the typed pipe survives the spawn.
        let src = r#"
            (:wat::core::define (:my::producer
                                 (tx :crossbeam_channel::Sender<i64>)
                                 -> :())
              (:wat::kernel::send tx 99))
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1))
               ((handle :wat::kernel::ProgramHandle<()>)
                (:wat::kernel::spawn :my::producer tx))
               ((_ :()) (:wat::kernel::join handle)))
              (:wat::core::match (:wat::kernel::recv rx)
                ((Some v) v)
                (:None 0)))
        "#;
        match run(src).unwrap() {
            Value::i64(99) => {}
            v => panic!("expected 99, got {:?}", v),
        }
    }
}
