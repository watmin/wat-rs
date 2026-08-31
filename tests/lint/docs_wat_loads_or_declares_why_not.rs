//! Gate — EVERY `.wat` under `docs/arc/` must LOAD on the current runtime, **or** declare in its
//! own header, in a closed two-category rune, why it does not.
//!
//! ## Why this exists at all
//!
//! `every_wat_scripts_file_loads` (`wat_scripts_fixes_load.rs`) states the doctrine: *"ALL wat must
//! remain correct, always"*. It walks `wat-scripts/` **only**, and `docs/arc/**` is exempt by
//! omission. The reasoning that put `.wat` there is written down and is CORRECT —
//! `probes/red-owner-signals-child.wat` says so in its own header: a deliberately-failing probe
//! parked under `wat-scripts/` would break that gate, so it lives under `docs/`.
//!
//! Sound reasoning; graveyard consequence. Deliberately-red and genuinely-rotted files ended up in
//! one directory and nothing could tell them apart — `probes/surface-field-dispatch.wat`, whose own
//! header promises it prints 142, died at startup for ~8 weeks after `defsurface` gained a required
//! `:nature`, and looked exactly like the probes that are supposed to fail.
//!
//! ## The contract
//!
//! A file passes if it loads (`startup_from_source` + `FsLoader`, exactly as its sibling gate does
//! and exactly as `cargo wat` runs it), **or** if its header carries:
//!
//! ```text
//! ;; rune:lint(red-by-design) — <what the FAILURE PROVES>
//! ;; rune:lint(historical)    — <what past state this preserves; it must NOT be migrated>
//! ```
//!
//! The discriminating question: does the file fail because **failing is the point**
//! (`red-by-design`), or because it is a **photograph of a substrate that no longer exists**
//! (`historical`)? The set is CLOSED — a third category is a red build here, for the reason
//! `no_unknown_sequi_rune` documents: `rune:purgare`'s categories were left undefined and one of
//! them names a mechanism absent at all three of its sites.
//!
//! ## What this gate can and cannot do — stated, because the gap is the whole point
//!
//! It closes the SET and it demands a SENTENCE. It **cannot** tell you the sentence is TRUE. A
//! rune whose reason is "it fails" is not a reason, and no string check can distinguish that from
//! one a reader could check against the file's behaviour. Claiming otherwise would be the
//! decoration this repo keeps finding and removing. What the gate buys is that rot can no longer
//! be silent: a file that stops loading must acquire a rune, and acquiring one is a deliberate act
//! that names a category and writes a claim someone can falsify.
//!
//! It also asks ONE question — does the file LOAD — and a rune answers only that question. A probe
//! whose designed red fires LATER (at rule compilation, or at runtime) loads fine and therefore
//! needs NO rune: `harness-experiri/experiri-acc-head.wat` is exactly that shape, it passes here,
//! and runing it would have bought nothing while blinding this gate to its future rot. Declare a
//! file only when it genuinely cannot load.
//!
//! ⚠ A rune EXEMPTS the file from the load check — that is its whole function — so runing a
//! **rotted** file rebuilds the graveyard inside the gate and leaves it looking enforced. Migrate
//! first; rune only when the failure is the artifact.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::FsLoader;

/// The CLOSED set. A third member is a deliberate act: it needs its own discriminating question
/// (against the two above), written where a reader will meet it — not invented at a call site.
const DECLARED_CATEGORIES: &[&str] = &["red-by-design", "historical"];

/// A reason shorter than this is not a sentence a reader can check against the file's behaviour.
/// The gate cannot judge a reason's truth (see the header); it can refuse an empty gesture.
const MIN_REASON_CHARS: usize = 24;

fn collect_wat(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().is_some_and(|x| x == "wat") {
            out.push(p);
        }
    }
}

