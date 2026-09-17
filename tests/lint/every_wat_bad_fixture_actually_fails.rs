//! Gate — `.wat.bad` IS A CLAIM, AND THIS IS WHAT MAKES IT ONE.
//!
//! Every `*.wat.bad` in the tree is passed to [`startup_from_file`] — the driver the test suite
//! actually uses — and must return `Err`. A file that starts up CLEAN while wearing `.wat.bad` is
//! a lie in the filename, and until this gate existed nothing in the repo read the corpus for that
//! property at all.
//!
//! ## Why (C18, arc 278)
//!
//! The builder's definition: *"we use `.wat.bad` for tests that ensure files fail to parse,
//! correctly."* Nothing enforced it. Driven at `29b207e6e`, 16 of 281 `.wat.bad` files returned
//! `Ok` — they start up perfectly. Their TESTS were right in every case; the FILENAMES were wrong.
//! Thirteen were renamed to `.wat` by the strike that added this gate, in three shapes:
//!
//! - a premise was **retired** and the test correctly flipped to `is_ok()` (arc 300 C4/C5 adopted
//!   mixed-numeric coercion, retiring 237.8a's reject — seven files);
//! - the test starts the world up and then **INVOKES**, asserting the error arrives at EVAL
//!   (four files) — the builder's own pattern, and a valid program is not "bad";
//! - the test asserts, through a helper's sentinel, that startup **succeeded** (two files).
//!
//! ## ⛔ THE DRIVER IS THE WHOLE QUESTION — DO NOT MEASURE THIS WITH THE BINARY
//!
//! `./target/release/wat <file>` and `startup_from_file` give OPPOSITE verdicts on these same
//! files, and the first draft of this strike was withdrawn for using the wrong one. The BINARY
//! demands a `:user::main` because it EVALS one; `startup_from_file` does not — `src/freeze.rs`
//! guards that check on a main being *declared*, so `startup_bare()` (no main) passes cleanly.
//! 577 test files use `startup_from_file`/`startup_beside`; 8 shell out to the binary. Measured
//! through the binary, 17 fixtures "fail with `MainSignatureError`"; measured through the driver
//! the tests use, that is 2 — and both of those are `wat_arc170_slice_1e_user_main_nil_*`, whose
//! SUBJECT is the main signature. They fail for their own declared reason and are left alone.
//!
//! **A fixture never needs a `:user::main` added to satisfy this gate.** If that is ever the
//! proposed cure, the gate is being read through the binary again.
//!
//! ## The one exemption, and why it is VERIFIED rather than merely declared
//!
//! Three of the 16 are NOT mis-named. Their tests assert `is_err()` — and are `#[ignore]`d, with
//! the ignore reason saying so outright (*"RED-at-HEAD: … not yet built; unlock when we circle
//! back to arc 255"*). For those, `.wat.bad` is an **aspiration**: the file SHOULD be rejected,
//! the substrate is lenient, and a banked test records the gap honestly. Renaming them to `.wat`
//! would erase a tracked known-gap marker and assert the substrate is right when it is not.
//!
//! They carry, in their own header:
//!
//! ```text
//! ;; rune:lint(bad-is-banked) — <why the substrate SHOULD reject this> banked-by: <test fn name>
//! ```
//!
//! ⚠ **A declaration this gate could not check would rot exactly like the convention it replaces.**
//! So this one is checked, both halves: the named test must EXIST under `tests/`, and it must
//! still be `#[ignore]`d. That makes the exemption **self-clearing** — the day arc 255 lands and
//! the test is un-ignored, this gate goes RED and demands the file be dealt with, instead of the
//! rune sitting there forever vouching for a gap that closed. A rune whose owner passes is not an
//! exemption, it is a stale note.
//!
//! What this gate still cannot do is judge whether the REASON is true — no string check can, and
//! claiming otherwise would be the decoration this repo keeps removing. It closes the category
//! set, demands a sentence, and pins the sentence to a test whose state it re-reads every run.
//!
//! Shape and precedent: `tests/lint/docs_wat_loads_or_declares_why_not.rs` — the same
//! walk-and-start question asked in the opposite direction (there: every `.wat` must LOAD or
//! declare why not; here: every `.wat.bad` must FAIL or declare why not).
//!
//! ## Why this is SHARDED
//!
//! Starting up 281 files costs ~0.14s each in-process — ~39s sequentially, which blows nextest's
//! default 30s kill, and the loaded cost is 3.5-4.4x that at this repo's own recorded contention
//! band. `.config/nextest.toml` answers this case by name and refuses the easy road: *"⛔ IF THIS
//! EVER NEEDS RAISING, SPLIT INSTEAD."* So this gate splits into [`N_SHARDS`] tests rather than
//! asking for a budget, and it needs no override in that file at all. A failure also names WHICH
//! shard instead of arriving as one undifferentiated red.

