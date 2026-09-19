//! the-bracket-peer-path-faces-chaos — work-fn already reaches slow and
//! oversize; die is the dead-runner probe; collect-deadline-ms is injectable;
//! a 1-runner stall past the bound fires collect-gave-up!.
//!
//! cargo nextest run --release -E 'test(bracket_chaos)'

use std::path::Path;
use std::process::Command;

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

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
        // rune:lint(loose-assert) — 1-peer wait is recv-by-deadline, not unbounded select.
        code.contains("collect-wait-one"),
        "1-peer collect wait (recv-by-deadline) is gone — a stall cannot fire GaveUp"
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

/// Oversize work-fn: FrameTooLarge → Lost → REPORT-GONE. Not Malformed, not
/// Rejected. The bracket dies — a worker taking down its coordinator.
#[test]
fn chaos_oversize_is_lost_report_gone_not_malformed() {
    let output = wat_cmd()
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
        "oversize must take down the coordinator; stdout={stdout} stderr={stderr}"
    );
    assert!(
        // rune:lint(loose-assert) — AssertionFailure blob carries location/frames; the arm name is the claim.
        blob.contains("REPORT-GONE"),
        "oversize must hit the Lost empty-alive arm (REPORT-GONE); got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — non-vacuity of the frame cap inside the same blob.
        blob.contains("frame exceeded cap"),
        "oversize non-vacuity: the cap must have fired; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE of the Rejected-arm wording.
        !blob.contains("over-budget frame"),
        "oversize must not be the Rejected arm; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE of Malformed in the raise.
        !blob.contains("ServiceEvent::Malformed"),
        "oversize is not Malformed; got {blob}"
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
        "1-peer stall fires via recv-by-deadline TimedOut; got {blob}"
    );
}
