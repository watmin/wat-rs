//! `Identifier` — bare-name references with scope tracking.
//!
//! Bare symbols (let-binding names, fn parameters, match patterns,
//! and their reference sites) need scope discrimination for hygienic
//! macro expansion per Racket's sets-of-scopes model (Flatt 2016).
//!
//! An [`Identifier`] is a `(namespace, name)` tuple with a scope set
//! riding alongside (Racket sets-of-scopes hygiene — not a third
//! member of the name). `wat.core/+` is `[wat.core, +]`; `foo` is
//! `[$bound, foo]`.
//!
//! # ⭐ THE SPLIT RULE — the first slash separates namespace from name
//!
//! Builder's ruling, 2026-09-19. One sentence, and it is TOTAL: every
//! spelling has a first `/` or none, and neither case yields an empty name.
//!
//! ```text
//! wat.core/+                →  [wat.core, +]
//! wat.core//                →  [wat.core, /]          (Clojure's own `clojure.core//`)
//! user/whatever/name/here/  →  [user, whatever/name/here/]
//! foo                       →  [$bound, foo]
//! ```
//!
//! A pathological name is tolerated GRACEFULLY: it stays a weird *name*
//! instead of corrupting the *namespace*. The rule this replaced took the
//! LAST `/`, which answered both multi-slash cases above with an empty name
//! and a namespace that had swallowed the path — not a tolerated name, a
//! silently malformed one that rendered as the unspellable `:wat::core/::`.
//!
//! ⚠ The rule lives HERE and nowhere else. `tests/lint/one_name_grammar.rs`
//! fails any second implementation of it (`rfind('/')`, `rsplit_once('/')`,
//! …) elsewhere in the tree — which is why changing it is a four-line edit
//! in one file rather than a corpus census. Two identifiers are "the same"
//! iff their `(namespace, name)` pairs AND their scope sets are equal.
//! `flat` is the print cache of that pair (`"{namespace}/{name}"`, or the
//! bare name when the namespace is `$bound`) and does not decide equality.
//! A slashed local's binder (`{$bound, foo/bar}`) and its body reference
//! (`{foo, bar}`) are not equal; [`Identifier::same_local`] finds the binder.
//! Lexical scope lookups therefore distinguish `tmp` the user wrote from
//! `tmp` a macro introduced — same name, different scope sets, different identity.
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
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// The reserved namespace every non-namespaced (binder) symbol carries.
/// Reserved so user source cannot define into it — see
/// `src/resolve/reserved.rs`'s `RESERVED_PREFIXES` (entry `":$bound::"`,
/// doubled-colon form to match `is_reserved_prefix`'s stripping).
pub const BOUND_NAMESPACE: &str = "$bound";

/// A name is the pair `(namespace, name)`. No string is the identity.
/// [`std::fmt::Display`] is the one stringification: `"{namespace}/{name}"`, or the
/// bare `name` when the namespace is [`BOUND_NAMESPACE`].
///
/// `Eq` and `Hash` are the contents of the two fields. A pointer-equality
/// fast path in `Eq` is allowed; `Hash` always hashes the text, never the
/// pointer. There is no global interner.
#[derive(Clone, Debug)]
pub struct Name {
    pub namespace: std::sync::Arc<str>,
    pub name: std::sync::Arc<str>,
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        (std::sync::Arc::ptr_eq(&self.namespace, &other.namespace)
            || self.namespace == other.namespace)
            && (std::sync::Arc::ptr_eq(&self.name, &other.name) || self.name == other.name)
    }
}

impl Eq for Name {}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.namespace
            .as_ref()
            .cmp(other.namespace.as_ref())
            .then_with(|| self.name.as_ref().cmp(other.name.as_ref()))
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Contents, not the Arc pointer. Derived `Hash` for `Arc<str>` already
        // hashes the `str`; written out so a pointer hash cannot replace it.
        self.namespace.as_ref().hash(state);
        self.name.as_ref().hash(state);
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.namespace.as_ref() == BOUND_NAMESPACE {
            f.write_str(&self.name)
        } else {
            write!(f, "{}/{}", self.namespace, self.name)
        }
    }
}

