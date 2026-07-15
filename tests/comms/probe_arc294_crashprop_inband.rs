//! Arc 294 crash-prop (in-band, thread tier) — a service that crashes mid-request
//! surfaces its REAL crash reason to a connect'd client, not "channel disconnected".
//!
//! The fix reuses the reply channel (`resp_tx`) already on the service side: `accept'`
//! registers a clone; `spawn_thread_peer`'s death path sends a reserved crash-sentinel
//! frame in-band on it before the owner crash_tx; `recv'`/`select'` recognize it.
//!
//! Run:
//!   cargo test --release -p wat --test comms probe_arc294_crashprop_inband -- --test-threads=1

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

/// recv': the connect'd client's `recv'` raises with the service's REAL crash reason
/// (the full structured `#wat.kernel/AssertionFailure` envelope), not a generic disconnect.
#[test]
fn recv_surfaces_crash_reason_in_band() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute-recv)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(tv) => panic!(
            "expected recv' to FAIL (service crashed mid-request); got {:?}",
            tv.value_owned()
        ),
        Err(e) => {
            let s = format!("{e:?}");
            assert!(
                s.contains("RECV-CRASH-REASON-42"),
                "recv' did not surface the real crash reason in-band.\nsaw: {s}"
            );
            assert!(
                !s.contains("channel disconnected"),
                "recv' fell back to the generic disconnect message.\nsaw: {s}"
            );
        }
    }
}

/// poll': the REAL defservice serve loop. A poll'-driven service that crashes handling a
/// Message propagates its reason to the connect'd client's recv' (accepted via
/// wrap_connect_request, not plain accept').
#[test]
fn poll_service_crash_reaches_connected_client() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute-poll)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(tv) => panic!(
            "expected recv' to FAIL (poll' service crashed); got {:?}",
            tv.value_owned()
        ),
        Err(e) => {
            let s = format!("{e:?}");
            assert!(
                s.contains("POLL-CRASH-REASON-88"),
                "poll' service crash did not reach the connect'd client.\nsaw: {s}"
            );
            assert!(
                !s.contains("channel disconnected"),
                "poll' path fell back to the generic disconnect message.\nsaw: {s}"
            );
        }
    }
}

/// select': a crash routes to the `:Lost` ServiceEvent (marker 777), not `:Closed` (111).
#[test]
fn select_surfaces_crash_as_lost() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute-select)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("select' scenario errored: {e:?}"));
    let n = match &got {
        wat::runtime::Value::i64(n) => *n,
        other => panic!("expected i64 marker; got {other:?}"),
    };
    assert_eq!(
        n, 777,
        "select' on a crashing service must yield :Lost (777), not :Closed (111) / :Message (222); got {n}"
    );
}
