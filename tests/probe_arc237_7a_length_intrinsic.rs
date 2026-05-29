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
//! This probe is a BEHAVIOR REGRESSION GUARD — `length` works TODAY via define-dispatch and
//! must work IDENTICALLY after the mechanism swap. Green before AND after. The mechanism
//! change itself (decl gone / builtin present) is verified by EXPECTATIONS grep, not here.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval `(:wat::core::length <coll>)` declared `-> :i64`; return the i64 or an error string.
fn length_of(coll: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {coll}))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {:?}", other)),
    }
}

/// True iff `(:wat::core::length <expr>)` is an error at SOME phase (check or eval) — used for
/// the non-collection case (phase-agnostic: it must be rejected, before and after the swap).
fn length_errors(expr: &str) -> bool {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {expr}))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
    );
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Err(_) => true, // check-time rejection
        Ok(world) => {
            let ast = wat::parse_one!("(:user::compute)").expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new()).is_err() // runtime rejection
        }
    }
}

#[test] fn length_vector() { assert_eq!(length_of("[1 2 3]"), Ok(3)); }
#[test] fn length_vector_empty() { assert_eq!(length_of("[]"), Ok(0)); }
#[test] fn length_vector_strings() { assert_eq!(length_of("[\"a\" \"b\"]"), Ok(2)); } // element-agnostic
#[test] fn length_hashmap() { assert_eq!(length_of("{:a 1 :b 2}"), Ok(2)); }
#[test] fn length_hashset() { assert_eq!(length_of("(:wat::core::HashSet :wat::core::i64 1 2 3)"), Ok(3)); }
#[test] fn length_on_noncollection_errors() { assert!(length_errors("5")); }
