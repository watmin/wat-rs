//! **AN INTRA-DOC LINK THAT NAMES A PATH MUST RESOLVE — AND THE SET THAT DOES NOT IS FROZEN BY NAME.**
//!
//! ── THE CLASS ────────────────────────────────────────────────────────────────────────────────
//!
//! `src/value/signal.rs`'s ceiling docs cross-referenced
//! ``[`RuntimeErrorKind::FixpointRoundCapExceeded`]`` and
//! ``[`RuntimeErrorKind::SessionMemoryCeilingExceeded`]``. Arc 278 stone E4 moved both variants
//! onto `ReteCeiling`, and **both links stopped resolving in the same commit**. The floor was
//! green. Clippy was silent (`deny(clippy::all)`, workspace-wide). Nothing caught it, and nothing
//! could have: a broken intra-doc link is a **rustdoc** lint, and before this gate
//!
//! ```text
//! grep -rn 'cargo doc\|rustdoc' scripts/*.sh .github/workflows/*.yml   → nothing
//! grep -rn 'broken_intra_doc_links' src/lib.rs Cargo.toml              → nothing
//! ```
//!
//! **nothing in this tree ever built docs.** Every `[`path`]` in every doc comment was an
//! unverified citation — the same shape as the `file:line` citation-rot class, one level up:
//! rustdoc cannot check a line number in prose, but it CAN check a path, and a check that is
//! available and not run is a check nobody has.
//!
//! This is the sibling of `tests/lint/no_stale_path_in_doc.rs` (a source path named in a comment
//! must exist on disk). That one decides with the filesystem; this one decides with the compiler's
//! own name resolution, which is strictly stronger where it applies.
//!
//! ── ★ WHY A NAMED LIST AND NOT A COUNT — THIS TREE HAS ALREADY PAID FOR THE DISTINCTION ──────
//!
//! `src/rete/purity.rs`'s `KNOWN_UNREVIEWED` records the exact failure in its own words: *"The
//! gate wanted SET MEMBERSHIP and measured CARDINALITY"*, so *"a brand-new unruled verb walked in
//! free whenever a strike also ruled on one, which is the normal case for a strike."* A count
//! ratchet here fails identically:
//!
//! ```text
//! fix one broken link  +  add one broken link  =  the SAME number, gate stays GREEN
//! ```
//!
//! So [`KNOWN_BROKEN_DOC_LINKS`] freezes the links **by name**, and is a
//! **RATCHET IN BOTH DIRECTIONS**:
//!
//! - a link NOT in the list ⇒ RED, and the gate NAMES it. A new broken citation is never free.
//! - a listed link that now resolves ⇒ RED. Fix it, delete its line; the set only shrinks.
//!
//! Never add a line to make a red gate green — that is the laundering this gate exists to refuse.
//! `[[feedback_a_gate_freezes_names_never_a_count]]`.
//!
//! **The entry is `(file, link-target, sites)`, and the third field is NOT a smuggled cardinality
//! check.** Two occurrences of the same broken target in one file are indistinguishable by
//! `(file, target)` alone, so without it, fixing one of `parser.rs`'s two ``[`parse_one!`]`` links
//! would leave the gate green — the very direction `purity.rs` warns about, in miniature. The
//! count is scoped to a key that is always NAMED, so every red still says which file and which
//! target moved. That is the difference from a bare total, which can name nothing.
//!
//! ── THE INSTRUMENT (trap 4: a list with no instrument is unfalsifiable) ──────────────────────
//!
//! The list below was produced by exactly this command, from the repo root:
//!
//! ```text
//! RUSTDOCFLAGS="-W rustdoc::broken_intra_doc_links" cargo doc --release --no-deps --workspace
//! ```
//!
//! and the gate RUNS that command rather than reading a committed artifact — a checked-in copy of
//! rustdoc's output would be a hand-maintained mirror of a real thing, which is the drift
//! generator `tests/lint/gen_doc_surface_matches.rs` was built against. Regenerate the list with
//! the same command; `--release` is dropped when this test binary is a debug build, so the doc
//! build reuses whichever dependency artifacts the current run already has.
//!
//! **Measured cost (2026-09-01, arc 278 E3):** warm 1s, whole-workspace rustdoc after touching one
//! source file 5–11s, against a floor measured in minutes. Cargo replays cached rustdoc
//! diagnostics for fresh units, so a repeat run is ~1s and still reports the full set. The lint is
//! warn-by-default in rustdoc; `-W` is stated anyway so the instrument does not depend on a
//! default.
//!
//! ── ⛔ WHY THE SPAWNED DOC BUILD IS BOUNDED — A HUNG FLOOR IS WORSE THAN A RED ONE ──────────
//!
//! This gate spawns `cargo doc` into the SAME target directory the floor is running out of. Cargo
//! takes an exclusive lock at `target/<profile>/.cargo-lock`, and a cargo that cannot take it
//! does not fail — it prints *"Blocking waiting for file lock on …"* (cargo words the tail
//! differently per lock: *artifact directory*, *build directory*, *package cache*) and **waits
//! forever**. Driven 2026-09-01 by holding that exact file: cargo printed *"Blocking waiting for
//! file lock on artifact directory"* and waited. So a second cargo anywhere on the box (an interactive
//! `cargo build`, a second floor — this project's recovery notes record exactly that incident)
//! would turn this test into an unbounded wait.
//!
//! A hang is strictly worse than a failure here. There is no Summary line to read, the
//! capture-whole-then-do-not-re-run doctrine has nothing to capture, and the floor's own law —
//! read the Summary, never a piped exit code — is unusable against a run that never prints one.
//!
//! So the doc build runs under `timeout(1)` and an expiry is a NAMED red that says what to look
//! for. The floor is Linux-only and coreutils `timeout` is assumed; if it is missing this test
//! fails loudly rather than quietly falling back to an unbounded spawn.
//!
//! ── WHAT THIS GATE CANNOT DO ─────────────────────────────────────────────────────────────────
//!
//! It checks that a link RESOLVES. It cannot check that it resolves to the item the prose means —
//! `[`ReteCeiling::SessionMemoryCeilingExceeded`]` and
//! `[`ReteCeiling::SessionMemoryCeilingExceededOnInsert`]` both resolve, and pointing at the wrong
//! one reads fine. It also sees only `rustdoc::broken_intra_doc_links`; other rustdoc warnings
//! (unclosed HTML tags, bare URLs) are outside its contract and are NOT frozen here.
//!
//! `#![deny(rustdoc::broken_intra_doc_links)]` is the endpoint once this list is empty. It cannot
//! be the opening move: it would redden the build in 31 places across `wat-reader`, `resolve`,
//! `load`, `check` and `kernel` at once.

