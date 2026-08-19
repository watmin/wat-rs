//! Arc 272 — the capability narrow-waist — a capability wears its own type home on the wire.
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
use crate::types::TypeEnv;
use crate::value::value::AggregateValue;
use std::sync::OnceLock;
use wat_edn::OwnedValue;

/// A registered portable capability — the codec that crosses the capability waist.
pub struct CapCodec {
    /// The `RustOpaque` `type_path` this codec encodes AND decodes — the single key in both
    /// directions (arc 294.m collapsed the former two-key asymmetry: encode resolved by
    /// `type_path` while decode resolved by a separate `name` nickname stamped into a
    /// `wat-edn.cap` marker tag). The wire tag is now the capability's own type home, derived
    /// from this same `type_path` via `edn_shim::tag_from_type_path` — e.g.
    /// `:wat::kernel::Address` → `#wat.kernel/Address`.
    pub type_path: &'static str,
    /// Encode the opaque's PORTABLE form to a wire body. `None` when this opaque instance has no
    /// portable form (e.g. a thread-tier `Address'`, whose `Sender` cannot cross) → the caller falls
    /// back to the non-portable per-type-home tag (arc 294.i; refused on decode). `types` provides
    /// the type registry so record codecs can encode named fields.
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
/// capability's own type-home tag (`#wat.kernel/Address`, …) when `inner` is a registered
/// portable capability WITH a portable form; `None` otherwise (→ the caller emits the
/// non-portable opaque tag).
pub fn encode_capability(inner: &RustOpaqueInner, types: &TypeEnv) -> Option<OwnedValue> {
    encode_in(registry(), inner, types)
}

/// Arc 294.m — is `type_path` a registered capability codec? This is THE question the refusal
/// (`edn_shim::tagged_to_value`) asks before deciding a tag needs the trusted door at all — the
/// registry is the wall now, not a namespace string (arc 198's ruling).
pub fn is_capability_type_path(type_path: &str) -> bool {
    registry().iter().any(|c| c.type_path == type_path)
}

/// Generic DECODE dispatch — called by `edn_shim`'s capability-tag arm, which is reached ONLY off
/// the trusted door (`decode_trusted_wire`), and only once [`is_capability_type_path`] has already
/// confirmed `type_path` is registered. An unregistered `type_path` is refused.
pub fn decode_capability(type_path: &str, body: &OwnedValue, types: &TypeEnv) -> Result<Value, EdnReadError> {
    decode_in(registry(), type_path, body, types)
}

/// The encode dispatch over an EXPLICIT codec set. Identical for N capabilities — a linear find by
/// `type_path`. Split out so the waist's N-capability behaviour is directly testable (the strike-2
/// proof passes a 2-codec slice); `encode_capability` is just `encode_in(registry(), …)`.
fn encode_in(caps: &[CapCodec], inner: &RustOpaqueInner, types: &TypeEnv) -> Option<OwnedValue> {
    let codec = caps.iter().find(|c| c.type_path == inner.type_path)?;
    let body = (codec.encode)(inner, types)?;
    // Arc 294.m — the wire tag IS the capability's real type home, derived from the same
    // `type_path` key encode just resolved by (never a second, hand-rolled path-joiner; never a
    // `wat-edn.cap` marker namespace + nickname).
    Some(OwnedValue::Tagged(crate::edn_shim::tag_from_type_path(codec.type_path), Box::new(body)))
}

/// Construct a capability-decode error. Decode reconstructs off the trusted peer wire from an
/// `OwnedValue` body, which carries no source position — so the span is legitimately unknown,
/// attested here ONCE rather than filled by silent convention at each call site (a bare
/// `crate::rust_caller_span!()` reads identically to a discarded-span bug; this names why it is not one).
// rune:conformare(spanless-by-domain) — capability decode reconstructs off the trusted wire from an
// OwnedValue body that carries no source location; consumers of EdnReadError from decode_capability
// do not expect a span.
fn cap_decode_error(reason: impl Into<String>) -> EdnReadError {
    EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnsupportedTag(reason.into()) }
}

