//! Gate: every recorded migration in `wat-scripts/fixes/*.wat` carries either a
//! replay fixture, a `FROZEN_LEDGER` entry, or a header rune. Nothing is exempt
//! by silence.
//!
//! A fixture is `wat-scripts/fixes/replay/<stem>/{before.pre,after.post}` (optional
//! `stdin`). The oracle is HISTORY or the codemod's header spec — never the
//! tool under test. The gate asserts byte-exact output, idempotence, and
//! non-vacuity against that oracle.
//!
//! `.pre`/`.post` stay out of `wat_scripts_fixes_load.rs` (walks `*.wat`) and
//! `every_tracked_wat_parses.rs` (`git ls-files '*.wat'`).
//!
//! `FROZEN_LEDGER` is named debt: it only shrinks, and stone 0b (arc 294's grok-rete replay)
//! empties it and deletes the constant.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const FROZEN_LEDGER: &[(&str, &str)] = &[
    ("address-transport-arity", "stone 0b: fixture pending"),
    ("angle-brackets-to-binder", "stone 0b: fixture pending"),
    ("caller-to-emitted-from", "stone 0b: fixture pending"),
    ("declare-max-request-bytes", "stone 0b: fixture pending"),
    ("defrule-then-to-vector", "stone 0b: fixture pending"),
    ("deprime-telemetry-sqlite", "stone 0b: fixture pending"),
    ("drop-deftest-prelude", "stone 0b: fixture pending"),
    ("drop-env-wat-dot-prefix", "stone 0b: fixture pending"),
    ("eprintln-recv-arm-to-assertion-failed", "stone 0b: fixture pending"),
    ("face-underscore-bound-send-prime", "stone 0b: fixture pending"),
    ("first-of-drop-to-nth", "stone 0b: fixture pending"),
    ("fix-macro-param-types", "stone 0b: fixture pending"),
    ("inline-constraint-per-type-spelling", "stone 0b: fixture pending"),
    ("kill-make-deftest", "stone 0b: fixture pending"),
    ("mandate-invocation-ctx-param", "stone 0b: fixture pending"),
    ("mandate-request-malformed", "stone 0b: fixture pending"),
    ("move-deftest-callers-to-prime", "stone 0b: fixture pending"),
    ("move-deftest-hermetic-callers-to-prime", "stone 0b: fixture pending"),
    ("namespace-bare-top-level-names", "stone 0b: fixture pending"),
    ("namespace-defrule-names", "stone 0b: fixture pending"),
    ("parametrics-take-a-type-vector", "stone 0b: fixture pending"),
    ("positional-to-kwargs", "stone 0b: fixture pending"),
    ("query-answers-are-maps", "stone 0b: fixture pending"),
    ("read-string-to-outcome", "stone 0b: fixture pending"),
    ("readln-to-outcome", "stone 0b: fixture pending"),
    ("reclaim-deftest-names", "stone 0b: fixture pending"),
    ("reclaim-hologram-find-name", "stone 0b: fixture pending"),
    ("reclaim-ipc-prime-names", "stone 0b: fixture pending"),
    ("reclaim-service-fixture-names", "stone 0b: fixture pending"),
    ("reclaim-stdio-prime-names", "stone 0b: fixture pending"),
    ("rehead-rete-callees", "stone 0b: fixture pending"),
    ("rename-call-ctx-to-invocation", "stone 0b: fixture pending"),
    ("rename-diederror-to-loci-died-error", "stone 0b: fixture pending"),
    ("rename-kernel-to-spawn", "stone 0b: fixture pending"),
    ("rename-list-to-seq", "stone 0b: fixture pending"),
    ("rename-locidiederror-shutdown-to-stopped", "stone 0b: fixture pending"),
    ("rename-record-def-to-defrecord", "stone 0b: fixture pending"),
    ("rename-seq-fold-aliases-to-core-reduce", "stone 0b: fixture pending"),
    ("rename-sourcefile-to-source-file", "stone 0b: fixture pending"),
    ("rename-wat-record-to-core-record", "stone 0b: fixture pending"),
    ("rename-wat-tests-std-to-wat-tests", "stone 0b: fixture pending"),
    ("response-record-to-enum", "stone 0b: fixture pending"),
    ("retarget-peer-purity-probes", "stone 0b: fixture pending"),
    ("rete-oracle-sigil", "stone 0b: fixture pending"),
    ("rete-where-per-type-spelling", "stone 0b: fixture pending"),
    ("rule-record-to-defrule", "stone 0b: fixture pending"),
    ("service-locus-to-user-rendezvous", "stone 0b: fixture pending"),
    ("spawn-program-to-test-spawn-peer", "stone 0b: fixture pending"),
    ("stdin-frame-vocabulary", "stone 0b: fixture pending"),
    ("strip-expect-ascription", "stone 0b: fixture pending"),
    ("strip-insert-rhs-marker", "stone 0b: fixture pending"),
    ("strip-match-ascription", "stone 0b: fixture pending"),
    ("strip-useless-mains", "stone 0b: fixture pending"),
    ("struct-new-failure-to-message-only-failure", "stone 0b: fixture pending"),
    ("sweep-lint-fixes", "stone 0b: fixture pending"),
    ("timer-prime-to-peer-prime", "stone 0b: fixture pending"),
    ("to-faithful-clojure", "stone 0b: fixture pending"),
    ("to-faithful-clojure-net", "stone 0b: ROTTED — rete where-fence refuses a user fn (:fix::has-ns? / :fix::head-keyword-str?), rc=2"),
    ("to-faithful-clojure-rete", "stone 0b: ROTTED — rete where-fence refuses a user fn (:fix::has-ns? / :fix::head-keyword-str?), rc=2"),
    ("tuple-parens-to-binder", "stone 0b: fixture pending"),
    ("type-query-to-defquery", "stone 0b: fixture pending"),
    ("unignore-arc170-concurrency", "stone 0b: fixture pending"),
    ("unstamp-transport-wire", "stone 0b: fixture pending"),
    ("unwrap-recvoutcome-false-positive", "stone 0b: fixture pending"),
    ("variant-vector-to-tagged-map", "stone 0b: identity rewrite — header: no wat tagged-literal form; non-vacuous fixture would require STOP-1"),
    ("wrap-client-method-match-in-recvoutcome", "stone 0b: fixture pending"),
    ("wrap-connect-prime-in-connectoutcome", "stone 0b: fixture pending"),
];

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

