//! `wat_edn_bridge` — plain-EDN serializer/deserializer for `WatAST`.
//!
//! Converts a `Vec<WatAST>` program to a single plain-EDN frame and back
//! so the program can be catted over a wire fd as the EDN it already is.
//! No holon tags, no VSA encoding — a near-1:1 structural map between
//! `WatAST` (post-arc-257: List/Vector/Map/Set all native) and
//! `wat_edn::Value` / `OwnedValue`.
//!
//! ## Keyword codec
//!
//! Wat uses `::` as the segment separator (`:wat::core::foo`); EDN uses `/`
//! to split namespace from name (`:wat.core/foo`). This module REUSES the
//! proven codec already in `edn_shim`:
//! - Encode: `keyword_from_wat_path` — splits on last `::`, builds via
//!   `Keyword::try_ns(ns, name)`.
//! - Decode: `ns_to_wat_path` — rebuilds `:ns.sub/name` → `:ns::sub::name`.
//!
//! Do NOT hand-roll the `::` ↔ `.` / `/` translation.
//!
//! ## Arc 213 / BRIEF-213-SERIALIZER-BRIDGE context
//!
//! This bridge is the serializer corrected by arc 257's SUPERSEDED note:
//! the old `watast_to_holon` path encoded every node under a tagged-HolonAST
//! wire family (the VSA hologram — the same family arc 294.j later kills
//! outright on the HolonAST encode side too), which is the contract-vs-encoding
//! abuse, NOT EDN transport. Plain EDN is the correct wire format; arc 257's
//! native Map/Set nodes make the mapping 1:1.

use crate::ast::WatAST;
use crate::edn::render::{keyword_from_wat_path, ns_to_wat_path};
use crate::scope::{fresh_scope, Identifier, ScopeId};
use crate::span::{Pos, Span};
use std::collections::HashMap;
use std::sync::Arc;
use wat_edn::{Keyword, OwnedValue, Symbol, Tag};

// ─── The scoped-symbol tag ──────────────────────────────────────

/// Tag namespace/name for a symbol carrying hygiene scopes:
/// `#wat.ast/ScopedSymbol {:name "kwargs" :scopes [952]}`.
///
/// A BARE symbol (empty scope set — everything the parser emits, and all
/// hand-written code) stays a plain EDN `Symbol`, so every form that
/// round-trips today keeps its exact byte spelling. Only a symbol a MACRO
/// minted takes the tagged form, because only those carry scopes.
///
/// # The body is a RECORD, not a tuple
///
/// A `["kwargs" [952]]` pair would be a vector of non-uniform types, which is
/// exactly the shape this arc retired: `DESIGN-dynamic-edn-decode-and-opaque-sink.md`
/// pins **record → `{field-map}` (map); enum variant → `[field-vec]` (vector)**
/// as the one rule, so body-shape stays a perfect discriminator. A two-element
/// heterogeneous vector reads as a variant with two positional fields and is a
/// lie about what this is. The named fields also make the frame legible to a
/// human reading a wire dump, and make adding a field later a non-breaking act
/// rather than a positional re-cut.
///
/// Named `ScopedSymbol`, not `Scope`: the body carries the NAME as well, so the
/// thing on the wire is the whole symbol. A scope never exists on its own —
/// there is no free-floating `Scope` in the substrate, only
/// `Identifier { name, scopes }`.
const SCOPED_SYM_NS: &str = "wat.ast";
const SCOPED_SYM_NAME: &str = "ScopedSymbol";
/// `:name` — the identifier's bare name.
const FIELD_NAME: &str = "name";
/// `:scopes` — the hygiene scope ids, ascending (`BTreeSet` order is canonical).
const FIELD_SCOPES: &str = "scopes";

/// Tag namespace/name for a wat keyword EDN cannot spell:
/// `#wat.ast/Keyword {:path ":wat::core::Vector/length"}`.
///
/// # Why this exists
///
/// A wat keyword is not an EDN keyword. EDN's is a flat `ns/name`; wat's is a
/// small type/path language — `::` segments, `Type/method` accessors, `<>`
/// generics, `(A,B)` tuple types, `Fn(A)->B` function types, and trailing-`::`
/// namespace-prefix markers. Forcing one into the other does not merely lose a
/// value, it changes the FORM'S SHAPE: `:wat::core::Fn(wat::core::i64)->wat::core::i64`
/// is ONE keyword token going in and comes back as a keyword, a list, and a
/// symbol — three nodes. (`Keyword::try_ns` validates only the first character,
/// so the malformed namespace is accepted on the way out.)
///
/// So a keyword that cannot survive the crossing is carried VERBATIM — exactly
/// what a `.wat` source file does, and the same move [`SCOPED_SYM_NAME`] makes
/// for a symbol EDN cannot spell. The `::`↔`.` dial is not involved on this
/// path at all.
///
/// # Temporary, and bounded
///
/// This wrapper exists only because there are TWO readers — `wat-reader`'s
/// keyword grammar is wider than `wat-edn`'s. Arc 300 converts the corpus to
/// faithful-Clojure and RETIRES the rust-scheme surface (`VNVS LECTOR NE
/// DIVIDANTVR`); once nothing can produce a keyword EDN cannot spell, the
/// [`needs_verbatim_carriage`] test stops firing on its own and this tag can be
/// deleted. It is not scaffolding to be remembered — it is self-disarming.
const WAT_KEYWORD_NAME: &str = "Keyword";
/// `:path` — the wat keyword path, verbatim, leading colon included.
const FIELD_PATH: &str = "path";