/// The decode dispatch over an EXPLICIT codec set — a linear find by `type_path`, the SAME key
/// [`encode_in`] resolves by (arc 294.m collapsed the former encode-by-type_path /
/// decode-by-name asymmetry into a single key).
fn decode_in(caps: &[CapCodec], type_path: &str, body: &OwnedValue, types: &TypeEnv) -> Result<Value, EdnReadError> {
    let codec = caps
        .iter()
        .find(|c| c.type_path == type_path)
        .ok_or_else(|| cap_decode_error(format!("no registered capability codec for type path {type_path}")))?;
    (codec.decode)(body, types)
}

// ─── Registrants ──────────────────────────────────────────────────────────────

/// The canonical FQDN for the `SocketAddressWire` base record — the single source of truth for
/// both the encode and decode paths. Changing the wire class name is a ONE-site edit here.
const SOCKET_ADDRESS_WIRE_CLASS: &str = "wat::kernel::SocketAddressWire";

/// Build the `SocketAddressWire` `Value::wat__core__Record` from its two fields.
/// struct_form = [i64(minter_pid), Value::Vec of i64 name bytes].
/// Called by the encode closure so the class FQDN + field order live once, not twice.
fn socket_address_wire_to_record(minter_pid: i32, name_bytes: Vec<u8>) -> Value {
    let name_vec = Value::Vec(std::sync::Arc::new(
        name_bytes.into_iter().map(|b| Value::i64(b as i64)).collect(),
    ));
    Value::Aggregate(std::sync::Arc::new(AggregateValue::record(
        SOCKET_ADDRESS_WIRE_CLASS.into(),
        socket_address_wire_names(),
        std::sync::Arc::new(vec![Value::i64(minter_pid as i64), name_vec]),
    )))
}

// Arc 296 G-1 — class C, missing from the brief's table (which enumerated 16 hardcoded
// classes; `SocketAddressWire` is a 17th, declared at `wat/spawn.wat:33`).
::wat_source_derive::wat_field_names_from!(
    SOCKET_ADDRESS_WIRE_FIELDS, "wat/spawn.wat", ":wat::kernel::SocketAddressWire"
);
fn socket_address_wire_names() -> std::sync::Arc<Vec<String>> {
    static N: std::sync::OnceLock<std::sync::Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SOCKET_ADDRESS_WIRE_FIELDS)).clone()
}

/// Extract `(minter_pid, name_bytes)` from a decoded `SocketAddressWire` `Value`.
/// Owns the class check (vs `SOCKET_ADDRESS_WIRE_CLASS`), the 2-field check, and all
/// per-field validation (byte range, non-empty, ≤107 limit). Called by the decode closure.
fn socket_address_wire_from_record(rec: &Value) -> Result<(i32, Vec<u8>), EdnReadError> {
    let agg = match rec {
        Value::Aggregate(a) => a,
        _ => {
            return Err(cap_decode_error(
                "#wat.kernel/Address (expected a SocketAddressWire record)",
            ))
        }
    };
    if agg.class.as_ref() != SOCKET_ADDRESS_WIRE_CLASS {
        return Err(cap_decode_error(format!(
            "#wat.kernel/Address (wrong record class: {})",
            agg.class
        )));
    }
    if agg.fields.len() != 2 {
        return Err(cap_decode_error(
            "#wat.kernel/Address (SocketAddressWire must have 2 fields)",
        ));
    }
    // Field 0: minter-pid (i64)
    let minter_pid = match &agg.fields[0] {
        Value::i64(n) => *n as i32,
        _ => {
            return Err(cap_decode_error(
                "#wat.kernel/Address (minter-pid field must be i64)",
            ))
        }
    };
    // Field 1: name (Vector<i64> = Value::Vec of Value::i64)
    let name_bytes_vals = match &agg.fields[1] {
        Value::Vec(xs) => xs.clone(),
        _ => {
            return Err(cap_decode_error(
                "#wat.kernel/Address (name field must be Vector<i64>)",
            ))
        }
    };
    // Cap the decoded name at the abstract-UDS limit (sun_path[1..] ≤ 107 bytes).
    const ABSTRACT_UDS_NAME_MAX: usize = 107;
    if name_bytes_vals.is_empty() {
        return Err(cap_decode_error(
            "#wat.kernel/Address (empty name — a minted abstract name is never zero-length)",
        ));
    }
    if name_bytes_vals.len() > ABSTRACT_UDS_NAME_MAX {
        return Err(cap_decode_error(format!(
            "#wat.kernel/Address (name {} bytes exceeds the {}-byte abstract-UDS limit)",
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
                    "#wat.kernel/Address (name byte out of 0..=255)",
                ))
            }
        }
    }
    Ok((minter_pid, name_bytes))
}

