//! Arc 170 step 4 — the entry a freshly `execve`'d wat runtime takes.
//!
//! This is the "blank server" half of the program-over-the-wire model (arc 213):
//! a process that boots knowing NOTHING, is told its substrate and its program
//! over a pipe, and then runs it. Under COW the child could have read the
//! parent's inherited copies; after the exec there is nothing to read, which is
//! the point — the wire is the only path, so it is the tested path.

use std::process::ExitCode;

use crate::process::exec_plan::LIFELINE_FD;

/// Was this process spawned by a wat parent?
///
/// Not "is fd 3 open" — a harness control pipe is also open. The parent
/// writes [`crate::process::boot::BootFrame::Here`] onto the lifeline
/// before clone. We ask: is that frame waiting? No flag, no env, nothing
/// in `ps`. `wat < file` is still not a child (stdin is the wrong wire).
pub(crate) fn was_spawned() -> bool {
    // SAFETY: F_GETFD only interrogates the descriptor table.
    if unsafe { libc::fcntl(LIFELINE_FD, libc::F_GETFD) } < 0 {
        return false;
    }
    crate::process::boot::lifeline_has_here(LIFELINE_FD)
}

/// Boot as a spawned runtime: take the substrate and the program off fd 0, then
/// serve.
pub(crate) fn serve() -> ExitCode {
    // A spawned runtime has NO command line. It is told what to do over the
    // wire; anything it needs from its parent arrives on the program-env. This
    // is the builder's ruling made literal — and it works here precisely
    // BECAUSE the process is fresh: `ARGV` is a set-once `OnceLock`, so under
    // COW this same call was a silent no-op against the inherited value. The
    // exec is what makes the declaration true rather than ignored.
    crate::runtime::set_argv(Vec::new());

    // What `child_post_fork_init` used to do in the COW child, done here — and
    // now it can be done SAFELY, because this is a real process past its exec
    // rather than a half-formed image between clone3 and execve where a malloc
    // can deadlock. The lifeline sits at its known fd; registering it is what
    // makes parent-death reach the shutdown path.
    // Rust's default panic output must never reach fd 2 in a spawned runtime:
    // fd 2 is the parent's err CHANNEL, and `emit_structured_exit` is the sole
    // author of what crosses it. Installed FIRST — before anything that can
    // panic — and it deliberately replaces the CLI hook `run_with_args` set on
    // the way in, because this process is a child now, not a terminal session.
    crate::process::child::install_silent_panic_hook();

    crate::runtime::init_shutdown_signal_with_inputs(&[LIFELINE_FD]);
    crate::process::child::install_substrate_signal_handlers();

    let (substrate, program) = match crate::process::boot::receive_in_child(0, 1) {
        Ok(pair) => pair,
        Err(e) => crate::process::boot::report_boot_failure_and_exit(&format!(
            "spawned runtime could not read its boot sections: {e:?}"
        )),
    };
    let (config, env_fn) = match crate::process::boot::wire_to_substrate(&substrate) {
        Ok(pair) => pair,
        Err(e) => crate::process::boot::report_boot_failure_and_exit(&format!(
            "spawned runtime could not decode its substrate: {e:?}"
        )),
    };
    let forms = match crate::process::boot::wire_to_forms(&program) {
        Ok(forms) => forms,
        Err(e) => crate::process::boot::report_boot_failure_and_exit(&format!(
            "spawned runtime could not decode its program: {e:?}"
        )),
    };

    // Never returns — `run_forms_as_server_child` ends in `_exit`.
    crate::process::run_forms_as_server_child(forms, config, env_fn);
}
