//! Runs every `.wat` file in `wat-tests/` and asserts each reports
//! only `<name>:PASS` lines on stdout. The Rust-side runner for the
//! wat-tests/ convention, to be replaced by `wat test wat-tests/`
//! when arc 007 slice 4 lands.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run_wat_file(path: &std::path::Path) -> (Vec<String>, Vec<String>) {
    let src = std::fs::read_to_string(path).expect("read wat file");
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup {:?}: {}", path, e));
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args)
        .unwrap_or_else(|e| panic!("invoke :user::main in {:?}: {:?}", path, e));
    let stdout_lines = snapshot_lines(&*stdout);
    let stderr_lines = snapshot_lines(&*stderr);
    (stdout_lines, stderr_lines)
}

fn snapshot_lines(writer: &StringIoWriter) -> Vec<String> {
    let bytes = writer.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

#[test]
fn every_wat_test_file_passes() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wat-tests");
    let entries: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {:?}: {}", dir, e))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wat"))
        .collect();

    assert!(
        !entries.is_empty(),
        "wat-tests/ should contain at least one .wat file"
    );

    let mut failures: Vec<String> = Vec::new();
    for path in &entries {
        let (stdout, stderr) = run_wat_file(path);
        if stdout.is_empty() {
            failures.push(format!(
                "{:?}: no stdout — did :user::main write any test results?",
                path.file_name().unwrap_or_default()
            ));
            continue;
        }
        for line in &stdout {
            if !line.ends_with(":PASS") {
                failures.push(format!(
                    "{:?}: {}",
                    path.file_name().unwrap_or_default(),
                    line
                ));
            }
        }
        // Stderr isn't used as signal here — tests may legitimately
        // write to it. Just making sure we don't drop the capture
        // silently in case a future test wants it.
        let _ = stderr;
    }

    assert!(
        failures.is_empty(),
        "wat-tests/ reported {} failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
