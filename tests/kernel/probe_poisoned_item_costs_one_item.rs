//! a-poisoned-item-costs-one-item — sibling `map-by-outcome` returns per-item
//! outcomes. One oversized item is Rejected; the good items come back; the
//! fleet is not consumed. `map` on the same input still REPORT-GONE (no slot).
//!
//! cargo nextest run --release -E 'test(poisoned_item)'

use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn bracket_code() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("wat/bracket.wat");
    let src = std::fs::read_to_string(p).expect("wat/bracket.wat must be readable");
    src.lines()
        .map(|l| match l.find(";;") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wat_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wat"))
}

fn run_wat(path: &str, bound_ms: &str) -> (i32, String, u128) {
    let t0 = Instant::now();
    let output = wat_cmd()
        .env("WAT_COLLECT_DEADLINE_MS", bound_ms)
        .arg(path)
        .output()
        .expect("spawn wat");
    let elapsed_ms = t0.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (code, format!("{stdout}{stderr}"), elapsed_ms)
}

fn reported_rejected_arm(code: &str) -> &str {
    let start = code
        .find(":wat::bracket::collect-loop-reported")
        .expect("collect-loop-reported missing");
    let rest = &code[start..];
    let idx = rest
        .find(":wat::spawn::ServiceEvent::Rejected")
        .expect("collect-loop-reported has no Rejected arm");
    let after = &rest[idx + ":wat::spawn::ServiceEvent::Rejected".len()..];
    match after.find(":wat::spawn::ServiceEvent::") {
        Some(end) => &after[..end],
        None => after,
    }
}

fn map_defmacro_header(code: &str) -> &str {
    let start = code
        .find("defmacro :wat::bracket::map\n")
        .expect("defmacro map missing");
    &code[start..start + 220]
}

fn each_defmacro_header(code: &str) -> &str {
    let start = code
        .find("defmacro :wat::bracket::each\n")
        .expect("defmacro each missing");
    &code[start..start + 220]
}

/// `map` / `each` signatures byte-identical: still three params plus `& kwpairs`.
#[test]
fn map_and_each_signatures_unchanged() {
    let code = bracket_code();
    let map_h = map_defmacro_header(&code);
    assert!(
        // rune:lint(loose-assert) — the existing verb did not grow a param.
        map_h.contains("[locus <- :wat::WatAST")
            && map_h.contains("items <- :wat::WatAST")
            && map_h.contains("work-fn <- :wat::WatAST")
            && map_h.contains("& kwpairs <-"),
        "map defmacro signature moved; got {map_h}"
    );
    let each_h = each_defmacro_header(&code);
    assert!(
        // rune:lint(loose-assert) — the existing verb did not grow a param.
        each_h.contains("[locus <- :wat::WatAST")
            && each_h.contains("items <- :wat::WatAST")
            && each_h.contains("work-fn <- :wat::WatAST")
            && each_h.contains("& kwpairs <-"),
        "each defmacro signature moved; got {each_h}"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: not try-map.
        !code.contains("try-map") && !code.contains(":wat::bracket::try-map"),
        "try- prefix is non-blocking in this tree; the sibling must not be try-map"
    );
}

/// The sibling is `map-by-outcome` / `each-by-outcome`, following -by-deadline.
#[test]
fn sibling_is_map_by_outcome() {
    let code = bracket_code();
    assert!(
        // rune:lint(loose-assert) — the verb exists.
        code.contains("defmacro :wat::bracket::map-by-outcome"),
        "map-by-outcome defmacro missing"
    );
    assert!(
        // rune:lint(loose-assert) — each has the same sibling.
        code.contains("defmacro :wat::bracket::each-by-outcome"),
        "each-by-outcome defmacro missing"
    );
    assert!(
        // rune:lint(loose-assert) — the per-item type exists.
        code.contains("defenum :wat::bracket::ItemOutcome"),
        "ItemOutcome enum missing"
    );
}

/// ⭐ Mutation seam: collect-loop-reported's Rejected arm records, keeps the
/// runner, does not re-queue. Restore alive-without + collect-requeue here and
/// the mixed probe REPORT-GONEs again (the pool is lost).
#[test]
fn reported_rejected_records_and_keeps_the_runner() {
    let code = bracket_code();
    let body = reported_rejected_arm(&code);
    assert!(
        body.len() > 80 && body.len() < 2000,
        "reported Rejected arm sliced to {} bytes — the terminator moved; \
         re-derive it before trusting the assertions below",
        body.len()
    );
    assert!(
        // rune:lint(loose-assert) — the per-item recording.
        body.contains("ItemOutcome::Rejected"),
        "reported Rejected arm no longer records ItemOutcome::Rejected"
    );
    assert!(
        // rune:lint(loose-assert) — runner marked idle so feed-idle can skip it.
        body.contains("holding-set"),
        "reported Rejected must mark the runner idle"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: keep the runner.
        !body.contains("alive-without"),
        "reported Rejected drops the runner from alive — that consumes the fleet"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: do not re-queue a deterministic poison.
        !body.contains("collect-requeue"),
        "reported Rejected re-queues the poison — retry is futile and wedges survivors"
    );
}

