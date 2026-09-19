//! the-bracket-peer-path-faces-chaos — work-fn already reaches slow and
//! oversize; die is the dead-runner probe; collect-deadline-ms is injectable;
//! a 1-runner stall past the bound fires collect-gave-up!.
//!
//! cargo nextest run --release -E 'test(bracket_chaos)'

use std::path::Path;
use std::process::Command;

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn spawn_src() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/kernel/spawn.rs");
    std::fs::read_to_string(p).expect("src/kernel/spawn.rs must be readable")
}

fn bracket_code() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("wat/bracket.wat");
    let src = std::fs::read_to_string(p).expect("wat/bracket.wat must be readable");
    src.lines()
        .map(|l| match l.find(";;") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_string(path: &str) -> String {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(Value::i64(n)) => n.to_string(),
        Ok(other) => panic!("{path} returned {other:?}"),
        Err(e) => panic!("{path} raised: {e:?}"),
    }
}

fn wat_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wat"))
}

fn field<'a>(summary: &'a str, key: &str) -> &'a str {
    for part in summary.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("summary missing field {key:?}: {summary}");
}

fn i64_field(summary: &str, key: &str) -> i64 {
    field(summary, key)
        .parse()
        .unwrap_or_else(|_| panic!("{key} not an i64 in {summary}"))
}

/// One home for the value: the wat defn that held 300000 is gone; the call
/// site names the program intrinsic.
#[test]
fn collect_deadline_has_one_home() {
    let code = bracket_code();
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE of the retired literal in comment-stripped source.
        !code.contains("300000"),
        "collect-deadline-ms literal leaked back into wat/bracket.wat"
    );
    assert!(
        // rune:lint(loose-assert) — the call site must name the intrinsic.
        code.contains(":wat::program::collect-deadline-ms"),
        "map-worker no longer calls the injectable collect deadline"
    );
    assert!(
        // rune:lint(loose-assert) — N-way wait is the bounded verb, not unbounded select.
        code.contains("select-by-deadline"),
        "collect-loop no longer calls select-by-deadline — a stall cannot fire GaveUp"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: unbounded select must not be the wait.
        !code.contains(":wat::kernel::select live-peers")
            && !code.contains("collect-wait-one"),
        "collect-loop wait fell back to unbounded select or collect-wait-one"
    );
}

#[test]
fn collect_deadline_default_is_production() {
    let got = run_string("tests/kernel/probe_bracket_chaos_deadline.wat");
    assert_eq!(got, "300000", "production default must stay 300000; got {got}");
}

/// Slow work-fn: ServiceEvent::Message, map survives, elapsed at least 200ms.
#[test]
fn chaos_slow_is_message_and_survives() {
    let stored = run_string("tests/kernel/probe_bracket_chaos_slow.wat");
    eprintln!("slow {stored}");
    assert_eq!(field(&stored, "arm"), "Message", "slow arm; got {stored}");
    assert_eq!(field(&stored, "got"), "11", "slow result; got {stored}");
    assert!(
        i64_field(&stored, "elapsed-ms") >= 200,
        "slow nap must have fired; got {stored}"
    );
}

/// Re-dispatch must not block. collect-feed-idle uses try-send; a wedged
/// runner (write_all after FrameTooLarge) WouldBlock instead of hanging send.
#[test]
fn collect_feed_idle_uses_try_send() {
    let code = bracket_code();
    let start = code
        .find(":wat::bracket::collect-feed-idle")
        .expect("collect-feed-idle missing");
    let body = &code[start..];
    let end = body
        .find(":wat::bracket::collect-report-gone!")
        .unwrap_or(body.len());
    let door = &body[..end];
    assert!(
        // rune:lint(loose-assert) — the dispatch bound.
        door.contains(":wat::kernel::try-send"),
        "collect-feed-idle no longer try-sends — a blocked send is outside every deadline"
    );
    assert!(
        // rune:lint(loose-assert) — WouldBlock names the wedge.
        door.contains("TrySendOutcome::WouldBlock"),
        "collect-feed-idle does not face WouldBlock"
    );
}

