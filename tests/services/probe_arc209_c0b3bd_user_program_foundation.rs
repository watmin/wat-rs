//! Arc 209 C0b.3b-d (foundation) — `user.program` injection: the seam.
//!
//! `invoke_user_main` (freeze.rs) is the chokepoint BOTH the root main and every process child
//! run through. Today it hardcodes the 7th field of `:wat::program::Env` to
//! `(:wat::program::EmptyEnv)` (freeze.rs:1095) and offers NO way to supply a `user.program` —
//! so wat-cli can't inject one into the root universe and process children can't either. Only
//! thread children (via the `init-fn` closure) can. This stone opens the seam: `invoke_user_main`
//! accepts an optional produced `user.program` Record; `None` keeps the `EmptyEnv` default (every
//! current path unchanged). The consumers build on this — root (`wat-cli --env fqdn/fn`) and
//! process (`ProcessOpts` env-fn name → child resolves+runs) are follow-on sub-stones.
//!
//! TWO proofs:
//! 1. `injected_user_program_flows_to_main` — inject a `:user::MyEnv` Record; `:user::main` reads
//!    `(:wat::program::Env/user.program (:wat::program::env))` and returns it; the test asserts the
//!    returned value IS the injected record (class_fqdn `user::MyEnv`), not `EmptyEnv`.
//! 2. `default_user_program_is_empty_env` — `None` → `:user::main` sees `EmptyEnv` (the current
//!    behavior is preserved by the default; the regression guard).
//!
//! RED at HEAD: `invoke_user_main` takes 2 args; the 3-arg call (with the injected `user.program`)
//! does not compile. GREEN after 3b-d: the seam accepts + installs the injected Record.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bd_user_program_foundation

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, invoke_user_main, invoke_user_main_with_program, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::Record::def :user::MyEnv [token <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::Record
  (:wat::program::Env/user.program (:wat::program::env)))
"#;

#[test]
fn injected_user_program_flows_to_main() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-d: user.program injection foundation)");
    // Build the injected user.program Record in the frozen world.
    let injected = eval_in_frozen(
        &wat::parse_one!("(:user::MyEnv 42)").expect("parse MyEnv ctor"),
        &world,
        &Environment::new(),
    )
    .map(|tv| tv.value_owned())
    .expect("MyEnv constructs");
    // Inject it through the new additive seam (invoke_user_main stays 2-arg; the injecting
    // variant is a separate fn — zero ripple to the ~30 existing callers).
    let got = invoke_user_main_with_program(&world, vec![], injected)
        .unwrap_or_else(|e| panic!("invoke_user_main_with_program raised: {e:?}"));
    match got {
        Value::wat__Record { class_fqdn, .. } => assert_eq!(
            class_fqdn.as_str(),
            "user::MyEnv",
            "expected main to read the INJECTED user.program (user::MyEnv), not the EmptyEnv default"
        ),
        other => panic!("expected main to return the injected :wat::Record user.program; got {other:?}"),
    }
}

#[test]
fn default_user_program_is_empty_env() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    // The unchanged 2-arg invoke_user_main → the EmptyEnv default (current behavior preserved).
    let got = invoke_user_main(&world, vec![])
        .unwrap_or_else(|e| panic!("invoke_user_main raised: {e:?}"));
    match got {
        Value::wat__Record { class_fqdn, .. } => assert_eq!(
            class_fqdn.as_str(),
            "wat::program::EmptyEnv",
            "expected the default user.program to be EmptyEnv when none is injected"
        ),
        other => panic!("expected main to return the EmptyEnv default user.program; got {other:?}"),
    }
}
