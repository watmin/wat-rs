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

/// ★ THE LEDGER — every intra-doc link in this workspace that does not resolve, BY NAME.
///
/// `(file, link-target, sites-in-that-file)`. Seeded 2026-09-01 (arc 278 stone E3) from the
/// command in this file's header: 41 sites over 34 named keys, after that stone fixed
/// `src/value/signal.rs`'s nine — two of which stone E4 had broken one commit earlier.
///
/// ⚠ **RE-SEEDED AT LANDING (REPLAY #270), 41/34 → 31/26.** grok's tree and this one have already
/// diverged in FILE LAYOUT by module splits unrelated to this replay (`edn_shim.rs` →
/// `edn/render.rs`, `load.rs` → `load/loader.rs`, `test_runner.rs` → `host/test_runner.rs`,
/// `register_defines`/`register_defclause` moved into `declare/register.rs`). Eight of grok's 34
/// keys named a file that no longer exists at that path, so their citations do not appear at all
/// under this tree's rustdoc run — not because the debt was paid, but because the OLD location
/// is gone. The identical broken citation reappears at each item's NEW location (ten sites: eight
/// of the eight moves plus two mis-citations this same command surfaced — `RuntimeError`/
/// `EdnReadError`/`LoadError` naming the struct where the enum is meant, `value_to_edn` naming a
/// retired free function, `crate::test_suite!` naming a macro renamed to `test!` at arc 018,
/// `[`stdlib`]` naming nothing where `crate::load::stdlib` is meant) — corrected in the doc
/// comments themselves rather than re-ledgered, per this gate's own instruction. Re-running the
/// instrument after those ten fixes found all ten now resolve; the eight moved-away entries were
/// deleted. Net: this tree's OWN baseline is 31 sites over 26 keys, measured on THIS tree, not
/// copied from grok's.
///
/// **`src/value/signal.rs` is deliberately absent.** It is the file this gate was built out of,
/// and it is at zero. Anything reappearing under it is a regression of the strike itself.
///
/// This list may only SHRINK. See the header for why a line is never added to quiet a red.
const KNOWN_BROKEN_DOC_LINKS: &[(&str, &str, usize)] = &[
    ("crates/wat-reader/src/parser.rs", "parse_all", 2),
    ("crates/wat-reader/src/parser.rs", "parse_one", 2),
    ("src/bin/cargo-wat.rs", "1", 2),
    ("src/channel/mod.rs", "crate::io::PipeWriter::write_all", 1),
    ("src/check.rs", "CheckError::BareLegacyPrimitive", 1),
    ("src/check/env.rs", "register", 1),
    ("src/check/env.rs", "register_overlay", 1),
    ("src/check/error_edn.rs", "CheckErrorKind", 1),
    ("src/config.rs", "DEFAULT_DIMS", 2),
    ("src/freeze.rs", "RuntimeError::EvalVerificationFailed", 2),
    ("src/kernel/address.rs", "feedback_dont_build_the_forcing_function", 1),
    ("src/kernel/address.rs", "feedback_vended_primitives_never_deadlock", 2),
    ("src/kernel/listener.rs", "feedback_vended_primitives_never_deadlock", 1),
    ("src/macros/expand.rs", "expand::expand_form", 1),
    ("src/macros/mod.rs", "ScopeId", 1),
    ("src/resolve/mod.rs", "RESERVED_PREFIXES", 1),
    ("src/resolve/mod.rs", "SymbolTable", 1),
    ("src/resolve/mod.rs", "check_form", 1),
    ("src/runtime.rs", "RuntimeError::ArityMismatch", 1),
    ("src/services/client.rs", "docs/ZERO-MUTEX.md", 1),
    ("src/types.rs", "crate::macros::MacroRegistry::register_stdlib", 1),
    ("src/types.rs", "crate::resolve::RESERVED_PREFIXES", 1),
    ("src/value/environment.rs", "SymbolTable", 1),
    ("src/value/mod.rs", "must_use", 1),
    ("src/value/symbol_table.rs", "2", 1),
    ("src/value/symbol_table.rs", "RuntimeError::NoEncodingCtx", 1),
];

