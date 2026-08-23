//! Arc 214 Stone 4.6a-i — typed-peer FOUNDATION (FM-2-bis disconfirming probe).
//!
//! The foundation the polymorphic peer verbs (4.6a-ii) project from:
//!   - `:wat::kernel::Thread<I,O>` / `:wat::kernel::Process<I,O>` registered as
//!     parametric type heads (mirror `Sender<T>`/`Receiver<T>`).
//!   - `(:wat::kernel::spawn-program :tier env prog)` INFERS to the peer type at
//!     CHECK time — reading the program fn's `[Peer'<S,R>] -> nil` signature
//!     (a type-keyword → parametric-tuple inference shape).
//!
//! ## Why the NEGATIVE probes are the disconfirming core (measured 2026-06-07)
//!
//! At HEAD, `spawn-program'` has NO check-side inference, and an unknown keyword
//! head infers a FRESH type var that unifies with ANY declared annotation — so a
//! positive "it type-checks against the peer annotation" probe is VACUOUS (it
//! passes with `:wat::core::i64` and the wrong tier's peer type too; verified
//! empirically). The discriminating probes are the negatives: a WRONG return
//! annotation must FAIL at check. Today it wrongly passes → probes 4/5 are RED;
//! the foundation's real inference turns them green.
//!
//! (The fresh-var leniency for unknown kernel heads is the same class that let
//! `+'2` escape check — arc 255's registry annihilates the class; this stone
//! closes it for this one verb.)
//!
//! ## Arc 259 S2c-ii-a — apply-loop PURGE
//!
//! All `:thread` spawn progs are now self-peer `[self <- Peer'<S,R>] -> nil`
//! (the only valid form post-purge). Probe_2 / probe_4 / probe_5 are SWAPPED.
//! The `Thread'<i64,i64>` peer type is preserved — `Peer'<O,I>=Peer'<i64,i64>`
//! → `Thread'<R,S>=Thread'<I,O>=Thread'<i64,i64>`. All type assertions unchanged.
//!
//! Wat fixtures:
//!   probe_arc214_stone46i_typed_peer_probe{2,3}.wat (positive startup),
//!   probe_arc214_stone46i_typed_peer_probe{4,5}.wat.bad (negative startup).
//!
//! Run: `cargo nextest run --release -E 'binary(types)' -F probe_arc214_stone46i_typed_peer`

use wat::freeze::startup_from_file;

// ─── Probe 1: the parametric peer type parses ────────────────────────────────

/// `:wat::kernel::Thread` must parse as a type-keyword (the parametric head
/// must be a registered/parseable type). Documents the registration target.
///
/// Arc 109 ③ — angle brackets are illegal; `Head<args>` has no flat-string
/// spelling any more, so `parse_type_expr` (a &str -> TypeExpr fn) can no
/// longer express a parametric reference at all — the surviving spelling
/// `(Head :- [args])` only parses from a structural `WatAST::List`, which
/// (outside the crate, `parse_type_node` being `pub(crate)`) means going
/// through the freeze pipeline instead. Same claim, driven end-to-end via
/// the co-located fixture (a typealias referencing the parametric head).
#[test]
fn probe_1_thread_peer_type_parses() {
    startup_from_file("tests/types/probe_arc214_stone46i_typed_peer_probe1.wat")
        .unwrap_or_else(|e| panic!(
            "startup should succeed — :wat::kernel::Thread must parse as a registered parametric type head; got error:\n{}\n---\n{:?}",
            e, e
        ));
}

// ─── Probes 2/3: the peer annotations type-check (vacuously green at HEAD) ───

/// `spawn-program' :thread` against the Thread' annotation type-checks.
/// NOTE: green at HEAD by fresh-var vacuity (see module doc); load-bearing
/// only POST-foundation (it pins the positive path stays green once the
/// real inference exists). The discriminators are probes 4/5.
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; annotation assertion unchanged.
#[test]
fn probe_2_spawn_program_prime_thread_types_to_peer() {
    startup_from_file("tests/types/probe_arc214_stone46i_typed_peer_probe2.wat")
        .unwrap_or_else(|e| panic!("startup should succeed; got error:\n{}\n---\n{:?}", e, e));
}

/// `:process` tier against the Process' annotation type-checks.
/// Arc 259 S2c-ii-b: migrated to 2-arg form with (:wat::spawn::process) host +
/// forms prog (the process defclause clause accepts Vector<wat::WatAST>).
#[test]
fn probe_3_spawn_program_prime_process_types_to_peer() {
    startup_from_file("tests/types/probe_arc214_stone46i_typed_peer_probe3.wat")
        .unwrap_or_else(|e| panic!("startup should succeed; got error:\n{}\n---\n{:?}", e, e));
}

// ─── Probe 4 (LOAD-BEARING NEGATIVE): a wrong return annotation must FAIL ────

/// `spawn-program' (:wat::spawn::thread)` declared as returning `:wat::core::i64`
/// must be a CHECK ERROR — the spawn's real type is `Thread'<i64,i64>`, not i64.
///
/// Arc 259 S2c-ii-b: migrated to 2-arg `(:wat::spawn::thread)` host form.
#[test]
fn probe_4_wrong_scalar_return_annotation_rejected() {
    let result = startup_from_file(
        "tests/types/probe_arc214_stone46i_typed_peer_probe4.wat.bad",
    );
    assert!(result.is_err(), "expected startup failure (wrong return type); got Ok");
}

// ─── Probe 5 (LOAD-BEARING NEGATIVE): cross-tier annotation must FAIL ────────

/// A `(:wat::spawn::thread)` spawn declared as `Process'<...>` must be a CHECK
/// ERROR — the host type selects the peer head (ThreadOpts → Thread').
///
/// Arc 259 S2c-ii-b: migrated to 2-arg `(:wat::spawn::thread)` host form.
#[test]
fn probe_5_cross_tier_annotation_rejected() {
    let result = startup_from_file(
        "tests/types/probe_arc214_stone46i_typed_peer_probe5.wat.bad",
    );
    assert!(result.is_err(), "expected startup failure (cross-tier annotation); got Ok");
}
