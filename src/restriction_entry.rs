//! Arc 198 slice 2 Stone 1 — link-time registry for Rust-side substrate
//! restriction declarations.
//!
//! The wat-side `(:wat::core::def-restricted :name [prefixes] expr)` form
//! (arc 198 slice 1) lets wat source declare a caller-prefix whitelist on
//! a binding. This module provides the Rust-side analog: a struct +
//! `inventory::collect!` channel that lets Rust substrate code declare the
//! same restriction at the binding site.
//!
//! Subsequent stones plug in:
//! - **Stone 2** mints a `#[restricted_to(...)]` proc-macro attribute that
//!   emits one `inventory::submit!` per annotation.
//! - **Stone 3** applies that attribute to `eval_kernel_*_join_result`
//!   (currently policed by arc 170 Stone B's hard-coded walker rule).
//! - **Stone 4** retires the Stone B walker rule once the generic mechanism
//!   covers `*_join-result`.
//!
//! ## Wiring
//!
//! 1. `RestrictionEntry` carries two `'static` slices — `wat_name` (the
//!    binding's FQDN as it appears in wat source) and `prefixes` (the
//!    caller-FQDN whitelist, same semantics as the slice 1 wat-side
//!    declaration: trailing `::` → namespace prefix; no trailing → exact).
//! 2. `inventory::collect!(RestrictionEntry)` registers the iter target.
//!    Any crate that depends on `wat` can `inventory::submit!` entries at
//!    module scope; entries are gathered at link time.
//! 3. The startup pipeline (`startup_from_forms_post_config` in `freeze.rs`)
//!    iterates `inventory::iter::<RestrictionEntry>` AFTER all `register_defines`
//!    calls complete and BEFORE `check_program` runs, inserting each entry
//!    into `SymbolTable.defined_value_restrictions`. The existing
//!    `CheckEnv::from_symbols` mirror at slice 1's wiring then propagates
//!    the same map into `CheckEnv.defined_value_restrictions` so the walker
//!    (`validate_def_restricted_caller_namespace`) consults a unified store.
//!
//! ## Why `'static` everywhere
//!
//! `inventory::submit!` produces a static item, which means every borrowed
//! field on the submitted value must outlive the program. String literals
//! and array literals satisfy this naturally — the Stone 2 proc-macro will
//! emit submissions of the form
//! ```ignore
//! inventory::submit! {
//!     RestrictionEntry {
//!         wat_name: ":wat::kernel::some-fn",
//!         prefixes: &[":wat::"],
//!     }
//! }
//! ```
//! where both literals are `&'static str` / `&'static [&'static str]`.

/// A Rust-side declaration that some wat binding is restricted to the
/// given caller-prefix whitelist.
///
/// See module-level documentation for the role this struct plays in
/// arc 198 slice 2's wiring.
pub struct RestrictionEntry {
    /// The wat FQDN of the binding being restricted, e.g.
    /// `":wat::kernel::spawn-thread_join-result"`. Compared against call-site
    /// heads at check time.
    pub wat_name: &'static str,
    /// The allowed-caller whitelist. Each entry is either:
    /// - a namespace prefix ending in `::` (caller FQDN must start with it), or
    /// - an exact FQDN with no trailing `::` (caller FQDN must equal it).
    ///
    /// Semantics match arc 198 slice 1's wat-side `def-restricted` form.
    pub prefixes: &'static [&'static str],
}

inventory::collect!(RestrictionEntry);
