//! Arc 209 C0b PREREQUISITE — structured peer death, PROCESS tier (Sub-stone B).
//!
//! The thread tier (Sub-stone A, shipped) now carries the `#wat.kernel/AssertionFailure`
//! envelope over its crash channel. This probe asks the empirical question for the process
//! tier: does a process peer's `assertion-failed!` — carrying a known `actual`/`expected` —
//! surface those STRUCTURED fields through `recv'`, or only the bare message?
//!
//! The process tier already emits a structured `#wat.kernel/ProcessPanics` / DiedError chain
//! envelope over its Err channel (`emit_structured_exit` → `conj_died_chain_value`), and the
//! arc 214 1b-ii-α probe proved `recv'` auto-raises a process crash reason (DivisionByZero).
//! The open question is whether the ASSERTION structure (actual/expected) rides that envelope.
//! GREEN here → Sub-stone B is already satisfied (document it). RED → B needs its own strike.
//!
//! Forks a `:process` child — run SERIALLY:
//!   `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death_process -- --test-threads=1`

use wat::freeze::call_beside;

/// A `:process` peer dies via `assertion-failed!` carrying a structured `actual` + `expected`.
/// `recv'` raises — the raised reason MUST carry BOTH structured fields, not just the message.
#[test]
fn process_peer_recv_surfaces_structured_actual_and_expected() {
    let err = match call_beside(file!(), ":user::compute") {
        Ok(v) => panic!("expected recv' to RAISE (the process peer crashed); got Ok({v:?})"),
        Err(e) => format!("{e:?}"),
    };
    assert!(
        err.contains("proc-structured-marker"), // rune:lint(loose-assert) — crash error embeds machine-specific absolute path (startup_beside/file!()) and source frame paths
        "the crash MESSAGE must travel on the process tier. got: {err}"
    );
    assert!(
        err.contains("PROC-ACTUAL-5521"), // rune:lint(loose-assert) — crash error embeds machine-specific absolute path (startup_beside/file!()); targeted sentinel check is the portable assertion
        "process tier: the structured `actual` must survive recv'. got: {err}"
    );
    assert!(
        err.contains("PROC-EXPECTED-8841"), // rune:lint(loose-assert) — crash error embeds machine-specific absolute path (startup_beside/file!()); targeted sentinel check is the portable assertion
        "process tier: the structured `expected` must survive recv'. got: {err}"
    );
}
