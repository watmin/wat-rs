//! Name resolution pass.
//!
//! After macro expansion, every keyword-path reference used in call
//! position must resolve to one of:
//!
//! - A known `:wat::core::*` language form (define, fn, let, if,
//!   the builtin arithmetic / comparison / boolean ops, the list
//!   constructor, the quasiquote forms, the type-declaration heads).
//! - A root-level substrate form: `:wat::load-file!` / `:wat::load-string!` /
//!   `:wat::digest-load!` / `:wat::digest-load-string!` / `:wat::signed-load!` /
//!   `:wat::signed-load-string!` / `:wat::eval-ast!` / `:wat::eval-edn!` /
//!   `:wat::eval-file!` / `:wat::eval-digest!` / `:wat::eval-digest-string!` /
//!   `:wat::eval-signed!` / `:wat::eval-signed-string!` (arc 028 — load/eval
//!   hoisted from `:wat::core::*` with iface-keyword sub-namespaces retired).
//! - A known `:wat::holon::*` core form (`Atom`, `Bind`, `Bundle`,
//!   `Permute`, `Thermometer`, `Blend`, `cosine`, `dot`).
//! - A `:wat::kernel::*` primitive (queue / spawn / select / HandlePool /
//!   signals) — accepted here; the full kernel surface is live in runtime.
//! - A `:wat::std::*` name — accepted here; stdlib macros expand to
//!   core forms, but references that didn't expand (e.g., stdlib
//!   programs) pass through.
//! - A `:wat::config::*` setter or accessor.
//! - A `:wat::verify::*` keyword — either a verification algorithm
//!   (`:wat::verify::digest-sha256`, `:wat::verify::signed-ed25519`) or a
//!   payload-fetch interface (`:wat::verify::string`, `:wat::verify::file-path`).
//! - A user-registered `define`-function in the [`SymbolTable`].
//!
//! Anything else is an unresolved reference and halts startup with a
//! clear error citing the offending path.
//!
//! # What this pass does NOT do
//!
//! - It does NOT check bare-symbol (lexical) references. Scope-chain
//!   tracking is dynamic enough that the runtime catches those at
//!   call time via `UnboundSymbol`; a static scope walker can layer
//!   on later if strict startup-time errors are wanted.
//! - It does NOT check type-position references. That's the type
//!   checker's job (see [`crate::check`]); this pass treats type
//!   annotations and field types as opaque.
//! - It validates call-head references AND, since 251.1b, normalizes namespaced
//!   symbol refs (`wat.core/+`) to their keyword FQDN via the [`normalize`] module.
//!
//! # Module layout
//!
//! - [`error`] — [`ResolveError`] and [`UnresolvedReference`] types.
//! - [`reserved`] — [`RESERVED_PREFIXES`], [`is_reserved_prefix`], [`reserved_prefix_list`].
//! - [`rust_use`] — `:wat::core::use!` declaration collection and rust-deps coverage.
//! - [`boundary`] — the single source of truth for special-form argument
//!   boundaries (`quote_boundary`), shared by `walk` and `normalize` so the two
//!   passes cannot drift on which heads capture arguments as data.
//! - [`quote`] — quasiquote/quote boundary descent.
//! - [`normalize`] — [`normalize_symbol_refs`]: namespaced symbol-ref → keyword FQDN (arc 251.1b).
//! - [`walk`] — [`resolve_references`] entry, [`check_form`] recursive walk.
//!
//! # Warded home
//!
//! Stone 251.1a — lifted from flat `src/resolve.rs` (709 lines) into this
//! warded home. Pure structural move; zero behavior change.
//!
//! <!-- rune:vigilatum(...) PLACEHOLDER — ward earned in orchestrator's follow-up vigilia pass; do NOT self-stamp -->

mod boundary;
mod error;
mod normalize;
mod quote;
mod reserved;
mod rust_use;
mod walk;

