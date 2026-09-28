//! The `:wat::kernel::LociDiedError` process-tier construction vocabulary —
//! arc 109 Stone 4b (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-
//! error-cluster-decomposes.md`, map item 4b). Originally ten items: the four
//! `ProcessDiedError::{Panic,RuntimeError,MainSignature,BadReturn}` builders
//! and their four `_value` cross-module accessor siblings, plus the
//! `conj_died_chain`/`conj_died_chain_value` pair — `conj_died_chain`'s only
//! caller in the tree is `conj_died_chain_value`, so the two move together.
//! Excursus 003 strike A (F7) retired the `BadReturn` pair (no legally
//! reachable producer; `:user::main` returning non-nil now routes to
//! `process_died_error_panic_value` instead), so eight remain.
//!
//! Measured: `src/process/verbs.rs` and `src/distribution/mod.rs` are this
//! vocabulary's only callers anywhere in the tree; every other reference is
//! a doc mention. `failure_value_from_assertion_payload` (called here by
//! `process_died_error_panic`) stays in `runtime.rs` — it is the shared
//! `:wat::core::Fault`/`Failure` diagnostic language (map item 4d), not this
//! vocabulary's to own; it is bumped to `pub(crate)` for this move (its only
//! other caller, `thread_died_error_panic`, went to `src/kernel/error.rs` in
//! stone 4a — so both of its callers now live outside `runtime.rs`, and it
//! remains there as 4d residue on its own merits, not for want of a mover).
//!
//! Functions lifted out of `runtime.rs` — bodies verbatim; only the
//! visibility keyword changed.

use crate::runtime::{
    builtin_enum_variant_names, failure_value_from_assertion_payload, flat_message_failure,
    runtime_error_failure,
};
use crate::value::{EnumValue, RuntimeError, Value};
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

/// Build a `:wat::kernel::LociDiedError::Panic` enum value (arc 112; excursus 003
/// step 3b — the envelope carries `Failure`). Sibling of `thread_died_error_panic`
/// for the `(Process :- [I O])` subject. ONE mandatory `Failure` field now: a plain
/// panic's `failure.error` is a synthesized `:wat::core::Fault`
/// (`flat_message_failure`), never a bare message string.
pub(crate) fn process_died_error_panic(
    message: String,
    assertion: Option<crate::assertion::AssertionPayload>,
) -> Value {
    let failure_field = match assertion {
        Some(p) => failure_value_from_assertion_payload(p),
        None => flat_message_failure(message),
    };
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "Panic".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "Panic"),
        fields: vec![failure_field],
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

/// Build a `:wat::kernel::LociDiedError::RuntimeError(failure)` enum value (arc 112;
/// excursus 003 step 3b) from a FLAT message with no `RuntimeError` of its own —
/// e.g. `validate_user_grep_signature`'s `GrepSignatureError` `FlatMessage`.
/// `failure.error` is a synthesized `:wat::core::Fault` (`flat_message_failure`). A
/// genuine `RuntimeError` goes through [`process_died_error_runtime_from_error`]
/// instead, which preserves its declared `:wat::runtime::<Kind>` record and its own
/// already-captured frames.
pub(crate) fn process_died_error_runtime(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "RuntimeError".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "RuntimeError"),
        fields: vec![flat_message_failure(message)],
    }))
}

/// Build a `:wat::kernel::LociDiedError::RuntimeError(failure)` enum value from a
/// GENUINE `RuntimeError` (excursus 003 step 3b, item 2's "From a RuntimeError"
/// branch) — `failure.error` is `re.to_record()` (step 3a's declared
/// `:wat::runtime::<Kind>` record); `frames` / `frames-elided` are the error's OWN
/// already-captured trace (step 2), never re-snapshotted. Replaces the generic
/// `process_died_error_runtime_value::<RuntimeError>` call sites — `RuntimeError`
/// and a bare `FlatMessage` need DIFFERENT construction (a real error's frames vs. a
/// synthesized Fault's), so one generic fn covering both silently lost the
/// RuntimeError's own capture. The thread-tier sibling is
/// [`crate::kernel::error::thread_died_error_runtime_from_error`].
pub(crate) fn process_died_error_runtime_from_error(re: &RuntimeError) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "RuntimeError".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "RuntimeError"),
        fields: vec![runtime_error_failure(re)],
    }))
}

/// Cross-module pub(crate) accessor for spawn_process.rs / fork.rs
/// (arc 170 slice 1i — structured runtime-error exit path).
///
/// Arc 296 strike 2 / excursus 003 step 3b — generic over
/// [`crate::edn::contract::WatError`], for a FLAT-message producer (a
/// [`crate::edn::contract::FlatMessage`] with no recoverable location of its own,
/// e.g. `validate_user_grep_signature`'s `GrepSignatureError`). Reads `e.message()`
/// directly — NOT `to_wire_edn(e)`, which would serialize `e`'s own floor
/// (`:message`/`:location`/`:causes`) into the string and double-quote it once
/// `process_died_error_runtime` wraps it in a `Fault`. A genuine `RuntimeError` (a
/// non-flat `WatError` with its own captured frames) goes through
/// [`process_died_error_runtime_from_error`] instead — this generic fn is for
/// callers with nothing but a message to give.
pub(crate) fn process_died_error_runtime_value(e: &impl crate::edn::contract::WatError) -> Value {
    process_died_error_runtime(e.message())
}

/// Build a `:wat::kernel::LociDiedError::MainSignature(failure)` enum value
/// (arc 170 slice 1i; excursus 003 step 3b). Emitted by fork child branches when
/// `validate_user_main_signature` returns `Err`. `failure.error` is a synthesized
/// `:wat::core::Fault` (`flat_message_failure`).
pub(crate) fn process_died_error_main_signature(message: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::kernel::LociDiedError".into(),
        variant_name: "MainSignature".into(),
        names: builtin_enum_variant_names(":wat::kernel::LociDiedError", "MainSignature"),
        fields: vec![flat_message_failure(message)],
    }))
}

/// Cross-module pub(crate) accessor.
///
/// Arc 296 strike 2 / excursus 003 step 3b — generic over
/// [`crate::edn::contract::WatError`]. The main-signature validation message is a
/// flat message carried via a [`crate::edn::contract::FlatMessage`] (itself a
/// `WatError`); reads `e.message()`, never `to_wire_edn(e)` (see
/// `process_died_error_runtime_value`'s doc for why).
pub(crate) fn process_died_error_main_signature_value(e: &impl crate::edn::contract::WatError) -> Value {
    process_died_error_main_signature(e.message())
}

// `process_died_error_bad_return` / `process_died_error_bad_return_value` — RETIRED
// (excursus 003 strike A, F7). `LociDiedError::BadReturn` had no producer that could
// ever be legally reached (the type checker refuses any `:user::main` body that could
// return non-nil before it runs); `:user::main` returning non-nil now routes to
// `process_died_error_panic_value` instead (`src/process/verbs.rs`'s
// `bad_return_panic_value`), so these two builders have no caller left.
