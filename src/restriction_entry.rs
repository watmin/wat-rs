//! Arc 198 slice 2 Stone 1 — link-time registry for Rust-side substrate
//! restriction declarations.
//!
//! The user-facing form `(def :name {:restricted-to [<prefix-kw>...]} expr)`
//! (and its `defn` sibling) lets wat source declare a caller-prefix whitelist
//! on a binding via the metadata-map mechanism (Stone 241.14). This module
//! provides the Rust-side analog: a struct + `inventory::collect!` channel
//! that lets Rust substrate code declare the same restriction at the binding
//! site.
//!
//! Arc 198 originally minted `:wat::core::def-restricted` +
//! `:wat::core::defn-restricted` forms with parallel storage
//! `SymbolTable.defined_value_restrictions`. Stone 241.14 retired those forms
//! (HARD CUT) and unified restriction storage into `SymbolTable.binding_metadata`
//! (per Stone 241.6/7's metadata-map mechanism). The `RestrictionEntry`
//! inventory channel and `#[restricted_to(...)]` proc-macro surface are
//! preserved; only the populate-target changed.
//!
//! Arc 198 slice 2 introduced this module in four stones — all SHIPPED:
//! - **Stone 1** (this file) — `RestrictionEntry` struct + `inventory::collect!`
//!   channel.
//! - **Stone 2** — `#[restricted_to(...)]` proc-macro attribute that emits one
//!   `inventory::submit!` per annotation.
//! - **Stone 3** — applied to `eval_kernel_*_join_result` (policed by arc 170
//!   Stone B's walker rule).
//! - **Stone 4** — retired the Stone B hard-coded walker rule; replaced by the
//!   generic `walk_for_restricted_call` mechanism.
//!
//! ## Wiring
//!
//! 1. `RestrictionEntry` carries two `'static` slices — `wat_name` (the
//!    binding's FQDN as it appears in wat source) and `prefixes` (the
//!    caller-FQDN whitelist: trailing `::` → namespace prefix; no trailing `::` →
//!    exact FQDN match).
//! 2. `inventory::collect!(RestrictionEntry)` registers the iter target.
//!    Any crate that depends on `wat` can `inventory::submit!` entries at
//!    module scope; entries are gathered at link time.
//! 3. The startup pipeline (`startup_from_forms_post_config` in `freeze.rs`)
//!    iterates `inventory::iter::<RestrictionEntry>` AFTER all `register_defines`
//!    calls complete and BEFORE `check_program` runs. Each entry is converted
//!    to AST via `restrictions_to_binding_metadata_ast` (in `src/runtime.rs`)
//!    and inserted into `SymbolTable.binding_metadata[wat_name][":restricted-to"]`.
//!    `CheckEnv::from_symbols` mirrors the map into `CheckEnv.binding_metadata`.
//!    The `walk_for_restricted_call` walker (in `src/check.rs`) reads from
//!    `CheckEnv.binding_metadata` and validates that every call-site FQDN
//!    matches at least one prefix entry.
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
/// arc 198 slice 2's wiring and how Stone 241.14 migrated the
/// populate-target to `binding_metadata`.
pub struct RestrictionEntry {
    /// The wat FQDN of the binding being restricted, e.g.
    /// `":wat::kernel::spawn-thread_join-result"`. Compared against call-site
    /// heads at check time by `walk_for_restricted_call`.
    pub wat_name: &'static str,
    /// The allowed-caller whitelist. Each entry is either:
    /// - a namespace prefix ending in `::` (caller FQDN must start with it), or
    /// - an exact FQDN with no trailing `::` (caller FQDN must equal it).
    ///
    /// Semantics: prefix-keywords whitelist. Originally minted at arc 198 slice 1
    /// (wat-side `def-restricted` form); migrated to Stone 241.14's
    /// `binding_metadata` path. The Rust-side declaration surface
    /// (`#[restricted_to(...)]` proc-macro + this inventory channel) is unchanged.
    pub prefixes: &'static [&'static str],
}

inventory::collect!(RestrictionEntry);
