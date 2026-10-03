//! Stone 255.86 — `:wat::linkedlist::*` is unregistered.
//!
//! The List implementations stay in `crate::collection::eval` (`list_*_inner`).
//! `conj` on a List still prepends, through `:wat::core::conj`. This module
//! remains so `intrinsic/mod.rs` still names the file; it submits no registry row.