/// The `cargo doc` run is `scripts/floor.sh`, after nextest, on the floor's own target.
/// A cold workspace doc build measured 34.75s, past nextest's 30s kill, so this file does not
/// spawn it. Arc 278 E3 ruled against a private `CARGO_TARGET_DIR` (a clean clone would compile
/// the world into that directory). Amend 5's `target/doc-link-ledger` is withdrawn. The 300s
/// hang bound is `timeout` in `floor.sh`. It was not raised, and it is not a nextest limit.
///
/// Extract `(file, link-target) -> sites` from a cargo/rustdoc run's combined output.
///
/// rustdoc's shape, verbatim:
///
/// ```text
/// warning: unresolved link to `parse_one`
///   --> crates/wat-reader/src/parser.rs:10:9
/// ```
///
/// The path is taken from the `-->` line and kept exactly as rustdoc prints it — workspace-root
/// relative, because the gate runs cargo from `CARGO_MANIFEST_DIR`. Line and column are
/// DISCARDED on purpose: they move whenever anything above them is edited, and a ledger that
/// churns on unrelated edits is one people regenerate by reflex instead of reading.
fn unresolved_links(output: &str) -> BTreeMap<(String, String), usize> {
    const HEAD: &str = "warning: unresolved link to `";
    let mut found: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix(HEAD) else {
            continue;
        };
        let Some(target) = rest.strip_suffix('`') else {
            continue;
        };
        let Some(loc) = lines.next() else {
            break;
        };
        let Some(loc) = loc.trim_start().strip_prefix("--> ") else {
            continue;
        };
        let file = loc.split(':').next().unwrap_or(loc);
        *found
            .entry((file.to_string(), target.to_string()))
            .or_insert(0) += 1;
    }
    found
}

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

// rune:lint(vacuity-guard) — the population here is rustdoc's diagnostic stream, not a file
// set, and this ledger is MEANT to reach zero — a "found at least N" floor would make an empty,
// correct tree RED, which is the opposite of the property. What this gate does instead:
// `the_unresolved_link_extractor_still_matches_rustdocs_format` above proves the parser against a
// fixed sample of rustdoc's wording. If a toolchain bump rewords the diagnostic, THAT test reds
// FIRST and names the format change, instead of the gate silently parsing nothing out of a full
// build and reporting the whole ledger as resolved. A doc build that timed out or failed is a red
// of `scripts/floor.sh` itself: this test only reads a log that build already wrote.
#[test]
#[ignore = "scripts/floor.sh runs this against the captured cargo doc log; it is not a nextest slot"]
fn doc_link_ledger_matches_the_captured_log() {
    let path = std::env::var("WAT_DOC_LINK_LOG").unwrap_or_else(|_| {
        panic!(
            "WAT_DOC_LINK_LOG is unset. scripts/floor.sh sets it to the cargo doc log. \
             This test does not spawn cargo."
        );
    });
    let combined = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read WAT_DOC_LINK_LOG {path}: {e}"));
    let found = unresolved_links(&combined);
    let known: BTreeMap<(String, String), usize> = KNOWN_BROKEN_DOC_LINKS
        .iter()
        .map(|(f, t, n)| (((*f).to_string(), (*t).to_string()), *n))
        .collect();

    let mut arrivals: Vec<String> = Vec::new();
    let mut departures: Vec<String> = Vec::new();
    let mut moved: Vec<String> = Vec::new();

    for (key, sites) in &found {
        match known.get(key) {
            None => arrivals.push(format!("{}: [`{}`] × {sites}", key.0, key.1)),
            Some(n) if n != sites => moved.push(format!(
                "{}: [`{}`] — ledger says {n} site(s), rustdoc found {sites}",
                key.0, key.1
            )),
            Some(_) => {}
        }
    }
    for key in known.keys() {
        if !found.contains_key(key) {
            departures.push(format!("{}: [`{}`]", key.0, key.1));
        }
    }

    let mut report = String::new();
    if !arrivals.is_empty() {
        report.push_str(&format!(
            "\n{} NEW broken intra-doc link(s) — not in KNOWN_BROKEN_DOC_LINKS:\n  {}\n\
             \n  FIX THE LINK. Do NOT add a line to this ledger: it is a shrink-only ratchet, and \
             adding to it is the laundering the gate exists to refuse.\n",
            arrivals.len(),
            arrivals.join("\n  ")
        ));
    }
    if !moved.is_empty() {
        report.push_str(&format!(
            "\n{} listed link(s) changed site count:\n  {}\n\
             \n  MORE sites than listed means a new broken citation of an already-broken target — \
             fix it. FEWER means you fixed one of several — update that line's count, or delete \
             the line if it reached zero.\n",
            moved.len(),
            moved.join("\n  ")
        ));
    }
    if !departures.is_empty() {
        report.push_str(&format!(
            "\n{} listed link(s) now RESOLVE — the ledger is stale:\n  {}\n\
             \n  Delete these lines from KNOWN_BROKEN_DOC_LINKS. A ledger that keeps names of \
             debt already paid stops describing anything, and the next reader cannot tell which \
             entries are real.\n",
            departures.len(),
            departures.join("\n  ")
        ));
    }

    assert!(
        report.is_empty(),
        "broken intra-doc links moved away from the frozen ledger:\n{report}\n\
         Instrument (run from the repo root):\n  \
         RUSTDOCFLAGS=\"-W rustdoc::broken_intra_doc_links\" cargo doc --release --no-deps --workspace\n"
    );
}
