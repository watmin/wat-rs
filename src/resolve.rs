//! Name resolution pass.
//!
//! After macro expansion, every keyword-path reference used in call
//! position must resolve to one of:
//!
//! - A known `:wat::core::*` language form (define, lambda, let, if,
//!   the builtin arithmetic / comparison / boolean ops, the list
//!   constructor, the quasiquote forms, the type-declaration heads,
//!   `load!`).
//! - A known `:wat::algebra::*` core form (`Atom`, `Bind`, `Bundle`,
//!   `Permute`, `Thermometer`, `Blend`, `cosine`, `dot`).
//! - A `:wat::kernel::*` primitive (queue / spawn / select / HandlePool /
//!   signals) — accepted here; runtime not yet implementing kernel.
//! - A `:wat::std::*` name — accepted here; stdlib macros expand to
//!   core forms, but references that didn't expand (e.g., stdlib
//!   programs) pass through.
//! - A `:wat::config::*` setter or accessor.
//! - A `:wat::load::*` interface keyword (source-fetch selector for
//!   `load!` / `digest-load!` / `signed-load!`).
//! - A `:wat::verify::*` keyword — either a verification algorithm
//!   (`:wat::verify::digest-sha256`, `:wat::verify::signed-ed25519`) or a
//!   payload-fetch interface (`:wat::verify::string`, `:wat::verify::file-path`).
//! - A `:wat::eval::*` keyword — source-fetch selector for runtime
//!   eval forms (`:wat::eval::string`, `:wat::eval::file-path`).
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
//!   checker's job (task #137), which has access to the `TypeEnv`
//!   and instantiation logic. This pass treats type annotations
//!   and field types as opaque.
//! - It does NOT transform the AST. Just validates references.

use crate::ast::WatAST;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use crate::types::TypeEnv;
use std::fmt;

/// One unresolved reference, with context about where it appeared.
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedReference {
    /// The keyword path that didn't resolve.
    pub path: String,
    /// Human-friendly context: a short phrase like "call head" or
    /// "macro call (not expanded)".
    pub context: &'static str,
}

