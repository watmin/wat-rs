//! Reserved wat-level keyword prefixes.
//!
//! [`RESERVED_PREFIXES`] — the authoritative list of language-owned prefixes.
//! [`is_reserved_prefix`] — predicate used across registration, resolution, and closure extraction.
//! [`reserved_prefix_list`] — human-readable form for error messages.

/// Reserved keyword prefixes the language owns. User definitions
/// under these paths are refused at registration time (define /
/// defmacro / type declarations).
///
/// Every consumer that renders an error message about reserved
/// prefixes should read this list via [`reserved_prefix_list`] so
/// the user-facing message stays in sync with [`is_reserved_prefix`].
pub const RESERVED_PREFIXES: &[&str] = &[
    // Arc 028 slice 4 — :wat:: reserved at the root. Covers every
    // sub-namespace (:wat::core::, :wat::holon::, etc.) AND the
    // root-level forms hoisted in this slice (:wat::load-file!,
    // :wat::eval-ast!, :wat::digest-load!, and siblings). User
    // source cannot define anything under :wat::*; substrate owns
    // the whole root.
    //
    // The :wat::load::* and :wat::eval::* sub-namespaces retired in
    // arc 028 slice 1 + 3 — the iface-keyword dispatch shape they
    // supported is gone.
    ":wat::",
    // :rust::* reserved for #[wat_dispatch]-surfaced Rust types.
    ":rust::",
    // Arc 251 Stone 251.8a — $bound is the reserved namespace every
    // non-namespaced (binder) symbol carries (see
    // `wat_reader::identifier::BOUND_NAMESPACE`). Reserved so user source
    // cannot define into it — a user-defined `$bound/x` would be
    // indistinguishable from a real local binder. Doubled-colon form to
    // match `is_reserved_prefix`'s stripping, same shape as the two
    // entries above.
    ":$bound::",
];

/// True if `keyword` falls UNDER a reserved prefix — i.e. it is, or is nested
/// beneath, a language-owned namespace (`:wat::core::+` and `:rust::lru::*` both
/// return true). Not: "is `keyword` itself one of the prefix entries." Callers use
/// it as the "this path is language-owned, don't treat it as user-definable" gate.
pub fn is_reserved_prefix(keyword: &str) -> bool {
    let stripped = keyword.strip_prefix(':').unwrap_or(keyword);
    RESERVED_PREFIXES
        .iter()
        .any(|p| stripped.starts_with(p.strip_prefix(':').unwrap_or(p)))
}

/// Human-readable comma-joined list of reserved prefixes, for use in
/// error messages. Source of truth: [`RESERVED_PREFIXES`].
pub fn reserved_prefix_list() -> String {
    RESERVED_PREFIXES.join(", ")
}
