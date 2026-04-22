//! `wat::source` — the user-facing surface for wat-source
//! contribution. Arc 015 slice 4 rename (formerly
//! `wat::stdlib::StdlibFile` — renamed because community wat
//! crates don't ship "stdlib" from their authors' perspective;
//! they ship **source** that gets composed).
//!
//! # The two-part external-crate contract (arc 013)
//!
//! An external wat crate exposes two `pub fn`s:
//!
//! 1. `pub fn wat_sources() -> &'static [wat::WatSource]` — the
//!    wat-level source files (typealiases / defines / defmacros)
//!    the crate contributes. Typically baked via `include_str!`
//!    from the crate's `wat/` directory.
//! 2. `pub fn register(&mut wat::rust_deps::RustDepsBuilder)` —
//!    the Rust shim: `#[wat_dispatch]`-generated dispatch +
//!    schemes + type decls for any `:rust::*` types the crate
//!    surfaces.
//!
//! Either function may be trivial — a Rust-surface-only crate
//! returns `&[]` from `wat_sources()`; a pure-wat-extension crate
//! has a no-op `register()`. Most wrapper crates (like `wat-lru`)
//! populate both.
//!
//! # Install-once discipline
//!
//! [`install_dep_sources`] is a process-global OnceLock —
//! symmetric with [`crate::rust_deps::install`]. First caller
//! wins. One test binary / consumer binary = one consistent dep
//! set. Once installed, every subsequent freeze (main, test,
//! sandbox, fork) transparently sees the dep surface via
//! [`crate::stdlib::stdlib_forms`].

use std::sync::OnceLock;

/// One wat source file a crate contributes. A `path` (for error
/// messages and diagnostics) plus `source` (the actual wat text).
/// Both fields are `&'static str` so external crates can construct
/// these via `include_str!` without lifetime concerns.
pub struct WatSource {
    pub path: &'static str,
    pub source: &'static str,
}

/// Process-global slot holding the wat sources installed by the
/// test or binary entry point. Subsequent calls to
/// [`install_dep_sources`] fail silently — same OnceLock shape as
/// [`crate::rust_deps::install`].
static DEP_SOURCES: OnceLock<Vec<&'static [WatSource]>> = OnceLock::new();

/// Install the wat sources contributed by external crates for
/// this process. After install, every subsequent freeze (main,
/// test, sandbox via `run-sandboxed-ast`, fork child via
/// `run-hermetic-ast`) transparently sees them as part of
/// [`crate::stdlib::stdlib_forms`].
///
/// Returns `Err` if dep sources were already installed. Idempotent
/// callers can ignore the result (best-effort install).
pub fn install_dep_sources(
    sources: Vec<&'static [WatSource]>,
) -> Result<(), &'static str> {
    DEP_SOURCES
        .set(sources)
        .map_err(|_| "wat::source::install_dep_sources already called in this process")
}

/// Read the installed dep sources. Returns empty if no one has
/// called [`install_dep_sources`]. Used by
/// [`crate::stdlib::stdlib_forms`] to compose baked + installed
/// into every freeze pass.
pub fn installed_dep_sources() -> &'static [&'static [WatSource]] {
    match DEP_SOURCES.get() {
        Some(v) => v.as_slice(),
        None => &[],
    }
}
