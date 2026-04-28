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
//!   emits `#wat-edn.float/nan nil` / `#wat-edn.float/inf nil` /
//!   `#wat-edn.float/neg-inf nil` so `f64` round-trips losslessly. Other
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
pub mod escapes;
pub mod lexer;
pub mod parser;
pub mod value;
pub mod writer;

// ─── Public surface ─────────────────────────────────────────────
//
// Everything a caller needs — parse / parse_owned / parse_all,
// the OwnedValue alias, the canonical types, and the writer
// helpers — is re-exported here so a downstream `use wat_edn::*`
// reaches the whole API in one line.

pub use error::{Error, ErrorKind, Result};
pub use parser::Parser;
pub use value::{Keyword, Symbol, Tag, Value};
pub use writer::{write, write_to};

/// A `Value` that owns all of its string data — no input-buffer
/// lifetime to track. Storable across threads, returnable from
/// functions, persistable beyond the parsed source. Equivalent to
/// `Value<'static>`; the alias gives the storage-crossing case a
/// name. Returned by [`Value::into_owned`] and [`parse_owned`].
pub type OwnedValue = Value<'static>;

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
