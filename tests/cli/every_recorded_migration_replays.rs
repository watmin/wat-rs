//! Gate: every recorded migration in `wat-scripts/fixes/*.wat` carries a
//! replay fixture XOR a header rune `rune:replay(unreadable-preimage)`, and a
//! `;; SCOPE:` line. Nothing is exempt by silence.
//!
//! A fixture is `wat-scripts/fixes/replay/<stem>/{before.pre,after.post}` (optional
//! `stdin`). The oracle is HISTORY or the codemod's header spec — never the
//! tool under test. The gate asserts byte-exact output, idempotence, and
//! non-vacuity against that oracle.
//!
//! `.pre`/`.post` stay out of `wat_scripts_fixes_load.rs` (walks `*.wat`) and
//! `every_tracked_wat_parses.rs` (`git ls-files '*.wat'`).
//!
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixes_dir() -> PathBuf {
    manifest().join("wat-scripts/fixes")
}

fn replay_dir() -> PathBuf {
    fixes_dir().join("replay")
}

fn top_level_stems() -> Vec<String> {
    let mut stems: Vec<String> = fs::read_dir(fixes_dir())
        .unwrap_or_else(|e| panic!("read {}: {e}", fixes_dir().display()))
        .filter_map(|e| {
            // rune:lint(one-variant-separator, not-a-name) — DirEntry::path() is a filesystem path.
            let p = e.ok()?.path();
            (p.is_file() && p.extension().is_some_and(|x| x == "wat")).then_some(p)
        })
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    stems.sort();
    stems
}

fn fixture_dir(stem: &str) -> PathBuf {
    replay_dir().join(stem)
}

fn has_fixture(stem: &str) -> bool {
    let d = fixture_dir(stem);
    d.join("before.pre").is_file() && d.join("after.post").is_file()
}

fn rune_reason(stem: &str) -> Option<String> {
    let path = fixes_dir().join(format!("{stem}.wat"));
    let src = fs::read_to_string(&path).ok()?;
    for line in src.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(";;") else {
            continue;
        };
        let rest = rest.trim();
        let Some(rest) = rest.strip_prefix("rune:replay(unreadable-preimage)") else {
            continue;
        };
        let rest = rest.trim();
        let rest = rest.strip_prefix("—").or_else(|| rest.strip_prefix("-"))?;
        let reason = rest.trim();
        if !reason.is_empty() {
            return Some(reason.to_string());
        }
    }
    None
}

fn has_rune_line(stem: &str) -> bool {
    let path = fixes_dir().join(format!("{stem}.wat"));
    let Ok(src) = fs::read_to_string(&path) else {
        return false;
    };
    src.lines().any(|l| {
        l.trim()
            .trim_start_matches(';')
            .trim()
            .starts_with("rune:replay(unreadable-preimage)")
    })
}

/// Header `;; SCOPE: <entry> …` lines. Exactly one, non-empty, is required of every
/// fixtured or runed stem. `corpus` means tracked `*.wat` outside `wat-scripts/fixes/`.
fn scope_lines(stem: &str) -> Vec<String> {
    let path = fixes_dir().join(format!("{stem}.wat"));
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{stem}: read: {e}"));
    let mut found = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(";;") else {
            continue;
        };
        let rest = rest.trim();
        let Some(rest) = rest.strip_prefix("SCOPE:") else {
            continue;
        };
        found.push(rest.trim().to_string());
    }
    found
}

