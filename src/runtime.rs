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
//! The runtime treats type annotations as opaque — parse-level
//! validation rejects `:Any` and malformed type keywords, but no
//! runtime-level type enforcement happens here. The type checker
//! runs its own phase during the startup pipeline (see
//! [`crate::check`]); by the time `eval` runs, every expression
//! has already been type-verified.

use crate::ast::WatAST;
use crate::config::Config;
use holon::{encode, AtomTypeRegistry, HolonAST, ScalarEncoder, Similarity, VectorManager};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Kernel-owned stop flag read by `(:wat::kernel::stopped?)`.
///
/// The wat binary installs OS signal handlers for SIGINT and
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

/// Set the kernel stop flag to `true`. Called by the wat CLI's
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
    /// `:u8` — unsigned 8-bit integer, 0..=255. Produced by
    /// `:wat::core::u8` (range-checked cast from i64), consumed by
    /// byte-oriented IO (`:wat::io::read`, `:wat::io::write`) and
    /// `:Vec<u8>` carriers. Arithmetic is wrapping per Rust's
    /// default u8 semantics. Slice 1 of arc 008.
    u8(u8),
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
    /// A `:HashMap<K,V>` — Rust std's `HashMap` backing, wrapped for
    /// cheap Arc-cloning. Keys are serialized to type-tagged strings
    /// at insertion so heterogeneous-K programs don't collide
    /// (`"42"` vs `42` vs `:42`). Stored entries carry the ORIGINAL
    /// key Value alongside the Value so lookups round-trip the
    /// caller's key variant. Scoped to primitive keys in this slice
    /// (i64, f64, bool, String, keyword); composite keys land when
    /// a caller demands them.
    wat__std__HashMap(Arc<std::collections::HashMap<String, (Value, Value)>>),
    /// A `:HashSet<T>` — Rust std's HashSet semantically; stored as
    /// a `HashMap<canonical-key, original-value>` so `get` can
    /// return the stored variant on hit. Primitive elements only in
    /// this slice (matches HashMap's key scope).
    wat__std__HashSet(Arc<std::collections::HashMap<String, Value>>),
    /// Generic opaque handle to a Rust-shim-owned value. The
    /// target-form for any `:rust::*` type that doesn't have its own
    /// dedicated Value variant. The inner `RustOpaqueInner` carries a
    /// `type_path` identifier plus an erased payload; shim dispatch
    /// code downcasts via [`crate::rust_deps::downcast_ref_opaque`].
    /// Used by the `#[wat_dispatch]` macro's generated code for all
    /// Self-returning methods.
    RustOpaque(Arc<crate::rust_deps::RustOpaqueInner>),
    /// Abstract byte-source handle — `:wat::io::IOReader`. Wraps any
    /// `WatReader` implementation (real stdin, in-memory `StringIoReader`,
    /// …). Arc 008 slice 2.
    io__IOReader(Arc<dyn crate::io::WatReader>),
    /// Abstract byte-sink handle — `:wat::io::IOWriter`. Wraps any
    /// `WatWriter` implementation (real stdout/stderr, in-memory
    /// `StringIoWriter`, …). Arc 008 slice 2.
    io__IOWriter(Arc<dyn crate::io::WatWriter>),
    /// An `:Option<T>` value — `:None` or `(Some v)`. Built-in
    /// parametric enum per 058-030; used as the return type of
    /// `:wat::kernel::recv` / `try-recv` / `select` and of structural
    /// retrieval (`get` on HashMap/Vec/HashSet). The `std::option::Option`
    /// here is the Rust host's own Option — wat's `:Option<T>`
    /// compiles to it directly.
    Option(Arc<std::option::Option<Value>>),
    /// A `:Result<T,E>` value — `(Ok v)` or `(Err e)`. Built-in
    /// parametric enum for fallible operations. Surfaced by Rust-dep
    /// shims that wrap crates returning `std::result::Result` (rusqlite
    /// and friends). Constructors are symbol-dispatched (`Ok` / `Err`
    /// as bare identifiers, arity 1 each); consumers use
    /// `(:wat::core::match ...)`.
    Result(Arc<std::result::Result<Value, Value>>),
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
    /// An instance of a user-declared `:wat::core::struct` type — a
    /// tagged positional tuple. `type_name` carries the struct's
    /// keyword path (e.g., `:wat::algebra::CapacityExceeded`); `fields`
    /// holds the values in declaration order. Produced by the
    /// auto-generated `<struct>/new` constructor. Read via the
    /// auto-generated `<struct>/<field>` accessors — both of which are
    /// ordinary [`Function`] entries in the symbol table whose bodies
    /// invoke the `:wat::core::struct-new` / `:wat::core::struct-field`
    /// primitives. No field-by-name dispatch at runtime: accessors are
    /// resolved at parse time like any other keyword-path call.
    Struct(Arc<StructValue>),
}