// Public API — re-exported for the external importers (freeze.rs, lib.rs,
// macros/registry.rs, closure_extract.rs).
pub use error::{ResolveError, UnresolvedReference};
pub use normalize::normalize_symbol_refs;
pub use reserved::{is_reserved_prefix, reserved_prefix_list};
pub use walk::resolve_references;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::{register_defmacros, MacroRegistry};
    use crate::runtime::{register_defines, Environment, SymbolTable};

    /// Full pipeline helper: parse → register-defmacros → expand → register-defines → resolve.
    fn resolve(src: &str) -> Result<(), ResolveError> {
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut macros = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let env = Environment::default();
        let sym = SymbolTable::default();
        let expanded =
            crate::macros::expand_all(rest, &mut macros, &env, &sym).expect("expand");
        let mut sym = SymbolTable::new();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        resolve_references(&rest, &sym, &macros)
    }

    /// Arc 251 stone 251.1b — normalize-then-resolve pipeline helper.
    ///
    /// Runs the full pipeline INCLUDING the normalize pass so symbol-ref tests
    /// exercise the same path as `freeze.rs` step 7.
    fn normalize_resolve(src: &str) -> Result<(), ResolveError> {
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut macros = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let env = Environment::default();
        let sym = SymbolTable::default();
        let expanded =
            crate::macros::expand_all(rest, &mut macros, &env, &sym).expect("expand");
        let mut sym = SymbolTable::new();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        // normalize first (arc 251.1b), then validate references.
        let normalized = normalize_symbol_refs(rest, &sym, &macros)?;
        resolve_references(&normalized, &sym, &macros)
    }

    // ─── Happy paths ────────────────────────────────────────────────────

    #[test]
    fn algebra_core_calls_resolve() {
        assert!(resolve(r#"(:wat::holon::Atom "x")"#).is_ok());
        assert!(resolve(r#"(:wat::holon::Bind (:wat::holon::Atom "r") (:wat::holon::Atom "f"))"#).is_ok());
        assert!(resolve(r#"(:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST (:wat::holon::Atom "a")))"#).is_ok());
    }

    #[test]
    fn core_arithmetic_resolves() {
        assert!(resolve(r#"(:wat::core::i64::+ 1 2)"#).is_ok());
        assert!(resolve(r#"(:wat::core::i64::* (:wat::core::i64::+ 1 2) 3)"#).is_ok());
    }

    #[test]
    fn user_define_resolves() {
        // Stone 241.11 — the resolve() test helper does not load stdlib macros,
        // so `defn` (a macro) would not expand. Use `def` + `fn` (the post-expansion
        // form) to directly test the resolver without requiring macro expansion.
        assert!(resolve(
            r#"
            (:wat::core::def :my::app::inc (:wat::core::fn [x <- :i64] -> :i64 (:wat::core::i64::+ x 1)))
            (:my::app::inc 41)
            "#,
        )
        .is_ok());
    }

    #[test]
    fn kernel_and_std_prefixes_accepted() {
        // These aren't implemented yet but shouldn't fail resolution —
        // they're under reserved prefixes that the spec carves out.
        assert!(resolve(r#"(:wat::kernel::send sender value)"#).is_ok());
        assert!(resolve(r#"(:wat::holon::Subtract a b)"#).is_ok());
    }

    #[test]
    fn config_accessors_accepted() {
        assert!(resolve(r#"(:wat::config::dim-count)"#).is_ok());
        assert!(resolve(r#"(:wat::config::set-dim-count! 4096)"#).is_ok());
    }

    #[test]
    fn nested_references_all_resolve() {
        // Stone 241.11 — use `def` + `fn` (the post-macro-expansion form)
        // since the resolve() test helper does not load stdlib macros.
        assert!(resolve(
            r#"
            (:wat::core::def :my::app::add-one (:wat::core::fn [x <- :i64] -> :i64 (:wat::core::i64::+ x 1)))
            (:wat::core::def :my::app::double (:wat::core::fn [x <- :i64] -> :i64 (:wat::core::i64::* x 2)))
            (:my::app::add-one (:my::app::double 10))
            "#,
        )
        .is_ok());
    }

    // ─── Error paths ────────────────────────────────────────────────────

    #[test]
    fn unknown_user_path_rejected() {
        let err = resolve(r#"(:my::app::missing 1)"#).unwrap_err();
        match err {
            ResolveError::UnresolvedReferences(refs) => {
                assert_eq!(refs.len(), 1);
                assert_eq!(refs[0].path, ":my::app::missing");
            }
        }
    }

    #[test]
    fn multiple_unresolved_reported_together() {
        let err = resolve(
            r#"
            (:my::app::missing-a 1)
            (:my::app::missing-b 2)
            (:wat::core::i64::+ (:my::app::missing-c) (:my::app::missing-d))
            "#,
        )
        .unwrap_err();
        match err {
            ResolveError::UnresolvedReferences(refs) => {
                assert_eq!(refs.len(), 4, "expected 4 unresolved refs, got {}", refs.len());
            }
        }
    }

    #[test]
    fn user_define_not_yet_registered_rejected() {
        // Calling a function before it's defined in the same file is OK
        // at startup (all defines register first), but if it's NEVER
        // defined, resolve errors.
        let err = resolve(r#"(:my::app::never-defined 1)"#).unwrap_err();
        match err {
            ResolveError::UnresolvedReferences(refs) => {
                assert_eq!(refs[0].path, ":my::app::never-defined");
            }
        }
    }

    // ─── is_reserved_prefix ─────────────────────────────────────────────

    #[test]
    fn reserved_prefix_recognized() {
        assert!(is_reserved_prefix(":wat::core::define"));
        assert!(is_reserved_prefix(":wat::kernel::spawn"));
        assert!(is_reserved_prefix(":wat::holon::Atom"));
        assert!(is_reserved_prefix(":wat::holon::Subtract"));
        assert!(is_reserved_prefix(":wat::config::dim-count"));
        assert!(is_reserved_prefix(":wat::load::file-path"));
        assert!(is_reserved_prefix(":wat::load::string"));
        assert!(is_reserved_prefix(":wat::verify::digest-sha256"));
        assert!(is_reserved_prefix(":wat::verify::signed-ed25519"));
        assert!(is_reserved_prefix(":wat::verify::string"));
        assert!(is_reserved_prefix(":wat::verify::file-path"));
        assert!(is_reserved_prefix(":wat::eval::string"));
        assert!(is_reserved_prefix(":wat::eval::file-path"));
    }

    #[test]
    fn user_prefix_not_reserved() {
        assert!(!is_reserved_prefix(":my::app::foo"));
        assert!(!is_reserved_prefix(":project::market::Candle"));
        assert!(!is_reserved_prefix(":alice::math::clamp"));
    }

    #[test]
    fn bare_name_not_reserved() {
        assert!(!is_reserved_prefix(":foo"));
        assert!(!is_reserved_prefix(":42"));
    }

    // ─── use! declaration enforcement ───────────────────────────────────
    //
    // Success-path tests retired in arc 013 slice 4b — they used
    // `:rust::lru::LruCache` as the fixture, which moved to the
    // external `wat-lru` crate. End-to-end happy-path coverage
    // for the use! mechanism (declaration + covers-all-methods +
    // idempotent re-declaration) lives in
    // crates/wat-lru/tests/wat_lru_tests.rs, where a real shim is
    // registered via dep_registrars and exercised through wat
    // source. Failure-path tests below don't need a registered
    // type — the diagnostics fire regardless — so they stay.

    #[test]
    fn rust_call_without_use_declaration_fails() {
        let err = resolve(r#"(:rust::lru::LruCache::new 16)"#).unwrap_err();
        let ResolveError::UnresolvedReferences(list) = err;
        assert!(
            list.iter()
                .any(|u| u.path == ":rust::lru::LruCache::new"
                    && u.context
                        .contains("not covered by any (:wat::core::use! ...)")),
            "expected use!-not-covered diagnostic; got {:?}",
            list
        );
    }

    #[test]
    fn use_of_unknown_rust_symbol_fails() {
        let err = resolve(r#"(:wat::core::use! :rust::imaginary::Thing)"#).unwrap_err();
        let ResolveError::UnresolvedReferences(list) = err;
        assert!(
            list.iter()
                .any(|u| u.path == ":rust::imaginary::Thing"
                    && u.context.contains("not available in wat")),
            "expected not-available diagnostic; got {:?}",
            list
        );
    }

    // use!-success paths previously checked against :rust::lru::LruCache
    // retired in arc 013 slice 4b — that type no longer ships in the
    // wat-rs default registry. Equivalent coverage lives in
    // crates/wat-lru/tests/ where the shim is present.
    // Failure-path tests above still exercise the resolver's own logic
    // without depending on any specific shim.

    // ─── Arc 251 stone 251.1b — normalize pass ──────────────────────────

    #[test]
    fn namespaced_symbol_head_normalizes_to_keyword() {
        // `wat.core/i64::+` is a reserved prefix → normalize rewrites it to
        // `:wat::core::i64::+`; resolve then accepts it as a known builtin.
        // Uses the normalize_resolve() helper which runs the 251.1b normalize
        // pass before the standard resolve_references check.
        assert!(
            normalize_resolve(r#"(wat.core/i64::+ 1 2)"#).is_ok(),
            "namespaced symbol head wat.core/i64::+ should normalize to :wat::core::i64::+"
        );
    }

    #[test]
    fn unknown_namespaced_symbol_gives_located_error() {
        // A well-formed but unknown namespaced ref (`foo.bar/baz`) must surface
        // as a LOCATED error (ResolveError::UnresolvedReferences with a path),
        // NOT as a bare `UnboundSymbol` at runtime. Arc 251.0 contract.
        let err = normalize_resolve(r#"(foo.bar/baz 1)"#).unwrap_err();
        match err {
            ResolveError::UnresolvedReferences(refs) => {
                assert_eq!(refs.len(), 1, "expected exactly one unresolved ref");
                // The path should be the primary FQDN candidate (`:foo::bar::baz`),
                // not the raw symbol name.
                assert!(
                    refs[0].path.starts_with(':'),
                    "located error path must be a keyword FQDN; got {:?}",
                    refs[0].path
                );
                assert!(
                    refs[0].path.contains("foo") && refs[0].path.contains("bar") && refs[0].path.contains("baz"),
                    "located error path must contain the namespace segments; got {:?}",
                    refs[0].path
                );
                assert!(
                    refs[0].context.contains("arc 251"),
                    "error context should name arc 251; got {:?}",
                    refs[0].context
                );
            }
        }
    }

    #[test]
    fn bare_local_symbol_untouched_by_normalize() {
        // Bare symbols (no `/`) are local binders — `x`, `acc`, `xs`.
        // The normalize pass must NOT rewrite them. A let-binding that uses a
        // bare symbol as both binder and reference still resolves cleanly.
        assert!(
            normalize_resolve(
                r#"
                (:wat::core::def :my::app::square
                  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                    (:wat::core::i64::* x x)))
                (:my::app::square 5)
                "#
            )
            .is_ok(),
            "bare local symbols (x) must pass through the normalize pass untouched"
        );
    }
}
