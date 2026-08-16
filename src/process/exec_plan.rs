//! Arc 170 step 4 — the exec handoff: everything allocated PARENT-side, so the
//! window between `clone3` and `execveat` can touch nothing but raw syscalls.
//!
//! # Why this file is separate, and why it is paranoid
//!
//! `clone3` without an `execve` leaves the child sharing the parent's address
//! image — including glibc's malloc arena locks, frozen in whatever state a
//! sibling thread held them at the clone instant. A child that then `malloc`s
//! can block forever on a lock whose owner does not exist in the child, and the
//! parent's `waitid` hangs behind it. That is the deadlock arc 213 was opened
//! for, it only reproduces under parallelism, and it is why POSIX says a child
//! between fork and exec may call only async-signal-safe functions.
//!
//! So the rule for [`ExecPlan::exec_in_child`] is absolute and mechanical:
//! **no allocation, no free, no lock, no Rust machinery.** No `Vec`, `String`,
//! `format!`, `CString::new`, `println!`, `eprintln!`, no `?`, no trait object
//! call that might allocate, no destructor. Only `libc`. Every byte the child
//! needs is built here, in the parent, before the clone.
//!
//! If you touch that function, read it back line by line against this list. A
//! single stray allocation reintroduces an intermittent deadlock that green
//! tests will not catch.

use std::ffi::{CString, OsStr};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// The child's lifeline read-end, placed at a KNOWN fd number so it survives
/// `execve` and can be found by a process that has no memory of being forked.
///
/// It doubles as the ROUTING SIGNAL: only a wat parent hands a child this fd, so
/// its mere existence answers "was I spawned as a forms-server?" without a CLI
/// flag. That is deliberate — a flag would be public user surface for an
/// internal mechanism, typeable at a shell and visible in `ps`, and it would be
/// a CLAIM where this is a WITNESS. Reusing the lifeline rather than minting a
/// marker fd keeps it one object: the thing that routes you is the thing that
/// proves a parent is holding the other end.
pub(crate) const LIFELINE_FD: i32 = 3;

/// Temporary slot for the runtime image fd in the child, above the placed
/// set (0/1/2 + lifeline 3). `dup2` here, `close_range` from 5, then
/// `execveat` this fd with `AT_EMPTY_PATH`. CLOEXEC so a successful exec
/// does not leak it into the new image.
const EXEC_IMAGE_FD: i32 = LIFELINE_FD + 1;

/// Which binary a spawned runtime should be.
///
/// The child does **not** `execve` a path. It `execveat`s a file descriptor
/// the parent opened (or dups from the fd captured at CLI entry). Arc 213
/// rejected `/proc/self/exe` as the exec path (a test binary becoming a
/// server via a magic path is spooky action; `/proc` as oracle has already
/// been purged once). `current_exe()` is `readlink("/proc/self/exe")` and
/// becomes `"…/wat (deleted)"` after unlink — that string cannot be opened.
///
/// `WAT_RUNTIME_BIN` overrides the image, and TESTS need that override: a
/// cargo test binary's image is the test harness. Arc 213 named this as the
/// one piece of configuration the model needs ("cli/remote: themselves;
/// tests: the built artifact").
///
/// This is CONFIG (which program to become), not identity. Nothing is
/// authorized by it; the boot handshake remains the only gate.
fn open_runtime_image() -> std::io::Result<(OwnedFd, CString)> {
    match image_source(
        std::env::var_os("WAT_RUNTIME_BIN"),
        entered_wat_entry(),
        self_image_fd(),
    )? {
        ImageSource::Override(path) => open_named(path.as_os_str()),
        ImageSource::HeldSelf(raw) => dup_named(raw, display_argv0()),
        ImageSource::BuiltArtifact => open_named(OsStr::new(env!("WAT_RUNTIME_BIN_DEFAULT"))),
    }
}

/// Where the image fd comes from. Pure so the deleted-exe case is a unit,
/// not a live MCP, and so `/proc/self/exe` cannot sneak back in as a path.
#[derive(Debug, PartialEq, Eq)]
enum ImageSource {
    Override(std::path::PathBuf),
    HeldSelf(i32),
    BuiltArtifact,
}

