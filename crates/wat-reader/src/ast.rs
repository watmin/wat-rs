//! `WatAST` — the language-surface AST the parser produces.
//!
//! Distinct from `wat::holon::HolonAST`. `WatAST` represents everything the
//! s-expression grammar admits at parse time: literals, keyword-path
//! tokens, bare symbols, parenthesized forms. Classification into higher
//! forms (`Define`, `Fn`, `Struct`, `UpperCall`, macro invocations,
//! …) happens at later passes (macro-expansion, name-resolution,
//! lowering) dispatching on the head of a `List` whose first element is
//! a `Keyword`.
//!
//! Standard Lisp parser discipline: parse to a uniform tree; interpret
//! structure at semantic passes, not at lex/parse time.
//!
//! # Hygiene
//!
//! `Symbol` carries an [`Identifier`](crate::identifier::Identifier) —
//! a (name, scope-set) pair that lets lexical-scope lookups distinguish
//! `tmp` the user wrote from `tmp` a macro introduced. Fresh-parsed
//! symbols have empty scope sets; macro expansion (slice 5c) adds
//! scopes per Racket's sets-of-scopes model. Keywords (full paths)
//! carry no scope tracking — hygiene only matters for bare names.

use std::borrow::Cow;

use crate::identifier::Identifier;
use crate::span::Span;
use num_bigint::BigInt;
use num_rational::BigRational;

/// Arc 257 slice 2 — discriminant for `MapDestructure`.
///
/// `Keys` → `{:keys [x y z]}` (keys-destructure; binding_name == field_name).
/// `Hash` → `{var :field var2 :field2 ...}` (hash-destructure; var ← field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapDestructureKind {
    Keys,
    Hash,
}

/// Arc 257 slice 2 — result of `WatAST::classify_map_destructure`.
///
/// `bindings` is a flat list of `(binding_identifier, field_name_bare, binding_span)`:
/// - For `Keys`: `binding_identifier.as_str() == field_name_bare` (same name).
/// - For `Hash`: `binding_identifier` is the user var; `field_name_bare` is the
///   keyword stripped of its leading `:` (e.g. `:magnitude` → `"magnitude"`).
#[derive(Debug, Clone)]
pub struct MapDestructure {
    pub kind: MapDestructureKind,
    pub bindings: Vec<(Identifier, String, Span)>,
}

/// The parsed source tree. One variant per terminal kind plus a `List`
/// variant for any parenthesized form.
///
/// Every variant carries a trailing [`Span`] with the source location
/// the node was parsed from. Span comparison is structural-transparent
/// (see [`crate::span`] module docs) — two nodes with the same
/// structure but different spans compare equal and hash identically.
#[derive(Debug, Clone, PartialEq)]
pub enum WatAST {
    /// Integer literal, as in `42`, `-1`, `0`. Fits in `i64`.
    IntLit(i64, Span),

    /// Floating-point literal, as in `3.14`, `-0.5`, `1e10`.
    FloatLit(f64, Span),

    /// Rational literal, as in `1/2`, `-3/2` (arc 300 stone B). Numeric-
    /// literal lane (follows `IntLit`/`FloatLit`), NOT a desugared
    /// constructor call — see `DESIGN-STONE-rational-B-runtime.md`. Always
    /// a genuine ratio already reduced to lowest terms with the sign on
    /// the numerator and denominator >= 2; a literal reducing to a whole
    /// number (`4/2`) parses to `IntLit` instead, never this variant.
    RationalLit(BigRational, Span),

    /// Arbitrary-precision integer literal, as in `1N`, `-5N`,
    /// `9223372036854775807N` (arc 300 stone C1). Numeric-literal lane
    /// (follows `IntLit`/`FloatLit`/`RationalLit`), NOT a desugared
    /// constructor call. Unlike `RationalLit`'s den==1 reduction to
    /// `IntLit`, this NEVER reduces — `1N` is always bigint regardless of
    /// magnitude (clj: `(class 1N)` is `clojure.lang.BigInt`).
    BigIntLit(BigInt, Span),