/// The payload of a [`Value::Struct`] — the struct's fully-qualified
/// declared type name plus its positional field values in declaration
/// order. Cheap to clone (stored in an `Arc` at the Value level).
#[derive(Debug, Clone)]
pub struct StructValue {
    /// Full keyword path of the struct type, e.g.
    /// `:wat::algebra::CapacityExceeded`. Matches the declaration's
    /// name verbatim; identity for type-tag comparisons.
    pub type_name: String,
    /// Field values in declaration order. Length matches the
    /// `StructDef::fields` length at construction time; the type
    /// checker enforces alignment.
    pub fields: Vec<Value>,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::bool(_) => "bool",
            Value::i64(_) => "i64",
            Value::u8(_) => "u8",
            Value::f64(_) => "f64",
            Value::String(_) => "String",
            Value::Vec(_) => "Vec",
            Value::Unit => "()",
            Value::wat__core__keyword(_) => "wat::core::keyword",
            Value::wat__core__lambda(_) => "wat::core::lambda",
            Value::holon__HolonAST(_) => "holon::HolonAST",
            Value::wat__WatAST(_) => "wat::WatAST",
            Value::crossbeam_channel__Sender(_) => "rust::crossbeam_channel::Sender",
            Value::crossbeam_channel__Receiver(_) => "rust::crossbeam_channel::Receiver",
            Value::wat__std__HashMap(_) => "rust::std::collections::HashMap",
            Value::wat__std__HashSet(_) => "rust::std::collections::HashSet",
            Value::RustOpaque(inner) => inner.type_path,
            Value::io__IOReader(_) => "wat::io::IOReader",
            Value::io__IOWriter(_) => "wat::io::IOWriter",
            Value::Option(_) => "Option",
            Value::Result(_) => "Result",
            Value::Tuple(_) => "tuple",
            Value::wat__kernel__ProgramHandle(_) => "wat::kernel::ProgramHandle",
            Value::wat__kernel__HandlePool { .. } => "wat::kernel::HandlePool",
            Value::Struct(_) => "Struct",
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

/// Keyword-path ↦ Function registry + runtime capabilities.
///
/// The `encoding_ctx` and `source_loader` fields are populated at
/// freeze time by the startup pipeline. Test harnesses
/// (`SymbolTable::new()`) leave them `None`; primitives that require
/// the capability (presence / encode for ctx, `:wat::eval::file-path`
/// for loader) error cleanly if invoked without one attached.
///
/// Runtime-capability attachment follows the pattern established by
/// Rust's compiler `Session`, Common Lisp special variables,
/// Clojure dynamic vars, and Haskell `ReaderT`. See arc 007 DESIGN.md.
#[derive(Clone)]
pub struct SymbolTable {
    pub functions: HashMap<String, Arc<Function>>,
    pub encoding_ctx: Option<Arc<EncodingCtx>>,
    pub source_loader: Option<Arc<dyn crate::load::SourceLoader>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
            encoding_ctx: None,
            source_loader: None,
        }
    }
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolTable")
            .field("functions", &self.functions.len())
            .field("encoding_ctx", &self.encoding_ctx.is_some())
            .field("source_loader", &self.source_loader.is_some())
            .finish()
    }
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

    /// Attach a source loader. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`], mirrors
    /// [`SymbolTable::set_encoding_ctx`].
    pub fn set_source_loader(&mut self, loader: Arc<dyn crate::load::SourceLoader>) {
        self.source_loader = Some(loader);
    }

    /// Borrow the source loader, if one is attached. Runtime primitives
    /// that read files (`:wat::eval::file-path`,
    /// `:wat::verify::file-path`) call this and raise an error on
    /// `None` — a host that didn't attach a loader doesn't have the
    /// capability.
    pub fn source_loader(&self) -> Option<&Arc<dyn crate::load::SourceLoader>> {
        self.source_loader.as_ref()
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
    /// Raised when `:wat::kernel::join` reaps a spawned program
    /// whose thread panicked before yielding a result — the internal
    /// handle channel's Sender was dropped without sending, so the
    /// join's `recv` sees disconnected.
    ///
    /// User channels (`:wat::kernel::send` / `recv` / `try-recv`)
    /// are symmetric on disconnect — both endpoints report it via
    /// `:Option` rather than via this error, so no call path in the
    /// user-level channel primitives produces this variant. It
    /// remains only for the join-on-panic case.
    ChannelDisconnected { op: String },
    /// A vector-level primitive (`:wat::algebra::cosine`,
    /// `:wat::config::noise-floor`, etc.) was invoked but the
    /// [`SymbolTable`] has no attached [`EncodingCtx`]. Reachable from
    /// test harnesses that don't go through freeze; the frozen startup
    /// pipeline always installs one.
    NoEncodingCtx { op: String },
    /// A file-reading primitive (`:wat::eval::file-path`,
    /// `:wat::verify::file-path`) was invoked but the [`SymbolTable`]
    /// has no attached source loader. The frozen startup pipeline
    /// attaches the loader handed to `startup_from_source`; test
    /// harnesses that build a SymbolTable directly must call
    /// [`SymbolTable::set_source_loader`] to grant file-I/O capability.
    NoSourceLoader { op: String },
    /// A `(:wat::core::match scrutinee ...)` ran with no arm whose
    /// pattern matches the scrutinee's shape. Exhaustiveness is the
    /// type checker's job; this variant fires only when the check was
    /// bypassed or hasn't caught up with a new pattern form.
    PatternMatchFailed { value_type: &'static str },
    /// Internal control-flow signal raised by `:wat::core::try` on an
    /// `Err` value. Carries the `Err` payload up to the innermost
    /// enclosing function/lambda boundary; [`apply_function`] catches
    /// it and converts it into the function's own `Err(e)` return.
    ///
    /// This variant should never escape to `:user::main` — the type
    /// checker guarantees every `try` appears inside a Result-returning
    /// function, so every TryPropagate hits an `apply_function` catch
    /// before unwinding further. If this variant does reach the binary,
    /// it indicates either a checker bug or a `try` used inside
    /// constrained eval (which doesn't have an enclosing function for
    /// propagation — that's a planned follow-up slice).
    TryPropagate(Value),
    /// Internal tail-call signal raised by `eval_tail` when it
    /// recognizes a user-defined function call in tail position.
    /// Carries the next function and its already-evaluated args up
    /// to the enclosing [`apply_function`]'s trampoline loop, which
    /// reassigns `cur_func`/`cur_args` and re-iterates without
    /// recursing into eval — constant Rust stack across arbitrary
    /// tail-recursion depth.
    ///
    /// Stage 1 of the TCO arc (see `docs/arc/2026/04/003-*`) covers
    /// user-defined functions registered in the `SymbolTable`
    /// (`define`-registered). Lambda self/mutual-tail-calls land in
    /// Stage 2. A lambda's body that itself tail-calls a named
    /// define is already covered — the signal fires at the named
    /// call, `apply_function`'s loop catches it just as it does for
    /// a named-define self-recursion.
    ///
    /// Like [`TryPropagate`], this variant must never surface to
    /// user code. Reaching it in production is a bug.
    TailCall {
        func: Arc<Function>,
        args: Vec<Value>,
    },
    /// Raised by `:wat::kernel::assertion-failed!` when an assertion in
    /// a `:wat::test::*` form (or any user code that calls the primitive
    /// directly) fails. Intended to travel as a panic payload via the
    /// [`crate::assertion::AssertionPayload`] struct and be caught by
    /// `run-sandboxed`'s `catch_unwind`, where actual/expected land in
    /// the `:wat::kernel::Failure`'s slots. Outside a sandbox, this
    /// variant surfaces as an ordinary RuntimeError — reporting that
    /// an assertion fired without a test harness to catch it.
    AssertionFailed {
        message: String,
        actual: Option<String>,
        expected: Option<String>,
    },
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
            RuntimeError::NoSourceLoader { op } => write!(
                f,
                "{}: no source loader attached to SymbolTable; file-reading primitives require a loader. Call via the freeze pipeline, or set_source_loader on the test SymbolTable.",
                op
            ),
            RuntimeError::PatternMatchFailed { value_type } => write!(
                f,
                ":wat::core::match: no arm matched scrutinee of type {}; exhaustiveness should be caught at type-check time",
                value_type
            ),
            RuntimeError::TryPropagate(_) => write!(
                f,
                ":wat::core::try: internal error — an Err propagation escaped its enclosing Result-returning function. The type checker should prevent this; reaching it indicates a checker gap or a try used in a context without a Result return type.",
            ),
            RuntimeError::TailCall { .. } => write!(
                f,
                "TCO: internal error — a tail-call signal escaped its enclosing apply_function. The evaluator should catch TailCall at every function boundary; reaching the user with one unwound indicates an interpreter bug.",
            ),
            RuntimeError::AssertionFailed { message, actual, expected } => {
                write!(f, "assertion failed: {}", message)?;
                if let Some(a) = actual {
                    write!(f, "\n  actual:   {}", a)?;
                }
                if let Some(e) = expected {
                    write!(f, "\n  expected: {}", e)?;
                }
                Ok(())
            }
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

/// Walk every `:wat::core::struct` declaration in `types` and
/// synthesize its auto-generated constructor + per-field accessors
/// into `sym`. Runs after both stdlib and user defines have been
/// registered so any name collision with a user-supplied path raises
/// `DuplicateDefine` at a sensible point in the pipeline.
///
/// **What's synthesized, per struct `:my::ns::T` with fields
/// `(f1 :T1) (f2 :T2) ... (fn :Tn)`:**
///
/// - One constructor at keyword path `:my::ns::T/new`:
///   ```text
///   :fn(T1, T2, ..., Tn) -> :my::ns::T
///   body: (:wat::core::struct-new :my::ns::T p1 p2 ... pn)
///   ```
/// - One accessor per field at `:my::ns::T/<field-name>`:
///   ```text
///   :fn(:my::ns::T) -> Ti
///   body: (:wat::core::struct-field self i)
///   ```
///
/// Users never write these; they invoke them by full keyword path.
/// The checker picks them up through [`crate::check::CheckEnv::from_symbols`]
/// as ordinary [`Function`] entries — no new scheme-registration path.
///
/// **Self-trust bypass.** Struct-method paths under `:wat::algebra::*`
/// (the built-in `:wat::algebra::CapacityExceeded/…`) would otherwise
/// hit the reserved-prefix check. This function skips the check: the
/// paths it emits are derived mechanically from struct declarations
/// the user / builtins authored legitimately, so emitting them under
/// the same prefix is legitimate too.
pub fn register_struct_methods(
    types: &crate::types::TypeEnv,
    sym: &mut SymbolTable,
) -> Result<(), RuntimeError> {
    use crate::identifier::Identifier;
    use crate::types::TypeDef;

    for (_name, def) in types.iter() {
        let struct_def = match def {
            TypeDef::Struct(s) => s,
            _ => continue,
        };

        let struct_type = crate::types::TypeExpr::Path(struct_def.name.clone());

        // Constructor — `<struct>/new`. One param per field, same
        // order as declaration. Body invokes `:wat::core::struct-new`
        // with the struct's type-name keyword and the params as
        // symbols.
        let constructor_path = format!("{}/new", struct_def.name);
        let param_names: Vec<String> =
            struct_def.fields.iter().map(|(n, _)| n.clone()).collect();
        let param_types: Vec<crate::types::TypeExpr> = struct_def
            .fields
            .iter()
            .map(|(_, t)| t.clone())
            .collect();
        let mut new_body_items = Vec::with_capacity(2 + struct_def.fields.len());
        new_body_items.push(WatAST::Keyword(":wat::core::struct-new".into()));
        new_body_items.push(WatAST::Keyword(struct_def.name.clone()));
        for param_name in &param_names {
            new_body_items.push(WatAST::Symbol(Identifier::bare(param_name.clone())));
        }
        let new_func = Function {
            name: Some(constructor_path.clone()),
            params: param_names.clone(),
            type_params: struct_def.type_params.clone(),
            param_types: param_types.clone(),
            ret_type: struct_type.clone(),
            body: Arc::new(WatAST::List(new_body_items)),
            closed_env: None,
        };
        if sym.functions.contains_key(&constructor_path) {
            return Err(RuntimeError::DuplicateDefine(constructor_path));
        }
        sym.functions.insert(constructor_path, Arc::new(new_func));

        // Accessors — `<struct>/<field>` per field, positional body.
        // The accessor's single param is called `self` by convention;
        // the body references it as a symbol and the `struct-field`
        // primitive reads by the index baked into the body.
        for (index, (field_name, field_type)) in struct_def.fields.iter().enumerate() {
            let accessor_path = format!("{}/{}", struct_def.name, field_name);
            let accessor_body = WatAST::List(vec![
                WatAST::Keyword(":wat::core::struct-field".into()),
                WatAST::Symbol(Identifier::bare("self")),
                WatAST::IntLit(index as i64),
            ]);
            let accessor_func = Function {
                name: Some(accessor_path.clone()),
                params: vec!["self".into()],
                type_params: struct_def.type_params.clone(),
                param_types: vec![struct_type.clone()],
                ret_type: field_type.clone(),
                body: Arc::new(accessor_body),
                closed_env: None,
            };
            if sym.functions.contains_key(&accessor_path) {
                return Err(RuntimeError::DuplicateDefine(accessor_path));
            }
            sym.functions.insert(accessor_path, Arc::new(accessor_func));
        }
    }
    Ok(())
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

/// Evaluate `ast` in **tail position** with respect to the innermost
/// enclosing [`apply_function`]. When a user-defined function call
/// appears here, emit [`RuntimeError::TailCall`] instead of recursing
/// through `apply_function`; the enclosing loop catches the signal,
/// reassigns `cur_func`/`cur_args`, and re-iterates without stack
/// growth. Everything else delegates to [`eval`].
///
/// The tail-carrying forms (`if`, `match`, `let`, `let*`) have sibling
/// tail-aware helpers (`eval_if_tail`, `eval_match_tail`,
/// `eval_let_tail`, `eval_let_star_tail`) that reuse the same
/// validation as their non-tail twins but dispatch the body through
/// `eval_tail` rather than `eval`.
///
/// Three tail-call shapes are detected (Stage 2 covers all three):
///
/// 1. **Keyword head** resolving in `sym.functions` — a
///    `define`-registered named function (Stage 1's original scope).
/// 2. **Bare-symbol head** resolving to a lambda value in `env` —
///    lambda-valued params and let-bound lambdas. Enables
///    Y-combinator-lite self-recursion (lambda passed as argument)
///    without a letrec mechanism.
/// 3. **Inline-lambda-literal head** `((lambda ...) args)` — the
///    head evaluates to a lambda value directly.
///
/// Non-lambda, non-registered, non-form heads delegate to [`eval`]
/// so error handling (NotCallable, UnboundSymbol, primitive
/// dispatch, `Some`/`Ok`/`Err` constructors) is unchanged.
fn eval_tail(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let items = match ast {
        WatAST::List(items) if !items.is_empty() => items,
        _ => return eval(ast, env, sym),
    };
    let args = &items[1..];
    match &items[0] {
        WatAST::Keyword(k) => {
            let head = k.as_str();
            match head {
                ":wat::core::if" => eval_if_tail(args, env, sym),
                ":wat::core::match" => eval_match_tail(args, env, sym),
                ":wat::core::let" => eval_let_tail(args, env, sym),
                ":wat::core::let*" => eval_let_star_tail(args, env, sym),
                // A user-defined function call in tail position — signal.
                // Head resolves in sym.functions; anything else (kernel/
                // algebra/config primitive, :rust:: shim) runs through
                // regular eval.
                other if sym.functions.contains_key(other) => {
                    let func = sym.get(other).expect("contains_key above").clone();
                    emit_tail_call(func, args, env, sym)
                }
                _ => eval(ast, env, sym),
            }
        }
        // Bare-symbol head: a lambda-valued local binding. `Some`,
        // `Ok`, `Err` are constructor symbols that are NEVER bound in
        // env, so `env.lookup` returns None for them and we delegate
        // to eval (which special-cases the three constructors).
        WatAST::Symbol(ident) => {
            if let Some(Value::wat__core__lambda(f)) = env.lookup(ident.as_str()) {
                emit_tail_call(f, args, env, sym)
            } else {
                eval(ast, env, sym)
            }
        }
        // Inline lambda-literal head `((lambda ...) args)`. Evaluate
        // the head non-tail; if the value is a lambda, signal tail
        // call; otherwise delegate to `apply_value` with the
        // already-evaluated callee so we don't re-evaluate.
        WatAST::List(_) => {
            let callee = eval(&items[0], env, sym)?;
            match callee {
                Value::wat__core__lambda(f) => emit_tail_call(f, args, env, sym),
                other => apply_value(&other, args, env, sym),
            }
        }
        // Literal head (int/float/bool/string) — not callable; let
        // eval raise the right error.
        _ => eval(ast, env, sym),
    }
}

/// Evaluate `raw_args` non-tail and emit a [`RuntimeError::TailCall`]
/// carrying `func`. Shared helper for all three tail-call shapes
/// (named define, bare-symbol lambda, inline-lambda literal). Arity
/// is enforced by [`apply_function`]'s trampoline loop on its next
/// iteration.
fn emit_tail_call(
    func: Arc<Function>,
    raw_args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let vals = raw_args
        .iter()
        .map(|a| eval(a, env, sym))
        .collect::<Result<Vec<_>, _>>()?;
    Err(RuntimeError::TailCall { func, args: vals })
}

/// Tail-position twin of [`eval_if`]. Same validation; the selected
/// branch body is evaluated via [`eval_tail`] instead of [`eval`].
fn eval_if_tail(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() == 3 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` now requires `-> :T` between cond and then-branch; write (:wat::core::if cond -> :T then else)".into(),
        });
    }
    if args.len() != 5 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond -> :T then else) — 5 args; got {}",
                args.len()
            ),
        });
    }
    match &args[1] {
        WatAST::Symbol(s) if s.as_str() == "->" => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: format!(
                    "expected `->` at position 2; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    match &args[2] {
        WatAST::Keyword(_) => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: format!(
                    "expected type keyword at position 3 (after `->`); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    let cond_val = eval(&args[0], env, sym)?;
    match cond_val {
        Value::bool(true) => eval_tail(&args[3], env, sym),
        Value::bool(false) => eval_tail(&args[4], env, sym),
        other => Err(RuntimeError::BadCondition {
            got: other.type_name(),
        }),
    }
}

/// Tail-position twin of [`eval_let`]. Bindings evaluate in the outer
/// env (as with the non-tail form); the body runs through
/// [`eval_tail`] so a tail-call inside it propagates.
fn eval_let_tail(
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
                let value = eval(rhs, env, sym)?;
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
    eval_tail(body, &scope, sym)
}

/// Tail-position twin of [`eval_let_star`]. Bindings accumulate
/// sequentially (each RHS sees prior bindings); the body runs through
/// [`eval_tail`] so a tail-call inside it propagates.
fn eval_let_star_tail(
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
    eval_tail(body, &scope, sym)
}

/// Tail-position twin of [`eval_match`]. The matched arm's body is
/// evaluated via [`eval_tail`] — a tail-call inside an arm body
/// propagates through to `apply_function`'s trampoline.
fn eval_match_tail(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() < 4 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: if args.len() >= 2
                && !matches!(
                    args.get(1),
                    Some(WatAST::Symbol(s)) if s.as_str() == "->"
                )
            {
                "`:wat::core::match` now requires `-> :T` between scrutinee and arms; write (:wat::core::match scrut -> :T (pat body) ...)".into()
            } else {
                format!(
                    "expected (:wat::core::match scrut -> :T arm1 arm2 ...) — at least 4 args; got {}",
                    args.len()
                )
            },
        });
    }
    match &args[1] {
        WatAST::Symbol(s) if s.as_str() == "->" => {}
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "expected `->` after scrutinee (write `-> :T` between scrutinee and arms)".into(),
            });
        }
    }
    match &args[2] {
        WatAST::Keyword(_) => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "expected type keyword after `->`; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    let scrutinee = eval(&args[0], env, sym)?;
    for arm in &args[3..] {
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
            return eval_tail(body, &arm_env, sym);
        }
    }
    Err(RuntimeError::PatternMatchFailed {
        value_type: scrutinee.type_name(),
    })
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
            // Arc 009 — names are values. If the keyword is a registered
            // user/stdlib define, lift it to a callable Function value.
            // Parallels `:wat::kernel::spawn`'s long-standing accept-by-
            // name convention — generalized here so every `:fn(...)`-
            // typed parameter accepts a bare keyword-path reference.
            // Primitives (kernel/algebra/config/io) stay call-only at
            // runtime; they can pass the type check but won't evaluate
            // to a Function until a caller demands that extension.
            if let Some(func) = sym.get(k) {
                return Ok(Value::wat__core__lambda(func.clone()));
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
    // `()` evaluates to Unit. Natural reading: the empty list /
    // empty tuple IS the unit value. Lets `(if cond do-work ())`
    // cleanly express "if else unit" without awkward placeholder
    // calls. Matches the type-level `:()` keyword (unit type) at
    // the value level.
    let head = match items.first() {
        Some(h) => h,
        None => return Ok(Value::Unit),
    };
    let rest = &items[1..];

    match head {
        WatAST::Keyword(k) => dispatch_keyword_head(k, rest, env, sym),
        WatAST::Symbol(ident) if ident.as_str() == "Some" => eval_some_ctor(rest, env, sym),
        WatAST::Symbol(ident) if ident.as_str() == "Ok" => eval_ok_ctor(rest, env, sym),
        WatAST::Symbol(ident) if ident.as_str() == "Err" => eval_err_ctor(rest, env, sym),
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
        ":wat::core::forms" => Ok(eval_forms(args)?),
        ":wat::core::atom-value" => eval_atom_value(args, env, sym),
        ":wat::core::match" => eval_match(args, env, sym),
        ":wat::core::try" => eval_try(args, env, sym),
        ":wat::core::struct-new" => eval_struct_new(args, env, sym),
        ":wat::core::struct-field" => eval_struct_field(args, env, sym),
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

        // :u8 range-checked cast from :i64. Arc 008 slice 1.
        ":wat::core::u8" => eval_u8_cast(args, env, sym),

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
        // String basics — per-type ops under :wat::core::string::*,
        // following the :wat::core::i64::* precedent. Char-oriented.
        ":wat::core::string::contains?" => {
            crate::string_ops::eval_string_contains(args, env, sym)
        }
        ":wat::core::string::starts-with?" => {
            crate::string_ops::eval_string_starts_with(args, env, sym)
        }
        ":wat::core::string::ends-with?" => {
            crate::string_ops::eval_string_ends_with(args, env, sym)
        }
        ":wat::core::string::length" => crate::string_ops::eval_string_length(args, env, sym),
        ":wat::core::string::trim" => crate::string_ops::eval_string_trim(args, env, sym),
        ":wat::core::string::split" => crate::string_ops::eval_string_split(args, env, sym),
        ":wat::core::string::join" => crate::string_ops::eval_string_join(args, env, sym),

        // Regex — pattern matching. Lives in its own :wat::core::regex::*
        // namespace since the regex crate is a distinct concern.
        ":wat::core::regex::matches?" => crate::string_ops::eval_regex_matches(args, env, sym),

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
        ":wat::core::=" => eval_eq(head, args, env, sym),
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
        ":wat::core::conj" => eval_conj(args, env, sym),
        ":wat::core::tuple" => eval_tuple_ctor(args, env, sym),
        ":wat::core::length" => eval_vec_length(args, env, sym),
        ":wat::core::empty?" => eval_vec_empty(args, env, sym),
        ":wat::core::reverse" => eval_vec_reverse(args, env, sym),
        ":wat::core::range" => eval_vec_range(args, env, sym),
        ":wat::core::take" => eval_vec_take(args, env, sym),
        ":wat::core::drop" => eval_vec_drop(args, env, sym),
        ":wat::core::map" => eval_vec_map(args, env, sym),
        ":wat::core::foldl" => eval_vec_foldl(args, env, sym),
        ":wat::core::foldr" => eval_vec_foldr(args, env, sym),
        ":wat::core::filter" => eval_vec_filter(args, env, sym),
        ":wat::std::list::zip" => eval_list_zip(args, env, sym),
        ":wat::std::list::window" => eval_list_window(args, env, sym),
        ":wat::std::list::remove-at" => eval_list_remove_at(args, env, sym),
        ":wat::std::HashMap" => eval_hashmap_ctor(args, env, sym),
        ":wat::std::HashSet" => eval_hashset_ctor(args, env, sym),
        ":wat::std::get" => eval_get(args, env, sym),
        ":wat::std::contains?" => eval_hashmap_contains(args, env, sym),
        ":wat::std::member?" => eval_hashset_member(args, env, sym),
        // :wat::io::IOReader / :wat::io::IOWriter — abstract IO
        // substrate (arc 008 slice 2). Two wat-level types; multiple
        // concrete backings (real stdio, StringIo). Byte-oriented
        // primitives with char-level conveniences.
        ":wat::io::IOReader/from-bytes" => crate::io::eval_ioreader_from_bytes(args, env, sym),
        ":wat::io::IOReader/from-string" => crate::io::eval_ioreader_from_string(args, env, sym),
        ":wat::io::IOReader/read" => crate::io::eval_ioreader_read(args, env, sym),
        ":wat::io::IOReader/read-all" => crate::io::eval_ioreader_read_all(args, env, sym),
        ":wat::io::IOReader/read-line" => crate::io::eval_ioreader_read_line(args, env, sym),
        ":wat::io::IOReader/rewind" => crate::io::eval_ioreader_rewind(args, env, sym),
        ":wat::io::IOWriter/new" => crate::io::eval_iowriter_new(args, env, sym),
        ":wat::io::IOWriter/to-bytes" => crate::io::eval_iowriter_to_bytes(args, env, sym),
        ":wat::io::IOWriter/to-string" => crate::io::eval_iowriter_to_string(args, env, sym),
        ":wat::io::IOWriter/write" => crate::io::eval_iowriter_write(args, env, sym),
        ":wat::io::IOWriter/write-all" => crate::io::eval_iowriter_write_all(args, env, sym),
        ":wat::io::IOWriter/write-string" => crate::io::eval_iowriter_write_string(args, env, sym),
        ":wat::io::IOWriter/print" => crate::io::eval_iowriter_print(args, env, sym),
        ":wat::io::IOWriter/println" => crate::io::eval_iowriter_println(args, env, sym),
        ":wat::io::IOWriter/writeln" => crate::io::eval_iowriter_writeln(args, env, sym),
        ":wat::io::IOWriter/flush" => crate::io::eval_iowriter_flush(args, env, sym),

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
        ":wat::kernel::run-sandboxed" => crate::sandbox::eval_kernel_run_sandboxed(args, env, sym),
        ":wat::kernel::run-sandboxed-hermetic" => {
            crate::sandbox::eval_kernel_run_sandboxed_hermetic(args, env, sym)
        }
        ":wat::kernel::run-sandboxed-ast" => {
            crate::sandbox::eval_kernel_run_sandboxed_ast(args, env, sym)
        }
        ":wat::kernel::assertion-failed!" => {
            crate::assertion::eval_kernel_assertion_failed(args, env, sym)
        }
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

        // :wat::core::use! — resolve-pass declaration, no-op at runtime.
        // Validation happens during resolve; by the time eval runs, the
        // declaration has done its job. Returns :() for the value
        // position (if a user writes it inside an expression — unusual
        // but not illegal).
        ":wat::core::use!" => Ok(Value::Unit),

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

        // :rust::* — dispatch through the rust-deps registry. Each
        // symbol's shim handles its own arg evaluation and marshaling.
        other if other.starts_with(":rust::") => {
            let registry = crate::rust_deps::get();
            match registry.get_symbol(other) {
                Some(sym_entry) => (sym_entry.dispatch)(args, env, sym),
                None => Err(RuntimeError::UnknownFunction(format!(
                    "{} is not registered in the rust-deps registry",
                    other
                ))),
            }
        }

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
            apply_function(func, vals, sym)
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
        // Parse for validation side-effect — `:Any` and malformed type
        // expressions surface here before runtime evaluation begins.
        // The parsed type itself isn't consumed at runtime; the type
        // checker handles the actual type-level work earlier in the
        // startup pipeline.
        if let WatAST::Keyword(k) = &binder[1] {
            parse_type_keyword(k)?;
        }
        return Ok(LetBinding::Single {
            name,
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

/// `(:wat::core::if cond -> :T then else)` — typed conditional per
/// the 2026-04-20 INSCRIPTION. Both branches must produce `:T`; the
/// annotation is check-time only (runtime ignores it but validates
/// the form's arity).
///
/// Arity: exactly 5 args. Positions: [cond, `->`, `:T`, then, else].
/// The old 3-arg form is refused with a migration-hint error; this
/// is a hard break, no deprecation.
fn eval_if(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() == 3 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` now requires `-> :T` between cond and then-branch; write (:wat::core::if cond -> :T then else)".into(),
        });
    }
    if args.len() != 5 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond -> :T then else) — 5 args; got {}",
                args.len()
            ),
        });
    }
    // Validate the `-> :T` shape at runtime too — belt-and-suspenders
    // for programs that reach the dispatcher without the checker
    // having run.
    match &args[1] {
        WatAST::Symbol(s) if s.as_str() == "->" => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: format!(
                    "expected `->` at position 2; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    match &args[2] {
        WatAST::Keyword(_) => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: format!(
                    "expected type keyword at position 3 (after `->`); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    let cond_val = eval(&args[0], env, sym)?;
    match cond_val {
        Value::bool(true) => eval(&args[3], env, sym),
        Value::bool(false) => eval(&args[4], env, sym),
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

/// `:wat::core::u8 <i64-expr>` — range-checked cast from `:i64` to
/// `:u8`. Arc 008 slice 1. Rejects values outside 0..=255 at runtime
/// with a MalformedForm describing the offending value. The argument
/// type is enforced statically; this primitive only runs if the
/// checker saw an `:i64` at the call site.
fn eval_u8_cast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::u8".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    match v {
        Value::i64(n) => {
            if !(0..=255).contains(&n) {
                return Err(RuntimeError::MalformedForm {
                    head: ":wat::core::u8".into(),
                    reason: format!("value {} out of :u8 range 0..=255", n),
                });
            }
            Ok(Value::u8(n as u8))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::core::u8".into(),
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

/// `:wat::core::=` — structural equality. Composites (Vec, Tuple,
/// Option, Result, Struct) compare element-/field-wise; primitives
/// fall through to the `eval_compare` path. Split from `eval_compare`
/// because equality generalizes cleanly over composite values while
/// ordering (`<`, `>`, `<=`, `>=`) does not — a Vec of structs has no
/// canonical ordering worth inventing here.
fn eval_eq(
    head: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
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
    match values_equal(&a, &b) {
        Some(eq) => Ok(Value::bool(eq)),
        None => Err(RuntimeError::TypeMismatch {
            op: head.into(),
            expected: "matching comparable pair",
            got: a.type_name(),
        }),
    }
}

/// Structural equality on [`Value`] — returns `Some(bool)` for pairs
/// whose types support equality, `None` for pairs whose shapes aren't
/// comparable at all (e.g., comparing a `Value::Function` to anything;
/// two values of different top-level kinds; a struct to a tuple).
///
/// f64 uses `PartialEq`; `NaN == NaN` is false (Rust's standard
/// IEEE-754 semantics). Callers who need exact bit equality should
/// encode through an integer representation.
fn values_equal(a: &Value, b: &Value) -> Option<bool> {
    match (a, b) {
        (Value::i64(x), Value::i64(y)) => Some(x == y),
        (Value::u8(x), Value::u8(y)) => Some(x == y),
        (Value::f64(x), Value::f64(y)) => Some(x == y),
        (Value::String(x), Value::String(y)) => Some(x == y),
        (Value::bool(x), Value::bool(y)) => Some(x == y),
        (Value::wat__core__keyword(x), Value::wat__core__keyword(y)) => Some(x == y),
        (Value::Unit, Value::Unit) => Some(true),
        (Value::Vec(xs), Value::Vec(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                match values_equal(x, y) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Option(x), Value::Option(y)) => match (&**x, &**y) {
            (None, None) => Some(true),
            (Some(_), None) | (None, Some(_)) => Some(false),
            (Some(xv), Some(yv)) => values_equal(xv, yv),
        },
        (Value::Result(x), Value::Result(y)) => match (&**x, &**y) {
            (Ok(xv), Ok(yv)) => values_equal(xv, yv),
            (Err(xv), Err(yv)) => values_equal(xv, yv),
            _ => Some(false),
        },
        (Value::Struct(x), Value::Struct(y)) => {
            if x.type_name != y.type_name {
                return Some(false);
            }
            if x.fields.len() != y.fields.len() {
                return Some(false);
            }
            for (xf, yf) in x.fields.iter().zip(y.fields.iter()) {
                match values_equal(xf, yf) {
                    Some(true) => continue,
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        _ => None,
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
        (Value::u8(x), Value::u8(y)) => x.cmp(y),
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

/// `(:wat::core::vec :T x1 x2 ...)` / `(:wat::core::list :T x1 x2 ...)` —
/// typed list/vec constructor. First argument is a TYPE KEYWORD read by
/// the type checker; the runtime transports any `Value`. Remaining args
/// are element values. Matches the `make-bounded-queue` precedent for
/// resource-like constructors — explicit `:T` is required even when
/// elements could drive inference, so the shape never depends on context.
fn eval_list_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::vec".into(),
            expected: 1,
            got: 0,
        });
    }
    if !matches!(&args[0], WatAST::Keyword(_)) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::vec".into(),
            reason: "first argument must be a type keyword (e.g., :i64)".into(),
        });
    }
    let items = args[1..]
        .iter()
        .map(|a| eval(a, env, sym))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Vec(Arc::new(items)))
}

/// `(:wat::core::conj vec item)` → new Vec with `item` appended.
/// Immutable append; wat has no mutation. The type checker enforces
/// that `item` matches the Vec's element type.
fn eval_conj(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::conj".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let vec = match eval(&args[0], env, sym)? {
        Value::Vec(v) => v,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::conj".into(),
                expected: "Vec",
                got: other.type_name(),
            });
        }
    };
    let item = eval(&args[1], env, sym)?;
    let mut out = (*vec).clone();
    out.push(item);
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::tuple a b c ...)` — build a heterogeneous tuple
/// `Value::Tuple`. Arity 1+; the 0-tuple is the unit `:()` handled
/// elsewhere. Ships 2026-04-19 to support wat-source programs that
/// need to RETURN tuples (earlier slices saw tuples only as
/// primitive return values; Path-B Console needs to construct
/// `(pool, driver-handle)` in wat source).
fn eval_tuple_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::tuple".into(),
            reason: "tuple must have at least one element; the 0-tuple is :() (Unit)".into(),
        });
    }
    let items = args
        .iter()
        .map(|a| eval(a, env, sym))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Tuple(Arc::new(items)))
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
        out.push(apply_function(func.clone(), vec![x.clone()], sym)?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::foldl xs init f)` → acc. `f : (acc, item) → acc`.
