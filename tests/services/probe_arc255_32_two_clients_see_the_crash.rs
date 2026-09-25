//! Stone 255.32 — two connected clients and the owner, when the service
//! panics on one op. Pins today's notice on each locus. The thread child
//! also writes its dying declaration to stderr; that is the crash, and
//! stdout is the measurement.

use std::path::Path;
use std::process::{Command, Stdio};

fn run(rel: &str) -> (i32, String, String) {
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

#[test]
fn two_clients_and_the_owner_see_a_service_panic() {
    let (rc, stdout, stderr) = run("tests/services/probe_arc255_32_two_clients_see_the_crash.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    assert_eq!(
        stdout.trim(),
        r#""thread client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread client-b Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"
"process client-b Lost io_uring read failed"
"process owner Lost P32-SERVICE-PANIC-REASON""#,
        "stderr:\n{stderr}\nstdout:\n{stdout}"
    );
}