    /// Character literal, as in `\a`, `\newline`, `A` (arc 300 stone D).
    /// Scalar-literal lane, NOT a desugared constructor call (arc 244 /
    /// arc 300 B / C1 precedent) — the parser previously desugared `\c`
    /// into `(:wat::core::char/of "c")`, a three-node call where a
    /// one-node literal belonged. The lexer resolves named
    /// (`\newline`/`\space`/`\tab`) and unicode forms to a `char` before
    /// this point; the parser sees only the resolved value. The
    /// `:wat::core::char` verb keeps working as a real runtime
    /// String→char conversion; it simply stops being the reader's parse
    /// target.
    CharLit(char, Span),

    /// Boolean literal, as in `true` or `false`.
    BoolLit(bool, Span),

    /// String literal, as in `"hello"` — quotes stripped, escape sequences
    /// applied.
    StringLit(String, Span),

    /// Nil literal, as in bare `nil` — the unit / absent value.
    ///
    /// Arc 244 — nil joins `int / float / bool / string` as a first-class
    /// literal variant; the asymmetry (nil-as-Symbol while every other
    /// scalar is a *Lit variant) is annihilated. The parser produces
    /// `NilLit(span)` for bare `nil`. Synthesized nil values use the
    /// `WatAST::nil()` constructor.
    NilLit(Span),

    /// Keyword token, as in `:foo`, `:wat::holon::Atom`,
    /// `:wat::holon::Holons`, `:fn(T,U)->R`. The leading `:` is part of the
    /// stored string. Used both as keyword literals (payloads for wat
    /// keyword atoms) and as keyword-path references (heads of calls,
    /// type annotations). Distinguished by context at later passes.
    ///
    /// Keywords carry no scope tracking — their full-path spelling
    /// already disambiguates `:my::app::foo` from `:my::macro::foo`.
    Keyword(String, Span),

    /// Bare identifier, as in `x`, `role`, `tmp`. Used in `let` bindings,
    /// `fn` parameter names, `match` patterns — the only places the
    /// language admits bare names. The `Identifier` carries a scope
    /// set for macro hygiene (empty on fresh parse).
    Symbol(Identifier, Span),

    /// Parenthesized form `(head arg1 arg2 ...)`. Also covers
    /// empty list `()`. The first child is typically the head —
    /// a `Keyword` for language or algebra calls, a `Symbol` for
    /// bare-scoped fn/let invocation.
    List(Vec<WatAST>, Span),

    /// Bracketed form `[a b c ...]`. Also covers empty vector
    /// `[]`. Distinct from `List` at the AST level so consumers
    /// (slice 2's fn / defn signature parser; slice ≥3's let
    /// binding-block parser) can syntactically distinguish a
    /// vector from a list.
    ///
    /// Arc 167 slice 1 (additive substrate). Vectors are admitted
    /// only in **binding-syntax positions**; appearing at value
    /// position errors at eval/check time. The legal-position
    /// consumers are wired in slice 2 (`fn` / `defn` signatures)
    /// and arc 168 (`let`).
    Vector(Vec<WatAST>, Span),

    /// Braced form `{a b c ...}` — the struct-destructure binder
    /// shape. Each child is a bare `Symbol` that is BOTH the
    /// field-name (looked up against the struct type of the
    /// adjacent expression) AND the local binding-name in the
    /// Map literal `{k0 v0 k1 v1 ...}` — a first-class key/value
    /// collection node. Pairs are stored as `(key, value)` tuples so
    /// odd arity is unrepresentable by construction.
    ///
    /// Arc 257 slice 1 (additive substrate). In value position this
    /// evaluates to `Value::wat__std__HashMap` (same as the explicit
    /// `(:wat::core::HashMap :K :V k v ...)` constructor, but without
    /// the leading type-keyword sentinels). In binder/pattern position
    /// (after arc 257.3) it becomes a map-destructure.
    ///
    /// Odd-arity body → parse error before this node is ever produced.
    Map(Vec<(WatAST, WatAST)>, Span),

    /// Set literal `#{x y z ...}` — a first-class unordered collection
    /// node. Elements are stored as a flat `Vec<WatAST>`; duplicates
    /// collapse at eval time (HashSet semantics).
    ///
    /// Arc 257 slice 1 (additive substrate). Evaluates to
    /// `Value::wat__std__HashSet` (same as the explicit
    /// `(:wat::core::HashSet :T x y z)` constructor form).
    Set(Vec<WatAST>, Span),
}