/// `Address'` (arc 272 6c.2 D1) — the kernel-minted abstract UDS address. The first inhabitant of
/// the waist. Portable as a process-tier socket address: carries the minter pid + name bytes as a
/// registered `SocketAddressWire` base record (the general record encode/decode path handles
/// field naming). A thread-tier `Address'` (a crossbeam `Sender`) has no portable form →
/// `encode` returns `None`.
fn address_codec() -> CapCodec {
    CapCodec {
        type_path: crate::kernel::spawn::ADDRESS_TYPE_PATH,
        encode: |inner, types| {
            let addr = inner.payload.downcast_ref::<crate::kernel::address::Address>()?;
            let (minter_pid, name_bytes) = addr.portable_form()?;
            Some(crate::edn_shim::value_to_edn_with(
                &socket_address_wire_to_record(minter_pid, name_bytes),
                Some(types),
            ))
        },
        decode: |body, types| {
            // body is the OwnedValue body of #wat.kernel/Address — expected to be a
            // #wat.kernel/SocketAddressWire tagged map (as produced by value_to_edn_with on the
            // SocketAddressWire record).
            // ctx=None: this codec only ever decodes the fixed `SocketAddressWire` Record
            // (never a user-declared HolonRecord class), so no EncodingCtx is ever needed.
            let record_val = crate::edn_shim::edn_to_value(body, Some(types), None).map_err(|_| {
                cap_decode_error("#wat.kernel/Address (body failed edn_to_value)")
            })?;
            let (minter_pid, name_bytes) = socket_address_wire_from_record(&record_val)?;
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
            type_path: ":test::Token",
            encode: |inner, _types| {
                Some(OwnedValue::Integer(*inner.payload.downcast_ref::<u64>()? as i64))
            },
            decode: |body, _types| match body {
                OwnedValue::Integer(n) => Ok(make_rust_opaque(":test::Token", *n as u64)),
                // Route through the SAME attested spanless helper the production codecs use, so
                // span-omission is ONE runed path, not a parallel hand-built struct literal.
                _ => Err(cap_decode_error("test::Token (expected an integer)")),
            },
        }
    }

    /// Build a minimal TypeEnv with SocketAddressWire registered — enough for the codec tests.
    fn make_types_with_wire() -> TypeEnv {
        use crate::types::{AggregateDef, Nature, TypeDef, TypeExpr};
        // with_builtins seeds :wat::core::Record (the required parent) + other kernel builtins.
        let mut env = TypeEnv::with_builtins();
        env.register_stdlib(TypeDef::Aggregate(AggregateDef {
            name: ":wat::kernel::SocketAddressWire".to_string(),
            type_params: vec![],
            nature: Nature::Record,
            restrictions: None,
            // minter-pid <- :wat::core::i64
            // name       <- :wat::core::Vector<wat::core::i64>
            fields: vec![
                ("minter-pid".to_string(), TypeExpr::Path(":wat::core::i64".to_string())),
                ("name".to_string(), TypeExpr::Parametric {
                    head: "wat::core::Vector".to_string(),
                    args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
                }),
            ],
        }))
        .expect("SocketAddressWire registration must succeed in tests");
        env
    }

    /// Encode a `Value::RustOpaque` through the capability waist and unwrap the tag name + body.
    /// Owns the encode + tag-unwrap step so round-trip tests don't repeat anonymous match/panic
    /// scaffolding. Channel and opaque CONSTRUCTION stay inline in each test.
    fn encode_through_waist(
        caps: &[CapCodec],
        opaque: &Value,
        types: &crate::types::TypeEnv,
    ) -> (String, wat_edn::OwnedValue) {
        let inner = match opaque {
            Value::RustOpaque(inner) => inner,
            _ => unreachable!(),
        };
        let tag = encode_in(caps, inner, types).expect("capability must encode through the waist");
        match tag {
            OwnedValue::Tagged(t, b) => (t.name().to_string(), *b),
            _ => panic!("expected the capability's own type-home tag"),
        }
    }

    /// Decode a capability body through the waist dispatch. Owns the decode step so round-trip
    /// tests don't repeat the inline expect scaffold. The final downcast/assert stays inline in
    /// each test because each test asserts a different payload type.
    fn decode_through_waist(
        caps: &[CapCodec],
        type_path: &str,
        body: &wat_edn::OwnedValue,
        types: &crate::types::TypeEnv,
    ) -> Value {
        decode_in(caps, type_path, body, types).expect("capability must decode through the waist")
    }

    #[test]
    fn a_second_capability_rides_the_same_waist() {
        let types = TypeEnv::default();
        let caps = vec![address_codec(), toy_token_codec()];

        // Encode a Token through the SAME generic dispatch that carries Address'.
        let token = make_rust_opaque(":test::Token", 42u64);
        let (tag_name, body) = encode_through_waist(&caps, &token, &types);
        assert_eq!(tag_name, "Token", "the toy cap wears its own type-home tag through the generic dispatch");

        // Decode it back through the SAME generic dispatch, keyed by the SAME type_path encode
        // resolved by (arc 294.m: one key, both directions) → a live :test::Token.
        let back = decode_through_waist(&caps, ":test::Token", &body, &types);
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
        let (_, body) = encode_through_waist(&caps, &opaque, &types);
        let back = decode_through_waist(&caps, crate::kernel::spawn::ADDRESS_TYPE_PATH, &body, &types);
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

    /// Build a `SocketAddressWire` OwnedValue body with a caller-supplied `name` `Value` and the
    /// given `minter_pid`. Takes a raw `Value` name (not `Vec<u8>`) so callers can pass malformed
    /// or oversized names that `socket_address_wire_to_record`'s `Vec<u8>` path cannot construct.
    fn make_address_body(minter_pid: i32, name: Value, types: &crate::types::TypeEnv) -> wat_edn::OwnedValue {
        let record = Value::Aggregate(std::sync::Arc::new(AggregateValue::record(
            SOCKET_ADDRESS_WIRE_CLASS.into(),
            socket_address_wire_names(),
            std::sync::Arc::new(vec![Value::i64(minter_pid as i64), name]),
        )));
        crate::edn_shim::value_to_edn_with(&record, Some(types))
    }

    #[test]
    fn address_decode_rejects_overlong_name() {
        // A name longer than the abstract-UDS limit (107 bytes) is refused at decode.
        let types = make_types_with_wire();
        let name_vec = Value::Vec(std::sync::Arc::new(
            (0..200).map(|_| Value::i64(b'a' as i64)).collect(),
        ));
        let body = make_address_body(1, name_vec, &types);
        let err = decode_in(&[address_codec()], crate::kernel::spawn::ADDRESS_TYPE_PATH, &body, &types)
            .expect_err("an over-long address name must be refused at decode");
        match err.kind {
            EdnReadErrorKind::UnsupportedTag(msg) => {
                assert_eq!(msg, "#wat.kernel/Address (name 200 bytes exceeds the 107-byte abstract-UDS limit)");
            }
            other => panic!("expected UnsupportedTag for an over-long name, got {other:?}"),
        }
    }

    #[test]
    fn registry_rows_have_distinct_keys() {
        // Arc 294.m — `type_path` is now the ONLY key, in both directions; a duplicate would
        // silently shadow both encode (already true before) AND decode (new: decode used to key
        // on the separate `name` nickname, which is now gone).
        let rows = registry();
        let type_paths: std::collections::HashSet<_> = rows.iter().map(|c| c.type_path).collect();
        assert_eq!(type_paths.len(), rows.len(), "duplicate type_path in registry — encode AND decode would silently shadow");
    }

    #[test]
    fn address_decode_rejects_empty_name() {
        // An empty byte vector is rejected at decode — symmetric with the over-long rejection.
        let types = make_types_with_wire();
        let name_vec = Value::Vec(std::sync::Arc::new(vec![]));
        let body = make_address_body(1, name_vec, &types);
        let err = decode_in(&[address_codec()], crate::kernel::spawn::ADDRESS_TYPE_PATH, &body, &types)
            .expect_err("an empty address name must be refused at decode");
        match err.kind {
            EdnReadErrorKind::UnsupportedTag(msg) => {
                assert_eq!(msg, "#wat.kernel/Address (empty name — a minted abstract name is never zero-length)");
            }
            other => panic!("expected UnsupportedTag for an empty name, got {other:?}"),
        }
    }
}
