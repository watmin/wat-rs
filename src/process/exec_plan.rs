//! Arc 170 step 4 — the exec handoff: everything allocated PARENT-side, so the
//! window between `clone3` and `execve` can touch nothing but raw syscalls.
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

use std::ffi::CString;

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

/// Which binary a spawned runtime should be.
///
/// Defaults to `current_exe()` — for `wat` itself, and for any embedder whose
/// binary carries wat's entry, re-exec'ing yourself is exactly right.
///
/// `WAT_RUNTIME_BIN` overrides it, and TESTS need that override: a cargo test
/// binary's `current_exe()` is the test harness, whose `main` belongs to
/// libtest and never reaches wat's entry — so a child exec'ing it would re-run
/// the test suite instead of serving. Arc 213 named this as the one piece of
/// configuration the model needs ("cli/remote: themselves; tests: the built
/// artifact"), and rejected the alternative — a pre-`main` constructor that
/// silently turns any binary into a wat server — as spooky action.
///
/// This is CONFIG (which program to become), not identity. Nothing is
/// authorized by it; the boot handshake remains the only gate.
fn runtime_binary() -> std::io::Result<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("WAT_RUNTIME_BIN").filter(|p| !p.is_empty()) {
        return Ok(std::path::PathBuf::from(p));
    }
    if entered_wat_entry() {
        // This process IS a wat runtime — re-exec'ing itself is exactly right,
        // and it is what an embedder with its own batteries needs.
        return std::env::current_exe();
    }
    // We never entered wat's entry, so we cannot serve as one. Fall back to the
    // `wat` binary cargo built beside us. This is the cargo-test case, and it
    // is the ONE piece of configuration arc 213 said the model needs.
    Ok(std::path::PathBuf::from(env!("WAT_RUNTIME_BIN_DEFAULT")))
}

/// Set once, by `distribution::run_with_args`, at the top of wat's CLI entry.
static ENTERED_WAT_ENTRY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Record that this process went through wat's CLI entry — i.e. that it can
/// serve as a spawned runtime if it is re-exec'd.
pub(crate) fn mark_wat_entry() {
    ENTERED_WAT_ENTRY.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn entered_wat_entry() -> bool {
    ENTERED_WAT_ENTRY.load(std::sync::atomic::Ordering::Relaxed)
}

/// Everything `execve` needs, owned and pre-built.
///
/// The `*_ptrs` vectors are NUL-terminated pointer arrays into the `CString`s
/// beside them; they are built here so the child never has to walk a `Vec<CString>`
/// (which would allocate). The struct owns the `CString`s so those pointers stay
/// valid for its lifetime, and the child's lifetime ends at `execve`.
pub(crate) struct ExecPlan {
    exe: CString,
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
        let exe_path = runtime_binary()?;
        let exe = CString::new(exe_path.as_os_str().as_encoded_bytes())
            .map_err(|_| std::io::Error::other("executable path contains a NUL byte"))?;

        let mut argv: Vec<CString> = vec![exe.clone()];
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

        let mut argv_ptrs: Vec<*const libc::c_char> =
            argv.iter().map(|c| c.as_ptr()).collect();
        argv_ptrs.push(std::ptr::null());
        let mut envp_ptrs: Vec<*const libc::c_char> =
            envp.iter().map(|c| c.as_ptr()).collect();
        envp_ptrs.push(std::ptr::null());

        Ok(ExecPlan { exe, _argv: argv, _envp: envp, argv_ptrs, envp_ptrs })
    }

    /// THE WINDOW. Runs in the child, after `clone3`, and never returns.
    ///
    /// ⛔ ALLOCATION-FREE. See the module doc. Every call below is a raw syscall
    /// on a value that already exists; nothing is built, grown, formatted or
    /// dropped. `execve` replaces the image on success, so no destructor here
    /// ever runs — and on failure we `_exit` immediately rather than unwind,
    /// because unwinding would run drops in a child holding inherited locks.
    ///
    /// `stdio` are the three comms fds to place on 0/1/2; `lifeline_r` is moved
    /// to [`LIFELINE_FD`], where it is both the parent-death signal and the
    /// routing witness.
    pub(crate) unsafe fn exec_in_child(&self, stdio: [i32; 3], lifeline_r: i32) -> ! {
        libc::setpgid(0, 0);

        // Place the wire on 0/1/2 and the lifeline on its known number. dup2
        // clears CLOEXEC on the NEW fd, so all four survive the exec by
        // construction — no fcntl needed.
        libc::dup2(stdio[0], 0);
        libc::dup2(stdio[1], 1);
        libc::dup2(stdio[2], 2);
        libc::dup2(lifeline_r, LIFELINE_FD);

        // Everything above the placed set goes. close_range is one syscall and
        // allocates nothing; the fallback loop is bounded and equally quiet.
        // CLOSE_RANGE_UNSHARE is deliberately NOT used — we are about to exec.
        if libc::syscall(libc::SYS_close_range, LIFELINE_FD as u32 + 1, u32::MAX, 0) < 0 {
            let mut fd = LIFELINE_FD + 1;
            while fd < 4096 {
                libc::close(fd);
                fd += 1;
            }
        }

        libc::execve(self.exe.as_ptr(), self.argv_ptrs.as_ptr(), self.envp_ptrs.as_ptr());

        // execve returns ONLY on failure. There is no channel to explain on —
        // fd 2 belongs to the parent's err pipe and writing a formatted reason
        // would allocate. The parent sees this exit code plus the boot
        // handshake never completing, which is a located failure on its side.
        libc::_exit(crate::process::EXIT_STARTUP_ERROR);
    }
}
