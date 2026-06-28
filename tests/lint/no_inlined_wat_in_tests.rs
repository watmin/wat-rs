//! THE ABSOLUTE LINT (test-infra annihilation, part C) — bans inlined-wat worlds in tests.
//!
//! Builder-directed ("hard bandaid pull… we witness the fire of our creation"): a test must get its
//! world from a co-located `.wat` fixture — `startup_beside(file!())` for real wat-under-test — or
//! from `startup_bare()` for an incidental world (a pure-Rust substrate test that just needs a frozen
//! world). Building the world from an inlined string is the violation this lint annihilates.
//!
//! It scans every `tests/**/*.rs` and FAILS listing every offender — the campaign's progress meter.
//! Drive it to ZERO, chunk by chunk (group by group). Until then this is the ONE expected-red test;
//! nextest isolates it, so a SECOND red is a real regression.
//!
//! Escape hatch: a file with a genuine need for a dynamically-constructed world carries a
//! `// LINT-ALLOW-INLINE-WAT: <reason>` rune and is skipped (rare — the reason must earn it).

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

#[test]
fn tests_carry_no_inlined_wat() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = Path::new(manifest).join("tests");
    let mut files = Vec::new();
    collect_rs(&root, &mut files);
    files.sort();

    // The forbidden world-builder. A real wat-under-test world comes from a co-located fixture
    // (`startup_beside`); an incidental world from `startup_bare()` — neither is this call.
    let needle = ["startup_from", "_source("].concat();

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden call in its own detector/doc — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_inlined_wat_in_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(f).expect("read test source");
        if src.to_lowercase().contains("// lint-allow-inline-wat") {
            continue;
        }
        if src.contains(&needle) {
            let rel = f.strip_prefix(manifest).unwrap_or(f);
            violations.push(rel.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 INLINED-WAT IN TESTS — {} file(s) still build their world from an inlined string.\n\
         A test gets its world from a co-located `.wat` (`startup_beside(file!())`) for real wat-under-\n\
         test, or from `startup_bare()` for an incidental world. This is the fire — drive it to ZERO\n\
         (test-infra annihilation, group by group). Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}
