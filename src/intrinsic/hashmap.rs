//! Stone 255.86 — `:wat::hashmap::*` is unregistered.
//!
//! The HashMap implementations stay in `crate::collection::eval` (`hashmap_*_inner`)
//! and are reached only through the polymorphic `:wat::core::` verbs. This module
//! remains so `intrinsic/mod.rs` still names the file; it submits no registry row.

