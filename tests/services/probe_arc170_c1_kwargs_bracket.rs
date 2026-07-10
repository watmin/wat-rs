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

use wat::freeze::{invoke_user_main, startup_from_file};
use wat::runtime::Value;

#[test]
fn c1_kwargs_workfn_invoked_with_dialed_peer() {
    let world = startup_from_file("tests/services/probe_arc170_c1_kwargs_bracket.wat")
        .expect("startup should succeed (arc 170 C1 kwargs: bracket-walk fixture)");
    // Drive via `invoke_user_main` (not a `parse_one!`-string) so this test inlines no wat form;
    // `:user::main` returns the concatenated reply String directly.
    let got = invoke_user_main(&world, Vec::new())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
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
