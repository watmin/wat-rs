//! `wat-edn` — spec-conforming EDN parser and writer.
//!
//! EDN is the data interchange format Rich Hickey defined for
//! Clojure: <https://github.com/edn-format/edn>. This crate is a
//! second conforming implementation — peer to Clojure's reference
//! reader, written in Rust, designed for wat but useful anywhere
//! Rust code needs to read or write EDN.
//!
//! # Example
//!
//! ```
//! use wat_edn::{parse, write, Value};
//!
//! let v = parse("[1 2 3]").unwrap();
//! assert!(matches!(v, Value::Vector(_)));
//!
//! let s = write(&v);
//! assert_eq!(s, "[1 2 3]");
//! ```
//!
//! # Coverage
//!
//! Every literal type defined by the EDN spec:
//!
//! - `nil`, `true`, `false`
//! - integers (`i64`) and big integers (`42N`)
//! - floats (`f64`) and big decimals (`3.14M`)
//! - strings (with `\n \t \r \b \f \" \\ \/ \uXXXX` escapes)
//! - characters (`\c \newline \space \tab \return \formfeed \backspace \uXXXX`)
//! - symbols and namespaced symbols (`foo`, `ns/foo`)
//! - keywords and namespaced keywords (`:foo`, `:ns/foo`)
//! - lists `(1 2 3)`, vectors `[1 2 3]`, maps `{:k :v}`, sets `#{1 2 3}`
//! - tagged elements `#tag value` with arbitrary nesting
//! - built-in tags `#inst` (RFC 3339) and `#uuid` (canonical form)
//! - comments (`;`) and discard (`#_`)
//!
//! User tags must have a namespace prefix per the spec (`#myapp/Type`);
//! tagless symbols are reserved for the `#inst` and `#uuid` built-ins.
//!
//! # Spec extensions
//!
//! wat-edn aligns with Clojure's reader (and JSON conventions) on a few
//! points the EDN spec does not require:
//!
//! - String escapes: spec defines `\t \r \n \\ \"` only. wat-edn also
//!   accepts `\b`, `\f`, `\/`, and `\uXXXX` for round-trip with arbitrary
//!   text. Emitted on write only when the source character requires it.
//! - Character literals: spec defines `\space \newline \tab \return`
//!   plus `\uXXXX`. wat-edn also accepts `\formfeed` and `\backspace`.
//! - Non-finite floats: spec doesn't define NaN or Infinity. wat-edn
//!   emits `#wat.core.f64/NaN []` / `#wat.core.f64/+Inf []` /
//!   `#wat.core.f64/-Inf []` so `f64` round-trips losslessly. Other
//!   EDN readers see ordinary user tags and may pass through or ignore.
//!
//! These extensions are read-and-write symmetric and round-trip cleanly.
//! A future strict-mode flag will gate them off for spec-pure output.
//!
//! # Performance
//!
//! - Hand-rolled byte-level lexer; no regex.
//! - Single-pass recursive-descent parser.
//! - Borrowed string bodies via `Cow<str>` until escape sequences
//!   force allocation.
//! - Comma is whitespace per the spec.
//!
//! See `examples/bench.rs` for the timing harness.

pub mod error;
pub mod vocab;
pub mod json;
pub mod lexer;
pub mod parser;
pub mod value;
pub mod writer;

// ─── Shared namespace constants ──────────────────────────────────────────────
//
// These live in `wat-edn` (the foundation) so every crate — `wat-reader`,
// `wat`, and any future crate — can reference them without a dependency cycle.
// `error_ns.rs` in the `wat` crate re-exports / aliases these as needed.

/// The `"wat.core"` namespace: typed value records (`Span`, `Pos`, `Option`, …).
pub const CORE: &str = "wat.core";

// ─── Public surface ─────────────────────────────────────────────
//
// Everything a caller needs — parse / parse_owned / parse_all,
// the OwnedValue alias, the canonical types, and the writer
// helpers — is re-exported here so a downstream `use wat_edn::*`
// reaches the whole API in one line.

pub use error::{Error, ErrorKind, Result};
pub use json::{
    from_json_string, to_json_string, to_json_string_pretty,
    JsonError, JsonResult,
};
pub use parser::Parser;
pub use value::{Keyword, Symbol, Tag, Value};
pub use writer::{write, write_pretty, write_to};

/// A `Value` that owns all of its string data — no input-buffer
/// lifetime to track. Storable across threads, returnable from
/// functions, persistable beyond the parsed source. Equivalent to
/// `Value<'static>`; the alias gives the storage-crossing case a
/// name. Returned by [`Value::into_owned`] and [`parse_owned`].
pub type OwnedValue = Value<'static>;

