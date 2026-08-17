//! Liveness gate — the 18 `wat-scripts/perf/grid/*.wat` rete axes must actually RUN, and RUN
//! NON-VACUOUSLY, on the current runtime.
//!
//! `every_wat_scripts_file_loads_on_the_current_runtime` (tests/lint/wat_scripts_fixes_load.rs)
//! parses + type-checks every `.wat` under `wat-scripts/` but never RUNS one. The rete fence
//! (`wat/rete.wat`'s "expr is not total" arm) is a RUNTIME check that fires when a rule
//! *compiles* — i.e. only when the program actually runs — so on 2026-08-06 four grid axes
//! (min-finding, node-share, strat-neg, user-reduce) were found DEAD, having died the hour that
//! fence armed, with every existing gate green for days. This test closes that blind spot by
//! driving each axis through a REAL `wat` subprocess (the `tests/process/
//! wat_arc170_closure6_label_wall.rs` pattern — `env!("CARGO_BIN_EXE_wat")`, never a hardcoded
//! `target/release/wat`, so there is no "did someone `cargo build` first" dependency) and
//! checking it derives something.
//!
//! ## Two populations, two shapes (discovered, never listed)
//!
//! 9 axes (accum, asym-join, deep-cascade, fanout, min-finding, negation, node-share, strat-neg,
//! user-reduce) are the `run-axis.sh` contract: stdin is an i64 size vector, stdout is ONE
//! `#grid/Result {... :derived #wat.core/PersistentVector [...] ...}` line. Those are asserted
//! here per axis: (1) exit 0, (2) a `#grid/Result` line present, (3) `:derived` is NOT `[]`.
//!
//! The other 10 are the `where-*.wat` expressivity corpus (`check-where-shapes.sh`'s population
//! vs Clara; `check-spec-native.sh` / `spec_equals_native_on_every_where_family` vs the oracle):
//! no stdin, and stdout is N `row ... n=... ->...` lines, not a `#grid/Result`. They get their
//! own assertion shape (run + produce output) rather than being silently skipped — and the
//! EXEMPT SET itself is asserted exactly equal to the known 9 names, so a 10th `where-*.wat`
//! cannot fall into the exemption by accident; it will fail this test until someone deliberately
//! updates `WHERE_FAMILY` below.
//!
//! ## Non-vacuity is the hard part
//!
//! A run that derives `[]` proves nothing (e.g. min-finding at [4 100] correctly derives nothing
//! — see that axis's own header). `SIZED_AXES` below pins ONE small size per axis, chosen (and
//! justified inline) to GUARANTEE a non-empty `:derived`, grounded by hand against each axis's
//! own doc-comment usage example and shape description. All nine were run by hand at these sizes
//! before this file was written; every one derives and completes in well under a second.
//!
//! ## Discovery, not a list
//!
//! The directory is walked, not enumerated by name: `entries.len() >= 18` is floor-asserted so
//! neither an empty glob nor a shrunk corpus can pass vacuously, and every SIZED axis discovered
//! on disk must have a `SIZED_AXES` entry (a 19th sized axis with no assigned size fails loudly,
//! naming itself, rather than silently not being run).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// One entry per non-`where-*` grid axis: (file stem, stdin size vector, why that size must
/// derive something non-empty). Justifications are grounded against each axis's own `.wat`
/// header (`wat-scripts/perf/grid/<stem>.wat`), not invented here.
const SIZED_AXES: &[(&str, &[i64], &str)] = &[
    (
        "accum",
        &[2, 3],
        "size=[groups readings]; W=3>=1 always makes min/max Some, so every group emits all \
         five derived facts (count/sum/min/max/exists) per the axis's own header — 2 groups \
         is already non-empty.",
    ),
    (
        "asym-join",
        &[5],
        "size=[items]; every inserted A(k) derives B(k) unconditionally (R1), then the \
         asymmetric join derives C(k) for every k once caught up (P6) — non-empty for any \
         items>=1.",
    ),
    (
        "deep-cascade",
        &[2, 3],
        "size=[depth width]; every seeded id survives every level by construction (\"the \
         joins never drop anyone\"), so depth=2 width=3 derives 2*depth*width=12 facts.",
    ),
    (
        "fanout",
        &[400],
        "size=[items], items = keys*F^2 with F fixed at 20; items=400 -> keys=1, deriving \
         F^2=400 Pair facts (the minimal non-zero key count).",
    ),
    (
        "min-finding",
        &[6, 2],
        "size=[stations threshold]; readings per station = loc mod (2*threshold), so \
         stations>threshold guarantees some loc lands in [threshold, 2*threshold) and \
         activates — 6 stations at threshold 2 activates loc 2 and 3 (counts 2,3). \
         (The worked warning in the brief: [4 100] derives [] correctly, because no loc can \
         reach 100 — this size was chosen specifically to clear the threshold instead.)",
    ),
    (
        "negation",
        &[4],
        "size=[items]; Bad seeded for even k, Ok fires for odd k — items=4 leaves the odd keys \
         {1,3} non-empty.",
    ),
    (
        "neg-consumer",
        &[4],
        "size=[items]; the THREE-WAY axis — Ok(k) :- Item(k), NOT Bad(k) is the negation gate \
         and Final(k) :- Ok(k), Tag(k) is the POSITIVE consumer downstream of it (task #94). \
         items=4 leaves the odd keys {1,3} non-empty. Emits :oracle-derived as well, so the \
         verdict carries :oracle-accuracy (spec vs Clara) and :port-accuracy (spec vs native).",
    ),
    (
        "node-share",
        &[2, 4],
        "size=[rules items]; every k in [0,items) satisfies EXACTLY one of the N rules \
         (i == k mod N) by construction, so the derived Out set is items-many regardless of \
         rules — {0,1,2,3} for items=4.",
    ),
    (
        "strat-neg",
        &[2, 4],
        "size=[strata items]; S0(k) marks even k, S1(k) marks NOT-S0(k) i.e. odd k — strata=2 \
         items=4 derives S0={0,2} and S1={1,3}, both non-empty.",
    ),
    (
        "user-reduce",
        &[2, 3],
        "size=[locs reads]; every location gets `reads` readings and sum-of-squares over a \
         non-empty PV is always emitted as one Agg fact per location — 2 locs is non-empty.",
    ),
];

