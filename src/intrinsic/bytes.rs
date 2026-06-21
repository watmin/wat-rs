//! core::Bytes intrinsics registered into the registry (arc 255 first home).
use super::IntrinsicRegistry;

pub(super) fn register(r: &mut IntrinsicRegistry) {
    r.register(":wat::core::Bytes::to-hex", crate::runtime::eval_bytes_to_hex);
    r.register(":wat::core::Bytes::from-hex", crate::runtime::eval_bytes_from_hex);
}