/// Serialize `self` to a structured tagged [`OwnedValue`].
///
/// Every error and diagnostic type in `wat` implements this trait. The wire and
/// IPC boundaries in `wat` are generic over `ToEdn`, so a type that does not
/// implement this trait cannot reach the wire.
///
/// The trait lives here (in `wat-edn`, the foundational crate) so that
/// `wat-reader` — and every crate — can `#[derive(wat_edn::ToEdn)]` without a
/// dependency cycle. The companion derive macro lives in `wat-to-edn-derive`
/// and is re-exported under the `derive` feature (the serde/serde_derive
/// pattern).
///
/// ## Contract
/// - The returned value MUST be structured tagged EDN, NOT a bare
///   `OwnedValue::String`.
/// - The `OwnedValue` passthrough impl (`impl ToEdn for OwnedValue`) allows
///   pre-computed EDN values to cross the boundary without unwrapping.
pub trait ToEdn {
    fn to_edn(&self) -> OwnedValue;
}

/// Identity implementation: an already-serialized [`OwnedValue`] is itself
/// the EDN form.
impl ToEdn for OwnedValue {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        self.clone()
    }
}

// ─── Primitive + container `ToEdn` impls ────────────────────────────────────
//
// These live here (where the trait is defined) to satisfy the orphan rule:
// implementing a foreign trait for a foreign type is forbidden. String, i64,
// Vec<T>, Option<T> etc. are all std/core types; only the trait's defining
// crate may implement it for them.

impl ToEdn for String {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::String(std::borrow::Cow::Owned(self.clone()))
    }
}

impl ToEdn for str {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::String(std::borrow::Cow::Owned(self.to_owned()))
    }
}

impl ToEdn for i64 {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::Integer(*self)
    }
}

impl ToEdn for usize {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::Integer(*self as i64)
    }
}

impl ToEdn for u32 {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::Integer(*self as i64)
    }
}

impl ToEdn for bool {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::Bool(*self)
    }
}

impl<T: ToEdn> ToEdn for Vec<T> {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        OwnedValue::Vector(self.iter().map(|x| x.to_edn()).collect())
    }
}

impl<T: ToEdn> ToEdn for Option<T> {
    /// Arc 278 Stone A.0 — uniform VECTOR-bodied variant encoding, in lockstep
    /// with `edn_shim`'s Option encoder/decoder:
    /// `None` → `#wat.core.Option/None []` (empty field-vector).
    /// `Some(v)` → `#wat.core.Option/Some [<v.to_edn()>]` (one-field vector).
    ///
    /// `nil` is the unit value ONLY — a variant's body is always its field-vector,
    /// so `Some(nil)` → `[nil]` (arity visible) never collides with `None` → `[]`.
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        match self {
            None => OwnedValue::Tagged(
                value::Tag::ns("wat.core.Option", "None"),
                Box::new(OwnedValue::Vector(vec![])),
            ),
            Some(v) => OwnedValue::Tagged(
                value::Tag::ns("wat.core.Option", "Some"),
                Box::new(OwnedValue::Vector(vec![v.to_edn()])),
            ),
        }
    }
}

impl<T: ToEdn> ToEdn for Box<T> {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        (**self).to_edn()
    }
}

/// Blanket: a reference to any `ToEdn` type is itself `ToEdn` (by delegation).
impl<T: ToEdn + ?Sized> ToEdn for &T {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        (**self).to_edn()
    }
}

/// Blanket: `Arc<T>` delegates to the inner `T`. Allows `Arc<String>` (and
/// any other `Arc<ToEdn>`) to participate in derived structs without a
/// hand-written via override.
impl<T: ToEdn + ?Sized> ToEdn for std::sync::Arc<T> {
    #[inline]
    fn to_edn(&self) -> OwnedValue {
        (**self).to_edn()
    }
}

/// Re-export the companion derive so consumers write
/// `#[derive(wat_edn::ToEdn)]` — the serde/serde_derive pattern. A trait
/// and a derive macro may share a name (type vs macro namespace), exactly as
/// `serde::Serialize` is both.
#[cfg(feature = "derive")]
pub use wat_to_edn_derive::ToEdn;

/// `#[derive(wat_edn::Edn)]` — the round-trip derive: generates the `ToEdn`
/// write impl AND submits an `EdnSchema` entry into the link-time inventory so
/// the reader can reconstruct the type.  One derive → both faces.
///
/// Arc 296 stone D: `#[derive(Edn)]` replaces `#[derive(ToEdn)]` for types
/// that must be readable back.  `#[derive(ToEdn)]` stays for write-only types
/// (opaque handles, etc.) that never need to be read back.
#[cfg(feature = "derive")]
pub use wat_to_edn_derive::Edn;

// ─── EDN schema registry (arc 296 stone D) ──────────────────────────────────
//
// `#[derive(Edn)]` emits one `inventory::submit!(EdnSchema { … })` per tagged
// type it processes.  The drain in `wat/src/types.rs::register_builtin_types`
// iterates `inventory::iter::<EdnSchema>()` at startup and calls
// `TypeEnv::register_builtin` for each, making the type readable via
// `reconstruct_record` in `edn_shim.rs`.
//
// Design invariant: `inventory::collect!` MUST live in the same crate as the
// type definition (`wat-edn`) so the linker sees both the collector and all
// submitted entries when building the final binary.

