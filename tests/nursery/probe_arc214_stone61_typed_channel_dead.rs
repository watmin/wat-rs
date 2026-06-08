//! Arc 214 Stone 6.1 — the wall falls (FM-2-bis disconfirming gate).
//!
//! Slice 6 per the campaign map (RESUME-SLICE-4-9 § Slice 6): "retire
//! `typed_channel.rs` — typed_channel dies here." The shim's perfected
//! survivors (the transport-polymorphic SenderInner/ReceiverInner seam, the
//! Send/RecvOutcome surface, the typed ops) lift into the `src/channel/`
//! home per the 8.1w/8.2w precedent; `bounded<T>`'s two live tenants convert
//! to `comms::thread::pair`; the quarry file is `git rm`'d.
//!
//! RED while the file exists or any code path says `typed_channel::`;
//! GREEN when the survivors live home and the quarry is gone.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone61_typed_channel_dead`

use std::fs;
use std::path::Path;

fn scan_dir(dir: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("readable dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            scan_dir(&path, hits);
        } else if path
            .file_name()
            .is_some_and(|n| n == "probe_arc214_stone61_typed_channel_dead.rs")
        {
            // Self-exclusion from birth: the tombstone bears the dead man's
            // name (the 82w lesson — the first quarry gate read its own
            // epitaph; this one never does).
            continue;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = fs::read_to_string(&path).expect("readable file");
            for (i, line) in src.lines().enumerate() {
                if line.contains("typed_channel::") {
                    hits.push(format!("{}:{} → {}", path.display(), i + 1, line.trim()));
                }
            }
        }
    }
}

/// The quarry file itself must be gone.
#[test]
fn probe_1_typed_channel_rs_is_dead() {
    assert!(
        !Path::new("src/typed_channel.rs").exists(),
        "src/typed_channel.rs must be git rm'd — the survivors live in \
         src/channel/ (the Slice-6 structural wall)"
    );
}

/// No live `typed_channel::` path reference anywhere in src/ or tests/.
#[test]
fn probe_2_no_typed_channel_path_references() {
    let mut hits = Vec::new();
    scan_dir(Path::new("src"), &mut hits);
    scan_dir(Path::new("tests"), &mut hits);
    assert!(
        hits.is_empty(),
        "every typed_channel:: path must repoint to channel:: — the module is dead.\n{}",
        hits.join("\n")
    );
}
