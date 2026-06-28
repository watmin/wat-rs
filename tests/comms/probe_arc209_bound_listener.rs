//! Arc 209 (protocol tooling, sub-stone 1) — `listener'` (thread tier) returns a
//! `Bound<S,R>` struct instead of the bare `Tuple<Listener'<S,R>, Address'<S,R>>`.
//!
//! `Bound` is a parametric STRUCT (not a record) because its fields are non-EDN
//! RustOpaque kernel entities (`Listener'`/`Address'`):
//!   (:wat::core::defstruct :wat::spawn::Bound<S,R>
//!     [listener <- :wat::kernel::Listener'<S,R>
//!      address  <- :wat::kernel::Address'<S,R>])
//! The thread tier of `listener'` builds it; the accessors `Bound/listener` and
//! `Bound/address` replace the positional `first`/`second` on the old tuple.
//!
//! This probe is `probe_arc209_c0b1b_select_listener` reduced to a single client,
//! with EXACTLY two lines changed: `(first pair)` → `(:wat::spawn::Bound/listener b)`
//! and `(second pair)` → `(:wat::spawn::Bound/address b)`. So a failure isolates
//! precisely to `Bound` — everything around it is the proven c0b1b round-trip.
//!
//! RED at HEAD: `:wat::spawn::Bound` is unregistered (no `defstruct`) AND `listener'`
//! returns a `Tuple` — so the `Bound/listener` / `Bound/address` accessors do not
//! resolve and the program fails to check on exactly that gap. GREEN once the
//! `defstruct` ships in `wat/spawn.wat` and `eval_listener_prime`'s thread tier
//! returns `Value::Struct{ ":wat::spawn::Bound", [listener, address] }`.
//!
//! Run SERIALLY (spawns a thread):
//!   cargo test --release -p wat --test probe_arc209_bound_listener -- --test-threads=1

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn listener_thread_tier_returns_bound_struct() {
    let world = startup_beside(file!())
        .expect("startup should succeed (Bound defstruct + listener' thread tier returns Bound)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(10)),
        "expected 10 = 5*2: Bound/listener fed serve's poll', Bound/address dialed the client, \
         the round-trip succeeded; got {got:?}"
    );
}