/// Link-time schema registry entry for round-trippable EDN types.
///
/// A `#[derive(Edn)]` emits `inventory::submit!(EdnSchema { … })` alongside
/// the `ToEdn` write impl so `reconstruct_record` (the read path in
/// `edn_shim.rs`) can find the type during startup without any hand-written
/// registration.
///
/// The drain in `wat/src/types.rs::register_builtin_types` iterates
/// `inventory::iter::<EdnSchema>()` and calls `TypeEnv::register_builtin`
/// for each entry — exactly the shape the hand-written PROBE used.
pub struct EdnSchema {
    /// Namespace component of the EDN tag, e.g. `"wat.core"`.
    /// Typically a reference to a `wat_edn::*` namespace constant.
    pub tag_ns: &'static str,
    /// Name component of the EDN tag, e.g. `"Pos"`.
    pub tag_name: &'static str,
    /// Field pairs in declaration order: `(edn-kebab-key, ":wat::type::Path")`.
    /// Skipped fields (`#[to_edn(skip)]`) are absent.
    pub fields: &'static [(&'static str, &'static str)],
}
inventory::collect!(EdnSchema);

/// Parse a single top-level EDN form from a string.
///
/// Returns `Value<'_>` borrowing from `input` for the [`Value::String`]
/// variant when the lexer's fast path produced no escape sequences
/// (zero-copy). All other variants — `Symbol`, `Keyword`, `Tag`,
/// numbers, etc. — are stored owned (identifiers go through
/// [`compact_str::CompactString`] which inlines short names).
///
/// Call [`Value::into_owned`] (or use [`parse_owned`]) to lift the
/// result to [`OwnedValue`] for storage beyond `input`'s lifetime.
///
/// # Errors
///
/// Returns the first parse error encountered; parsing stops at that
/// point. EDN trailing content after the first complete form is also
/// reported as an error (use [`parse_all`] to read multiple forms).
pub fn parse(input: &str) -> Result<Value<'_>> {
    Parser::new(input).parse_top()
}

/// Parse and immediately materialize to an [`OwnedValue`].
/// Equivalent to `parse(input)?.into_owned()`.
///
/// Use when the parsed value must outlive `input`'s borrow scope —
/// common for callers that store, return, or thread parsed values
/// across function boundaries. For zero-copy reads inside `input`'s
/// scope, prefer [`parse`].
pub fn parse_owned(input: &str) -> Result<OwnedValue> {
    parse(input).map(Value::into_owned)
}

/// Parse all top-level EDN forms from a string. Whitespace and
/// comments between forms are skipped.
///
/// # Errors
///
/// Fail-fast: returns the first error encountered; forms parsed
/// before the failure are discarded. Use [`Parser`] directly for
/// streaming consumption that recovers across errors.
pub fn parse_all(input: &str) -> Result<Vec<Value<'_>>> {
    Parser::new(input).parse_all()
}

/// Mint a fresh v4 (random) UUID. Output is canonical 8-4-4-4-12
/// hyphenated form when stringified, the only form wat-edn's `#uuid`
/// parser accepts (per RFC 9562 + the wat-edn strictness on round-trip
/// fidelity).
///
/// Pulls `uuid`'s `v4` capability and transitively the `getrandom` system
/// entropy source. Unconditional as of arc 296 — the `mint` feature was
/// removed; uuid generation is core to wat, not an opt-in.
///
/// # Example
///
/// ```
/// use wat_edn::{new_uuid_v4, parse, write, Value};
///
/// let id = new_uuid_v4();
/// let edn = write(&Value::Uuid(id));
/// let parsed = parse(&edn).expect("EDN must parse");
/// assert_eq!(parsed.as_uuid(), Some(&id));
/// ```
// Arc 092. The first consumer is `wat-measure`'s `WorkUnit`, which
// keys every measurement scope by uuid.
pub fn new_uuid_v4() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

/// Mint a deterministic v5 (SHA-1-based) UUID from a `namespace` UUID and a
/// `name` string. Output is canonical 8-4-4-4-12 hyphenated form when
/// stringified — always 36 chars, always lowercase, always hyphenated at
/// positions 8, 13, 18, 23.
///
/// Same inputs always produce the same output (deterministic). This makes v5
/// suitable for content addressing and hierarchical UUID derivation.
///
/// Pulls `uuid`'s `v5` capability. Unconditional as of arc 296 — the `mint`
/// feature was removed; uuid generation is core to wat, not an opt-in.
///
/// Arc 206 slice 1.5 — substrate promotion of `:wat::core::uuid::v5`.
///
/// # Example
///
/// ```
/// use wat_edn::new_uuid_v5;
/// use uuid::Uuid;
///
/// let ns = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
/// let id1 = new_uuid_v5(ns, "hello");
/// let id2 = new_uuid_v5(ns, "hello");
/// assert_eq!(id1, id2, "v5 is deterministic");
/// ```
pub fn new_uuid_v5(namespace: uuid::Uuid, name: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(&namespace, name.as_bytes())
}
