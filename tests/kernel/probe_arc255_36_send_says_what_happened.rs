//! Stone 255.36 — SendOutcome says what happened.
//!
//! A send after this handle was closed is `HandleClosed`. `close'` is
//! restricted to `:wat::kernel::` callers, so this test does what `close'`
//! does: `take` the peer out of its cell, then send on the same value.
//! A send to a far end that left, on both loci, is `Closed`.
//! A thread send never yields `Stopped`. `comms/thread.rs` maps every
//! crossbeam send error to `SendError::Disconnected`.

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::kernel::spawn::{PeerCell, PEER_TYPE_PATH};
use wat::load::loader::InMemoryLoader;
use wat::runtime::{apply_function, Value};

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
            assert_eq!(ev.type_path, ":wat::kernel::SendOutcome", "{ev:?}");
            ev.variant_name.clone()
        }
        other => panic!("send must return SendOutcome, got {other:?}"),
    }
}

const LIVE: &str = r#"
(:wat::core::defn :user::live-peer [] -> (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [b (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     a (:wat::spawn::Bound/address b)]
    (:wat::core::match (:wat::kernel::connect a)
      [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
      [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
      [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])))

(:wat::core::defn :user::send-one [p <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::kernel::SendOutcome
  (:wat::kernel::send p 1))
"#;

#[test]
fn a_send_after_close_is_handle_closed() {
    let world = startup_from_source(LIVE, None, Arc::new(InMemoryLoader::new()))
        .expect("world");
    let peer_fn = world
        .symbols
        .get(":user::live-peer")
        .expect("live-peer")
        .clone();
    let send_fn = world
        .symbols
        .get(":user::send-one")
        .expect("send-one")
        .clone();
    let span = wat::rust_caller_span!();
    let peer = apply_function(peer_fn, vec![], world.symbols(), span.clone())
        .expect("mint a peer");
    {
        let Value::RustOpaque(inner) = &peer else {
            panic!("peer opaque, got {peer:?}");
        };
        assert_eq!(inner.type_path, PEER_TYPE_PATH);
        let cell = inner
            .payload
            .downcast_ref::<PeerCell>()
            .expect("PeerCell");
        cell.with_mut(":user::probe", span.clone(), |opt| {
            opt.take();
        })
        .expect("take the peer out, which is what close' does");
    }
    let outcome = apply_function(send_fn, vec![peer], world.symbols(), span)
        .expect("send on a closed handle must not raise");
    assert_eq!(variant_name(&outcome), "HandleClosed");
}

#[test]
fn a_send_to_a_far_end_that_left_is_closed_on_both_loci() {
    let (rc, stdout, stderr) =
        run_wat("tests/kernel/probe_arc255_36_send_says_what_happened.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines,
        [
            "\"thread-far Closed the far end is gone\"",
            "\"process-far Closed the far end is gone\"",
        ],
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
