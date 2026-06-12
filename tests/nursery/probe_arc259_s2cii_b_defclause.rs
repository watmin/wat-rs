//! Arc 259 S2c-ii-b — `spawn-program'` as the host-type `defclause` (FM-2-bis probe).
//!
//! The keystone: `spawn-program'` becomes a 2-arg `(host prog)` wat defclause
//! dispatching on the host record type (`ThreadOpts`/`ProcessOpts`), retiring the
//! 3-arg Rust intrinsic. Unblocked by S2c-ii.0 (class_fqdn dispatch); simplified by
//! S2c-ii-a (apply-loop purged → one thread clause, no overlap). Full design:
//! `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2c-ii.md`.
//!
//! ## Why this is RED at HEAD
//!
//! At HEAD `spawn-program'` is the 3-arg `(:tier env prog)` intrinsic; the 2-arg
//! `(spawn-program' (thread) prog)` form is an arity mismatch. Post-S2c-ii-b the
//! defclause dispatches on `(thread)`'s `ThreadOpts` type → spawns → 42.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2cii_b`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(src: &str) -> i64 {
    let src = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup (RED at HEAD: spawn-program' is the 3-arg intrinsic; 2-arg is arity-mismatch)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// LOAD-BEARING: the 2-arg `(spawn-program' (thread) <self-peer-prog>)` form
/// dispatches on the `ThreadOpts` host type and round-trips 42.
#[test]
fn s2cii_b_two_arg_host_dispatch() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                                   (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                     (:wat::kernel::send' self (:wat::kernel::recv' self))))
                            _ (:wat::kernel::send' peer 42)
                            got (:wat::kernel::recv' peer)]
            got))
    "#;
    assert_eq!(run_compute_i64(src), 42, "(spawn-program' (thread) prog) host-dispatches + echoes 42");
}
