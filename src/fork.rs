//! `:wat::kernel::fork-with-forms` — the fork substrate (arc 012 slice 2).
//!
//! Creates three pipe pairs, calls `libc::fork(2)`, redirects the
//! child's stdio to the pipes via `dup2`, runs the caller's forms
//! through `startup_from_forms` + `invoke_user_main` in the child
//! (inside `catch_unwind`), exits with a code per the `EXIT_*`
//! convention below. The parent receives a
//! `:wat::kernel::ForkedChild` struct holding the child's pid plus
//! the three parent-side pipe ends.
//!
//! Fork-safety discipline (see DESIGN.md "Fork safety discipline"):
//! - Child uses `PipeReader` / `PipeWriter` (direct `libc::read/write`)
//!   for stdio — never `std::io::stdin/stdout/stderr` (those hold
//!   reentrant Mutexes inherited from the parent).
//! - Child calls `libc::_exit(2)` — skips parent atexit handlers,
//!   async-signal-safe, doesn't touch inherited Rust heap.
//! - Child builds a fresh `FrozenWorld` via `startup_from_forms` on
//!   the inherited `Vec<WatAST>` — parent's runtime state is visible
//!   in memory (COW) but the child never touches it.
//! - Child closes every inherited fd above 2 (best-effort
//!   `/proc/self/fd` / `/dev/fd` iteration).

use crate::ast::WatAST;
use crate::freeze::{
    invoke_user_main, startup_from_forms, validate_user_main_signature,
};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::InMemoryLoader;
use crate::runtime::{eval, Environment, RuntimeError, StructValue, SymbolTable, Value};

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Exit-code convention shared between slice 2 (this file — child
/// exits with one of these) and slice 3 (hermetic stdlib define
/// reads the code back and reconstructs a `:wat::kernel::Failure`).
/// Keep in sync with both endpoints; changes require matching slice
/// 3 updates.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_RUNTIME_ERROR: i32 = 1;
pub const EXIT_PANIC: i32 = 2;
pub const EXIT_STARTUP_ERROR: i32 = 3;
pub const EXIT_MAIN_SIGNATURE: i32 = 4;

/// The payload of a `Value::wat__kernel__ChildHandle`. Holds the
/// child's pid plus a `reaped` flag set by
/// `:wat::kernel::wait-child`. `Drop` sends `SIGKILL` and blocks on
/// `waitpid` if the caller never waited — keeps zombies out of the
/// process table.
#[derive(Debug)]
pub struct ChildHandleInner {
    pub pid: libc::pid_t,
    pub reaped: AtomicBool,
}

impl ChildHandleInner {
    pub fn new(pid: libc::pid_t) -> Self {
        Self {
            pid,
            reaped: AtomicBool::new(false),
        }
    }

    /// Mark the handle as reaped. Called by `wait-child` after a
    /// successful `waitpid`.
    pub fn mark_reaped(&self) {
        self.reaped.store(true, Ordering::SeqCst);
    }
}

impl Drop for ChildHandleInner {
    fn drop(&mut self) {
        if self.reaped.load(Ordering::SeqCst) {
            return;
        }
        // Caller never called wait-child. Kill + reap. SIGKILL is
        // unignorable; waitpid with status pointer reaps the
        // zombie.
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
            let mut status: libc::c_int = 0;
            libc::waitpid(self.pid, &mut status, 0);
        }
    }
}

/// Allocate a pipe pair; returns `(read_end, write_end)` as OwnedFds.
fn make_pipe(op: &str) -> Result<(OwnedFd, OwnedFd), RuntimeError> {
    let mut fds = [0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!("pipe(2): {}", err),
        });
    }
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((r, w))
}

/// Best-effort: close every inherited fd above 2 in the child.
/// Iterates `/proc/self/fd` (Linux) or `/dev/fd` (macOS / BSD).
///
/// The iteration itself opens a directory fd which appears in the
/// listing. Closing that fd mid-iteration aborts the walk
/// (`closedir` sees EBADF and panics under std's read_dir). The
/// fix: collect candidate fds first, let the iterator drop (closing
/// its own fd cleanly), then close the collected fds. Any that
/// were the iterator's own fd are already closed — `libc::close`
/// returns -1 with EBADF which we ignore.
fn close_inherited_fds_above_stdio() {
    let mut to_close: Vec<i32> = Vec::new();
    for candidate in ["/proc/self/fd", "/dev/fd"] {
        if let Ok(entries) = std::fs::read_dir(candidate) {
            for entry in entries.flatten() {
                if let Some(fname) = entry.file_name().to_str() {
                    if let Ok(fd) = fname.parse::<i32>() {
                        if fd > 2 {
                            to_close.push(fd);
                        }
                    }
                }
            }
            break;
        }
    }
    for fd in to_close {
        unsafe {
            libc::close(fd);
        }
    }
}

