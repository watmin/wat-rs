//! Dev-only staleness guard for the installed `wat` / `cargo-wat` binary.
//!
//! When a developer edits source in the wat repo and runs `cargo build`
//! without reinstalling (`cargo install --path . --force`),
//! `~/.cargo/bin/wat` silently stays old while `cargo wat` picks up the
//! stale installed copy.  This module detects that drift and screams on
//! stderr — but *only* when the dev source repo is present relative to
//! `pwd`.  A plain user who has just `cargo install`ed wat and has no
//! source checkout never sees a warning.
//!
//! # How the self-disable gate works
//!
//! [`find_dev_repo_root`] walks up the ancestor chain of `cwd` looking
//! for a `Cargo.toml` that carries both `[workspace]` and the sentinel
//! `name = "cargo-wat"`.  That combination is unique to this repo (the
//! workspace root is the `wat` package; `cargo-wat` is one of its two
//! `[[bin]]` targets, arc 170's fold-in of the former `wat-cli` crate).
//! If no such file exists, the guard returns silently — the call is a
//! no-op for anyone who is not in a dev tree.
//!
//! Pre-arc-170 this sentinel was `"crates/wat-cli"` (the sub-crate
//! member line). Folding that crate into core removed the string it
//! anchored on, which would have silently and permanently disabled this
//! guard — caught while relocating this file, not a pre-existing defect.
//!
//! # Staleness detection
//!
//! Uses mtime comparison, not git — mtime catches uncommitted edits
//! that a git-HEAD marker would miss, and requires no build.rs.
//! [`is_stale`] is a pure predicate kept as a standalone function so it
//! can be unit-tested with injected timestamps.
//!
//! # Machine-output cleanliness
//!
//! The warning goes to stderr only, and only when stderr is a tty.
//! When `--check-output edn` or `--check-output json` is detected in
//! argv (machine-readable pipeline mode), or when stderr is piped
//! (process children, nextest capture), the warning is suppressed —
//! a warning-only `eprintln` on a process child's stderr becomes
//! `LociDiedError::Panic.message`. A child that inherits a tty still
//! shares that channel; this gate is a check, not an unrepresentable
//! shape.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ─── Sentinel ──────────────────────────────────────────────────────────────
//
// Both conditions must hold in the same Cargo.toml for the file to be
// accepted as the wat workspace root:
//
// 1. `[workspace]` — it is a workspace manifest, not a member manifest.
// 2. `name = "cargo-wat"` — the `[[bin]]` entry that makes `cargo wat`
//    resolve at all (the binary is literally named `cargo-wat`); unique
//    to this repo.
//
// Together these are unambiguous: any random Rust workspace won't declare
// a `cargo-wat` binary. Adding the package name `name = "wat"` would add
// a third condition but this sentinel alone is already specific enough.

const WORKSPACE_SENTINEL: &str = "name = \"cargo-wat\"";

// ─── Public helpers (unit-tested) ─────────────────────────────────────────

/// Walk up the ancestor chain of `start` looking for the wat workspace
/// root.  Returns `Some(root_dir)` if a `Cargo.toml` that carries both
/// `[workspace]` and the `cargo-wat` bin sentinel is found; `None` otherwise.
///
/// The `None` path is the self-disable gate: if the dev repo is absent
/// (plain binary user, different project) the caller does nothing.
pub fn find_dev_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join("Cargo.toml");
        if let Ok(contents) = std::fs::read_to_string(&candidate) {
            if contents.contains("[workspace]") && contents.contains(WORKSPACE_SENTINEL) {
                return Some(current);
            }
        }
        // Move to parent; stop at filesystem root.
        if !current.pop() {
            return None;
        }
    }
}

/// Pure staleness predicate.  Returns `true` if the newest source mtime
/// is strictly newer than the binary mtime — i.e. the binary is stale.
///
/// Kept as a standalone pure function so unit tests can inject arbitrary
/// `SystemTime` values without touching the filesystem.
pub fn is_stale(binary_mtime: SystemTime, newest_source_mtime: SystemTime) -> bool {
    newest_source_mtime > binary_mtime
}

// ─── Source-tree mtime scan ────────────────────────────────────────────────

/// Walk `dir` recursively, calling `f` on each regular file.
/// Skips sub-directories named `target` or `.git` so the walk stays
/// cheap and bounded even in large workspaces.
fn walk_files(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Prune heavy/irrelevant subtrees.
        if name == "target" || name == ".git" {
            continue;
        }
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => walk_files(&path, f),
            Ok(_) => f(&path),
            Err(_) => {}
        }
    }
}

/// Walk `dir` recursively, calling `f` on each `*.wat` file.
/// Skips `target/` and `.git/` directories.
fn walk_wat_files(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "target" || name == ".git" {
            continue;
        }
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => walk_wat_files(&path, f),
            Ok(_) => {
                if path.extension().and_then(|e| e.to_str()) == Some("wat") {
                    f(&path);
                }
            }
            Err(_) => {}
        }
    }
}

/// Update `newest` if `path`'s mtime is later.
fn update_newest(newest: &mut Option<SystemTime>, path: &Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            match *newest {
                None => *newest = Some(mtime),
                Some(n) if mtime > n => *newest = Some(mtime),
                _ => {}
            }
        }
    }
}