/// A wat keyword carried verbatim: `#wat.ast/Keyword {:path "…"}`.
///
/// `pub(crate)` because `edn_shim::keyword_from_wat_path` reaches for it as its
/// honest last resort. It used to fall back to a bare `OwnedValue::String`
/// there — a SILENT TYPE CHANGE: a keyword went in, a string came out, and
/// nothing said so. That is defensible for the logger it was written for and
/// indefensible anywhere a value is read back. This is the same shape, without
/// the lie: still one value, still non-panicking, and it decodes to the keyword
/// it started as.
pub(crate) fn verbatim_keyword(path: &str) -> OwnedValue {
    OwnedValue::Tagged(
        Tag::ns(SCOPED_SYM_NS, WAT_KEYWORD_NAME),
        Box::new(OwnedValue::Map(vec![(
            OwnedValue::Keyword(Keyword::new(FIELD_PATH)),
            OwnedValue::String(std::borrow::Cow::Owned(path.to_owned())),
        )])),
    )
}

/// Does this keyword survive the crossing as a plain EDN keyword?
///
/// Answered by a RUN, never by a grammar predicate: encode the candidate, WRITE
/// it, READ it back, and decode — if what returns is not the identical path,
/// the keyword needs verbatim carriage. A hand-written "is it legal EDN?" check
/// would be a second grammar living beside `wat-edn`'s, free to drift from it;
/// this cannot drift, because it *is* the round trip it is asking about, and it
/// covers cases nobody enumerated (the arity bug above was found this way, not
/// by reading the grammar).
fn needs_verbatim_carriage(path: &str) -> bool {
    let candidate = keyword_from_wat_path(path);
    // The codec already declined — it fell back to a String (a silent type
    // change on a program wire; here it is a clean "carry it verbatim").
    let OwnedValue::Keyword(_) = &candidate else {
        return true;
    };
    let written = wat_edn::write(&candidate);
    match wat_edn::parse_owned(&written) {
        Ok(OwnedValue::Keyword(kw)) => {
            let back = match kw.namespace() {
                Some(ns) => ns_to_wat_path(ns, kw.name()),
                None => format!(":{}", kw.name()),
            };
            back != path
        }
        // Re-read as some other shape (the arity bug), or did not parse at all.
        _ => true,
    }
}

/// Does this symbol NAME survive the crossing as a plain EDN symbol?
///
/// Same discipline as [`needs_verbatim_carriage`], and it catches the same
/// class one type over: a wat symbol may be a generic method head like
/// `mk<S,R>`, which is ONE token to `wat-reader` — but **EDN treats `,` as
/// whitespace**, so it re-reads as two symbols (`mk<S`, `R>`) and the form's
/// arity changes. Answered by a run for the same reason: a hand-written
/// "which characters are safe?" predicate would be a second lexer beside
/// `wat-edn`'s, free to drift.
fn symbol_needs_verbatim(name: &str) -> bool {
    let written = wat_edn::write(&OwnedValue::Symbol(Symbol::new(name)));
    match wat_edn::parse_owned(&written) {
        Ok(OwnedValue::Symbol(sym)) => {
            let back = match sym.namespace() {
                Some(ns) => format!("{ns}/{}", sym.name()),
                None => sym.name().to_owned(),
            };
            back != name
        }
        _ => true,
    }
}


/// Decode-side scope table: maps each distinct scope id seen ON THE WIRE to a
/// scope freshly allocated IN THIS PROCESS.
///
/// # Why remap rather than import the wire's numbers
///
/// `ScopeId` has no public constructor from a `u64` — deliberately. Its whole
/// contract is that every id comes from [`fresh_scope`], a process-global
/// monotonic counter, so two distinct scopes are never confusable. A decoder
/// that minted `ScopeId(952)` from a wire integer would break exactly that:
/// the sender's 952 and a future `fresh_scope()` in THIS process would be
/// different scopes wearing one number — the variable capture hygiene exists
/// to prevent, reintroduced by the transport.
///
/// So the wire number is treated as what the type's own docs say it is —
/// *opaque, carrying no domain meaning*. What must survive is the STRUCTURE:
/// which identifiers share a scope, and which do not. Threading one table
/// across a whole program gives exactly that, with every resulting id fresh,
/// so a collision with the receiver's own scopes is unrepresentable rather
/// than unlikely. (`hash.rs`'s `ScopeRenumber` applies the same reasoning to
/// canonical hashing — "a RENUMBER, not a strip".)
#[derive(Default)]
struct ScopeImport {
    map: HashMap<i64, ScopeId>,
}

impl ScopeImport {
    /// The local scope for a wire id — the same wire id always yields the same
    /// local scope within one table, so sharing is preserved exactly.
    fn local(&mut self, wire_id: i64) -> ScopeId {
        *self.map.entry(wire_id).or_insert_with(fresh_scope)
    }
}

// ─── Span carriage (stone J, arc 296) ────────────────────────────

