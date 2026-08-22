//! vigilatum: 2026-06-10T03:11:19Z — resolve/ ward (arc 251.1), L1+L2=0, clippy-clean in-home.
//! Earned by COMBAT — the full inward guard cast (intueri · purgare · struere ·
//! solvere · sequi · conformare · exigere · cernere · temperare · circumspicere
//! last), every finding weighed against the disk in both directions. The guard
//! earned its keep three times: solvere caught the KEYSTONE braid (the quote-family
//! boundary table encoded twice and DRIFTED → decomplected to one
//! [`boundary::quote_boundary`] classifier both passes match EXHAUSTIVELY, so drift
//! is now a compile error) plus a second escape-head braid (→ [`boundary`]'s
//! `is_unquote_escape`); exigere caught the orchestrator's OWN deferral-prose;
//! circumspicere caught the untested negative space (the rewritten normalize
//! boundary handling → 6 AST-inspecting boundary tests + an ordering-contract pin).
//! L3 accepted-with-reason: the `match` positional grammar (spec single-sourced in
//! `Boundary::Match`; the per-pass traversal is the irreducible ownership split),
//! `is_resolvable_call_head`'s home (a documented `pub(super)` shared predicate), and
//! `UnresolvedReference`'s span shape (the substrate-wide `Span` convention — a
//! substrate conformare-arc concern, not this home's defect). Gates: lib 950/0/1,
//! resolve 23/23, arc251 probe 2/2, clippy-in-home empty. Full record:
//! docs/arc/2026/06/251-types-as-forms/SCORE-STONE-251.1-WARD.md.
//!
//! Name resolution pass.
//!
//! After macro expansion, every keyword-path reference used in call
//! position must resolve to one of:
//!
//! - A known `:wat::core::*` language form (defn, fn, let, if,
//!   the builtin arithmetic / comparison / boolean ops, the list
//!   constructor, the quasiquote forms, the type-declaration heads).
//!   (`define` was retired at Stone 241.11 — `defn` is the successor; the
//!   resolver still boundary-guards `:wat::core::define` so it does not walk a
//!   retired form's body before the checker rejects it.)
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
//! - It does NOT check INLINE type-position references (`let`/`match` ascriptions inside a
//!   function body). Those are checked at use by the existing unifier (see [`crate::check`]).
//!   Arc 109 (a-type-reference-must-resolve) added a SEPARATE sweep
//!   ([`type_refs::sweep_type_references`], called from [`walk::resolve_references`]) for
//!   DECLARED type positions — params, returns, fields, variant payloads, alias RHS, surface
//!   members — which the checker only ever reaches at a USE site, never validating an uncalled
//!   declaration at all.
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
//! warded home. Pure structural move; zero behavior change. The vigilatum stamp
//! (top of this file) was earned at 251.1's ward close — see the SCORE it cites.

pub(crate) mod boundary;
mod error;
mod normalize;
mod quote;
mod registration;
mod reserved;
mod rust_use;
mod type_refs;
mod walk;

