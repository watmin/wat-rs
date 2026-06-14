//! Arc 209 C0b.3b-e — process-tier `user.program` injection via an env-fn source string.
//!
//! 3b-d shipped the seam (`invoke_user_main_with_program`). This stone wires the PROCESS tier:
//! `ProcessOpts` carries an `env-fn` — a wat SOURCE STRING — that the spawned child evals in its
//! own frozen world to produce `user.program`. A string crosses the clone3 fork trivially; the
//! child resolves everything against its own loaded forms. The eval result is dispatched: a 0-arg
//! fn → applied; a `:wat::Record` → used directly; else error. So `(process/env "(my/make-env)")`
//! (named call), `(process/env "(fn [] -> :wat::Record …)")` (bare anon fn), and
//! `(process/env "(my/Cfg 99)")` (direct ctor expr) all work. `(process)` defaults env-fn to
//! `"(:wat::program::EmptyEnv)"`.
//!
//! TWO proofs (both dispatch branches), each: the spawned child's `:user::main` reads
//! `user.program` and sends it back over its self-peer; the owner recv's the Value and asserts it
//! is the injected `:child::Cfg` record (class_fqdn), not `EmptyEnv`.
//! 1. `env_fn_as_bare_fn` — env-fn is a bare `(fn [] …)` → evals to a fn → child applies it.
//! 2. `env_fn_as_call_expr` — env-fn is `(:child::make-env)` → evals to a `:wat::Record` directly.
//!
//! RED at HEAD: `(:wat::spawn::process/env …)` is an unknown ctor → the spawn fails.
//!
//! These tests FORK (spawn-program' (process)). Run:
//! cargo test --release -p wat --test probe_arc209_c0b3be_process_env_fn -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The spawned child's forms: a Cfg record + an env-fn + a main that ships user.program home.
// (SERVICE_FORMS is interpolated into each program's spawn-program' call.)
const CHILD_FORMS: &str = r#"
             (:wat::Record::def :child::Cfg [token <- :wat::core::i64])
             (:wat::core::defn :child::make-env [] -> :wat::Record (:child::Cfg 99))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [up   (:wat::program::Env/user.program (:wat::program::env))
                  self (:wat::program::self-peer :wat::Record :wat::core::i64)
                  _    (:wat::kernel::send' self up)]
                 nil))
"#;

fn program_with_env_fn(env_fn: &str) -> String {
    format!(
        r#"
(:wat::core::defn :user::compute [] -> :wat::Record
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process/env "{env_fn}")
           (:wat::core::forms
{CHILD_FORMS}))
     up  (:wat::kernel::recv' svc)]
    up))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#
    )
}

fn run_and_get_user_program(env_fn: &str) -> Value {
    let program = program_with_env_fn(env_fn);
    let world = startup_from_source(&program, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-e: process env-fn)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"))
}

fn assert_is_child_cfg(got: Value, via: &str) {
    match got {
        Value::wat__Record { class_fqdn, .. } => assert_eq!(
            class_fqdn.as_str(),
            "child::Cfg",
            "expected the child's user.program to be the injected :child::Cfg (via {via}), \
             not EmptyEnv"
        ),
        other => panic!("expected the injected :child::Cfg record via {via}; got {other:?}"),
    }
}

#[test]
fn env_fn_as_bare_fn() {
    // env-fn evals to a 0-arg fn → the child applies it.
    let got = run_and_get_user_program(
        "(:wat::core::fn [] -> :wat::Record (:child::Cfg 99))",
    );
    assert_is_child_cfg(got, "bare anon fn");
}

#[test]
fn env_fn_as_call_expr() {
    // env-fn evals to a :wat::Record directly (a call of the forms-defined fn).
    let got = run_and_get_user_program("(:child::make-env)");
    assert_is_child_cfg(got, "named call expr");
}
