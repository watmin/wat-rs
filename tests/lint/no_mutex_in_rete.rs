//! Stone 27 gate 1: `src/rete` has no `Mutex`. ZERO-MUTEX intern is thread-owned
//! `RefCell` (`DESIGN-STONE-intern-zero-mutex`). A Mutex landing in the intern
//! door must be a red build, not a breadcrumb.

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn uses_mutex(line: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with("//") {
        return false;
    }
    line.contains("Mutex<") || line.contains("Mutex::") || line.contains("use std::sync::Mutex")
}

#[test]
fn rete_home_has_no_mutex() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let rete = Path::new(manifest).join("src/rete");
    let mut files = Vec::new();
    collect_rs(&rete, &mut files);
    assert!(
        files.len() > 10,
        "the no-Mutex walk found only {} src/rete .rs files",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        for (idx, line) in src.lines().enumerate() {
            if uses_mutex(line) && !line.contains("rune:lint(rete-mutex)") {
                violations.push(format!("{}:{}  {}", rel, idx + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "src/rete contains Mutex (stone 27 gate 1):\n{}",
        violations.join("\n")
    );
}
