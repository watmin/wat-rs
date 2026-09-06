//! Row 12 — the example from the metadata-of LOOKUP formats to the ruled shape,
//! `widest <= 120`. No `:wat::intrinsic::examples` scan.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cli").join(name)
}

#[test]
fn example_from_the_lookup_formats_widest_le_120() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let program = fixture("metadata_of_example_formats.wat");
    let output = Command::new(bin)
        .arg(&program)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat metadata_of_example_formats.wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected clean run; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let line = stdout
        .lines()
        .find(|l| l.contains("FORMATTED widest="))
        .unwrap_or_else(|| panic!("expected FORMATTED widest= line; stdout:\n{stdout}"));
    // `println` EDN-quotes the string: `"SOURCE  widest=…   FORMATTED widest=72"`
    let widest: i64 = line
        .rsplit("FORMATTED widest=")
        .next()
        .and_then(|s| s.trim().trim_matches('"').parse().ok())
        .unwrap_or_else(|| panic!("could not parse widest from {line:?}"));
    let source_widest: i64 = line
        .split("widest=")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("could not parse SOURCE widest from {line:?}"));
    assert!(
        source_widest > 120,
        "the looked-up example must be a real form (source widest > 120); got {source_widest} ({line})"
    );
    assert!(
        widest <= 120,
        "formatted example from the lookup must be widest <= 120; got {widest} ({line})"
    );
}