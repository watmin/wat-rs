//! Gate: every `wat-scripts/grep/*.wat` program must MATCH a known target.
//!
//! STONE-three-spellings-one-seam RELAND: Named.name is canonical for every
//! node. A `--check` of a grep program is green even when every name
//! comparison is dead. Only a non-zero match count discriminates.
//! `wat_grep::*` cannot catch this — its fixtures compare `"<-"`, which is
//! identity under the fold.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn run_grep(program: &Path, targets: &[PathBuf]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_wat");
    let stdin_paths: Vec<String> =
        targets.iter().map(|p| format!("\"{}\"", p.display())).collect();
    let mut stdin_edn = String::new();
    stdin_edn.push('[');
    stdin_edn.push_str(&stdin_paths.join(" "));
    stdin_edn.push(']');

    let mut child = Command::new(bin)
        .arg("--grep")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat --grep");
    {
        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(stdin_edn.as_bytes()).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("wait wat --grep")
}

fn match_count(stdout: &str) -> usize {
    stdout.lines().filter(|l| l.contains("#wat.grep/Match")).count()
}

#[test]
fn every_wat_scripts_grep_program_matches_a_known_target() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let grep_dir = manifest.join("wat-scripts/grep");
    let target = manifest.join("tests/cli/grep_smoke_target.wat");
    let mut programs: Vec<PathBuf> = fs::read_dir(&grep_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", grep_dir.display()))
        .filter_map(|e| {
            let p = e.ok()?.path();
            (p.extension().is_some_and(|x| x == "wat")).then_some(p)
        })
        .collect();
    programs.sort();
    assert!(
        !programs.is_empty(),
        "wat-scripts/grep/ has no .wat programs — this gate would pass vacuously"
    );

    let mut failures = Vec::new();
    for program in &programs {
        let output = run_grep(program, std::slice::from_ref(&target));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let n = match_count(&stdout);
        if !output.status.success() || n == 0 {
            failures.push(format!(
                "  {}  matches={n} status={} stderr={stderr} stdout={stdout}",
                program.display(),
                output.status
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "wat-scripts/grep programs produced 0 matches (or failed) on the smoke target — \
         name comparisons are dead or the program does not run:\n{}",
        failures.join("\n")
    );
}