impl Name {
    /// A binder's pair: namespace [`BOUND_NAMESPACE`], name the whole spelling.
    pub fn bound(name: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            namespace: std::sync::Arc::from(BOUND_NAMESPACE),
            name: name.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The pair a keyword spelling of a name has, as the recorded converter
    /// rewrites it (`wat.keyword/to-symbol`, plus a trailing-`::` marker).
    ///
    /// A keyword that is not a name (`:k`, `:else` — no `::`) is `None`.
    /// One spelling has one image; there is no position argument. Deleted
    /// at the wall, with keywords.
    pub fn from_keyword(kw: &str) -> Option<Self> {
        let body = kw.strip_prefix(':')?;
        // A trailing `::` is a namespace marker (`:my::kernel::` → `my.kernel`),
        // which `wat_keyword_to_clojure_symbol` declines. The name is the whole
        // dotted spelling and the namespace is [`BOUND_NAMESPACE`], matching
        // `Identifier::bare` of that symbol (no `/`).
        if let Some(rest) = body.strip_suffix("::") {
            let dotted = rest.replace("::", ".");
            return Some(Name::bound(dotted));
        }
        if !body.contains("::") {
            return None;
        }
        // `body` contains `::` and does not end in `::`, so there are at least
        // two non-empty segments. Same split `wat_keyword_to_clojure_symbol` uses.
        let final_seg = leaf(body);
        let mut ns_parts: Vec<&str> = path(body).split("::").collect();
        let name: &str = if final_seg.contains('/') && !receiver(final_seg).is_empty() {
            ns_parts.push(receiver(final_seg));
            method(final_seg)
        } else {
            final_seg
        };
        let namespace = ns_parts.join(".");
        Some(Name {
            namespace: std::sync::Arc::from(namespace),
            name: std::sync::Arc::from(name),
        })
    }

    /// A spelling enters a registry here, once.
    ///
    /// A keyword [`Self::from_keyword`] accepts is that pair. A rust-scheme path
    /// with no leading colon (`wat::core::foo`) is the keyword `:{path}`.
    /// Anything else — a clojure symbol, a binder, a value keyword — is
    /// [`Identifier::bare`]'s pair. A rendered parametric form (`(`, `<`, or a
    /// `:-` binder that is not a `::` leaf beginning with `-`) is not a name:
    /// this returns `None` and the caller keeps the text. `:wat::core::->` is a name.
    pub fn enter(spelling: &str) -> Option<Self> {
        if is_rendered_type_text(spelling) {
            return None;
        }
        if let Some(name) = Self::from_keyword(spelling) {
            return Some(name);
        }
        if !spelling.starts_with(':') && spelling.contains("::") {
            let mut kw = String::with_capacity(1 + spelling.len());
            kw.push(':');
            kw.push_str(spelling);
            if let Some(name) = Self::from_keyword(&kw) {
                return Some(name);
            }
        }
        Some(Identifier::bare(spelling).pair().clone())
    }
}

fn is_rendered_type_text(spelling: &str) -> bool {
    if spelling.contains('(') || spelling.contains('<') {
        return true;
    }
    // `:-` is the binder separator in a rendered type. A leaf that begins with
    // `-` (`->`, `->>`, `-`, `-type-slot-name`) is written after `::`, so the
    // two characters `:-` occur inside a name. That spelling is still a name.
    let bytes = spelling.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b':' && bytes[i + 1] == b'-' {
            let after_double_colon = i > 0 && bytes[i - 1] == b':';
            if !after_double_colon {
                return true;
            }
        }
        i += 1;
    }
    false
}

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
#[derive(Clone)]
pub struct Identifier {
    /// The `(namespace, name)` pair. Equality and hashing read this, not `flat`.
    pair: Name,
    /// Print cache of the pair: the spelling [`as_str`](Self::as_str) returns.
    /// Derived once in [`bare`](Self::bare). Stone 5 deletes it. Nothing
    /// decides identity by it; [`local_spelling`](Self::local_spelling) borrows
    /// it for a reference only because [`std::fmt::Display`] of that pair is this string.
    flat: std::sync::Arc<str>,
    /// Macro hygiene — orthogonal to the `(namespace, name)` pair.
    scopes: BTreeSet<ScopeId>,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.pair == other.pair && self.scopes == other.scopes
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pair.hash(state);
        self.scopes.hash(state);
    }
}

