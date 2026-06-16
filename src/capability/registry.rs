//! Arc 272 — the capability narrow-waist (`wat-edn.cap`).
//!
//! A **portable capability** is a substrate value whose wire content is meaningful + safe across a
//! process boundary (vs an ordinary opaque handle — an fd, a `Sender` — which must NOT cross). This
//! module is the **frozen waist**: a registry of `PortableCapability` codecs + two generic dispatch
//! fns (`encode_capability` / `decode_capability`) that `edn_shim` calls. Adding a new capability is a
//! `CapCodec` row in [`registry`] (append-only) — `edn_shim`'s dispatch never changes. The rigidity of
//! the waist is what enables unbounded capabilities above it (the hourglass / narrow-waist law).
//!
//! The trust gating lives at the door (`edn_shim::decode_trusted_wire`): `decode_capability` is reached
//! ONLY off the trusted peer wire — a capability is handed over a lineage channel, never forged from
//! parsed data (object-capability transfer-only).

use crate::edn_shim::{EdnReadError, EdnReadErrorKind};
use crate::rust_deps::marshal::RustOpaqueInner;
use crate::runtime::Value;
use crate::span::Span;
use crate::types::TypeEnv;
use std::sync::OnceLock;
use wat_edn::{OwnedValue, Tag};

/// A registered portable capability — the codec that crosses the `wat-edn.cap` waist.
pub struct CapCodec {
    /// The `#wat-edn.cap/<name>` tag name — the DECODE key.
    pub name: &'static str,
    /// The `RustOpaque` `type_path` this codec encodes — the ENCODE key.
    pub type_path: &'static str,
    /// Encode the opaque's PORTABLE form to a wire body. `None` when this opaque instance has no
    /// portable form (e.g. a thread-tier `Address'`, whose `Sender` cannot cross) → the caller falls
    /// back to the non-portable `wat-edn.opaque` tag (refused on decode). `types` provides the type
    /// registry so record codecs can encode named fields.
    pub encode: fn(&RustOpaqueInner, &TypeEnv) -> Option<OwnedValue>,
    /// Reconstruct a live capability `Value` from a wire body (called only off the trusted door).
    /// `types` provides the type registry so record codecs can decode named fields.
    pub decode: fn(&OwnedValue, &TypeEnv) -> Result<Value, EdnReadError>,
}

/// THE registry — built once. **Adding a capability = a `CapCodec` row here (append-only); the
/// `edn_shim` dispatch (the waist) never changes.** This central row is the open EDGE; the frozen
/// WAIST is the two generic dispatch fns + the wire contract.
fn registry() -> &'static [CapCodec] {
    // rune:perspicere(read-once) — built once at registry init; a single-use CapRegistry alias would read worse
    static REG: OnceLock<Vec<CapCodec>> = OnceLock::new();
    REG.get_or_init(|| {
        vec![
            address_codec(),
            // ◄── future portable capabilities register HERE. Zero edit to edn_shim's dispatch.
        ]
    })
}

/// Generic ENCODE dispatch — called by `edn_shim`'s single `RustOpaque` arm. Returns the
/// `#wat-edn.cap/<name>` tag when `inner` is a registered portable capability WITH a portable form;
/// `None` otherwise (→ the caller emits the non-portable opaque tag).
pub fn encode_capability(inner: &RustOpaqueInner, types: &TypeEnv) -> Option<OwnedValue> {
    encode_in(registry(), inner, types)
}

/// Generic DECODE dispatch — called by `edn_shim`'s `wat-edn.cap` tag arm, which is reached ONLY off
/// the trusted door (`decode_trusted_wire`). An unregistered name is refused.
pub fn decode_capability(name: &str, body: &OwnedValue, types: &TypeEnv) -> Result<Value, EdnReadError> {
    decode_in(registry(), name, body, types)
}

/// The encode dispatch over an EXPLICIT codec set. Identical for N capabilities — a linear find by
/// `type_path`. Split out so the waist's N-capability behaviour is directly testable (the strike-2
/// proof passes a 2-codec slice); `encode_capability` is just `encode_in(registry(), …)`.
fn encode_in(caps: &[CapCodec], inner: &RustOpaqueInner, types: &TypeEnv) -> Option<OwnedValue> {
    let codec = caps.iter().find(|c| c.type_path == inner.type_path)?;
    let body = (codec.encode)(inner, types)?;
    Some(OwnedValue::Tagged(Tag::ns("wat-edn.cap", codec.name), Box::new(body)))
}