/// ⭐ Mutation: FrameTooLarge at classify_peer_error is Rejected, not Lost.
/// Put Lost back and the oversize probe kills the coordinator again.
#[test]
fn classify_peer_error_frame_too_large_is_rejected() {
    let src = spawn_src();
    let start = src
        .find("pub fn classify_peer_error")
        .expect("classify_peer_error missing");
    let body = &src[start..];
    let end = body.find("pub struct ProcessPeerBundle").unwrap_or(body.len());
    let door = &body[..end];
    assert!(
        // rune:lint(loose-assert) — the reclassification at the one door.
        door.contains("RecvError::FrameTooLarge => PeerDeath::Rejected"),
        "FrameTooLarge no longer maps to PeerDeath::Rejected in classify_peer_error"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE of the death classification.
        !door.contains("RecvError::FrameTooLarge => PeerDeath::Lost"),
        "FrameTooLarge is death again — the DoS this stone closes"
    );
}

/// Oversize: FrameTooLarge → Rejected → drop wedged runner → REPORT-GONE.
/// Bound 2000 ms is above the RETRY deadlock cliff; this must finish promptly,
/// not hang. Cap reason named. Not the old Rejected panic. Not Malformed.
#[test]
fn chaos_oversize_does_not_deadlock_above_the_cliff() {
    let output = wat_cmd()
        .env("WAT_COLLECT_DEADLINE_MS", "2000")
        .arg("tests/kernel/probe_bracket_chaos_oversize.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let blob = format!("{stdout}{stderr}");
    eprintln!("oversize status={:?} blob={blob}", output.status.code());
    assert_eq!(
        output.status.code(),
        Some(2),
        "1-runner oversize must REPORT-GONE (runner unusable); stdout={stdout} stderr={stderr}"
    );
    assert!(
        // rune:lint(loose-assert) — last runner dropped, no survivor.
        blob.contains("REPORT-GONE"),
        "oversize last-runner must REPORT-GONE; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — non-vacuity: the cap named in the raise.
        blob.contains("frame exceeded cap"),
        "oversize non-vacuity: the cap must have fired; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE of the old Rejected-arm panic.
        !blob.contains("over-budget frame"),
        "oversize must not be the old Rejected panic; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — not Malformed; different fact.
        !blob.contains("ServiceEvent::Malformed"),
        "oversize is not Malformed; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — the RETRY deadlock was GaveUp-or-hang; this is prompt.
        !blob.contains("GaveUp"),
        "oversize must not wait the wall (RETRY deadlock); got {blob}"
    );
}

/// Stall past an injected 200ms bound. collect-gave-up! with wall-clock reason.
#[test]
fn chaos_stall_fires_collect_gave_up() {
    let output = wat_cmd()
        .env("WAT_COLLECT_DEADLINE_MS", "200")
        .arg("tests/kernel/probe_bracket_chaos_stall.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let blob = format!("{stdout}{stderr}");
    eprintln!("stall status={:?} blob={blob}", output.status.code());
    assert_eq!(
        output.status.code(),
        Some(2),
        "stall past the bound must GaveUp; stdout={stdout} stderr={stderr}"
    );
    assert!(
        // rune:lint(loose-assert) — AssertionFailure blob carries location/frames; GaveUp is the claim.
        blob.contains("GaveUp"),
        "collect-gave-up! must have fired; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — the injected bound is a substring of the interpolate.
        blob.contains("wall-clock bound 200 ms"),
        "the injected bound must be the one named; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — last=TimedOut names the recv-by-deadline arm, not a golden.
        blob.contains("last=TimedOut"),
        "1-peer stall fires via select-by-deadline TimedOut; got {blob}"
    );
}