impl std::fmt::Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identifier")
            .field("name", &self.flat)
            .field("scopes", &self.scopes)
            .finish()
    }
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
        let flat: std::sync::Arc<str> = name.into().into();
        // rune:struere(performance-hotspot) — release-mode validation here would
        // put a contains() scan on every Identifier construction (the parse hot
        // path); the debug-checked single chokepoint + the lexer's token rules are
        // the chosen rung. Promote to a validated newtype if the invariant ever
        // becomes security-load-bearing.
        debug_assert!(
            !flat.contains('\u{1}'),
            "Identifier name must not contain U+0001 (env-key separator); got {:?}",
            flat
        );
        // The tuple is derived ONCE, here, from THE FIRST `/` — the builder's
        // ruling, 2026-09-19: *"the first slash separates namespace from name."*
        // Accessors return the stored fields; they do not re-split.
        //
        // This was `rfind` (the LAST `/`) until the ruling. Last-slash answered a
        // multi-slash spelling with an EMPTY name and a namespace that had eaten
        // the path (`user/whatever/name/here/` → `["user/whatever/name/here", ""]`,
        // rendering as the unspellable `:user/whatever/name/here::`). First-slash
        // is total: every spelling has a first `/` or none, and neither case is
        // empty. It also makes this agree with `wat-edn`'s `split_namespaced`,
        // which has always used `find` — the wat surface was the OUTLIER, not the
        // definition. (`wat-edn`'s EDN-TEXT parser separately rejects 2+ slashes
        // per the EDN spec; that stays, and is a different question from how a
        // name that reaches us anyway is split.)
        let pair = match flat.find('/') {
            Some(slash) => Name {
                namespace: std::sync::Arc::<str>::from(&flat[..slash]),
                name: std::sync::Arc::<str>::from(&flat[slash + 1..]),
            },
            None => Name::bound(std::sync::Arc::clone(&flat)),
        };
        Identifier {
            pair,
            flat,
            scopes: BTreeSet::new(),
        }
    }

    /// Binder position. The whole spelling is the name, whatever `/` it
    /// holds; the namespace is [`BOUND_NAMESPACE`]. `flat` is unchanged,
    /// so [`as_str`](Self::as_str) still prints the spelling. Equality is
    /// the pair: a slashed binder (`{$bound, foo/bar}`) does not compare
    /// equal to the body reference (`{foo, bar}`). [`same_local`](Self::same_local)
    /// is how `substitute` and `env_key` find that binder — the binder's
    /// name against the reference's print cache, plus the scope set — not
    /// a second split of `flat`.
    ///
    /// Idempotent: a symbol that is already a binder is returned as-is.
    /// The reader does not call this. Normalize and argspec parsing do,
    /// once they know the symbol is sitting in a binder position.
    pub fn into_bound(self) -> Self {
        if !self.is_reference() {
            return self;
        }
        Identifier {
            pair: Name::bound(std::sync::Arc::clone(&self.flat)),
            flat: self.flat,
            scopes: self.scopes,
        }
    }

    /// A new `Identifier` equal to `self` but with `scope` added to its
    /// scope set. Original unmodified — the name bytes are shared (`Arc`);
    /// the scope set is a new `BTreeSet`.
    pub fn add_scope(&self, scope: ScopeId) -> Self {
        let mut scopes = self.scopes.clone();
        scopes.insert(scope);
        Identifier {
            pair: self.pair.clone(),
            flat: std::sync::Arc::clone(&self.flat),
            scopes,
        }
    }

    /// The `(namespace, name)` pair.
    pub fn pair(&self) -> &Name {
        &self.pair
    }

    /// The spelling a local binding is known by.
    ///
    /// A binder borrows its name field (the whole spelling). A reference
    /// borrows `flat`, the print cache, because [`std::fmt::Display`] of its pair is
    /// that same `namespace/name`. The branch is which pair this is, not a
    /// re-split of the cache.
    pub fn local_spelling(&self) -> &str {
        if self.is_reference() {
            &self.flat
        } else {
            &self.pair.name
        }
    }

    /// Whether `self` and `other` resolve to the same local binder.
    ///
    /// Scope sets match, and [`local_spelling`](Self::local_spelling) matches.
    /// Pair equality is stricter and does not hold for a slashed binder
    /// against its body reference.
    pub fn same_local(&self, other: &Self) -> bool {
        self.scopes == other.scopes && self.local_spelling() == other.local_spelling()
    }

    /// The bare name, scope-free. For env keying route through `env_key` —
    /// the bare str alone is not a resolution key for scoped identifiers.
    pub fn as_str(&self) -> &str {
        &self.flat
    }

    /// The symbol's namespace. TOTAL — every symbol has one; a binder's is
    /// [`BOUND_NAMESPACE`] (`$bound`). Never an absence: the uniform shape is
    /// the point (see `DESIGN-STONE-251.8-symbol-proper.md`'s pinned
    /// contract).
    ///
    /// STONE 251.8b: stored at construction ([`bare`](Self::bare)), not
    /// re-derived. Same `&str` signature 251.8a promised.
    pub fn namespace(&self) -> &str {
        &self.pair.namespace
    }

    /// True when this symbol names something defined elsewhere, false when
    /// it is a local binder. Exactly `namespace() != BOUND_NAMESPACE` — the
    /// one indirection a reader has to cross between `$bound` and this.
    pub fn is_reference(&self) -> bool {
        self.namespace() != BOUND_NAMESPACE
    }

    /// The last `::`-delimited segment of the spelling. See [`leaf`].
    pub fn leaf(&self) -> &str {
        leaf(&self.flat)
    }

    /// Everything before [`leaf`](Self::leaf). See [`path`].
    pub fn path(&self) -> &str {
        path(&self.flat)
    }

    /// Everything before the `/` of a surface-method call head. See [`receiver`].
    ///
    /// The prefix *is* the stored namespace whenever the spelling had a `/`.
    /// A slash-less binder has receiver `""`, not `$bound`. A binder that
    /// was forced with [`into_bound`](Self::into_bound) keeps its `/` in
    /// `flat`, so its receiver is the stored namespace (`$bound`).
    pub fn receiver(&self) -> &str {
        if self.flat.contains('/') {
            &self.pair.namespace
        } else {
            ""
        }
    }

    /// Everything after the `/` of a surface-method call head. See [`method`].
    /// The stored name half of the tuple (the whole spelling, for a binder).
    pub fn method(&self) -> &str {
        &self.pair.name
    }

    /// Is the spelling primed (ends in `'`)? See [`prime`].
    pub fn prime(&self) -> bool {
        prime(&self.flat)
    }

    /// The spelling with a trailing `'` removed, if present. See [`deprimed`].
    pub fn deprimed(&self) -> &str {
        deprimed(&self.flat)
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
// that one re-parse, written once. `leaf`/`path`/`prime`/`deprimed` on
// `Identifier` still delegate to the free-function twin (the `::` / `'`
// grammar is not the stored tuple). `namespace`/`receiver`/`method` return
// stored fields. Most call sites hold a keyword's raw `&str`, not an
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

/// Everything before the FIRST `/` of a surface-method call head
/// (`:S/mk` → `:S`). No `/` present → `""` — there is no receiver on a name
/// with no method call.
///
/// ⛔ **First, not last — and this is load-bearing beyond method dispatch.**
/// `resolve/normalize.rs` calls `receiver`/`method` to compute a symbol's
/// (namespace, local_name), so these two ARE the namespace splitter on that
/// path. Were they to keep the last-`/` rule while `Identifier::bare` took
/// the first, one process would hold two different namespaces for the same
/// string. They move with the ruling for that reason, measured — not because
/// method dispatch and namespacing are the same concept.
pub fn receiver(name: &str) -> &str {
    match name.find('/') {
        Some(idx) => &name[..idx],
        None => "",
    }
}

/// Everything after the FIRST `/` of a surface-method call head
/// (`:S/mk` → `mk`). No `/` present → the WHOLE string — a bare name is its
/// own method with no receiver. See [`receiver`] for why this is the first
/// `/` and not the last.
pub fn method(name: &str) -> &str {
    match name.find('/') {
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

// ── STONE-variant-composition-door (arc 255) ────────────────────────────
//
// The one-name grammar above had ten decomposition accessors and, until this
// stone, zero composers: a name that needed to be split was split here,
// once; a name that needed to be BUILT was built by hand at fifteen call
// sites in two spellings
// (`docs/arc/2026/06/255-builtin-registry/DESIGN-the-dot-flip-is-a-COMPOSITION-problem.md`).
// These two functions are the inverse of `path`/`leaf` above, specialized to
// the one thing all fifteen call sites actually compose: an enum variant's
// name from its declared enum and its variant.

/// Compose an enum variant's LOOKUP/keyword spelling: `enum_path.variant_name`.
/// The `.` join is the inverse of [`path`]/[`leaf`] applied to a variant
/// reference: `compose_variant(path(x), leaf(x)) == x` for any `x` this
/// function could have produced.
///
/// **Precondition — `enum_path` is used exactly as given; this function does
/// NOT strip or add a leading colon.** Two live storage conventions disagree
/// on that colon (`EnumValue.type_path` keeps it — "matches the enum's
/// declared name verbatim" — while `AggregateValue.class` and declaration-
/// time enum names drop it; see `src/value/value.rs`). Unifying those
/// storage conventions is a separate, larger stone
/// (`BRIEF-the-variant-name-gets-ONE-composition-door.md` STOP-2); this door
/// is colon-agnostic by design, exactly as `leaf`/`path` above are
/// colon-agnostic on the way in (see the "leading colon is never
/// special-cased" edge case documented above). The caller presents whatever
/// shape it already holds — a pre-stripped `enum_path` composes a
/// colon-free result, an as-stored `enum_path` composes a colon-ful one —
/// and this function's only business is the `.` in the middle.
///
/// The separator is THIS function's decision, made once: flipping
/// `Enum::Variant` to `Enum.Variant` (the wat-rs dot-flip) is a one-line
/// change to this body and nothing else.
pub fn compose_variant(enum_path: &str, variant_name: &str) -> String {
    format!("{enum_path}.{variant_name}")
}

/// The INVERSE of [`compose_variant`]: split a variant's LOOKUP/keyword spelling
/// (`enum_path.variant_name`) back into `(enum_path, variant_name)`.
///
/// The two must always agree — the separator is one decision, and it is spelled in
/// exactly these two function bodies. `compose_variant` writes `.`; this reads `.`.
/// Moving the separator means changing both, together, in this file only.
///
/// This exists as its own function, rather than a caller reaching for [`path`]/[`leaf`]
/// above, because those two are GENERAL splitters used for every namespaced name in the
/// substrate — they cannot be given a variant-specific separator without breaking every
/// other namespaced name that also goes through them. The variant pair is deliberately
/// narrow so the general grammar can stay general.
///
/// And it lives here, beside `compose_variant`, rather than as a hand-rolled `rfind` at
/// its one caller (`TypeEnv::variant_parent_enum`, `src/types.rs`): the one-name-grammar
/// lint bans a hand-rolled `rfind` outside `identifier.rs`.
///
/// `None` when `name` has no `.` — mirroring [`path`]'s `""` return for the same input,
/// which callers already treat as "not a variant" (e.g. `variant_parent_enum`'s
/// `parent.is_empty()` guard).
pub fn decompose_variant(name: &str) -> Option<(&str, &str)> {
    let idx = name.rfind('.')?;
    Some((&name[..idx], &name[idx + 1..]))
}

/// Compose an enum variant's RENDER spelling: `enum_leaf.variant_name`.
///
/// Distinct from [`compose_variant`], not a mode flag on it: the two EDN
/// render call sites hold only the enum's bare LEAF — never its full path —
/// and already join with `.` rather than `::` (the wat-rs target notation,
/// live at these two sites ahead of the rest of the flip). Flattening this
/// into one signature would hide that the two call sites have different
/// preconditions (leaf vs. path); see
/// `BRIEF-the-variant-name-gets-ONE-composition-door.md`.
pub fn compose_variant_render(enum_leaf: &str, variant_name: &str) -> String {
    format!("{enum_leaf}.{variant_name}")
}

/// Split a DOT-separated coercion-error path (`".items.[0]"`, built leaf-upward by
/// `EdnCoerceError::at`) into its non-empty segments (`["items", "[0]"]`).
///
/// This is a DIFFERENT grammar from the `::`/`/`/`'` name grammar above — dot-joined,
/// not a wat `Identifier` spelling at all — but the same disease STONE-one-name-grammar
/// (arc 109) attacks: `edn::error::edn_path_segments` and
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
        assert_eq!(id.method(), "x");
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
        assert_eq!(id.method(), "+");
        assert_eq!(id.method(), "+");
    }

    #[test]
    fn foo_is_bound_foo() {
        let id = Identifier::bare("foo");
        assert_eq!(id.namespace(), BOUND_NAMESPACE);
        assert_eq!(id.method(), "foo");
        assert_eq!(id.as_str(), "foo");
        assert!(!id.is_reference());
    }

    /// Negative control: `namespace()` must borrow the stored `ns` field,
    /// not a `rfind` slice of `flat`. Revert the accessor to a derivation
    /// and this goes RED.
    #[test]
    fn namespace_borrows_the_stored_field() {
        let id = Identifier::bare("wat.core/+");
        assert!(
            std::ptr::eq(id.namespace(), &*id.pair.namespace),
            "namespace() must return the stored namespace field"
        );
        let binder = Identifier::bare("foo");
        assert!(
            std::ptr::eq(binder.namespace(), &*binder.pair.namespace),
            "binder namespace() must return the stored namespace field, not the static BOUND_NAMESPACE"
        );
    }

    /// ⭐ THE BUILDER'S RULING, 2026-09-19 — *"the first slash separates
    /// namespace from name."* Both of the builder's own examples, pinned as
    /// LITERALS. This test previously asserted the opposite
    /// (`wat.core//` → `["wat.core/", ""]`) under the name
    /// `..._is_the_current_last_slash_split`, with the docstring "Reported,
    /// not fixed."
    ///
    /// ⛔ The expectations below are written out, never derived from the
    /// implementation. A test that recomputes the split it is checking
    /// passes under EITHER rule and so checks nothing
    /// (`[[feedback_a_derived_structure_makes_every_reader_guess]]`).
    #[test]
    fn the_first_slash_separates_namespace_from_name() {
        // Example 1 — `wat.core//`. The name is `/` (Clojure's own
        // `clojure.core//`), NOT the empty string.
        let id = Identifier::bare("wat.core//");
        assert_eq!(id.namespace(), "wat.core");
        assert_eq!(id.method(), "/");
        assert_eq!(id.receiver(), "wat.core");
        assert_eq!(id.method(), "/");
        assert_eq!(id.as_str(), "wat.core//");
        assert!(id.is_reference());

        // Example 2 — a pathological name is TOLERATED: it stays a weird
        // NAME instead of corrupting the NAMESPACE.
        let id = Identifier::bare("user/whatever/name/here/");
        assert_eq!(id.namespace(), "user");
        assert_eq!(id.method(), "whatever/name/here/");
        assert_eq!(id.receiver(), "user");
        assert_eq!(id.method(), "whatever/name/here/");
        assert_eq!(id.as_str(), "user/whatever/name/here/");
        assert!(id.is_reference());
    }

    /// A binder position keeps the whole spelling, including every `/`.
    #[test]
    fn into_bound_keeps_the_whole_spelling_as_the_name() {
        let id = Identifier::bare("foo/bar").into_bound();
        assert_eq!(id.namespace(), "$bound");
        assert_eq!(id.method(), "foo/bar");
        assert_eq!(id.as_str(), "foo/bar");
        assert!(!id.is_reference());
        assert_eq!(id.receiver(), "$bound");
        assert_eq!(id.method(), "foo/bar");
        let again = id.clone().into_bound();
        assert_eq!(again, id);

        let slashless = Identifier::bare("acc");
        assert_eq!(slashless.clone().into_bound(), slashless);
    }

    /// Stone 255.91 — pair equality. `a.b/c` is the image of both keyword
    /// spellings. A scoped copy is a different identifier. A slashed binder
    /// and its body reference do not compare equal; `same_local` resolves them.
    #[test]
    fn the_name_is_the_pair() {
        let bare = Identifier::bare("a.b/c");
        let from_colons = Name::from_keyword(":a::b::c").expect("path keyword");
        let from_slash = Name::from_keyword(":a::b/c").expect("member keyword");
        assert_eq!(bare.pair(), &from_colons);
        assert_eq!(bare.pair(), &from_slash);
        assert_eq!(from_colons.to_string(), "a.b/c");
        assert_eq!(Name::enter(":a::b::c").as_ref(), Some(bare.pair()));
        assert_eq!(Name::enter("a.b/c").as_ref(), Some(bare.pair()));
        assert_eq!(Name::enter("(:a::b :- [:T])"), None);
        let threading = Name::from_keyword(":wat::core::->").expect("threading macro");
        assert_eq!(Name::enter(":wat::core::->").as_ref(), Some(&threading));
        assert_eq!(Name::enter("wat.core/->").as_ref(), Some(&threading));
        assert_eq!(
            Name::enter(":wat::bracket::-type-slot-name").as_ref(),
            Name::from_keyword(":wat::bracket::-type-slot-name").as_ref()
        );
        assert!(Name::from_keyword(":else").is_none());
        assert!(Name::from_keyword(":k").is_none());

        let scoped = bare.add_scope(fresh_scope());
        assert_ne!(scoped, bare);
        assert_eq!(scoped.pair(), bare.pair());

        let binder = Identifier::bare("foo/bar").into_bound();
        let body = Identifier::bare("foo/bar");
        assert_ne!(binder, body);
        assert!(binder.same_local(&body));
        assert!(body.same_local(&binder));
        assert_eq!(binder.local_spelling(), "foo/bar");
        assert_eq!(body.local_spelling(), "foo/bar");

        let mut set = std::collections::HashSet::new();
        set.insert(from_colons);
        set.insert(Name::from_keyword(":a::b::c").unwrap());
        assert_eq!(set.len(), 1, "equal pairs hash as one");
    }

    /// Stone 255.91 — the committed sample of the keyword→symbol zip.
    /// `KW_PAIRS_FULL` points at the full unique-pair list (three columns:
    /// count, keyword, symbol) and checks every row the same way.
    #[test]
    fn from_keyword_matches_bare_of_the_converted_symbol() {
        let sample = include_str!("../tests/fixtures/keyword-symbol-pairs.tsv");
        let n = assert_keyword_pairs(sample);
        assert!(n > 200, "sample shrank to {n}");
        if let Ok(path) = std::env::var("KW_PAIRS_FULL") {
            let full = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path}: {e}"));
            let full_n = assert_keyword_pairs(&full);
            eprintln!("KW_PAIRS_FULL {full_n}");
        }
    }

    fn assert_keyword_pairs(text: &str) -> usize {
        let mut n = 0;
        for (line_no, line) in text.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut cols = line.split('\t');
            let a = cols.next().unwrap_or("");
            let b = cols.next().unwrap_or("");
            let c = cols.next();
            let (kw, sym) = if let Some(sym) = c { (b, sym) } else { (a, b) };
            let got = Name::from_keyword(kw);
            let expect = Identifier::bare(sym);
            assert_eq!(
                got.as_ref(),
                Some(expect.pair()),
                "line {} keyword {kw:?} symbol {sym:?}",
                line_no + 1
            );
            n += 1;
        }
        n
    }

    /// ⛔ NON-VACUITY CONTROL for the ruling. Single-slash spellings are
    /// every real identifier in this tree, and the two rules AGREE on all of
    /// them — so the test above could pass while the ordinary case silently
    /// broke. These rows must be unchanged by the flip; revert `bare` to
    /// `rfind` and the test above goes red while THIS one stays green, which
    /// is exactly the discrimination it exists to provide.
    #[test]
    fn the_ruling_leaves_every_single_slash_spelling_untouched() {
        for (spelling, ns, name) in [
            ("wat.core/+", "wat.core", "+"),
            ("wat.map/get", "wat.map", "get"),
            (":S/mk", ":S", "mk"),
            (":sort'/apply", ":sort'", "apply"),
            ("$bound/foo", "$bound", "foo"),
        ] {
            let id = Identifier::bare(spelling);
            assert_eq!(id.namespace(), ns, "namespace {spelling}");
            assert_eq!(id.method(), name, "name {spelling}");
            assert_eq!(id.receiver(), ns, "receiver {spelling}");
            assert_eq!(id.method(), name, "method {spelling}");
        }
        // And a name with NO slash is still bound, under either rule.
        let id = Identifier::bare("foo");
        assert_eq!(id.namespace(), BOUND_NAMESPACE);
        assert_eq!(id.method(), "foo");
        assert!(!id.is_reference());
    }

    #[test]
    fn identifier_accessors_match_the_free_functions_on_todays_inputs() {
        let spellings = [
            "foo",
            "wat.core/+",
            "wat.core//",
            ":S/mk",
            ":wat::cache::Lru",
            ":sort'",
            ":sort'/apply",
            "x",
            "$bound/foo",
        ];
        for spelling in spellings {
            let id = Identifier::bare(spelling);
            assert_eq!(id.as_str(), spelling, "as_str {spelling}");
            // ⛔ Derived from the RULE (first `/`), deliberately NOT from
            // `Identifier::bare`'s own code path — this row exists to catch
            // an accessor drifting from the rule, so it must not recompute
            // whatever the accessor happens to do.
            let expected_ns = match spelling.find('/') {
                Some(i) => &spelling[..i],
                None => BOUND_NAMESPACE,
            };
            assert_eq!(id.namespace(), expected_ns, "namespace {spelling}");
            assert_eq!(id.receiver(), receiver(spelling), "receiver {spelling}");
            assert_eq!(id.method(), method(spelling), "method {spelling}");
            assert_eq!(id.leaf(), leaf(spelling), "leaf {spelling}");
            assert_eq!(id.path(), path(spelling), "path {spelling}");
            assert_eq!(id.deprimed(), deprimed(spelling), "deprimed {spelling}");
            assert_eq!(id.prime(), prime(spelling), "prime {spelling}");
            assert_eq!(
                id.is_reference(),
                expected_ns != BOUND_NAMESPACE,
                "is_reference {spelling}"
            );
        }
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

    // ── STONE-variant-composition-door: the two composers ───────────────

    #[test]
    fn compose_variant_joins_with_double_colon_verbatim() {
        // As-stored EnumValue.type_path (colon kept) — the dominant shape at
        // twelve of the thirteen `::` call sites.
        assert_eq!(
            compose_variant(":trading::types::PhaseLabel", "Valley"),
            ":trading::types::PhaseLabel.Valley"
        );
        // A caller-pre-stripped path (runtime.rs:4108's own trim) — the door
        // does not care; it is colon-agnostic, not colon-normalizing.
        assert_eq!(
            compose_variant("trading::types::PhaseLabel", "Valley"),
            "trading::types::PhaseLabel.Valley"
        );
    }

    #[test]
    fn compose_variant_is_the_inverse_of_decompose_variant() {
        // NOT path()/leaf() — those are the GENERAL `::` splitters, and compose_variant's
        // separator is `.` (the dot flip). decompose_variant is the pair's own dedicated
        // inverse; see the doc comment on both functions above.
        let composed = compose_variant(":wat::cache::Lru", "Hit");
        let (path, leaf) = decompose_variant(&composed).expect("a composed variant decomposes");
        assert_eq!(path, ":wat::cache::Lru");
        assert_eq!(leaf, "Hit");
    }

    #[test]
    fn compose_variant_render_joins_with_a_dot_from_a_leaf() {
        assert_eq!(compose_variant_render("Box", "Full"), "Box.Full");
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