/// Construct a capability-decode error. Decode reconstructs off the trusted peer wire from an
/// `OwnedValue` body, which carries no source position — so the span is legitimately unknown,
/// attested here ONCE rather than filled by silent convention at each call site (a bare
/// `Span::unknown()` reads identically to a discarded-span bug; this names why it is not one).
// rune:conformare(spanless-by-domain) — capability decode reconstructs off the trusted wire from an
// OwnedValue body that carries no source location; consumers of EdnReadError from decode_capability
// do not expect a span.
fn cap_decode_error(reason: impl Into<String>) -> EdnReadError {
    EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnsupportedTag(reason.into()) }
}

/// The decode dispatch over an EXPLICIT codec set — a linear find by tag `name`.
fn decode_in(caps: &[CapCodec], name: &str, body: &OwnedValue, types: &TypeEnv) -> Result<Value, EdnReadError> {
    let codec = caps
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| cap_decode_error(format!("wat-edn.cap/{name}")))?;
    (codec.decode)(body, types)
}

// ─── Registrants ──────────────────────────────────────────────────────────────

/// `Address'` (arc 272 6c.2 D1) — the kernel-minted abstract UDS address. The first inhabitant of
/// the waist. Portable as a process-tier socket address: carries the minter pid + name bytes as a
/// registered `SocketAddressWire` base record (the general record encode/decode path handles
/// field naming). A thread-tier `Address'` (a crossbeam `Sender`) has no portable form →
/// `encode` returns `None`.
fn address_codec() -> CapCodec {
    CapCodec {
        name: "address",
        type_path: crate::kernel::spawn::ADDRESS_TYPE_PATH,
        encode: |inner, types| {
            let addr = inner.payload.downcast_ref::<crate::kernel::address::Address>()?;
            let (minter_pid, name_bytes) = addr.portable_form()?;
            // Build a SocketAddressWire record: struct_form = [minter_pid, name as Vec<i64>].
            // Value::Vec is :wat::core::Vector<T> at runtime (runtime.rs:6334).
            let name_vec = Value::Vec(std::sync::Arc::new(
                name_bytes.into_iter().map(|b| Value::i64(b as i64)).collect(),
            ));
            let record = Value::wat__Record {
                class_fqdn: std::sync::Arc::new("wat::kernel::SocketAddressWire".into()),
                struct_form: std::sync::Arc::new(vec![Value::i64(minter_pid as i64), name_vec]),
            };
            Some(crate::edn_shim::value_to_edn_with(&record, Some(types)))
        },
        decode: |body, types| {
            // body is the OwnedValue body of #wat-edn.cap/address — expected to be a
            // #wat.kernel/SocketAddressWire tagged map (as produced by value_to_edn_with on the
            // SocketAddressWire record).
            let record_val = crate::edn_shim::edn_to_value(body, Some(types)).map_err(|_| {
                cap_decode_error("wat-edn.cap/address (body failed edn_to_value)")
            })?;
            let (class_fqdn, struct_form) = match record_val {
                Value::wat__Record { ref class_fqdn, ref struct_form } => {
                    (class_fqdn.clone(), struct_form.clone())
                }
                _ => {
                    return Err(cap_decode_error(
                        "wat-edn.cap/address (expected a SocketAddressWire record)",
                    ))
                }
            };
            if class_fqdn.as_str() != "wat::kernel::SocketAddressWire" {
                return Err(cap_decode_error(format!(
                    "wat-edn.cap/address (wrong record class: {})",
                    class_fqdn
                )));
            }
            if struct_form.len() != 2 {
                return Err(cap_decode_error(
                    "wat-edn.cap/address (SocketAddressWire must have 2 fields)",
                ));
            }
            // Field 0: minter-pid (i64)
            let minter_pid = match &struct_form[0] {
                Value::i64(n) => *n as i32,
                _ => {
                    return Err(cap_decode_error(
                        "wat-edn.cap/address (minter-pid field must be i64)",
                    ))
                }
            };
            // Field 1: name (Vector<i64> = Value::Vec of Value::i64)
            let name_bytes_vals = match &struct_form[1] {
                Value::Vec(xs) => xs.clone(),
                _ => {
                    return Err(cap_decode_error(
                        "wat-edn.cap/address (name field must be Vector<i64>)",
                    ))
                }
            };
            // Cap the decoded name at the abstract-UDS limit (sun_path[1..] ≤ 107 bytes).
            const ABSTRACT_UDS_NAME_MAX: usize = 107;
            if name_bytes_vals.is_empty() {
                return Err(cap_decode_error(
                    "wat-edn.cap/address (empty name — a minted abstract name is never zero-length)",
                ));
            }
            if name_bytes_vals.len() > ABSTRACT_UDS_NAME_MAX {
                return Err(cap_decode_error(format!(
                    "wat-edn.cap/address (name {} bytes exceeds the {}-byte abstract-UDS limit)",
                    name_bytes_vals.len(),
                    ABSTRACT_UDS_NAME_MAX
                )));
            }
            let mut name_bytes = Vec::with_capacity(name_bytes_vals.len());
            for v in name_bytes_vals.iter() {
                match v {
                    Value::i64(n) if (0..=255).contains(n) => name_bytes.push(*n as u8),
                    _ => {
                        return Err(cap_decode_error(
                            "wat-edn.cap/address (name byte out of 0..=255)",
                        ))
                    }
                }
            }
            let addr = crate::kernel::address::Address::from_socket_name_bytes(name_bytes, minter_pid);
            Ok(crate::rust_deps::marshal::make_rust_opaque(
                crate::kernel::spawn::ADDRESS_TYPE_PATH,
                addr,
            ))
        },
    }
}

