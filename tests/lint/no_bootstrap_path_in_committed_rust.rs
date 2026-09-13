//! Committed tests must not depend on `bootstrap/`, which is gitignored and
//! empty on a clone. A string literal beginning `"bootstrap/` in `src/` or
//! `tests/` is that dependency.

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".claude" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn code_before_comment(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

fn has_bootstrap_string_literal(line: &str) -> bool {
    let code = code_before_comment(line);
    code.contains("\"bootstrap/") || code.contains("r\"bootstrap/") || code.contains("r#\"bootstrap/")
}

#[test]
fn committed_rust_does_not_literal_bootstrap_paths() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in ["src", "tests"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        if f.file_name().and_then(|n| n.to_str())
            == Some("no_bootstrap_path_in_committed_rust.rs")
        {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        for (idx, line) in src.lines().enumerate() {
            if has_bootstrap_string_literal(line) {
                violations.push(format!("{}:{}", rel, idx + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "committed Rust under src/ or tests/ contains a string literal beginning \"bootstrap/\" \
         (gitignored; empty on a clone): {violations:?}"
    );
}
