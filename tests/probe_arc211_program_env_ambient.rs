//! #211 RED probe — the program env as ambient, thread-local-carried context.
//!
//! The design (converged 2026-06-11): `:wat::program::Env` is the platform-stamped
//! ambient context, installed into a `PROGRAM_ENV` thread-local (mirroring
//! `AMBIENT_STDIO`) at the post-bootstrap / pre-`:user::main` seam, and read back
//! via an ambient verb. The base carries two kernel-stamped fields:
//!   - `wat.started-at`      : Instant — the CLI-boot instant, INHERITED unchanged down the spawn tree
//!   - `wat.peer-started-at`: Instant — THIS frame's start, RE-STAMPED per peer (via `assoc`)
//! User extension is the nested `user` field (later); brackets subtypes with
//! `wat.worker-id` (#196). This probe is the FLOOR.
//!
//! Run: `cargo test --release --test probe_arc211_program_env_ambient`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use wat::services::install_program_env;

fn eval_i64(decls: &str, body: &str) -> Result<i64, String> {
    let src = format!(
        "{decls}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

/// Build a `:wat::program::Env` value with known field values by evaluating the
/// constructor in the frozen world. Returns the Value.
fn build_program_env(started_at_millis: i64, peer_started_at_millis: i64) -> Value {
    // Eval `(:wat::program::Env (:wat::time::at-millis SA) (:wat::time::at-millis PSA))`.
    // We need a world with program.wat loaded (startup_from_source loads it).
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::program::Env \
           (:wat::program::Env \
             (:wat::time::at-millis {started_at_millis}) \
             (:wat::time::at-millis {peer_started_at_millis}) \
             0 0 :wat::program::PeerKind::process)) \
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup: build_program_env");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval: build_program_env")
}

#[test]
fn c02_program_env_carries_peer_started_at() {
    // program::Env gains a SECOND field `wat.peer-started-at`.
    // RED at HEAD: program::Env has only `wat.started-at` (arity 1) → the 2-arg
    // constructor is an arity error.
    let got = eval_i64(
        "",
        "(:wat::time::epoch-millis \
           (:wat::program::Env/wat.peer-started-at \
             (:wat::program::Env (:wat::time::at-millis 5000) (:wat::time::at-millis 6000) 0 0 :wat::program::PeerKind::process)))",
    );
    assert_eq!(
        got,
        Ok(6000),
        "program::Env carries wat.peer-started-at as its second field (re-stamped per frame)"
    );
}

#[test]
fn c03_installed_env_flows_to_the_verb() {
    // Value-flow test: install a :wat::program::Env with a KNOWN started-at
    // (at-millis 5000) on the test thread, then eval a :user::compute fn
    // that reads it back through (:wat::program::env) and returns the epoch-millis.
    //
    // This replaces the hollow c01 which false-greened via the
    // reserved-prefix-blanket-accept leniency (startup/check succeeds even when
    // the verb is undefined). The real proof is value-flow: the eval must return
    // the EXACT millis we installed.

    // 1. Build the env Value by evaluating the 2-arg constructor.
    let env_val = build_program_env(5000, 0);

    // 2. Install into this thread's PROGRAM_ENV slot (RAII — held for the eval).
    let _guard = install_program_env(env_val);

    // 3. Eval a fn that reads started-at through the verb.
    let got = eval_i64(
        "",
        "(:wat::time::epoch-millis \
           (:wat::program::Env/wat.started-at \
             (:wat::program::env)))",
    );
    assert_eq!(
        got,
        Ok(5000),
        "(:wat::program::env) must return the installed :wat::program::Env; \
         started-at epoch-millis must equal 5000"
    );
}

#[test]
fn c04_invoke_installs_env_for_main() {
    // Stone 259.0c — the PRODUCTION path. `invoke_user_main` must construct +
    // install a :wat::program::Env at the post-bootstrap / pre-main seam, so
    // `:user::main` can read `(:wat::program::env)`. If invoke did NOT install,
    // the verb returns "no env installed" → main errors → invoke returns Err.
    // (No explicit assert in wat: the READ itself is the test — it executes for
    // effect in the `do` and fails if no env is installed.)
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::program::Env/wat.started-at (:wat::program::env)) \
                   nil))";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let result = invoke_user_main(&world, vec![]);
    assert!(
        result.is_ok(),
        "invoke_user_main must install the program env before :user::main; \
         main's (:wat::program::env) read failed: {:?}",
        result.err()
    );
}
