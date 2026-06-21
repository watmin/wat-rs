//! Builtin registry — arc 255. The single home where Rust builtins become
//! registered, queryable entities.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! This strike (255.1b-i) registers only what is *consumed*: a `name → handler`
//! map, routed by the runtime dispatch. The baseline metadata the LOCKED record
//! model defines (`arity`, `purity`, `determinism`, `expand_time`) is NOT stored
//! yet — it has no reader. Each baseline field is added in the SAME strike that
//! builds its reader, so it is never dead-code and never `#[allow]`/`pub`-leaked:
//!   - `arity`        → the dispatch-time arity-check strike
//!   - `purity` / `determinism` → the rete/`pure?`/`deterministic?` consumer strike
//!   - `expand_time`  → the macro-expand-gate strike
//!   - all of them    → the reflection strike (255.2, `metadata-of`)
//! The end-state baseline is complete; the build accretes it consumer-by-consumer.
//! (Earlier draft stored the fields unread and made the module `pub` to hide the
//! dead_code — reverted; that was the pub-leak silence-the-signal cheat.)
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, SymbolTable, Value, EvalBreak};

/// The native dispatch handler — matches the eval-fn signature exactly.
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;

/// `name → handler`. Built once at startup; consulted by runtime dispatch.
/// Grows into the full baseline ⊕ per-kind record as readers land (see module doc).
pub(crate) struct BuiltinRegistry {
    handlers: std::collections::HashMap<&'static str, NativeHandler>,
}

impl BuiltinRegistry {
    fn new() -> Self { BuiltinRegistry { handlers: std::collections::HashMap::new() } }

    /// Register a builtin head → its native handler. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN).
    pub(crate) fn register(&mut self, name: &'static str, handler: NativeHandler) {
        debug_assert!(!self.handlers.contains_key(name), "duplicate builtin registration: {name}");
        self.handlers.insert(name, handler);
    }

    /// Look up a builtin's handler by FQDN head. `None` = not a registered builtin.
    pub(crate) fn lookup(&self, name: &str) -> Option<NativeHandler> {
        self.handlers.get(name).copied()
    }
}

/// The process-wide builtin registry, built once on first access.
pub(crate) fn registry() -> &'static BuiltinRegistry {
    static REGISTRY: std::sync::OnceLock<BuiltinRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut r = BuiltinRegistry::new();
        crate::registry::bytes::register(&mut r); // each home contributes; more homes accrete
        r
    })
}

mod bytes;
