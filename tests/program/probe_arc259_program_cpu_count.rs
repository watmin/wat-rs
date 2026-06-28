//! Arc 259 S3.2b-i — `(:wat::program::cpu-count)`: the LIVE host-parallelism verb.
//!
//! cpu-count is a host fact wat surfaces two ways, mirroring time:
//!   - the STAMPED env field `wat.cpu-count` (snapshot at peer-start; interrogation)
//!     — already exists, read via `(:wat::program::env)` (needs a seam-installed env);
//!   - the LIVE verb `(:wat::program::cpu-count)` (always available; pool sizing)
//!     — THIS stone.
//! Like `(:wat::time::now)` (the live twin of the stamped `wat.started-at`), the verb
//! needs NO installed program env — it answers `std::thread::available_parallelism()`
//! directly, in ANY eval context. The brackets pool sizes its default runner count
//! from it: the env field is unreachable without a seam install, and a pool must be
//! able to size itself anywhere it runs.
//!
//! RED at HEAD: `:wat::program::cpu-count` does not exist (UnknownFunction).
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_program_cpu_count.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run: `cargo test --release --test program probe_arc259_program_cpu_count`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// The verb returns the real host parallelism as i64 — WITHOUT any installed
/// program env (bare `eval_in_frozen`, like every nursery probe). This is the
/// exact property the brackets pool relies on: cpu-count is answerable anywhere.
#[test]
fn cpu_count_is_live_and_install_free() {
    let expected = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    let world = startup_beside(file!()).expect("startup/check should succeed");
    let ast = wat::parse_one!("(:probe::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval — cpu-count needs no installed program env")
        .value_owned();
    assert_eq!(
        got,
        Value::i64(expected),
        "(:wat::program::cpu-count) returns available_parallelism(), live and install-free"
    );
}