/// Left-associative: `f(f(f(init, x0), x1), x2)`. Sequential's driver.
/// `:wat::core::foldr` ships alongside — see [`eval_vec_foldr`].
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
        acc = apply_function(func.clone(), vec![acc, x.clone()], sym)?;
    }
    Ok(acc)
}

/// `(:wat::core::foldr xs init f)` → acc. Right-associative fold.
/// `f(x0, f(x1, f(..., f(xn, init))))`. Iterates the Vec in reverse
/// so the call stack is bounded by iteration, not recursion.
fn eval_vec_foldr(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::foldr".into(),
            expected: 3,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::foldr", eval(&args[0], env, sym)?)?;
    let mut acc = eval(&args[1], env, sym)?;
    let f = eval(&args[2], env, sym)?;
    let func = match &f {
        Value::wat__core__lambda(func) => func.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::foldr".into(),
                expected: "wat::core::lambda",
                got: other.type_name(),
            });
        }
    };
    for x in xs.iter().rev() {
        acc = apply_function(func.clone(), vec![x.clone(), acc], sym)?;
    }
    Ok(acc)
}

/// `(:wat::core::filter xs pred)` → `Vec<T>`. Keeps elements for
/// which `pred` returns `:bool true`. `pred` signature: `T -> :bool`.
fn eval_vec_filter(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::filter".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::core::filter", eval(&args[0], env, sym)?)?;
    let f = eval(&args[1], env, sym)?;
    let func = match &f {
        Value::wat__core__lambda(func) => func.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::filter".into(),
                expected: "wat::core::lambda",
                got: other.type_name(),
            });
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for x in xs.iter() {
        match apply_function(func.clone(), vec![x.clone()], sym)? {
            Value::bool(true) => out.push(x.clone()),
            Value::bool(false) => {}
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::core::filter".into(),
                    expected: "bool",
                    got: other.type_name(),
                });
            }
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::std::list::zip xs ys)` → `Vec<(T,U)>`. Short-circuits at
/// the shorter input's length (matches Rust's `xs.iter().zip(ys)`).
fn eval_list_zip(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::list::zip".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::std::list::zip", eval(&args[0], env, sym)?)?;
    let ys = require_vec(":wat::std::list::zip", eval(&args[1], env, sym)?)?;
    let n = xs.len().min(ys.len());
    let mut out = Vec::with_capacity(n);
    for (x, y) in xs.iter().zip(ys.iter()).take(n) {
        out.push(Value::Tuple(Arc::new(vec![x.clone(), y.clone()])));
    }
    Ok(Value::Vec(Arc::new(out)))
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

/// `(:wat::std::list::remove-at xs i)` → `Vec<T>`. New Vec with
/// the element at `i` removed. Out-of-range index returns the Vec
/// unchanged (rather than erroring) — matches the inline select
/// loop's "drop the disconnected receiver if it happens to be at
/// index i" idiom without requiring a pre-check. Negative i also
/// no-ops.
fn eval_list_remove_at(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::list::remove-at".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let xs = require_vec(":wat::std::list::remove-at", eval(&args[0], env, sym)?)?;
    let i = require_i64(":wat::std::list::remove-at", eval(&args[1], env, sym)?)?;
    if i < 0 || (i as usize) >= xs.len() {
        return Ok(Value::Vec(xs));
    }
    let target = i as usize;
    let mut out = Vec::with_capacity(xs.len() - 1);
    for (idx, v) in xs.iter().enumerate() {
        if idx != target {
            out.push(v.clone());
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// Canonicalize a Value to a type-tagged String key for HashMap
/// storage. Type-tags prevent cross-type collision (`42` vs `"42"`).
/// Scoped to primitive keys; composite keys (Vec, Tuple, HolonAST,
/// etc.) error.
pub(crate) fn hashmap_key(op: &str, v: &Value) -> Result<String, RuntimeError> {
    match v {
        Value::String(s) => Ok(format!("S:{}", s)),
        Value::i64(n) => Ok(format!("I:{}", n)),
        Value::f64(x) => Ok(format!("F:{}", x.to_bits())),
        Value::bool(b) => Ok(format!("B:{}", b)),
        Value::wat__core__keyword(k) => Ok(format!("K:{}", k)),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "primitive key (i64, f64, bool, String, keyword)",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::std::HashMap :(K,V) k1 v1 k2 v2 ...)` — first arg is a