/// Tag namespace/name for a span-wrapped node under [`Carriage::Transport`]:
/// `#wat.ast/Spanned {:node <inner> :origin N :line N :col N [:end-line N :end-col N]}`.
///
/// EVERY node crosses wrapped this way under Transport — a delivered program
/// is EXECUTED, and execution produces diagnostics, so any node (not only call
/// forms) may end up as a `Fault`'s location. `Display` carriage never wraps:
/// it renders for a reader and nothing downstream re-parses it, so a span
/// would be pure noise there.
const SPANNED_NS: &str = "wat.ast";
const SPANNED_NAME: &str = "Spanned";
/// `:node` — the wrapped node's own plain-EDN encoding (recursively spanned
/// again for any children it has).
const FIELD_NODE: &str = "node";
/// `:origin` — index into the program-level `:origins` table (see
/// [`OriginTable`]); resolves to the span's `file`.
const FIELD_ORIGIN: &str = "origin";
const FIELD_LINE: &str = "line";
const FIELD_COL: &str = "col";
/// `:end-line` / `:end-col` — present together, or absent together (point-span,
/// e.g. a `rust_caller_span!()` origin with no known range end).
const FIELD_END_LINE: &str = "end-line";
const FIELD_END_COL: &str = "end-col";

/// Tag namespace/name for the whole-program frame:
/// `#wat.ast/Program {:origins [file0 file1 …] :forms [form0 form1 …]}`.
const PROGRAM_NS: &str = "wat.ast";
const PROGRAM_NAME: &str = "Program";
const FIELD_ORIGINS: &str = "origins";
const FIELD_FORMS: &str = "forms";

/// Encode-side file table: interns each distinct `Span::file` once so a
/// program with many nodes sharing one source file does not repeat that
/// string on every node — only its index does.
///
/// This is the measured choice over a naive per-node `#wat.core/Span
/// {:file "…" …}` (see the BRIEF's byte comparison): the file string is by
/// far the largest part of a `Span`, and it is overwhelmingly the SAME string
/// across a whole program's nodes.
#[derive(Default)]
struct OriginTable {
    files: Vec<String>,
    index: HashMap<String, i64>,
}

impl OriginTable {
    /// The table index for `file`, interning it on first sight.
    fn intern(&mut self, file: &str) -> i64 {
        if let Some(&i) = self.index.get(file) {
            return i;
        }
        let i = self.files.len() as i64;
        self.files.push(file.to_owned());
        self.index.insert(file.to_owned(), i);
        i
    }

    /// Drain the table to the `:origins` vector's EDN values, in the order
    /// each file was first interned (== the order the indices reference).
    fn into_values(self) -> Vec<OwnedValue> {
        self.files
            .into_iter()
            .map(|f| OwnedValue::String(std::borrow::Cow::Owned(f)))
            .collect()
    }
}

/// Wrap an already-encoded node's content with its span, under Transport
/// carriage. See [`SPANNED_NS`]/[`SPANNED_NAME`].
fn wrap_spanned(content: OwnedValue, span: &Span, origins: &mut OriginTable) -> OwnedValue {
    let o = origins.intern(&span.file);
    let mut fields = vec![
        (OwnedValue::Keyword(Keyword::new(FIELD_NODE)), content),
        (OwnedValue::Keyword(Keyword::new(FIELD_ORIGIN)), OwnedValue::Integer(o)),
        (OwnedValue::Keyword(Keyword::new(FIELD_LINE)), OwnedValue::Integer(span.line)),
        (OwnedValue::Keyword(Keyword::new(FIELD_COL)), OwnedValue::Integer(span.col)),
    ];
    if let Some(end) = &span.end {
        fields.push((
            OwnedValue::Keyword(Keyword::new(FIELD_END_LINE)),
            OwnedValue::Integer(end.line),
        ));
        fields.push((
            OwnedValue::Keyword(Keyword::new(FIELD_END_COL)),
            OwnedValue::Integer(end.col),
        ));
    }
    OwnedValue::Tagged(Tag::ns(SPANNED_NS, SPANNED_NAME), Box::new(OwnedValue::Map(fields)))
}

// ─── Error type ─────────────────────────────────────────────────

/// Error returned by the decode path (`edn_to_watast`, `edn_to_program`).
/// Never panics; callers receive a clean typed error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatEdnBridgeError {
    /// An EDN form shape that has no `WatAST` counterpart appeared in the
    /// encoded program frame (e.g. `Tagged`, `Inst`, `Uuid`,
    /// `BigDec`, namespaced `Symbol`). Arc 300 stone C1: `BigInt` now HAS a
    /// counterpart (`WatAST::BigIntLit`) and is no longer in this list.
    /// Arc 300 stone D: `Char` now HAS a counterpart (`WatAST::CharLit`)
    /// and is no longer in this list either.
    UnsupportedEdnForm { shape: String },
    /// A keyword in the EDN frame could not be decoded to a wat keyword path.
    KeywordDecode { raw: String },
    /// A `#wat.ast/ScopedSymbol` tagged literal whose body is not the required
    /// `{:name "…" :scopes [ids…]}` record. Named separately from
    /// `UnsupportedEdnForm` because it is a DIFFERENT failure: the form is one
    /// this bridge owns and expects, malformed — not one it has no counterpart
    /// for. A caller can tell "your frame is corrupt" from "this EDN is not a
    /// wat program" without parsing a string.
    MalformedScopedSymbol { detail: String },
    /// A `#wat.ast/Spanned` tagged literal whose body is not the required
    /// `{:node … :origin N :line N :col N}` record, or whose `:origin` index
    /// has no matching entry in the frame's `:origins` table. Same rationale
    /// as `MalformedScopedSymbol`: this bridge owns and expects this shape.
    MalformedSpanned { detail: String },
    /// The top-level frame failed to parse as EDN.
    ParseFrame { msg: String },
    /// The top-level frame is valid EDN but not a Vector (program must be wrapped
    /// in `[...]` by `program_to_edn`).
    ExpectedVector { got: String },
    /// The top-level frame is valid EDN but not the `#wat.ast/Program
    /// {:origins […] :forms […]}` record `program_to_edn` produces.
    ExpectedProgram { got: String },
}

