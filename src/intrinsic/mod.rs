//! Intrinsic registry — arc 255. The home where wat **intrinsics** (callables
//! implemented in Rust, exposed under a `:wat::…` FQDN — `runtime.rs:23931`:
//! "intrinsics are custom Rust by definition") become registered, queryable
//! entities. The `#[wat_intrinsic]` preamble (255.1b-ii) lives over each handler
//! in this home.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! Each baseline field is added in the SAME strike that builds its reader, so it
//! is never dead-code and never `#[allow]`/`pub`-leaked:
//!   - `name` / `handler` → 255.1b-i/ii: the dispatch route (`lookup`).
//!   - `arity`            → sniffed from the `#[wat_intrinsic]` fixed-arg signature;
//!                          255.1b-iii: consumed by `metadata-of`'s intrinsic branch.
//!   - `doc`              → sniffed from the handler's `///` docstring (Clojure-verbatim);
//!                          255.1b-iii: consumed by `metadata-of`'s intrinsic branch.
//!   - `purity` / `determinism` → DERIVED at the reflection site (namespace deriver
//!                          via `is_effectful_op` + a small nondeterministic-set),
//!                          not stored on the entry.
//! The end-state baseline is complete; the build accretes it consumer-by-consumer.
//! (Earlier draft stored the fields unread and made the module `pub` to hide the
//! dead_code — reverted; that was the pub-leak silence-the-signal cheat.)
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, SymbolTable, Value, EvalBreak};

/// The native dispatch handler — matches the eval-fn signature exactly.
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;

/// A link-time submission of one intrinsic, gathered by `inventory`. The
/// `#[wat_intrinsic("<fqdn>")]` proc-macro emits one `inventory::submit!` of
/// this type per annotated handler; `registry()` builds itself by iterating
/// `inventory::iter::<IntrinsicSubmission>`. All fields are `'static` — the
/// macro emits a string-literal `name`/`doc`, a fn-pointer `handler`, and a
/// `const` `arity`, all of which outlive the program.
///
/// Arc 255.1b-iii — the full baseline now rides each submission. `arity` and
/// `doc` are CONSUMED by `metadata-of`'s intrinsic branch (runtime.rs
/// `eval_metadata_of`), so neither is dead-code.
pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    pub doc: Option<&'static str>,
}

inventory::collect!(IntrinsicSubmission);

/// One registered intrinsic's full baseline. `handler` is consumed by the
/// runtime dispatch route (`lookup`); `name`/`arity`/`doc` are consumed by
/// `metadata-of`'s intrinsic branch (`lookup_entry`) — every field has a
/// reader, so none is dead-code.
pub(crate) struct IntrinsicEntry {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    pub doc: Option<&'static str>,
}

/// `name → entry`. Built once at startup; the dispatch route reads `handler`
/// via `lookup`, `metadata-of` reads the baseline via `lookup_entry`.
pub(crate) struct IntrinsicRegistry {
    entries: std::collections::HashMap<&'static str, IntrinsicEntry>,
}

impl IntrinsicRegistry {
    fn new() -> Self { IntrinsicRegistry { entries: std::collections::HashMap::new() } }

    /// Register an intrinsic's full baseline. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN).
    fn register(&mut self, entry: IntrinsicEntry) {
        debug_assert!(!self.entries.contains_key(entry.name), "duplicate intrinsic registration: {}", entry.name);
        self.entries.insert(entry.name, entry);
    }

    /// The dispatch route — the native handler for `name` (255.1b-i/ii).
    /// `None` = not a registered intrinsic.
    pub(crate) fn lookup(&self, name: &str) -> Option<NativeHandler> {
        self.entries.get(name).map(|e| e.handler)
    }

    /// The reflection route — the full baseline entry for `name` (255.1b-iii),
    /// read by `metadata-of`'s intrinsic branch. `None` = not registered.
    pub(crate) fn lookup_entry(&self, name: &str) -> Option<&IntrinsicEntry> {
        self.entries.get(name)
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
            r.register(IntrinsicEntry {
                name: submission.name,
                handler: submission.handler,
                arity: submission.arity,
                doc: submission.doc,
            });
        }
        r
    })
}

mod bytes;
