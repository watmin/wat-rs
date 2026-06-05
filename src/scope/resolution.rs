//! Scope-aware reference resolution — the policy that makes the tags the
//! expander mints (src/scope/identifier.rs) load-bearing at lookup time.
//!
//! # Why this exists
//!
//! The macro expander (`src/macros/expand.rs`) calls `Identifier::add_scope`
//! on every symbol that originates in a macro template, giving it a fresh
//! `ScopeId`. Identifiers that arrive via unquote (`,x`) keep their
//! caller-site scopes. This tagging correctly distinguishes the macro's
//! `tmp` from the caller's `tmp` AT THE AST LEVEL — but only if the runtime
//! resolution policy respects the full (name, scope-set) identity.
//!
//! Without this module, runtime key derivation used bare `ident.name` /
//! `ident.as_str()`, silently dropping `.scopes`. Every bind/lookup pair
//! therefore operated on the bare name, making macro scope tags inert —
//! the classic variable-capture defect.
//!
//! Stone 249.5b ships this module to close the gap: every env bind and
//! lookup that derives its key from an `Identifier` routes through
//! `env_key`, so the scope set is load-bearing.
//!
//! # Exact-match model
//!
//! `walk_template` adds the macro's `ScopeId` uniformly to EVERY
//! template-origin identifier in a single expansion step, so a binder
//! (`let [tmp ...]`) and every reference to it inside the same template
//! (`(i64::+ tmp ...)`) carry the SAME scope set. `env_key` uses
//! exact-set equality as the lookup key — no subset matching needed.
//!
//! # Bare identifiers are untouched
//!
//! For a bare identifier (empty scope set), `env_key` returns the name
//! unchanged. Non-macro code never calls `add_scope`, so all existing
//! bindings and lookups on user-written code are unaffected.
//!
//! # Key-space collision safety
//!
//! The separator byte `\u{1}` (ASCII SOH) is chosen because identifier
//! names produced by the lexer never contain it (it is a control character;
//! `lexer::is_symbol_break` only breaks on whitespace and `()[]{}";,`).
//! The invariant is enforced by a debug assertion in [`env_key`], not by
//! the type. Scope IDs are `u64` integers joined by `,`; the `BTreeSet`
//! iteration order is deterministic (ascending numeric), so the encoding
//! is canonical.

// PARTITION — CLAUSE vs INTRINSIC: `env_key` is a pure function (clause
// territory in the dispatch sense — monomorphic, no type-var flow). It
// lives here rather than in runtime.rs so the resolution policy has a
// single, testable home that can be read independently of the 31k-line
// runtime.

use crate::ast::WatAST;
use crate::scope::Identifier;

/// Derive the environment key for an identifier.
///
/// - **Bare** (empty scope set) → the name unchanged. All non-macro code
///   that never calls `add_scope` continues to use bare names as keys;
///   no behavioural change for any existing binding or lookup.
/// - **Scoped** → `"name\u{1}<sorted-scope-ids>"`. The scoped suffix
///   makes the key unique to this expansion instance, preventing the
///   macro's binder from capturing a caller-site variable of the same
///   name.
///
/// A binder and all references that should resolve to it must carry
/// the IDENTICAL scope set so they compute the same key. The expander
/// guarantees this: `walk_template` adds one `ScopeId` uniformly to
/// every template-origin identifier in a single expansion pass.
///
/// # Panics (debug builds only)
///
/// Asserts that the identifier name does not contain `\u{1}`, which
/// would break the separator invariant. This should never fire for any
/// name the lexer produces.
pub fn env_key(ident: &Identifier) -> String {
    debug_assert!(
        !ident.as_str().contains('\u{1}'),
        "env_key separator U+0001 must not appear in an identifier name; got {:?}",
        ident.as_str()
    );
    if ident.scopes().is_empty() {
        ident.as_str().to_owned()
    } else {
        // BTreeSet iterates in ascending order → canonical encoding.
        // \u{1} (SOH) is chosen because lexer-produced identifier names never
        // contain it; the debug_assert above is the defence-in-depth guard.
        let scopes: Vec<String> = ident.scopes().iter().map(|s| s.as_u64().to_string()).collect();
        format!("{}\u{1}{}", ident.as_str(), scopes.join(","))
    }
}