impl std::fmt::Display for WatEdnBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatEdnBridgeError::UnsupportedEdnForm { shape } => {
                write!(f, "EDN form has no WatAST counterpart: {shape}")
            }
            WatEdnBridgeError::KeywordDecode { raw } => {
                write!(f, "keyword cannot be decoded to a wat path: {raw}")
            }
            WatEdnBridgeError::MalformedScopedSymbol { detail } => {
                write!(
                    f,
                    "#{SCOPED_SYM_NS}/{SCOPED_SYM_NAME} body must be \
                     {{:{FIELD_NAME} \"…\" :{FIELD_SCOPES} [ids…]}}: {detail}"
                )
            }
            WatEdnBridgeError::MalformedSpanned { detail } => {
                write!(
                    f,
                    "#{SPANNED_NS}/{SPANNED_NAME} body must be \
                     {{:{FIELD_NODE} … :{FIELD_ORIGIN} N :{FIELD_LINE} N :{FIELD_COL} N}}: {detail}"
                )
            }
            WatEdnBridgeError::ParseFrame { msg } => {
                write!(f, "EDN parse error: {msg}")
            }
            WatEdnBridgeError::ExpectedVector { got } => {
                write!(f, "program frame must be a Vector, got: {got}")
            }
            WatEdnBridgeError::ExpectedProgram { got } => {
                write!(
                    f,
                    "program frame must be #{PROGRAM_NS}/{PROGRAM_NAME} \
                     {{:{FIELD_ORIGINS} […] :{FIELD_FORMS} […]}}: {got}"
                )
            }
        }
    }
}

impl std::error::Error for WatEdnBridgeError {}

// ─── Encode: WatAST → OwnedValue ────────────────────────────────

/// Convert a single `WatAST` node to its plain-EDN `OwnedValue` twin.
///
/// This is a near-1:1 structural map — every `WatAST` variant has an
/// EDN counterpart. Keyword encoding reuses `edn_shim::keyword_from_wat_path`
/// so the `::` ↔ `.` / `/` translation is identical everywhere.
///
/// # Notes
///
/// - A BARE `Symbol` (empty scope set) → EDN `Symbol` (bare, no namespace).
///   That is every symbol the parser emits and every one in hand-written
///   code, so nothing that round-trips today changes spelling.
/// - A SCOPED `Symbol` (a macro minted it) →
///   `#wat.ast/ScopedSymbol {:name "…" :scopes [ids…]}`.
///   The scope set is load-bearing at every bind and lookup (`env_key`,
///   `src/scope/resolution.rs`), so it must cross. The wire ids are opaque
///   markers — the decode side remaps them to fresh local scopes, preserving
///   which identifiers share a scope; see [`ScopeImport`].
///
///   This CORRECTS a prior claim in this doc comment — that dropping scopes
///   "is correct for programs shipped over the wire (freeze re-derives scopes
///   from source position)". It does not: `spawn-process` ships an evaluated
///   `Vector<WatAST>` a macro built (`src/kernel/spawn.rs:485`), whose scopes
///   were minted at the SENDER's expansion and are never re-derived. Erasing
///   them on one side of a wire manufactures `HygieneScopeDivergence`
///   (`check.rs:2035`). Disconfirmed by a run in
///   `tests/program/probe_arc170_edn_bridge_hygiene.rs`.
/// - `Span` is preserved under [`Carriage::Transport`] — every node is
///   wrapped `#wat.ast/Spanned {:node … :origin N :line N :col N …}` (see
///   [`wrap_spanned`]) — but NOT under [`Carriage::Display`]: this doc used to
///   claim spans are unnecessary everywhere because `startup_from_forms` /
///   `freeze` re-derive what THEY need from the semantic structure, not the
///   span. That is still true of resolution — it is false of EXECUTION. A
///   delivered program is run, running raises, and a raise reads the AST
///   node's span for its `Fault` location; dropping it on the wire means
///   every diagnostic from a program shipped this way names this decoder's
///   own Rust line instead of the user's source (stone J, arc 296). Display
///   carriage still drops spans on purpose: it renders for a reader and
///   nothing downstream re-parses it, so a span there is pure noise.
pub fn watast_to_edn(a: &WatAST) -> OwnedValue {
    // DISPLAY, not transport. `write-forms`, `ast->source` and reflection's
    // `signature-of` / `lookup-define` all render through here, and a rendering
    // is allowed to be pretty: `:wat.core/Vector/length` reads better than a
    // tagged record, and nothing downstream re-parses it.
    watast_to_edn_with(a, Carriage::Display, &mut OriginTable::default())
}

/// Display renders for a human; transport must survive a re-read.
///
/// The two were one function until arc 170, and braiding them was a real
/// defect in both directions: transport silently mangled forms EDN cannot
/// spell, and when the fix landed on the shared path it changed every
/// `signature-of` rendering in the reflection suite. They want different
/// things — display wants legible and never re-parses; transport wants exact
/// and always does. `solvere`: one reason to change each.
#[derive(Clone, Copy, PartialEq)]
enum Carriage {
    /// Rendering for a reader. Unspellable lexemes render as the codec's best
    /// effort; nothing re-parses the result.
    Display,
    /// Crossing a process boundary. What EDN cannot spell is carried VERBATIM,
    /// because the far side parses this back into the same program.
    Transport,
}