/// Write a diagnostic directly to fd 2 via `libc::write(2)`. Used
/// only inside the child branch after dup2 has redirected stderr
/// to the parent-observable pipe. Bypasses `std::io::Stderr`'s
/// Mutex.
fn write_direct_to_stderr(s: &str) {
    let bytes = s.as_bytes();
    unsafe {
        let _ = libc::write(2, bytes.as_ptr() as *const _, bytes.len());
    }
}

/// `(:wat::kernel::fork-with-forms (forms :Vec<wat::WatAST>)) ->
/// :wat::kernel::ForkedChild`.
///
/// Forks a fresh wat evaluation on top of the current runtime's
/// loaded substrate. The child runs the caller's forms as its own
/// `:user::main`-bearing program with captured stdio; the parent
/// gets the ForkedChild struct (handle + stdin writer + stdout
/// reader + stderr reader).
pub fn eval_kernel_fork_with_forms(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-with-forms";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }

    // Evaluate the forms argument — same unwrap pattern as
    // run-sandboxed-ast.
    let forms = match eval(&args[0], env, sym)? {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        return Err(RuntimeError::TypeMismatch {
                            op: OP.into(),
                            expected: "wat::WatAST",
                            got: other.type_name(),
                        });
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Vec<wat::WatAST>",
                got: other.type_name(),
            });
        }
    };

    // Three pipes for stdin/stdout/stderr.
    let (stdin_r, stdin_w) = make_pipe(OP)?;
    let (stdout_r, stdout_w) = make_pipe(OP)?;
    let (stderr_r, stderr_w) = make_pipe(OP)?;

    // Grab raw fds before fork — child uses them in dup2 after
    // dropping the OwnedFd wrappers would close them.
    let stdin_r_raw = stdin_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // SAFETY: fork is legal at this call site. The child branch
    // runs `child_branch` which restricts itself to syscalls and
    // fresh-world wat evaluation — no std::io, no parent-thread
    // lock inheritance, no atexit handlers. See DESIGN.md "Fork
    // safety discipline".
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("fork(2): {}", err),
        });
    }

    if pid == 0 {
        // ── CHILD BRANCH ─────────────────────────────────────────
        child_branch(
            forms,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
        );
    }

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds (our copies; child still has them).
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    let handle = Arc::new(ChildHandleInner::new(pid));

    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::ForkedChild".into(),
        fields: vec![
            Value::wat__kernel__ChildHandle(handle),
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
        ],
    })))
}

/// The child's post-fork pipeline. Never returns — exits via
/// `libc::_exit` with one of the `EXIT_*` codes. Takes ownership of
/// all six OwnedFds so Rust's Drop semantics close the child's
/// copies cleanly after dup2.
fn child_branch(
    forms: Vec<WatAST>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
) -> ! {
    // Drop parent-side pipe ends (close our inherited copies).
    drop(stdin_pair.1); // parent writes
    drop(stdout_pair.0); // parent reads
    drop(stderr_pair.0); // parent reads

    // Redirect stdio onto the child-side pipes.
    unsafe {
        if libc::dup2(stdin_r_raw, 0) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stdout_w_raw, 1) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stderr_w_raw, 2) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    // Drop the originals — dup2 made copies at 0/1/2.
    drop(stdin_pair.0);
    drop(stdout_pair.1);
    drop(stderr_pair.1);

    // Hygiene: close any other inherited fd above 2.
    close_inherited_fds_above_stdio();

    // Build wat-level stdio over fd 0/1/2.
    let stdin_reader: Arc<dyn WatReader> =
        Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));

    // Fresh world from the inherited AST. InMemoryLoader (no disk)
    // matches the `scope :None` behavior today's hermetic provides.
    // Scope-through-fork is deferred per DESIGN.
    let loader = Arc::new(InMemoryLoader::new());

    let world = match startup_from_forms(forms, None, loader) {
        Ok(w) => w,
        Err(e) => {
            write_direct_to_stderr(&format!("startup: {}\n", e));
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    if let Err(msg) = validate_user_main_signature(&world) {
        write_direct_to_stderr(&format!(":user::main: {}\n", msg));
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    let main_args = vec![
        Value::io__IOReader(stdin_reader),
        Value::io__IOWriter(stdout_writer),
        Value::io__IOWriter(stderr_writer),
    ];

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&world, main_args)
    }));

    match outcome {
        Ok(Ok(_)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Err(runtime_err)) => {
            write_direct_to_stderr(&format!("runtime: {:?}\n", runtime_err));
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(_panic_payload) => {
            write_direct_to_stderr("panic: sandboxed :user::main panicked\n");
            unsafe { libc::_exit(EXIT_PANIC) };
        }
    }
}
