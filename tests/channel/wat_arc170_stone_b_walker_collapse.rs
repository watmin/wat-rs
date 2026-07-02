//! Arc 170 Stone B — walker collapse: hide `*_join-result` from user
//! namespace.
//!
//! Originally enforced by Stone B's hard-coded `validate_join_result_user_
//! namespace` walker; that ad-hoc rule was deleted in arc 198 slice 2
//! Stone 4 once arc 198's generic `walk_for_def_restricted_call` walker
//! covered the same callees (via Stone 3's `#[restricted_to(...)]`
//! attribute on `eval_kernel_*_join_result`). Stone 241.14 renamed the
//! walker to `walk_for_restricted_call` and migrated restriction storage
//! from `defined_value_restrictions` to `binding_metadata`. The enforcement
//! contract is UNCHANGED:
//!
//! - Caller's enclosing wat `define` FQDN starts with `:wat::` → ALLOWED
//! - Otherwise → compile error naming the offending callee verb plus
//!   the `DefRestrictedCallerNotAllowed` variant (name preserved per
//!   `feedback_inscription_immutable`).
//!
//! ## Tests
//!
//! - **Negative (Thread)**: user-namespace fn calls `Thread/join-result`
//!   → startup fails; error names the verb + arc 198's whitelist wording.
//! - **Negative (Process)**: same shape for Process.
//! - **Positive (Thread)**: `:wat::*` namespace fn calls
//!   `Thread/join-result` → startup succeeds.
//! - **Positive (Process)**: `:wat::*` namespace fn calls
//!   `Process/join-result` → startup succeeds.

use wat::freeze::{startup_bare, startup_from_file};

/// Returns the Debug-formatted error bundle from freezing a co-located NEGATIVE fixture that MUST
/// fail. Tests grep this for the new walker variant + message text.
fn startup_err(fixture_rel: &str) -> String {
    match startup_from_file(fixture_rel) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Negative cases — user-namespace callers MUST be rejected ──────────

#[test]
fn stone_b_user_namespace_thread_join_result_is_rejected() {
    // A user-namespace fn (`:my::test::call-thread-join`) reaches for
    // `:wat::kernel::Thread/join-result` directly. Post-arc-198, arc 198's
    // generic `walk_for_restricted_call` walker refuses (the callee
    // carries `#[restricted_to(":wat::")]` per arc 198 slice 2 Stone 3;
    // Stone 241.14 migrated restriction storage to binding_metadata);
    // the diagnostic names the callee + the allowed-caller whitelist.
    let err = startup_err("tests/channel/wat_arc170_stone_b_walker_collapse_thread_violation.wat");
    // Stone B (arc 296): Debug now emits EDN (the {:?}-impostor wall). Golden recaptured.
    assert_eq!(
        err,
        "#wat.check/CheckErrors {:message \"1 type-check error\" :location nil :causes [] :errors [#wat.check/DefRestrictedCallerNotAllowed {:message \"\u{60}:wat::kernel::Thread/join-result\u{60} has a restricted caller whitelist [:wat::]; the enclosing fn \u{60}:my::test::call-thread-join\u{60} does not match any entry (declared via \u{60}{:restricted-to [...]}\u{60} metadata-map). An entry ending in \u{60}::\u{60} is a namespace prefix (caller FQDN must start with it); an entry without trailing \u{60}::\u{60} is an exact-FQDN match. Either move the caller into one of the allowed namespaces, or add \u{60}:my::test::call-thread-join\u{60} to the \u{60}:restricted-to\u{60} list at the binding site.\" :location #wat.core/Span {:file \"tests/channel/wat_arc170_stone_b_walker_collapse_thread_violation.wat\" :line 4 :col 195 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 227}} :causes [] :callee \":wat::kernel::Thread/join-result\" :enclosing-fn \":my::test::call-thread-join\" :prefixes [\":wat::\"]}]}",
        "error must match arc 198 DefRestrictedCallerNotAllowed golden for Thread/join-result"
    );
}

#[test]
fn stone_b_user_namespace_process_join_result_is_rejected() {
    // Mirror of the Thread negative case for Process. Arc 198 slice 2
    // Stone 3 applied `#[restricted_to(":wat::")]` to
    // `eval_kernel_process_join_result`; arc 198's walker now enforces.
    let err = startup_err("tests/channel/wat_arc170_stone_b_walker_collapse_process_violation.wat");
    // Stone B (arc 296): Debug now emits EDN (the {:?}-impostor wall). Golden recaptured.
    assert_eq!(
        err,
        "#wat.check/CheckErrors {:message \"1 type-check error\" :location nil :causes [] :errors [#wat.check/DefRestrictedCallerNotAllowed {:message \"`:wat::kernel::Process/join-result` has a restricted caller whitelist [:wat::]; the enclosing fn `:my::test::call-process-join` does not match any entry (declared via `{:restricted-to [...]}` metadata-map). An entry ending in `::` is a namespace prefix (caller FQDN must start with it); an entry without trailing `::` is an exact-FQDN match. Either move the caller into one of the allowed namespaces, or add `:my::test::call-process-join` to the `:restricted-to` list at the binding site.\" :location #wat.core/Span {:file \"tests/channel/wat_arc170_stone_b_walker_collapse_process_violation.wat\" :line 4 :col 199 :end #wat.core.Option/Some #wat.core/Pos {:line 4 :col 232}} :causes [] :callee \":wat::kernel::Process/join-result\" :enclosing-fn \":my::test::call-process-join\" :prefixes [\":wat::\"]}]}",
        "error must match arc 198 DefRestrictedCallerNotAllowed golden for Process/join-result"
    );
}

// ─── Positive cases — substrate-namespace callers stay allowed ─────────

// The substrate stdlib loaded on every `startup_from_source` already
// contains substrate-namespace fns that call `Thread/join-result` and
// `Process/join-result` directly — for Thread, `:wat::test::run-thread-
// driver` at `wat/test.wat`; for Process, `:wat::test::run-hermetic-
// driver-with-io` at `wat/test.wat` (plus
// `:wat::kernel::run-sandboxed-ast` at `wat/kernel/sandbox.wat`
// (plus helpers in `wat/kernel/hermetic.wat`). A trivial user-code startup exercising the
// full freeze pipeline runs the new walker over those substrate bodies;
// IF the substrate-namespace exemption is broken, freeze fails with the
// new walker variant on those substrate bodies.
//
// These positive tests therefore prove the exemption holds by asserting
// that startup with trivial user-namespace source succeeds — the freeze
// implicitly runs the walker over the stdlib's substrate-namespace
// `*_join-result` calls and they must pass.

#[test]
fn stone_b_substrate_namespace_thread_join_result_is_allowed() {
    // Freeze exercises the walker over `:wat::test::run-thread-driver`
    // (wat/test.wat) and other substrate fns that call
    // `Thread/join-result`. If the substrate exemption fails, freeze
    // fails. Trivial user source + clean startup = exemption proven.
    startup_bare().expect("substrate-namespace join-result exemption must hold (freeze walks the stdlib)");
}

#[test]
fn stone_b_substrate_namespace_process_join_result_is_allowed() {
    // Mirror for Process — the substrate's `wat/kernel/sandbox.wat` and
    // `wat/kernel/hermetic.wat` call `Process/join-result` directly. The
    // freeze pipeline walks them; the new walker must not fire on those
    // substrate-namespace bodies.
    startup_bare().expect("substrate-namespace join-result exemption must hold (freeze walks the stdlib)");
}