/// Name-resolution errors.
#[derive(Debug)]
pub enum ResolveError {
    /// One or more references don't resolve. `unresolved` carries ALL
    /// failures so the user can fix them in a single pass.
    UnresolvedReferences(Vec<UnresolvedReference>),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::UnresolvedReferences(list) => {
                writeln!(f, "{} unresolved reference(s):", list.len())?;
                for r in list {
                    writeln!(f, "  - {} ({})", r.path, r.context)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Check that every call-position keyword-path reference in `forms`
/// resolves somewhere. Returns Ok iff all references are known;
/// otherwise reports every failure at once.
pub fn resolve_references(
    forms: &[WatAST],
    sym: &SymbolTable,
    macros: &MacroRegistry,
    _types: &TypeEnv,
) -> Result<(), ResolveError> {
    let mut unresolved = Vec::new();
    for form in forms {
        check_form(form, sym, macros, &mut unresolved);
    }
    if unresolved.is_empty() {
        Ok(())
    } else {
        Err(ResolveError::UnresolvedReferences(unresolved))
    }
}

fn check_form(
    form: &WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    unresolved: &mut Vec<UnresolvedReference>,
) {
    if let WatAST::List(items) = form {
        if let Some(WatAST::Keyword(head)) = items.first() {
            if !is_resolvable_call_head(head, sym, macros) {
                unresolved.push(UnresolvedReference {
                    path: head.clone(),
                    context: if macros.contains(head) {
                        "macro call survived expansion (expansion pass ran before this check?)"
                    } else {
                        "call head — not a builtin, not a registered function"
                    },
                });
            }
        }
        // Recurse into all children.
        for child in items {
            check_form(child, sym, macros, unresolved);
        }
    }
}

/// True if `head` resolves as a call target.
fn is_resolvable_call_head(head: &str, sym: &SymbolTable, macros: &MacroRegistry) -> bool {
    // Kernel, algebra, std, config, and core prefixes are reserved for
    // the language; accept them as-is. A wrong name under those
    // prefixes (e.g. :wat::algebra::Bogus) fails DOWNSTREAM at
    // runtime or lowering, but the name-resolution pass is scoped
    // to catch "no such namespace" mistakes, not "wrong name inside
    // a known namespace" mistakes. The spec's name-resolution layer
    // wants the path-prefix shape validated; leaf-level validation
    // is the type checker's concern.
    if is_reserved_prefix(head) {
        return true;
    }
    // A user-registered function.
    if sym.get(head).is_some() {
        return true;
    }
    // A macro call — shouldn't survive expansion, but accept for
    // completeness. The checker notes it as suspicious in the
    // context string when a macro is the reason.
    if macros.contains(head) {
        return true;
    }
    false
}

/// Is `keyword` under one of the reserved wat-level prefixes?
///
/// This utility is shared across the registration functions that
/// must refuse user-declarations under these prefixes (define,
/// defmacro, type declarations).
/// Reserved keyword prefixes the language owns. User definitions
/// under these paths are refused at registration time (define /
/// defmacro / type declarations).
///
/// Every consumer that renders an error message about reserved
/// prefixes should read this list via [`reserved_prefix_list`] so
/// the user-facing message stays in sync with [`is_reserved_prefix`].
pub const RESERVED_PREFIXES: &[&str] = &[
    ":wat::core::",
    ":wat::kernel::",
    ":wat::algebra::",
    ":wat::std::",
    ":wat::config::",
    ":wat::load::",
    ":wat::verify::",
    ":wat::eval::",
    ":wat::io::",
];

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::{register_defmacros, MacroRegistry};
    use crate::parser::parse_all;
    use crate::runtime::{register_defines, SymbolTable};
    use crate::types::TypeEnv;

    /// Full pipeline helper: parse → register-defmacros → expand → register-defines → resolve.
    fn resolve(src: &str) -> Result<(), ResolveError> {
        let forms = parse_all(src).expect("parse ok");
        let mut macros = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let expanded = crate::macros::expand_all(rest, &macros).expect("expand");
        let mut sym = SymbolTable::new();
        let rest = register_defines(expanded, &mut sym).expect("register defines");
        let types = TypeEnv::new();
        resolve_references(&rest, &sym, &macros, &types)
    }

    // ─── Happy paths ────────────────────────────────────────────────────

    #[test]
    fn algebra_core_calls_resolve() {
        assert!(resolve(r#"(:wat::algebra::Atom "x")"#).is_ok());
        assert!(resolve(r#"(:wat::algebra::Bind (:wat::algebra::Atom "r") (:wat::algebra::Atom "f"))"#).is_ok());
        assert!(resolve(r#"(:wat::algebra::Bundle (:wat::core::vec (:wat::algebra::Atom "a")))"#).is_ok());
    }

    #[test]
    fn core_arithmetic_resolves() {
        assert!(resolve(r#"(:wat::core::i64::+ 1 2)"#).is_ok());
        assert!(resolve(r#"(:wat::core::i64::* (:wat::core::i64::+ 1 2) 3)"#).is_ok());
    }

    #[test]
    fn user_define_resolves() {
        assert!(resolve(
            r#"
            (:wat::core::define (:my::app::inc (x :i64) -> :i64) (:wat::core::i64::+ x 1))
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
        assert!(resolve(r#"(:wat::std::Subtract a b)"#).is_ok());
    }

    #[test]
    fn config_accessors_accepted() {
        assert!(resolve(r#"(:wat::config::dims)"#).is_ok());
        assert!(resolve(r#"(:wat::config::set-dims! 4096)"#).is_ok());
    }

    #[test]
    fn nested_references_all_resolve() {
        assert!(resolve(
            r#"
            (:wat::core::define (:my::app::add-one (x :i64) -> :i64) (:wat::core::i64::+ x 1))
            (:wat::core::define (:my::app::double (x :i64) -> :i64) (:wat::core::i64::* x 2))
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
        assert!(is_reserved_prefix(":wat::algebra::Atom"));
        assert!(is_reserved_prefix(":wat::std::Subtract"));
        assert!(is_reserved_prefix(":wat::config::dims"));
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
}
