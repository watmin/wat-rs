//! Canonical-EDN hashing + cryptographic verification.
//!
//! Three distinct responsibilities:
//!
//! 1. **AST identity.** [`canonical_edn_wat`] serializes a [`WatAST`] to
//!    deterministic bytes — same shape → same bytes. [`hash_canonical_ast`]
//!    SHA-256-hashes those bytes. Per FOUNDATION's cryptographic-
//!    provenance section, `hash(expanded AST) IS holon identity`;
//!    downstream caching / signing / content-addressed storage use
//!    this hash as the key.
//! 2. **Program identity.** [`canonical_edn_program`] handles a flat
//!    `&[WatAST]` (the output of load+expand). A distinct tag byte
//!    prevents collision with a single top-level list of the same
//!    children. [`hash_canonical_program`] SHA-256-hashes those bytes.
//! 3. **Source-file integrity.** [`verify_source_hash`] computes the
//!    hash of a loaded file's raw bytes and compares against the hex
//!    digest the user supplied in `(:wat::core::load! path (<algo>
//!    "hex"))`. Halts the pipeline if the hash doesn't match. This is
//!    file-bytes integrity — "did the bytes on disk change?"
//! 4. **Semantic provenance.** [`verify_ast_signature`] and
//!    [`verify_program_signature`] verify an Ed25519 signature over the
//!    SHA-256 of the canonical-EDN form of a parsed AST. This is
//!    meaning-level provenance — "was this AST authored by the holder
//!    of this private key?" Robust to comment / whitespace changes.
//!
//! # Algorithm pluggability
//!
//! Both hash and signature verification dispatch on an algorithm name
//! baked into the source form. Today `sha256` and `ed25519` are
//! implemented; every other name returns an `Unsupported*` error and
//! halts startup. Add new algorithm arms as deployment needs dictate.
//!
//! # Signing vs hashing — two roles
//!
//! - `(sha256 "hex")` answers **"file bytes unchanged"** (disk
//!   integrity). Hashes RAW SOURCE BYTES.
//! - `(signed ed25519 "b64-sig" "b64-pubkey")` answers **"AST
//!   authored by holder of private key"** (semantic provenance). Signs
//!   SHA-256 of canonical-EDN.
//!
//! The modes have different semantic targets and different trust
//! models. They are complementary, not alternatives.
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
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fmt;

/// Variant tags in canonical-EDN byte stream. Distinct per variant so
/// a `Keyword("foo")` cannot collide with a `StringLit("foo")`. The
/// `PROGRAM` tag discriminates a multi-form program from a single
/// top-level list with the same children.
const TAG_INT: u8 = 0x10;
const TAG_FLOAT: u8 = 0x11;
const TAG_BOOL: u8 = 0x12;
const TAG_STRING: u8 = 0x13;
const TAG_KEYWORD: u8 = 0x14;
const TAG_SYMBOL: u8 = 0x15;
const TAG_LIST: u8 = 0x16;
const TAG_PROGRAM: u8 = 0x17;

/// Ed25519 signature length in bytes.
const ED25519_SIG_LEN: usize = 64;
/// Ed25519 public key length in bytes.
const ED25519_PUBKEY_LEN: usize = 32;

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

/// Deterministic byte serialization of a multi-form program.
///
/// Emits `TAG_PROGRAM` + form count + each form's canonical bytes. The
/// distinct tag byte prevents collision with a single top-level list
/// that happens to have the same children: a program of `[A, B, C]`
/// and a single form `(A B C)` produce different bytes.
pub fn canonical_edn_program(forms: &[WatAST]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(TAG_PROGRAM);
    out.extend_from_slice(&(forms.len() as u32).to_le_bytes());
    for f in forms {
        write_canonical_wat(f, &mut out);
    }
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
    sha256_digest(&canonical_edn_wat(ast))
}

/// Hash a flat form list (a program) via canonical-EDN + SHA-256.
pub fn hash_canonical_program(forms: &[WatAST]) -> [u8; 32] {
    sha256_digest(&canonical_edn_program(forms))
}

fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Hash source-file bytes with the named algorithm and compare to the
/// hex-encoded expected digest. Used by `(:wat::core::load! path
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
        "sha256" => hex_encode(&sha256_digest(source)),
        other => {
            return Err(HashError::UnsupportedAlgorithm {
                algo: other.to_string(),
            });
        }
    };
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

/// Verify a signature over a single WatAST.
///
/// Computes the SHA-256 of the canonical-EDN of `ast` and dispatches
/// to the named signature algorithm with base64-decoded sig + pub-key.
/// Used for per-file signatures inside `(:wat::core::load! path (signed
/// <algo> <sig> <pubkey>))`.
///
/// Supported algorithms: `ed25519`. Any other name returns
/// [`HashError::UnsupportedSignatureAlgorithm`].
pub fn verify_ast_signature(
    ast: &WatAST,
    algo: &str,
    sig_b64: &str,
    pubkey_b64: &str,
) -> Result<(), HashError> {
    let hash = hash_canonical_ast(ast);
    verify_hash_signature(&hash, algo, sig_b64, pubkey_b64)
}

/// Verify a signature over a flat form list.
///
/// Computes the SHA-256 of the canonical-EDN of `forms` and dispatches
/// to the named signature algorithm. Used per-form by
/// `(:wat::core::signed-load! ...)` (the payload is a loaded file's
/// parsed form list) and `(:wat::core::eval-signed! ...)` (the payload
/// is a runtime AST resolved from the source interface). Each call
/// verifies one form's provenance independently.
///
/// There is no CLI-level invocation of this function; signature
/// verification is per-form by design — a program is a collection of
/// forms with independent provenance needs, and one CLI-level
/// signature cannot cover that structure. See FOUNDATION's
/// cryptographic-provenance section.
pub fn verify_program_signature(
    forms: &[WatAST],
    algo: &str,
    sig_b64: &str,
    pubkey_b64: &str,
) -> Result<(), HashError> {
    let hash = hash_canonical_program(forms);
    verify_hash_signature(&hash, algo, sig_b64, pubkey_b64)
}

fn verify_hash_signature(
    message: &[u8],
    algo: &str,
    sig_b64: &str,
    pubkey_b64: &str,
) -> Result<(), HashError> {
    match algo {
        "ed25519" => verify_ed25519(message, sig_b64, pubkey_b64),
        other => Err(HashError::UnsupportedSignatureAlgorithm {
            algo: other.to_string(),
        }),
    }
}

