//! FM 2-bis probe — arc 237 Stone 237.7a: `:wat::core::length` reborn as a `∀T` INTRINSIC.
//!
//! 237.7 RESHAPED (intrinsic-boundary doctrine, memory project_intrinsic_boundary): the
//! collection ops are NOT defclauses (closed universe — userland can't bind "any value");
//! they are Rust `∀T` intrinsics, the SAME shape as `:wat::core::type` (`∀T. T -> String`,
//! runtime.rs eval_type). This stone proves the recipe on ONE op: `length`.
//!
//! The change Sonnet makes: register `:wat::core::length` as a `∀T. T -> :i64` Rust builtin
//! (eval matches Value::Vector/HashMap/HashSet → len, else teaching error); DELETE the
//! `(:wat::core::define-dispatch :wat::core::length ...)` decl at core.wat:12. The per-type
//! leaves (`:Vector/length` etc.) and the DispatchRegistry STAY (other ops still tenant it).
//!
//! This probe is a BEHAVIOR REGRESSION GUARD — `length` works as a ∀T intrinsic
//! (Stone 237.7a shipped). The define-dispatch mechanism was retired at Stone 241.13.
//! These contracts remain green as proof the intrinsic path is solid.

//! Wat source: tests/function/probe_arc237_7a_length_intrinsic.wat

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, RuntimeErrorKind, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for 7a length-intrinsic fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed for positive test")
}

#[test]
fn length_vector() {
    assert_eq!(run(":user::length-vector"), Value::i64(3));
}

#[test]
fn length_vector_empty() {
    assert_eq!(run(":user::length-vector-empty"), Value::i64(0));
}

#[test]
fn length_vector_strings() {
    // element-agnostic — vector of strings has length 2
    assert_eq!(run(":user::length-vector-strings"), Value::i64(2));
}

#[test]
fn length_hashmap() {
    assert_eq!(run(":user::length-hashmap"), Value::i64(2));
}

#[test]
fn length_hashset() {
    assert_eq!(run(":user::length-hashset"), Value::i64(3));
}

#[test]
fn length_on_noncollection_errors() {
    // ∀T intrinsic accepts i64 at check time; raises a teaching error at eval (not a collection).
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":user::length-noncollection")
        .expect("fixture defines :user::length-noncollection")
        .clone();
    let result = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!());
    assert!(
        matches!(
            &result,
            Err(e) if matches!(
                e.kind(),
                RuntimeErrorKind::TypeMismatch { op, got, .. }
                    if op == ":wat::core::length" && got.type_name == "wat::core::i64"
            )
        ),
        "length on non-collection (i64) must error at runtime with RuntimeErrorKind::TypeMismatch{{op: \":wat::core::length\", got: i64}}; got {:?}",
        result
    );
}
