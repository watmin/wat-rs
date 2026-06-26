//! Arc 198 (post-Stone 241.14) — caller-restriction via binding metadata-map.
//!
//! Stone 241.14 retires `:wat::core::def-restricted` and
//! `:wat::core::defn-restricted`. Restrictions now live as `:restricted-to`
//! in a metadata-map on `def`/`defn`:
//!
//!   (:wat::core::defn :my::kernel::restricted-fn
//!     {:restricted-to [:my::kernel::]}
//!     [x <- :wat::core::i64] -> :wat::core::i64 x)
//!
//! The restriction walker (`walk_for_restricted_call` in check.rs) reads
//! `binding_metadata` (sole restriction store post-stone) and fires
//! `DefRestrictedCallerNotAllowed` for callers outside the whitelist.
//!
//! Prefix matching:
//! - Whitelist entry ending in `::` (e.g. `:wat::kernel::`) → caller FQDN
//!   must start with this prefix (namespace prefix match).
//! - Whitelist entry NOT ending in `::` (e.g. `:wat::kernel::specific-fn`)
//!   → caller FQDN must equal this entry exactly (exact FQDN match).

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

// ─── Test 1 — Positive prefix match ───────────────────────────────────────

#[test]
fn def_restricted_caller_inside_allowed_namespace_passes() {
    // A restricted fn declared with {:restricted-to [:my::kernel::]}. A caller
    // FQDN `:my::kernel::caller` starts with that prefix, so the walker allows.
    let src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :my::kernel::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    startup_ok(src);
}

// ─── Test 2 — Negative prefix mismatch ────────────────────────────────────

#[test]
fn def_restricted_caller_outside_allowed_namespace_fails() {
    // Same restricted fn whitelist `[:my::kernel::]` but the caller FQDN
    // `:user::app::caller` does NOT start with that prefix. Walker fires.
    let src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :user::app::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains(":my::kernel::restricted-fn"),
        "error should name the restricted callee; got: {}",
        err
    );
    assert!(
        err.contains(":user::app::caller"),
        "error should name the offending caller FQDN; got: {}",
        err
    );
    assert!(
        err.contains(":my::kernel::"),
        "error should name the whitelist prefix; got: {}",
        err
    );
}

// ─── Test 3 — Exact FQDN match (no trailing ::) ───────────────────────────

#[test]
fn def_restricted_exact_fqdn_match_only_allows_named_caller() {
    // Whitelist entry `:my::kernel::specific-caller` (no trailing `::`) is an
    // exact FQDN. Only that one caller can reach the restricted fn; a sibling
    // in the same namespace (`:my::kernel::other-caller`) fails.
    let allowed_src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::specific-caller]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :my::kernel::specific-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    startup_ok(allowed_src);

    let denied_src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::specific-caller]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :my::kernel::other-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let err = startup_err(denied_src);
    assert!(
        err.contains(":my::kernel::other-caller"),
        "error should name the denied caller (sibling in the same namespace); got: {}",
        err
    );
    assert!(
        err.contains(":my::kernel::restricted-fn"),
        "error should name the restricted callee; got: {}",
        err
    );
}

// ─── Test 4 — Multi-prefix whitelist ──────────────────────────────────────

#[test]
fn def_restricted_multi_prefix_whitelist_admits_either_namespace() {
    // Whitelist `[:my::kernel:: :my::test::]` admits any caller whose FQDN
    // starts with either prefix. Two callers — one in each namespace —
    // both pass.
    let src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel:: :my::test::]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :my::kernel::kernel-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 1))

        (:wat::core::defn :my::test::test-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 2))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    startup_ok(src);
}

// ─── Test 5 — defn metadata-map enforces restriction ──────────────────────

#[test]
fn defn_metadata_restricted_enforces_for_caller_outside_whitelist() {
    // Stone 241.14: defn-restricted is retired. Restrictions on defn live
    // as {:restricted-to [...]} metadata-map. This test proves the metadata-
    // map restriction on defn is enforced: an allowed caller passes; a
    // non-allowed caller fails with DefRestrictedCallerNotAllowed.
    //
    // Positive: caller in allowed namespace → startup succeeds.
    let positive_src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :my::kernel::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 9))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    startup_ok(positive_src);

    // Negative: caller outside allowed namespace → walker fires.
    let negative_src = r#"
        (:wat::core::defn :my::kernel::restricted-fn
          {:restricted-to [:my::kernel::]}
          [x <- :wat::core::i64] -> :wat::core::i64 x)

        (:wat::core::defn :user::app::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 9))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let err = startup_err(negative_src);
    assert!(
        err.contains(":my::kernel::restricted-fn"),
        "error should name the restricted callee; got: {}",
        err
    );
    assert!(
        err.contains(":user::app::caller"),
        "error should name the offending caller; got: {}",
        err
    );
}
