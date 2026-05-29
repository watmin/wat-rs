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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Returns the Debug-formatted error bundle from a startup that MUST fail.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts the given source starts up cleanly.
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
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
    let src = r#"
        (:wat::core::defstruct :my::Token
          {:restricted-to  [:my::issuer::]
           :field-metadata {:secret {:restricted-to [:my::issuer::]}}}
          [secret <- :wat::core::i64
           id     <- :wat::core::i64])

        (:wat::core::defn :my::issuer::mint [] -> :my::Token
          (:my::Token/new 42 99))

        (:wat::core::defn :my::issuer::get-secret
          [tok <- :my::Token] -> :wat::core::i64
          (:my::Token/secret tok))

        (:wat::core::defn :any::caller::read-id
          [tok <- :my::Token] -> :wat::core::i64
          (:my::Token/id tok))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── Test 2 — Constructor restriction fires on illegal caller ──────────────

#[test]
fn struct_restricted_ctor_restriction_fires_on_illegal_caller() {
    // Token/new is guarded by :restricted-to [:my::issuer::]. A caller in
    // namespace :user:: does NOT start with that prefix — the walker fires
    // DefRestrictedCallerNotAllowed.
    let src = r#"
        (:wat::core::defstruct :my::Token
          {:restricted-to [:my::issuer::]}
          [id <- :wat::core::i64])

        (:wat::core::defn :user::bad-mint [] -> :my::Token
          (:my::Token/new 7))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains(":my::Token/new"),
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
    let denied_src = r#"
        (:wat::core::defstruct :my::Vault
          {:restricted-to  [:my::admin::]
           :field-metadata {:secret {:restricted-to [:my::admin::]}}}
          [secret <- :wat::core::i64
           name   <- :wat::core::i64])

        (:wat::core::defn :user::outsider::read-secret
          [v <- :my::Vault] -> :wat::core::i64
          (:my::Vault/secret v))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(denied_src);
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
    let allowed_src = r#"
        (:wat::core::defstruct :my::Vault
          {:restricted-to  [:my::admin::]
           :field-metadata {:secret {:restricted-to [:my::auditor::]}}}
          [secret <- :wat::core::i64
           name   <- :wat::core::i64])

        (:wat::core::defn :my::admin::mint [] -> :my::Vault
          (:my::Vault/new 0 0))

        (:wat::core::defn :my::auditor::audit
          [v <- :my::Vault] -> :wat::core::i64
          (:my::Vault/secret v))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(allowed_src);
}

// ─── Test 4 — Public accessors unrestricted ─────────────────────────────────

#[test]
fn struct_restricted_public_accessors_unrestricted() {
    // The public field carries no :field-metadata entry. Any caller can read
    // public fields regardless of namespace — including a caller entirely
    // outside the ctor or any field whitelist.
    let src = r#"
        (:wat::core::defstruct :my::Token
          {:restricted-to  [:my::issuer::]
           :field-metadata {:private-field {:restricted-to [:my::issuer::]}}}
          [private-field <- :wat::core::i64
           public-field  <- :wat::core::i64])

        (:wat::core::defn :my::issuer::mint [] -> :my::Token
          (:my::Token/new 1 2))

        (:wat::core::defn :totally::different::ns::read-pub
          [tok <- :my::Token] -> :wat::core::i64
          (:my::Token/public-field tok))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── Test 5 — Various capability shapes ─────────────────────────────────────

#[test]
fn struct_restricted_empty_sections_honored() {
    // Case A: ctor restricted, no per-field restrictions — all fields public;
    // only whitelisted callers can mint.
    let ctor_only_src = r#"
        (:wat::core::defstruct :my::PublicToken
          {:restricted-to [:my::issuer::]}
          [payload <- :wat::core::i64])

        (:wat::core::defn :my::issuer::mint [] -> :my::PublicToken
          (:my::PublicToken/new 42))

        (:wat::core::defn :anyone::read
          [tok <- :my::PublicToken] -> :wat::core::i64
          (:my::PublicToken/payload tok))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(ctor_only_src);

    // Case B: ctor restricted + all fields restricted — only whitelisted callers
    // can read any field or mint.
    let all_restricted_src = r#"
        (:wat::core::defstruct :my::Secret
          {:restricted-to  [:my::internal::]
           :field-metadata {:data {:restricted-to [:my::internal::]}}}
          [data <- :wat::core::i64])

        (:wat::core::defn :my::internal::make [] -> :my::Secret
          (:my::Secret/new 0))

        (:wat::core::defn :my::internal::get-data
          [s <- :my::Secret] -> :wat::core::i64
          (:my::Secret/data s))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(all_restricted_src);

    // Case C: ctor restricted + field restricted — outsider cannot read data field.
    let field_denied_src = r#"
        (:wat::core::defstruct :my::Secret
          {:restricted-to  [:my::internal::]
           :field-metadata {:data {:restricted-to [:my::internal::]}}}
          [data <- :wat::core::i64])

        (:wat::core::defn :user::outsider::get-data
          [s <- :my::Secret] -> :wat::core::i64
          (:my::Secret/data s))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(field_denied_src);
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
    let empty_metadata_src = r#"
        (:wat::core::defstruct :my::Bad
          {}
          [field <- :wat::core::i64])

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(empty_metadata_src);
    assert!(
        err.contains("MalformedDecl") || err.contains("empty") || err.contains("metadata"),
        "empty metadata error should mention MalformedDecl or empty; got: {}",
        err
    );

    // Case B: legacy :wat::core::struct-restricted HARD CUT — rejected.
    let legacy_src = r#"
        (:wat::core::struct-restricted :my::Bad
          [:my::ns::])

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(legacy_src);
    assert!(
        err.contains("struct-restricted") || err.contains("retired") || err.contains("MalformedForm"),
        "legacy struct-restricted must be HARD CUT rejected; got: {}",
        err
    );
}