/// The known `where-*.wat` expressivity-corpus stems. Asserted EXACTLY (not merely `<=` or
/// `>=`) against what's discovered on disk, so a new family added to the corpus cannot land in
/// this exemption silently — it will fail the exact-set assertion below and force a deliberate
/// update here.
const WHERE_FAMILY: &[&str] = &[
    "where-accum-lead",
    "where-accum-where",
    "where-boolean",
    "where-collection",
    "where-control",
    "where-exists",
    "where-join-order",
    "where-multivar",
    "where-nesting",
    "where-not-and",
    "where-not-bound",
    "where-not-fact",
    "where-not-not",
    "where-not-or",
    "where-not-where",
    "where-not-windy",
    "where-numeric",
    "where-or-and",
    "where-or-conditions",
    "where-record",
    "where-shapes",
    "where-string",
    "where-test-chain",
];

fn grid_dir() -> std::path::PathBuf {
    Path::new("wat-scripts/perf/grid").to_path_buf()
}

/// Discover every `.wat` directly under `wat-scripts/perf/grid/` (flat, not recursive — the
/// grid has no subdirectories of axes). Returns file stems, sorted.
fn discover_axis_stems() -> Vec<String> {
    let dir = grid_dir();
    let mut stems: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "wat"))
        .map(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .expect("utf8 stem")
                .to_string()
        })
        .collect();
    stems.sort();
    stems
}

