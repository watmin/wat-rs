//! Arc 209 C0b.3b-d (foundation) — `user.program` injection: the seam.
//!
//! `invoke_user_main` (freeze.rs) is the chokepoint BOTH the root main and every process child
//! run through. Today it hardcodes the 7th field of `:wat::program::Env` to
//! `(:wat::program::EmptyEnv')` (freeze.rs:1095) and offers NO way to supply a `user.program` —
//! so wat-cli can't inject one into the root universe and process children can't either. Only
//! thread children (via the `init-fn` closure) can. This stone opens the seam: `invoke_user_main`
//! accepts an optional produced `user.program` Record; `None` keeps the `EmptyEnv` default (every
//! current path unchanged). The consumers build on this — root (`wat-cli --env fqdn/fn`) and
//! process (`ProcessOpts` env-fn name → child resolves+runs) are follow-on sub-stones.
//!
//! TWO proofs (arc-170 update: `:user::main` returns `:nil` per the wall, so — per the IPC triangle,
//! recovery §13 — it WRITES the `user.program` as EDN to stdout; the test captures + asserts it):
//! 1. `injected_user_program_flows_to_main` — inject a `:user::MyEnv` Record; `:user::main` reads
//!    `(:wat::program::Env/user.program (:wat::program::env))` and prints it; the test captures the
//!    stdout EDN and asserts it is the injected record (`#user/MyEnv {:token 42}`), not `EmptyEnv`.
//! 2. `default_user_program_is_empty_env` — `None` → `:user::main` sees `EmptyEnv` (the current
//!    behavior is preserved by the default; the regression guard).
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bd_user_program_foundation

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{
    invoke_user_main, invoke_user_main_with_program, startup_beside,
};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::runtime::apply_function;
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (
        Arc::new(PipeReader::from_owned_fd(read_fd)),
        Arc::new(PipeWriter::from_owned_fd(write_fd)),
    )
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader.read_all(wat::rust_caller_span!()).expect("read-all");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// Install a captured ambient stdio, run `main` (which prints the user.program EDN), drain stdout.
fn capture_stdout(run: impl FnOnce()) -> Vec<String> {
    let _ = take_ambient_stdio();
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    run();
    let _ = take_ambient_stdio();
    drain_lines(&stdout_capture)
}

#[test]
fn injected_user_program_flows_to_main() {
    let world = startup_beside(file!())
        .expect("startup should succeed (C0b.3b-d: user.program injection foundation)");
    // Build the injected user.program Record in the frozen world via the co-located
    // zero-arg wrapper :user::make-my-env (no inline ctor in the .rs).
    let make_my_env = world
        .symbols()
        .get(":user::make-my-env")
        .expect("no :user::make-my-env in world")
        .clone();
    let injected = apply_function(make_my_env, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("MyEnv constructs");
    // Inject it through the additive seam; main reads user.program + prints it as EDN.
    let lines = capture_stdout(|| {
        invoke_user_main_with_program(&world, vec![], injected)
            .unwrap_or_else(|e| panic!("invoke_user_main_with_program raised: {e:?}"));
    });
    assert_eq!(
        lines,
        vec!["#user/MyEnv {:token 42}".to_string()],
        "expected main to read + emit the INJECTED user.program (user::MyEnv), not EmptyEnv"
    );
}

#[test]
fn default_user_program_is_empty_env() {
    let world = startup_beside(file!()).expect("startup should succeed");
    // The unchanged 2-arg invoke_user_main → the EmptyEnv default (current behavior preserved).
    let lines = capture_stdout(|| {
        invoke_user_main(&world, vec![])
            .unwrap_or_else(|e| panic!("invoke_user_main raised: {e:?}"));
    });
    assert_eq!(
        lines,
        vec!["#wat.program/EmptyEnv {}".to_string()],
        "expected the default user.program to be EmptyEnv when none is injected"
    );
}