/// Scan build-relevant sources under `repo_root` and return the newest
/// mtime found, or `None` if nothing was readable.
///
/// Scanned paths (mirroring what `cargo build` bakes into the binary):
/// - `{root}/Cargo.toml`
/// - `{root}/src/**` (all files)
/// - `{root}/wat/**/*.wat`
/// - `{root}/crates/*/Cargo.toml` + `{root}/crates/*/src/**`
///
/// `target/` and `.git/` are pruned from every walk.
fn newest_source_mtime(repo_root: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;

    // Root Cargo.toml.
    update_newest(&mut newest, &repo_root.join("Cargo.toml"));

    // src/** (root crate source).
    walk_files(&repo_root.join("src"), &mut |p| {
        update_newest(&mut newest, p);
    });

    // wat/**/*.wat (embedded wat library sources).
    walk_wat_files(&repo_root.join("wat"), &mut |p| {
        update_newest(&mut newest, p);
    });

    // crates/*/ — each sub-crate's manifest + src tree.
    if let Ok(entries) = std::fs::read_dir(repo_root.join("crates")) {
        for entry in entries.flatten() {
            let crate_dir = entry.path();
            if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }
            update_newest(&mut newest, &crate_dir.join("Cargo.toml"));
            walk_files(&crate_dir.join("src"), &mut |p| {
                update_newest(&mut newest, p);
            });
        }
    }

    newest
}

// ─── Entry point ──────────────────────────────────────────────────────────

/// Check whether the installed `wat` binary looks stale compared to the
/// dev source tree.
///
/// - If the current working directory is **not** inside a wat dev
///   checkout (no ancestor `Cargo.toml` with the workspace sentinel),
///   returns **silently** — this is the plain-user fast path.
/// - If `--check-output edn` or `--check-output json` is present in
///   argv, returns silently (machine-readable pipeline; stderr warning
///   would be unexpected noise).
/// - If stderr is **not a tty** (piped, captured, a process child),
///   returns silently. Process children capture stderr as the death
///   wire — a warning-only `eprintln` becomes `LociDiedError::Panic.message`.
///   A child that inherits a tty still shares that channel.
/// - Otherwise, compares the installed binary's mtime against the newest
///   source file mtime.  If the binary is older, prints a loud warning to
///   **stderr** and continues (warning-only; never changes behavior or exit
///   code).
pub fn check_dev_staleness() {
    // Suppress in machine-readable (--check-output) modes.
    // We peek at raw argv ourselves since arg parsing hasn't happened yet.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--check-output") {
        return;
    }

    // Process children, nextest capture, and pipes own stderr as the
    // death/user wire. Only scream at a human terminal.
    if !std::io::stderr().is_terminal() {
        return;
    }

    // Find the dev repo root from cwd upward.  If absent → non-dev user →
    // return silently.
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let repo_root = match find_dev_repo_root(&cwd) {
        Some(r) => r,
        None => return,
    };

    // Get the installed binary's mtime.
    let binary_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let binary_mtime = match std::fs::metadata(&binary_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return,
    };

    // Scan source tree for newest mtime.
    let source_mtime = match newest_source_mtime(&repo_root) {
        Some(t) => t,
        None => return,
    };

    if is_stale(binary_mtime, source_mtime) {
        eprintln!(
            "⚠ wat: the installed binary looks STALE (older than the source at {}). \
             Reinstall:  cargo install --path . --force",
            repo_root.display()
        );
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    // ── is_stale (pure, no I/O) ──────────────────────────────────────────

    #[test]
    fn stale_when_source_newer_than_binary() {
        let binary = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let source = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);
        assert!(is_stale(binary, source));
    }

    #[test]
    fn not_stale_when_binary_newer_than_source() {
        let binary = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);
        let source = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        assert!(!is_stale(binary, source));
    }

    #[test]
    fn not_stale_when_binary_and_source_equal() {
        let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1500);
        assert!(!is_stale(t, t));
    }

    // ── find_dev_repo_root ───────────────────────────────────────────────
    //
    // These tests create tiny temporary directory trees with controlled
    // Cargo.toml contents so results are deterministic regardless of the
    // real filesystem.

    /// Create an isolated temp subtree for one test.  Returns the root dir.
    fn temp_tree(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wat-staleness-test-{}", label));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn find_root_when_cwd_is_the_root() {
        let root = temp_tree("root-at-cwd");
        // Write a Cargo.toml that looks like the wat workspace root.
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\".\"]\n[package]\nname = \"wat\"\n[[bin]]\nname = \"cargo-wat\"\n",
        )
        .unwrap();

        let result = find_dev_repo_root(&root);
        assert_eq!(result, Some(root.clone()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn find_root_when_cwd_is_nested_subdir() {
        let root = temp_tree("root-nested");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\".\"]\n[[bin]]\nname = \"cargo-wat\"\n",
        )
        .unwrap();

        // Start from a deep subdir.
        let subdir = root.join("src").join("distribution");
        fs::create_dir_all(&subdir).unwrap();

        let result = find_dev_repo_root(&subdir);
        assert_eq!(result, Some(root.clone()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn returns_none_when_sentinel_absent() {
        let root = temp_tree("no-sentinel");
        // Cargo.toml with [workspace] but WITHOUT the cargo-wat bin entry.
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"other-crate\"]\n",
        )
        .unwrap();

        let subdir = root.join("src");
        fs::create_dir_all(&subdir).unwrap();

        let result = find_dev_repo_root(&subdir);
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn returns_none_when_no_cargo_toml_anywhere() {
        let root = temp_tree("no-cargo");
        // No Cargo.toml written at all.

        let result = find_dev_repo_root(&root);
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn returns_none_workspace_marker_missing() {
        let root = temp_tree("no-workspace");
        // Has the sentinel string but NOT [workspace].
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"wat\"\n# name = \"cargo-wat\" mentioned for no reason\n",
        )
        .unwrap();

        let result = find_dev_repo_root(&root);
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
