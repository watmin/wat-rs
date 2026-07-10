//! Arc 170 C1 kwargs — a kwargs work-fn invoked via its companion `:key val` call, on a runtime-
//! DIALED peer bound to the `:key`, served on a real spawned process worker.
//!
//! Promoted from `scratchpad/probe-c1-kwargs-invoke.wat` (GREEN today: prints `echo:a echo:b
//! echo:c`). The worker's serve loop holds a peer dialed at runtime via `Setup` (`connect'`), then
//! its `Work` arm invokes the kwargs work-fn `:probe::work` through the COMPANION `:key val` call
//!   (:probe::work s :echo held)
//! instead of calling the surface feature directly — proving the AST-walk synthesizes exactly this
//! call shape (item positional + `:key <dialed-peer>` per `uses` field). Commit `b0a1a211` (C1
//! N=1) shipped the bracket walk with no committed test; this is its load-bearing crux.
//!
//! This test FORKS a process (spawn-program' (process)) → run with --test-threads=1:
//! cargo nextest run -p wat -E 'test(/probe_arc170_c1_kwargs_bracket/)' --test-threads=1
//! Driven via `invoke_user_main` (not an inline `parse_one!`) so it trips no `no_inlined_wat` lint.

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn c1_kwargs_workfn_invoked_with_dialed_peer() {
    let world = startup_from_file("tests/services/probe_arc170_c1_kwargs_bracket.wat")
        .expect("startup should succeed (arc 170 C1 kwargs: bracket-walk fixture)");
    // `:probe::run` (a non-main defn — the fixture carries no top-level `:user::main`, per the
    // arc-170 `[] -> :nil` / UselessMain wall) returns the concatenated reply String directly.
    // Eval it via a PROGRAMMATICALLY built call AST (not a `parse_one!`-string), so this test
    // inlines no wat form (no_inlined_wat clean).
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::run".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("run raised: {e:?}"))
        .value_owned();
    match got {
        Value::String(ref s) if s.as_str() == "echo:a echo:b echo:c" => {
            // the kwargs work-fn was invoked via the companion :key val call on the
            // runtime-dialed peer, for all three Work round-trips.
        }
        other => panic!(
            "expected Ok \"echo:a echo:b echo:c\": the worker's Work arm invokes the kwargs \
             work-fn :probe::work via the companion :key val call (:probe::work s :echo held) \
             with a peer dialed at runtime from Setup. got {other:?}"
        ),
    }
}
