//! Canonical-EDN hashing + source-file verification.
//!
//! Two distinct responsibilities:
//!
//! 1. **AST identity.** [`canonical_edn_wat`] serializes a [`WatAST`] to
//!    deterministic bytes — same shape → same bytes. [`hash_canonical_ast`]
//!    SHA-256-hashes those bytes. Per FOUNDATION's cryptographic-
//!    provenance section, `hash(expanded AST) IS holon identity`;
//!    downstream caching / signing / content-addressed storage use
//!    this hash as the key.
//! 2. **Source-file verification.** [`verify_source_hash`] computes
//!    the hash of a loaded file's raw bytes and compares against the
//!    hex digest the user supplied in `(:wat/core/load! path (<algo>
//!    "hex"))`. Halts the pipeline if the hash doesn't match.
//!
//! # Hash algorithms
//!
//! `sha256` ships. `md5` and other algorithms aren't implemented in
//! this slice (the MD5 crate dependency is avoided; MD5 is
//! cryptographically broken and sha256 is the appropriate default).
//! Parsing accepts any algorithm name; verification dispatches and
//! returns [`HashError::UnsupportedAlgorithm`] for anything other
//! than sha256.
//!
//! # Hygiene-scope caveat
//!
//! Symbols in WatAST carry an [`Identifier`](crate::identifier::Identifier)
//! with a `BTreeSet<ScopeId>`. ScopeIds are monotonic u64s allocated
//! per-process by `fresh_scope()`, so TWO RUNS of the same program
//! produce different scope IDs, hence different canonical bytes and
//! different hashes. This is fine for AST IDENTITY WITHIN a single
//! process (the cache and symbol-table use case), but breaks the
//! cross-node / cross-run determinism claim.
//!
//! Slice 7b (real hygienic expansion with canonical scope numbering)
//! will address this: at hash time, renumber scope IDs in a canonical
//! order (e.g., first-appearance via DFS). Until then, canonical-EDN
//! is deterministic within a run but not across runs.

use crate::ast::WatAST;
use sha2::{Digest, Sha256};
use std::fmt;

/// Variant tags in canonical-EDN byte stream. Distinct per variant so
/// a `Keyword("foo")` cannot collide with a `StringLit("foo")`.
const TAG_INT: u8 = 0x10;
const TAG_FLOAT: u8 = 0x11;
const TAG_BOOL: u8 = 0x12;
const TAG_STRING: u8 = 0x13;
const TAG_KEYWORD: u8 = 0x14;
const TAG_SYMBOL: u8 = 0x15;
const TAG_LIST: u8 = 0x16;

/// Deterministic byte serialization of a WatAST tree.
///
/// Each node: variant tag byte, then variant-specific payload. Lengths
/// are u32 little-endian prefixes. Symbol scope sets are emitted in
/// sorted order (BTreeSet iteration is already sorted).
///
/// The hash of these bytes (via [`hash_canonical_ast`]) is the AST's
/// identity — used for content-addressed caching and the hash-is-
/// identity claim in FOUNDATION.
pub fn canonical_edn_wat(ast: &WatAST) -> Vec<u8> {
    let mut out = Vec::new();
    write_canonical_wat(ast, &mut out);
    out
}

fn write_canonical_wat(ast: &WatAST, out: &mut Vec<u8>) {
    match ast {
        WatAST::IntLit(n) => {
            out.push(TAG_INT);
            out.extend_from_slice(&n.to_le_bytes());
        }
        WatAST::FloatLit(x) => {
            out.push(TAG_FLOAT);
            out.extend_from_slice(&x.to_bits().to_le_bytes());
        }
        WatAST::BoolLit(b) => {
            out.push(TAG_BOOL);
            out.push(*b as u8);
        }
        WatAST::StringLit(s) => {
            out.push(TAG_STRING);
            let bytes = s.as_bytes();
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        }
        WatAST::Keyword(k) => {
            out.push(TAG_KEYWORD);
            let bytes = k.as_bytes();
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        }
        WatAST::Symbol(ident) => {
            out.push(TAG_SYMBOL);
            let name_bytes = ident.name.as_bytes();
            out.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(&(ident.scopes.len() as u32).to_le_bytes());
            for scope in &ident.scopes {
                out.extend_from_slice(&scope.0.to_le_bytes());
            }
        }
        WatAST::List(items) => {
            out.push(TAG_LIST);
            out.extend_from_slice(&(items.len() as u32).to_le_bytes());
            for child in items {
                write_canonical_wat(child, out);
            }
        }
    }
}

/// Hash a WatAST tree via canonical-EDN + SHA-256.
///
/// Returns a 32-byte digest. Two ASTs of the same shape produce the
/// same digest (within a single process, subject to the scope-ID
/// caveat named in the module doc).
pub fn hash_canonical_ast(ast: &WatAST) -> [u8; 32] {
    let bytes = canonical_edn_wat(ast);
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Hash source-file bytes with the named algorithm and compare to the
/// hex-encoded expected digest. Used by `(:wat/core/load! path
/// (<algo> "hex"))` verification.
///
/// Supported algorithms: `sha256`. Any other algorithm name returns
/// [`HashError::UnsupportedAlgorithm`]; add more as needed.
pub fn verify_source_hash(
    source: &[u8],
    algo: &str,
    expected_hex: &str,
) -> Result<(), HashError> {
    let actual_hex = match algo {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(source);
            let digest = hasher.finalize();
            hex_encode(&digest)
        }
        other => {
            return Err(HashError::UnsupportedAlgorithm {
                algo: other.to_string(),
            });
        }
    };
    // Case-insensitive hex comparison.
    if actual_hex.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(HashError::Mismatch {
            algo: algo.to_string(),
            expected: expected_hex.to_string(),
            actual: actual_hex,
        })
    }
}