impl WatAST {
    /// Borrow the span this node was parsed from.
    pub fn span(&self) -> &Span {
        match self {
            WatAST::IntLit(_, s)
            | WatAST::FloatLit(_, s)
            | WatAST::RationalLit(_, s)
            | WatAST::BigIntLit(_, s)
            | WatAST::CharLit(_, s)
            | WatAST::BoolLit(_, s)
            | WatAST::StringLit(_, s)
            | WatAST::NilLit(s)
            | WatAST::Keyword(_, s)
            | WatAST::Symbol(_, s)
            | WatAST::List(_, s)
            | WatAST::Vector(_, s)
            | WatAST::Map(_, s)
            | WatAST::Set(_, s) => s,
        }
    }

    /// Convenience constructors using `rust_caller_span!()` — for
    /// synthetic forms / tests / runtime-constructed ASTs.
    /// Arc 298.2: every span is a real Rust caller location.
    pub fn int(n: i64) -> Self {
        WatAST::IntLit(n, crate::rust_caller_span!())
    }
    pub fn float(x: f64) -> Self {
        WatAST::FloatLit(x, crate::rust_caller_span!())
    }
    /// Synthetic rational literal — `r` must already be reduced (den >= 2,
    /// sign on numerator); see `RationalLit`'s doc. Arc 300 stone B.
    pub fn rational(r: BigRational) -> Self {
        WatAST::RationalLit(r, crate::rust_caller_span!())
    }
    /// Synthetic bigint literal. Arc 300 stone C1.
    pub fn bigint(n: BigInt) -> Self {
        WatAST::BigIntLit(n, crate::rust_caller_span!())
    }
    /// Synthetic char literal. Arc 300 stone D.
    pub fn char(c: char) -> Self {
        WatAST::CharLit(c, crate::rust_caller_span!())
    }
    pub fn bool(b: bool) -> Self {
        WatAST::BoolLit(b, crate::rust_caller_span!())
    }
    pub fn string(s: impl Into<String>) -> Self {
        WatAST::StringLit(s.into(), crate::rust_caller_span!())
    }
    /// Synthetic nil literal with `rust_caller_span!()` — the canonical
    /// constructor for synthesized nil values. Arc 244: nil joins the
    /// int/float/bool/string value-constructor family; use this (not
    /// `Keyword` with the nil type path) in all synthesis paths.
    pub fn nil() -> Self {
        WatAST::NilLit(crate::rust_caller_span!())
    }
    pub fn keyword(k: impl Into<String>) -> Self {
        WatAST::Keyword(k.into(), crate::rust_caller_span!())
    }
    pub fn symbol(ident: Identifier) -> Self {
        WatAST::Symbol(ident, crate::rust_caller_span!())
    }
    pub fn list(items: Vec<WatAST>) -> Self {
        WatAST::List(items, crate::rust_caller_span!())
    }
    /// Synthetic Vector with `rust_caller_span!()` — for tests and
    /// runtime-constructed bracketed forms.
    pub fn vector(items: Vec<WatAST>) -> Self {
        WatAST::Vector(items, crate::rust_caller_span!())
    }
    /// Synthetic Map literal with `rust_caller_span!()` — for tests
    /// and runtime-constructed map forms. Arc 257 slice 1.
    pub fn map(pairs: Vec<(WatAST, WatAST)>) -> Self {
        WatAST::Map(pairs, crate::rust_caller_span!())
    }
    /// Synthetic Set literal with `rust_caller_span!()` — for tests
    /// and runtime-constructed set forms. Arc 257 slice 1.
    pub fn set(items: Vec<WatAST>) -> Self {
        WatAST::Set(items, crate::rust_caller_span!())
    }

    /// Returns true if this is a bare `Symbol` whose name equals `name`.
    /// Used to detect structural tokens (`<-`, `->`, `&`) without allocating.
    pub fn is_bare_symbol(&self, name: &str) -> bool {
        matches!(self, WatAST::Symbol(ident, _) if ident.as_str() == name)
    }

