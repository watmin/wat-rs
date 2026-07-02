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
use crate::scope::Identifier;
use wat_edn::{OwnedValue, Symbol};

// ─── Error type ─────────────────────────────────────────────────

/// Error returned by the decode path (`edn_to_watast`, `edn_to_program`).
/// Never panics; callers receive a clean typed error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatEdnBridgeError {
    /// An EDN form shape that has no `WatAST` counterpart appeared in the
    /// encoded program frame (e.g. `Tagged`, `Inst`, `Uuid`, `Char`,
    /// `BigInt`, `BigDec`, namespaced `Symbol`).
    UnsupportedEdnForm { shape: String },
    /// A keyword in the EDN frame could not be decoded to a wat keyword path.
    KeywordDecode { raw: String },
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
/// - `Symbol` → EDN `Symbol` (bare, no namespace). Scope sets are not
///   represented in EDN (EDN carries no hygiene metadata); they are
///   reconstructed as empty on the decode path, which is correct for
///   programs shipped over the wire (freeze re-derives scopes from source
///   position, not from the AST's scope sets).
/// - `Span` is not preserved (EDN carries no source location). On decode,
///   `crate::rust_caller_span!()` is used throughout; `startup_from_forms` / `freeze`
///   re-derives what it needs from the semantic structure, not the span.
pub fn watast_to_edn(a: &WatAST) -> OwnedValue {
    match a {
        WatAST::IntLit(n, _) => OwnedValue::Integer(*n),
        WatAST::FloatLit(x, _) => OwnedValue::Float(*x),
        WatAST::BoolLit(b, _) => OwnedValue::Bool(*b),
        WatAST::StringLit(s, _) => OwnedValue::String(std::borrow::Cow::Owned(s.clone())),
        WatAST::NilLit(_) => OwnedValue::Nil,
        WatAST::Keyword(k, _) => keyword_from_wat_path(k),
        WatAST::Symbol(ident, _) => OwnedValue::Symbol(Symbol::new(ident.as_str())),
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
/// no WatAST counterpart: `Tagged`, `Inst`, `Uuid`, `Char`, `BigInt`,
/// `BigDec`, or a namespaced `Symbol`.
///
/// Span is not preserved — all reconstructed nodes carry `crate::rust_caller_span!()`.
/// `startup_from_forms` and the freeze pipeline work correctly with unknown
/// spans; type-check and resolution operate on the semantic structure.
pub fn edn_to_watast(v: &OwnedValue) -> Result<WatAST, WatEdnBridgeError> {
    use wat_edn::Value as Edn;
    match v {
        Edn::Nil => Ok(WatAST::NilLit(crate::rust_caller_span!())),
        Edn::Bool(b) => Ok(WatAST::BoolLit(*b, crate::rust_caller_span!())),
        Edn::Integer(n) => Ok(WatAST::IntLit(*n, crate::rust_caller_span!())),
        Edn::Float(x) => Ok(WatAST::FloatLit(*x, crate::rust_caller_span!())),
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
            let nodes: Result<Vec<WatAST>, _> = items.iter().map(edn_to_watast).collect();
            Ok(WatAST::List(nodes?, crate::rust_caller_span!()))
        }
        Edn::Vector(items) => {
            let nodes: Result<Vec<WatAST>, _> = items.iter().map(edn_to_watast).collect();
            Ok(WatAST::Vector(nodes?, crate::rust_caller_span!()))
        }
        Edn::Map(pairs) => {
            let mut out: Vec<(WatAST, WatAST)> = Vec::with_capacity(pairs.len());
            for (k, val) in pairs {
                out.push((edn_to_watast(k)?, edn_to_watast(val)?));
            }
            Ok(WatAST::Map(out, crate::rust_caller_span!()))
        }
        Edn::Set(items) => {
            let nodes: Result<Vec<WatAST>, _> = items.iter().map(edn_to_watast).collect();
            Ok(WatAST::Set(nodes?, crate::rust_caller_span!()))
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
        Edn::BigInt(n) => Err(WatEdnBridgeError::UnsupportedEdnForm {
            shape: format!("BigInt({n})"),
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
    match owned {
        OwnedValue::Vector(items) => items.iter().map(edn_to_watast).collect(),
        other => Err(WatEdnBridgeError::ExpectedVector {
            got: other.type_name().to_owned(),
        }),
    }
}
