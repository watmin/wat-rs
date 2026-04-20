//! Rust-symbol bindings surfaced to wat source via the `:rust::` namespace.
//!
//! A Rust SHIM is the bridge between a Rust crate's types/methods and
//! wat's runtime. Each shim:
//! 1. Registers a dispatch function per method-path keyword.
//! 2. Registers a type scheme per method-path for the type checker.
//! 3. Handles the marshaling between wat `Value` and Rust types.
//!
//! # The `:rust::` contract from a wat perspective
//!
//! wat source references Rust symbols as `:rust::<crate>::<Type>[::method]`:
//!
//! ```text
//! (:wat::core::use! :rust::lru::LruCache)
//!
//! (let* (((cache :rust::lru::LruCache<String,i64>)
//!         (:rust::lru::LruCache<String,i64>::new 16)))
//!   (:rust::lru::LruCache<String,i64>::put cache "x" 42)
//!   (:rust::lru::LruCache<String,i64>::get cache "x"))
//! ```
//!
//! # Transitivity
//!
//! wat-rs ships with its own set of default shims (lru for caching).
//! Consumer crates (e.g., holon-lab-trading) depend on wat-rs, inherit
//! those defaults via Cargo, and add their own shims (rusqlite, aya, …)
//! via the [`RustDepsBuilder`] pattern:
//!
//! ```ignore
//! let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
//! rusqlite_shim::register(&mut deps);   // consumer's shim
//! wat::run_with(deps);
//! ```
//!
//! # use! discipline
//!
//! wat programs declare their intended Rust dependencies via
//! `(:wat::core::use! :rust::<crate>::<Type>)`. Current implementation:
//! program-global set-insert (one declaration anywhere in the program
//! enables it everywhere). Per-file enforcement is a planned upgrade
//! (tracked in docs/caching-design-2026-04-19.md) pending a caller
//! that has multiple files with distinct rust deps.
//!
//! # Zero Mutex
//!
//! Shims that own mutable state use the thread-id-guard pattern (see
//! `lru::LruCacheCell`). The shim is responsible for the scope
//! discipline of its own Rust type; the registry itself is read-only
//! after initialization and carries no mutable state of its own.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::ast::WatAST;
use crate::runtime::{Environment, RuntimeError, SymbolTable, Value};

pub mod lru;
pub mod marshal;

pub use marshal::{downcast_ref_opaque, make_rust_opaque, rust_opaque_arc, FromWat, RustOpaqueInner, ToWat};

/// A Rust shim's dispatch function. Called when a wat program invokes
/// a method path registered by the shim. The `args` are the raw
/// sub-forms after the keyword head; the shim evaluates them as needed.
pub type RustDispatch = fn(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError>;

/// A scheme registration — hands the type checker everything it needs
/// to reason about a `:rust::...` call. Opaque at this layer; the
/// checker interprets it.
///
/// Stored as a raw closure that invokes the shim's type-checking logic
/// so `check.rs` doesn't need to depend on every shim individually.
pub type RustScheme = fn(
    args: &[WatAST],
    ctx: &mut dyn SchemeCtx,
) -> Option<crate::types::TypeExpr>;

/// The subset of the type checker's state that a Rust scheme needs to
/// read/write. Kept as a trait so `check.rs` can supply a concrete
/// implementation without the shim having to know the checker's
/// internal types.
pub trait SchemeCtx {
    fn fresh_var(&mut self) -> crate::types::TypeExpr;

    fn infer(&mut self, ast: &WatAST) -> Option<crate::types::TypeExpr>;

    /// Unify two types under the active substitution. Returns `true`
    /// on success, `false` if the types are incompatible. Callers
    /// typically push a type-mismatch error themselves on failure
    /// so the message can name the specific param involved.
    fn unify_types(
        &mut self,
        a: &crate::types::TypeExpr,
        b: &crate::types::TypeExpr,
    ) -> bool;

    fn apply_subst(&self, t: &crate::types::TypeExpr) -> crate::types::TypeExpr;

    fn push_type_mismatch(&mut self, callee: &str, param: &str, expected: String, got: String);

    fn push_arity_mismatch(&mut self, callee: &str, expected: usize, got: usize);

    fn push_malformed(&mut self, head: &str, reason: String);

    fn parse_type_keyword(
        &self,
        keyword: &str,
    ) -> Result<crate::types::TypeExpr, crate::types::TypeError>;
}

/// A single entry in the rust-deps registry. One per keyword-path
/// (e.g., `:rust::lru::LruCache::new`).
pub struct RustSymbol {
    /// Full keyword path, including the `:rust::` prefix.
    pub path: &'static str,
    pub dispatch: RustDispatch,
    pub scheme: RustScheme,
}

/// The declarations a shim registers about a type (distinct from
/// method paths). Used by `use!` to validate that the type itself is
/// available — e.g., `(:wat::core::use! :rust::lru::LruCache)` checks
/// for the TYPE path, not any individual method.
pub struct RustTypeDecl {
    /// Type path, e.g., `:rust::lru::LruCache`.
    pub path: &'static str,
}

/// Builder for a [`RustDepsRegistry`]. Consumer crates compose:
///
/// ```ignore
/// let mut deps = RustDepsBuilder::with_wat_rs_defaults();
/// my_shim::register(&mut deps);
/// let registry = deps.build();
/// ```
pub struct RustDepsBuilder {
    symbols: HashMap<String, RustSymbol>,
    types: HashSet<String>,
}

impl RustDepsBuilder {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            types: HashSet::new(),
        }
    }

    /// Start a builder pre-loaded with wat-rs's default shims (lru).
    pub fn with_wat_rs_defaults() -> Self {
        let mut b = Self::new();
        lru::register(&mut b);
        b
    }

    /// Register a Rust symbol (one method path). Later calls with the
    /// same path OVERWRITE — shim authors should never register the
    /// same path twice; doing so is a programming error, not a user
    /// error. We don't panic here because wat-rs tests construct
    /// registries frequently and benign reinit is harmless.
    pub fn register_symbol(&mut self, sym: RustSymbol) {
        self.symbols.insert(sym.path.to_string(), sym);
    }

    pub fn register_type(&mut self, decl: RustTypeDecl) {
        self.types.insert(decl.path.to_string());
    }

    pub fn build(self) -> RustDepsRegistry {
        RustDepsRegistry {
            symbols: self.symbols,
            types: self.types,
        }
    }
}