fn watast_to_edn_with(a: &WatAST, carriage: Carriage, origins: &mut OriginTable) -> OwnedValue {
    let verbatim = carriage == Carriage::Transport;
    let content = match a {
        WatAST::IntLit(n, _) => OwnedValue::Integer(*n),
        WatAST::FloatLit(x, _) => OwnedValue::Float(*x),
        // Arc 300 stone B — rational literal, already reduced/normalized.
        WatAST::RationalLit(r, _) => OwnedValue::Rational(Box::new(r.clone())),
        // Arc 300 stone C1 — bigint literal (mirrors Rational immediately above,
        // one type over).
        WatAST::BigIntLit(n, _) => OwnedValue::BigInt(Box::new(n.clone())),
        // Arc 300 stone D — char literal decodes to/from `Edn::Char` directly
        // (mirrors BigInt/Rational immediately above, one type over). This is
        // the motion arc 300 C1 already performed for BigInt — see the
        // `Char` doc-line update at `:540`-ish and the `Edn::Char` decode arm
        // near `:816`.
        WatAST::CharLit(c, _) => OwnedValue::Char(*c),
        WatAST::BoolLit(b, _) => OwnedValue::Bool(*b),
        WatAST::StringLit(s, _) => OwnedValue::String(std::borrow::Cow::Owned(s.clone())),
        WatAST::NilLit(_) => OwnedValue::Nil,
        // A keyword EDN can spell crosses as a plain EDN keyword, so every frame
        // that round-trips today keeps its exact spelling. One EDN cannot spell
        // is carried VERBATIM rather than mangled — see [`WAT_KEYWORD_NAME`].
        WatAST::Keyword(k, _) if !verbatim || !needs_verbatim_carriage(k) => {
            keyword_from_wat_path(k)
        }
        WatAST::Keyword(k, _) => verbatim_keyword(k),
        WatAST::Symbol(ident, _)
            if ident.scopes().is_empty()
                && (!verbatim || !symbol_needs_verbatim(ident.as_str())) =>
        {
            OwnedValue::Symbol(Symbol::new(ident.as_str()))
        }
        WatAST::Symbol(ident, _) => {
            // Either a macro minted this name (carry its hygiene scopes), or EDN
            // cannot spell the name itself (carry it verbatim) — one tag, because
            // both are the same act: this symbol does not survive as a plain one.
            //
            // The ids go out as EDN integers. `ScopeId` wraps a `u64` drawn
            // from a monotonic per-process counter incremented once per macro
            // expansion, so the i64 range is not reachable by any program that
            // can be expanded at all; the debug assert guards the conversion at
            // the single chokepoint, the rung `Identifier::bare` already chose
            // for its own U+0001 invariant.
            let ids: Vec<OwnedValue> = ident
                .scopes()
                .iter()
                .map(|s| {
                    let raw = s.as_u64();
                    debug_assert!(
                        raw <= i64::MAX as u64,
                        "ScopeId {raw} exceeds the EDN integer range"
                    );
                    OwnedValue::Integer(raw as i64)
                })
                .collect();
            OwnedValue::Tagged(
                Tag::ns(SCOPED_SYM_NS, SCOPED_SYM_NAME),
                Box::new(OwnedValue::Map(vec![
                    (
                        OwnedValue::Keyword(Keyword::new(FIELD_NAME)),
                        OwnedValue::String(std::borrow::Cow::Owned(ident.as_str().to_owned())),
                    ),
                    (
                        OwnedValue::Keyword(Keyword::new(FIELD_SCOPES)),
                        OwnedValue::Vector(ids),
                    ),
                ])),
            )
        }
        WatAST::List(items, _) => OwnedValue::List(
            items.iter().map(|i| watast_to_edn_with(i, carriage, origins)).collect(),
        ),
        WatAST::Vector(items, _) => OwnedValue::Vector(
            items.iter().map(|i| watast_to_edn_with(i, carriage, origins)).collect(),
        ),
        WatAST::Map(pairs, _) => OwnedValue::Map(
            pairs
                .iter()
                .map(|(k, v)| {
                    (
                        watast_to_edn_with(k, carriage, origins),
                        watast_to_edn_with(v, carriage, origins),
                    )
                })
                .collect(),
        ),
        WatAST::Set(items, _) => OwnedValue::Set(
            items.iter().map(|i| watast_to_edn_with(i, carriage, origins)).collect(),
        ),
    };
    if carriage == Carriage::Transport {
        wrap_spanned(content, a.span(), origins)
    } else {
        content
    }
}

// ─── Decode: OwnedValue → WatAST ────────────────────────────────

/// Convert a single plain-EDN `OwnedValue` back to a `WatAST` node.
///
/// The inverse of `watast_to_edn`. Keyword decoding reuses
/// `edn_shim::ns_to_wat_path` so the namespace→`::` path rebuild is
/// identical to every other decode site.
///
/// Returns a `WatEdnBridgeError` (never panics) for EDN forms that have
/// no WatAST counterpart: `Tagged`, `Inst`, `Uuid`, `BigDec`, or a
/// namespaced `Symbol`. Arc 300 stone C1: `BigInt` now decodes to
/// `WatAST::BigIntLit` — no longer in this list. Arc 300 stone D: `Char`
/// now decodes to `WatAST::CharLit` — no longer in this list either.
///
/// Span survives ONLY when the wire actually carries one — a node written
/// under `Carriage::Transport` arrives wrapped `#wat.ast/Spanned {…}` (see
/// [`wrap_spanned`]) and that real span is decoded onto the reconstructed
/// node. A node with no such wrapper (everything `watast_to_edn`/Display
/// writes, and anything else EDN handed to this decoder) carries
/// `crate::rust_caller_span!()` — the truth about where THIS call
/// reconstructed it, not a guess at where it originally came from.
pub fn edn_to_watast(v: &OwnedValue) -> Result<WatAST, WatEdnBridgeError> {
    // A lone node's scopes are internally consistent, so a table of its own is
    // correct here. Whole PROGRAMS go through `edn_to_program`, which threads
    // ONE table across every form so sharing survives across form boundaries.
    edn_to_watast_with(v, &mut ScopeImport::default(), &[])
}

