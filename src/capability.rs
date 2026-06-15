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
    /// back to the non-portable `wat-edn.opaque` tag (refused on decode).
    pub encode: fn(&RustOpaqueInner) -> Option<OwnedValue>,
    /// Reconstruct a live capability `Value` from a wire body (called only off the trusted door).
    pub decode: fn(&OwnedValue) -> Result<Value, EdnReadError>,
}

/// THE registry — built once. **Adding a capability = a `CapCodec` row here (append-only); the
/// `edn_shim` dispatch (the waist) never changes.** This central row is the open EDGE; the frozen
/// WAIST is the two generic dispatch fns + the wire contract.
fn registry() -> &'static [CapCodec] {
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
pub fn encode_capability(inner: &RustOpaqueInner) -> Option<OwnedValue> {
    encode_in(registry(), inner)
}

/// Generic DECODE dispatch — called by `edn_shim`'s `wat-edn.cap` tag arm, which is reached ONLY off
/// the trusted door (`decode_trusted_wire`). An unregistered name is refused.
pub fn decode_capability(name: &str, body: &OwnedValue) -> Result<Value, EdnReadError> {
    decode_in(registry(), name, body)
}

/// The encode dispatch over an EXPLICIT codec set. Identical for N capabilities — a linear find by
/// `type_path`. Split out so the waist's N-capability behaviour is directly testable (the strike-2
/// proof passes a 2-codec slice); `encode_capability` is just `encode_in(registry(), …)`.
fn encode_in(caps: &[CapCodec], inner: &RustOpaqueInner) -> Option<OwnedValue> {
    let codec = caps.iter().find(|c| c.type_path == inner.type_path)?;
    let body = (codec.encode)(inner)?;
    Some(OwnedValue::Tagged(Tag::ns("wat-edn.cap", codec.name), Box::new(body)))
}

/// The decode dispatch over an EXPLICIT codec set — a linear find by tag `name`.
fn decode_in(caps: &[CapCodec], name: &str, body: &OwnedValue) -> Result<Value, EdnReadError> {
    let codec = caps.iter().find(|c| c.name == name).ok_or_else(|| EdnReadError {
        span: Span::unknown(),
        kind: EdnReadErrorKind::UnsupportedTag(format!("wat-edn.cap/{name}")),
    })?;
    (codec.decode)(body)
}

// ─── Registrants ──────────────────────────────────────────────────────────────

/// `Address'` (arc 272 6a-i) — the kernel-minted abstract UDS name. The first inhabitant of the
/// waist. Portable as a process-tier socket address (the name bytes); a thread-tier `Address'`
/// (a crossbeam `Sender`) has no portable form → `encode` returns `None`.
fn address_codec() -> CapCodec {
    CapCodec {
        name: "address",
        type_path: crate::kernel::spawn::ADDRESS_TYPE_PATH,
        encode: |inner| {
            let bytes = inner
                .payload
                .downcast_ref::<crate::kernel::address::Address>()?
                .portable_name_bytes()?;
            Some(OwnedValue::Vector(
                bytes.into_iter().map(|b| OwnedValue::Integer(b as i64)).collect(),
            ))
        },
        decode: |body| {
            let items = match body {
                OwnedValue::Vector(items) => items,
                _ => {
                    return Err(EdnReadError {
                        span: Span::unknown(),
                        kind: EdnReadErrorKind::UnsupportedTag(
                            "wat-edn.cap/address (expected a byte vector)".into(),
                        ),
                    })
                }
            };
            let mut bytes = Vec::with_capacity(items.len());
            for it in items {
                match it {
                    OwnedValue::Integer(n) if (0..=255).contains(n) => bytes.push(*n as u8),
                    _ => {
                        return Err(EdnReadError {
                            span: Span::unknown(),
                            kind: EdnReadErrorKind::UnsupportedTag(
                                "wat-edn.cap/address (byte out of 0..=255)".into(),
                            ),
                        })
                    }
                }
            }
            let addr = crate::kernel::address::Address::from_socket_name_bytes(bytes);
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
    use crate::rust_deps::marshal::make_rust_opaque;

    /// A toy second capability: `:test::Token` over a `u64`. Real shape, trivial payload.
    fn toy_token_codec() -> CapCodec {
        CapCodec {
            name: "test-token",
            type_path: ":test::Token",
            encode: |inner| Some(OwnedValue::Integer(*inner.payload.downcast_ref::<u64>()? as i64)),
            decode: |body| match body {
                OwnedValue::Integer(n) => Ok(make_rust_opaque(":test::Token", *n as u64)),
                _ => Err(EdnReadError {
                    span: Span::unknown(),
                    kind: EdnReadErrorKind::UnsupportedTag(
                        "wat-edn.cap/test-token (expected an integer)".into(),
                    ),
                }),
            },
        }
    }

    #[test]
    fn a_second_capability_rides_the_same_waist() {
        // The registry extended by exactly ONE row — the only change a new capability requires.
        let caps = vec![address_codec(), toy_token_codec()];

        // Encode a Token through the SAME generic dispatch that carries Address'.
        let token = make_rust_opaque(":test::Token", 42u64);
        let tag = match &token {
            Value::RustOpaque(inner) => encode_in(&caps, inner).expect("the 2nd cap encodes generically"),
            _ => unreachable!(),
        };
        let (name, body) = match tag {
            OwnedValue::Tagged(t, b) => (t.name().to_string(), *b),
            _ => panic!("expected a #wat-edn.cap/<name> tag"),
        };
        assert_eq!(name, "test-token", "the toy cap got its own tag through the generic dispatch");

        // Decode it back through the SAME generic dispatch → a live :test::Token.
        let back = decode_in(&caps, &name, &body).expect("the 2nd cap decodes generically");
        match back {
            Value::RustOpaque(inner) => {
                assert_eq!(inner.payload.downcast_ref::<u64>(), Some(&42u64))
            }
            _ => panic!("expected a reconstructed :test::Token opaque"),
        }
        // ZERO lines of edn_shim changed to add this capability. The waist is frozen; the edge grew.
    }
}
