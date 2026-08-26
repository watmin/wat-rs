//! The `Battery` extension-point type + battery installation. Split
//! out of `distribution/mod.rs` (arc 170) — the composition concern,
//! distinct from argv parsing and the fork/proxy/reap run path.
//!
//! `Battery` is **published surface**: see `distribution/mod.rs`'s
//! module docs for the capability statement. This file is where a
//! third-party distribution's `(register, wat_sources)` pair actually
//! gets folded into the process-global `RustDepsBuilder` + dep-source
//! registry before `:user::main` runs.

/// One `#[wat_dispatch]` extension's installation pair. Arc 100.
///
/// First element: the crate's `register(builder: &mut RustDepsBuilder)`
/// function — registers the crate's Rust shims.
///
/// Second element: the crate's `wat_sources` function — yields the
/// `&'static [WatSource]` baked into the crate.
///
/// Any `#[wat_dispatch]` extension crate exposes both functions with
/// these signatures. As of arc 278 Cache Stone 5 (the cache tooling
/// moved into core; its two study-oracle extension crates were
/// retired) this workspace ships no extension crates of its own —
/// both canonical binaries below run with an empty battery slice.
/// A downstream extension crate following the same shape (per arc
/// 013's `wat::main!` external-crate contract) drops in identically.
pub type Battery = (
    fn(&mut crate::rust_deps::RustDepsBuilder),
    fn() -> &'static [crate::WatSource],
);

/// Install every battery's `register` (Rust shims) + `wat_sources`
/// (baked wat sources). Both halves install via process-global
/// OnceLocks per `wat::run_program`'s docs.
pub(super) fn install_batteries(batteries: &[Battery]) {
    let mut builder = crate::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
    for (register, _) in batteries {
        register(&mut builder);
    }
    let _ = crate::rust_deps::install(builder.build());

    let dep_sources: Vec<&'static [crate::WatSource]> =
        batteries.iter().map(|(_, sources)| sources()).collect();
    let _ = crate::load::source::install_dep_sources(dep_sources);
}
