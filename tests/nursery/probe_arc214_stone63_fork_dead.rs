//! Arc 214 Stone 6.3 — fork.rs dies (FM-2-bis disconfirming gate).
//!
//! The builder's call: "fork.rs dies in this marathon." The process family
//! (fork.rs + spawn.rs + spawn_process.rs + process_stdio.rs) rehomes into
//! `src/process/` — the layout RATIFIED BY THE INTUERI MINT CAST (the cast
//! names; mod/clone/child/handle/verbs/stdio). 6.3a fells fork.rs; 6.3b
//! brings the family; this gate covers the full kill.
//!
//! RED while any of the four flat files exist or any code path says the old
//! module names; GREEN when the family lives home and the quarries are gone.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone63_fork_dead`

use std::fs;
use std::path::Path;

const OLD_PATHS: [&str; 4] = [
    "crate::fork::",
    "wat::fork::",
    "crate::spawn_process::",
    "wat::spawn_process::",
];

fn scan_dir(dir: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("readable dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            scan_dir(&path, hits);
        } else if path
            .file_name()
            .is_some_and(|n| n == "probe_arc214_stone63_fork_dead.rs")
        {
            // Self-exclusion from birth (the 82w epitaph lesson).
            continue;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = fs::read_to_string(&path).expect("readable file");
            for (i, line) in src.lines().enumerate() {
                if OLD_PATHS.iter().any(|p| line.contains(p)) {
                    hits.push(format!("{}:{} → {}", path.display(), i + 1, line.trim()));
                }
            }
        }
    }
}

/// The quarry files must be gone (6.3a: fork.rs; 6.3b: the rest).
#[test]
fn probe_1_flat_process_family_files_are_dead() {
    let corpses = [
        "src/fork.rs",
        "src/spawn.rs",
        "src/spawn_process.rs",
        "src/process_stdio.rs",
    ];
    let alive: Vec<&str> = corpses
        .iter()
        .filter(|p| Path::new(p).exists())
        .copied()
        .collect();
    assert!(
        alive.is_empty(),
        "the process family lives in src/process/ (the intueri-ratified home); \
         flat files still standing: {:?}",
        alive
    );
}

/// No live old-module path reference anywhere in src/ or tests/.
#[test]
fn probe_2_no_old_process_family_paths() {
    let mut hits = Vec::new();
    scan_dir(Path::new("src"), &mut hits);
    scan_dir(Path::new("tests"), &mut hits);
    assert!(
        hits.is_empty(),
        "every fork::/spawn_process:: path must repoint to process:: — the \
         flat modules are dead.\n{}",
        hits.join("\n")
    );
}
