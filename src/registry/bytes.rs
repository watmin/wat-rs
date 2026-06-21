//! core::Bytes builtins registered into the registry (arc 255 first home).
use super::BuiltinRegistry;

pub(super) fn register(r: &mut BuiltinRegistry) {
    r.register(":wat::core::Bytes::to-hex", crate::runtime::eval_bytes_to_hex);
    r.register(":wat::core::Bytes::from-hex", crate::runtime::eval_bytes_from_hex);
}
