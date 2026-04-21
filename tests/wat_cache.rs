//! End-to-end integration test for `:wat::std::program::Cache`.
//!
//! A `:user::main` with stdio handles: sets up a Console for
//! diagnostics, sets up a Cache driver, does a put/get round-trip,
//! prints "hit" or "miss" to stdout, exits.
//!
//! The T1/T2/T3 stderr checkpoints served as the probe that found
//! the original thread-ownership bug (LocalCache created on main
//! thread, passed to driver, tripped the thread-id guard). They
//! stay in place as regression sentinels — a future hang would
//! halt at the last visible checkpoint.

use std::io::Write;
use std::process::{Command, Stdio};

fn write_temp(contents: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "wat-cache-{}-{}.wat",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let mut f = std::fs::File::create(&path).expect("create temp");
    f.write_all(contents.as_bytes()).expect("write");
    path
}

const CACHE_PROGRAM: &str = r#"
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  ;; Outer scope holds only the driver-handles. The inner scope binds
  ;; everything that keeps Console/Cache drivers alive (senders);
  ;; when the inner scope exits, those handles drop, the drivers see
  ;; the disconnect, and the outer `join`s flush-and-exit cleanly
  ;; before wat returns. Without this layering, wat can exit
  ;; while the Console driver still has queued stdout writes pending.
  (:wat::core::let*
    (((con-state :(wat::kernel::HandlePool<rust::crossbeam_channel::Sender<(i64,String)>>,wat::kernel::ProgramHandle<()>))
      (:wat::std::program::Console stdout stderr 2))
     ((con-drv :wat::kernel::ProgramHandle<()>)
      (:wat::core::second con-state))
     ((state :(wat::kernel::HandlePool<rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>>,wat::kernel::ProgramHandle<()>))
      (:wat::std::program::Cache 16 1))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::core::second state))

     ;; Inner work scope — owns the senders. When this let* returns,
     ;; all senders drop, and con-drv / driver see their disconnects.
     ((_ :())
      (:wat::core::let*
        (((con-pool :wat::kernel::HandlePool<rust::crossbeam_channel::Sender<(i64,String)>>)
          (:wat::core::first con-state))
         ((diag :rust::crossbeam_channel::Sender<(i64,String)>)
          (:wat::kernel::HandlePool::pop con-pool))
         ((spare :rust::crossbeam_channel::Sender<(i64,String)>)
          (:wat::kernel::HandlePool::pop con-pool))
         ((_ :()) (:wat::kernel::HandlePool::finish con-pool))

         ((pool :wat::kernel::HandlePool<rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>>)
          (:wat::core::first state))
         ((req-tx :rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>)
          (:wat::kernel::HandlePool::pop pool))
         ((_ :()) (:wat::kernel::HandlePool::finish pool))
         ((reply-pair :(rust::crossbeam_channel::Sender<Option<i64>>,rust::crossbeam_channel::Receiver<Option<i64>>))
          (:wat::kernel::make-bounded-queue :Option<i64> 1))
         ((reply-tx :rust::crossbeam_channel::Sender<Option<i64>>)
          (:wat::core::first reply-pair))
         ((reply-rx :rust::crossbeam_channel::Receiver<Option<i64>>)
          (:wat::core::second reply-pair))

         ((_ :()) (:wat::std::program::Console/err diag "T1: about-to-put\n"))
         ((_ :()) (:wat::std::program::Cache/put req-tx reply-tx reply-rx "answer" 42))
         ((_ :()) (:wat::std::program::Console/err diag "T2: put-acked\n"))
         ((got :Option<i64>) (:wat::std::program::Cache/get req-tx reply-tx reply-rx "answer"))
         ((_ :()) (:wat::std::program::Console/err diag "T3: get-returned\n")))
        (:wat::core::match got -> :()
          ((Some v) (:wat::std::program::Console/out diag "hit\n"))
          (:None    (:wat::std::program::Console/out diag "miss\n")))))

     ;; Inner scope ended — all senders dropped. Drain both drivers.
     ((_ :()) (:wat::kernel::join driver))
     ((_ :()) (:wat::kernel::join con-drv)))
    ()))
"#;

#[test]
fn cache_put_then_get_round_trip() {
    let path = write_temp(CACHE_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    // 5-second timeout guard. A regression to the thread-ownership
    // bug would re-introduce the hang; we'd rather fail loudly than
    // let CI block indefinitely.
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() > std::time::Duration::from_secs(5) {
                    let _ = child.kill();
                    panic!(
                        "cache round-trip timed out — regression to the thread-ownership bug?"
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => panic!("try_wait: {}", e),
        }
    }
    let out = child.wait_with_output().expect("wait_with_output");
    let _ = std::fs::remove_file(&path);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "wat exited non-zero.\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("hit\n"),
        "expected 'hit' on stdout (successful put→get round-trip).\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
    assert!(stderr.contains("T1: about-to-put"), "missing T1: {}", stderr);
    assert!(stderr.contains("T2: put-acked"), "missing T2: {}", stderr);
    assert!(stderr.contains("T3: get-returned"), "missing T3: {}", stderr);
}
