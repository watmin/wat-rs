//! `Identifier` — bare-name references with scope tracking.
//!
//! Bare symbols (let-binding names, fn parameters, match patterns,
//! and their reference sites) need scope discrimination for hygienic
//! macro expansion per Racket's sets-of-scopes model (Flatt 2016).
//!
//! An [`Identifier`] is a (name, `BTreeSet<ScopeId>`) pair. Two
//! identifiers are "the same" iff both their names AND their scope
//! sets are equal. Lexical scope lookups therefore distinguish
//! `tmp` the user wrote from `tmp` a macro introduced — same name,
//! different scope sets, different identity.
//!
//! # When scopes are added
//!
//! - **Fresh parse.** Every identifier the parser produces has an
//!   empty scope set. All references-by-name work the same as before
//!   the Identifier refactor until a macro expands.
//! - **Macro expansion.** At each `defmacro` invocation the expander
//!   mints a fresh [`ScopeId`] and adds it to every identifier that
//!   originated in the macro's template. Identifiers that came from
//!   the macro's arguments (via `,x` unquote) keep their original
//!   scope sets. See `src/macros/expand.rs` (arc 249 / slice 5c).
//!
//! # Keywords do not need scopes
//!
//! Keyword tokens (`:wat::core::define`, `:my::app::foo`) are fully-
//! qualified paths. A macro introducing `:my::macro::tmp` cannot collide
//! with user code's `:my::app::tmp` because the paths differ. Hygiene
//! only attaches to `WatAST::Symbol`.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique integer identifying a lexical scope — macro invocation,
/// `let` / `fn` / `match` scope, etc.
///
/// `ScopeId`s are monotonically allocated by [`fresh_scope`] across the
/// whole process. The numeric value is opaque for semantics (never inspect
/// it for domain meaning). `hash.rs` consumes `ScopeId` via its derived
/// `Hash`/`Eq` traits (not via `as_u64`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(u64);

impl ScopeId {
    /// The raw `u64` token for env-key string encoding. Do not interpret
    /// the value as domain state.
    pub(crate) fn as_u64(self) -> u64 {
        self.0
    }
}

// rune:sequi(host-idiom) — draws a process-global `static NEXT: AtomicU64`
// (hidden from the signature); the ScopeId it returns is threaded explicitly
// through all downstream call sites (expand.rs). The counter carries no domain
// state — only process-unique scope identity; threading a mutable counter
// through every expansion signature would pollute them for no sequi benefit.
// rune:struere(host-constraint) — the global AtomicU64 is hidden from the
// signature by design: threading a counter through every expansion call site
// would pollute signatures for a value that carries no domain state; the
// monotone increment is the entire contract.
/// Allocate a fresh, unique [`ScopeId`].
pub fn fresh_scope() -> ScopeId {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    // Ordering::Relaxed is correct: the counter synchronizes no other memory;
    // uniqueness via atomic fetch_add is the entire contract; stronger orderings
    // (Acquire/Release/SeqCst) buy nothing here.
    ScopeId(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A name-with-scopes reference.
///
/// The name must never contain `\u{1}` (U+0001, ASCII SOH). The lexer now
/// REJECTS all raw control characters in source (Stone 249 scope-closure),
/// so this invariant is ENFORCED by the lexer, not merely conventional.
/// Construction is additionally guarded in debug builds at
/// [`Identifier::bare`] — the single chokepoint. See `resolution`'s module
/// doc for why.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    name: String,
    scopes: BTreeSet<ScopeId>,
}

impl Identifier {
    /// An identifier with an empty scope set — what the parser emits.
    ///
    /// # Panics (debug builds only)
    ///
    /// Asserts that `name` does not contain `\u{1}` (U+0001). The lexer now
    /// REJECTS raw control characters in source (Stone 249 scope-closure), so
    /// lexer-produced names structurally cannot contain this byte. This assert
    /// guards the debug path for names constructed via other routes.
    /// See `resolution`'s module doc for why.
    pub fn bare(name: impl Into<String>) -> Self {
        let name = name.into();
        // rune:struere(performance-hotspot) — release-mode validation here would
        // put a contains() scan on every Identifier construction (the parse hot
        // path); the debug-checked single chokepoint + the lexer's token rules are
        // the chosen rung. Promote to a validated newtype if the invariant ever
        // becomes security-load-bearing.
        debug_assert!(
            !name.contains('\u{1}'),
            "Identifier name must not contain U+0001 (env-key separator); got {:?}",
            name
        );
        Identifier {
            name,
            scopes: BTreeSet::new(),
        }
    }

    /// A new `Identifier` equal to `self` but with `scope` added to its
    /// scope set. Original unmodified — cheap via `BTreeSet::clone` +
    /// one insert.
    pub fn add_scope(&self, scope: ScopeId) -> Self {
        let mut scopes = self.scopes.clone();
        scopes.insert(scope);
        Identifier {
            name: self.name.clone(),
            scopes,
        }
    }

    /// The bare name, scope-free. For env keying route through `env_key` —
    /// the bare str alone is not a resolution key for scoped identifiers.
    pub fn as_str(&self) -> &str {
        &self.name
    }

    // rune:struere(invariant-coupling) — &BTreeSet IS the contract: its sorted,
    // deterministic iteration is load-bearing (env_key's canonical encoding and
    // hash.rs's scope renumbering both depend on the ordering); an opaque iterator
    // would hide the very guarantee consumers must rely on.
    /// Borrow the scope set — read-only. To add a scope, use [`add_scope`].
    ///
    /// [`add_scope`]: Self::add_scope
    pub fn scopes(&self) -> &BTreeSet<ScopeId> {
        &self.scopes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_has_empty_scopes() {
        let id = Identifier::bare("x");
        assert_eq!(id.name, "x");
        assert!(id.scopes.is_empty());
    }

    #[test]
    fn same_name_empty_scopes_are_equal() {
        assert_eq!(Identifier::bare("x"), Identifier::bare("x"));
    }

    #[test]
    fn same_name_different_scopes_are_distinct() {
        let s = fresh_scope();
        assert_ne!(Identifier::bare("x"), Identifier::bare("x").add_scope(s));
    }

    #[test]
    fn scopes_are_monotonic_unique() {
        let a = fresh_scope();
        let b = fresh_scope();
        let c = fresh_scope();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn add_scope_is_idempotent() {
        let s = fresh_scope();
        let base = Identifier::bare("x");
        assert_eq!(base.add_scope(s), base.add_scope(s).add_scope(s));
    }

    #[test]
    fn identifiers_are_hashable() {
        use std::collections::HashSet;
        let mut set: HashSet<Identifier> = HashSet::new();
        set.insert(Identifier::bare("x"));
        set.insert(Identifier::bare("x").add_scope(fresh_scope()));
        assert_eq!(set.len(), 2, "identifiers differ by scope");
    }
}
