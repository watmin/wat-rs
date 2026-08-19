//! Arc 209 C0b.3b-e — the env-fn resolver: a source string → `user-data` record.
//!
//! `ProcessOpts` carries `env-fn`, a wat SOURCE STRING the spawned child evals IN ITS OWN
//! frozen world (post-load, just before `:user::main`) to produce `user-data`. The core is
//! `resolve_env_program(world, src)`: parse the string, eval it in `world`, and dispatch on the
//! result — a 0-arg fn → apply it; a `:wat::core::Record` (ANY subtype: `app::Env`, holon, …) → use
//! directly; anything else → error. The result feeds the 3b-d seam
//! (`invoke_user_main_with_program`).
//!
//! THE OBSERVABLE RESPECTS THE TYPE-LOADING REQUIREMENT (builder, 2026-06-13): a rich record can
//! only be USED by a universe that has its type loaded. The env-fn runs in the spawned universe
//! (which has the forms/types); so the honest test observes `resolve_env_program` IN-PROCESS,
//! against a world that DEFINES the type — exactly how it runs. No record is shipped to a
//! type-less consumer (that SHOULD fail; it's the requirement, not a gap). The full child-side
//! wiring (spawn-process' → child seam → invoke_user_main_with_program) is composition with the
//! shipped 3b-d seam.
//!
//! Four proofs:
//! 1. `resolves_named_call_to_subtype_record` — `"(:app::make-env)"` → `app::Env` (record branch;
//!    a bespoke `:wat::core::Record` SUBTYPE flows — the gate is a variant match, not `== :wat::core::Record`).
//! 2. `resolves_bare_fn_by_applying_it`     — `"(fn [] -> :wat::core::Record (:app::Env :token 7))"` → `app::Env`
//!    (the 0-arg fn is applied).
//! 3. `default_empty_env_resolves`          — `"(:wat::program::EmptyEnv)"` → `wat::program::EmptyEnv`.
//! 4. `non_record_non_fn_is_an_error`       — `"(:wat::core::+ 1 2)"` → Err (must be a record).
//!
//! RED at HEAD: `resolve_env_program` does not exist (the dispatch is inline in
//! `run_user_main_in_child`, which is `-> !` and untestable).
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3be_process_env_fn

use wat::freeze::{resolve_env_program, startup_beside};
use wat::runtime::Value;
use wat::types::Nature;

// A world that has `app::Env` (a :wat::core::Record SUBTYPE) + a named env-fn loaded — i.e. the
// type's code is present, exactly as it is in the spawned universe that runs the env-fn.
// Wat source lives in the co-located fixture: probe_arc209_c0b3be_process_env_fn.wat

fn world() -> wat::freeze::FrozenWorld {
    startup_beside(file!()).expect("startup should succeed (C0b.3b-e: env-fn resolver)")
}

fn assert_class(got: Value, expected_fqdn: &str, via: &str) {
    match got {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            assert_eq!(a.class.as_ref(), expected_fqdn, "via {via}")
        }
        other => panic!("expected a :wat::core::Record ({expected_fqdn}) via {via}; got {other:?}"),
    }
}

// The four env-fn source strings under test live in co-located .wat fixtures (no inline wat in
// this .rs) — resolve_env_program's own contract is "a source string -> user-data record", so
// the fixture content IS the raw source text it parses, loaded via include_str! rather than typed
// as a Rust string literal.

#[test]
fn resolves_named_call_to_subtype_record() {
    // A bespoke :wat::core::Record SUBTYPE (app::Env) flows through — proving the gate is a variant
    // match, not an exact `== :wat::core::Record`.
    let src = include_str!("probe_arc209_c0b3be_process_env_fn_named_call.wat");
    let got = resolve_env_program(&world(), src.trim()).expect("resolve named call");
    assert_class(got, "app::Env", "named call");
}

#[test]
fn resolves_bare_fn_by_applying_it() {
    let src = include_str!("probe_arc209_c0b3be_process_env_fn_bare_fn.wat");
    let got = resolve_env_program(&world(), src.trim()).expect("resolve bare fn");
    assert_class(got, "app::Env", "bare fn (applied)");
}

#[test]
fn default_empty_env_resolves() {
    let src = include_str!("probe_arc209_c0b3be_process_env_fn_default.wat");
    let got = resolve_env_program(&world(), src.trim()).expect("resolve default");
    assert_class(got, "wat::program::EmptyEnv", "EmptyEnv default");
}

#[test]
fn non_record_non_fn_is_an_error() {
    // An env-fn that produces a non-record (here an i64) is rejected — user-data MUST be a
    // :wat::core::Record.
    let src = include_str!("probe_arc209_c0b3be_process_env_fn_non_record.wat");
    let outcome = resolve_env_program(&world(), src.trim());
    assert!(
        outcome.is_err(),
        "expected an error: env-fn must produce a :wat::core::Record, not an i64; got {outcome:?}"
    );
}
