//! Arc 203 — capability-flavored struct declarations via `:wat::core::defstruct`.
//!
//! Stone 241.8 HARD CUT retired `:wat::core::struct-restricted`. All restriction
//! metadata now lives in the defstruct metadata-map:
//!
//! ```scheme
//! (:wat::core::defstruct :Name
//!   {:restricted-to   [<ctor-whitelist-prefixes>...]    ;; guards Name/new
//!    :field-metadata  {:field-kw {:restricted-to [...]}  ;; per-field restriction
//!                      ...}}
//!   [field <- :T ...])
//! ```
//!
//! Prefix matching (inherited from arc 198):
//! - Whitelist entry ending in `::` → caller FQDN must START WITH the prefix.
//! - Whitelist entry NOT ending in `::` → caller FQDN must EQUAL the entry exactly.

use wat::freeze::startup_from_file;

/// Returns the Debug-formatted error bundle from a startup that MUST fail.
fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts the given fixture starts up cleanly.
fn startup_ok(path: &str) {
    if let Err(e) = startup_from_file(path) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

// ─── Test 1 — Form parses + struct accessors callable from whitelisted prefix ──

#[test]
fn struct_restricted_form_parses_and_accessors_callable_from_whitelist() {
    // A defstruct declaration with :restricted-to + :field-metadata compiles
    // cleanly. The auto-synthesized constructor (Token/new) is restricted to
    // :my::issuer::. The restricted accessor (Token/secret) is accessible only
    // from :my::issuer:: (per :field-metadata). The public accessor (Token/id)
    // is callable from any namespace.
    startup_ok("tests/types/struct_restricted_whitelist.wat");
}

// ─── Test 2 — Constructor restriction fires on illegal caller ──────────────

#[test]
fn struct_restricted_ctor_restriction_fires_on_illegal_caller() {
    // Token/new is guarded by :restricted-to [:my::issuer::]. A caller in
    // namespace :user:: does NOT start with that prefix — the walker fires
    // DefRestrictedCallerNotAllowed.
    let err = startup_err("tests/types/struct_restricted_ctor_denied_bad.wat");
    assert!(
        err.contains(":my::Token"),
        "error should name the restricted constructor; got: {}",
        err
    );
    assert!(
        err.contains(":user::bad-mint"),
        "error should name the offending caller; got: {}",
        err
    );
    assert!(
        err.contains(":my::issuer::"),
        "error should name the whitelist prefix; got: {}",
        err
    );
}

// ─── Test 3 — Per-field restriction fires per restricted accessor ───────────

#[test]
fn struct_restricted_per_field_restriction_fires_on_illegal_caller() {
    // A struct with one restricted field (secret) and one public field (name).
    // A caller outside the secret's whitelist trying to call Vault/secret
    // gets DefRestrictedCallerNotAllowed.
    let err = startup_err("tests/types/struct_restricted_field_denied_bad.wat");
    assert!(
        err.contains(":my::Vault/secret"),
        "error should name the restricted accessor; got: {}",
        err
    );
    assert!(
        err.contains(":user::outsider::read-secret"),
        "error should name the offending caller; got: {}",
        err
    );

    // A caller whose FQDN IS in the field's whitelist can access the restricted
    // field, even if it's not in the ctor whitelist.
    startup_ok("tests/types/struct_restricted_field_allowed.wat");
}

// ─── Test 4 — Public accessors unrestricted ─────────────────────────────────

#[test]
fn struct_restricted_public_accessors_unrestricted() {
    // The public field carries no :field-metadata entry. Any caller can read
    // public fields regardless of namespace — including a caller entirely
    // outside the ctor or any field whitelist.
    startup_ok("tests/types/struct_restricted_public_accessor.wat");
}

// ─── Test 5 — Various capability shapes ─────────────────────────────────────

#[test]
fn struct_restricted_empty_sections_honored() {
    // Case A: ctor restricted, no per-field restrictions — all fields public;
    // only whitelisted callers can mint.
    startup_ok("tests/types/struct_restricted_ctor_only.wat");

    // Case B: ctor restricted + all fields restricted — only whitelisted callers
    // can read any field or mint.
    startup_ok("tests/types/struct_restricted_all_restricted.wat");

    // Case C: ctor restricted + field restricted — outsider cannot read data field.
    let err = startup_err("tests/types/struct_restricted_field_denied_c_bad.wat");
    assert!(
        err.contains(":my::Secret/data"),
        "error should name the restricted accessor; got: {}",
        err
    );
}

// ─── Test 6 — Malformed shapes rejected ──────────────────────────────────────

#[test]
fn struct_restricted_malformed_shapes_rejected() {
    // Case A: empty metadata map {} ILLEGAL (FORM-COLLAPSE-NOTES).
    let err = startup_err("tests/types/struct_restricted_empty_metadata_bad.wat");
    assert!(
        err.contains("MalformedDecl") || err.contains("empty") || err.contains("metadata"),
        "empty metadata error should mention MalformedDecl or empty; got: {}",
        err
    );

    // Case B: legacy :wat::core::struct-restricted HARD CUT — rejected.
    let err = startup_err("tests/types/struct_restricted_legacy_bad.wat");
    assert!(
        err.contains("struct-restricted") || err.contains("retired") || err.contains("MalformedForm"),
        "legacy struct-restricted must be HARD CUT rejected; got: {}",
        err
    );
}