use std::path::{Path, PathBuf};
use wat::freeze::startup_from_file;

/// Where a `.wat.bad` may live. All 281 sit under `tests/` today; the other two roots are walked
/// so a fixture dropped into either is caught on arrival rather than after it has rotted. They are
/// a const so the empty-population mutation is a one-line change a reviewer can re-run.
const ROOTS: &[&str] = &["tests", "wat-scripts", "docs"];

/// The CLOSED set of declarations. A second member is a deliberate act needing its own
/// discriminating question against this one, written where a reader will meet it.
const DECLARED_CATEGORIES: &[&str] = &["bad-is-banked"];

/// The field that pins a declaration to the test that owns the gap.
const OWNER_FIELD: &str = "banked-by:";

/// A reason shorter than this cannot say what the substrate should do instead.
const MIN_REASON_CHARS: usize = 24;

/// How many parallel shards the corpus is split across. Sixteen is the sibling C19 gate's number
/// over this same corpus, and it lands each shard at ~2.5s alone / ~11s loaded — inside the
/// DEFAULT 15s-warn / 30s-kill budget, which is why this gate needs no `nextest.toml` override.
const N_SHARDS: usize = 16;

/// The floor the walk must clear. Measured 2026-09-03: 268 `.wat.bad` in the tree (281 before this
/// gate's own strike renamed 13 mis-named fixtures away). Deliberately
/// well under it — this catches a walk gone blind (a moved root, a renamed extension), and must
/// not red when a strike legitimately renames a handful of mis-named fixtures away.
const CORPUS_FLOOR: usize = 200;

fn collect_bad(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_bad(&p, out);
        } else if p.to_str().is_some_and(|s| s.ends_with(".wat.bad")) {
            out.push(p);
        }
    }
}

fn corpus() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for root in ROOTS {
        collect_bad(Path::new(root), &mut paths);
    }
    paths.sort();
    paths
}

/// The declaration on one line, if any: `(category, reason)`.
///
/// Split on the fixed needle rather than regex, exactly as the sibling gate does. An UNCLOSED
/// `rune:lint(` yields the rest of the line as the category, so a malformed rune is REPORTED
/// rather than read as "no declaration at all" — the failure mode that would let a typo silently
/// become a permanent exemption.
fn declaration_on(line: &str) -> Option<(&str, &str)> {
    let at = line.find("rune:lint(")?;
    let after = &line[at + "rune:lint(".len()..];
    match after.find(')') {
        Some(close) => Some((
            &after[..close],
            after[close + 1..].trim_start_matches(['\u{2014}', '-', ' ']).trim(),
        )),
        None => Some((after, "")),
    }
}

/// The test-function name a declaration names as its owner, if it names one.
fn owner_in(reason: &str) -> Option<&str> {
    let at = reason.find(OWNER_FIELD)?;
    reason[at + OWNER_FIELD.len()..]
        .split_whitespace()
        .next()
        .filter(|n| !n.is_empty())
}

/// Every `.rs` under `tests/`, read once — the haystack the owner claim is checked against.
fn test_sources() -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                if let Ok(s) = std::fs::read_to_string(&p) {
                    out.push(s);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(Path::new("tests"), &mut out);
    out
}

/// What the owning test looks like right now.
#[derive(Debug, PartialEq, Eq)]
enum Owner {
    /// The test exists and is still `#[ignore]`d — the gap it banks is still open.
    Ignored,
    /// The test exists and RUNS — so the gap has closed and the exemption is stale.
    Live,
    /// No such test function anywhere under `tests/`.
    Absent,
}

/// Read the owning test's state out of one `.rs` source, if it is defined there.
///
/// `#[ignore]` must sit within the attribute block immediately above the `fn` — scanning upward
/// past a blank line or another item would let an unrelated ignored test above vouch for a live
/// one below.
fn owner_state_in(src: &str, name: &str) -> Option<Owner> {
    let needle = format!("fn {name}(");
    let lines: Vec<&str> = src.lines().collect();
    let i = lines.iter().position(|l| l.contains(&needle))?;
    let mut j = i;
    while j > 0 {
        let t = lines[j - 1].trim();
        if t.starts_with("#[") {
            if t.contains("ignore") {
                return Some(Owner::Ignored);
            }
            j -= 1;
            continue;
        }
        break;
    }
    Some(Owner::Live)
}