fn verify_ed25519(
    message: &[u8],
    sig_b64: &str,
    pubkey_b64: &str,
) -> Result<(), HashError> {
    let sig_bytes = B64
        .decode(sig_b64.as_bytes())
        .map_err(|e| HashError::InvalidBase64 {
            field: "signature",
            reason: e.to_string(),
        })?;
    let pk_bytes = B64
        .decode(pubkey_b64.as_bytes())
        .map_err(|e| HashError::InvalidBase64 {
            field: "pub_key",
            reason: e.to_string(),
        })?;

    if sig_bytes.len() != ED25519_SIG_LEN {
        return Err(HashError::InvalidSignatureLength {
            algo: "ed25519".into(),
            expected: ED25519_SIG_LEN,
            got: sig_bytes.len(),
        });
    }
    if pk_bytes.len() != ED25519_PUBKEY_LEN {
        return Err(HashError::InvalidPubKeyLength {
            algo: "ed25519".into(),
            expected: ED25519_PUBKEY_LEN,
            got: pk_bytes.len(),
        });
    }

    // Length-checked just above; array conversion cannot fail.
    let sig_arr: [u8; ED25519_SIG_LEN] = sig_bytes.as_slice().try_into().unwrap();
    let pk_arr: [u8; ED25519_PUBKEY_LEN] = pk_bytes.as_slice().try_into().unwrap();

    let signature = Signature::from_bytes(&sig_arr);
    let verifying_key =
        VerifyingKey::from_bytes(&pk_arr).map_err(|e| HashError::InvalidPubKey {
            algo: "ed25519".into(),
            reason: e.to_string(),
        })?;

    // verify_strict rejects small-order R components (malleability
    // resistance) — stricter than verify() and the RFC 8032 default.
    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| HashError::SignatureMismatch {
            algo: "ed25519".into(),
        })
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
    UnsupportedAlgorithm {
        algo: String,
    },
    Mismatch {
        algo: String,
        expected: String,
        actual: String,
    },
    UnsupportedSignatureAlgorithm {
        algo: String,
    },
    InvalidBase64 {
        field: &'static str,
        reason: String,
    },
    InvalidSignatureLength {
        algo: String,
        expected: usize,
        got: usize,
    },
    InvalidPubKeyLength {
        algo: String,
        expected: usize,
        got: usize,
    },
    InvalidPubKey {
        algo: String,
        reason: String,
    },
    SignatureMismatch {
        algo: String,
    },
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
            HashError::UnsupportedSignatureAlgorithm { algo } => write!(
                f,
                "unsupported signature algorithm {:?} — this build supports ed25519",
                algo
            ),
            HashError::InvalidBase64 { field, reason } => {
                write!(f, "{} is not valid base64: {}", field, reason)
            }
            HashError::InvalidSignatureLength {
                algo,
                expected,
                got,
            } => write!(
                f,
                "{} signature length mismatch: expected {} bytes, got {}",
                algo, expected, got
            ),
            HashError::InvalidPubKeyLength {
                algo,
                expected,
                got,
            } => write!(
                f,
                "{} public key length mismatch: expected {} bytes, got {}",
                algo, expected, got
            ),
            HashError::InvalidPubKey { algo, reason } => {
                write!(f, "{} public key rejected: {}", algo, reason)
            }
            HashError::SignatureMismatch { algo } => {
                write!(f, "{} signature verification failed", algo)
            }
        }
    }
}

