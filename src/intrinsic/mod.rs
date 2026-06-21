//! Intrinsic registry — arc 255. The home where wat **intrinsics** (callables
//! implemented in Rust, exposed under a `:wat::…` FQDN — `runtime.rs:23931`:
//! "intrinsics are custom Rust by definition") become registered, queryable
//! entities. The `#[wat_intrinsic]` preamble (255.1b-ii) lives over each handler
//! in this home.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! Most fields are added in the SAME strike that builds their reader:
//!   - `name` / `handler` → 255.1b-i/ii: the dispatch route (`lookup`).
//!   - `arity`            → sniffed from the `#[wat_intrinsic]` fixed-arg signature;
//!                          255.1b-iii: consumed by `metadata-of`'s intrinsic branch.
//!   - `prose` / `added` / `ret` → parsed from the `///` via `wat-doc` by the macro
//!                          (255.1b-iv-b1); consumed by `metadata-of`'s intrinsic branch.
//!   - `purity` / `determinism` → DERIVED at the reflection site (namespace deriver
//!                          via `is_effectful_op` + a small nondeterministic-set),
//!                          not stored on the entry.
//!
//! **The one bounded exception (builder-sanctioned, 2026-06-21):** `args` /
//! `examples` / `deprecated` / `see` are parsed + carried by the iv-b1 macro but
//! their reader — iv-b2's `verify-examples` reflection seam — lands one strike
//! later. They are NOT deleted (they are about to be used, not unneeded) and NOT
//! hidden: each carries an explicit `#[allow(dead_code)]` with a removal clause
//! naming iv-b2. That is the honest opposite of the pub-leak cheat an earlier
//! draft used (making the module `pub` to FAKE external use and silence the
//! signal — reverted): a dated, named, loud allow whose clause says exactly when
//! it comes off. When iv-b2's seam reads these, the allows are removed.
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, SymbolTable, Value, EvalBreak};

/// The native dispatch handler — matches the eval-fn signature exactly.
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;

/// One `@example` / `@example-norun` entry carried on the registry — the
/// structured form of `wat_doc::DocExample`, lowered to `'static` literals
/// by the `#[wat_intrinsic]` macro.
///
/// REMOVE-ALLOW when iv-b2 lands: the fields are read by the `verify-examples`
/// reflection seam then. Until that one strike, they are parsed + carried but
/// unread in a normal build — a dated, bounded `#[allow]`, not a hidden silence.
#[allow(dead_code)] // iv-b2: read by the verify-examples seam — remove this then
pub(crate) struct ExampleSubmission {
    pub expr: &'static str,
    pub expected: Option<&'static str>,
    pub run: bool,
}

/// A link-time submission of one intrinsic, gathered by `inventory`. The
/// `#[wat_intrinsic("<fqdn>")]` proc-macro emits one `inventory::submit!` of
/// this type per annotated handler; `registry()` builds itself by iterating
/// `inventory::iter::<IntrinsicSubmission>`. All fields are `'static` — the
/// macro emits string-literal fields and a fn-pointer `handler`, all of which
/// outlive the program.
///
/// Arc 255.1b-iv-b1 — the structured doc now rides each submission. `prose`,
/// `added`, and `ret` are CONSUMED by `metadata-of`'s intrinsic branch;
/// `args`, `examples`, and `see` are carried for iv-b2's verifier seam.
pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    /// GFM prose body (everything before the first `@`-tag line).
    pub prose: &'static str,
    /// `@added` version string.
    pub added: &'static str,
    /// `@arg` directives: `(name, desc)` pairs, in source order.
    pub args: &'static [(&'static str, &'static str)],
    /// `@ret` description.
    pub ret: &'static str,
    /// `@example` / `@example-norun` directives, in source order (≥1).
    pub examples: &'static [ExampleSubmission],
    /// `@deprecated (since, use_instead)`, if present.
    pub deprecated: Option<(&'static str, &'static str)>,
    /// `@see` FQDNs, in source order.
    pub see: &'static [&'static str],
}

inventory::collect!(IntrinsicSubmission);

/// One registered intrinsic's full baseline. `handler` is consumed by the
/// runtime dispatch route (`lookup`); `name`/`arity`/`prose`/`added`/`ret` are
/// consumed by `metadata-of`'s intrinsic branch (`lookup_entry`); `args`/
/// `examples`/`see` are carried for iv-b2's doctest verifier seam — every
/// field has a reader, so none is dead-code.
pub(crate) struct IntrinsicEntry {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    pub prose: &'static str,
    pub added: &'static str,
    pub ret: &'static str,
    // The iv-b2 carry: parsed + carried now, read by the `verify-examples`
    // reflection seam one strike later. REMOVE these `#[allow]`s when iv-b2 lands
    // (each is dated + named, the honest opposite of a pub-leak silence).
    #[allow(dead_code)] // iv-b2: verify-examples seam reads this — remove then
    pub args: &'static [(&'static str, &'static str)],
    #[allow(dead_code)] // iv-b2: verify-examples seam reads this — remove then
    pub examples: &'static [ExampleSubmission],
    #[allow(dead_code)] // iv-b2: verify-examples seam reads this — remove then
    pub deprecated: Option<(&'static str, &'static str)>,
    #[allow(dead_code)] // iv-b2: verify-examples seam reads this — remove then
    pub see: &'static [&'static str],
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
                prose: submission.prose,
                added: submission.added,
                args: submission.args,
                ret: submission.ret,
                examples: submission.examples,
                deprecated: submission.deprecated,
                see: submission.see,
            });
        }
        r
    })
}

mod bytes;

#[cfg(test)]
mod tests {
    use super::*;

    /// Arc 255.1b-iv-b1 confirmation: the registry entry for `core::Bytes::to-hex`
    /// carries the full structured doc — args, examples, added, ret — as proven
    /// by reading crate-private `registry().lookup_entry(...)`.
    #[test]
    fn bytes_to_hex_entry_carries_structured_doc() {
        let entry = registry()
            .lookup_entry(":wat::core::Bytes::to-hex")
            .expect("to-hex must be registered");

        // args: exactly one, named "bs"
        assert_eq!(entry.args.len(), 1, "to-hex documents exactly one @arg");
        assert_eq!(entry.args[0].0, "bs", "@arg name must match param ident 'bs'");

        // examples: at least one, and the first is run=true
        assert!(!entry.examples.is_empty(), "to-hex must carry at least one @example");
        assert!(entry.examples[0].run, "first @example must be runnable (run=true)");

        // added + ret are non-empty
        assert_eq!(entry.added, "1.0.0", "@added must be 1.0.0");
        assert!(!entry.ret.is_empty(), "@ret description must be non-empty");
    }
}
