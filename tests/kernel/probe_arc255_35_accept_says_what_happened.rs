//! Stone 255.35 — AcceptOutcome says what happened.
//!
//! A dropped thread listener is `Closed`. A stop, on either locus, is
//! `Stopped`. Wat has no verb that fires the substrate cascade, so the
//! stop rows are driven here the way
//! `probe_arc278_shutdown_priority_is_the_ruling` fires it: write the
//! wake pipe and wait until the broadcast fd is readable. The thread
//! row also drops `SHUTDOWN_TX` via `trigger_shutdown`, which is the
//! sever `thread::Receiver::recv` actually selects on. The listener's
//! sender is held, so the accept is not a drop.
//!
//! Shutdown infra is process-global and one-way. nextest forks each
//! test into its own process.

use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::kernel::listener::Listener;
use wat::load::loader::InMemoryLoader;
use wat::runtime::{trigger_shutdown, Value};

fn run_wat(rel: &str) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel}: {e}"));
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn variant_name(v: &Value) -> String {
    match v {
        Value::Enum(ev) => {
            assert_eq!(ev.type_path, ":wat::kernel::AcceptOutcome", "{ev:?}");
            ev.variant_name.clone()
        }
        other => panic!("accept must return AcceptOutcome, got {other:?}"),
    }
}

fn world() -> wat::freeze::FrozenWorld {
    startup_from_source("", None, Arc::new(InMemoryLoader::new())).expect("empty world")
}

/// Write the wake pipe and return once the broadcast fd is readable.
/// Same wire wait as the shutdown-priority gate. No sleep.
fn fire_cascade_until_broadcast_is_ready() {
    wat::runtime::init_shutdown_signal();
    let broadcast_fd = wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(Ordering::SeqCst);
    let wake_fd = wat::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
    assert!(broadcast_fd >= 0, "broadcast fd absent ({broadcast_fd})");
    assert!(wake_fd >= 0, "wake fd absent ({wake_fd})");
    let byte = b"!";
    let n = unsafe { libc::write(wake_fd, byte.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "wake write");
    let mut pfd = libc::pollfd {
        fd: broadcast_fd,
        events: libc::POLLIN | libc::POLLHUP,
        revents: 0,
    };
    loop {
        let r = unsafe { libc::poll(&mut pfd, 1, 5_000) };
        if r > 0 {
            break;
        }
        assert!(r != 0, "the shutdown worker never wrote the broadcast");
        let err = std::io::Error::last_os_error();
        assert!(
            err.raw_os_error() == Some(libc::EINTR),
            "poll on the broadcast fd failed: {err}"
        );
    }
}

#[test]
fn a_dropped_thread_listener_is_closed() {
    let (rc, stdout, stderr) = run_wat("tests/kernel/probe_arc255_35_accept_says_what_happened.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines, ["\"thread-dropped Closed\""], "stdout:\n{stdout}\nstderr:\n{stderr}");
}

#[test]
fn a_stop_is_stopped_on_both_loci() {
    wat::runtime::init_shutdown_signal();
    let w = world();
    let span = wat::rust_caller_span!();

    let (tx, rx) = wat::comms::thread::pair::<Value>();
    let thread_listener = Listener::from_crossbeam(rx);
    trigger_shutdown();
    let thread_got = thread_listener
        .accept_as_value(&w.symbols, &span)
        .unwrap_or_else(|e| panic!("thread accept on a stop must not raise: {e:?}"));
    let thread_name = variant_name(&thread_got);
    drop(tx);

    fire_cascade_until_broadcast_is_ready();
    let name = format!(
        "wat.255.35.{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let sa = SocketAddr::from_abstract_name(name.as_bytes()).expect("abstract name");
    let bound = UnixListener::bind_addr(&sa).expect("bind");
    let process_listener = Listener::from_socket(
        bound,
        wat::edn::render::DEFAULT_MAX_FRAME_BYTES,
    );
    let process_got = process_listener
        .accept_as_value(&w.symbols, &span)
        .unwrap_or_else(|e| panic!("process accept on a stop must not raise: {e:?}"));
    let process_name = variant_name(&process_got);

    assert_eq!(thread_name, "Stopped", "thread stop");
    assert_eq!(process_name, "Stopped", "process stop");
}