fn owner_state(name: &str, sources: &[String]) -> Owner {
    sources
        .iter()
        .find_map(|s| owner_state_in(s, name))
        .unwrap_or(Owner::Absent)
}

fn check_shard(shard: usize) {
    let paths = corpus();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS —
    // the defect this arc keeps re-finding. A moved root or a renamed extension must RED here, not
    // sail through green. Measured 2026-09-03: 268 files; the floor sits well under that so an
    // honest rename of a few mis-named fixtures does not trip it.
    assert!(
        paths.len() >= CORPUS_FLOOR,
        "the .wat.bad walk found only {} file(s) under {ROOTS:?} — under the floor of \
         {CORPUS_FLOOR}, so it is not reaching the corpus it claims to guard and a green verdict \
         below would mean nothing",
        paths.len()
    );

    let mut mine: Vec<&PathBuf> = paths.iter().skip(shard).step_by(N_SHARDS).collect();
    mine.sort();
    assert!(
        !mine.is_empty(),
        "shard {shard}/{N_SHARDS} covers no files — the sharding arithmetic is wrong and this \
         test's green is vacuous"
    );

    // Read lazily: only a fixture that starts up CLEAN needs its declaration checked, and that is
    // the rare case. Building this eagerly in all 16 shards would read every .rs under tests/
    // sixteen times for nothing.
    let mut sources: Option<Vec<String>> = None;
    let mut failures = Vec::new();
    let mut declared = 0usize;

    for path in mine {
        let rel = path.to_str().expect("utf8 path");
        if startup_from_file(rel).is_err() {
            continue;
        }

        // The file starts up CLEAN while claiming to be bad. Either the name lies, or a banked
        // test owns the gap and says so.
        let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let Some((cat, reason)) = src.lines().find_map(declaration_on) else {
            failures.push(format!(
                "  {rel}\n      starts up CLEAN (startup_from_file returned Ok) but is named \
                 `.wat.bad`, and declares nothing"
            ));
            continue;
        };
        declared += 1;

        if !DECLARED_CATEGORIES.contains(&cat) {
            failures.push(format!(
                "  {rel}\n      rune:lint({cat}) is not one of {DECLARED_CATEGORIES:?} — a second \
                 category needs its own discriminating question against that one, added to this \
                 gate deliberately"
            ));
            continue;
        }
        if reason.chars().count() < MIN_REASON_CHARS {
            failures.push(format!(
                "  {rel}\n      rune:lint({cat}) carries no reason ({} chars). It must say what \
                 the substrate SHOULD do with this file instead of accepting it",
                reason.chars().count()
            ));
            continue;
        }
        let Some(owner) = owner_in(reason) else {
            failures.push(format!(
                "  {rel}\n      rune:lint({cat}) names no owner. Append `{OWNER_FIELD} \
                 <test fn name>` — the ignored test that banks this gap is what makes the \
                 exemption checkable and self-clearing"
            ));
            continue;
        };
        let sources = sources.get_or_insert_with(test_sources);
        assert!(
            !sources.is_empty(),
            "no .rs sources found under tests/ — the owner check cannot run, so {rel}'s \
             declaration would be waved through unverified"
        );
        match owner_state(owner, sources) {
            Owner::Ignored => {}
            Owner::Absent => failures.push(format!(
                "  {rel}\n      rune:lint({cat}) names `{owner}` as its owner, but no `fn {owner}` \
                 exists under tests/. The exemption points at nothing"
            )),
            Owner::Live => failures.push(format!(
                "  {rel}\n      rune:lint({cat}) names `{owner}`, which is NO LONGER #[ignore]d — \
                 the gap this fixture banked has CLOSED. The exemption is stale: either the file \
                 now fails (drop the rune) or it does not and the name is wrong (rename it .wat)"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "\n\n🔥 {} `.wat.bad` file(s) in shard {shard}/{N_SHARDS} START UP CLEAN and do not \
         declare why. `.wat.bad` \
         claims a file fails to start up; nothing checked that claim until this gate, and a \
         fixture that starts up fine makes every assertion resting on it a coincidence.\n\
         \n\
         THE FIX, one of two:\n\
         \n\
         1. If the test that drives it asserts `is_ok()`, or starts the world up and INVOKES \
         (asserting the error comes at EVAL), the file is a valid program and the NAME is wrong — \
         `git mv` it to `.wat` and update every `.rs` referrer. That is the common case: 13 of the \
         first 16 were exactly this.\n\
         \n\
         2. If its test asserts `is_err()` and is `#[ignore]`d as RED-at-HEAD, the badness is \
         BANKED against a substrate change that has not landed. Declare it: \
         `;; rune:lint(bad-is-banked) \u{2014} <what the substrate should do instead> {OWNER_FIELD} \
         <the ignored test's fn name>`.\n\
         \n\
         ⛔ NOT a fix: adding a `:user::main`. This gate drives `startup_from_file`, which does \
         not want one — that is the binary, and measuring this corpus through the binary is the \
         error that got the first draft of this gate withdrawn.\n\
         \n\
         ({declared} of the offenders below declared something; the rest declared nothing.)\n\n{}\n",
        failures.len(),
        failures.join("\n")
    );
}

/// Expand one `#[test]` per shard. Written out rather than looped so nextest can schedule them in
/// parallel and so a failure names WHICH shard — the same shape, and the same reason, as the C19
/// determinism gate over this identical corpus.
macro_rules! shards {
    ($($name:ident = $idx:expr;)*) => {
        $(
            #[test]
            fn $name() { check_shard($idx); }
        )*
    };
}

shards! {
    every_wat_bad_fixture_actually_fails_shard_00 = 0;
    every_wat_bad_fixture_actually_fails_shard_01 = 1;
    every_wat_bad_fixture_actually_fails_shard_02 = 2;
    every_wat_bad_fixture_actually_fails_shard_03 = 3;
    every_wat_bad_fixture_actually_fails_shard_04 = 4;
    every_wat_bad_fixture_actually_fails_shard_05 = 5;
    every_wat_bad_fixture_actually_fails_shard_06 = 6;
    every_wat_bad_fixture_actually_fails_shard_07 = 7;
    every_wat_bad_fixture_actually_fails_shard_08 = 8;
    every_wat_bad_fixture_actually_fails_shard_09 = 9;
    every_wat_bad_fixture_actually_fails_shard_10 = 10;
    every_wat_bad_fixture_actually_fails_shard_11 = 11;
    every_wat_bad_fixture_actually_fails_shard_12 = 12;
    every_wat_bad_fixture_actually_fails_shard_13 = 13;
    every_wat_bad_fixture_actually_fails_shard_14 = 14;
    every_wat_bad_fixture_actually_fails_shard_15 = 15;
}

#[cfg(test)]
mod reader {
    use super::*;

    #[test]
    fn a_rune_line_yields_its_category_and_reason() {
        assert_eq!(
            declaration_on(";; rune:lint(bad-is-banked) \u{2014} arc 255 should reject this banked-by: some_test"),
            Some((
                "bad-is-banked",
                "arc 255 should reject this banked-by: some_test"
            ))
        );
    }

    #[test]
    fn an_invented_category_is_read_not_skipped() {
        assert_eq!(
            declaration_on(";; rune:lint(whatever) \u{2014} a category nobody defined"),
            Some(("whatever", "a category nobody defined"))
        );
    }

    #[test]
    fn a_malformed_rune_surfaces_as_a_category() {
        assert_eq!(declaration_on(";; rune:lint(bad-is-banked"), Some(("bad-is-banked", "")));
    }

    #[test]
    fn prose_naming_the_words_is_not_a_rune() {
        assert!(declaration_on(";; this fixture is bad by design, banked for later").is_none());
    }

    #[test]
    fn an_owner_field_yields_the_test_name() {
        assert_eq!(
            owner_in("the substrate should reject it banked-by: wrong_leaf_is_a_check_error"),
            Some("wrong_leaf_is_a_check_error")
        );
    }

    #[test]
    fn a_reason_with_no_owner_field_names_nobody() {
        assert!(owner_in("the substrate should reject it, one day, somehow").is_none());
    }

    #[test]
    fn an_ignored_test_reads_as_banked() {
        let src = "// a banked gate\n#[test]\n#[ignore = \"RED-at-HEAD: arc 255\"]\nfn t() {\n}\n";
        assert_eq!(owner_state_in(src, "t"), Some(Owner::Ignored));
    }

    #[test]
    fn a_running_test_reads_as_live() {
        let src = "// a live gate\n#[test]\nfn t() {\n}\n";
        assert_eq!(owner_state_in(src, "t"), Some(Owner::Live));
    }

    #[test]
    fn an_ignore_on_a_different_test_does_not_vouch_for_this_one() {
        // The upward scan must stop at the blank line: `other`'s `#[ignore]` is not `t`'s.
        let src = "// two gates\n#[test]\n#[ignore = \"banked\"]\nfn other() {\n}\n\n#[test]\nfn t() {\n}\n";
        assert_eq!(owner_state_in(src, "t"), Some(Owner::Live));
    }

    #[test]
    fn a_test_that_is_not_there_is_absent() {
        assert_eq!(owner_state("no_such_test_function_anywhere", &[String::from("fn t() {}")]), Owner::Absent);
    }
}
