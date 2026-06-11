//! Arc 214 Stone 4.6a-i — typed-peer FOUNDATION (FM-2-bis disconfirming probe).
//!
//! The foundation the polymorphic peer verbs (4.6a-ii) project from:
//!   - `:wat::kernel::Thread'<I,O>` / `:wat::kernel::Process'<I,O>` registered as
//!     parametric type heads (mirror `Sender<T>`/`Receiver<T>`).
//!   - `(:wat::kernel::spawn-program' :tier env prog)` INFERS to the peer type at
//!     CHECK time — reading the program fn's `[I] -> O` signature
//!     (mirror `infer_make_channel`, check.rs:10423).
//!
//! ## Why the NEGATIVE probes are the disconfirming core (measured 2026-06-07)
//!
//! At HEAD, `spawn-program'` has NO check-side inference, and an unknown keyword
//! head infers a FRESH type var that unifies with ANY declared annotation — so a
//! positive "it type-checks against the peer annotation" probe is VACUOUS (it
//! passes with `:wat::core::i64` and the wrong tier's peer type too; verified
//! empirically). The discriminating probes are the negatives: a WRONG return
//! annotation must FAIL at check. Today it wrongly passes → probes 4/5 are RED;
//! the foundation's real inference turns them green.
//!
//! (The fresh-var leniency for unknown kernel heads is the same class that let
//! `+'2` escape check — arc 255's registry annihilates the class; this stone
//! closes it for this one verb.)
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

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
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

// ─── Probes 2/3: the peer annotations type-check (vacuously green at HEAD) ───

/// `spawn-program' :thread` against the Thread' annotation type-checks.
/// NOTE: green at HEAD by fresh-var vacuity (see module doc); load-bearing
/// only POST-foundation (it pins the positive path stays green once the
/// real inference exists). The discriminators are probes 4/5.
#[test]
fn probe_2_spawn_program_prime_thread_types_to_peer() {
    let src = r#"
        (:wat::core::defn :user::mk-echo-peer [] -> :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    startup_ok(src);
}

/// `:process` tier against the Process' annotation type-checks (same vacuity
/// note as probe 2).
#[test]
fn probe_3_spawn_program_prime_process_types_to_peer() {
    let src = r#"
        (:wat::core::defn :user::mk-echo-proc [] -> :wat::kernel::Process'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :process (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    startup_ok(src);
}

// ─── Probe 4 (LOAD-BEARING NEGATIVE): a wrong return annotation must FAIL ────

/// `spawn-program' :thread` declared as returning `:wat::core::i64` must be a
/// CHECK ERROR — the spawn's real type is `Thread'<i64,i64>`, not i64.
/// RED at HEAD: the unknown head's fresh var unifies with i64 and startup
/// wrongly succeeds.
#[test]
fn probe_4_wrong_scalar_return_annotation_rejected() {
    let src = r#"
        (:wat::core::defn :user::mk-wrong [] -> :wat::core::i64
          (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    let _err = startup_err(src);
}

// ─── Probe 5 (LOAD-BEARING NEGATIVE): cross-tier annotation must FAIL ────────

/// A `:thread` spawn declared as `Process'<...>` must be a CHECK ERROR — the
/// tier keyword picks the peer head. RED at HEAD (fresh-var vacuity).
#[test]
fn probe_5_cross_tier_annotation_rejected() {
    let src = r#"
        (:wat::core::defn :user::mk-cross [] -> :wat::kernel::Process'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
            (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input)))
    "#;
    let _err = startup_err(src);
}