/// `edn_to_watast`, threading the scope-import table and the origin table
/// (empty for a lone node with no `:origins` frame around it). Dispatches on
/// whether `v` is a `#wat.ast/Spanned` wrapper: unwrap it and decode the real
/// span, or fall through to [`edn_to_watast_node`] with a fresh
/// `rust_caller_span!()`.
fn edn_to_watast_with(
    v: &OwnedValue,
    scopes: &mut ScopeImport,
    origins: &[Arc<String>],
) -> Result<WatAST, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    if let Edn::Tagged(tag, body) = v {
        if tag.namespace() == SPANNED_NS && tag.name() == SPANNED_NAME {
            return decode_spanned(body, scopes, origins);
        }
    }
    edn_to_watast_node(v, scopes, origins, crate::rust_caller_span!())
}

/// Unwrap a `#wat.ast/Spanned {:node … :origin N :line N :col N …}` body,
/// resolve its real [`Span`], and decode the wrapped node onto it.
fn decode_spanned(
    body: &OwnedValue,
    scopes: &mut ScopeImport,
    origins: &[Arc<String>],
) -> Result<WatAST, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    let Edn::Map(fields) = body else {
        return Err(WatEdnBridgeError::MalformedSpanned {
            detail: format!("body is {}, want a record Map", body.type_name()),
        });
    };
    let field = |want: &str| {
        fields.iter().find_map(|(k, v)| match k {
            Edn::Keyword(kw) if kw.namespace().is_none() && kw.name() == want => Some(v),
            _ => None,
        })
    };
    let node = field(FIELD_NODE).ok_or_else(|| WatEdnBridgeError::MalformedSpanned {
        detail: format!("missing :{FIELD_NODE}"),
    })?;
    let int_field = |want: &str| -> Result<i64, WatEdnBridgeError> {
        match field(want) {
            Some(Edn::Integer(n)) => Ok(*n),
            Some(other) => Err(WatEdnBridgeError::MalformedSpanned {
                detail: format!(":{want} is {}, want an Integer", other.type_name()),
            }),
            None => Err(WatEdnBridgeError::MalformedSpanned {
                detail: format!("missing :{want}"),
            }),
        }
    };
    let origin_idx = int_field(FIELD_ORIGIN)?;
    let line = int_field(FIELD_LINE)?;
    let col = int_field(FIELD_COL)?;
    let end = match (field(FIELD_END_LINE), field(FIELD_END_COL)) {
        (Some(Edn::Integer(el)), Some(Edn::Integer(ec))) => Some(Pos { line: *el, col: *ec }),
        (None, None) => None,
        _ => {
            return Err(WatEdnBridgeError::MalformedSpanned {
                detail: format!(":{FIELD_END_LINE}/:{FIELD_END_COL} must both be present or both absent"),
            })
        }
    };
    let file = origins
        .get(usize::try_from(origin_idx).unwrap_or(usize::MAX))
        .cloned()
        .ok_or_else(|| WatEdnBridgeError::MalformedSpanned {
            detail: format!(
                ":{FIELD_ORIGIN} {origin_idx} has no entry in a {}-file :{FIELD_ORIGINS} table",
                origins.len()
            ),
        })?;
    let span = Span { file, line, col, end };
    edn_to_watast_node(node, scopes, origins, span)
}

