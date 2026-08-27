//! #211 RED probe — the program env as ambient, thread-local-carried context.
//!
//! The design (converged 2026-06-11): `:wat::program::Env` is the platform-stamped
//! ambient context, installed into a `PROGRAM_ENV` thread-local (mirroring
//! `AMBIENT_STDIO`) at the post-bootstrap / pre-`:user::main` seam, and read back
//! via an ambient verb. The base carries two kernel-stamped fields:
//!   - `started-at`      : Instant — the CLI-boot instant, INHERITED unchanged down the spawn tree
//!   - `peer-started-at`: Instant — THIS frame's start, RE-STAMPED per peer (via `assoc`)
//!
//! User extension is the nested `user` field (later); brackets subtypes with
//! `wat.worker-id` (#196). This probe is the FLOOR.
//!
//! Wat source lives in the co-located sibling fixture `probe_arc211_program_env_ambient.wat`,
//! slurped via `startup_beside(file!())`. Named probe functions replace the inline
//! `eval_i64` / `build_program_env` helpers: `:probe::c02-compute`, `:probe::c03-compute`,
//! `:probe::build-env`, and `:user::main` (for c04).
//!
//! Run: `cargo test --release --test probe_arc211_program_env_ambient`

use wat::freeze::{invoke_user_main, startup_beside, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use wat::services::install_program_env;

fn call_i64(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Result<i64, StartupError> {
    let func = world.symbols().get(fn_name).ok_or_else(|| {
        StartupError::Runtime(Box::new(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::UnboundSymbol(fn_name.to_string()),
        )))
    })?;
    match apply_function(func.clone(), Vec::new(), world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))?
    {
        Value::i64(n) => Ok(n),
        other => Err(StartupError::Runtime(Box::new(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.to_string(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )))),
    }
}

fn call_value(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Value {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("{fn_name} must be defined in world"));
    apply_function(func.clone(), Vec::new(), world.symbols(), wat::rust_caller_span!())
        .expect("apply_function ok")
}

#[test]
fn c02_program_env_carries_peer_started_at() {
    // program::Env gains a SECOND field `peer-started-at`.
    // RED at HEAD: program::Env has only `started-at` (arity 1) → the 2-arg
    // constructor is an arity error.
    let world = startup_beside(file!()).expect("startup");
    match call_i64(&world, ":probe::c02-compute") {
        Ok(n) => assert_eq!(
            n, 6000,
            "program::Env carries peer-started-at as its second field (re-stamped per frame)"
        ),
        Err(e) => panic!(
            "program::Env carries peer-started-at as its second field (re-stamped per frame); \
             call failed: {e:?}"
        ),
    }
}

#[test]
fn c03_installed_env_flows_to_the_verb() {
    // Value-flow test: install a :wat::program::Env with a KNOWN started-at
    // (at-millis 5000) on the test thread, then call :probe::c03-compute which
    // reads it back through (:wat::program::env) and returns the epoch-millis.
    //
    // This replaces the hollow c01 which false-greened via the
    // reserved-prefix-blanket-accept leniency (startup/check succeeds even when
    // the verb is undefined). The real proof is value-flow: the eval must return
    // the EXACT millis we installed.

    // 1. Build the env Value by calling :probe::build-env (constructs a ProgramEnv
    //    with started-at=5000, peer-started-at=0 via the fixture).
    let world = startup_beside(file!()).expect("startup");
    let env_val = call_value(&world, ":probe::build-env");

    // 2. Install into this thread's PROGRAM_ENV slot (RAII — held for the eval).
    let _guard = install_program_env(env_val);

    // 3. Call :probe::c03-compute which reads started-at through the verb.
    match call_i64(&world, ":probe::c03-compute") {
        Ok(n) => assert_eq!(
            n, 5000,
            "(:wat::program::env) must return the installed :wat::program::Env; \
             started-at epoch-millis must equal 5000"
        ),
        Err(e) => panic!(
            "(:wat::program::env) must return the installed :wat::program::Env; \
             started-at epoch-millis must equal 5000; call failed: {e:?}"
        ),
    }
}

#[test]
fn c04_invoke_installs_env_for_main() {
    // Stone 259.0c — the PRODUCTION path. `invoke_user_main` must construct +
    // install a :wat::program::Env at the post-bootstrap / pre-main seam, so
    // `:user::main` can read `(:wat::program::env)`. If invoke did NOT install,
    // the verb returns "no env installed" → main errors → invoke returns Err.
    // (No explicit assert in wat: the READ itself is the test — it executes for
    // effect in the `do` and fails if no env is installed.)
    //
    // :user::main in the co-located fixture reads (:wat::program::env) for effect.
    let world = startup_beside(file!()).expect("startup");
    let result = invoke_user_main(&world, vec![]);
    assert!(
        result.is_ok(),
        "invoke_user_main must install the program env before :user::main; \
         main's (:wat::program::env) read failed: {:?}",
        result.err()
    );
}
