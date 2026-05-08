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
//!   4. `vector_at_value_position_errors_clearly` — startup-time error
//!      contains the literal "vector literals at value position are not
//!      supported" string
//!   5. `vector_at_value_position_in_define_body_errors` — same error
//!      surfaces when the Vector is inside a `:wat::core::define` body

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::parse_one;

/// Asserts startup fails and returns the Debug-formatted error string.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Test 1 — top-level vector parses as Vector ────────────────────────────

/// `[1 2 3]` must parse as `WatAST::Vector` with three integer
/// children — NOT as `WatAST::List`. Verifies the parser's bracket
/// path fires.
#[test]
fn vector_at_top_level_parses_as_vector() {
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

// ─── Test 4 — vector at value position errors clearly ─────────────────────

/// A Vector at top-level value position fires the substrate error
/// describing why vector literals are not yet supported. Error
/// message contains the literal "vector literals at value position
/// are not supported" string per BRIEF + scorecard row L.
#[test]
fn vector_at_value_position_errors_clearly() {
    // Wrap in a define so startup actually evaluates / checks the
    // body. A bare top-level `[1 2 3]` would also error at parse
    // / check time, but the define wrapper exercises the same
    // error path through the standard pipeline.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          [1 2 3])
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("vector literals at value position are not supported"),
        "expected 'vector literals at value position are not supported' in error; \
         got: {}",
        err
    );
}

// ─── Test 5 — vector in define body errors with same message ──────────────

/// Confirms test 4's error path also fires when the vector is the
/// body expression of a `:wat::core::define` whose declared return
/// type is `Vector<i64>`. The literal-vector-as-value path errors
/// at type-check time (the body's type can't be inferred, so the
/// declared-vs-actual return type unification doesn't even reach
/// the discriminant).
#[test]
fn vector_at_value_position_in_define_body_errors() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::Vector<wat::core::i64>)
          [1 2 3])
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("vector literals at value position are not supported"),
        "expected 'vector literals at value position are not supported' in error; \
         got: {}",
        err
    );
}