/// Run a sized axis: pipe `size` as an EDN i64 vector on stdin, return (success, stdout, stderr).
fn run_sized_axis(stem: &str, size: &[i64]) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = grid_dir().join(format!("{stem}.wat"));
    // Built with `push`, not `format!("[{}]", …)`, and the reason is a lint — `no_inlined_edn`
    // flags any string literal whose trimmed content opens with `[`, and a bare `"[{}]"` scaffold
    // is indistinguishable from a complete inlined EDN vector to its detector. The rune is
    // reserved for genuine EDN that cannot be a file; this is neither EDN-under-comparison nor a
    // golden — it is an INPUT we construct — so the lint's own remedy applies: restructure so the
    // literal does not open that way. Delimiters as `char`s do exactly that, and read fine.
    let mut size_edn = String::new();
    size_edn.push('[');
    size_edn.push_str(
        &size
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    size_edn.push(']');
    let size_json = size_edn;

    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));

    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(format!("{size_json}\n").as_bytes())
        .unwrap_or_else(|e| panic!("write size {size_json} to {stem}: {e}"));

    let output = child.wait_with_output().expect("wait for child");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Run a `where-*` axis: no stdin required (it seeds from an internal formula, per its own
/// header). Return (success, stdout, stderr).
fn run_where_axis(stem: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = grid_dir().join(format!("{stem}.wat"));
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Extract the contents of the `:derived #wat.core/PersistentVector [...]` bracket from a
/// `#grid/Result` line. `:derived` elements are plain i64s (no nested brackets), so the first
/// `]` after the opening `[` is always the close. Returns `None` if the shape isn't found at all
/// (a missing `#grid/Result` line, handled separately by the caller).
fn extract_derived(stdout: &str) -> Option<String> {
    let key = ":derived";
    let key_pos = stdout.find(key)?;
    let after_key = &stdout[key_pos + key.len()..];
    let open = after_key.find('[')?;
    let close = after_key[open..].find(']')?;
    Some(after_key[open + 1..open + close].to_string())
}

#[test]
fn grid_axes_run_and_derive_nonvacuously() {
    let stems = discover_axis_stems();
    assert!(
        stems.len() >= 18,
        "found only {} .wat files under wat-scripts/perf/grid/ (expected >= 18: 9 sized axes + \
         10 where-* expressivity files) — the gate is measuring less than the known grid, or the \
         directory moved: {stems:?}",
        stems.len()
    );

    let mut where_stems: Vec<&str> = stems
        .iter()
        .filter(|s| s.starts_with("where-"))
        .map(String::as_str)
        .collect();
    where_stems.sort();
    let mut expected_where = WHERE_FAMILY.to_vec();
    expected_where.sort();
    assert_eq!(
        where_stems, expected_where,
        "the where-* expressivity family on disk does not match the exempt set this gate knows \
         about — a new where-*.wat (or a removed one) must be added to WHERE_FAMILY in \
         tests/rete/wat_scripts_grid_axes_live.rs deliberately; it cannot silently fall into (or \
         out of) the run-only exemption"
    );

    let sized_stems: Vec<&str> = stems
        .iter()
        .filter(|s| !s.starts_with("where-"))
        .map(String::as_str)
        .collect();

    let mut failures: Vec<String> = Vec::new();

    // ── the 9 sized axes: must RUN and DERIVE non-vacuously ────────────────────────────────
    for &stem in &sized_stems {
        let Some(&(_, size, why)) = SIZED_AXES.iter().find(|(name, _, _)| *name == stem) else {
            failures.push(format!(
                "  {stem}: discovered on disk under wat-scripts/perf/grid/ but has NO entry in \
                 SIZED_AXES (tests/rete/wat_scripts_grid_axes_live.rs) — a new sized axis must be \
                 given a non-vacuous size deliberately, it cannot run unassigned"
            ));
            continue;
        };

        let (ok, stdout, stderr) = run_sized_axis(stem, size);
        if !ok {
            failures.push(format!(
                "  {stem} (size {size:?}, justification: {why}): process did NOT exit \
                 successfully.\n      stdout: {stdout:?}\n      stderr: {stderr:?}"
            ));
            continue;
        }
        if !stdout.contains("#grid/Result") {
            failures.push(format!(
                "  {stem} (size {size:?}): exited 0 but stdout carries NO #grid/Result line — \
                 the axis produced nothing.\n      stdout: {stdout:?}\n      stderr: {stderr:?}"
            ));
            continue;
        }
        match extract_derived(&stdout) {
            None => failures.push(format!(
                "  {stem} (size {size:?}): #grid/Result present but no :derived \
                 #wat.core/PersistentVector [...] shape found in it.\n      stdout: {stdout:?}"
            )),
            Some(derived) if derived.trim().is_empty() => failures.push(format!(
                "  {stem} (size {size:?}, justification: {why}): DIED — ran and exited 0, but \
                 :derived is EMPTY ([]). Full line: {stdout:?}\n      stderr: {stderr:?}"
            )),
            Some(_) => {} // non-empty — this axis is alive.
        }
    }

    // ── the where-* axes: must RUN and produce output rows (different shape, same intent) ──
    for &stem in &where_stems {
        let (ok, stdout, stderr) = run_where_axis(stem);
        if !ok {
            failures.push(format!(
                "  {stem}: process did NOT exit successfully.\n      stdout: {stdout:?}\n      \
                 stderr: {stderr:?}"
            ));
            continue;
        }
        let row_count = stdout.lines().filter(|l| !l.trim().is_empty()).count();
        if row_count == 0 {
            failures.push(format!(
                "  {stem}: exited 0 but produced ZERO output rows — the expressivity corpus did \
                 not run any shape.\n      stderr: {stderr:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} grid axes are DEAD (did not run, or ran vacuously):\n{}",
        failures.len(),
        sized_stems.len() + where_stems.len(),
        failures.join("\n")
    );
}

/// Rewrite the public production verb to the oracle. Does not touch an
/// already-spec call (`fire-rules-spec`).
fn rewrite_fire_to_spec(src: &str) -> String {
    let needle = ":wat::rete::fire-rules";
    let mut out = String::with_capacity(src.len() + 64);
    let mut rest = src;
    while let Some(i) = rest.find(needle) {
        out.push_str(&rest[..i]);
        let after = &rest[i + needle.len()..];
        if after.starts_with("-spec") {
            out.push_str(needle);
            rest = after;
        } else {
            out.push_str(":wat::rete::fire-rules-spec");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn run_wat_path(path: &Path) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// THE oracle/native gate. `check-where-shapes.sh` is Clara vs native. This
/// test is spec vs native on the SAME where-* rows. A split here is a rete
/// defect, not a purity cut — both sides are the value session.
#[test]
fn spec_equals_native_on_every_where_family() {
    let tmp = std::env::temp_dir().join(format!(
        "wat-spec-native-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).expect("temp dir for spec rewrite");

    let mut failures: Vec<String> = Vec::new();
    let mut rows_total = 0usize;

    for stem in WHERE_FAMILY {
        let native_path = grid_dir().join(format!("{stem}.wat"));
        let src = std::fs::read_to_string(&native_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", native_path.display()));
        let spec_src = rewrite_fire_to_spec(&src);
        let spec_calls = spec_src.match_indices(":wat::rete::fire-rules-spec").count();
        let doubled = spec_src.match_indices(":wat::rete::fire-rules-spec-spec").count();
        assert_ne!(
            spec_calls, 0,
            "{stem}: rewrite produced no fire-rules-spec call — the family never fires?"
        );
        assert_eq!(
            doubled, 0,
            "{stem}: rewrite double-applied fire-rules-spec"
        );
        let spec_path = tmp.join(format!("{stem}.spec.wat"));
        std::fs::write(&spec_path, spec_src).expect("write spec rewrite");

        let (n_ok, n_out, n_err) = run_wat_path(&native_path);
        let (s_ok, s_out, s_err) = run_wat_path(&spec_path);
        if !n_ok {
            failures.push(format!("{stem}: native FAILED\n  stderr: {n_err}"));
            continue;
        }
        if !s_ok {
            failures.push(format!("{stem}: spec FAILED\n  stderr: {s_err}"));
            continue;
        }
        let n_rows = n_out.lines().filter(|l| !l.trim().is_empty()).count();
        let s_rows = s_out.lines().filter(|l| !l.trim().is_empty()).count();
        if n_rows == 0 {
            failures.push(format!("{stem}: native emitted no rows"));
            continue;
        }
        if n_out != s_out {
            failures.push(format!(
                "{stem}: spec != native (native {n_rows} rows, spec {s_rows} rows)\n\
                 --- spec\n{s_out}+++ native\n{n_out}"
            ));
            continue;
        }
        rows_total += n_rows;
    }

    let _ = std::fs::remove_dir_all(&tmp);
    assert!(
        failures.is_empty(),
        "spec != native on {} family(ies) ({} rows agreed before first miss):\n{}",
        failures.len(),
        rows_total,
        failures.join("\n")
    );
}
