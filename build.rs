//! build.rs — auto-generate the module list for every grouped integration-test
//! binary, so a test file can NEVER be silently forgotten.
//!
//! # Why this exists (the failure class it annihilates)
//!
//! Grouped integration tests live under `tests/<group>/` with a single
//! `[[test]] path = "tests/<group>/mod.rs"` Cargo entry; the sibling `*.rs`
//! files are brought in via `mod` declarations. Previously that list was
//! hand-maintained by a bash script (`gen-test-mods.sh`) you had to REMEMBER
//! to run; a `--check` gate caught drift only at green-gate time, leaving a
//! dev-loop window where a freshly-added test was simply not compiled.
//!
//! This build script removes the window by construction. Cargo runs `build.rs`
//! on every build/test, so the list is ALWAYS current — there is no committed
//! mod-list to drift, no script to forget, no gate to maintain. Drop a `.rs`
//! into a group dir and it is compiled + run on the next `cargo test`. The
//! mod-list is generated into `OUT_DIR` (never the source tree — a build script
//! that rewrites tracked files dirties git and can need two builds to settle);
//! each group's committed `mod.rs` is a thin `include!` stub.
//!
//! # Later (the wat-as-management track)
//!
//! The dir-listing + list computation will migrate to wat once wat has fs
//! syscalls (`readdir`) — run as a separately-distributed, prebuilt stage0 wat
//! binary used to bootstrap the build (the rustc pattern), NOT called from
//! within this crate's own build (which is circular: the binary is what's being
//! built). Until then this stays pure Rust — the always-runs guarantee.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let tests_dir = Path::new(&manifest).join("tests");

    // Re-run if this script changes.
    println!("cargo:rerun-if-changed=build.rs");

    // ── wat-tests/ discovery — a NEW .wat file must not be invisible ──────────────
    //
    // `wat::test! {}` (crates/wat-macros) globs `wat-tests/` at EXPANSION time and emits
    // an `include_bytes!` per discovered file, which makes Cargo recompile the test binary
    // when a known file's CONTENTS change. It cannot catch an ADDITION: a file that did not
    // exist when the macro last ran has no `include_bytes!` pointing at it, so Cargo has no
    // edge to it and never re-expands. Measured 2026-08-29: dropping in a new wat-tests file
    // and running `cargo build --release --tests -p wat` finished in 0.08 s with the deftest
    // NOT registered, and reads as "my test did not register" — sending you after the deftest
    // name or the macro instead of the build graph.
    //
    // The cure is the idiom this file already uses for `tests/<group>` below: watch the
    // DIRECTORY. Linux bumps a directory's mtime when a child is added or removed, so a
    // directory edge catches exactly the case the file edges cannot. Subdirectories are
    // walked because `wat-tests/edn/x.wat` bumps `wat-tests/edn`, not `wat-tests`.
    fn watch_wat_dirs(dir: &Path) {
        if let Some(s) = dir.to_str() {
            println!("cargo:rerun-if-changed={s}");
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    watch_wat_dirs(&p);
                }
            }
        }
    }
    let wat_tests_dir = Path::new(&manifest).join("wat-tests");
    if wat_tests_dir.is_dir() {
        watch_wat_dirs(&wat_tests_dir);
    }

    let entries = match fs::read_dir(&tests_dir) {
        Ok(e) => e,
        Err(_) => return, // no tests/ dir → nothing to generate
    };

    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        // A group is any tests/<group>/ that opts in by having a mod.rs.
        if !dir.join("mod.rs").exists() {
            continue;
        }
        let group = match dir.file_name().and_then(|n| n.to_str()) {
            Some(g) => g.to_string(),
            None => continue,
        };

        // Re-run when files are added/removed in this group dir (Linux tracks
        // the dir's mtime, so a new/deleted sibling .rs re-triggers generation).
        println!("cargo:rerun-if-changed=tests/{group}");

        // Every sibling *.rs except mod.rs, sorted for a stable, diffable list.
        let mut stems: Vec<String> = fs::read_dir(&dir)
            .expect("read group dir")
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "rs").unwrap_or(false))
            .filter(|p| p.file_name().map(|n| n != "mod.rs").unwrap_or(false))
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect();
        stems.sort();

        let mut generated = String::new();
        generated.push_str(&format!(
            "// @generated by build.rs — do not edit. Module list for the `{group}`\n\
             // integration-test group, derived from tests/{group}/*.rs at build time.\n"
        ));
        for stem in &stems {
            let abs = dir.join(format!("{stem}.rs"));
            let abs = abs.to_str().expect("utf-8 path");
            // Absolute #[path] is robust regardless of the include! site.
            // #[allow(non_snake_case)]: some arc-numbered test files carry
            // capitals (e.g. probe_arc237_sC2ab_*); the names are intentional.
            generated.push_str(&format!(
                "#[allow(non_snake_case)]\n#[path = {abs:?}]\nmod {stem};\n"
            ));
        }

        let dest = Path::new(&out_dir).join(format!("{group}_mods.rs"));
        fs::write(&dest, generated).expect("write generated mod list");
    }

    // Arc 170 step 4 — a spawned runtime EXECS the `wat` binary. A process that
    // never entered wat's CLI entry (a cargo test harness, whose `main` belongs
    // to libtest) cannot serve as one, so it needs to know where the real
    // binary is. OUT_DIR is `target/<profile>/build/<pkg>-<hash>/out`; three
    // parents up is `target/<profile>/`, where cargo puts the bin targets.
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let profile_dir = std::path::Path::new(&out_dir)
            .ancestors()
            .nth(3)
            .map(std::path::Path::to_path_buf);
        if let Some(dir) = profile_dir {
            println!(
                "cargo:rustc-env=WAT_RUNTIME_BIN_DEFAULT={}",
                dir.join("wat").display()
            );
        }
    }
}
