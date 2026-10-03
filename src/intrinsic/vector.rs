//! Stone 255.86 — `:wat::vector::*` is unregistered.
//!
//! The PersistentVector implementations stay in `crate::collection::eval`
//! (`persistentvector_*_inner`). `:wat::vector::concat` was `into`, not
//! `:wat::core::concat`. This module remains so `intrinsic/mod.rs` still names
//! the file; it submits no registry row.