    /// Arc 257 slice 1 — ONE authoritative metadata-map discriminant.
    ///
    /// Returns `Some(pairs)` when `self` is a metadata-map literal in either
    /// of its two forms:
    ///
    /// - **`WatAST::Map(pairs, _)`** — the new native form produced by the
    ///   parser after arc 257.1 (e.g. `{:tag "foo"}`).
    /// - **`WatAST::List`** with head `:wat::core::HashMap` — the legacy
    ///   constructor-call form produced by the OLD parser (pre-arc-257) and
    ///   still emitted by `closure_extract::encode_value_with_path` for
    ///   runtime re-encoding of captured HashMaps (where types are known).
    ///
    /// Returns `None` for anything else (keyword, symbol, vector, set, etc.).
    ///
    /// Called at all 8 metadata-sniff sites (parser.rs, check.rs,
    /// runtime.rs, types.rs, closure_extract.rs, function/metadata.rs,
    /// types/defstruct.rs) so every site stays in sync when the
    /// `is_metadata_map` contract changes.
    ///
    /// PARTITION — CLAUSE vs INTRINSIC: this is a structural predicate on an
    /// AST node (no type-var flow, monomorphic args). It lives here at the
    /// substrate layer so the source is the single authoritative home.
    pub fn is_metadata_map(&self) -> bool {
        match self {
            WatAST::Map(_, _) => true,
            WatAST::List(items, _) => {
                matches!(
                    items.first(),
                    Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap"
                )
            }
            _ => false,
        }
    }