fn image_source(
    override_bin: Option<std::ffi::OsString>,
    entered_wat: bool,
    held_fd: Option<i32>,
) -> std::io::Result<ImageSource> {
    if let Some(p) = override_bin.filter(|p| !p.is_empty()) {
        return Ok(ImageSource::Override(std::path::PathBuf::from(p)));
    }
    if entered_wat {
        return match held_fd {
            Some(fd) => Ok(ImageSource::HeldSelf(fd)),
            None => Err(std::io::Error::other(
                "this process's image was not captured at entry; refusing to \
                 execve a /proc path or a deleted current_exe() readlink",
            )),
        };
    }
    Ok(ImageSource::BuiltArtifact)
}

fn open_named(path: &OsStr) -> std::io::Result<(OwnedFd, CString)> {
    let c = CString::new(path.as_bytes())
        .map_err(|_| std::io::Error::other("executable path contains a NUL byte"))?;
    // O_PATH: a handle to the inode for later execveat, not a read. Survives
    // unlink of the directory entry. O_CLOEXEC: the parent copy must not leak
    // across an unrelated exec.
    let raw = unsafe { libc::open(c.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    if raw < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok((unsafe { OwnedFd::from_raw_fd(raw) }, c))
}

fn dup_named(raw: i32, argv0: CString) -> std::io::Result<(OwnedFd, CString)> {
    let duped = unsafe { libc::fcntl(raw, libc::F_DUPFD_CLOEXEC, 0) };
    if duped < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok((unsafe { OwnedFd::from_raw_fd(duped) }, argv0))
}

fn display_argv0() -> CString {
    std::env::current_exe()
        .ok()
        .and_then(|p| CString::new(p.as_os_str().as_encoded_bytes()).ok())
        .unwrap_or_else(|| CString::new("wat").expect("'wat' is a valid CString"))
}

/// Set once, by `distribution::run_with_args`, at the top of wat's CLI entry.
static ENTERED_WAT_ENTRY: AtomicBool = AtomicBool::new(false);

/// Process-lifetime `O_PATH` fd to the image we were exec'd from. Captured
/// at entry while the directory entry still exists; later unlinks (a cargo
/// rebuild against a live `wat --mcp`) do not revoke it. `-1` means none.
static SELF_IMAGE_FD: AtomicI32 = AtomicI32::new(-1);

/// Record that this process went through wat's CLI entry — i.e. that it can
/// serve as a spawned runtime if it is re-exec'd. Opens the running image
/// *now*, by its real path, and holds the fd. That is what makes a later
/// unlink survivable without walking `/proc`.
pub(crate) fn mark_wat_entry() {
    ENTERED_WAT_ENTRY.store(true, Ordering::Relaxed);
    if SELF_IMAGE_FD.load(Ordering::Relaxed) >= 0 {
        return;
    }
    let Ok(path) = std::env::current_exe() else {
        return;
    };
    let Ok((fd, _)) = open_named(path.as_os_str()) else {
        return;
    };
    let raw = fd.into_raw_fd();
    if SELF_IMAGE_FD
        .compare_exchange(-1, raw, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        unsafe { libc::close(raw) };
    }
}

fn entered_wat_entry() -> bool {
    ENTERED_WAT_ENTRY.load(Ordering::Relaxed)
}

fn self_image_fd() -> Option<i32> {
    let fd = SELF_IMAGE_FD.load(Ordering::Relaxed);
    (fd >= 0).then_some(fd)
}

/// Everything `execveat` needs, owned and pre-built.
///
/// The `*_ptrs` vectors are NUL-terminated pointer arrays into the `CString`s
/// beside them; they are built here so the child never has to walk a `Vec<CString>`
/// (which would allocate). The struct owns the `CString`s so those pointers stay
/// valid for its lifetime, and the child's lifetime ends at `execveat`.
pub(crate) struct ExecPlan {
    exe_fd: OwnedFd,
    empty_path: CString,
    _argv: Vec<CString>,
    _envp: Vec<CString>,
    argv_ptrs: Vec<*const libc::c_char>,
    envp_ptrs: Vec<*const libc::c_char>,
}

// SAFETY: the raw pointers point into `_argv`/`_envp`, owned by this same
// struct, so they travel with their referents. The plan is built in the parent
// and moved into the child closure before `clone3`; nothing mutates it after.
unsafe impl Send for ExecPlan {}

impl ExecPlan {
    /// Build the plan — ALL allocation happens here, in the parent.
    ///
    /// `label` is arc 170 closure #6's `ps`-visible identity: the `edn::write`
    /// rendering of the spawner's `user-data` record (e.g. `#my.app/CounterSvc
    /// {}`, `#wat.brackets/Worker {:id 3}`), or `None` when the caller declared no
    /// identity. When present it becomes argv\[1\]; `None` leaves argv exactly
    /// `[exe]`, unchanged from before this field existed.
    ///
    /// ⛔ THE WALL — the label DESCRIBES, it must never ROUTE.
    /// `distribution::mod::run_with_args`'s rejection of a `--forms-server`
    /// flag (arc 170 step 4; see the comment there — *"a flag would be public
    /// user surface for an internal mechanism: typeable at a shell and visible
    /// in `ps`, and it would be a CLAIM where this is a WITNESS"*) is the same
    /// reasoning applied to a flag; a label is the identical trap in a
    /// friendlier costume. fd 3 ([`LIFELINE_FD`]) is where a parent writes
    /// `#wat.boot/Here` — `spawned_runtime::was_spawned()` requires that
    /// frame, not mere openness, and `run_with_args` checks it BEFORE argv
    /// is ever parsed for a Mode.
    /// Nothing may ever parse argv\[1\] to decide behavior: a shell invocation
    /// with a forged `#ns/Thing {}` argument and no fd 3 must be
    /// indistinguishable from any other bogus argv\[1\] (it falls through to
    /// the ordinary CLI parser, is read as an entry-file path exactly like any
    /// other string, and fails to load — see
    /// `tests/process/wat_arc170_closure6_label_wall.rs`). Never add a branch
    /// that inspects this string.
    ///
    /// A spawned runtime still takes no COMMAND LINE in the sense that matters:
    /// it is told what to do over the wire, and `(:wat::runtime::argv)` is
    /// empty in a child by contract (`spawned_runtime::serve` calls
    /// `set_argv(Vec::new())` unconditionally, never reading OS argv — the
    /// label and the ambient argv are disjoint by code path, gated below). The
    /// environment is inherited verbatim so config-by-env keeps working.
    pub(crate) fn build(label: Option<&str>) -> std::io::Result<Self> {
        let (exe_fd, argv0) = open_runtime_image()?;

        let mut argv: Vec<CString> = vec![argv0];
        if let Some(l) = label {
            argv.push(
                CString::new(l.as_bytes())
                    .map_err(|_| std::io::Error::other("identity label contains a NUL byte"))?,
            );
        }
        let envp: Vec<CString> = std::env::vars_os()
            .filter_map(|(k, v)| {
                let mut kv = k.into_encoded_bytes();
                kv.push(b'=');
                kv.extend_from_slice(&v.into_encoded_bytes());
                CString::new(kv).ok()
            })
            .collect();

        let mut argv_ptrs: Vec<*const libc::c_char> = argv.iter().map(|c| c.as_ptr()).collect();
        argv_ptrs.push(std::ptr::null());
        let mut envp_ptrs: Vec<*const libc::c_char> = envp.iter().map(|c| c.as_ptr()).collect();
        envp_ptrs.push(std::ptr::null());

        Ok(ExecPlan {
            exe_fd,
            empty_path: CString::new("").expect("empty pathname is a valid CString"),
            _argv: argv,
            _envp: envp,
            argv_ptrs,
            envp_ptrs,
        })
    }

    /// THE WINDOW. Runs in the child, after `clone3`, and never returns.
    ///
    /// ⛔ ALLOCATION-FREE. See the module doc. Every call below is a raw syscall
    /// on a value that already exists; nothing is built, grown, formatted or
    /// dropped. `execveat` replaces the image on success, so no destructor here
    /// ever runs — and on failure we `_exit` immediately rather than unwind,
    /// because unwinding would run drops in a child holding inherited locks.
    ///
    /// `stdio` are the three comms fds to place on 0/1/2; `lifeline_r` is moved
    /// to [`LIFELINE_FD`], where it is both the parent-death signal and the
    /// routing witness. The image fd is parked at [`EXEC_IMAGE_FD`] and
    /// consumed by `execveat(..., AT_EMPTY_PATH)` — no path lookup, no `/proc`.
    pub(crate) unsafe fn exec_in_child(&self, stdio: [i32; 3], lifeline_r: i32) -> ! {
        libc::setpgid(0, 0);

        // Park the image on a free fd ≥ 5 so placing 0/1/2/3 (or later
        // parking it at 4) cannot overwrite a still-needed comms end.
        // F_DUPFD_CLOEXEC is async-signal-safe; it never picks an open fd.
        let parked = libc::fcntl(
            self.exe_fd.as_raw_fd(),
            libc::F_DUPFD_CLOEXEC,
            EXEC_IMAGE_FD + 1,
        );
        if parked < 0 {
            libc::_exit(crate::process::EXIT_STARTUP_ERROR);
        }

        // Place the wire on 0/1/2 and the lifeline on its known number. dup2
        // clears CLOEXEC on the NEW fd, so those four survive the exec by
        // construction — no further fcntl needed.
        libc::dup2(stdio[0], 0);
        libc::dup2(stdio[1], 1);
        libc::dup2(stdio[2], 2);
        libc::dup2(lifeline_r, LIFELINE_FD);

        // Now the comms ends live on 0/1/2/3. Drop the image onto 4 and
        // restore CLOEXEC (dup2 clears it) so a successful exec does not leak it.
        libc::dup2(parked, EXEC_IMAGE_FD);
        libc::fcntl(EXEC_IMAGE_FD, libc::F_SETFD, libc::FD_CLOEXEC);

        // Everything above the placed set + the parked image goes.
        // CLOSE_RANGE_UNSHARE is deliberately NOT used — we are about to exec.
        if libc::syscall(libc::SYS_close_range, EXEC_IMAGE_FD as u32 + 1, u32::MAX, 0) < 0 {
            let mut fd = EXEC_IMAGE_FD + 1;
            while fd < 4096 {
                libc::close(fd);
                fd += 1;
            }
        }

        // The syscall, not glibc `fexecve`: glibc's fallback is
        // execve("/proc/self/fd/N"), the path-oracle we are not going back to.
        libc::execveat(
            EXEC_IMAGE_FD,
            self.empty_path.as_ptr(),
            self.argv_ptrs.as_ptr() as *const *mut libc::c_char,
            self.envp_ptrs.as_ptr() as *const *mut libc::c_char,
            libc::AT_EMPTY_PATH,
        );

        // execveat returns ONLY on failure. There is no channel to explain on —
        // fd 2 belongs to the parent's err pipe and writing a formatted reason
        // would allocate. The parent sees this exit code plus the boot
        // handshake never completing, which is a located failure on its side.
        libc::_exit(crate::process::EXIT_STARTUP_ERROR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wat_entry_uses_the_held_fd_not_a_proc_path() {
        let src = image_source(None, true, Some(7)).expect("resolve");
        assert_eq!(src, ImageSource::HeldSelf(7));
    }

    #[test]
    fn a_wat_entry_without_a_held_fd_refuses_proc() {
        let err = image_source(None, true, None).expect_err("must not invent a /proc path");
        assert_eq!(
            err.to_string(),
            "this process's image was not captured at entry; refusing to \
             execve a /proc path or a deleted current_exe() readlink"
        );
    }

    #[test]
    fn an_override_still_wins() {
        let src = image_source(Some("/opt/wat".into()), true, Some(7)).expect("resolve");
        assert_eq!(
            src,
            ImageSource::Override(std::path::PathBuf::from("/opt/wat"))
        );
    }

    #[test]
    fn a_test_harness_falls_back_to_the_built_artifact() {
        let src = image_source(None, false, None).expect("resolve");
        assert_eq!(src, ImageSource::BuiltArtifact);
    }
}