/// tuple-type keyword read by the checker; remaining args are
/// alternating key/value pairs. Odd pair count errors. Duplicate
/// keys: later entries overwrite earlier.
fn eval_hashmap_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::HashMap".into(),
            expected: 1,
            got: 0,
        });
    }
    if !matches!(&args[0], WatAST::Keyword(_)) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::std::HashMap".into(),
            reason: "first argument must be a tuple type keyword :(K,V)".into(),
        });
    }
    let pairs = &args[1..];
    if !pairs.len().is_multiple_of(2) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::std::HashMap".into(),
            reason: format!(
                "arity after :(K,V) must be even (alternating key/value pairs); got {}",
                pairs.len()
            ),
        });
    }
    let mut map = std::collections::HashMap::with_capacity(pairs.len() / 2);
    for pair in pairs.chunks(2) {
        let k = eval(&pair[0], env, sym)?;
        let v = eval(&pair[1], env, sym)?;
        let key = hashmap_key(":wat::std::HashMap", &k)?;
        map.insert(key, (k, v));
    }
    Ok(Value::wat__std__HashMap(Arc::new(map)))
}

/// `(:wat::std::get container locator)` — unified accessor per
/// FOUNDATION line 2634. Dispatches on the container's runtime
/// variant:
///   - `:HashMap<K,V>` × `:K` → `:Option<V>`
///   - `:HashSet<T>`   × `:T` → `:Option<T>` (Some of the stored
///     element on membership, None on miss — round-trips the
///     caller's value)
///
/// Vec index-get graduates when a caller demands it.
fn eval_get(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::get".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let container = eval(&args[0], env, sym)?;
    let k = eval(&args[1], env, sym)?;
    match container {
        Value::wat__std__HashMap(m) => {
            let key = hashmap_key(":wat::std::get", &k)?;
            match m.get(&key) {
                Some((_stored_k, v)) => Ok(Value::Option(Arc::new(Some(v.clone())))),
                None => Ok(Value::Option(Arc::new(None))),
            }
        }
        Value::wat__std__HashSet(s) => {
            let key = hashmap_key(":wat::std::get", &k)?;
            match s.get(&key) {
                Some(stored) => Ok(Value::Option(Arc::new(Some(stored.clone())))),
                None => Ok(Value::Option(Arc::new(None))),
            }
        }
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::std::get".into(),
            expected: "HashMap | HashSet",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::std::HashSet :T x1 x2 x3 ...)` — first arg is a type
/// keyword read by the checker; remaining args are elements. Duplicate
/// elements collapse (last stored wins on the exact canonical key).
fn eval_hashset_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::HashSet".into(),
            expected: 1,
            got: 0,
        });
    }
    if !matches!(&args[0], WatAST::Keyword(_)) {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::std::HashSet".into(),
            reason: "first argument must be a type keyword (e.g., :i64)".into(),
        });
    }
    let mut set = std::collections::HashMap::with_capacity(args.len() - 1);
    for a in &args[1..] {
        let v = eval(a, env, sym)?;
        let key = hashmap_key(":wat::std::HashSet", &v)?;
        set.insert(key, v);
    }
    Ok(Value::wat__std__HashSet(Arc::new(set)))
}

/// `(:wat::std::member? s x)` — boolean membership test over
/// `:HashSet<T>`.
fn eval_hashset_member(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::member?".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let set = eval(&args[0], env, sym)?;
    let x = eval(&args[1], env, sym)?;
    match set {
        Value::wat__std__HashSet(s) => {
            let key = hashmap_key(":wat::std::member?", &x)?;
            Ok(Value::bool(s.contains_key(&key)))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::std::member?".into(),
            expected: "HashSet",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::std::contains? m k)` — boolean membership test.
fn eval_hashmap_contains(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::std::contains?".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let container = eval(&args[0], env, sym)?;
    let k = eval(&args[1], env, sym)?;
    match container {
        Value::wat__std__HashMap(m) => {
            let key = hashmap_key(":wat::std::contains?", &k)?;
            Ok(Value::bool(m.contains_key(&key)))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::std::contains?".into(),
            expected: "HashMap",
            got: other.type_name(),
        }),
    }
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

/// `(:wat::core::forms f1 f2 ... fn)` → `:Vec<wat::WatAST>`.
///
/// Variadic sibling of `quote`. Takes N unevaluated forms and returns
/// a Vec of `:wat::WatAST` values — one per form, each captured as
/// data. Semantically equivalent to
/// `(vec :wat::WatAST (quote f1) (quote f2) ... (quote fn))` but
/// without the per-form quote ceremony.
///
/// Use case: building program-as-data payloads for
/// `:wat::kernel::run-sandboxed-ast`, `:wat::core::eval-ast!`, or
/// any consumer of AST sequences. The test stdlib's `:wat::test::
/// program` macro expands directly to this.
///
/// Like `quote`, this is a special form — arguments are NOT
/// evaluated. The type checker returns `:Vec<wat::WatAST>`
/// unconditionally; see `check.rs::infer_list` for the handling.
fn eval_forms(args: &[WatAST]) -> Result<Value, RuntimeError> {
    let items: Vec<Value> = args
        .iter()
        .map(|a| Value::wat__WatAST(Arc::new(a.clone())))
        .collect();
    Ok(Value::Vec(Arc::new(items)))
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
            func.clone(),
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

/// `(Ok <expr>)` — tagged constructor for the built-in `:Result<T,E>`
/// enum. Reserved bare identifier. Arity 1. Evaluates `expr` and wraps
/// in `Value::Result(Ok(_))`.
fn eval_ok_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: "Ok".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    Ok(Value::Result(Arc::new(Ok(v))))
}

/// `(Err <expr>)` — tagged constructor for the built-in `:Result<T,E>`
/// enum. Reserved bare identifier. Arity 1. Evaluates `expr` and wraps
/// in `Value::Result(Err(_))`.
fn eval_err_ctor(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: "Err".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    Ok(Value::Result(Arc::new(Err(v))))
}

/// `(:wat::core::try <result-expr>)` — unwrap a `:Result<T,E>` to its
/// inner `T`, or short-circuit the enclosing Result-returning function
/// with `Err(e)`.
///
/// Semantics on the inner Result:
/// - `(Ok v)` — evaluates to `v`; execution continues.
/// - `(Err e)` — raises [`RuntimeError::TryPropagate(e)`]. The walker
///   unwinds through `let*` / `match` / `if` / any nested form until it
///   reaches the innermost enclosing [`apply_function`], which catches
///   the signal and packages it as the function's own `Err(e)` return
///   value.
///
/// The type checker guarantees the enclosing function is Result-typed
/// and that the propagated `E` matches. This dispatcher assumes both
/// and does not re-verify at runtime.
///
/// Type error (not a checker guarantee — the runtime still guards):
/// arg is not a `Value::Result`. Caller surfaces `TypeMismatch`.
fn eval_try(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::try".into(),
            expected: 1,
            got: args.len(),
        });
    }
    let v = eval(&args[0], env, sym)?;
    match v {
        Value::Result(r) => match std::sync::Arc::try_unwrap(r) {
            Ok(std::result::Result::Ok(ok)) => Ok(ok),
            Ok(std::result::Result::Err(e)) => Err(RuntimeError::TryPropagate(e)),
            Err(shared) => match &*shared {
                std::result::Result::Ok(ok) => Ok(ok.clone()),
                std::result::Result::Err(e) => Err(RuntimeError::TryPropagate(e.clone())),
            },
        },
        other => Err(RuntimeError::TypeMismatch {
            op: ":wat::core::try".into(),
            expected: "Result<T,E>",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::core::struct-new <type-name-keyword> <v1> <v2> ...)` — the
/// internal primitive every auto-generated `<struct>/new` constructor
/// body invokes. Users do not call this directly; they call the
/// per-struct constructor, which expands to a `struct-new` call with
/// the right type name baked in.
///
/// Validates:
/// - First arg is a keyword (the struct's type name).
/// - Remaining args evaluate; their count becomes the field count.
///
/// Emits [`Value::Struct`] with the type name and positional fields.
/// Arity vs field-count mismatch is enforced by the type checker at
/// the `<struct>/new` scheme — this primitive trusts the caller.
fn eval_struct_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::struct-new".into(),
            expected: 1,
            got: 0,
        });
    }
    let type_name = match &args[0] {
        WatAST::Keyword(k) => k.clone(),
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::struct-new".into(),
                reason: format!(
                    "first argument must be a keyword literal (the struct's type name); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    let mut fields = Vec::with_capacity(args.len() - 1);
    for arg in &args[1..] {
        fields.push(eval(arg, env, sym)?);
    }
    Ok(Value::Struct(Arc::new(StructValue { type_name, fields })))
}

/// `(:wat::core::struct-field <struct-value> <field-index>)` — the
/// internal primitive every auto-generated `<struct>/<field>` accessor
/// body invokes. Users do not call this directly; they call the
/// per-struct accessor (e.g., `:wat::algebra::CapacityExceeded/cost`),
/// which expands to a `struct-field` call with the field's index
/// baked in.
///
/// Validates:
/// - First arg evaluates to a [`Value::Struct`].
/// - Second arg is an integer literal in range `[0, fields.len())`.
///
/// Returns the field value by position. Bounds and type alignment are
/// enforced by the type checker at the `<struct>/<field>` scheme —
/// this primitive trusts the caller for well-typed programs, and
/// raises `MalformedForm` for the ill-typed runtime path.
fn eval_struct_field(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::core::struct-field".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let struct_val = eval(&args[0], env, sym)?;
    let inner = match struct_val {
        Value::Struct(s) => s,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::core::struct-field".into(),
                expected: "Struct",
                got: other.type_name(),
            });
        }
    };
    let index = match &args[1] {
        WatAST::IntLit(n) if *n >= 0 => *n as usize,
        WatAST::IntLit(n) => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::struct-field".into(),
                reason: format!("field index must be non-negative; got {}", n),
            });
        }
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::struct-field".into(),
                reason: format!(
                    "second argument must be an integer literal (the field index); got {}",
                    ast_variant_name(other)
                ),
            });
        }
    };
    if index >= inner.fields.len() {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::struct-field".into(),
            reason: format!(
                "field index {} out of range for struct {} with {} fields",
                index,
                inner.type_name,
                inner.fields.len()
            ),
        });
    }
    Ok(inner.fields[index].clone())
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
/// `(:wat::core::match scrutinee -> :T arm1 arm2 ...)` — typed
/// pattern match per the 2026-04-20 INSCRIPTION. Every arm body must
/// produce `:T`; mismatches are reported per-arm. The annotation is
/// check-time only at runtime (validated for shape, ignored for
/// dispatch).
///
/// Arity: at least 4 args (scrutinee, `->`, `:T`, one arm). The old
/// no-annotation form — `(match scrutinee arm1 ...)` — is refused
/// with a migration-hint MalformedForm. Hard break, no deprecation.
fn eval_match(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() < 4 {
        // Two bad-shape possibilities to distinguish:
        //   - Pre-inscription `(match scrutinee arm1)` — 2 args, no `->`
        //   - Too few args overall
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: if args.len() >= 2
                && !matches!(
                    args.get(1),
                    Some(WatAST::Symbol(s)) if s.as_str() == "->"
                )
            {
                "`:wat::core::match` now requires `-> :T` between scrutinee and arms; write (:wat::core::match scrut -> :T (pat body) ...)".into()
            } else {
                format!(
                    "expected (:wat::core::match scrut -> :T arm1 arm2 ...) — at least 4 args; got {}",
                    args.len()
                )
            },
        });
    }
    // Validate the `-> :T` shape.
    match &args[1] {
        WatAST::Symbol(s) if s.as_str() == "->" => {}
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "expected `->` after scrutinee (write `-> :T` between scrutinee and arms)".into(),
            });
        }
    }
    match &args[2] {
        WatAST::Keyword(_) => {}
        other => {
            return Err(RuntimeError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "expected type keyword after `->`; got {}",
                    ast_variant_name(other)
                ),
            });
        }
    }
    let scrutinee = eval(&args[0], env, sym)?;
    for arm in &args[3..] {
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
                WatAST::Symbol(ident) if ident.as_str() == "Ok" => {
                    if items.len() != 2 {
                        return Err(RuntimeError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Ok binder) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        });
                    }
                    match value {
                        Value::Result(r) => match &**r {
                            Ok(inner) => {
                                let binder = match &items[1] {
                                    WatAST::Symbol(b) => b.as_str().to_string(),
                                    other => {
                                        return Err(RuntimeError::MalformedForm {
                                            head: ":wat::core::match".into(),
                                            reason: format!(
                                                "(Ok _): binder must be a bare symbol, got {}",
                                                ast_variant_name(other)
                                            ),
                                        });
                                    }
                                };
                                Ok(Some(outer.child().bind(binder, inner.clone()).build()))
                            }
                            Err(_) => Ok(None),
                        },
                        _ => Ok(None),
                    }
                }
                WatAST::Symbol(ident) if ident.as_str() == "Err" => {
                    if items.len() != 2 {
                        return Err(RuntimeError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "(Err binder) takes exactly one field, got {}",
                                items.len() - 1
                            ),
                        });
                    }
                    match value {
                        Value::Result(r) => match &**r {
                            Err(inner) => {
                                let binder = match &items[1] {
                                    WatAST::Symbol(b) => b.as_str().to_string(),
                                    other => {
                                        return Err(RuntimeError::MalformedForm {
                                            head: ":wat::core::match".into(),
                                            reason: format!(
                                                "(Err _): binder must be a bare symbol, got {}",
                                                ast_variant_name(other)
                                            ),
                                        });
                                    }
                                };
                                Ok(Some(outer.child().bind(binder, inner.clone()).build()))
                            }
                            Ok(_) => Ok(None),
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

/// `(:wat::algebra::Bundle <list-of-holons>)` — superposition, with
/// Kanerva-capacity enforcement per the committed capacity-mode.
///
/// Return type is `:Result<:holon::HolonAST, :wat::algebra::CapacityExceeded>`.
/// Always. Under every mode. Callers are forced by the type system to
/// acknowledge the possibility of failure — either matching on the
/// Result explicitly or propagating with `:wat::core::try`.
///
/// Capacity math: `budget = floor(sqrt(dims))` per the lab's prior-art
/// trimming convention (`src/encoding/rhythm.rs` in holon-lab-trading).
/// At d=10_000 → budget 100; at d=4_096 → 64; at d=1_024 → 32. Matches
/// FOUNDATION's empirical "~100 at d=10k" statement exactly. There is
/// no codebook factor — under AST-primary, the only physical bound is
/// the noise floor, and `sqrt(d)` is the safe-side item count.
///
/// Modes (`:wat::config::CapacityMode`):
/// - `:silent` — always `Ok(h)`. No check. Author opted into risk.
/// - `:warn`   — always `Ok(h)`. `eprintln!` the cost/budget/dims
///   triple when over budget. The substrate still produces the
///   degraded vector; the author sees the diagnostic.
/// - `:error`  — `Ok(h)` under budget; `Err(CapacityExceeded{cost,
///   budget})` over. The program continues with the Err value; the
///   type system requires the caller to handle it.
/// - `:abort`  — `Ok(h)` under budget; `panic!` over, carrying the
///   cost/budget/dims diagnostic. Fail-closed: the bad frame never
///   leaves this dispatcher. No unwinding of user state.
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
    let children: Vec<HolonAST> = list
        .iter()
        .map(|v| {
            require_holon(":wat::algebra::Bundle list element", v.clone())
                .map(|h| (*h).clone())
        })
        .collect::<Result<Vec<HolonAST>, _>>()?;

    // Capacity arithmetic needs `dims` and the committed mode; both
    // live on the frozen `EncodingCtx` attached to the symbol table.
    // Without a ctx we cannot compute the budget — match the pattern
    // the other config-consuming primitives use and surface
    // NoEncodingCtx.
    let ctx = require_encoding_ctx(":wat::algebra::Bundle", sym)?;
    let dims = ctx.config.dims;
    let budget = (dims as f64).sqrt().floor() as usize;
    let cost = children.len();
    let mode = ctx.config.capacity_mode;

    // Build the Bundle AST up front — under every non-Abort mode we
    // return it wrapped; only `:abort` + overflow skips this step.
    let bundle_ast = HolonAST::bundle(children);

    if cost > budget {
        match mode {
            crate::config::CapacityMode::Silent => {
                // Measure but don't surface. Author opted out of checks;
                // the degraded vector is the expected consequence.
            }
            crate::config::CapacityMode::Warn => {
                eprintln!(
                    ":wat::algebra::Bundle: capacity exceeded — cost {} > budget {} at dims {}",
                    cost, budget, dims
                );
            }
            crate::config::CapacityMode::Error => {
                let err = Value::Struct(Arc::new(StructValue {
                    type_name: ":wat::algebra::CapacityExceeded".into(),
                    fields: vec![Value::i64(cost as i64), Value::i64(budget as i64)],
                }));
                return Ok(Value::Result(Arc::new(Err(err))));
            }
            crate::config::CapacityMode::Abort => {
                // Fail-closed. No unwinding; the process is done.
                panic!(
                    ":wat::algebra::Bundle: capacity exceeded under :abort — cost {} > budget {} at dims {}",
                    cost, budget, dims
                );
            }
        }
    }

    // Ok path — under budget OR under `:silent`/`:warn` over budget.
    let ok = Value::holon__HolonAST(Arc::new(bundle_ast));
    Ok(Value::Result(Arc::new(Ok(ok))))
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
    apply_function(func, vals, sym)
}

/// Apply a function to a list of argument values, evaluated under the
/// given symbol table. Arity must match the function's declared
/// parameters; mismatch returns [`RuntimeError::ArityMismatch`].
///
/// Public so the freeze module's `:user::main` invocation and
/// constrained-eval paths can apply pre-registered functions from a
/// frozen world without duplicating the param-binding logic.
///
/// ## Tail-call trampoline (TCO, Stage 1 — named defines)
///
/// The body runs inside a loop that catches
/// [`RuntimeError::TailCall`]. When `eval_tail` recognizes a
/// user-defined function call in tail position it emits `TailCall`
/// carrying the next function and its already-evaluated args; this
/// loop reassigns `cur_func`/`cur_args` and re-iterates without
/// recursing. Rust stack stays constant across arbitrary
/// tail-recursion depth (`Console/loop`, `Cache/loop-step`, any
/// `gen_server`-shaped driver). See
/// `docs/arc/2026/04/003-tail-call-optimization/DESIGN.md` for the
/// full treatment.
///
/// Lambda self-tail-calls still consume stack in Stage 1 — the
/// evaluator's user-function-call detection keys on
/// `sym.functions`, which holds named defines only. A lambda body
/// that tail-calls a *named* define IS covered: the signal fires
/// at the named call, this loop catches it exactly as it does for
/// a define calling itself. Stage 2 extends detection to
/// lambda-valued calls.
pub fn apply_function(
    func: Arc<Function>,
    args: Vec<Value>,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let mut cur_func = func;
    let mut cur_args = args;
    loop {
        if cur_args.len() != cur_func.params.len() {
            return Err(RuntimeError::ArityMismatch {
                op: cur_func.name.clone().unwrap_or_else(|| "<lambda>".into()),
                expected: cur_func.params.len(),
                got: cur_args.len(),
            });
        }
        // Build the call env: parent is the closed env (lambda) or a
        // fresh root (define — the body resolves global names via sym).
        let parent = cur_func.closed_env.clone().unwrap_or_default();
        let mut builder = parent.child();
        for (name, value) in cur_func.params.iter().zip(cur_args.drain(..)) {
            builder = builder.bind(name.clone(), value);
        }
        let call_env = builder.build();
        // Evaluate the body in tail position. `eval_tail` is the
        // tail-aware sibling of `eval`; it emits `RuntimeError::TailCall`
        // when it meets a user-defined function call at the tail — the
        // match below converts that signal into loop continuation.
        //
        // `TryPropagate` keeps its legacy behavior: wrap in the
        // function's own `Err(e)` return. The type checker guarantees
        // this function's declared return type is `:Result<_,E>`
        // whenever its body contains a `try`, so the wrap is
        // type-correct by construction.
        match eval_tail(&cur_func.body, &call_env, sym) {
            Ok(v) => return Ok(v),
            Err(RuntimeError::TailCall { func: next, args: next_args }) => {
                cur_func = next;
                cur_args = next_args;
                continue;
            }
            Err(RuntimeError::TryPropagate(e)) => {
                return Ok(Value::Result(Arc::new(Err(e))));
            }
            Err(other) => return Err(other),
        }
    }
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
/// stop flag as a `:bool`. The wat's signal handler sets the flag
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
/// accepted by the channel OR every receiver has been dropped.
/// Returns `:Option<()>`: `(Some ())` on a successful send,
/// `:None` when the receiver is gone. Type scheme
/// `∀T. crossbeam_channel::Sender<T> -> T -> :Option<()>`.
///
/// Symmetric with `recv` — both endpoints report disconnect through
/// the same `:Option` shape. Producers write
/// `(match (send tx v) -> :() ((Some _) (loop ...)) (:None ()))`
/// to flush state and exit cleanly when the consumer drops. Prior
/// behavior (raising `ChannelDisconnected` on the send path) is
/// retired — it forced callers to either `try` or panic, which
/// breaks the clean shutdown cascade the stream stdlib wants. The
/// runtime transports any `Value` through the channel; the type
/// checker enforces that the declared `Sender<T>` matches the
/// value's type.
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
                expected: "rust::crossbeam_channel::Sender",
                got: other.type_name(),
            });
        }
    };
    let msg = eval(&args[1], env, sym)?;
    match sender.send(msg) {
        Ok(()) => Ok(Value::Option(Arc::new(Some(Value::Unit)))),
        Err(_) => Ok(Value::Option(Arc::new(None))),
    }
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
                expected: "rust::crossbeam_channel::Receiver",
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
                expected: "rust::crossbeam_channel::Receiver",
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
            expected: "rust::crossbeam_channel::Sender | rust::crossbeam_channel::Receiver",
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
                    expected: "rust::crossbeam_channel::Receiver",
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
            expected: 1, // minimum — function keyword path or lambda value
            got: 0,
        });
    }
    // First argument: keyword path (look up in sym.functions) or any
    // expression evaluating to a lambda value. Both produce an
    // Arc<Function>; the trampoline inside apply_function handles
    // closed_env for lambdas and fresh root for defines.
    let func = match &args[0] {
        WatAST::Keyword(k) => match sym.get(k) {
            Some(f) => f.clone(),
            None => return Err(RuntimeError::UnknownFunction(k.clone())),
        },
        _ => match eval(&args[0], env, sym)? {
            Value::wat__core__lambda(f) => f,
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: ":wat::kernel::spawn".into(),
                    expected: "function keyword path or lambda value",
                    got: other.type_name(),
                });
            }
        },
    };
    let mut arg_values = Vec::with_capacity(args.len() - 1);
    for a in &args[1..] {
        arg_values.push(eval(a, env, sym)?);
    }
    let thread_sym = sym.clone();
    let (tx, rx) = crossbeam_channel::bounded::<Result<Value, RuntimeError>>(1);
    std::thread::spawn(move || {
        let result = apply_function(func, arg_values, &thread_sym);
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

/// Map a [`RuntimeError`] to an [`EvalError`] struct value — the
/// Err payload returned by the eval-family forms on any failure
/// that isn't a control-flow signal.
///
/// Matches struct-field order `(kind, message)` from
/// [`crate::types::TypeEnv::with_builtins`]'s registration of
/// `:wat::core::EvalError`.
fn runtime_error_to_eval_error_value(err: &RuntimeError) -> Value {
    let (kind, message): (&'static str, String) = match err {
        RuntimeError::EvalVerificationFailed { err } => {
            ("verification-failed", format!("{}", err))
        }
        RuntimeError::EvalForbidsMutationForm { head } => (
            "mutation-form-refused",
            format!("eval refused mutation form: {}", head),
        ),
        RuntimeError::UnknownFunction(path) => {
            ("unknown-function", format!("unknown function: {}", path))
        }
        RuntimeError::UnboundSymbol(name) => {
            ("unbound-symbol", format!("unbound symbol: {}", name))
        }
        RuntimeError::TypeMismatch { op, expected, got } => (
            "type-mismatch",
            format!("{}: expected {}, got {}", op, expected, got),
        ),
        RuntimeError::ArityMismatch { op, expected, got } => (
            "arity-mismatch",
            format!("{}: expected {} args, got {}", op, expected, got),
        ),
        RuntimeError::ChannelDisconnected { op } => (
            "channel-disconnected",
            format!("{}: channel disconnected", op),
        ),
        RuntimeError::BadCondition { got } => {
            ("bad-condition", format!("if/when condition not :bool; got {}", got))
        }
        RuntimeError::DivisionByZero => ("division-by-zero", "division by zero".into()),
        RuntimeError::PatternMatchFailed { value_type } => (
            "pattern-match-failed",
            format!("no match arm fired for {} scrutinee", value_type),
        ),
        RuntimeError::MalformedForm { head, reason } => {
            ("malformed-form", format!("{}: {}", head, reason))
        }
        RuntimeError::NotCallable { got } => {
            ("not-callable", format!("not callable: {}", got))
        }
        // Control-flow signals (TryPropagate, and a future TailCall)
        // must NOT pass through this helper — callers filter those out
        // before reaching here. This arm exists to keep the match
        // exhaustive and name the invariant in code.
        RuntimeError::TryPropagate(_) => {
            ("runtime-error", "internal: TryPropagate reached EvalError mapper (checker invariant violation)".into())
        }
        // Fallback for variants that don't deserve a dedicated kind.
        other => ("runtime-error", format!("{}", other)),
    };
    Value::Struct(Arc::new(StructValue {
        type_name: ":wat::core::EvalError".into(),
        fields: vec![
            Value::String(Arc::new(kind.into())),
            Value::String(Arc::new(message)),
        ],
    }))
}

/// Wrap an inner evaluation's `Result<Value, RuntimeError>` as the
/// `Value::Result<V, EvalError>` the eval-family forms return.
///
/// Preserves the `TryPropagate` control-flow signal so `:wat::core::try`
/// inside eval'd code still propagates to the calling function. Every
/// other runtime error becomes `Err(EvalError{...})` as a value.
fn wrap_as_eval_result(inner: Result<Value, RuntimeError>) -> Result<Value, RuntimeError> {
    match inner {
        Ok(v) => Ok(Value::Result(Arc::new(Ok(v)))),
        Err(RuntimeError::TryPropagate(_)) => inner, // pass through
        Err(e) => {
            let err_struct = runtime_error_to_eval_error_value(&e);
            Ok(Value::Result(Arc::new(Err(err_struct))))
        }
    }
}

fn eval_form_ast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // Structural pre-check — NOT wrapped as EvalError. This is the
    // caller's syntactic shape; the type checker should have caught
    // it at startup. If it fires at runtime, it's a checker gap or
    // eval-ast! reached from a path that skipped the check (unlikely
    // but possible).
    if args.len() != 1 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-ast!".into(),
            reason: format!(
                "(:wat::core::eval-ast! <ast-value>) takes exactly 1 argument; got {}",
                args.len()
            ),
        });
    }
    // From here, any RuntimeError (except TryPropagate) becomes an
    // `EvalError` in the Err slot of the returned Value::Result. The
    // value-extraction, mutation-form refusal, and the inner eval
    // are all "dynamic evaluation" concerns.
    wrap_as_eval_result((|| -> Result<Value, RuntimeError> {
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
    })())
}

