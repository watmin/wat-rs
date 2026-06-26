//! Arc 232 follow-on — GENERIC protocol method signatures (the 267 sibling).
//!
//! 232 shipped protocol method dispatch for MONOMORPHIC sigs (probe_arc232_3 used
//! `greet [self loudness] -> String`). A method sig with a free type var — e.g.
//! `make<T> [self x <- :T] -> Vector<T>` — is NOT instantiated: the call-site checker
//! (check.rs:5506-5571) checks args against `sig.arg_types[i]` and returns `sig.ret`
//! DIRECTLY, so `:T` is treated as a literal `Path(":T")`, never bound to the caller's
//! type. (Confirmed: a `:T` arg/ret yields `expected :T, got :wat::core::i64`.)
//!
//! THE CALLER THAT SURFACED THIS: the arc-209 host seam. A host-agnostic `start [host <- :Host]`
//! needs the `Host` protocol's launch method to mediate `listener' :Op :Reply` over an abstract
//! host — i.e. a method generic over the service's `:Op`/`:Reply`. Same deferred-dep pattern as
//! arc 267 (parametric protocol bounds): 232 built the monomorphic mechanism, a caller surfaced
//! the generic need.
//!
//! THE FIX (mirror generic fns): collect the method's type params (the `<T>` suffix, like generic
//! fn `:name<T>` / 251.7 raw_type_params) at parse; instantiate them to fresh unification vars at
//! the call site (mirror `instantiate`, check.rs:5795) and unify args, returning the instantiated
//! ret.
//!
//! RED at HEAD: either `make<T>` doesn't parse, or `:T` isn't instantiated → the call fails to
//! type-check. GREEN once method type params are collected + instantiated.
//!
//! Run: cargo test --release -p wat --test probe_arc232_generic_method

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; A protocol with a GENERIC method: T is bound per-call (the arg type → the Vector element type).
(:wat::core::defprotocol :t::Maker
  (make<T> [self <- :t::Maker  x <- :T] -> :wat::core::Vector<T>))

(:wat::core::defrecord :t::Dup [])
(:wat::core::extend-type :t::Dup :t::Maker (make [self x] [x x]))

;; Call with a concrete i64 → T must instantiate to i64 → result is Vector<i64>.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::nth (:t::Maker/make (:t::Dup) 5) 0))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn generic_protocol_method_instantiates_at_call_site() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (generic protocol method make<T>: T binds to i64 at the call)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: make<T> called with i64 → T=i64 → Vector<i64>, nth 0 = 5; got {got:?}"
    );
}