/// Hex-encode a byte slice (lowercase, no separators).
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(hex_digit(b >> 4));
        out.push(hex_digit(b & 0x0F));
    }
    out
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + nibble - 10) as char,
        _ => unreachable!(),
    }
}

/// Errors from hash / verification operations.
#[derive(Debug, Clone, PartialEq)]
pub enum HashError {
    UnsupportedAlgorithm { algo: String },
    Mismatch {
        algo: String,
        expected: String,
        actual: String,
    },
    /// A cryptographic signature was requested but the slice doesn't
    /// implement verification (ed25519 / RSA). Parse-accepted; runtime
    /// refuses.
    SignedNotImplemented,
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::UnsupportedAlgorithm { algo } => write!(
                f,
                "unsupported hash algorithm {:?} — this build supports sha256",
                algo
            ),
            HashError::Mismatch {
                algo,
                expected,
                actual,
            } => write!(
                f,
                "{} mismatch: expected {}, got {}",
                algo, expected, actual
            ),
            HashError::SignedNotImplemented => write!(
                f,
                "(signed ...) verification is parse-accepted but not yet implemented in this build; use (sha256 ...) for tamper-detection"
            ),
        }
    }
}

impl std::error::Error for HashError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{fresh_scope, Identifier};
    use crate::parser::parse_one;

    fn parse(src: &str) -> WatAST {
        parse_one(src).expect("parse ok")
    }

    // ─── canonical_edn_wat determinism ──────────────────────────────────

    #[test]
    fn same_ast_same_bytes() {
        let a = parse(r#"(:wat/algebra/Atom "x")"#);
        let b = parse(r#"(:wat/algebra/Atom "x")"#);
        assert_eq!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    #[test]
    fn different_ast_different_bytes() {
        let a = parse(r#"(:wat/algebra/Atom "x")"#);
        let b = parse(r#"(:wat/algebra/Atom "y")"#);
        assert_ne!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    #[test]
    fn variant_discrimination() {
        // (Atom "42") ≠ (Atom 42) — string vs int, same printable bytes.
        let a = parse(r#"(:wat/algebra/Atom "42")"#);
        let b = parse(r#"(:wat/algebra/Atom 42)"#);
        assert_ne!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    #[test]
    fn keyword_vs_string_distinct() {
        let kw = parse(":foo");
        let str_form = parse(r#""foo""#);
        assert_ne!(canonical_edn_wat(&kw), canonical_edn_wat(&str_form));
    }

    #[test]
    fn nested_list_preserves_shape() {
        let a = parse("(a (b c))");
        let b = parse("((a b) c)");
        assert_ne!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    // ─── hash_canonical_ast ─────────────────────────────────────────────

    #[test]
    fn hash_is_32_bytes() {
        let a = parse(r#"(:wat/algebra/Atom "x")"#);
        assert_eq!(hash_canonical_ast(&a).len(), 32);
    }

    #[test]
    fn same_ast_same_hash() {
        let a = parse(r#"(:wat/algebra/Bind (:wat/algebra/Atom "r") (:wat/algebra/Atom "f"))"#);
        let b = parse(r#"(:wat/algebra/Bind (:wat/algebra/Atom "r") (:wat/algebra/Atom "f"))"#);
        assert_eq!(hash_canonical_ast(&a), hash_canonical_ast(&b));
    }

    #[test]
    fn different_ast_different_hash() {
        let a = parse(r#"(:wat/algebra/Bind (:wat/algebra/Atom "r") (:wat/algebra/Atom "f"))"#);
        let b = parse(r#"(:wat/algebra/Bind (:wat/algebra/Atom "f") (:wat/algebra/Atom "r"))"#);
        assert_ne!(hash_canonical_ast(&a), hash_canonical_ast(&b));
    }

    // ─── Symbol scope impact on hash ────────────────────────────────────

    #[test]
    fn symbol_with_different_scopes_hashes_differently() {
        let bare = WatAST::Symbol(Identifier::bare("tmp"));
        let scoped = WatAST::Symbol(Identifier::bare("tmp").add_scope(fresh_scope()));
        assert_ne!(hash_canonical_ast(&bare), hash_canonical_ast(&scoped));
    }

    // ─── Source-file verification ───────────────────────────────────────

    #[test]
    fn sha256_verify_matches() {
        let source = b"hello world";
        // Pre-computed sha256("hello world").
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_source_hash(source, "sha256", expected).is_ok());
    }

    #[test]
    fn sha256_verify_case_insensitive() {
        let source = b"hello world";
        let upper = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9";
        assert!(verify_source_hash(source, "sha256", upper).is_ok());
    }

    #[test]
    fn sha256_verify_mismatch() {
        let source = b"hello world";
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let err = verify_source_hash(source, "sha256", wrong).unwrap_err();
        assert!(matches!(err, HashError::Mismatch { .. }));
    }

    #[test]
    fn unsupported_algorithm_rejected() {
        let source = b"hello world";
        let err = verify_source_hash(source, "md5", "abc").unwrap_err();
        assert!(matches!(err, HashError::UnsupportedAlgorithm { .. }));
    }

    // ─── hex_encode ─────────────────────────────────────────────────────

    #[test]
    fn hex_encode_basic() {
        assert_eq!(hex_encode(&[0x00, 0xFF, 0xAB, 0x12]), "00ffab12");
    }

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }
}