/// The actual node decode — every constructed `WatAST` carries `span`
/// (cloned per node; `Span` clones cheaply, see its module docs) rather than
/// stamping `rust_caller_span!()` directly, so a caller with a REAL span
/// (from [`decode_spanned`]) gets it faithfully, and a caller with none
/// (bare [`edn_to_watast_with`] fallback) gets the honest default.
fn edn_to_watast_node(
    v: &OwnedValue,
    scopes: &mut ScopeImport,
    origins: &[Arc<String>],
    span: Span,
) -> Result<WatAST, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    match v {
        Edn::Nil => Ok(WatAST::NilLit(span)),
        Edn::Bool(b) => Ok(WatAST::BoolLit(*b, span)),
        Edn::Integer(n) => Ok(WatAST::IntLit(*n, span)),
        Edn::Float(x) => Ok(WatAST::FloatLit(*x, span)),
        // Arc 300 stone B — rational literal round-trip.
        Edn::Rational(r) => Ok(WatAST::RationalLit((**r).clone(), span)),
        // Arc 300 stone C1 — bigint literal round-trip (mirrors Rational
        // immediately above, one type over; symmetric with this file's own
        // `watast_to_edn` encode arm, which now emits `OwnedValue::BigInt`).
        Edn::BigInt(n) => Ok(WatAST::BigIntLit((**n).clone(), span)),
        // Arc 300 stone D — char literal round-trip (mirrors BigInt
        // immediately above, one type over; symmetric with this file's own
        // `watast_to_edn` encode arm, which now emits `OwnedValue::Char`).
        // This is the one arm the brief called non-mechanical: it retires
        // the `Edn::Char => Err(UnsupportedEdnForm)` refusal below (near the
        // other "no WatAST counterpart" arms) into an honest decode.
        Edn::Char(c) => Ok(WatAST::CharLit(*c, span)),
        Edn::String(s) => Ok(WatAST::StringLit(s.as_ref().to_owned(), span)),
        Edn::Keyword(kw) => {
            let path = match kw.namespace() {
                Some(ns) => ns_to_wat_path(ns, kw.name()),
                None => format!(":{}", kw.name()),
            };
            Ok(WatAST::Keyword(path, span))
        }
        Edn::Symbol(sym) => {
            // A wat symbol's name may itself contain `/` — a faithful-Clojure
            // head like `wat.core/typealias` is ONE symbol to wat-reader. EDN
            // re-lexes that as a NAMESPACED symbol, so the namespace is not a
            // second concept here; it is the front half of the name, and the
            // faithful inverse is to rejoin it.
            //
            // This corrects a prior claim on this arm — "a program AST never
            // contains them; reject cleanly". Arc 300's own conversion fixtures
            // (`tests/resolve/probe_arc251_decl_migrator__*.wat`, already written
            // in the faithful surface) are programs that do, and they were 45 of
            // the corpus's decode failures.
            let name = match sym.namespace() {
                Some(ns) => format!("{ns}/{}", sym.name()),
                None => sym.name().to_owned(),
            };
            Ok(WatAST::Symbol(Identifier::bare(name), span))
        }
        Edn::List(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes, origins)).collect();
            Ok(WatAST::List(nodes?, span))
        }
        Edn::Vector(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes, origins)).collect();
            Ok(WatAST::Vector(nodes?, span))
        }
        Edn::Map(pairs) => {
            let mut out: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs.len());
            for (k, val) in pairs {
                out.push((
                    edn_to_watast_with(k, scopes, origins)?,
                    edn_to_watast_with(val, scopes, origins)?,
                ));
            }
            Ok(WatAST::Map(out, span))
        }
        Edn::Set(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes, origins)).collect();
            Ok(WatAST::Set(nodes?, span))
        }
        // A wat keyword EDN cannot spell — carried verbatim on the way out.
        Edn::Tagged(tag, body)
            if tag.namespace() == SCOPED_SYM_NS && tag.name() == WAT_KEYWORD_NAME =>
        {
            let Edn::Map(fields) = body.as_ref() else {
                return Err(WatEdnBridgeError::MalformedScopedSymbol {
                    detail: format!("body is {}, want a record Map", body.type_name()),
                });
            };
            let path = fields.iter().find_map(|(k, v)| match (k, v) {
                (Edn::Keyword(kw), Edn::String(s))
                    if kw.namespace().is_none() && kw.name() == FIELD_PATH =>
                {
                    Some(s.as_ref())
                }
                _ => None,
            });
            match path {
                Some(p) => Ok(WatAST::Keyword(p.to_owned(), span)),
                None => Err(WatEdnBridgeError::MalformedScopedSymbol {
                    detail: format!("missing or non-String :{FIELD_PATH}"),
                }),
            }
        }
        // A scoped symbol — the encode side's `#wat.ast/ScopedSymbol {…}`.
        // The wire ids are opaque markers; each distinct one becomes a FRESH
        // local scope, so sharing is preserved and a collision with this
        // process's own scopes is unrepresentable. See [`ScopeImport`].
        Edn::Tagged(tag, body)
            if tag.namespace() == SCOPED_SYM_NS && tag.name() == SCOPED_SYM_NAME =>
        {
            let fields = match body.as_ref() {
                Edn::Map(fields) => fields,
                other => {
                    return Err(WatEdnBridgeError::MalformedScopedSymbol {
                        detail: format!("body is {}, want a record Map", other.type_name()),
                    })
                }
            };
            let field = |want: &str| {
                fields.iter().find_map(|(k, v)| match k {
                    Edn::Keyword(kw) if kw.namespace().is_none() && kw.name() == want => Some(v),
                    _ => None,
                })
            };
            let name = match field(FIELD_NAME) {
                Some(Edn::String(s)) => s.as_ref(),
                Some(other) => {
                    return Err(WatEdnBridgeError::MalformedScopedSymbol {
                        detail: format!(":{FIELD_NAME} is {}, want a String", other.type_name()),
                    })
                }
                None => {
                    return Err(WatEdnBridgeError::MalformedScopedSymbol {
                        detail: format!("missing :{FIELD_NAME}"),
                    })
                }
            };
            let ids = match field(FIELD_SCOPES) {
                Some(Edn::Vector(ids)) => ids,
                Some(other) => {
                    return Err(WatEdnBridgeError::MalformedScopedSymbol {
                        detail: format!(":{FIELD_SCOPES} is {}, want a Vector", other.type_name()),
                    })
                }
                None => {
                    return Err(WatEdnBridgeError::MalformedScopedSymbol {
                        detail: format!("missing :{FIELD_SCOPES}"),
                    })
                }
            };
            // An empty scope vector is legitimate ONLY when the name itself is
            // unspellable in EDN (`mk<S,R>`). If a plain EDN symbol would have
            // round-tripped it, the tag is a second way to write one thing —
            // refuse it, so the encoder cannot drift into tagging everything.
            if ids.is_empty() && !symbol_needs_verbatim(name) {
                return Err(WatEdnBridgeError::MalformedScopedSymbol {
                    detail: format!(
                        "no scopes and {name:?} spells fine as a plain EDN Symbol — \
                         the tag is redundant"
                    ),
                });
            }
            let mut ident = Identifier::bare(name);
            for id in ids {
                match id {
                    Edn::Integer(n) => ident = ident.add_scope(scopes.local(*n)),
                    other => {
                        return Err(WatEdnBridgeError::MalformedScopedSymbol {
                            detail: format!("scope id is {}, want an Integer", other.type_name()),
                        })
                    }
                }
            }
            Ok(WatAST::Symbol(ident, span))
        }
        // EDN forms with no WatAST counterpart — STOP trigger: if a real
        // program AST actually contains these, the mapping is incomplete.
        Edn::Tagged(tag, _body) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("Tagged #{}/{}", tag.namespace(), tag.name()),
        }),
        Edn::Inst(dt) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("Inst({dt})"),
        }),
        Edn::Uuid(u) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("Uuid({u})"),
        }),
        Edn::BigDec(d) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("BigDec({d})"),
        }),
    }
}

