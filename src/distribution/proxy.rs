//! Stdio proxy threads (arc 104c) + child reaping. Split out of
//! `distribution/mod.rs` (arc 170) — part of the run/exit path, kept
//! in its own file since it's a self-contained bridging concern: real
//! OS stdio (fd 0/1/2 in the cli's process) to one end of one pipe
//! shared with the forked child process. Direct libc::read /
//! libc::write — no std::io::Stdin's reentrant Mutex involved. Same
//! discipline as fork.rs's PipeReader / PipeWriter.

use std::os::fd::{AsRawFd, OwnedFd};

/// Spawn the stdin → child pipe bridge. Reads from the cli's real
/// stdin (fd 0); writes to `child_stdin` (the child's stdin pipe
/// write end). Drops `child_stdin` on EOF, closing the pipe so
/// the child sees EOF on its read-line.
pub(super) fn spawn_stdin_proxy(child_stdin: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(libc::STDIN_FILENO, child_stdin.as_raw_fd());
        // child_stdin drops here; OwnedFd::Drop closes the fd.
    })
}

/// Spawn the child stdout → real stdout bridge.
pub(super) fn spawn_stdout_proxy(child_stdout: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(child_stdout.as_raw_fd(), libc::STDOUT_FILENO);
    })
}

/// Spawn the child stderr → real stderr bridge.
pub(super) fn spawn_stderr_proxy(child_stderr: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(child_stderr.as_raw_fd(), libc::STDERR_FILENO);
    })
}

/// Tight read/write loop. Reads up to 4096 bytes from `from_fd`,
/// writes them to `to_fd`. Exits when read returns 0 (EOF) or
/// either side errors persistently.
fn proxy_loop(from_fd: libc::c_int, to_fd: libc::c_int) {
    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::read(from_fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            // EBADF (fd closed by signal handler), EIO, etc. — exit.
            return;
        }
        if n == 0 {
            // EOF from peer.
            return;
        }
        let mut written = 0usize;
        while written < n as usize {
            let w = unsafe {
                libc::write(
                    to_fd,
                    buf.as_ptr().add(written) as *const _,
                    n as usize - written,
                )
            };
            if w < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // EPIPE (peer closed), EBADF, etc. — exit.
                return;
            }
            written += w as usize;
        }
    }
}

/// Block on waitpid for the child; extract exit code with shell
/// conventions (WEXITSTATUS or 128+WTERMSIG). Doesn't loop on
/// EINTR — signals are forwarded by arc 104d's handlers; the
/// next waitpid call here picks up where it left off.
pub(super) fn wait_child(pid: libc::pid_t) -> i32 {
    loop {
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(pid, &mut status, 0) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            // Should not happen for a child we forked. Surface as 1.
            eprintln!("wat: waitpid: {}", err);
            return 1;
        }
        if libc::WIFEXITED(status) {
            return libc::WEXITSTATUS(status);
        }
        if libc::WIFSIGNALED(status) {
            return 128 + libc::WTERMSIG(status);
        }
        // WIFSTOPPED — we don't pass WUNTRACED, so this shouldn't fire.
        return 1;
    }
}