impl std::error::Error for HashError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{fresh_scope, Identifier};
    use crate::parser::{parse_all, parse_one};
    use ed25519_dalek::{Signer, SigningKey};

    fn parse(src: &str) -> WatAST {
        parse_one(src).expect("parse ok")
    }

    /// Fixed 32-byte seed for deterministic test keypair. Any bytes
    /// work; Ed25519 derives pub-key and private scalars from the seed.
    const TEST_SEED: [u8; 32] = [
        0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c,
        0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae,
        0x7f, 0x60,
    ];

    fn test_signing_key() -> SigningKey {
        SigningKey::from_bytes(&TEST_SEED)
    }

    fn other_signing_key() -> SigningKey {
        let mut seed = TEST_SEED;
        seed[0] ^= 0xFF;
        SigningKey::from_bytes(&seed)
    }

    fn b64(bytes: &[u8]) -> String {
        B64.encode(bytes)
    }

    // ─── canonical_edn_wat determinism ──────────────────────────────────

    #[test]
    fn same_ast_same_bytes() {
        let a = parse(r#"(:wat::algebra::Atom "x")"#);
        let b = parse(r#"(:wat::algebra::Atom "x")"#);
        assert_eq!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    #[test]
    fn different_ast_different_bytes() {
        let a = parse(r#"(:wat::algebra::Atom "x")"#);
        let b = parse(r#"(:wat::algebra::Atom "y")"#);
        assert_ne!(canonical_edn_wat(&a), canonical_edn_wat(&b));
    }

    #[test]
    fn variant_discrimination() {
        let a = parse(r#"(:wat::algebra::Atom "42")"#);
        let b = parse(r#"(:wat::algebra::Atom 42)"#);
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

    // ─── canonical_edn_program ──────────────────────────────────────────

    #[test]
    fn program_tag_discriminates_from_list() {
        // A program of [A, B, C] must not collide with a single list (A B C).
        let forms = parse_all("a b c").unwrap();
        let single = parse("(a b c)");
        assert_ne!(canonical_edn_program(&forms), canonical_edn_wat(&single));
    }

    #[test]
    fn program_deterministic() {
        let f1 = parse_all(r#"(:wat::algebra::Atom "x") (:wat::algebra::Atom "y")"#).unwrap();
        let f2 = parse_all(r#"(:wat::algebra::Atom "x") (:wat::algebra::Atom "y")"#).unwrap();
        assert_eq!(canonical_edn_program(&f1), canonical_edn_program(&f2));
    }

    #[test]
    fn program_order_matters() {
        let f1 = parse_all(r#"(:wat::algebra::Atom "x") (:wat::algebra::Atom "y")"#).unwrap();
        let f2 = parse_all(r#"(:wat::algebra::Atom "y") (:wat::algebra::Atom "x")"#).unwrap();
        assert_ne!(canonical_edn_program(&f1), canonical_edn_program(&f2));
    }

    // ─── hash_canonical_ast / _program ──────────────────────────────────

    #[test]
    fn hash_is_32_bytes() {
        let a = parse(r#"(:wat::algebra::Atom "x")"#);
        assert_eq!(hash_canonical_ast(&a).len(), 32);
        let forms = parse_all(r#"(:wat::algebra::Atom "x")"#).unwrap();
        assert_eq!(hash_canonical_program(&forms).len(), 32);
    }

    #[test]
    fn same_ast_same_hash() {
        let a = parse(r#"(:wat::algebra::Bind (:wat::algebra::Atom "r") (:wat::algebra::Atom "f"))"#);
        let b = parse(r#"(:wat::algebra::Bind (:wat::algebra::Atom "r") (:wat::algebra::Atom "f"))"#);
        assert_eq!(hash_canonical_ast(&a), hash_canonical_ast(&b));
    }

    #[test]
    fn different_ast_different_hash() {
        let a = parse(r#"(:wat::algebra::Bind (:wat::algebra::Atom "r") (:wat::algebra::Atom "f"))"#);
        let b = parse(r#"(:wat::algebra::Bind (:wat::algebra::Atom "f") (:wat::algebra::Atom "r"))"#);
        assert_ne!(hash_canonical_ast(&a), hash_canonical_ast(&b));
    }

    // ─── Symbol scope impact on hash ────────────────────────────────────

    #[test]
    fn symbol_with_different_scopes_hashes_differently() {
        let bare = WatAST::Symbol(Identifier::bare("tmp"));
        let scoped = WatAST::Symbol(Identifier::bare("tmp").add_scope(fresh_scope()));
        assert_ne!(hash_canonical_ast(&bare), hash_canonical_ast(&scoped));
    }

    // ─── Source-file hash verification ──────────────────────────────────

    #[test]
    fn sha256_verify_matches() {
        let source = b"hello world";
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
    fn unsupported_hash_algorithm_rejected() {
        let source = b"hello world";
        let err = verify_source_hash(source, "md5", "abc").unwrap_err();
        assert!(matches!(err, HashError::UnsupportedAlgorithm { .. }));
    }

    // ─── Ed25519 AST signature — round trip + tamper ───────────────────

    #[test]
    fn ed25519_ast_round_trip() {
        let sk = test_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "payload")"#);
        let hash = hash_canonical_ast(&ast);
        let sig = sk.sign(&hash);
        let sig_b64 = b64(&sig.to_bytes());
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        verify_ast_signature(&ast, "ed25519", &sig_b64, &pk_b64).expect("verify ok");
    }

    #[test]
    fn ed25519_ast_tampered_rejected() {
        let sk = test_signing_key();
        let authored = parse(r#"(:wat::algebra::Atom "authored")"#);
        let sig = sk.sign(&hash_canonical_ast(&authored));
        let sig_b64 = b64(&sig.to_bytes());
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        // Verify against a DIFFERENT AST — signature must not match.
        let tampered = parse(r#"(:wat::algebra::Atom "tampered")"#);
        let err = verify_ast_signature(&tampered, "ed25519", &sig_b64, &pk_b64).unwrap_err();
        assert!(matches!(err, HashError::SignatureMismatch { .. }));
    }

    #[test]
    fn ed25519_ast_wrong_pubkey_rejected() {
        let signer = test_signing_key();
        let other = other_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let sig = signer.sign(&hash_canonical_ast(&ast));
        let sig_b64 = b64(&sig.to_bytes());
        // Verify with the WRONG pub-key.
        let pk_b64 = b64(other.verifying_key().as_bytes());
        let err = verify_ast_signature(&ast, "ed25519", &sig_b64, &pk_b64).unwrap_err();
        assert!(matches!(err, HashError::SignatureMismatch { .. }));
    }

    // ─── Ed25519 program signature — round trip + tamper ───────────────

    #[test]
    fn ed25519_program_round_trip() {
        let sk = test_signing_key();
        let forms =
            parse_all(r#"(:wat::algebra::Atom "a") (:wat::algebra::Atom "b")"#).unwrap();
        let hash = hash_canonical_program(&forms);
        let sig = sk.sign(&hash);
        let sig_b64 = b64(&sig.to_bytes());
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        verify_program_signature(&forms, "ed25519", &sig_b64, &pk_b64).expect("verify ok");
    }

    #[test]
    fn ed25519_program_tampered_rejected() {
        let sk = test_signing_key();
        let authored = parse_all(r#"(:wat::algebra::Atom "a")"#).unwrap();
        let sig = sk.sign(&hash_canonical_program(&authored));
        let sig_b64 = b64(&sig.to_bytes());
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        // A program with an extra form is NOT the signed program.
        let tampered =
            parse_all(r#"(:wat::algebra::Atom "a") (:wat::algebra::Atom "injected")"#).unwrap();
        let err = verify_program_signature(&tampered, "ed25519", &sig_b64, &pk_b64).unwrap_err();
        assert!(matches!(err, HashError::SignatureMismatch { .. }));
    }

    // ─── Ed25519 input validation ──────────────────────────────────────

    #[test]
    fn ed25519_invalid_base64_sig() {
        let sk = test_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        let err = verify_ast_signature(&ast, "ed25519", "not valid base64!!!", &pk_b64)
            .unwrap_err();
        assert!(matches!(err, HashError::InvalidBase64 { field: "signature", .. }));
    }

    #[test]
    fn ed25519_invalid_base64_pubkey() {
        let sk = test_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let sig = sk.sign(&hash_canonical_ast(&ast));
        let sig_b64 = b64(&sig.to_bytes());
        let err =
            verify_ast_signature(&ast, "ed25519", &sig_b64, "not valid base64!!!").unwrap_err();
        assert!(matches!(err, HashError::InvalidBase64 { field: "pub_key", .. }));
    }

    #[test]
    fn ed25519_wrong_signature_length() {
        let sk = test_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let pk_b64 = b64(sk.verifying_key().as_bytes());
        // 10-byte "signature" — valid base64, wrong length.
        let short_sig = b64(&[0u8; 10]);
        let err = verify_ast_signature(&ast, "ed25519", &short_sig, &pk_b64).unwrap_err();
        assert!(matches!(
            err,
            HashError::InvalidSignatureLength { expected: 64, got: 10, .. }
        ));
    }

    #[test]
    fn ed25519_wrong_pubkey_length() {
        let sk = test_signing_key();
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let sig = sk.sign(&hash_canonical_ast(&ast));
        let sig_b64 = b64(&sig.to_bytes());
        let short_pk = b64(&[0u8; 8]);
        let err = verify_ast_signature(&ast, "ed25519", &sig_b64, &short_pk).unwrap_err();
        assert!(matches!(
            err,
            HashError::InvalidPubKeyLength { expected: 32, got: 8, .. }
        ));
    }

    #[test]
    fn unsupported_signature_algorithm_rejected() {
        let ast = parse(r#"(:wat::algebra::Atom "x")"#);
        let dummy_sig = b64(&[0u8; 64]);
        let dummy_pk = b64(&[0u8; 32]);
        let err = verify_ast_signature(&ast, "rsa", &dummy_sig, &dummy_pk).unwrap_err();
        assert!(matches!(
            err,
            HashError::UnsupportedSignatureAlgorithm { .. }
        ));
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