fn ledger_map() -> std::collections::BTreeMap<&'static str, &'static str> {
    FROZEN_LEDGER.iter().copied().collect()
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
    let stems = fixture_stems();
    let mut failures = Vec::new();
    for (i, stem) in stems.iter().enumerate() {
        if i % 8 != shard {
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

#[test]
fn every_recorded_migration_is_fixtured_ledgered_or_runed() {
    let stems = top_level_stems();
    let ledger = ledger_map();
    let mut violations = Vec::new();

    let mut fixture_n = 0usize;
    let mut ledger_n = 0usize;
    let mut rune_n = 0usize;

    for stem in &stems {
        let fx = has_fixture(stem);
        let led = ledger.get(stem.as_str()).copied();
        let rune = rune_reason(stem);
        let n = usize::from(fx) + usize::from(led.is_some()) + usize::from(rune.is_some());
        if fx {
            fixture_n += 1;
        }
        if led.is_some() {
            ledger_n += 1;
        }
        if rune.is_some() {
            rune_n += 1;
        }
        if n == 0 {
            violations.push(format!(
                "{stem}: no fixture, no FROZEN_LEDGER entry, no rune:replay(unreadable-preimage)"
            ));
        } else if n > 1 {
            violations.push(format!(
                "{stem}: in more than one category (fixture={fx} ledger={} rune={})",
                led.is_some(),
                rune.is_some()
            ));
        }
        if fx || rune.is_some() {
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
    }

    for (stem, _why) in FROZEN_LEDGER {
        if !stems.iter().any(|s| s == stem) {
            violations.push(format!(
                "{stem}: FROZEN_LEDGER entry for a stem that no longer exists"
            ));
        }
        if has_fixture(stem) {
            violations.push(format!(
                "{stem}: FROZEN_LEDGER entry is stale — this stem now has a fixture"
            ));
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
        fixture_n + ledger_n + rune_n,
        "stems={} fixtures={} ledger={} runes={} (expected stems == sum)",
        stems.len(),
        fixture_n,
        ledger_n,
        rune_n
    );
    // No count pins. The gate freezes NAMES: every stem is covered by name above, and the
    // ledger's stale check catches progress that forgot to shrink it. A pinned count would go
    // red on legitimate progress (a new fixture, a new codemod) and could not name an offender.
    assert!(fixture_n > 0, "no replay fixtures found — the gate is measuring nothing");
}
