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
//! The invariant is enforced at construction in `Identifier::bare` (debug
//! builds) — the single chokepoint for name admission. Scope IDs are `u64`
//! integers joined by `,`; the `BTreeSet` iteration order is deterministic
//! (ascending numeric), so the encoding is canonical.

// PARTITION — CLAUSE vs INTRINSIC: `env_key` is a pure function (clause
// territory in the dispatch sense — monomorphic, no type-var flow). It
// lives here rather than in runtime.rs so the resolution policy has a
// single, testable home that can be read independently of the 31k-line
// runtime.

use crate::scope::Identifier;

/// Derive the environment key for an identifier.
///
/// - **Bare** (empty scope set) → borrows the name unchanged (zero alloc).
///   All non-macro code that never calls `add_scope` continues to use bare
///   names as keys; no behavioural change for any existing binding or lookup.
/// - **Scoped** → owned `"name\u{1}<sorted-scope-ids>"`. The scoped suffix
///   makes the key unique to this expansion instance, preventing the macro's
///   binder from capturing a caller-site variable of the same name.
///
/// The return type is `Cow<'_, str>`: bare = borrowed (zero alloc — runs
/// once per symbol lookup in every program); scoped = owned single allocation
/// (runs at both expansion time and on every scoped-symbol eval).
///
/// A binder and all references that should resolve to it must carry
/// the IDENTICAL scope set so they compute the same key. The expander
/// guarantees this: `walk_template` adds one `ScopeId` uniformly to
/// every template-origin identifier in a single expansion pass.
///
/// # Panics (debug builds only)
///
/// The U+0001 separator invariant is enforced at construction by
/// `Identifier::bare` — the single chokepoint for name admission (debug
/// builds). `env_key` itself does not re-assert; the invariant is guaranteed
/// before this function is called.
pub fn env_key(ident: &Identifier) -> std::borrow::Cow<'_, str> {
    if ident.scopes().is_empty() {
        std::borrow::Cow::Borrowed(ident.as_str())
    } else {
        // BTreeSet iterates in ascending order → canonical encoding.
        // \u{1} (SOH) is chosen because lexer-produced identifier names never
        // contain it; the invariant is enforced at Identifier::bare construction.
        // Single allocation: name + separator + comma-joined ascending scope ids.
        // Scoped identifiers are evaluated post-expansion (every scoped-symbol
        // eval calls env_key), so this path is not expansion-time-only — hence
        // the single-alloc form rather than Vec+join.
        let name = ident.as_str();
        let mut key = String::with_capacity(name.len() + 16);
        key.push_str(name);
        key.push('\u{1}');
        let mut first = true;
        for s in ident.scopes() {
            if !first {
                key.push(',');
            }
            // itoa-style: write the u64 directly
            use std::fmt::Write as _;
            let _ = write!(key, "{}", s.as_u64());
            first = false;
        }
        std::borrow::Cow::Owned(key)
    }
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
    fn scoped_key_contains_separator_byte() {
        // The scoped key embeds \u{1} (SOH), which a bare-ident key never can.
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
