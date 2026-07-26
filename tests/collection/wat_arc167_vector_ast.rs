//! Integration tests for arc 167 slice 1 — `WatAST::Vector`
//! substrate foundation.
//!
//! Slice 1 mints `WatAST::Vector` as a first-class AST node distinct
//! from `WatAST::List`. The parser produces `WatAST::Vector` from
//! `[...]` forms; eval and check error clearly when a Vector appears
//! at value position (slice 2 wires the legal consumers in
//! `:wat::core::fn` / `:wat::core::defn` signature positions).
//!
//! Five test cases:
//!   1. `vector_at_top_level_parses_as_vector` — `[1 2 3]` parses cleanly
//!      to `WatAST::Vector`
//!   2. `empty_vector_parses` — `[]` parses as empty Vector
//!   3. `nested_vector_in_list_parses` — `(:foo [1 2 3])` parses with
//!      the inner `[1 2 3]` as a Vector child of the outer List
//!   4. `vector_at_value_position_works_after_arc215` — startup-time success
//!      (arc 215 stone 2: `[1 2 3]` at expression position now works)
//!   5. `vector_at_value_position_in_define_body_works_after_arc215` — same
//!
//! Tests 4 and 5 share the co-located fixture: wat_arc167_vector_ast.wat,
//! driven via call_beside_value(file!(), ":my::probe").

use wat::ast::WatAST;
use wat::freeze::call_beside_value;
use wat::parse_one;
use wat::runtime::Value;

// ─── Test 1 — top-level vector parses as Vector ────────────────────────────

/// `[1 2 3]` must parse as `WatAST::Vector` with three integer
/// children — NOT as `WatAST::List`. Verifies the parser's bracket
/// path fires.
#[test]
fn vector_at_top_level_parses_as_vector() {
    // rune:lint(no-inlined-wat) — this probe IS the parser (raw literal in, WatAST-shape
    // out); the subject is the reader's bracket-vs-list dispatch, not an evaluated value.
    // rune:lint(no-inlined-edn) — input under test: literal fed to parse_one!; the subject is the reader's bracket-vs-list dispatch, not an evaluated value.
    let parsed = parse_one!("[1 2 3]").expect("parse");
    match parsed {
        WatAST::Vector(items, _) => {
            assert_eq!(items.len(), 3, "expected 3 items, got {}", items.len());
            assert!(
                matches!(items[0], WatAST::IntLit(1, _)),
                "expected IntLit(1) at index 0, got {:?}",
                items[0]
            );
            assert!(
                matches!(items[1], WatAST::IntLit(2, _)),
                "expected IntLit(2) at index 1, got {:?}",
                items[1]
            );
            assert!(
                matches!(items[2], WatAST::IntLit(3, _)),
                "expected IntLit(3) at index 2, got {:?}",
                items[2]
            );
        }
        other => panic!("expected WatAST::Vector; got {:?}", other),
    }
}

// ─── Test 2 — empty vector parses ──────────────────────────────────────────

/// `[]` must parse as an empty Vector — distinct from `()` which is
/// the unit value (empty List). The substrate distinguishes the two
/// cleanly.
#[test]
fn empty_vector_parses() {
    // rune:lint(no-inlined-wat) — this probe IS the parser (raw literal in, WatAST-shape
    // out); the subject is the reader's bracket-vs-list dispatch, not an evaluated value.
    // rune:lint(no-inlined-edn) — input under test: literal fed to parse_one!; the subject is the reader's empty-vector parse path, not an evaluated value.
    let parsed = parse_one!("[]").expect("parse");
    match parsed {
        WatAST::Vector(items, _) => {
            assert!(
                items.is_empty(),
                "expected empty Vector, got {} items",
                items.len()
            );
        }
        other => panic!("expected WatAST::Vector(empty); got {:?}", other),
    }
}

// ─── Test 3 — nested vector in list parses ─────────────────────────────────

/// `(:foo [1 2 3])` must parse as a List whose second child is a
/// Vector. Verifies the bracket parser composes inside list bodies.
#[test]
fn nested_vector_in_list_parses() {
    // rune:lint(no-inlined-wat) — this probe IS the parser (raw literal in, WatAST-shape
    // out); the subject is the reader's bracket-vs-list dispatch, not an evaluated value.
    let parsed = parse_one!("(:foo [1 2 3])").expect("parse");
    let items = match parsed {
        WatAST::List(items, _) => items,
        other => panic!("expected outer WatAST::List; got {:?}", other),
    };
    assert_eq!(items.len(), 2, "expected outer list of 2 items");
    assert!(
        matches!(&items[0], WatAST::Keyword(k, _) if k == ":foo"),
        "expected :foo head; got {:?}",
        items[0]
    );
    match &items[1] {
        WatAST::Vector(vec_items, _) => {
            assert_eq!(vec_items.len(), 3, "expected 3 Vector children");
            assert!(matches!(vec_items[0], WatAST::IntLit(1, _)));
            assert!(matches!(vec_items[1], WatAST::IntLit(2, _)));
            assert!(matches!(vec_items[2], WatAST::IntLit(3, _)));
        }
        other => panic!("expected WatAST::Vector child; got {:?}", other),
    }
}

// ─── Test 4 — vector at value position (arc 215 stone 2 update) ──────────

/// Arc 215 stone 2 — `[...]` at expression/value position NOW WORKS.
///
/// HISTORICAL NOTE: This test previously asserted `startup_err` with the
/// message "vector literals at value position are not supported". Arc 167
/// slice 1 said "a future arc enables vector literals as Value::Vec values."
/// Arc 215 stone 2 is that future arc: `WatAST::Vector` at expression position
/// now routes through `infer_list_constructor` with `:wat::type::Infer`.
///
/// This test is updated to assert SUCCESS: `[1 2 3]` in a define body
/// type-checks as `Vec<i64>` and evaluates to length 3 at runtime.
#[test]
fn vector_at_value_position_works_after_arc215() {
    match call_beside_value(file!(), ":my::probe")
        .expect("arc 215 stone 2: [1 2 3] at value position must type-check + eval")
    {
        Value::i64(n) => assert_eq!(n, 3, "length of [1 2 3] must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 5 — vector literal as Vec<i64> return (arc 215 stone 2 update) ──

/// Arc 215 stone 2 — `[1 2 3]` as define body with explicit `Vector<i64>`
/// return type now type-checks and evaluates correctly.
///
/// HISTORICAL NOTE: Previously asserted "vector literals at value position
/// are not supported" startup error. That error path is retired by arc 215
/// stone 2. The test now verifies the happy path: define returns Vec<i64>;
/// length is 3.
#[test]
fn vector_at_value_position_in_define_body_works_after_arc215() {
    // Shares the co-located fixture with test 4.
    match call_beside_value(file!(), ":my::probe")
        .expect("arc 215 stone 2: [1 2 3] in define body must type-check + eval")
    {
        Value::i64(n) => assert_eq!(n, 3, "length must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}
