//! Committed tests must not depend on `bootstrap/`, which is gitignored and
//! empty on a clone. Any committed `.rs` line in `src/` or `tests/` that names
//! `bootstrap/` outside a `//` line comment is that dependency: a literal
//! `"bootstrap/…"`, a path built as `format!("{}/bootstrap/…", env!("CARGO_MANIFEST_DIR"))`,
//! a raw string. A trailing comment that names it fails too — loud, never silent.

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

/// Not "a literal beginning `"bootstrap/`": that missed `format!("{}/bootstrap/…")`, the shape a
/// test builds an absolute path in (measured 2026-09-13, mutation M2a). Only a whole `//` line is
/// exempt, so a `//` inside a string (`"https://…"`) cannot hide the rest of its line.
fn names_bootstrap_dir(line: &str) -> bool {
    !line.trim_start().starts_with("//") && line.contains("bootstrap/")
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
            if names_bootstrap_dir(line) {
                violations.push(format!("{}:{}", rel, idx + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "committed Rust under src/ or tests/ names the gitignored bootstrap/ dir \
         (empty on a clone): {violations:?}"
    );
}