use std::collections::BTreeMap;

use wat::doc_link::{unresolved_links, KNOWN_BROKEN_DOC_LINKS};

// The ledger, the extractor, and the judge live in `src/doc_link.rs`.
// `scripts/floor.sh` runs `doc_link_ledger` against the captured cargo doc log.
// These two tests stay in nextest. The judge is not a test, so it is not an ignore.

// rune:lint(vacuity-guard) — the population here is rustdoc's diagnostic stream, not a file
// set, and this ledger is MEANT to reach zero — a "found at least N" floor would make an empty,
// correct tree RED, which is the opposite of the property. What this gate does instead:
// `the_unresolved_link_extractor_still_matches_rustdocs_format` proves the parser against a
// fixed sample of rustdoc's wording. If a toolchain bump rewords the diagnostic, THAT test reds
// FIRST and names the format change, instead of the gate silently parsing nothing out of a full
// build and reporting the whole ledger as resolved. A doc build that timed out or failed is a red
// of `scripts/floor.sh` itself: the judge reads a log that build already wrote.


/// **The extractor must be proven against rustdoc's format, not against the population.**
///
/// A vacuity guard of the usual shape — "assert we found at least N" — is unavailable here, and
/// stating why matters: this ledger is meant to reach ZERO, at which point such an assert would
/// make an empty, correct tree RED. So the instrument is proven on a fixed sample instead. If a
/// toolchain bump reworded the diagnostic, this test goes red FIRST and says so, instead of the
/// gate silently parsing nothing and reporting the whole ledger as resolved.
#[test]
fn the_unresolved_link_extractor_still_matches_rustdocs_format() {
    const SAMPLE: &str = "\
warning: unresolved link to `parse_one`
  --> crates/wat-reader/src/parser.rs:10:9
   |
10 | //! - [`parse_one!`] — macro that parses a single top-level form and
   |         ^^^^^^^^^^ no item named `parse_one` in scope

warning: unresolved link to `parse_one`
   --> crates/wat-reader/src/parser.rs:237:7
    |
237 | /// [`parse_one!`] with an explicit span-label for diagnostics.
    |       ^^^^^^^^^^ no item named `parse_one` in scope

warning: unclosed HTML tag `T`
  --> src/value/value.rs:1:1

warning: `wat` (lib doc) generated 3 warnings
";
    let got = unresolved_links(SAMPLE);
    assert_eq!(
        got.len(),
        1,
        "extractor should fold the two sites into one named key, got: {got:?}"
    );
    assert_eq!(
        got.get(&(
            "crates/wat-reader/src/parser.rs".to_string(),
            "parse_one".to_string()
        )),
        Some(&2),
        "extractor lost the site count or the key shape: {got:?}"
    );
}

/// The ledger is a set: two lines for one key would let one of them rot unnoticed.
#[test]
fn the_broken_doc_link_ledger_has_no_duplicate_keys() {
    let mut seen: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (file, target, _) in KNOWN_BROKEN_DOC_LINKS {
        *seen.entry((file, target)).or_insert(0) += 1;
    }
    let dups: Vec<String> = seen
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|((f, t), n)| format!("{f}: `{t}` listed {n} times"))
        .collect();
    assert!(
        dups.is_empty(),
        "KNOWN_BROKEN_DOC_LINKS has duplicate keys — merge them into one line with the site \
         count:\n  {}",
        dups.join("\n  ")
    );
}