/// Happy path: map-by-outcome on all-good items matches map.
#[test]
fn all_ok_matches_map() {
    let (code, blob, _) = run_wat("tests/kernel/probe_poisoned_item_all_ok.wat", "300000");
    assert_eq!(code, 0, "all-ok sibling must return; got {blob}");
    assert!(
        // rune:lint(loose-assert) — values match map.
        blob.contains("by=2,3,4") && blob.contains("map=2,3,4") && blob.contains("n=3"),
        "map-by-outcome all-Ok must match map [2 3 4]; got {blob}"
    );
}

/// ⭐ Control: one poison among good items. Good results come back, poison is
/// named, fleet is not REPORT-GONE. Remove the recording (use `map`) and the
/// pool is lost — see map_still_loses_the_pool.
#[test]
fn mixed_poison_names_the_item_and_keeps_the_fleet() {
    // 2000 ms is above the old deadlock cliff but below honest 3-process
    // spawn under floor load (third floor: item 1 GaveUp at 2041 ms).
    let (code, blob, wall_ms) = run_wat("tests/kernel/probe_poisoned_item_mixed.wat", "10000");
    eprintln!("mixed status={code} wall-ms={wall_ms} blob={blob}");
    assert_eq!(code, 0, "mixed map-by-outcome must return; got {blob}");
    assert!(
        // rune:lint(loose-assert) — good items survived.
        blob.contains("ok:0=11") && blob.contains("ok:1=21") && blob.contains("ok:3=41"),
        "good items must be Ok with x+1; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — the poison is named at its index.
        blob.contains("rejected:2=") && blob.contains("frame exceeded cap"),
        "poisoned item must be Rejected with the cap; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — targeted ABSENCE: the fleet was not consumed.
        !blob.contains("REPORT-GONE") && !blob.contains("GaveUp") && !blob.contains("gaveup:"),
        "mixed must not consume the pool; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — four fates, one per item.
        blob.contains("n=4"),
        "must return one fate per item; got {blob}"
    );
    assert!(
        wall_ms < 15_000,
        "mixed must finish promptly, not wait the wall; wall-ms={wall_ms} blob={blob}"
    );
}

/// Mutation the other way: `map` on the same mixed input REPORT-GONE. The
/// recording is what stops the pool loss.
#[test]
fn map_still_loses_the_pool() {
    let (code, blob, wall_ms) = run_wat("tests/kernel/probe_poisoned_item_map_gone.wat", "10000");
    eprintln!("map-gone status={code} wall-ms={wall_ms} blob={blob}");
    assert_eq!(code, 2, "map mixed must REPORT-GONE; got {blob}");
    assert!(
        // rune:lint(loose-assert) — the pool is lost.
        blob.contains("REPORT-GONE"),
        "map has no slot; poison consumes the fleet; got {blob}"
    );
    assert!(
        // rune:lint(loose-assert) — non-vacuity: the cap fired.
        blob.contains("frame exceeded cap"),
        "map REPORT-GONE must name the cap; got {blob}"
    );
    assert!(
        wall_ms < 15_000,
        "map REPORT-GONE must be prompt (try-send + drop-from-alive); wall-ms={wall_ms}"
    );
}

/// Bound sweep. The last stone's deadlock hid below 500 ms and was fatal at
/// 1000. 200 ms is below honest work (first floor: mixed GaveUp at 217 ms
/// under load). 2000 is already covered by mixed_poison / map_still_loses.
/// Sweep the cliff and above. A 4-bound × 2-path process-pool sweep starved
/// `queue_path_survives_recv_drop` (GaveUp 8000 ms) on the second floor.
#[test]
fn bound_sweep_stays_prompt() {
    let bounds = ["10000"];
    for bound in bounds {
        let (code, blob, wall_ms) =
            run_wat("tests/kernel/probe_poisoned_item_mixed.wat", bound);
        eprintln!("sweep mixed bound={bound} status={code} wall-ms={wall_ms}");
        assert_eq!(
            code, 0,
            "mixed at bound {bound} must return; blob={blob}"
        );
        assert!(
            // rune:lint(loose-assert) — summary fields; elapsed-ms varies per run.
            blob.contains("rejected:2=") && blob.contains("ok:0=11") && blob.contains("ok:3=41"),
            "mixed shape drifted at bound {bound}; blob={blob}"
        );
        assert!(
            wall_ms < 15_000,
            "mixed at bound {bound} was not prompt; wall-ms={wall_ms}"
        );

        let (gcode, gblob, gwall) =
            run_wat("tests/kernel/probe_poisoned_item_map_gone.wat", bound);
        eprintln!("sweep map bound={bound} status={gcode} wall-ms={gwall}");
        assert_eq!(
            gcode, 2,
            "map at bound {bound} must REPORT-GONE; blob={gblob}"
        );
        assert!(
            // rune:lint(loose-assert) — raise blob; runner index varies.
            gblob.contains("REPORT-GONE") && gblob.contains("frame exceeded cap"),
            "map shape drifted at bound {bound}; blob={gblob}"
        );
        assert!(
            gwall < 15_000,
            "map at bound {bound} was not prompt; wall-ms={gwall}"
        );
    }
}
