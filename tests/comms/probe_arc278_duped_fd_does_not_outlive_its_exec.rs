//! Arc 278 — PROBE: a duped fd does not outlive its exec.
//!
//! `libc::dup(2)` is not CLOEXEC-atomic: the fd it returns has
//! `FD_CLOEXEC` clear and rides into any later `exec`. This probe
//! gates the property (every fd the substrate hands out has
//! `FD_CLOEXEC`) and the tree invariant (bare `libc::dup(` survives
//! only in the post-fork region, where CLOEXEC would be wrong).
//!
//! Expected RED on the unfixed tree. The red is the demonstration.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::Path;

use wat::ast::WatAST;
use wat::runtime::{Environment, SymbolTable, Value};

fn cloexec_set(fd: RawFd) -> bool {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    assert!(
        flags >= 0,
        "F_GETFD on a live fd {fd} failed: {}",
        std::io::Error::last_os_error()
    );
    flags & libc::FD_CLOEXEC != 0
}

fn make_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(
        rc,
        0,
        "pipe2(O_CLOEXEC) failed: {}",
        std::io::Error::last_os_error()
    );
    unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
}

fn from_fd_writer(fd: i64) -> Value {
    let ast = WatAST::int(fd);
    let span = ast.span().clone();
    wat::io::eval_iowriter_from_fd(&[ast], &span, &Environment::new(), &SymbolTable::new())
        .unwrap_or_else(|e| panic!("IOWriter/from-fd on fd {fd} raised: {e:?}"))
}

fn from_fd_reader(fd: i64) -> Value {
    let ast = WatAST::int(fd);
    let span = ast.span().clone();
    wat::io::eval_ioreader_from_fd(&[ast], &span, &Environment::new(), &SymbolTable::new())
        .unwrap_or_else(|e| panic!("IOReader/from-fd on fd {fd} raised: {e:?}"))
}

fn writer_fd(v: &Value) -> RawFd {
    match v {
        Value::io__IOWriter(w) => w
            .as_raw_fd_for_poll()
            .expect("from-fd writer is fd-backed"),
        other => panic!("expected IOWriter, got {other:?}"),
    }
}

fn reader_fd(v: &Value) -> RawFd {
    match v {
        Value::io__IOReader(r) => r
            .as_raw_fd_for_poll()
            .expect("from-fd reader is fd-backed"),
        other => panic!("expected IOReader, got {other:?}"),
    }
}

fn row(name: &str, fd: RawFd, failures: &mut Vec<String>) {
    let set = cloexec_set(fd);
    eprintln!("{name} fd={fd} FD_CLOEXEC={}", if set { "SET" } else { "clear" });
    if !set {
        failures.push(format!("{name} fd={fd} FD_CLOEXEC is clear"));
    }
}

fn walk_rs(dir: &Path, visit: &mut dyn FnMut(&Path, &str)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, visit);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                visit(&path, &contents);
            }
        }
    }
}

fn is_post_fork_region(path: &Path, manifest: &Path) -> bool {
    path.strip_prefix(manifest)
        .ok()
        .map(|p| p == Path::new("src/process/exec_plan.rs"))
        .unwrap_or(false)
}

#[test]
fn a_duped_fd_does_not_outlive_its_exec() {
    let mut failures: Vec<String> = Vec::new();

    // Part 1 — the property, per fd the substrate hands out.
    let (r, w) = make_pipe();

    let iowriter = from_fd_writer(w.as_raw_fd() as i64);
    row("io-writer", writer_fd(&iowriter), &mut failures);

    let ioreader = from_fd_reader(r.as_raw_fd() as i64);
    row("io-reader", reader_fd(&ioreader), &mut failures);

    let ambient = wat::process::lend_ambient();
    row(
        "ambient-stdin",
        ambient
            .stdin
            .as_raw_fd_for_poll()
            .expect("ambient stdin is fd-backed"),
        &mut failures,
    );
    row(
        "ambient-stdout",
        ambient
            .stdout
            .as_raw_fd_for_poll()
            .expect("ambient stdout is fd-backed"),
        &mut failures,
    );
    row(
        "ambient-stderr",
        ambient
            .stderr
            .as_raw_fd_for_poll()
            .expect("ambient stderr is fd-backed"),
        &mut failures,
    );

    let cloned = w
        .try_clone()
        .expect("OwnedFd::try_clone on a live pipe write-end");
    let try_clone_fd = cloned.as_raw_fd();
    let try_clone_set = cloexec_set(try_clone_fd);
    eprintln!(
        "OwnedFd::try_clone fd={try_clone_fd} FD_CLOEXEC={}",
        if try_clone_set { "SET" } else { "clear" }
    );
    if !try_clone_set {
        failures.push(format!(
            "OwnedFd::try_clone fd={try_clone_fd} FD_CLOEXEC is clear"
        ));
    }

    // Part 2 — the invariant. Bare `libc::dup(` survives only where
    // CLOEXEC would be wrong (the post-fork dup2 region in exec_plan.rs).
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest.join("src");
    let mut bare_dup: Vec<String> = Vec::new();
    walk_rs(&src_dir, &mut |path, contents| {
        if is_post_fork_region(path, manifest) {
            return;
        }
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains("libc::dup(") {
                let rel = path
                    .strip_prefix(manifest)
                    .unwrap_or(path)
                    .display()
                    .to_string();
                bare_dup.push(format!("{}:{}  {}", rel, idx + 1, line.trim()));
            }
        }
    });
    for site in &bare_dup {
        eprintln!("bare libc::dup(  {site}");
        failures.push(format!("bare libc::dup( outside post-fork: {site}"));
    }

    assert!(
        failures.is_empty(),
        "a duped fd must not outlive its exec; {} finding(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