    /// Arc 257 slice 1 — extract key/value pairs from a metadata-map node.
    ///
    /// For `WatAST::Map(pairs, _)` returns the pairs directly.
    /// For a legacy `WatAST::List` with `:wat::core::HashMap` head, rebuilds
    /// pairs from the flat `[head, K-type, V-type, k0, v0, …]` layout
    /// (allocates). Returns `None` if `self` is not a metadata-map.
    ///
    /// Callers that only need the bool discriminant should use `is_metadata_map`.
    pub fn metadata_map_pairs(&self) -> Option<Vec<(WatAST, WatAST)>> {
        match self {
            WatAST::Map(pairs, _) => Some(pairs.clone()),
            WatAST::List(items, _) => {
                match items.first() {
                    Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap" => {}
                    _ => return None,
                }
                // Legacy layout: [head, K-type, V-type, k0, v0, k1, v1, ...]
                if items.len() < 3 {
                    return None;
                }
                let pairs_flat = &items[3..];
                if !pairs_flat.len().is_multiple_of(2) {
                    return None;
                }
                let mut pairs: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs_flat.len() / 2);
                let mut i = 0;
                while i < pairs_flat.len() {
                    pairs.push((pairs_flat[i].clone(), pairs_flat[i + 1].clone()));
                    i += 2;
                }
                Some(pairs)
            }
            _ => None,
        }
    }

    /// Arc 257 slice 2 — classify a Map node in binder/pattern position.
    ///
    /// Returns `Some(MapDestructure)` when `pairs` encodes a valid destructure:
    ///
    /// - **keys-destructure**: exactly one pair `(Keyword(":keys"), Vector([Symbol, ...]))`.
    ///   Each symbol in the vector binds the same-named field
    ///   (binding_name == field_name). Field-name normalization: bare symbol
    ///   name (no leading `:`).
    ///
    /// - **hash-destructure**: every pair is `(Symbol(var), Keyword(":field"))`.
    ///   `var` binds field `:field`; field_name = keyword stripped of leading `:`.
    ///
    /// Returns `None` for anything else (value-position map literal, malformed
    /// binder, empty map). The caller is responsible for raising an appropriate
    /// error when `None` is returned in a binder context.
    ///
    /// PARTITION — CLAUSE vs INTRINSIC: this is a structural predicate on a
    /// concrete slice of pairs (monomorphic args, no type-var flow). Lives at
    /// the substrate layer so check, runtime, AND closure_extract share ONE
    /// authoritative home.
    pub fn classify_map_destructure(pairs: &[(WatAST, WatAST)]) -> Option<MapDestructure> {
        if pairs.is_empty() {
            return None;
        }
        // Keys-destructure: exactly one pair (Keyword(":keys"), Vector([Symbol, ...]))
        if pairs.len() == 1 {
            if let (WatAST::Keyword(k, _), WatAST::Vector(syms, sym_span)) = &pairs[0] {
                if k == ":keys" {
                    if syms.is_empty() {
                        return None; // empty :keys vector → not a destructure
                    }
                    let mut bindings = Vec::with_capacity(syms.len());
                    for sym in syms {
                        if let WatAST::Symbol(ident, sp) = sym {
                            let name = ident.as_str().to_owned();
                            bindings.push((ident.clone(), name.clone(), sp.clone()));
                        } else {
                            return None; // non-Symbol inside :keys vector
                        }
                    }
                    let _ = sym_span; // suppress unused-var
                    return Some(MapDestructure {
                        kind: MapDestructureKind::Keys,
                        bindings,
                    });
                }
            }
        }
        // Hash-destructure: every pair is (Symbol(var), Keyword(":field"))
        let mut bindings = Vec::with_capacity(pairs.len());
        for (k, v) in pairs {
            match (k, v) {
                (WatAST::Symbol(ident, sp), WatAST::Keyword(field_kw, _)) => {
                    let field_name = field_kw.trim_start_matches(':').to_string();
                    bindings.push((ident.clone(), field_name, sp.clone()));
                }
                _ => return None, // not a hash-destructure pair
            }
        }
        Some(MapDestructure {
            kind: MapDestructureKind::Hash,
            bindings,
        })
    }

    /// The children of this AST node. Compound shapes return their
    /// `items`; leaves return an empty slice.
    ///
    /// Arc 212 (failure engineering applied at the walker layer).
    /// Walkers that recurse generically through the AST MUST use
    /// this method rather than pattern-matching on `WatAST::List`
    /// specifically. The motivation:
    ///
    /// - When `WatAST::Vector` was added in arc 167 slice 1, the
    ///   macro expand-time walker (`macros.rs::walk_template`) was
    ///   correctly updated to recurse into Vector children. The
    ///   runtime quasiquote walker (`runtime.rs::walk_quasiquote`)
    ///   was NOT updated; it skipped Vectors silently. Latent
    ///   substrate flaw for ~50 arcs; surfaced via t6 in arc 212.
    ///
    /// - The fix was tempting to ship per-walker (add a Vector arm
    ///   to each function that was missing one). But that produces
    ///   N copies of the same logic and N opportunities for the
    ///   same bug when the next AST variant lands. The honest fix
    ///   is at the substrate layer: own "what are the children of
    ///   an AST node?" here, so walkers can't get it wrong.
    ///
    /// When a NEW compound `WatAST` variant lands, update this
    /// method's match arm to include it. Every walker that descends
    /// via `children()` automatically benefits without per-walker
    /// audit.
    ///
    /// **The bug class is structurally eliminated** — failure
    /// engineering at the walker layer.
    /// Arc 257 slice 1 — returns `Cow::Borrowed` for flat-children variants
    /// (List, Vector, Set) and `Cow::Owned` for `Map` (pairs flattened to
    /// `[k0, v0, k1, v1, …]`). Leaf nodes return `Cow::Borrowed(&[])`.
    ///
    /// **Callers must use `.iter()` on the returned `Cow` to iterate**, e.g.:
    /// `for child in node.children().iter() { … }`.
    ///
    /// `Cow::Borrowed` variants are zero-cost (no allocation). `Map` allocates
    /// once to produce the flattened slice; this is acceptable for the
    /// generic-walk paths.
    pub fn children(&self) -> Cow<'_, [WatAST]> {
        match self {
            WatAST::List(items, _)
            | WatAST::Vector(items, _)
            | WatAST::Set(items, _) => Cow::Borrowed(items.as_slice()),
            WatAST::Map(pairs, _) => {
                let mut flat: Vec<WatAST> = Vec::with_capacity(pairs.len() * 2);
                for (k, v) in pairs {
                    flat.push(k.clone());
                    flat.push(v.clone());
                }
                Cow::Owned(flat)
            }
            _ => Cow::Borrowed(&[]),
        }
    }

    /// Canonical bare-word name for this AST variant — "int", "float",
    /// "bool", "string", "keyword", "symbol", "list", "vector",
    /// "map", "set". Used in type-error messages across check.rs,
    /// types.rs, and runtime.rs; one authoritative site so all paths
    /// emit the same label for the same node kind.
    pub fn variant_name(&self) -> &'static str {
        match self {
            WatAST::IntLit(_, _) => "int",
            WatAST::FloatLit(_, _) => "float",
            WatAST::RationalLit(_, _) => "rational",
            WatAST::BigIntLit(_, _) => "bigint",
            WatAST::CharLit(_, _) => "char",
            WatAST::BoolLit(_, _) => "bool",
            WatAST::StringLit(_, _) => "string",
            WatAST::NilLit(_) => "nil",
            WatAST::Keyword(_, _) => "keyword",
            WatAST::Symbol(_, _) => "symbol",
            WatAST::List(_, _) => "list",
            WatAST::Vector(_, _) => "vector",
            WatAST::Map(_, _) => "map",
            WatAST::Set(_, _) => "set",
        }
    }
}