#[cfg(test)]
mod waist_proof {
    //! Arc 272 narrow-waist STRIKE 2 — the proof. A SECOND capability round-trips through the SAME
    //! generic dispatch that carries `Address'`, with `edn_shim`'s core UNTOUCHED — the entire diff
    //! for capability #2 is one `CapCodec`. That is the waist working: N capabilities, one frozen core.
    use super::*;
    use crate::runtime::Value;
    use crate::rust_deps::marshal::make_rust_opaque;

    /// A toy second capability: `:test::Token` over a `u64`. Real shape, trivial payload.
    fn toy_token_codec() -> CapCodec {
        CapCodec {
            name: "test-token",
            type_path: ":test::Token",
            encode: |inner, _types| {
                Some(OwnedValue::Integer(*inner.payload.downcast_ref::<u64>()? as i64))
            },
            decode: |body, _types| match body {
                OwnedValue::Integer(n) => Ok(make_rust_opaque(":test::Token", *n as u64)),
                // Route through the SAME attested spanless helper the production codecs use, so
                // span-omission is ONE runed path, not a parallel hand-built struct literal.
                _ => Err(cap_decode_error("wat-edn.cap/test-token (expected an integer)")),
            },
        }
    }

    /// Build a minimal TypeEnv with SocketAddressWire registered — enough for the codec tests.
    fn make_types_with_wire() -> TypeEnv {
        use crate::types::{RecordDef, TypeDef};
        // with_builtins seeds :wat::Record (the required parent) + other kernel builtins.
        let mut env = TypeEnv::with_builtins();
        env.register_stdlib(TypeDef::Record(RecordDef {
            name: ":wat::kernel::SocketAddressWire".to_string(),
            parent: ":wat::Record".to_string(),
            field_names: vec!["minter-pid".to_string(), "name".to_string()],
            field_types: None,
        }))
        .expect("SocketAddressWire registration must succeed in tests");
        env
    }

    #[test]
    fn a_second_capability_rides_the_same_waist() {
        let types = TypeEnv::default();
        let caps = vec![address_codec(), toy_token_codec()];

        // Encode a Token through the SAME generic dispatch that carries Address'.
        let token = make_rust_opaque(":test::Token", 42u64);
        let tag = match &token {
            Value::RustOpaque(inner) => {
                encode_in(&caps, inner, &types).expect("the 2nd cap encodes generically")
            }
            _ => unreachable!(),
        };
        let (name, body) = match tag {
            OwnedValue::Tagged(t, b) => (t.name().to_string(), *b),
            _ => panic!("expected a #wat-edn.cap/<name> tag"),
        };
        assert_eq!(name, "test-token", "the toy cap got its own tag through the generic dispatch");

        // Decode it back through the SAME generic dispatch → a live :test::Token.
        let back = decode_in(&caps, &name, &body, &types).expect("the 2nd cap decodes generically");
        match back {
            Value::RustOpaque(inner) => {
                assert_eq!(inner.payload.downcast_ref::<u64>(), Some(&42u64))
            }
            _ => panic!("expected a reconstructed :test::Token opaque"),
        }
        // ZERO lines of edn_shim changed to add this capability. The waist is frozen; the edge grew.
    }

