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
//! names produced by the lexer never contain it. The lexer now REJECTS all
//! raw control characters in source (Stone 249 scope-closure), so this
//! invariant is ENFORCED by the lexer, not merely conventional. Construction
//! is additionally guarded in `Identifier::bare` (debug builds) — the single
//! chokepoint for name admission. Scope IDs are `u64` integers joined by `,`;
//! the `BTreeSet` iteration order is deterministic (ascending numeric), so
//! the encoding is canonical.

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
        // \u{1} (SOH) is chosen because the lexer now REJECTS all raw control
        // characters in source (Stone 249 scope-closure), so lexer-produced
        // identifier names structurally cannot contain it; the invariant is
        // additionally enforced at Identifier::bare construction (debug builds).
        // Initial capacity sized for the common single-scope case (name length +
        // separator byte + one scope id); identifiers with many or large-id scopes
        // may reallocate (safely). BTreeSet ascending iteration → canonical encoding.
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

/// A hygiene-scope divergence: `ident` is unbound (its env_key missed), but a
/// binder of the SAME NAME exists under a DIFFERENT hygiene scope. Only ever a
/// faulty macro that rebuilt a binder from its name (changing its ScopeId)
/// instead of reusing the original node — never a legitimate polymorphic
/// placeholder. Returns the diverging binder's key.
///
/// Logic: let `me = env_key(ident)`. For each key `k` in `local_keys`, extract
/// the NAME part (everything before the first `'\u{1}'`, or the whole key if
/// none). If `name_part(k) == ident.as_str()` AND `k != me`, return
/// `k.to_owned()` (a same-name, different-scope binder). Else `None`.
///
/// Examples:
/// - bare ref `"a"` (me = `"a"`) vs scoped binder `"a\u{1}433"` → Some
/// - scoped ref `"a\u{1}7"` (me = `"a\u{1}7"`) vs bare binder `"a"` → Some
/// - same key both sides → None
/// - different name → None
pub fn scope_divergent_binder<'a>(
    ident: &Identifier,
    local_keys: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let me = env_key(ident);
    let me_ref: &str = me.as_ref();
    let my_name = ident.as_str();
    for k in local_keys {
        // Extract name part: everything before the first SOH separator.
        let name_part = k.splitn(2, '\u{1}').next().unwrap_or(k);
        if name_part == my_name && k != me_ref {
            return Some(k.to_owned());
        }
    }
    None
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

    // --- scope_divergent_binder tests ---

    #[test]
    fn scope_divergent_binder_bare_ref_scoped_binder_detects() {
        // bare ref "a" vs scoped binder "a\u{1}433" → Some
        let bare_ref = Identifier::bare("a");
        let scoped_key = "a\u{1}433";
        let result = super::scope_divergent_binder(&bare_ref, std::iter::once(scoped_key));
        assert_eq!(result, Some(scoped_key.to_owned()), "bare ref vs scoped binder must detect divergence");
    }

    #[test]
    fn scope_divergent_binder_scoped_ref_bare_binder_detects() {
        // scoped ref "a\u{1}7" vs bare binder "a" → Some
        let s = fresh_scope();
        let scoped_ref = Identifier::bare("a").add_scope(s);
        let bare_key = "a";
        let result = super::scope_divergent_binder(&scoped_ref, std::iter::once(bare_key));
        assert_eq!(result, Some(bare_key.to_owned()), "scoped ref vs bare binder must detect divergence");
    }

    #[test]
    fn scope_divergent_binder_same_key_no_detection() {
        // same key both sides → None
        let bare_ref = Identifier::bare("a");
        let same_key = "a";
        let result = super::scope_divergent_binder(&bare_ref, std::iter::once(same_key));
        assert_eq!(result, None, "same key must not detect divergence");
    }

    #[test]
    fn scope_divergent_binder_different_name_no_detection() {
        // different name → None
        let bare_ref = Identifier::bare("a");
        let different_key = "b\u{1}433";
        let result = super::scope_divergent_binder(&bare_ref, std::iter::once(different_key));
        assert_eq!(result, None, "different name must not detect divergence");
    }

    #[test]
    fn scope_divergent_binder_empty_locals_no_detection() {
        // empty locals → None
        let bare_ref = Identifier::bare("a");
        let result = super::scope_divergent_binder(&bare_ref, std::iter::empty());
        assert_eq!(result, None, "empty locals must not detect divergence");
    }
}