/// The declaration on one line, if any: `(category, reason)`.
///
/// Split on the fixed needle rather than regex, as `no_unknown_sequi_rune` does — the marker is a
/// literal and the category is everything up to the closing paren. An UNCLOSED `rune:lint(` yields
/// the rest of the line as the category so the caller reports a malformed rune instead of walking
/// silently past one.
fn declaration_on(line: &str) -> Option<(&str, &str)> {
    let at = line.find("rune:lint(")?;
    let after = &line[at + "rune:lint(".len()..];
    match after.find(')') {
        Some(close) => Some((&after[..close], after[close + 1..].trim_start_matches(['—', '-', ' ']).trim())),
        None => Some((after, "")),
    }
}

#[test]
fn every_docs_wat_loads_or_declares_why_not() {
    let mut entries = Vec::new();
    collect_wat(Path::new("docs/arc"), &mut entries);
    entries.sort();

    // NON-VACUITY (the shape of `no_ceiling_raise_in_rete.rs:92`): the walk must actually find the
    // tree. A moved or renamed docs root finding zero files would make this gate pass forever
    // while checking nothing — `complectens` found 10 of 15 file-walking gates in tests/lint/ with
    // no such guard. Deliberately NOT a count: a hardcoded number here would be a second copy of
    // "how much wat lives under docs/" and would rot the first time an arc lands one.
    assert!(
        !entries.is_empty(),
        "no .wat found under docs/arc/ — the gate is measuring nothing"
    );

    let mut failures = Vec::new();
    let mut declared = 0usize;
    for path in &entries {
        let rel = path.to_str().expect("utf8 path");
        let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));

        if let Some((cat, reason)) = src.lines().find_map(declaration_on) {
            declared += 1;
            if !DECLARED_CATEGORIES.contains(&cat) {
                failures.push(format!(
                    "  {rel}\n      rune:lint({cat}) is not one of the two categories \
                     {DECLARED_CATEGORIES:?} — a THIRD category needs its own discriminating \
                     question against those two, added here deliberately"
                ));
            } else if reason.chars().count() < MIN_REASON_CHARS {
                failures.push(format!(
                    "  {rel}\n      rune:lint({cat}) carries no reason ({} chars). \
                     red-by-design must name WHAT THE FAILURE PROVES; historical must name WHAT \
                     PAST STATE IS PRESERVED. \"it fails\" is not a reason",
                    reason.chars().count()
                ));
            }
            continue;
        }

        // FsLoader (the disk loader cargo-wat uses) — NOT InMemoryLoader — so a probe's relative
        // `(:wat::load-file! "…")` resolves against its own dir exactly as it does when run. The
        // gate must measure the way the file actually runs, or it lies.
        if let Err(e) = startup_from_source(&src, Some(rel), Arc::new(FsLoader)) {
            failures.push(format!("  {rel}\n      {e:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} docs/arc/ .wat files neither load on the current runtime nor declare why not \
         ({declared} declared). A file that fails must EITHER be migrated (it is rot) OR carry \
         `;; rune:lint(red-by-design) — <what the failure proves>` / \
         `;; rune:lint(historical) — <what past state this preserves>`:\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
}

#[test]
fn the_declaration_reader_sees_a_rune_and_ignores_prose() {
    assert_eq!(
        declaration_on(";; rune:lint(red-by-design) — the refusal is the proof"),
        Some(("red-by-design", "the refusal is the proof"))
    );
    // An invented category is READ (so the walk can report it), not silently skipped.
    assert_eq!(
        declaration_on(";; rune:lint(too-slow) — a category nobody defined"),
        Some(("too-slow", "a category nobody defined"))
    );
    // A malformed (unclosed) rune surfaces as a category, not as "no declaration".
    assert_eq!(declaration_on(";; rune:lint(historical"), Some(("historical", "")));
    // Prose mentioning the word is not a rune.
    assert!(declaration_on(";; this file is historical and red by design").is_none());
    // The needle is exact: `lint(` without the `rune:` prefix is not a rune. (This case replaced
    // an assertion that fed the reader a real wat form — `no_inlined_wat_in_tests` flags a string
    // literal wat's reader parses, and its carve-out rune is not something to reach for to win an
    // argument between gates when a stronger boundary case says the same thing.)
    assert!(declaration_on(";; see the lint(docs_wat) gate for why").is_none());
}
