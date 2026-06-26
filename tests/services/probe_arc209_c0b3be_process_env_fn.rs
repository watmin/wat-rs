//! Arc 209 C0b.3b-e — the env-fn resolver: a source string → `user.program` record.
//!
//! `ProcessOpts` carries `env-fn`, a wat SOURCE STRING the spawned child evals IN ITS OWN
//! frozen world (post-load, just before `:user::main`) to produce `user.program`. The core is
//! `resolve_env_program(world, src)`: parse the string, eval it in `world`, and dispatch on the
//! result — a 0-arg fn → apply it; a `:wat::Record` (ANY subtype: `app::Env`, holon, …) → use
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
//!    a bespoke `:wat::Record` SUBTYPE flows — the gate is a variant match, not `== :wat::Record`).
//! 2. `resolves_bare_fn_by_applying_it`     — `"(fn [] -> :wat::Record (:app::Env 7))"` → `app::Env`
//!    (the 0-arg fn is applied).
//! 3. `default_empty_env_resolves`          — `"(:wat::program::EmptyEnv)"` → `wat::program::EmptyEnv`.
//! 4. `non_record_non_fn_is_an_error`       — `"(:wat::core::+ 1 2)"` → Err (must be a record).
//!
//! RED at HEAD: `resolve_env_program` does not exist (the dispatch is inline in
//! `run_user_main_in_child`, which is `-> !` and untestable).
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3be_process_env_fn

use std::sync::Arc;
use wat::freeze::{resolve_env_program, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

// A world that has `app::Env` (a :wat::Record SUBTYPE) + a named env-fn loaded — i.e. the
// type's code is present, exactly as it is in the spawned universe that runs the env-fn.
const PROGRAM: &str = r#"
(:wat::Record::def :app::Env [token <- :wat::core::i64])
(:wat::core::defn :app::make-env [] -> :wat::Record (:app::Env 7))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn world() -> wat::freeze::FrozenWorld {
    startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-e: env-fn resolver)")
}

fn assert_class(got: Value, expected_fqdn: &str, via: &str) {
    match got {
        Value::wat__Record { class_fqdn, .. } | Value::wat__holon__Record { class_fqdn, .. } => {
            assert_eq!(class_fqdn.as_str(), expected_fqdn, "via {via}")
        }
        other => panic!("expected a :wat::Record ({expected_fqdn}) via {via}; got {other:?}"),
    }
}

#[test]
fn resolves_named_call_to_subtype_record() {
    // A bespoke :wat::Record SUBTYPE (app::Env) flows through — proving the gate is a variant
    // match, not an exact `== :wat::Record`.
    let got = resolve_env_program(&world(), "(:app::make-env)").expect("resolve named call");
    assert_class(got, "app::Env", "named call");
}

#[test]
fn resolves_bare_fn_by_applying_it() {
    let got = resolve_env_program(&world(), "(:wat::core::fn [] -> :wat::Record (:app::Env 7))")
        .expect("resolve bare fn");
    assert_class(got, "app::Env", "bare fn (applied)");
}

#[test]
fn default_empty_env_resolves() {
    let got = resolve_env_program(&world(), "(:wat::program::EmptyEnv)").expect("resolve default");
    assert_class(got, "wat::program::EmptyEnv", "EmptyEnv default");
}

#[test]
fn non_record_non_fn_is_an_error() {
    // An env-fn that produces a non-record (here an i64) is rejected — user.program MUST be a
    // :wat::Record.
    let outcome = resolve_env_program(&world(), "(:wat::core::+ 1 2)");
    assert!(
        outcome.is_err(),
        "expected an error: env-fn must produce a :wat::Record, not an i64; got {outcome:?}"
    );
}