fn git_ls_files(globs: &[&str]) -> Vec<String> {
    let out = Command::new("git")
        .args(["-C", manifest().to_str().unwrap(), "ls-files", "--"])
        .args(globs)
        .output()
        .expect("git ls-files");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn scope_globs_hit_a_tracked_file(stem: &str, entries: &[String]) -> Result<(), String> {
    for ent in entries {
        if ent == "corpus" {
            let all = git_ls_files(&["*.wat"]);
            let n = all
                .iter()
                .filter(|p| !p.starts_with("wat-scripts/fixes/"))
                .count();
            if n == 0 {
                return Err(format!("{stem}: SCOPE corpus matches no tracked *.wat outside wat-scripts/fixes/"));
            }
            continue;
        }
        let hits = git_ls_files(&[ent.as_str()]);
        if hits.is_empty() {
            return Err(format!("{stem}: SCOPE glob `{ent}` matches no tracked file"));
        }
    }
    Ok(())
}

fn fixture_stems() -> Vec<String> {
    top_level_stems()
        .into_iter()
        .filter(|s| has_fixture(s))
        .collect()
}

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn fresh_dir() -> PathBuf {
    let n = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!(
        "wat-replay-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    fs::create_dir_all(&p).unwrap_or_else(|e| panic!("mkdir {}: {e}", p.display()));
    p
}

fn run_codemod(codemod: &Path, stdin_edn: &str) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(codemod)
        .current_dir(manifest())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn wat {}: {e}", codemod.display()));
    {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin_edn.as_bytes())
            .unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("wait wat")
}

fn default_path_vector(file: &str) -> String {
    // Built from chars so the default stdin is not an inlined EDN string literal.
    let mut s = String::new();
    s.push('[');
    s.push('"');
    s.push_str(file);
    s.push('"');
    s.push(']');
    s.push('\n');
    s
}

/// `None` = pass. `Some(msg)` = this stem failed. IO on the fixture files is a
/// broken test precondition and panics (no `Result<(), String>`).
fn replay_one(stem: &str) -> Option<String> {
    let dir = fixture_dir(stem);
    let before = fs::read(dir.join("before.pre"))
        .unwrap_or_else(|e| panic!("{stem}: read before.pre: {e}"));
    let after = fs::read(dir.join("after.post"))
        .unwrap_or_else(|e| panic!("{stem}: read after.post: {e}"));
    if before == after {
        return Some(format!("{stem}: before.pre == after.post (vacuous fixture)"));
    }
    let tmp = fresh_dir();
    let work = tmp.join(format!("{stem}.wat"));
    fs::write(&work, &before).unwrap_or_else(|e| panic!("{stem}: write temp: {e}"));
    let work_abs = work
        .canonicalize()
        .unwrap_or_else(|e| panic!("{stem}: canonicalize: {e}"));
    let work_abs_s = work_abs.display().to_string();
    let stdin_edn = match fs::read_to_string(dir.join("stdin")) {
        Ok(s) => s.replace("{FILE}", &work_abs_s),
        Err(_) => default_path_vector(&work_abs_s),
    };
    let codemod_abs = fixes_dir()
        .join(format!("{stem}.wat"))
        .canonicalize()
        .unwrap_or_else(|e| panic!("{stem}: canonicalize codemod: {e}"));

    let out1 = run_codemod(&codemod_abs, &stdin_edn);
    if !out1.status.success() {
        let _ = fs::remove_dir_all(&tmp);
        return Some(format!(
            "{stem}: first run rc={} stdout={} stderr={}",
            out1.status,
            String::from_utf8_lossy(&out1.stdout),
            String::from_utf8_lossy(&out1.stderr)
        ));
    }
    let got = fs::read(&work).unwrap_or_else(|e| panic!("{stem}: read result: {e}"));
    if got != after {
        let _ = fs::remove_dir_all(&tmp);
        return Some(format!(
            "{stem}: result != after.post\n--- got ---\n{}\n--- want ---\n{}",
            String::from_utf8_lossy(&got),
            String::from_utf8_lossy(&after)
        ));
    }

    let out2 = run_codemod(&codemod_abs, &stdin_edn);
    if !out2.status.success() {
        let _ = fs::remove_dir_all(&tmp);
        return Some(format!(
            "{stem}: second run rc={} stdout={} stderr={}",
            out2.status,
            String::from_utf8_lossy(&out2.stdout),
            String::from_utf8_lossy(&out2.stderr)
        ));
    }
    let got2 = fs::read(&work).unwrap_or_else(|e| panic!("{stem}: read 2nd: {e}"));
    let _ = fs::remove_dir_all(&tmp);
    if got2 != after {
        return Some(format!("{stem}: second run was not idempotent"));
    }
    None
}

fn replay_fixtures_shard(shard: usize) {
    let stems: Vec<String> = fixture_stems()
        .into_iter()
        .filter(|s| s != "positional-ctor-to-map")
        .collect();
    let mut failures = Vec::new();
    for (i, stem) in stems.iter().enumerate() {
        if i % 16 != shard {
            continue;
        }
        if let Some(e) = replay_one(stem) {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "recorded-migration replay shard {shard} failed:\n{}",
        failures.join("\n")
    );
}

macro_rules! replay_shard {
    ($name:ident, $n:expr) => {
        #[test]
        fn $name() {
            replay_fixtures_shard($n);
        }
    };
}

replay_shard!(every_recorded_migration_replays_shard_0, 0);
replay_shard!(every_recorded_migration_replays_shard_1, 1);
replay_shard!(every_recorded_migration_replays_shard_2, 2);
replay_shard!(every_recorded_migration_replays_shard_3, 3);
replay_shard!(every_recorded_migration_replays_shard_4, 4);
replay_shard!(every_recorded_migration_replays_shard_5, 5);
replay_shard!(every_recorded_migration_replays_shard_6, 6);
replay_shard!(every_recorded_migration_replays_shard_7, 7);
replay_shard!(every_recorded_migration_replays_shard_8, 8);
replay_shard!(every_recorded_migration_replays_shard_9, 9);
replay_shard!(every_recorded_migration_replays_shard_10, 10);
replay_shard!(every_recorded_migration_replays_shard_11, 11);
replay_shard!(every_recorded_migration_replays_shard_12, 12);
replay_shard!(every_recorded_migration_replays_shard_13, 13);
replay_shard!(every_recorded_migration_replays_shard_14, 14);
replay_shard!(every_recorded_migration_replays_shard_15, 15);

#[test]
fn every_recorded_migration_replays_positional_ctor() {
    if let Some(e) = replay_one("positional-ctor-to-map") {
        panic!("{e}");
    }
}

#[test]
fn every_recorded_migration_is_fixtured_or_runed() {
    let stems = top_level_stems();
    let mut violations = Vec::new();

    let mut fixture_n = 0usize;
    let mut rune_n = 0usize;

    for stem in &stems {
        let fx = has_fixture(stem);
        let rune = rune_reason(stem);
        let rune_line = has_rune_line(stem);
        if rune_line && rune.is_none() {
            violations.push(format!(
                "{stem}: rune:replay(unreadable-preimage) has an empty reason"
            ));
        }
        let n = usize::from(fx) + usize::from(rune_line);
        if fx {
            fixture_n += 1;
        }
        if rune_line {
            rune_n += 1;
        }
        if n == 0 {
            violations.push(format!(
                "{stem}: no fixture, no rune:replay(unreadable-preimage)"
            ));
        } else if n > 1 {
            violations.push(format!(
                "{stem}: in more than one category (fixture={fx} rune={rune_line})"
            ));
        }
        let lines = scope_lines(stem);
        if lines.len() != 1 {
            violations.push(format!(
                "{stem}: expected exactly one `;; SCOPE:` line, found {}",
                lines.len()
            ));
        } else if lines[0].is_empty() {
            violations.push(format!("{stem}: SCOPE line is empty"));
        } else {
            let entries: Vec<String> = lines[0].split_whitespace().map(str::to_string).collect();
            if let Err(e) = scope_globs_hit_a_tracked_file(stem, &entries) {
                violations.push(e);
            }
        }
    }

    // Extra fixture dirs whose stem is not a top-level codemod.
    if replay_dir().is_dir() {
        for e in fs::read_dir(replay_dir()).unwrap() {
            // rune:lint(one-variant-separator, not-a-name) — DirEntry::path() is a filesystem path.
            let p = e.unwrap().path();
            if !p.is_dir() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if !stems.iter().any(|s| s == name) {
                violations.push(format!(
                    "{name}: replay/ dir has no matching wat-scripts/fixes/{name}.wat"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "recorded-migration coverage failed ({}):\n{}",
        violations.len(),
        violations.join("\n")
    );
    assert_eq!(
        stems.len(),
        fixture_n + rune_n,
        "stems={} fixtures={} runes={} (expected stems == fixtures + runes)",
        stems.len(),
        fixture_n,
        rune_n
    );
    assert!(fixture_n > 0, "no replay fixtures found — the gate is measuring nothing");
}
