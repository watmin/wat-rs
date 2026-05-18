//! Arc 198 — `:wat::core::def-restricted` substrate primitive +
//! `:wat::core::defn-restricted` defmacro sugar.
//!
//! `def-restricted` binds a name to a value AND records an allowed-caller
//! prefix whitelist. The walker enforces the whitelist at type-check time:
//! every call site whose head names a restricted binding has its enclosing
//! fn FQDN checked against the binding's whitelist.
//!
//! Prefix matching:
//! - Whitelist entry ending in `::` (e.g. `:wat::kernel::`) → caller FQDN
//!   must start with this prefix (namespace prefix match).
//! - Whitelist entry NOT ending in `::` (e.g. `:wat::kernel::specific-fn`)
//!   → caller FQDN must equal this entry exactly (exact FQDN match).
//!
//! `defn-restricted` is a mechanical defmacro over `def-restricted` + `fn`
//! (same shape as the existing `defn` → `def` + `fn` macro at
//! `wat/core.wat:202-206`).

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
    // A restricted fn is declared with whitelist `[:my::kernel::]`. A caller
    // FQDN `:my::kernel::caller` starts with that prefix, so the walker
    // allows the call site.
    let src = r#"
        (:wat::core::def-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::]
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x))

        (:wat::core::defn :my::kernel::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── Test 2 — Negative prefix mismatch ────────────────────────────────────

#[test]
fn def_restricted_caller_outside_allowed_namespace_fails() {
    // Same restricted fn whitelist `[:my::kernel::]` but the caller FQDN
    // `:user::app::caller` does NOT start with that prefix. Walker fires.
    let src = r#"
        (:wat::core::def-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::]
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x))

        (:wat::core::defn :user::app::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
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
        (:wat::core::def-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::specific-caller]
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x))

        (:wat::core::defn :my::kernel::specific-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(allowed_src);

    let denied_src = r#"
        (:wat::core::def-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::specific-caller]
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x))

        (:wat::core::defn :my::kernel::other-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 7))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
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
        (:wat::core::def-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel:: :my::test::]
          (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x))

        (:wat::core::defn :my::kernel::kernel-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 1))

        (:wat::core::defn :my::test::test-caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 2))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
}

// ─── Test 5 — defn-restricted defmacro expansion ──────────────────────────

#[test]
fn defn_restricted_macro_expands_to_def_restricted_plus_fn() {
    // The defmacro takes (name :restricted-to [prefixes] sig body) and expands
    // to (def-restricted name :restricted-to [prefixes] (fn sig body)) —
    // semantically equivalent to using def-restricted + fn directly. Both
    // forms in this test exercise the same walker.
    //
    // Positive: caller in allowed namespace → startup succeeds.
    let positive_src = r#"
        (:wat::core::defn-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::]
          [x <- :wat::core::i64] -> :wat::core::i64
          x)

        (:wat::core::defn :my::kernel::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 9))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(positive_src);

    // Negative: caller outside allowed namespace → walker fires with the
    // SAME error variant as the def-restricted path (proving the sugar is
    // semantically equivalent to the primitive).
    let negative_src = r#"
        (:wat::core::defn-restricted :my::kernel::restricted-fn
          :restricted-to [:my::kernel::]
          [x <- :wat::core::i64] -> :wat::core::i64
          x)

        (:wat::core::defn :user::app::caller [] -> :wat::core::i64
          (:my::kernel::restricted-fn 9))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
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