impl Default for RustDepsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The finalized, read-only registry of Rust symbols available to the
/// wat-vm. Installed once at startup via [`install`]; consulted by the
/// resolver (to validate `use!` decls), the type checker (to look up
/// schemes), and the runtime (to dispatch `:rust::...` calls).
pub struct RustDepsRegistry {
    symbols: HashMap<String, RustSymbol>,
    types: HashSet<String>,
}

impl RustDepsRegistry {
    pub fn get_symbol(&self, path: &str) -> Option<&RustSymbol> {
        self.symbols.get(path)
    }

    pub fn has_type(&self, path: &str) -> bool {
        self.types.contains(path)
    }

    pub fn types(&self) -> impl Iterator<Item = &str> {
        self.types.iter().map(|s| s.as_str())
    }

    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.symbols.keys().map(|s| s.as_str())
    }
}

/// Global registry slot. Set once at wat-vm startup; read by every
/// subsequent phase. [`get`] lazily initializes with wat-rs defaults
/// if nothing was installed — lets unit tests run without explicit
/// setup.
static REGISTRY: OnceLock<RustDepsRegistry> = OnceLock::new();

/// Install a consumer-built registry. Must be called before any wat
/// code runs. Returns `Err` if a registry has already been installed
/// (e.g., tests ran first with the default). Idempotent no-op wrapper
/// suitable for the `main.rs` entry of a custom wat-vm.
pub fn install(registry: RustDepsRegistry) -> Result<(), &'static str> {
    REGISTRY
        .set(registry)
        .map_err(|_| "rust_deps registry already installed")
}

/// Access the registry, lazily initializing with wat-rs defaults on
/// first access. Production binaries should call [`install`] before
/// wat code runs; tests rely on this lazy init.
pub fn get() -> &'static RustDepsRegistry {
    REGISTRY.get_or_init(|| RustDepsBuilder::with_wat_rs_defaults().build())
}

/// Program-global record of `(:wat::core::use! :rust::...)` declarations
/// encountered during load/resolve. The resolver populates this from
/// top-level use! forms; subsequent passes consult it to decide whether
/// a `:rust::X` reference is legal.
///
/// Current scope: program-global (one use! anywhere enables the symbol
/// everywhere). Per-file enforcement is a planned upgrade.
#[derive(Default, Debug, Clone)]
pub struct UseDeclarations {
    declared: HashSet<String>,
}

impl UseDeclarations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, path: String) {
        self.declared.insert(path);
    }

    pub fn contains(&self, path: &str) -> bool {
        self.declared.contains(path)
    }

    pub fn list(&self) -> impl Iterator<Item = &str> {
        self.declared.iter().map(|s| s.as_str())
    }
}
