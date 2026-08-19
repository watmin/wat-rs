//! Stone 118.B2c strike 1 — **THE REACHABILITY WALL**: a `defclause` arm that can never be
//! selected is refused at registration.
//!
//! Fixtures are the co-located `_neg.wat.bad` (must fail to register) and the bare `.wat` sibling (must register
//! and dispatch). Design: `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/`
//! `DESIGN-STONE-118.B2c-a-surface-typed-clause-arm-never-dispatches.md`.
//!
//! ## What this closes
//!
//! Dispatch is first-match-wins in declaration order. Before this wall,
//! `(defclause :my::pick ([x <- :i64] "FIRST") ([x <- :i64] "SECOND"))` type-checked, ran, and
//! answered `"FIRST"` forever — the second body dead code, silently, with no error and no warning.
//!
//! ★ **That was a hole in the redef rule.** Arc 054 made `typealias`/`define`/`defmacro`
//! *"if byte-equivalent, no-op"*, else `DuplicateDefine`; clause ARMS were never covered, because
//! an arm is not a definition BY NAME — so the one registry that dispatches on TYPES had no
//! define-once rule. An arm that can never fire is a definition with no effect. Builder,
//! 2026-08-18: *"you may only express something's def once and all other attempts must be
//! identical."*
//!
//! This file REPLACES `probe_stone_118_b2c_overlapping_arms_are_silent`, which asserted the defect
//! and was written to invert here. It did.
//!
//! ## ★★ Why `fallback_shape_still_registers` is the load-bearing test
//!
//! The wall's first predicate was **INTERSECTION**, and it was wrong. Two arms whose domains merely
//! intersect — a concrete arm then a type-var catch-all — are a legitimate FALLBACK: the later arm
//! still fires for the rest of its domain. `wat/bracket.wat`'s `thread-enter` and
//! `process-work-forms` are exactly that shape, and its comment (`:314-316`) names first-match-wins,
//! calls the generic arm a "PERMISSIVE catch-all", and states that ordering is load-bearing. An
//! intersection wall would have outlawed the stdlib.
//!
//! The rule is **SUBSUMPTION**: refuse only when an earlier arm accepts *everything* a later one
//! accepts. `fallback_shape_still_registers` is what holds that line.
//! `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

use wat::freeze::{call_beside_value, startup_from_file};

const NEG: &str = "tests/types/probe_stone_118_b2c_unreachable_arm_refused_neg.wat.bad";

fn call_string(entry: &str) -> String {
    let v = call_beside_value(file!(), entry)
        .unwrap_or_else(|e| panic!("{entry} must evaluate; got {e:?}"));
    match v {
        wat::Value::String(s) => s.to_string(),
        other => panic!("{entry}: expected a String, got {other:?}"),
    }
}

#[test]
fn unreachable_arm_is_refused_at_registration() {
    let err = startup_from_file(NEG)
        .expect_err("an arm no input can reach must be refused at registration, not run");
    let rendered = format!("{err:?}");

    // The error must NAME the defect and BOTH arms — "it failed" is not a characterization.
    // Which arm is dead, and which one shadows it, is the whole diagnostic.
    for needle in [
        "UnreachableClause",
        ":my::pick",
        "can never be selected",
        "first-match-wins",
    ] {
        assert!(
            rendered.contains(needle),
            "the refusal must name {needle:?} — got: {rendered}"
        );
    }
}

#[test]
fn disjoint_arms_still_register_and_dispatch() {
    assert_eq!(call_string(":my::describe-int"), "an int");
    assert_eq!(call_string(":my::describe-string"), "a string");
    assert_eq!(call_string(":my::describe-bool"), "a bool");
}

/// ★★ The line the wall must not cross. A concrete arm followed by a type-var catch-all: the
/// domains INTERSECT, the later arm is still REACHABLE for every non-keyword, and it must register.
/// This is `wat/bracket.wat`'s shape. If this goes red, the wall has reverted to refusing
/// intersection and the stdlib is next.
#[test]
fn fallback_shape_still_registers_and_both_arms_fire() {
    assert_eq!(
        call_string(":my::route-keyword"),
        "specific",
        "the concrete arm is declared first and must win for a keyword"
    );
    assert_eq!(
        call_string(":my::route-other"),
        "generic",
        "the type-var catch-all must still fire for everything else — that is what makes it \
         REACHABLE and therefore legal"
    );
}
