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

/// The reserved namespace every non-namespaced (binder) symbol carries.
/// Reserved so user source cannot define into it — see
/// `src/resolve/reserved.rs`'s `RESERVED_PREFIXES` (entry `":$bound::"`,
/// doubled-colon form to match `is_reserved_prefix`'s stripping).
pub const BOUND_NAMESPACE: &str = "$bound";

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
    pub fn as_u64(self) -> u64 {
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

    /// The symbol's namespace. TOTAL — every symbol has one; a binder's is
    /// [`BOUND_NAMESPACE`] (`$bound`). Never an absence: the uniform shape is
    /// the point (see `DESIGN-STONE-251.8-symbol-proper.md`'s pinned
    /// contract).
    ///
    /// STONE 251.8a: the namespace is DERIVED from the spelling (split on
    /// the last `/`), not stored — `Identifier` still holds one `name`
    /// string. 251.8b is where derived swaps for stored behind this same
    /// signature.
    pub fn namespace(&self) -> &str {
        match self.name.rfind('/') {
            Some(slash) => &self.name[..slash],
            None => BOUND_NAMESPACE,
        }
    }

    /// True when this symbol names something defined elsewhere, false when
    /// it is a local binder. Exactly `namespace() != BOUND_NAMESPACE` — the
    /// one indirection a reader has to cross between `$bound` and this.
    pub fn is_reference(&self) -> bool {
        self.namespace() != BOUND_NAMESPACE
    }

    /// The last `::`-delimited segment of the spelling. See [`leaf`].
    pub fn leaf(&self) -> &str {
        leaf(&self.name)
    }

    /// Everything before [`leaf`](Self::leaf). See [`path`].
    pub fn path(&self) -> &str {
        path(&self.name)
    }

    /// Everything before the `/` of a surface-method call head. See [`receiver`].
    pub fn receiver(&self) -> &str {
        receiver(&self.name)
    }

    /// Everything after the `/` of a surface-method call head. See [`method`].
    pub fn method(&self) -> &str {
        method(&self.name)
    }

    /// Is the spelling primed (ends in `'`)? See [`prime`].
    pub fn prime(&self) -> bool {
        prime(&self.name)
    }

    /// The spelling with a trailing `'` removed, if present. See [`deprimed`].
    pub fn deprimed(&self) -> &str {
        deprimed(&self.name)
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

// ─── The name grammar — free functions on `&str` ───────────────────────────
//
// STONE-one-name-grammar (arc 109): a name is an atom, and structure encoded
// inside an atom must be re-parsed by every consumer. These six functions are
// that one re-parse, written once. Each `Identifier` method above delegates
// to its free-function twin — one implementation, two surfaces, never two
// implementations (the discipline `namespace()` already set: one signature,
// callers never change). Most call sites hold a keyword's raw `&str`, not an
// `Identifier`, hence the free functions being the primary surface.
//
// Four edge cases are pinned in the tests below because the 33 hand-rolls
// this stone replaced did not all agree on them:
//
//   - no separator at all (`:foo`) — `leaf`/`method` (the "final component"
//     pair) return the WHOLE string; `path`/`receiver` (the "prefix" pair)
//     return `""`. This mirrors the near-universal `rsplit(...).next()
//     .unwrap_or(name)` idiom already at most call sites, generalized to
//     both pairs symmetrically.
//   - a leading colon (`:foo`, `:wat::cache::Lru`) — NEVER special-cased.
//     `::`/`/`/`'` search is colon-agnostic, so a leading `:` rides along in
//     whichever half it lands in (kept in `leaf`/`path`'s output exactly as
//     found). A caller that wants it gone (e.g. `option_result_tag`) strips
//     it itself with `.trim_start_matches(':')` — that remains an ordinary
//     caller-side string op, not part of this grammar.
//   - an empty segment (`:a::`, trailing `::`) — falls out of the plain
//     `rsplit`/`rfind` math with no special-casing: `leaf(":a::") == ""`,
//     `path(":a::") == ":a"`.
//   - primed AND slashed (`:sort'/apply`) — `prime`/`deprimed` never
//     descend into slash structure on their own: `prime(":sort'/apply")` is
//     `false` (the STRING ends in `apply`, not `'`). Asking "is the
//     receiver primed" is a compose: `prime(receiver(name))`.

/// The last `::`-delimited segment of `name` (`:wat::cache::Lru` → `Lru`). No
/// `::` present → the WHOLE string (nothing precedes it, so it is its own leaf).
pub fn leaf(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

/// Everything before [`leaf`] (`:wat::cache::Lru` → `:wat::cache`). No `::`
/// present → `""` — there is no path before a name that IS its own leaf.
pub fn path(name: &str) -> &str {
    match name.rfind("::") {
        Some(idx) => &name[..idx],
        None => "",
    }
}

/// Everything before the `/` of a surface-method call head (`:S/mk` → `:S`).
/// No `/` present → `""` — there is no receiver on a name with no method call.
pub fn receiver(name: &str) -> &str {
    match name.rfind('/') {
        Some(idx) => &name[..idx],
        None => "",
    }
}

/// Everything after the `/` of a surface-method call head (`:S/mk` → `mk`).
/// No `/` present → the WHOLE string — a bare name is its own method with no
/// receiver.
pub fn method(name: &str) -> &str {
    match name.rfind('/') {
        Some(idx) => &name[idx + 1..],
        None => name,
    }
}

/// Is `name` primed — does it end in `'`? (`:sort'` → `true`). Operates on
/// exactly the string given; does not descend into `/`-structure — see the
/// module note above for the primed-and-slashed edge case.
pub fn prime(name: &str) -> bool {
    name.ends_with('\'')
}

/// `name` with a trailing `'` removed, if present; unchanged otherwise
/// (`:sort'` → `:sort`).
pub fn deprimed(name: &str) -> &str {
    name.strip_suffix('\'').unwrap_or(name)
}

/// Split a DOT-separated coercion-error path (`".items.[0]"`, built leaf-upward by
/// `EdnCoerceError::at`) into its non-empty segments (`["items", "[0]"]`).
///
/// This is a DIFFERENT grammar from the `::`/`/`/`'` name grammar above — dot-joined,
/// not a wat `Identifier` spelling at all — but the same disease STONE-one-name-grammar
/// (arc 109) attacks: `runtime_error_edn.rs::edn_path_segments` and
/// `runtime.rs::edn_coerce_path_segments` were two independent implementations of this
/// exact split before the stone collapsed them onto this one. An empty `path` (the
/// mismatch is the value itself, not a sub-field) yields an empty `Vec`.
pub fn dot_path_segments(path: &str) -> Vec<&str> {
    path.split('.').filter(|s| !s.is_empty()).collect()
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

    // STONE 251.8a — the one-door probe. A binder identifier (no `/` in its
    // spelling) answers the reserved BOUND_NAMESPACE from `namespace()` and
    // `false` from `is_reference()`; a namespaced identifier answers its own
    // namespace and `true`. This is deliberately NOT just the negative case
    // (trap door: "the probe could pass on a tautology") — it also asserts
    // the reference direction on a real namespaced identifier.
    #[test]
    fn binder_symbol_is_not_a_reference() {
        let id = Identifier::bare("x");
        assert_eq!(id.namespace(), BOUND_NAMESPACE);
        assert!(!id.is_reference());
    }

    #[test]
    fn namespaced_symbol_is_a_reference() {
        let id = Identifier::bare("wat.core/+");
        assert_eq!(id.namespace(), "wat.core");
        assert!(id.is_reference());
    }

    // ── STONE-one-name-grammar: the four pinned edge cases ─────────────────

    #[test]
    fn leaf_and_path_split_on_the_last_double_colon() {
        assert_eq!(leaf(":wat::cache::Lru"), "Lru");
        assert_eq!(path(":wat::cache::Lru"), ":wat::cache");
    }

    #[test]
    fn receiver_and_method_split_on_the_slash() {
        assert_eq!(receiver(":S/mk"), ":S");
        assert_eq!(method(":S/mk"), "mk");
    }

    #[test]
    fn prime_and_deprimed_read_the_trailing_quote() {
        assert!(prime(":sort'"));
        assert_eq!(deprimed(":sort'"), ":sort");
        assert!(!prime(":sort"));
        assert_eq!(deprimed(":sort"), ":sort");
    }

    /// Edge case 1 — no separator at all. The "final component" pair
    /// (`leaf`/`method`) returns the WHOLE string; the "prefix" pair
    /// (`path`/`receiver`) returns `""`.
    #[test]
    fn no_separator_leaf_and_method_are_total_path_and_receiver_are_empty() {
        assert_eq!(leaf(":foo"), ":foo");
        assert_eq!(path(":foo"), "");
        assert_eq!(method(":foo"), ":foo");
        assert_eq!(receiver(":foo"), "");
    }

    /// Edge case 2 — a leading colon is never special-cased by any accessor;
    /// it rides along in whichever half it lands in, exactly as found.
    #[test]
    fn leading_colon_is_never_stripped_by_the_door() {
        assert_eq!(leaf(":wat::cache::Lru"), "Lru");
        assert_eq!(path(":wat::cache::Lru"), ":wat::cache"); // colon KEPT
        assert_eq!(receiver(":S/mk"), ":S"); // colon KEPT
        // A caller that wants it gone strips it itself, same as
        // `option_result_tag` (src/rete/expr_ir.rs) already did before this
        // stone and still does after it.
        assert_eq!(leaf(":wat::cache::Lru").trim_start_matches(':'), "Lru");
    }

    /// Edge case 3 — an empty segment (a trailing `::`, e.g. a namespace-
    /// prefix marker like `:counter::`). Falls out of the plain rsplit/rfind
    /// math with no special-casing.
    #[test]
    fn trailing_double_colon_leaves_an_empty_leaf() {
        assert_eq!(leaf(":a::"), "");
        assert_eq!(path(":a::"), ":a");
    }

    /// Edge case 4 — a name that is both primed AND slashed. `prime`/
    /// `deprimed` never descend into slash structure on their own: asking
    /// whether the receiver is primed is a compose, `prime(receiver(name))`.
    #[test]
    fn primed_and_slashed_prime_does_not_descend_into_receiver_method() {
        let name = ":sort'/apply";
        // The whole string does NOT end in `'` — it ends in `apply`.
        assert!(!prime(name));
        assert_eq!(deprimed(name), name);
        // The split into receiver/method happens first; prime reads the
        // receiver segment once it has been pulled out.
        assert_eq!(receiver(name), ":sort'");
        assert_eq!(method(name), "apply");
        assert!(prime(receiver(name)));
        assert_eq!(deprimed(receiver(name)), ":sort");
    }
}