    #[test]
    fn address_roundtrips_pid_and_name() {
        // A SocketAddress with a known pid + name round-trips through the codec and the
        // reconstructed Address has the same minter_pid and name.
        let types = make_types_with_wire();
        let caps = vec![address_codec()];

        let minter_pid: i32 = 4242;
        let name_bytes: Vec<u8> = vec![1, 2, 3, 4, 5];
        let addr = crate::kernel::address::Address::from_socket_name_bytes(
            name_bytes.clone(),
            minter_pid,
        );
        let opaque = crate::rust_deps::marshal::make_rust_opaque(
            crate::kernel::spawn::ADDRESS_TYPE_PATH,
            addr,
        );
        let tag = match &opaque {
            Value::RustOpaque(inner) => {
                encode_in(&caps, inner, &types).expect("address must encode")
            }
            _ => unreachable!(),
        };
        let body = match tag {
            OwnedValue::Tagged(_, b) => *b,
            _ => panic!("expected a cap tag"),
        };
        let back = decode_in(&caps, "address", &body, &types).expect("address must decode");
        match back {
            Value::RustOpaque(inner) => {
                let reconstructed = inner
                    .payload
                    .downcast_ref::<crate::kernel::address::Address>()
                    .expect("must downcast to Address");
                let (got_pid, got_name) = reconstructed
                    .portable_form()
                    .expect("reconstructed address must be a socket address");
                assert_eq!(got_pid, minter_pid, "minter_pid must survive the round-trip");
                assert_eq!(got_name, name_bytes, "name bytes must survive the round-trip");
            }
            _ => panic!("expected a reconstructed Address RustOpaque"),
        }
    }

    #[test]
    fn address_decode_rejects_overlong_name() {
        // A name longer than the abstract-UDS limit (107 bytes) is refused at decode.
        let types = make_types_with_wire();
        let minter_pid: i32 = 1;
        let name_vec = Value::Vec(std::sync::Arc::new(
            (0..200).map(|_| Value::i64(b'a' as i64)).collect(),
        ));
        let record = Value::wat__Record {
            class_fqdn: std::sync::Arc::new("wat::kernel::SocketAddressWire".into()),
            struct_form: std::sync::Arc::new(vec![Value::i64(minter_pid as i64), name_vec]),
        };
        let body = crate::edn_shim::value_to_edn_with(&record, Some(&types));
        let err = decode_in(&[address_codec()], "address", &body, &types)
            .expect_err("an over-long address name must be refused at decode");
        match err.kind {
            EdnReadErrorKind::UnsupportedTag(msg) => {
                assert!(msg.contains("exceeds"), "expected the over-long rejection, got: {msg}")
            }
            other => panic!("expected UnsupportedTag for an over-long name, got {other:?}"),
        }
    }

    #[test]
    fn registry_rows_have_distinct_keys() {
        let rows = registry();
        let names: std::collections::HashSet<_> = rows.iter().map(|c| c.name).collect();
        let type_paths: std::collections::HashSet<_> = rows.iter().map(|c| c.type_path).collect();
        assert_eq!(names.len(), rows.len(), "duplicate name in registry — decode would silently shadow");
        assert_eq!(type_paths.len(), rows.len(), "duplicate type_path in registry — encode would silently shadow");
    }

    #[test]
    fn address_decode_rejects_empty_name() {
        // An empty byte vector is rejected at decode — symmetric with the over-long rejection.
        let types = make_types_with_wire();
        let minter_pid: i32 = 1;
        let name_vec = Value::Vec(std::sync::Arc::new(vec![]));
        let record = Value::wat__Record {
            class_fqdn: std::sync::Arc::new("wat::kernel::SocketAddressWire".into()),
            struct_form: std::sync::Arc::new(vec![Value::i64(minter_pid as i64), name_vec]),
        };
        let body = crate::edn_shim::value_to_edn_with(&record, Some(&types));
        let err = decode_in(&[address_codec()], "address", &body, &types)
            .expect_err("an empty address name must be refused at decode");
        match err.kind {
            EdnReadErrorKind::UnsupportedTag(msg) => {
                assert!(msg.contains("empty"), "expected the empty-name rejection, got: {msg}")
            }
            other => panic!("expected UnsupportedTag for an empty name, got {other:?}"),
        }
    }
}
