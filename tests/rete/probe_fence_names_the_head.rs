//! RED probe for `docs/arc/2026/06/278-rules-engine/BRIEF-the-fence-names-the-head.md`.
//!
//! The rete `where`/accumulator compile fence used to reject an impure or non-deterministic
//! predicate with a message that named NEITHER the verb NOR the axis:
//!   "compile-condition: where expr must be pure and deterministic"
//! `src/rete/purity.rs`'s walk knew the offending head the instant it falsified an axis and threw
//! it away to return a bare `false`. This probe is MUTATION-PROVEN: it pins the panic message to
//! the EXACT byte-identical string (`assert_eq!`, not a substring check — `no_loose_string_assert`
//! requires the exact form for a deterministic value), so a regression that reverts to the old
//! blind message, drops the head, or names the WRONG axis goes red — not just "did it reject".
//!
//! Four fixtures, mirroring `probe_arc278_6b_ii_a_where_oracle_impure.wat`'s shape exactly:
//!   - `probe_arc278_6b_ii_a_where_oracle_impure.wat` — `(:wat::io::IOReader/open-file "x")`:
//!     impure (does IO) — the :pure axis must be named.
//!   - `probe_fence_names_the_head_nondet.wat` — `(:wat::uuid::v4)`: pure but NOT
//!     deterministic (random) — the :deterministic axis must be named, not :pure.
//!   - `probe_fence_names_the_head_partial.wat` — `(:wat::i64::/ ?c 1)`: pure and
//!     deterministic but NOT total — the :total axis must be named.
//!   - `probe_fence_names_the_head_core_op.wat` — `(:wat::i64::> ?c 0)`: pure, det,
//!     AND total, but a core spelling — Law A (`is not a rete primitive`) must be named,
//!     not :total. This is the pin that first-failing-axis's fourth arg exists for.
//!
//! Run: cargo test --release -p wat --test probe_fence_names_the_head

use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_file, FrozenWorld, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

const WORLD_IMPURE_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat";
const WORLD_NONDET_PATH: &str = "tests/rete/probe_fence_names_the_head_nondet.wat";
const WORLD_PARTIAL_PATH: &str = "tests/rete/probe_fence_names_the_head_partial.wat";
const WORLD_CORE_OP_PATH: &str = "tests/rete/probe_fence_names_the_head_core_op.wat";
const WORLD_THEN_PARTIAL_PATH: &str = "tests/rete/probe_fence_names_the_head_then_partial.wat";
const WORLD_THEN_CORE_OP_PATH: &str = "tests/rete/probe_fence_names_the_head_then_core_op.wat";
const WORLD_ACC_IMPURE_PATH: &str = "tests/rete/probe_fence_names_the_head_acc_impure.wat";
const WORLD_ACC_PARTIAL_PATH: &str = "tests/rete/probe_fence_names_the_head_acc_partial.wat";
const WORLD_ACC_CORE_OP_PATH: &str = "tests/rete/probe_fence_names_the_head_acc_core_op.wat";

/// Compile+run the world's zero-arg entry fn. The compile fence rejects by PANICKING
/// (`Option/expect` → `panic_any(AssertionPayload)`), so catch the unwind and pull the human
/// message back out of the payload — never just "did it reject".
fn compile_message(world_path: &str, fn_name: &str) -> Result<Value, StartupError> {
    let world: FrozenWorld = startup_from_file(world_path)?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| StartupError::Runtime(Box::new(e))),
        Err(panic_payload) => {
            let (message, actual, expected) = match panic_payload.downcast_ref::<AssertionPayload>() {
                Some(p) => (p.message.clone(), p.actual.clone(), p.expected.clone()),
                None => {
                    let message = panic_payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                        .unwrap_or_else(|| "panic-opaque".to_string());
                    (message, None, None)
                }
            };
            Err(StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::AssertionFailed { message, actual, expected },
            ))))
        }
    }
}

/// The impure (IO) `where` must be rejected, and the message must name the exact offending head
/// AND the :pure axis (not :deterministic — `IOReader/open-file` IS deterministic, only impure).
/// Exact `assert_eq!` on the whole message: strictly stronger than a substring check (also catches
/// reordering, an appended/missing clause, or the axis silently swapping), and it is the sanctioned
/// shape for a deterministic value under `no_loose_string_assert`.
#[test]
fn impure_where_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_IMPURE_PATH, ":user::run-gate-c5");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: where expr is not pure — ':wat::io::IOReader/open-file' is not pure"
        )
    );
}

/// The non-deterministic (Uuid/v4) `where` must be rejected, and the message must name the exact
/// offending head AND the :deterministic axis (not :pure — Uuid/v4 IS pure, only non-deterministic).
#[test]
fn nondeterministic_where_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_NONDET_PATH, ":user::run-gate-c5");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: where expr is not deterministic — ':wat::uuid::v4' is not deterministic"
        )
    );
}

/// The partial (i64::/) `where` must be rejected, and the message must name the exact offending
/// head AND the :total axis (not :pure — `i64::/` IS pure and deterministic, only partial).
#[test]
fn partial_where_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_PARTIAL_PATH, ":user::run-gate-c5");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: where expr is not total — ':wat::i64::/' is not total"
        )
    );
}

/// A total core op (`i64::>`) in `where` must be rejected on Law A, not totality. Exact
/// `assert_eq!`: a wrap that lets first-failing-axis's old `:else Total` swallow this stays red.
#[test]
fn core_op_where_names_law_a_not_total() {
    let r = compile_message(WORLD_CORE_OP_PATH, ":user::run-gate-c5");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: where expr is not a rete primitive — ':wat::i64::>' is not a rete primitive; a where admits only :wat::rete:: ops"
        )
    );
}

/// `:then` Total — `i64::/` must be named as not total, not as not-pure.
#[test]
fn partial_then_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_THEN_PARTIAL_PATH, ":user::run-compile");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: then expr is not total — ':wat::i64::/' is not total"
        )
    );
}

/// `:then` Law A — `i64::>` must be named as not a rete primitive, not as not-total.
#[test]
fn core_op_then_names_law_a_not_total() {
    let r = compile_message(WORLD_THEN_CORE_OP_PATH, ":user::run-compile");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: then expr is not a rete primitive — ':wat::i64::>' is not a rete primitive; a then admits only :wat::rete:: ops"
        )
    );
}

/// Accumulator user-fold Pure — IO in the fold body must name the fold head and :pure.
#[test]
fn impure_accumulator_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_ACC_IMPURE_PATH, ":user::run-compile");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: accumulator expr is not pure — ':wat::io::IOReader/open-file' is not pure"
        )
    );
}

/// Accumulator Total — `i64::/` in a user fold must be named as not total.
#[test]
fn partial_accumulator_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_ACC_PARTIAL_PATH, ":user::run-compile");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: accumulator expr is not total — ':wat::i64::/' is not total"
        )
    );
}

/// Accumulator Law A — `i64::>` in a user fold must be named as not a rete primitive.
#[test]
fn core_op_accumulator_names_law_a_not_total() {
    let r = compile_message(WORLD_ACC_CORE_OP_PATH, ":user::run-compile");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: accumulator expr is not a rete primitive — ':wf::core-fold' is not a rete primitive; a accumulator admits only :wat::rete:: ops"
        )
    );
}