fn eval_form_edn(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // (:wat::core::eval-edn! :wat::eval::<iface> <locator>)
    // Structural arity — pre-checked; EvalError wrap starts below.
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-edn!".into(),
            reason: format!(
                "(:wat::core::eval-edn! :wat::eval::<iface> <locator>) takes exactly 2 arguments; got {}",
                args.len()
            ),
        });
    }
    wrap_as_eval_result((|| -> Result<Value, RuntimeError> {
        // Source fetch: its errors (file-not-found, bad interface,
        // locator type mismatch) are dynamic evaluation failures.
        let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
        parse_and_run(&source, env, sym)
    })())
}

fn eval_form_digest(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // (:wat::core::eval-digest! :wat::eval::<iface> <locator>
    //                            :wat::verify::digest-<algo>
    //                            :wat::verify::<iface> <hex>)
    // Structural arity pre-check.
    if args.len() != 5 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-digest!".into(),
            reason: format!(
                "(:wat::core::eval-digest! :wat::eval::<iface> <locator> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>) takes exactly 5 arguments; got {}",
                args.len()
            ),
        });
    }
    wrap_as_eval_result((|| -> Result<Value, RuntimeError> {
        let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
        let algo = parse_verify_algo_keyword(&args[2], "digest-", ":wat::core::eval-digest!")?;
        let hex = resolve_verify_payload(&args[3], &args[4], env, sym)?;
        // Verify hash of raw source bytes BEFORE parse (mirrors digest-load!).
        // Verification failure becomes EvalError{kind="verification-failed"}
        // via runtime_error_to_eval_error_value's match on
        // EvalVerificationFailed.
        crate::hash::verify_source_hash(source.as_bytes(), &algo, hex.trim())
            .map_err(|err| RuntimeError::EvalVerificationFailed { err })?;
        parse_and_run(&source, env, sym)
    })())
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
    // Structural arity pre-check.
    if args.len() != 7 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::core::eval-signed!".into(),
            reason: format!(
                "(:wat::core::eval-signed! :wat::eval::<iface> <locator> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>) takes exactly 7 arguments; got {}",
                args.len()
            ),
        });
    }
    wrap_as_eval_result((|| -> Result<Value, RuntimeError> {
        let source = resolve_eval_source(&args[0], &args[1], env, sym)?;
        let algo = parse_verify_algo_keyword(&args[2], "signed-", ":wat::core::eval-signed!")?;
        let sig_b64 = resolve_verify_payload(&args[3], &args[4], env, sym)?;
        let pk_b64 = resolve_verify_payload(&args[5], &args[6], env, sym)?;
        // Parse FIRST (sig is over canonical-EDN of parsed AST, which
        // we need the AST to compute — same discipline as signed-load!).
        let ast = parse_program(&source, ":wat::core::eval-signed!")?;
        crate::hash::verify_program_signature(&ast, &algo, sig_b64.trim(), pk_b64.trim())
            .map_err(|err| RuntimeError::EvalVerificationFailed { err })?;
        // After verify, run each form under the mutation-refusal guard.
        run_program(&ast, env, sym)
    })())
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
            Value::String(s) => {
                let loader = sym.source_loader().ok_or_else(|| {
                    RuntimeError::NoSourceLoader {
                        op: ":wat::eval::file-path".into(),
                    }
                })?;
                loader.fetch_source_file(&s, None)
                    .map(|loaded| loaded.source)
                    .map_err(|e| RuntimeError::MalformedForm {
                        head: ":wat::eval::file-path".into(),
                        reason: format!("read {:?}: {:?}", s, e),
                    })
            }
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
            Value::String(s) => {
                let loader = sym.source_loader().ok_or_else(|| {
                    RuntimeError::NoSourceLoader {
                        op: ":wat::verify::file-path".into(),
                    }
                })?;
                loader.fetch_payload_file(&s, None)
                    .map_err(|e| RuntimeError::MalformedForm {
                        head: ":wat::verify::file-path".into(),
                        reason: format!("read {:?}: {:?}", s, e),
                    })
            }
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
    use std::sync::OnceLock;

    /// The stdlib is the standard library — always available, without
    /// ceremony. Test harnesses load it once per process via
    /// `OnceLock`, then clone the resulting SymbolTable / MacroRegistry
    /// / TypeEnv per test. This mirrors what `startup_from_source` does
    /// at the stdlib phase, minus the user-source passes.
    ///
    /// Without this, `run` and `eval_expr` would hand back bare
    /// `SymbolTable::new()` values where `:wat::std::*` names resolve
    /// to `UnknownFunction` — dishonest framing of what "standard
    /// library" means.
    fn stdlib_loaded() -> &'static (SymbolTable, crate::macros::MacroRegistry) {
        static LOADED: OnceLock<(SymbolTable, crate::macros::MacroRegistry)> = OnceLock::new();
        LOADED.get_or_init(|| {
            let stdlib = crate::stdlib::stdlib_forms().expect("stdlib parses");
            let mut macros = crate::macros::MacroRegistry::new();
            let stdlib_post_macros =
                crate::macros::register_stdlib_defmacros(stdlib, &mut macros)
                    .expect("stdlib defmacros register");
            let expanded_stdlib = crate::macros::expand_all(stdlib_post_macros, &macros)
                .expect("stdlib macro expansion");
            let mut types = crate::types::TypeEnv::with_builtins();
            let stdlib_post_types =
                crate::types::register_stdlib_types(expanded_stdlib, &mut types)
                    .expect("stdlib types register");
            let mut symbols = SymbolTable::new();
            let _ = register_stdlib_defines(stdlib_post_types, &mut symbols)
                .expect("stdlib defines register");
            register_struct_methods(&types, &mut symbols)
                .expect("built-in struct methods register");
            (symbols, macros)
        })
    }

    fn run(src: &str) -> Result<Value, RuntimeError> {
        let (stdlib_sym, macros) = stdlib_loaded();
        let forms = parse_all(src).expect("parse ok");
        // Expand any stdlib-macro calls in the user source before
        // registering defines and evaluating.
        let expanded =
            crate::macros::expand_all(forms, macros).expect("macro expansion");
        let mut sym = stdlib_sym.clone();
        let rest = register_defines(expanded, &mut sym)?;
        let env = Environment::new();
        let mut last = Value::Unit;
        for form in &rest {
            last = eval(form, &env, &sym)?;
        }
        Ok(last)
    }

    fn eval_expr(src: &str) -> Result<Value, RuntimeError> {
        let (stdlib_sym, macros) = stdlib_loaded();
        let ast = parse_one(src).expect("parse ok");
        let expanded = crate::macros::expand_all(vec![ast], macros)
            .expect("macro expansion");
        let ast = expanded.into_iter().next().expect("one form in, one form out");
        eval(&ast, &Environment::new(), stdlib_sym)
    }

    /// Same as [`eval_expr`] but clones the shared stdlib SymbolTable
    /// and attaches a real filesystem loader. Tests that exercise
    /// `:wat::eval::file-path` or `:wat::verify::file-path` need the
    /// capability explicitly — arc 007 closed the direct-fs bypass,
    /// so the loader must be announced per call site.
    fn eval_expr_with_fs(src: &str) -> Result<Value, RuntimeError> {
        let (stdlib_sym, macros) = stdlib_loaded();
        let mut sym = stdlib_sym.clone();
        sym.set_source_loader(std::sync::Arc::new(crate::load::FsLoader));
        let ast = parse_one(src).expect("parse ok");
        let expanded = crate::macros::expand_all(vec![ast], macros)
            .expect("macro expansion");
        let ast = expanded.into_iter().next().expect("one form in, one form out");
        eval(&ast, &Environment::new(), &sym)
    }

    // ─── Literals ───────────────────────────────────────────────────────

    #[test]
    fn int_literal() {
        assert!(matches!(eval_expr("42").unwrap(), Value::i64(42)));
    }

    #[test]
    fn float_literal() {
        match eval_expr("2.5").unwrap() {
            Value::f64(x) => assert_eq!(x, 2.5),
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
            eval_expr("(:wat::core::if true -> :i64 1 2)").unwrap(),
            Value::i64(1)
        ));
    }

    #[test]
    fn if_false_branch() {
        assert!(matches!(
            eval_expr("(:wat::core::if false -> :i64 1 2)").unwrap(),
            Value::i64(2)
        ));
    }

    #[test]
    fn if_non_bool_rejected() {
        assert!(matches!(
            eval_expr("(:wat::core::if 42 -> :i64 1 2)"),
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
              (:wat::core::if (:wat::core::= n 0) -> :i64
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
        // Bundle now returns Result<holon::HolonAST, CapacityExceeded>
        // under every mode — end-to-end tests in `tests/wat_bundle_*`
        // exercise the four capacity-mode paths. This unit test
        // confirms the Ok wrap happens at cost <= budget (at d=1024,
        // budget=32 and we Bundle 3 atoms).
        let v = eval_with_ctx(
            r#"(:wat::algebra::Bundle
                 (:wat::core::vec :holon::HolonAST
                   (:wat::algebra::Atom "a")
                   (:wat::algebra::Atom "b")
                   (:wat::algebra::Atom "c")))"#,
            1024,
        )
        .unwrap();
        match v {
            Value::Result(r) => match &*r {
                Ok(Value::holon__HolonAST(_)) => {}
                other => panic!("expected Ok(holon::HolonAST); got {:?}", other),
            },
            other => panic!("expected Value::Result; got {:?}", other),
        }
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
    //
    // Per 2026-04-20 INSCRIPTION: eval-ast! / eval-edn! / eval-digest! /
    // eval-signed! all return :Result<holon::HolonAST, :wat::core::EvalError>
    // now. Test helpers below unwrap the Result wrap so the assertions
    // against Ok values and Err-kind strings stay concise.

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

    /// Unwrap the outer `Value::Result(Ok(v))` from an eval-family
    /// call's return; panics with diagnostic if the value isn't a
    /// Result, or if the Result is Err.
    fn eval_ok_inner(v: Value) -> Value {
        match v {
            Value::Result(r) => match &*r {
                Ok(inner) => inner.clone(),
                Err(err) => panic!(
                    "expected Ok from eval-family; got Err({:?})",
                    err
                ),
            },
            other => panic!(
                "expected Value::Result from eval-family; got {:?}",
                other
            ),
        }
    }

    /// Unwrap an eval-family Err and return its (kind, message) as
    /// strings. Panics if the value isn't a Result or isn't Err or
    /// isn't a Struct with the expected EvalError field shape.
    fn eval_err_kind_and_message(v: Value) -> (String, String) {
        match v {
            Value::Result(r) => match &*r {
                Err(err) => match err {
                    Value::Struct(sv) => {
                        assert_eq!(sv.type_name, ":wat::core::EvalError");
                        let kind = match &sv.fields[0] {
                            Value::String(s) => (**s).clone(),
                            _ => panic!("EvalError.kind not String"),
                        };
                        let msg = match &sv.fields[1] {
                            Value::String(s) => (**s).clone(),
                            _ => panic!("EvalError.message not String"),
                        };
                        (kind, msg)
                    }
                    other => panic!("expected Struct(EvalError); got {:?}", other),
                },
                Ok(inner) => panic!(
                    "expected Err from eval-family; got Ok({:?})",
                    inner
                ),
            },
            other => panic!(
                "expected Value::Result from eval-family; got {:?}",
                other
            ),
        }
    }

    #[test]
    fn eval_ast_bang_runs_a_parsed_program() {
        let program = parse_one("(:wat::core::i64::+ 40 2)").unwrap();
        let result =
            run_with_ast_local("(:wat::core::eval-ast! program)", program).unwrap();
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    #[test]
    fn eval_ast_bang_refuses_mutation_form() {
        let program = parse_one(
            r#"(:wat::core::define (:evil (x :i64) -> :i64) x)"#,
        )
        .unwrap();
        let result = run_with_ast_local("(:wat::core::eval-ast! program)", program)
            .unwrap();
        let (kind, _msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "mutation-form-refused");
    }

    #[test]
    fn eval_ast_bang_rejects_non_ast_value() {
        // Binding a string as program; eval-ast! refuses because it
        // only accepts Value::wat__WatAST (not Value::String).
        // The refusal lands as Err(EvalError{kind="type-mismatch"}),
        // NOT a RuntimeError unwind — the eval-family Result-wrap
        // per the 2026-04-20 INSCRIPTION.
        let form = parse_one(r#"(:wat::core::eval-ast! "oops")"#).unwrap();
        let result = eval(&form, &Environment::new(), &SymbolTable::new()).unwrap();
        let (kind, msg) = eval_err_kind_and_message(result);
        assert_eq!(kind, "type-mismatch");
        assert!(msg.contains("eval-ast!"));
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
        // eval-ast! returns Value::Result now; unwrap Ok to get the
        // evaluated value.
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
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
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    #[test]
    fn eval_edn_bang_unknown_iface_refused() {
        let result = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::unknown "foo")"#,
        )
        .unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
    }

    #[test]
    fn eval_edn_bang_reserved_unimplemented_iface_refused() {
        let result = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::http-path "https://example.com/x")"#,
        )
        .unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
    }

    #[test]
    fn eval_edn_bang_refuses_mutation_inside_string() {
        // The parsed AST from the string still walks through the
        // mutation-form guard — now surfaced as EvalError data.
        let result = eval_expr(
            r#"(:wat::core::eval-edn! :wat::eval::string "(:wat::core::define (:evil (x :i64) -> :i64) x)")"#,
        )
        .unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "mutation-form-refused");
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
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(2)));
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
        let result = eval_expr(&form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    #[test]
    fn eval_digest_bang_unknown_algo_refused() {
        let form = r#"(:wat::core::eval-digest!
            :wat::eval::string "(:wat::core::i64::+ 1 1)"
            :wat::verify::signed-ed25519
            :wat::verify::string "abc")"#;
        let result = eval_expr(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        // signed-ed25519 in a digest slot is a grammar error surfaced
        // as malformed-form inside the wrap.
        assert_eq!(kind, "malformed-form");
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
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
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
        let result = eval_expr(&form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "verification-failed");
    }

    #[test]
    fn eval_signed_bang_wrong_algo_kind_refused() {
        // digest-sha256 in a signed slot is a grammar error.
        let form = r#"(:wat::core::eval-signed!
            :wat::eval::string "(:wat::core::i64::+ 1 1)"
            :wat::verify::digest-sha256
            :wat::verify::string "sig"
            :wat::verify::string "pk")"#;
        let result = eval_expr(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
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
        let result = eval_expr_with_fs(&form).expect("eval");
        let _ = std::fs::remove_file(&path);
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(21)));
    }

    #[test]
    fn eval_edn_bang_file_path_missing_errors() {
        let form = r#"(:wat::core::eval-edn! :wat::eval::file-path "/nonexistent/path/abc.xyz")"#;
        let result = eval_expr_with_fs(form).unwrap();
        let (kind, _) = eval_err_kind_and_message(result);
        assert_eq!(kind, "malformed-form");
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
        let result = eval_expr_with_fs(&form).expect("eval");
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&digest_path);
        let inner = eval_ok_inner(result);
        assert!(matches!(inner, Value::i64(42)));
    }

    // ─── User signals — kernel measures, userland owns transitions ─────
    //
    // The three user-signal flags are process-lifetime AtomicBool statics
    // (KERNEL_SIGUSR1 / SIGUSR2 / SIGHUP in this file). Under cargo
    // test's default parallel execution, multiple signal tests race on
    // the shared state — one test's `reset_user_signals()` clobbers
    // another test's `set_kernel_sigusr1()` assertion, producing
    // heisenbugs.
    //
    // wat's zero-Mutex discipline forbids reaching for `std::sync::Mutex`
    // (or any equivalent spin-gate) in our own code, even in tests.
    // The honest isolation is subprocess-per-test: each signal test
    // runs its body in a child process with fresh statics. No shared
    // mutable state; no race.
    //
    // Mechanism: re-invoke the current test binary with
    // `--exact <test-path> --nocapture`, setting the env var
    // `WAT_SIGNAL_TEST_CHILD=1`. The test function checks the env at
    // entry: if set, run the body (we're the child); otherwise spawn
    // a child and assert on its exit status (we're the parent). Same
    // pattern `tests/wat_vm_cli.rs` uses to run programs in spawned
    // wat processes — just pointed at the test binary instead.

    const WAT_SIGNAL_TEST_CHILD: &str = "WAT_SIGNAL_TEST_CHILD";

    /// Run `body` in an isolated subprocess for signal tests.
    /// `test_path` is the full `module::test_name` identifier passed to
    /// cargo test's `--exact` filter. The parent spawns the current
    /// test binary scoped to that one test; the child runs `body` to
    /// completion.
    fn in_signal_subprocess(test_path: &str, body: impl FnOnce()) {
        if std::env::var(WAT_SIGNAL_TEST_CHILD).is_ok() {
            body();
            return;
        }
        let exe = std::env::current_exe().expect("current_exe");
        let status = std::process::Command::new(exe)
            .arg("--exact")
            .arg(test_path)
            .arg("--nocapture")
            .env(WAT_SIGNAL_TEST_CHILD, "1")
            .status()
            .expect("spawn signal-test child");
        assert!(
            status.success(),
            "signal-test child exited with failure: {:?}",
            status
        );
    }

    #[test]
    fn sigusr1_query_reflects_flag_state() {
        in_signal_subprocess(
            "runtime::tests::sigusr1_query_reflects_flag_state",
            || {
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
            },
        );
    }

    #[test]
    fn sigusr2_and_sighup_independent() {
        in_signal_subprocess(
            "runtime::tests::sigusr2_and_sighup_independent",
            || {
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
            },
        );
    }

    #[test]
    fn reset_sigusr1_flips_flag_false() {
        in_signal_subprocess(
            "runtime::tests::reset_sigusr1_flips_flag_false",
            || {
                reset_user_signals();
                set_kernel_sigusr1();
                let _ = eval_expr("(:wat::kernel::reset-sigusr1!)").expect("reset");
                match eval_expr("(:wat::kernel::sigusr1?)").unwrap() {
                    Value::bool(false) => {}
                    v => panic!("expected false after reset, got {:?}", v),
                }
            },
        );
    }

    #[test]
    fn reset_sighup_returns_unit() {
        in_signal_subprocess(
            "runtime::tests::reset_sighup_returns_unit",
            || {
                reset_user_signals();
                set_kernel_sighup();
                let v = eval_expr("(:wat::kernel::reset-sighup!)").expect("reset");
                assert!(matches!(v, Value::Unit));
            },
        );
    }

    #[test]
    fn user_signal_predicates_refuse_arguments() {
        in_signal_subprocess(
            "runtime::tests::user_signal_predicates_refuse_arguments",
            || {
                reset_user_signals();
                assert!(matches!(
                    eval_expr("(:wat::kernel::sigusr1? 1)"),
                    Err(RuntimeError::ArityMismatch { .. })
                ));
                assert!(matches!(
                    eval_expr("(:wat::kernel::reset-sigusr1! true)"),
                    Err(RuntimeError::ArityMismatch { .. })
                ));
            },
        );
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
               ((sent :Option<()>) (:wat::kernel::send tx 42)))
              (:wat::core::match (:wat::kernel::recv rx) -> :i64
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
        let v1 = eval_expr("(:wat::core::list :i64 1 2 3)").unwrap();
        let v2 = eval_expr("(:wat::core::vec :i64 1 2 3)").unwrap();
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
        match eval_expr("(:wat::core::length (:wat::core::list :i64 1 2 3))").unwrap() {
            Value::i64(3) => {}
            v => panic!("expected 3, got {:?}", v),
        }
    }

    #[test]
    fn empty_true_on_empty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::list :i64))").unwrap() {
            Value::bool(true) => {}
            v => panic!("expected true, got {:?}", v),
        }
    }

    #[test]
    fn empty_false_on_nonempty_vec() {
        match eval_expr("(:wat::core::empty? (:wat::core::list :i64 1))").unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false, got {:?}", v),
        }
    }

    #[test]
    fn reverse_flips_order() {
        match eval_expr("(:wat::core::reverse (:wat::core::list :i64 1 2 3))").unwrap() {
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
        match eval_expr("(:wat::core::take (:wat::core::list :i64 1 2 3 4 5) 3)").unwrap() {
            Value::Vec(items) => assert_eq!(items.len(), 3),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn take_more_than_length_returns_full_vec() {
        match eval_expr("(:wat::core::take (:wat::core::list :i64 1 2) 99)").unwrap() {
            Value::Vec(items) => assert_eq!(items.len(), 2),
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn drop_skips_first_n() {
        match eval_expr("(:wat::core::drop (:wat::core::list :i64 1 2 3 4 5) 2)").unwrap() {
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
              (:wat::core::list :i64 1 2 3)
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
              (:wat::core::list :i64 1 2 3 4)
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
            (:wat::std::list::window (:wat::core::list :i64 1 2 3 4) 2)
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
        match eval_expr("(:wat::core::first (:wat::core::list :i64 10 20 30))").unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn second_polymorphic_on_vec() {
        match eval_expr("(:wat::core::second (:wat::core::list :i64 10 20 30))").unwrap() {
            Value::i64(20) => {}
            v => panic!("expected 20, got {:?}", v),
        }
    }

    #[test]
    fn third_on_vec() {
        match eval_expr("(:wat::core::third (:wat::core::list :i64 10 20 30))").unwrap() {
            Value::i64(30) => {}
            v => panic!("expected 30, got {:?}", v),
        }
    }

    #[test]
    fn rest_drops_first() {
        match eval_expr("(:wat::core::rest (:wat::core::list :i64 1 2 3))").unwrap() {
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
        let err = eval_expr("(:wat::core::rest (:wat::core::list :i64))").unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn map_with_index_attaches_positions() {
        let src = r#"
            (:wat::std::list::map-with-index
              (:wat::core::list :i64 10 20 30)
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

    // ─── HashMap ───────────────────────────────────────────────────────

    #[test]
    fn hashmap_constructor_even_arity() {
        let v = eval_expr(r#"(:wat::std::HashMap :(String,i64) "a" 1 "b" 2)"#).unwrap();
        match v {
            Value::wat__std__HashMap(m) => {
                assert_eq!(m.len(), 2);
            }
            v => panic!("expected HashMap, got {:?}", v),
        }
    }

    #[test]
    fn hashmap_constructor_odd_arity_errors() {
        let err = eval_expr(r#"(:wat::std::HashMap :(String,i64) "a" 1 "b")"#).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn hashmap_get_hit_returns_some() {
        let src = r#"
            (:wat::core::let*
              (((m :rust::std::collections::HashMap<String,i64>) (:wat::std::HashMap :(String,i64) "a" 10 "b" 20)))
              (:wat::core::match (:wat::std::get m "a") -> :i64
                ((Some n) n)
                (:None 0)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(10) => {}
            v => panic!("expected 10, got {:?}", v),
        }
    }

    #[test]
    fn hashmap_get_miss_returns_none() {
        let src = r#"
            (:wat::core::let*
              (((m :rust::std::collections::HashMap<String,i64>) (:wat::std::HashMap :(String,i64) "a" 10)))
              (:wat::core::match (:wat::std::get m "missing") -> :i64
                ((Some n) n)
                (:None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (miss path), got {:?}", v),
        }
    }

    #[test]
    fn hashmap_contains_tracks_membership() {
        let src = r#"
            (:wat::core::let*
              (((m :rust::std::collections::HashMap<String,i64>) (:wat::std::HashMap :(String,i64) "a" 10)))
              (:wat::std::contains? m "a"))
        "#;
        assert!(matches!(eval_expr(src).unwrap(), Value::bool(true)));
        let src_missing = r#"
            (:wat::core::let*
              (((m :rust::std::collections::HashMap<String,i64>) (:wat::std::HashMap :(String,i64) "a" 10)))
              (:wat::std::contains? m "b"))
        "#;
        assert!(matches!(eval_expr(src_missing).unwrap(), Value::bool(false)));
    }

    #[test]
    fn hashmap_int_and_string_keys_dont_collide() {
        // "42" (String) and 42 (i64) should be distinct keys — type-tag
        // prefix in the canonical key string prevents collision.
        let src = r#"
            (:wat::core::let*
              (((m :rust::std::collections::HashMap<String,i64>)
                (:wat::std::HashMap :(String,i64) "42" 100)))
              (:wat::std::contains? m 42))
        "#;
        // Map has one entry under String "42". Contains? with i64 key 42
        // would stringify to "I:42" — different from "S:42" — no match.
        match eval_expr(src).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false (no collision), got {:?}", v),
        }
    }

    #[test]
    fn hashmap_composite_key_errors() {
        // Keys restricted to primitives in this slice.
        let err = eval_expr(r#"(:wat::std::HashMap :(Vec<i64>,String) (:wat::core::list :i64 1 2) "x")"#).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn hashmap_get_requires_hashmap_arg() {
        let err = eval_expr(r#"(:wat::std::get 42 "k")"#).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── HashSet ───────────────────────────────────────────────────────

    #[test]
    fn hashset_constructor() {
        let v = eval_expr(r#"(:wat::std::HashSet :String "a" "b" "c")"#).unwrap();
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 3),
            v => panic!("expected HashSet, got {:?}", v),
        }
    }

    #[test]
    fn hashset_collapses_duplicates() {
        let v = eval_expr(r#"(:wat::std::HashSet :String "a" "a" "b")"#).unwrap();
        match v {
            Value::wat__std__HashSet(s) => assert_eq!(s.len(), 2),
            v => panic!("expected HashSet, got {:?}", v),
        }
    }

    #[test]
    fn hashset_member_present_and_absent() {
        let present = r#"(:wat::core::let*
            (((s :rust::std::collections::HashSet<String>) (:wat::std::HashSet :String "a" "b")))
            (:wat::std::member? s "a"))"#;
        assert!(matches!(eval_expr(present).unwrap(), Value::bool(true)));
        let absent = r#"(:wat::core::let*
            (((s :rust::std::collections::HashSet<String>) (:wat::std::HashSet :String "a" "b")))
            (:wat::std::member? s "z"))"#;
        assert!(matches!(eval_expr(absent).unwrap(), Value::bool(false)));
    }

    #[test]
    fn hashset_get_returns_stored_element() {
        // (get s x) on HashSet returns (Some stored-x) on hit —
        // round-trips the caller's element through the Rust backing.
        let src = r#"
            (:wat::core::let*
              (((s :rust::std::collections::HashSet<String>) (:wat::std::HashSet :String "apple" "banana")))
              (:wat::core::match (:wat::std::get s "apple") -> :String
                ((Some x) x)
                (:None "missing")))
        "#;
        match eval_expr(src).unwrap() {
            Value::String(s) => assert_eq!(&*s, "apple"),
            v => panic!("expected \"apple\", got {:?}", v),
        }
    }

    #[test]
    fn hashset_get_miss_returns_none() {
        let src = r#"
            (:wat::core::let*
              (((s :rust::std::collections::HashSet<String>) (:wat::std::HashSet :String "apple")))
              (:wat::core::match (:wat::std::get s "banana") -> :String
                ((Some x) x)
                (:None "not-found")))
        "#;
        match eval_expr(src).unwrap() {
            Value::String(s) => assert_eq!(&*s, "not-found"),
            v => panic!("expected fallback, got {:?}", v),
        }
    }

    #[test]
    fn hashset_rejects_composite_element() {
        let err = eval_expr(r#"(:wat::std::HashSet :Vec<i64> (:wat::core::list :i64 1 2))"#).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    // ─── LocalCache (058 L1) ───────────────────────────────────────────

    #[test]
    fn local_cache_put_then_get_returns_some() {
        let src = r#"
            (:wat::core::let*
              (((cache :wat::std::LocalCache<String,i64>)
                (:wat::std::LocalCache::new 16))
               ((_ :()) (:wat::std::LocalCache::put cache "answer" 42)))
              (:wat::core::match (:wat::std::LocalCache::get cache "answer") -> :i64
                ((Some v) v)
                (:None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(42) => {}
            v => panic!("expected 42, got {:?}", v),
        }
    }

    #[test]
    fn local_cache_miss_returns_none() {
        let src = r#"
            (:wat::core::let*
              (((cache :wat::std::LocalCache<String,i64>)
                (:wat::std::LocalCache::new 16)))
              (:wat::core::match (:wat::std::LocalCache::get cache "missing") -> :i64
                ((Some v) v)
                (:None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (miss), got {:?}", v),
        }
    }

    #[test]
    fn local_cache_evicts_at_capacity() {
        // Capacity 2: after putting 3 entries, the first is evicted.
        let src = r#"
            (:wat::core::let*
              (((cache :wat::std::LocalCache<i64,i64>)
                (:wat::std::LocalCache::new 2))
               ((_ :()) (:wat::std::LocalCache::put cache 1 10))
               ((_ :()) (:wat::std::LocalCache::put cache 2 20))
               ((_ :()) (:wat::std::LocalCache::put cache 3 30)))
              (:wat::core::match (:wat::std::LocalCache::get cache 1) -> :i64
                ((Some v) v)
                (:None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(-1) => {}
            v => panic!("expected -1 (evicted), got {:?}", v),
        }
    }

    #[test]
    fn local_cache_put_overwrites_existing_key() {
        let src = r#"
            (:wat::core::let*
              (((cache :wat::std::LocalCache<String,i64>)
                (:wat::std::LocalCache::new 16))
               ((_ :()) (:wat::std::LocalCache::put cache "k" 1))
               ((_ :()) (:wat::std::LocalCache::put cache "k" 99)))
              (:wat::core::match (:wat::std::LocalCache::get cache "k") -> :i64
                ((Some v) v)
                (:None -1)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(99) => {}
            v => panic!("expected 99, got {:?}", v),
        }
    }

    // Zero-capacity rejection was a pre-dispatch guard in the
    // hand-written shim (returned RuntimeError::MalformedForm). The
    // macro-regenerated version currently panics inside `new()`'s body
    // because the macro has no way to inject pre-method validation.
    // Lands when the macro gets a `#[wat_precondition]` hook or when
    // the return-type story supports `Result<Self, RuntimeError>`
    // unwrapping. Behavior equivalent: both forms refuse capacity=0;
    // the new path just announces differently.

    #[test]
    fn thread_owned_cell_crossing_thread_boundary_errors() {
        // The generic scope guard. Same shape as the old LruCacheCell
        // test — post-#195 (macro regeneration) the lru shim uses
        // ThreadOwnedCell<WatLruCache>, so this test is now scoped to
        // the generic guard itself.
        use crate::rust_deps::ThreadOwnedCell;
        let cell: Arc<ThreadOwnedCell<i64>> = Arc::new(ThreadOwnedCell::new(1));
        cell.with_mut(":test::put", |n| {
            *n = 42;
        })
        .unwrap();

        let cell_clone = Arc::clone(&cell);
        let handle = std::thread::spawn(move || {
            cell_clone.with_mut(":test::get", |n| *n)
        });
        let child_result = handle.join().unwrap();
        assert!(
            matches!(child_result, Err(RuntimeError::MalformedForm { .. })),
            "expected cross-thread access to error, got {:?}",
            child_result
        );
        let parent_result = cell.with_mut(":test::get", |n| *n).unwrap();
        assert_eq!(parent_result, 42);
    }

    // ─── foldr / filter / zip ──────────────────────────────────────────

    #[test]
    fn foldr_is_right_associative() {
        // (foldr [1 2 3] 0 -) = 1 - (2 - (3 - 0)) = 1 - (2 - 3) = 1 - (-1) = 2
        let src = r#"
            (:wat::core::foldr
              (:wat::core::list :i64 1 2 3)
              0
              (:wat::core::lambda ((x :i64) (acc :i64) -> :i64)
                (:wat::core::i64::- x acc)))
        "#;
        match eval_expr(src).unwrap() {
            Value::i64(2) => {}
            v => panic!("expected 2, got {:?}", v),
        }
    }

    #[test]
    fn foldl_vs_foldr_differ_on_nonassoc_op() {
        // (foldl [1 2 3] 0 -) = ((0 - 1) - 2) - 3 = -6
        let src_l = r#"
            (:wat::core::foldl
              (:wat::core::list :i64 1 2 3)
              0
              (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
                (:wat::core::i64::- acc x)))
        "#;
        match eval_expr(src_l).unwrap() {
            Value::i64(-6) => {}
            v => panic!("expected -6, got {:?}", v),
        }
    }

    #[test]
    fn filter_keeps_true_predicates() {
        let src = r#"
            (:wat::core::filter
              (:wat::core::list :i64 1 2 3 4 5)
              (:wat::core::lambda ((x :i64) -> :bool)
                (:wat::core::> x 2)))
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
                assert_eq!(ns, vec![3, 4, 5]);
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn filter_refuses_non_bool_predicate() {
        let src = r#"
            (:wat::core::filter
              (:wat::core::list :i64 1 2 3)
              (:wat::core::lambda ((x :i64) -> :i64) x))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn zip_pairs_shorter_length() {
        let src = r#"
            (:wat::std::list::zip
              (:wat::core::list :i64 1 2 3)
              (:wat::core::list :String "a" "b"))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    Value::Tuple(t) => {
                        assert_eq!(t.len(), 2);
                        match (&t[0], &t[1]) {
                            (Value::i64(1), Value::String(s)) => assert_eq!(&**s, "a"),
                            other => panic!("expected (1,\"a\"); got {:?}", other),
                        }
                    }
                    v => panic!("expected Tuple, got {:?}", v),
                }
            }
            v => panic!("expected Vec, got {:?}", v),
        }
    }

    #[test]
    fn zip_empty_with_nonempty_is_empty() {
        let src = r#"
            (:wat::std::list::zip
              (:wat::core::list :i64)
              (:wat::core::list :i64 1 2 3))
        "#;
        match eval_expr(src).unwrap() {
            Value::Vec(items) => assert!(items.is_empty()),
            v => panic!("expected empty Vec, got {:?}", v),
        }
    }

    #[test]
    fn hashset_int_and_string_keys_distinct() {
        // A HashSet carrying only the String "42" shouldn't report
        // membership for the i64 42 (type-tagged canonical key).
        let src = r#"
            (:wat::core::let*
              (((s :rust::std::collections::HashSet<String>) (:wat::std::HashSet :String "42")))
              (:wat::std::member? s 42))
        "#;
        match eval_expr(src).unwrap() {
            Value::bool(false) => {}
            v => panic!("expected false (no collision), got {:?}", v),
        }
    }

    #[test]
    fn list_window_bigger_than_length_is_empty() {
        match eval_expr("(:wat::std::list::window (:wat::core::list :i64 1 2) 5)").unwrap() {
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
              (:wat::core::match (:wat::kernel::try-recv rx) -> :bool
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
               ((_ :Option<()>) (:wat::kernel::send tx 7)))
              (:wat::core::match (:wat::kernel::try-recv rx) -> :i64
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
    fn spawn_refuses_non_callable_head() {
        // Per the 2026-04-20 relaxation, spawn accepts a keyword path
        // OR any expression that evaluates to a lambda value. An int
        // literal is neither — `eval` produces Value::i64, the lambda
        // extraction fails, TypeMismatch fires.
        let err = eval_expr("(:wat::kernel::spawn 42)").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
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
            (:wat::kernel::select (:wat::core::vec :rust::crossbeam_channel::Receiver<i64>))
        "#;
        let err = eval_expr(src).unwrap_err();
        assert!(matches!(err, RuntimeError::MalformedForm { .. }));
    }

    #[test]
    fn select_refuses_non_receiver_element() {
        let src = r#"
            (:wat::kernel::select (:wat::core::vec :i64 1 2 3))
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
                (:wat::kernel::HandlePool::new "test" (:wat::core::vec :i64 1 2 3)))
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
                (:wat::kernel::HandlePool::new "empty" (:wat::core::vec :i64)))
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
                (:wat::kernel::HandlePool::new "orphaned" (:wat::core::vec :i64 1 2 3)))
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
                (:wat::kernel::HandlePool::new "named-pool" (:wat::core::vec :i64)))
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
            (:wat::kernel::HandlePool::new 42 (:wat::core::vec :i64))
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
                                 (tx :rust::crossbeam_channel::Sender<i64>)
                                 -> :())
              (:wat::core::match (:wat::kernel::send tx 99) -> :()
                ((Some _) ())
                (:None ())))
            (:wat::core::let*
              (((tx rx) (:wat::kernel::make-bounded-queue :i64 1))
               ((handle :wat::kernel::ProgramHandle<()>)
                (:wat::kernel::spawn :my::producer tx))
               ((_ :()) (:wat::kernel::join handle)))
              (:wat::core::match (:wat::kernel::recv rx) -> :i64
                ((Some v) v)
                (:None 0)))
        "#;
        match run(src).unwrap() {
            Value::i64(99) => {}
            v => panic!("expected 99, got {:?}", v),
        }
    }
}
