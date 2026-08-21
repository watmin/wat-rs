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
//!   - `probe_fence_names_the_head_nondet.wat` — `(:wat::core::Uuid/v4)`: pure but NOT
//!     deterministic (random) — the :deterministic axis must be named, not :pure.
//!   - `probe_fence_names_the_head_partial.wat` — `(:wat::core::i64::/ ?c 1)`: pure and
//!     deterministic but NOT total — the :total axis must be named.
//!   - `probe_fence_names_the_head_core_op.wat` — `(:wat::core::i64::> ?c 0)`: pure, det,
//!     AND total, but a core spelling — Law A (`is not a rete primitive`) must be named,
//!     not :total. This is the pin that first-failing-axis's fourth arg exists for.
//!
//! Run: cargo test --release -p wat --test probe_fence_names_the_head

use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

const WORLD_IMPURE_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat";
const WORLD_NONDET_PATH: &str = "tests/rete/probe_fence_names_the_head_nondet.wat";
const WORLD_PARTIAL_PATH: &str = "tests/rete/probe_fence_names_the_head_partial.wat";
const WORLD_CORE_OP_PATH: &str = "tests/rete/probe_fence_names_the_head_core_op.wat";

/// Compile+run the world's zero-arg entry fn. The compile fence rejects by PANICKING
/// (`Option/expect` → `panic_any(AssertionPayload)`), so catch the unwind and pull the human
/// message back out of the payload — never just "did it reject".
fn compile_message(world_path: &str, fn_name: &str) -> Result<Value, String> {
    let world: FrozenWorld = startup_from_file(world_path).map_err(|e| format!("startup: {e:?}"))?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| format!("eval: {e:?}")),
        Err(panic_payload) => {
            if let Some(p) = panic_payload.downcast_ref::<AssertionPayload>() {
                Err(p.message.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
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
    let msg = r.expect_err("an impure (io) where must fail to compile");
    assert_eq!(
        msg,
        "compile-condition: where expr is not pure — ':wat::io::IOReader/open-file' is not pure"
    );
}

/// The non-deterministic (Uuid/v4) `where` must be rejected, and the message must name the exact
/// offending head AND the :deterministic axis (not :pure — Uuid/v4 IS pure, only non-deterministic).
#[test]
fn nondeterministic_where_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_NONDET_PATH, ":user::run-gate-c5");
    let msg = r.expect_err("a non-deterministic (Uuid/v4) where must fail to compile");
    assert_eq!(
        msg,
        "compile-condition: where expr is not deterministic — ':wat::core::Uuid/v4' is not deterministic"
    );
}

/// The partial (i64::/) `where` must be rejected, and the message must name the exact offending
/// head AND the :total axis (not :pure — `i64::/` IS pure and deterministic, only partial).
#[test]
fn partial_where_names_the_offending_head_and_axis() {
    let r = compile_message(WORLD_PARTIAL_PATH, ":user::run-gate-c5");
    let msg = r.expect_err("a partial (i64::/) where must fail to compile");
    assert_eq!(
        msg,
        "compile-condition: where expr is not total — ':wat::core::i64::/' is not total"
    );
}

/// A total core op (`i64::>`) in `where` must be rejected on Law A, not totality. Exact
/// `assert_eq!`: a wrap that lets first-failing-axis's old `:else Total` swallow this stays red.
#[test]
fn core_op_where_names_law_a_not_total() {
    let r = compile_message(WORLD_CORE_OP_PATH, ":user::run-gate-c5");
    let msg = r.expect_err("a total core op in where must fail to compile on Law A");
    assert_eq!(
        msg,
        "compile-condition: where expr is not a rete primitive — ':wat::core::i64::>' is not a rete primitive; a where admits only :wat::rete:: ops"
    );
}
