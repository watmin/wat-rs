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
fn decode_in(caps: &[CapCodec], name: &str, body: &OwnedValue) -> Result<Value, EdnReadError> {
    let codec = caps
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| cap_decode_error(format!("wat-edn.cap/{name}")))?;
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
                _ => return Err(cap_decode_error("wat-edn.cap/address (expected a byte vector)")),
            };
            // Cap the decoded name at the abstract-UDS limit (`sun_path` is 108 bytes; an abstract
            // name occupies `sun_path[1..]`, so ≤ 107). Reject an over-long name HERE — early, at
            // decode — rather than letting it fail late at `connect_addr` (kernel/address.rs). (The
            // wire body carries no source position, so the rejection is early, not span-located.)
            const ABSTRACT_UDS_NAME_MAX: usize = 107;
            if items.is_empty() {
                return Err(cap_decode_error(
                    "wat-edn.cap/address (empty name — a minted abstract name is never zero-length)",
                ));
            }
            if items.len() > ABSTRACT_UDS_NAME_MAX {
                return Err(cap_decode_error(format!(
                    "wat-edn.cap/address (name {} bytes exceeds the {}-byte abstract-UDS limit)",
                    items.len(),
                    ABSTRACT_UDS_NAME_MAX
                )));
            }
            let mut bytes = Vec::with_capacity(items.len());
            for it in items {
                match it {
                    OwnedValue::Integer(n) if (0..=255).contains(n) => bytes.push(*n as u8),
                    _ => return Err(cap_decode_error("wat-edn.cap/address (byte out of 0..=255)")),
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
                // Route through the SAME attested spanless helper the production codecs use, so
                // span-omission is ONE runed path, not a parallel hand-built struct literal.
                _ => Err(cap_decode_error("wat-edn.cap/test-token (expected an integer)")),
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

    #[test]
    fn address_decode_rejects_overlong_name() {
        // A name longer than the abstract-UDS limit (107 bytes) is refused at decode with a located
        // error — not deferred to a late, unlocated connect failure. (A trusted-wire codec rejects a
        // malformed body early.)
        let overlong = OwnedValue::Vector((0..200).map(|_| OwnedValue::Integer(b'a' as i64)).collect());
        let err = decode_in(&[address_codec()], "address", &overlong)
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
        // A kernel-minted autobind name is ALWAYS non-empty (5 random bytes); zero-length is
        // malformed by construction and must be caught early, not deferred to a connect failure.
        let empty = OwnedValue::Vector(vec![]);
        let err = decode_in(&[address_codec()], "address", &empty)
            .expect_err("an empty address name must be refused at decode");
        match err.kind {
            EdnReadErrorKind::UnsupportedTag(msg) => {
                assert!(msg.contains("empty"), "expected the empty-name rejection, got: {msg}")
            }
            other => panic!("expected UnsupportedTag for an empty name, got {other:?}"),
        }
    }
}