// Public API — re-exported for the external importers (freeze.rs, lib.rs,
// macros/registry.rs, closure_extract.rs).
pub use error::{ReferenceKind, ResolveError, UnresolvedReference};
pub use normalize::normalize_symbol_refs;
pub use registration::{is_namespaced, register, Existing, Privilege, Registration, Rejection};
pub use reserved::{is_reserved_prefix, reserved_prefix_list};
pub use walk::resolve_references;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::{register_defmacros, MacroRegistry};
    use crate::runtime::{register_defines, Environment, SymbolTable};
    use crate::types::TypeEnv;

    /// Full pipeline helper: parse → register-defmacros → expand → register-defines → resolve.
    ///
    /// Deliberately SKIPS the 251.1b normalize pass — it exercises
    /// `resolve_references` in isolation (keyword-spelled call heads). For tests
    /// that involve namespaced SYMBOL heads (`wat.core/+`), use
    /// [`normalize_resolve`] (full pipeline) or [`normalize_ast`] (inspect the
    /// rewritten AST) instead — `resolve_references` alone cannot see a symbol head.
    ///
    /// Arc 109 — also skips `register_types`, mirroring production's ORDER (step 5 precedes
    /// step 7) with an intentionally-empty user type set; `TypeEnv::with_builtins()` still
    /// seeds the same builtin `TypeDef`s a real freeze would carry, so a fixture's function
    /// signature naming a builtin AGGREGATE type (not a bare/FQDN scalar — see the module
    /// doc's primitives caveat) resolves exactly as it would in production.
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
        let types = TypeEnv::with_builtins();
        resolve_references(&rest, &sym, &macros, &types)
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
        let types = TypeEnv::with_builtins();
        resolve_references(&normalized, &sym, &macros, &types)
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
        //
        // Arc 109 — `:wat::core::i64` (FQDN), not bare `:i64`: bare primitives are a
        // RETIRED spelling (`check::BARE_PRIMITIVES`, flagged by `walk_for_bare_primitives`
        // at check time) that the new declared-type sweep also does not recognize — it
        // isn't in `TypeEnv` OR the sweep's builtin-leaf allowlist, deliberately, since
        // accepting it here would re-legitimize a spelling the corpus has been migrating
        // away from.
        assert!(resolve(
            r#"
            (:wat::core::def :my::app::inc (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1)))
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
            (:wat::core::def :my::app::add-one (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1)))
            (:wat::core::def :my::app::double (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2)))
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
        assert!(is_reserved_prefix(":wat::kernel::spawn-program"));
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
        assert_eq!(list.len(), 1, "expected exactly one unresolved ref; got {:?}", list);
        assert_eq!(list[0].path, ":rust::lru::LruCache::new");
        assert_eq!(list[0].context, ":rust::* reference not covered by any (:wat::core::use! ...) declaration");
    }

    #[test]
    fn use_of_unknown_rust_symbol_fails() {
        let err = resolve(r#"(:wat::core::use! :rust::imaginary::Thing)"#).unwrap_err();
        let ResolveError::UnresolvedReferences(list) = err;
        assert_eq!(list.len(), 1, "expected exactly one unresolved ref; got {:?}", list);
        assert_eq!(list[0].path, ":rust::imaginary::Thing");
        assert_eq!(list[0].context, "rust symbol not available in wat; declare it via its shim");
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
                assert_eq!(refs[0].path, ":foo::bar::baz", "located error path must be the keyword FQDN");
                assert_eq!(refs[0].context, "namespaced symbol ref — not a builtin, not a registered function (arc 251)");
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

    // ─── Arc 251.1 ward (circumspicere C4) — normalize boundary discipline ──────
    //
    // The keystone was a DRIFT in the data-vs-code boundary table between the two
    // passes. `resolve_references` cannot observe normalize's boundary behaviour —
    // it skips the same data positions — so these tests inspect the REWRITTEN AST
    // directly: a namespaced symbol in a CODE position must become a Keyword FQDN;
    // one in a DATA position (quoted form, match/cond/matches? pattern, quasiquote
    // template) must stay a Symbol.

    use crate::ast::WatAST;

    /// Parse → expand → register → normalize; return the rewritten top-level AST.
    fn normalize_ast(src: &str) -> Vec<WatAST> {
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut macros = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let env = Environment::default();
        let sym0 = SymbolTable::default();
        let expanded =
            crate::macros::expand_all(rest, &mut macros, &env, &sym0).expect("expand");
        let mut sym = SymbolTable::new();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        normalize_symbol_refs(rest, &sym, &macros).expect("normalize ok")
    }

    /// True if any node in the tree is a `Symbol` whose text equals `text`.
    fn contains_symbol(nodes: &[WatAST], text: &str) -> bool {
        nodes.iter().any(|n| match n {
            WatAST::Symbol(s, _) => s.as_str() == text,
            WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
                contains_symbol(items, text)
            }
            WatAST::Map(pairs, _) => pairs.iter().any(|(k, v)| {
                contains_symbol(std::slice::from_ref(k), text)
                    || contains_symbol(std::slice::from_ref(v), text)
            }),
            _ => false,
        })
    }

    /// True if any node in the tree is a `Keyword` whose text equals `text`.
    fn contains_keyword(nodes: &[WatAST], text: &str) -> bool {
        nodes.iter().any(|n| match n {
            WatAST::Keyword(k, _) => k.as_str() == text,
            WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
                contains_keyword(items, text)
            }
            WatAST::Map(pairs, _) => pairs.iter().any(|(k, v)| {
                contains_keyword(std::slice::from_ref(k), text)
                    || contains_keyword(std::slice::from_ref(v), text)
            }),
            _ => false,
        })
    }

    #[test]
    fn normalize_skips_quoted_form_symbols() {
        // A namespaced symbol inside `quote` is DATA — never rewritten.
        let ast = normalize_ast(r#"(:wat::core::quote (wat.core/i64::+ 1 2))"#);
        assert!(
            contains_symbol(&ast, "wat.core/i64::+"),
            "symbol inside quote must stay a Symbol (data); got {ast:?}"
        );
        assert!(
            !contains_keyword(&ast, ":wat::core::i64::+"),
            "symbol inside quote must NOT be rewritten to a keyword"
        );
    }

    #[test]
    fn normalize_skips_match_pattern_but_rewrites_body() {
        // Match arm = (pattern body). Pattern is DATA (Symbol preserved); body is
        // CODE (rewritten). This is the exact boundary the keystone braid drifted on.
        // If the pattern were wrongly walked, `scrut.ns/Variant` would normalize to
        // an unresolvable `:scrut::ns::Variant` and `normalize_ast` would panic.
        let ast = normalize_ast(
            r#"(:wat::core::match x
                  ((scrut.ns/Variant a) (wat.core/i64::+ a 1)))"#,
        );
        assert!(
            contains_symbol(&ast, "scrut.ns/Variant"),
            "match-arm PATTERN symbol must stay a Symbol (data)"
        );
        assert!(
            contains_keyword(&ast, ":wat::core::i64::+"),
            "match-arm BODY symbol must be rewritten to its keyword FQDN (code)"
        );
        assert!(
            !contains_symbol(&ast, "wat.core/i64::+"),
            "match-arm BODY symbol must not remain a Symbol"
        );
    }

    #[test]
    fn normalize_skips_quasiquote_template_but_rewrites_escapes() {
        // Quasiquote template is DATA except inside unquote/unquote-splicing escapes.
        let ast = normalize_ast(
            r#"(:wat::core::quasiquote
                  (wat.core/+ (:wat::core::unquote (wat.core/i64::* 2 3))))"#,
        );
        assert!(
            contains_symbol(&ast, "wat.core/+"),
            "quasiquote TEMPLATE symbol must stay a Symbol (data)"
        );
        assert!(
            contains_keyword(&ast, ":wat::core::i64::*"),
            "symbol inside an UNQUOTE escape must be rewritten (live code)"
        );
    }

    #[test]
    fn normalize_rewrites_matches_subject_keeps_pattern() {
        // `matches?` — subject (items[1]) is CODE; pattern (items[2..]) is DATA.
        let ast = normalize_ast(r#"(:wat::form::matches? (wat.core/i64::+ y 1) (pat.ns/Shape a))"#);
        assert!(
            contains_keyword(&ast, ":wat::core::i64::+"),
            "matches? SUBJECT is code → rewritten"
        );
        assert!(
            contains_symbol(&ast, "pat.ns/Shape"),
            "matches? PATTERN is data → Symbol preserved"
        );
    }

    #[test]
    fn resolve_alone_cannot_see_symbol_heads_normalize_must_precede() {
        // ORDERING CONTRACT (circumspicere C3): `resolve_references` validates only
        // KEYWORD call heads — a namespaced SYMBOL head is invisible to it. So
        // `normalize_symbol_refs` MUST run first (freeze.rs step 7) to turn the
        // symbol into a keyword the resolver can validate. This pins the order:
        // resolve-alone passes silently (it cannot see the symbol head), while the
        // full normalize→resolve pipeline rewrites then validates it.
        assert!(
            resolve(r#"(wat.core/i64::+ 1 2)"#).is_ok(),
            "resolve alone is blind to a namespaced symbol head (it is not a Keyword)"
        );
        assert!(
            normalize_resolve(r#"(wat.core/i64::+ 1 2)"#).is_ok(),
            "normalize→resolve (correct order) rewrites the symbol head, then validates it"
        );
    }
}
