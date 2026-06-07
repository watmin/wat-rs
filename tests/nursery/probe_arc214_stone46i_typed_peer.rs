//! Arc 214 Stone 4.6a-i — typed-peer FOUNDATION (FM-2-bis disconfirming probe).
//!
//! The foundation the polymorphic peer verbs (4.6a-ii) project from:
//!   - `:wat::kernel::Thread'<I,O>` / `:wat::kernel::Process'<I,O>` registered as
//!     parametric type heads (mirror `Sender<T>`/`Receiver<T>`).
//!   - `(:wat::kernel::spawn-program' :tier env prog)` INFERS to the peer type at
//!     CHECK time — reading the program fn's `[I] -> O` signature
//!     (mirror `infer_make_channel`, check.rs:10423).
//!
//! At HEAD: `spawn-program'` has NO check-side inference (the 4.5 prime is
//! runtime-only — only the legacy `spawn-program`/`-ast` appear in check.rs).
//! So Probe 2 (the load-bearing one) is RED until the foundation lands.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone46i_typed_peer`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::types::parse_type_expr;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn startup_ok(src: &str) {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup should succeed; got error:\n{}\n---\n{:?}", e, e));
}

// ─── Probe 1: the parametric peer type parses ────────────────────────────────

/// `:wat::kernel::Thread'` must parse as a type-keyword (the parametric head
/// must be a registered/parseable type). Documents the registration target.
#[test]
fn probe_1_thread_peer_type_parses() {
    let result = parse_type_expr(":wat::kernel::Thread'<wat::core::i64,wat::core::i64>");
    assert!(
        result.is_ok(),
        "parse_type_expr(:wat::kernel::Thread'<i64,i64>) must return Ok; got {:?}",
        result
    );
}

// ─── Probe 2 (LOAD-BEARING): spawn-program' :thread infers to the peer type ───

/// A function declaring it returns `:wat::kernel::Thread'<i64,i64>` whose body is
/// `(spawn-program' :thread {} <echo-fn>)` must type-check — i.e. the inferred
/// type of the spawn expression must BE the thread-peer parametric type, read
/// from the program fn's `[i64] -> i64` signature.
///
/// RED at HEAD: `spawn-program'` has no check-side inference, so the body's type
/// does not match the declared return type (or the head is unrecognized).
#[test]
fn probe_2_spawn_program_prime_thread_types_to_peer() {
    let src = r#"
        (:wat::core::defn :user::mk-echo-peer [] -> :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :thread {}
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    startup_ok(src);
}

// ─── Probe 3 (LOAD-BEARING): the :process tier infers to its own peer type ────

/// The `:process` tier of the same form must infer to `Process'<i64,i64>`, not
/// `Thread'`. Dispatch on the `:tier` keyword at CHECK time.
#[test]
fn probe_3_spawn_program_prime_process_types_to_peer() {
    let src = r#"
        (:wat::core::defn :user::mk-echo-proc [] -> :wat::kernel::Process'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :process {}
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    startup_ok(src);
}