/// Extract scope-aware env-key'd parameter names from an argspec args-vector.
///
/// Stone 249.5b (defclause fix) — canonical home for the scoped-arg-walk that
/// `fn`/`let`/`defclause` binding sites all need. Centralises the logic that was
/// previously duplicated across `runtime.rs::scoped_params_from_args_vec` and
/// `function/eval.rs::extract_scoped_params`.
///
/// # Arguments
///
/// - `args_vec_node` — the `WatAST::Vector` wrapping the argspec items
///   (`[name <- :T name <- :T ... [& rest <- :T]]`). The function unwraps the
///   Vector and processes the inner items.
/// - `fallback` — the bare-name list already computed by `parse_argspec_triples` /
///   `parse_fn_signature`. Returned unchanged when the vector is malformed or the
///   item count is not a multiple of 3 (preserving the existing error semantics;
///   the type checker reports the structural issue separately).
///
/// # Returns
///
/// Ordered list of env-key strings, one per declared parameter (fixed params
/// first, then the rest-binder if present — matching the order in `fallback`).
/// For bare identifiers (empty scope set) the key is the bare name; for scoped
/// identifiers (macro-template origin) the key encodes the scope set.
///
/// # Rest-binder handling
///
/// The `& name <- :T` rest-binder is handled identically to the fixed-param
/// walk: the `&` symbol is detected and skipped; `env_key` is applied to the
/// name symbol that follows. The rest-binder's key is appended after the fixed
/// params (it is always last in the source vector).
pub fn scoped_arg_names(args_vec_node: &WatAST, fallback: &[String]) -> Vec<String> {
    let items = match args_vec_node {
        WatAST::Vector(items, _) => items.as_slice(),
        _ => return fallback.to_vec(),
    };
    // Walk in triples (name <- :T).  `& rest <- :T` rest-binders share the
    // same structure; env_key applies to all.  If the count is not a multiple
    // of 3 we fall back (malformed, type checker will catch it separately).
    if items.len() % 3 != 0 {
        return fallback.to_vec();
    }
    let mut out = Vec::with_capacity(items.len() / 3);
    let mut i = 0;
    while i + 2 < items.len() {
        // Skip `&` rest-marker if present at this position.
        let name_item = if items[i].is_bare_symbol("&") {
            // rest binder: the name follows the `&`.
            if i + 3 >= items.len() { return fallback.to_vec(); }
            i += 1;
            &items[i]
        } else {
            &items[i]
        };
        match name_item {
            WatAST::Symbol(ident, _) => out.push(env_key(ident)),
            _ => return fallback.to_vec(),
        }
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::{fresh_scope, Identifier};

    #[test]
    fn bare_ident_gives_bare_name() {
        let id = Identifier::bare("x");
        assert_eq!(env_key(&id), "x");
    }

    #[test]
    fn same_name_same_scopes_give_same_key() {
        let s = fresh_scope();
        let a = Identifier::bare("tmp").add_scope(s);
        let b = Identifier::bare("tmp").add_scope(s);
        assert_eq!(env_key(&a), env_key(&b));
    }

    #[test]
    fn same_name_different_scopes_give_different_keys() {
        let s1 = fresh_scope();
        let s2 = fresh_scope();
        let a = Identifier::bare("tmp").add_scope(s1);
        let b = Identifier::bare("tmp").add_scope(s2);
        assert_ne!(env_key(&a), env_key(&b));
    }

    #[test]
    fn bare_vs_scoped_give_different_keys() {
        let s = fresh_scope();
        let bare = Identifier::bare("tmp");
        let scoped = Identifier::bare("tmp").add_scope(s);
        assert_ne!(env_key(&bare), env_key(&scoped));
    }

    #[test]
    fn key_does_not_collide_with_adjacent_bare_name() {
        // "tmp\u{1}42" must not equal the bare name "tmp\u{1}42" even if
        // someone named a variable that way — but that name is illegal in
        // the parser, so this is defence-in-depth. What we actually test:
        // the scoped key contains the separator byte, which a bare ident
        // key cannot.
        let s = fresh_scope();
        let scoped = Identifier::bare("tmp").add_scope(s);
        let key = env_key(&scoped);
        assert!(key.contains('\u{1}'), "scoped key must contain separator byte");
        assert!(!env_key(&Identifier::bare("tmp")).contains('\u{1}'));
    }

    #[test]
    fn multi_scope_key_is_sorted_and_stable() {
        let s1 = fresh_scope();
        let s2 = fresh_scope();
        // Add scopes in both orders; BTreeSet makes the result identical.
        let a = Identifier::bare("x").add_scope(s1).add_scope(s2);
        let b = Identifier::bare("x").add_scope(s2).add_scope(s1);
        assert_eq!(env_key(&a), env_key(&b), "multi-scope key must be order-independent");
    }
}
