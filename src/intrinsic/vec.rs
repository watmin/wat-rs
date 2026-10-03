//! Stone 255.86 — `:wat::vec::*` is unregistered.
//!
//! The Vector implementations stay in `crate::collection::eval` (`vector_*_inner`,
//! including `vector_concat_inner` and `vector_extend_inner`) and are reached only
//! through `:wat::core::` (`concat` for same-kind Vector, `into` for extend).
//! This module remains so `intrinsic/mod.rs` still names the file; it submits
//! no registry row.