// wat_ast_to_source / wat_ast_program_to_source — RETIRED in arc
// 012 slice 3 (the task-#269 commit). Added in arc 011 to bridge
// the AST → source → subprocess boundary of the old hermetic-ast
// primitive. With spawn-process, the child inherits AST in
// memory via COW — no textual round-trip, no serializer needed.
//
// REINTRODUCED in arc 278 Stone 1 as `:wat::core::ast->source`
// (src/edn/render.rs, `write_wat_source` beside `eval_write_forms`)
// — the sift Predicate's enabling primitive. Lives in edn::render
// rather than here because it's a runtime primitive (dispatched
// from runtime.rs, typed in check.rs), not a wat-reader-internal
// concern; the shape mirrors this retired printer plus the 6
// variants born since (RationalLit, BigIntLit, NilLit, Vector,
// Map, Set).

/// Arc 216 Stone 216.5a — `impl Hash for WatAST`.
///
/// Decision D1: direct structural impl mirroring `HolonAST`'s pattern in
/// holon-rs (`src/kernel/holon_ast.rs:196-232`). WatAST derives `PartialEq`
/// which is span-agnostic (Span's PartialEq is a no-op that always returns
/// true — see `span.rs`). The Hash impl mirrors that: `Span` contributes
/// nothing (its Hash is a no-op, `span.rs:128`). Discriminant tagging via
/// `std::mem::discriminant` prevents cross-variant collisions.
///
/// `FloatLit(f64, Span)` uses `f64::to_bits()` (NaN-safe; mirrors
/// `HolonAST::F64` arm). All other variants compose cleanly via std
/// lib's `Hash` impls on `i64`, `bool`, `String`, `Identifier`, and
/// `Vec<WatAST>` (recursive).
///
/// WHY D1 over D2 (Debug-string DefaultHasher): D1 is structurally honest.
/// Span already has a no-op Hash; Identifier already derives Hash; the
/// recursive Vec<WatAST> is straightforward. D2 would be a workaround
/// for a gap that doesn't actually exist — WatAST's fields all implement
/// Hash once f64 is handled via `to_bits()`.
impl std::hash::Hash for WatAST {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            WatAST::IntLit(n, _) => n.hash(state),
            WatAST::FloatLit(x, _) => x.to_bits().hash(state),
            WatAST::RationalLit(r, _) => r.hash(state),
            WatAST::BigIntLit(n, _) => n.hash(state),
            WatAST::CharLit(c, _) => c.hash(state),
            WatAST::BoolLit(b, _) => b.hash(state),
            WatAST::StringLit(s, _) => s.hash(state),
            // NilLit: leaf literal — discriminant (above) fully identifies it.
            WatAST::NilLit(_) => {}
            WatAST::Keyword(k, _) => k.hash(state),
            WatAST::Symbol(ident, _) => ident.hash(state),
            WatAST::List(items, _) => items.hash(state),
            WatAST::Vector(items, _) => items.hash(state),
            WatAST::Map(pairs, _) => pairs.hash(state),
            WatAST::Set(items, _) => items.hash(state),
        }
        // Span: no-op — Span's Hash impl contributes nothing (span.rs:128).
    }
}
