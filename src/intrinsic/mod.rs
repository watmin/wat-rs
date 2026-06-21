//! Intrinsic registry — arc 255. The home where wat **intrinsics** (callables
//! implemented in Rust, exposed under a `:wat::…` FQDN — `runtime.rs:23931`:
//! "intrinsics are custom Rust by definition") become registered, queryable
//! entities. The `#[wat_intrinsic]` preamble (255.1b-ii) lives over each handler
//! in this home.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! This strike (255.1b-i) registers only what is *consumed*: a `name → handler`
//! map, routed by the runtime dispatch. The baseline metadata the LOCKED record
//! model defines (`arity`, `purity`, `determinism`, `expand_time`) is NOT stored
//! yet — it has no reader. Each baseline field is added in the SAME strike that
//! builds its reader, so it is never dead-code and never `#[allow]`/`pub`-leaked:
//!   - `arity`        → sniffed from the `#[wat_intrinsic]` fixed-arg signature
//!   - `purity` / `determinism` → derived (namespace deriver + nondeterministic-set)
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

/// A link-time submission of one intrinsic (fqdn → shim), gathered by
/// `inventory`. The `#[wat_intrinsic("<fqdn>")]` proc-macro emits one
/// `inventory::submit!` of this type per annotated handler; `registry()`
/// builds itself by iterating `inventory::iter::<IntrinsicSubmission>`.
/// Both fields are `'static` — the macro emits a string-literal `name`
/// and a fn-pointer `handler`, both of which outlive the program.
pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
}

inventory::collect!(IntrinsicSubmission);

/// `name → handler`. Built once at startup; consulted by runtime dispatch.
/// Grows into the full baseline ⊕ per-kind record as readers land (see module doc).
pub(crate) struct IntrinsicRegistry {
    handlers: std::collections::HashMap<&'static str, NativeHandler>,
}

impl IntrinsicRegistry {
    fn new() -> Self { IntrinsicRegistry { handlers: std::collections::HashMap::new() } }

    /// Register an intrinsic head → its native handler. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN).
    pub(crate) fn register(&mut self, name: &'static str, handler: NativeHandler) {
        debug_assert!(!self.handlers.contains_key(name), "duplicate intrinsic registration: {name}");
        self.handlers.insert(name, handler);
    }

    /// Look up an intrinsic's handler by FQDN head. `None` = not a registered intrinsic.
    pub(crate) fn lookup(&self, name: &str) -> Option<NativeHandler> {
        self.handlers.get(name).copied()
    }
}

/// The process-wide intrinsic registry, built once on first access.
pub(crate) fn registry() -> &'static IntrinsicRegistry {
    static REGISTRY: std::sync::OnceLock<IntrinsicRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut r = IntrinsicRegistry::new();
        // Each `#[wat_intrinsic("<fqdn>")]` handler submits an entry via
        // `inventory`; gather them all into the registry at first access.
        for submission in inventory::iter::<IntrinsicSubmission> {
            r.register(submission.name, submission.handler);
        }
        r
    })
}

mod bytes;
