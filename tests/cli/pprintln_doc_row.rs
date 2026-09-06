//! pprintln of a `:wat::doc::Row` — byte golden. Capture is the binary's stdout
//! (pprintln of a VALUE, not a String).

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cli").join(name)
}

fn pprintln_stdout(program: &PathBuf) -> String {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(program)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected clean run; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout
}

#[test]
fn doc_row_pprintln_matches_byte_golden() {
    let stdout = pprintln_stdout(&fixture("pprintln_doc_row.wat"));
    assert_eq!(
        stdout.as_str(),
        include_str!("pprintln_doc_row__step_payload.edn"),
        "byte golden: #wat.doc/Row from a record value"
    );
}

#[test]
fn printed_row_edn_read_round_trips() {
    let stdout = pprintln_stdout(&fixture("pprintln_doc_row.wat"));
    wat_edn::parse_owned(stdout.trim_end()).unwrap_or_else(|e| {
        panic!("edn::read of the printed row must succeed (literal newlines are the prose mode):\n{e}\n{stdout}")
    });
}

#[test]
fn ordinary_multiline_string_stays_escaped() {
    let stdout = pprintln_stdout(&fixture("pprintln_doc_row_note.wat"));
    assert_eq!(
        stdout.as_str(),
        include_str!("pprintln_doc_row__note.edn"),
        "a non-Row multi-line string stays escaped"
    );
}
