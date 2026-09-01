//! The `:wat::kernel::LociDiedError` process-tier construction vocabulary —
//! arc 109 Stone 4b (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-
//! error-cluster-decomposes.md`, map item 4b). Ten items: the four
//! `ProcessDiedError::{Panic,RuntimeError,MainSignature,BadReturn}` builders
//! and their four `_value` cross-module accessor siblings, plus the
//! `conj_died_chain`/`conj_died_chain_value` pair — `conj_died_chain`'s only
//! caller in the tree is `conj_died_chain_value`, so the two move together.
//!
//! Measured: `src/process/verbs.rs` and `src/distribution/mod.rs` are this
//! vocabulary's only callers anywhere in the tree; every other reference is
//! a doc mention. `failure_value_from_assertion_payload` (called here by
//! `process_died_error_panic`) stays in `runtime.rs` — it is the shared
//! `:wat::core::Fault`/`Failure` diagnostic language (map item 4d), not this
//! vocabulary's to own; it is bumped to `pub(crate)` for this move (its only
//! other caller, `thread_died_error_panic`, stays in `runtime.rs` too — that
//! is 4a's, a later stone).
//!
//! Functions lifted out of `runtime.rs` — bodies verbatim; only the
//! visibility keyword changed.

use crate::runtime::{builtin_enum_variant_names, failure_value_from_assertion_payload};
use crate::value::{EnumValue, Value};
use std::sync::Arc;

/// Arc 113 slice 2 — conj a fresh DiedError onto the FRONT of an
/// existing chain (or build a singleton when no upstream exists).
///
/// Cascade semantics: when `result::expect` panics on an Err that
/// carried a chain, that chain rides through the panic on the
/// `AssertionPayload`. The spawn driver's catch_unwind reads it
/// here and pushes THIS thread's death (`fresh`) onto the head of
/// the inherited chain. Future joiners walking from the front see
/// the death-chain in causality order: head = the thread the
/// joiner waited on; second = whoever killed it; … last = the
/// originating cause.
pub(crate) fn conj_died_chain(fresh: Value, upstream: Option<Vec<Value>>) -> Value {
    let mut chain = vec![fresh];
    if let Some(tail) = upstream {
        chain.extend(tail);
    }
    Value::Vec(Arc::new(chain))
}

/// Cross-module sibling of [`conj_died_chain`] for `src/process/verbs.rs`'s
/// child-branch panic emission (arc 113 slice 3 — chain rendered to stderr
/// as EDN; call sites at `src/process/verbs.rs:125` and `:147`).
/// Renames-but-otherwise-identical so the caller reads naturally; the
/// `_value` suffix signals "produces a runtime Value" the way the parallel
/// `process_died_error_panic_value` does.
pub(crate) fn conj_died_chain_value(fresh: Value, upstream: Option<Vec<Value>>) -> Value {
    conj_died_chain(fresh, upstream)
}

/// Build a `:wat::kernel::ProcessDiedError::Panic` enum value
/// (arc 112). Sibling of `thread_died_error_panic` for the
/// (Process :- [I O]) subject. Same payload shape; the type_path
/// distinguishes them at runtime + at the type-checker.
pub(crate) fn process_died_error_panic(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
) -> Value {
    let failure_field = match assertion {
        Some(p) => Value::Option(Arc::new(Some(failure_value_from_assertion_payload(p)))),
        None => Value::Option(Arc::new(None)),
    };
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Panic".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "Panic"),
        fields: vec![Value::String(Arc::new(message)), failure_field],
    }))
}

/// Cross-module sibling of [`process_died_error_panic`] for
/// `src/process/verbs.rs`'s child-branch panic emission (arc 113
/// slice 3; call sites at `verbs.rs:142`, `:218` and `:277`).
/// The child renders its own ProcessDiedError::Panic to EDN on
/// stderr so the parent's wat-side `extract-panics` can read
/// it back into matching Value shapes.
pub(crate) fn process_died_error_panic_value(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
) -> Value {
    process_died_error_panic(message, assertion)
}

/// Build a `:wat::kernel::ProcessDiedError::RuntimeError(message)`
/// enum value (arc 112).
pub(crate) fn process_died_error_runtime(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "RuntimeError".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "RuntimeError"),
        fields: vec![Value::String(Arc::new(message))],
    }))
}

/// Cross-module pub(crate) accessor for spawn_process.rs / fork.rs
/// (arc 170 slice 1i — structured runtime-error exit path).
///
/// Arc 296 strike 2 — generic over [`crate::edn::contract::WatError`]: the payload
/// is produced from the error's `WatError::error_edn()` via
/// [`crate::edn::contract::to_wire_edn`], so a non-`WatError` type cannot reach this
/// wire boundary (it is a compile error). The floor (:message :location :causes)
/// is always present in the wire payload.
pub(crate) fn process_died_error_runtime_value(e: &impl crate::edn::contract::WatError) -> Value {
    process_died_error_runtime(crate::edn::contract::to_wire_edn(e))
}

/// Build a `:wat::kernel::ProcessDiedError::MainSignature(message)`
/// enum value (arc 170 slice 1i). Emitted by fork child branches when
/// `validate_user_main_signature` returns `Err`.
pub(crate) fn process_died_error_main_signature(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "MainSignature".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "MainSignature"),
        fields: vec![Value::String(Arc::new(message))],
    }))
}

/// Cross-module pub(crate) accessor.
///
/// Arc 296 strike 2 — generic over [`crate::edn::contract::WatError`]. The
/// main-signature validation message is a flat message carried via a
/// [`crate::edn::contract::FlatMessage`] (itself a `WatError`), so it too crosses
/// through the floor.
pub(crate) fn process_died_error_main_signature_value(e: &impl crate::edn::contract::WatError) -> Value {
    process_died_error_main_signature(crate::edn::contract::to_wire_edn(e))
}

/// Build a `:wat::kernel::ProcessDiedError::BadReturn(message)`
/// enum value (arc 170 slice 1i). Emitted by fork / spawn-process child
/// branches when `:user::main` returns a non-nil value at runtime.
pub(crate) fn process_died_error_bad_return(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "BadReturn".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "BadReturn"),
        fields: vec![Value::String(Arc::new(message))],
    }))
}

/// Cross-module pub(crate) accessor.
///
/// Arc 296 strike 2 — generic over [`crate::edn::contract::WatError`]. The bad-return
/// type name is a flat message carried via a [`crate::edn::contract::FlatMessage`]
/// (itself a `WatError`), so it too crosses through the floor.
pub(crate) fn process_died_error_bad_return_value(e: &impl crate::edn::contract::WatError) -> Value {
    process_died_error_bad_return(crate::edn::contract::to_wire_edn(e))
}
