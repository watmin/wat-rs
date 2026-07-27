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
//! Do NOT hand-roll the `::` ↔ `.` / `/` translation.
//!
//! ## Arc 213 / BRIEF-213-SERIALIZER-BRIDGE context
//!
//! This bridge is the serializer corrected by arc 257's SUPERSEDED note:
//! the old `watast_to_holon` path encoded every node under `#wat-edn.holon/*`
//! tags (the VSA hologram), which is the contract-vs-encoding abuse, NOT EDN
//! transport. Plain EDN is the correct wire format; arc 257's native Map/Set
//! nodes make the mapping 1:1.

use crate::ast::WatAST;
use crate::edn_shim::{keyword_from_wat_path, ns_to_wat_path};
use crate::scope::{fresh_scope, Identifier, ScopeId};
use std::collections::HashMap;
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

// ─── Error type ─────────────────────────────────────────────────

/// Error returned by the decode path (`edn_to_watast`, `edn_to_program`).
/// Never panics; callers receive a clean typed error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatEdnBridgeError {
    /// An EDN form shape that has no `WatAST` counterpart appeared in the
    /// encoded program frame (e.g. `Tagged`, `Inst`, `Uuid`, `Char`,
    /// `BigDec`, namespaced `Symbol`). Arc 300 stone C1: `BigInt` now HAS a
    /// counterpart (`WatAST::BigIntLit`) and is no longer in this list.
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
    /// The top-level frame failed to parse as EDN.
    ParseFrame { msg: String },
    /// The top-level frame is valid EDN but not a Vector (program must be wrapped
    /// in `[...]` by `program_to_edn`).
    ExpectedVector { got: String },
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
            WatEdnBridgeError::ParseFrame { msg } => {
                write!(f, "EDN parse error: {msg}")
            }
            WatEdnBridgeError::ExpectedVector { got } => {
                write!(f, "program frame must be a Vector, got: {got}")
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
/// - `Span` is not preserved (EDN carries no source location). On decode,
///   `crate::rust_caller_span!()` is used throughout; `startup_from_forms` / `freeze`
///   re-derives what it needs from the semantic structure, not the span.
pub fn watast_to_edn(a: &WatAST) -> OwnedValue {
    match a {
        WatAST::IntLit(n, _) => OwnedValue::Integer(*n),
        WatAST::FloatLit(x, _) => OwnedValue::Float(*x),
        // Arc 300 stone B — rational literal, already reduced/normalized.
        WatAST::RationalLit(r, _) => OwnedValue::Rational(Box::new(r.clone())),
        // Arc 300 stone C1 — bigint literal (mirrors Rational immediately above,
        // one type over).
        WatAST::BigIntLit(n, _) => OwnedValue::BigInt(Box::new(n.clone())),
        WatAST::BoolLit(b, _) => OwnedValue::Bool(*b),
        WatAST::StringLit(s, _) => OwnedValue::String(std::borrow::Cow::Owned(s.clone())),
        WatAST::NilLit(_) => OwnedValue::Nil,
        WatAST::Keyword(k, _) => keyword_from_wat_path(k),
        WatAST::Symbol(ident, _) if ident.scopes().is_empty() => {
            OwnedValue::Symbol(Symbol::new(ident.as_str()))
        }
        WatAST::Symbol(ident, _) => {
            // A macro minted this name. Carry its hygiene scopes.
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
        WatAST::List(items, _) => {
            OwnedValue::List(items.iter().map(watast_to_edn).collect())
        }
        WatAST::Vector(items, _) => {
            OwnedValue::Vector(items.iter().map(watast_to_edn).collect())
        }
        WatAST::Map(pairs, _) => OwnedValue::Map(
            pairs
                .iter()
                .map(|(k, v)| (watast_to_edn(k), watast_to_edn(v)))
                .collect(),
        ),
        WatAST::Set(items, _) => {
            OwnedValue::Set(items.iter().map(watast_to_edn).collect())
        }
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
/// no WatAST counterpart: `Tagged`, `Inst`, `Uuid`, `Char`, `BigDec`, or a
/// namespaced `Symbol`. Arc 300 stone C1: `BigInt` now decodes to
/// `WatAST::BigIntLit` — no longer in this list.
///
/// Span is not preserved — all reconstructed nodes carry `crate::rust_caller_span!()`.
/// `startup_from_forms` and the freeze pipeline work correctly with unknown
/// spans; type-check and resolution operate on the semantic structure.
pub fn edn_to_watast(v: &OwnedValue) -> Result<WatAST, WatEdnBridgeError> {
    // A lone node's scopes are internally consistent, so a table of its own is
    // correct here. Whole PROGRAMS go through `edn_to_program`, which threads
    // ONE table across every form so sharing survives across form boundaries.
    edn_to_watast_with(v, &mut ScopeImport::default())
}

/// `edn_to_watast`, threading the scope-import table. See [`ScopeImport`].
fn edn_to_watast_with(
    v: &OwnedValue,
    scopes: &mut ScopeImport,
) -> Result<WatAST, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    match v {
        Edn::Nil => Ok(WatAST::NilLit(crate::rust_caller_span!())),
        Edn::Bool(b) => Ok(WatAST::BoolLit(*b, crate::rust_caller_span!())),
        Edn::Integer(n) => Ok(WatAST::IntLit(*n, crate::rust_caller_span!())),
        Edn::Float(x) => Ok(WatAST::FloatLit(*x, crate::rust_caller_span!())),
        // Arc 300 stone B — rational literal round-trip.
        Edn::Rational(r) => Ok(WatAST::RationalLit((**r).clone(), crate::rust_caller_span!())),
        // Arc 300 stone C1 — bigint literal round-trip (mirrors Rational
        // immediately above, one type over; symmetric with this file's own
        // `watast_to_edn` encode arm, which now emits `OwnedValue::BigInt`).
        Edn::BigInt(n) => Ok(WatAST::BigIntLit((**n).clone(), crate::rust_caller_span!())),
        Edn::String(s) => Ok(WatAST::StringLit(s.as_ref().to_owned(), crate::rust_caller_span!())),
        Edn::Keyword(kw) => {
            let path = match kw.namespace() {
                Some(ns) => ns_to_wat_path(ns, kw.name()),
                None => format!(":{}", kw.name()),
            };
            Ok(WatAST::Keyword(path, crate::rust_caller_span!()))
        }
        Edn::Symbol(sym) => {
            if sym.namespace().is_some() {
                // Namespaced EDN symbols have no WatAST counterpart.
                // A program AST never contains them; reject cleanly.
                return Err(WatEdnBridgeError::UnsupportedEdnForm {
                    shape: format!("namespaced Symbol ({:?}/{:?})", sym.namespace(), sym.name()),
                });
            }
            Ok(WatAST::Symbol(
                Identifier::bare(sym.name()),
                crate::rust_caller_span!(),
            ))
        }
        Edn::List(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes)).collect();
            Ok(WatAST::List(nodes?, crate::rust_caller_span!()))
        }
        Edn::Vector(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes)).collect();
            Ok(WatAST::Vector(nodes?, crate::rust_caller_span!()))
        }
        Edn::Map(pairs) => {
            let mut out: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs.len());
            for (k, val) in pairs {
                out.push((
                    edn_to_watast_with(k, scopes)?,
                    edn_to_watast_with(val, scopes)?,
                ));
            }
            Ok(WatAST::Map(out, crate::rust_caller_span!()))
        }
        Edn::Set(items) => {
            let nodes: Result<Vec<WatAST>, _> =
                items.iter().map(|i| edn_to_watast_with(i, scopes)).collect();
            Ok(WatAST::Set(nodes?, crate::rust_caller_span!()))
        }
        // A scoped symbol — the encode side's `#wat.ast/sym ["name" [ids…]]`.
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
            if ids.is_empty() {
                // A bare symbol has a spelling of its own; a tagged one with no
                // scopes is two ways to write one thing. Refuse it.
                return Err(WatEdnBridgeError::MalformedScopedSymbol {
                    detail: "empty scope vector — a bare symbol must be a plain EDN Symbol"
                        .to_owned(),
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
            Ok(WatAST::Symbol(ident, crate::rust_caller_span!()))
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
        Edn::Char(c) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("Char({c:?})"),
        }),
        Edn::BigDec(d) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("BigDec({d})"),
        }),
    }
}

