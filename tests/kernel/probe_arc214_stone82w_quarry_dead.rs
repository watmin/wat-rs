//! Arc 214 Stone 8.2w — the quarry dies (FM-2-bis disconfirming gate).
//!
//! Stones 8.1/8.1b/8.2 emptied `src/thread_io.rs` of condemned code; what
//! remains is live, perfected, universe-resident machinery whose home is
//! `src/services/` (the 8.1w annihilation map: "the perfecting forms lift
//! into services/ as each converts; then git rm"). This gate is RED while
//! the quarry file exists or any code path still says `thread_io::`; GREEN
//! when the survivors live in the home and the quarry is `git rm`'d.
//!
//! Historical comments naming "thread_io.rs" as retired-record (no `::`)
//! survive — the scan matches the PATH form `thread_io::` that only live
//! imports/calls/diagnostic-strings use.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone82w_quarry_dead`

use std::fs;
use std::path::Path;

fn scan_dir(dir: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("readable dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            scan_dir(&path, hits);
        } else if path
            .file_name()
            .is_some_and(|n| n == "probe_arc214_stone82w_quarry_dead.rs")
        {
            // Self-exclusion: the tombstone bears the dead man's name — this
            // probe's own comments and needle strings say `thread_io::` and
            // must not count as live references. (8.2w scoring catch: the
            // first cast scanned itself; the only red in the suite was the
            // gate reading its own epitaph.)
            continue;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = fs::read_to_string(&path).expect("readable file");
            for (i, line) in src.lines().enumerate() {
                if line.contains("thread_io::") {
                    hits.push(format!("{}:{} → {}", path.display(), i + 1, line.trim()));
                }
            }
        }
    }
}

/// The quarry file itself must be gone.
#[test]
fn probe_1_thread_io_rs_is_dead() {
    assert!(
        !Path::new("src/thread_io.rs").exists(),
        "src/thread_io.rs must be git rm'd — the survivors live in src/services/ \
         (the 8.1w annihilation map's terminal state)"
    );
}

/// No live `thread_io::` path reference anywhere in src/ or tests/ —
/// imports, calls, AND user-facing diagnostic strings (signal.rs's
/// ServiceNotRunning message names the install fn by path).
#[test]
fn probe_2_no_thread_io_path_references() {
    let mut hits = Vec::new();
    scan_dir(Path::new("src"), &mut hits);
    scan_dir(Path::new("tests"), &mut hits);
    assert!(
        hits.is_empty(),
        "every thread_io:: path must repoint to services:: — the module is dead.\n{}",
        hits.join("\n")
    );
}