// ─── Program-level API ──────────────────────────────────────────

/// Serialize a whole `Vec<WatAST>` program to a single plain-EDN frame string.
///
/// The program is wrapped `#wat.ast/Program {:origins […] :forms […]}`:
/// `:origins` interns each distinct `Span::file` once ([`OriginTable`]), and
/// every node in `:forms` (recursively, at every depth) is wrapped
/// `#wat.ast/Spanned {:node … :origin N :line N :col N …}` so the span it was
/// parsed with — or synthesized with, if built programmatically — survives
/// the crossing. This matches the spec's "program = first frame on fd0"
/// decision from DESIGN-EXECVE-PROGRAM-OVER-WIRE.md §4 (same-fd framed).
///
/// The output contains **NO** tagged-HolonAST wire forms — it is plain EDN.
/// Contains native `{ }` map and `#{ }` set syntax, and `:ns/name` keywords.
pub fn program_to_edn(forms: &[WatAST]) -> String {
    let mut origins = OriginTable::default();
    let items: Vec<OwnedValue> = forms
        .iter()
        .map(|f| watast_to_edn_with(f, Carriage::Transport, &mut origins))
        .collect();
    let frame = OwnedValue::Tagged(
        Tag::ns(PROGRAM_NS, PROGRAM_NAME),
        Box::new(OwnedValue::Map(vec![
            (
                OwnedValue::Keyword(Keyword::new(FIELD_ORIGINS)),
                OwnedValue::Vector(origins.into_values()),
            ),
            (OwnedValue::Keyword(Keyword::new(FIELD_FORMS)), OwnedValue::Vector(items)),
        ])),
    );
    wat_edn::write(&frame)
}

/// Deserialize a program frame produced by `program_to_edn` back to
/// `Vec<WatAST>`.
///
/// Expects the frame to be the `#wat.ast/Program {:origins […] :forms […]}`
/// record `program_to_edn` produces; resolves `:origins` once, then decodes
/// each `:forms` entry against it so every node's real span survives.
/// Returns a `WatEdnBridgeError` on parse failure, wrong top-level shape, a
/// malformed `Spanned`/`Program` wrapper, or any EDN form that has no
/// `WatAST` counterpart.
pub fn edn_to_program(frame: &str) -> Result<Vec<WatAST>, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    let owned = wat_edn::parse_owned(frame).map_err(|e| WatEdnBridgeError::ParseFrame {
        msg: e.to_string(),
    })?;
    let Edn::Tagged(tag, body) = &owned else {
        return Err(WatEdnBridgeError::ExpectedProgram {
            got: owned.type_name().to_owned(),
        });
    };
    if tag.namespace() != PROGRAM_NS || tag.name() != PROGRAM_NAME {
        return Err(WatEdnBridgeError::ExpectedProgram {
            got: format!("Tagged #{}/{}", tag.namespace(), tag.name()),
        });
    }
    let Edn::Map(fields) = body.as_ref() else {
        return Err(WatEdnBridgeError::ExpectedProgram {
            got: format!("#{PROGRAM_NS}/{PROGRAM_NAME} body is {}, want a Map", body.type_name()),
        });
    };
    let field = |want: &str| {
        fields.iter().find_map(|(k, v)| match k {
            Edn::Keyword(kw) if kw.namespace().is_none() && kw.name() == want => Some(v),
            _ => None,
        })
    };
    let origins_val = field(FIELD_ORIGINS).ok_or_else(|| WatEdnBridgeError::ExpectedProgram {
        got: format!("missing :{FIELD_ORIGINS}"),
    })?;
    let origins: Vec<Arc<String>> = match origins_val {
        Edn::Vector(items) => items
            .iter()
            .map(|v| match v {
                Edn::String(s) => Ok(Arc::new(s.as_ref().to_owned())),
                other => Err(WatEdnBridgeError::ExpectedProgram {
                    got: format!(":{FIELD_ORIGINS} entry is {}, want a String", other.type_name()),
                }),
            })
            .collect::<Result<_, _>>()?,
        other => {
            return Err(WatEdnBridgeError::ExpectedProgram {
                got: format!(":{FIELD_ORIGINS} is {}, want a Vector", other.type_name()),
            })
        }
    };
    let forms_val = field(FIELD_FORMS).ok_or_else(|| WatEdnBridgeError::ExpectedProgram {
        got: format!("missing :{FIELD_FORMS}"),
    })?;
    // ONE table across every form: two forms that shared a scope at the sender
    // must still share one here.
    let mut scopes = ScopeImport::default();
    match forms_val {
        Edn::Vector(items) => items
            .iter()
            .map(|i| edn_to_watast_with(i, &mut scopes, &origins))
            .collect(),
        other => Err(WatEdnBridgeError::ExpectedVector {
            got: other.type_name().to_owned(),
        }),
    }
}