// ─── Program-level API ──────────────────────────────────────────

/// Serialize a whole `Vec<WatAST>` program to a single plain-EDN frame string.
///
/// The program is wrapped in a top-level EDN Vector: `[form0 form1 ...]`.
/// This matches the spec's "program = first frame on fd0" decision from
/// DESIGN-EXECVE-PROGRAM-OVER-WIRE.md §4 (same-fd framed).
///
/// The output contains **NO** `#wat-edn.holon` tags — it is plain EDN.
/// Contains native `{ }` map and `#{ }` set syntax, and `:ns/name` keywords.
pub fn program_to_edn(forms: &[WatAST]) -> String {
    let items: Vec<OwnedValue> = forms.iter().map(watast_to_edn).collect();
    wat_edn::write(&OwnedValue::Vector(items))
}

/// Deserialize a program frame produced by `program_to_edn` back to
/// `Vec<WatAST>`.
///
/// Expects the frame to be a top-level EDN Vector (as `program_to_edn`
/// produces). Returns a `WatEdnBridgeError` on parse failure, wrong top-level
/// shape, or any EDN form that has no `WatAST` counterpart.
pub fn edn_to_program(frame: &str) -> Result<Vec<WatAST>, WatEdnBridgeError> {
    let owned = wat_edn::parse_owned(frame).map_err(|e| WatEdnBridgeError::ParseFrame {
        msg: e.to_string(),
    })?;
    // ONE table across every form: two forms that shared a scope at the sender
    // must still share one here.
    let mut scopes = ScopeImport::default();
    match owned {
        OwnedValue::Vector(items) => items
            .iter()
            .map(|i| edn_to_watast_with(i, &mut scopes))
            .collect(),
        other => Err(WatEdnBridgeError::ExpectedVector {
            got: other.type_name().to_owned(),
        }),
    }
}
