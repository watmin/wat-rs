//! Arc 203 — `:wat::core::struct-restricted` substrate primitive.
//!
//! `struct-restricted` declares a struct AND records per-constructor +
//! per-accessor allowed-caller-prefix whitelists. The arc 198 walker
//! (`walk_for_def_restricted_call`) enforces the whitelists at type-check
//! time — no new walker code; same HashMap, same prefix-matching rules.
//!
//! Form:
//! ```scheme
//! (:wat::core::struct-restricted :Name
//!   [<ctor-whitelist-prefixes>...]          ;; slot 1 — guards Name/new
//!   ([<wlist>] field <- :T ...)             ;; slot 2 — restricted attrs
//!   (field <- :T ...))                      ;; slot 3 — public attrs
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
    // A struct-restricted declaration compiles cleanly. The auto-synthesized
    // constructor (Token/new) and restricted accessor (Token/secret) are
    // callable from the whitelisted namespace `:my::issuer::`. The public
    // accessor (Token/id) is callable from any namespace.
    let src = r#"
        (:wat::core::struct-restricted :my::Token
          [:my::issuer::]
          ([:my::issuer::] secret <- :wat::core::i64)
          (id <- :wat::core::i64))

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
    // Token/new is guarded by whitelist [:my::issuer::]. A caller in
    // namespace :user:: does NOT start with that prefix — the walker fires
    // DefRestrictedCallerNotAllowed.
    let src = r#"
        (:wat::core::struct-restricted :my::Token
          [:my::issuer::]
          ()
          (id <- :wat::core::i64))

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
    // A struct with one restricted field (secret) and one public field (id).
    // A caller outside the secret's whitelist trying to call Token/secret
    // gets DefRestrictedCallerNotAllowed. A caller outside the ctor whitelist
    // but inside a field's whitelist can still read that field.
    let denied_src = r#"
        (:wat::core::struct-restricted :my::Vault
          [:my::admin::]
          ([:my::admin::] secret <- :wat::core::i64)
          (name <- :wat::core::i64))

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

    // A caller whose FQDN IS in the field's whitelist can access the restricted field,
    // even if it's not in the ctor whitelist.
    let allowed_src = r#"
        (:wat::core::struct-restricted :my::Vault
          [:my::admin::]
          ([:my::auditor::] secret <- :wat::core::i64)
          (name <- :wat::core::i64))

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
    // The public-attrs section carries no whitelist. Any caller can read
    // public fields regardless of namespace — including a caller entirely
    // outside the ctor or any field whitelist.
    let src = r#"
        (:wat::core::struct-restricted :my::Token
          [:my::issuer::]
          ([:my::issuer::] private-field <- :wat::core::i64)
          (public-field <- :wat::core::i64))

        (:wat::core::defn :my::issuer::mint [] -> :my::Token
          (:my::Token/new 1 2))

        (:wat::core::defn :totally::different::ns::read-pub
          [tok <- :my::Token] -> :wat::core::i64
          (:my::Token/public-field tok))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── Test 5 — Empty sections honored ────────────────────────────────────────

#[test]
fn struct_restricted_empty_sections_honored() {
    // Case A: empty restricted section () — all fields are public; ctor still
    // restricted. Any caller can read the field; only whitelisted callers can
    // mint.
    let ctor_only_src = r#"
        (:wat::core::struct-restricted :my::PublicToken
          [:my::issuer::]
          ()
          (payload <- :wat::core::i64))

        (:wat::core::defn :my::issuer::mint [] -> :my::PublicToken
          (:my::PublicToken/new 42))

        (:wat::core::defn :anyone::read
          [tok <- :my::PublicToken] -> :wat::core::i64
          (:my::PublicToken/payload tok))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(ctor_only_src);

    // Case B: empty public section () — every field restricted; ctor restricted.
    // Only whitelisted callers can read any field or mint.
    let all_restricted_src = r#"
        (:wat::core::struct-restricted :my::Secret
          [:my::internal::]
          ([:my::internal::] data <- :wat::core::i64)
          ())

        (:wat::core::defn :my::internal::make [] -> :my::Secret
          (:my::Secret/new 0))

        (:wat::core::defn :my::internal::get-data
          [s <- :my::Secret] -> :wat::core::i64
          (:my::Secret/data s))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(all_restricted_src);

    // Case C: ctor restricted, empty public section — outsider cannot mint but
    // CAN the read field if in field whitelist.
    // Negative: outsider cannot call data field (restricted to :my::internal::)
    let field_denied_src = r#"
        (:wat::core::struct-restricted :my::Secret
          [:my::internal::]
          ([:my::internal::] data <- :wat::core::i64)
          ())

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
    // Case A: wrong arity — missing sections.
    let wrong_arity_src = r#"
        (:wat::core::struct-restricted :my::Bad
          [:my::ns::])

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(wrong_arity_src);
    assert!(
        err.contains("struct-restricted") || err.contains("MalformedDecl") || err.contains("args after head"),
        "wrong-arity error should mention struct-restricted or MalformedDecl; got: {}",
        err
    );

    // Case B: ctor whitelist entries must be keywords (not symbols).
    let bad_ctor_wlist_src = r#"
        (:wat::core::struct-restricted :my::Bad
          [not-a-keyword]
          ()
          (field <- :wat::core::i64))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(bad_ctor_wlist_src);
    assert!(
        err.contains("keyword") || err.contains("whitelist") || err.contains("MalformedDecl"),
        "bad ctor whitelist error should mention keywords or whitelist; got: {}",
        err
    );

    // Case C: restricted section items count not divisible by 4.
    let bad_restricted_section_src = r#"
        (:wat::core::struct-restricted :my::Bad
          [:my::ns::]
          ([:my::ns::] only-two-items)
          ())

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(bad_restricted_section_src);
    assert!(
        err.contains("4") || err.contains("chunk") || err.contains("MalformedDecl") || err.contains("divisible"),
        "bad restricted section error should mention chunk size; got: {}",
        err
    );
}
