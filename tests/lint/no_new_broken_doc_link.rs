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
//! be the opening move: it would redden the build in 41 places across `wat-reader`, `resolve`,
//! `load`, `check` and `kernel` at once.

use std::collections::BTreeMap;
use std::path::Path;

/// ★ THE LEDGER — every intra-doc link in this workspace that does not resolve, BY NAME.
///
/// `(file, link-target, sites-in-that-file)`. Seeded 2026-09-01 (arc 278 stone E3) from the
/// command in this file's header: 41 sites over 34 named keys, after that stone fixed
/// `src/value/signal.rs`'s nine — two of which stone E4 had broken one commit earlier.
///
/// **`src/value/signal.rs` is deliberately absent.** It is the file this gate was built out of,
/// and it is at zero. Anything reappearing under it is a regression of the strike itself.
///
/// This list may only SHRINK. See the header for why a line is never added to quiet a red.
const KNOWN_BROKEN_DOC_LINKS: &[(&str, &str, usize)] = &[
    ("crates/wat-reader/src/parser.rs", "parse_all", 2),
    ("crates/wat-reader/src/parser.rs", "parse_one", 2),
    ("crates/wat-reader/src/span.rs", "crate::hash::canonical_edn_wat", 1),
    ("src/bin/cargo-wat.rs", "1", 2),
    ("src/channel/mod.rs", "crate::io::PipeWriter::write_all", 1),
    ("src/check.rs", "CheckError::BareLegacyPrimitive", 1),
    ("src/check/env.rs", "register", 1),
    ("src/check/env.rs", "register_overlay", 1),
    ("src/check/error_edn.rs", "CheckErrorKind", 1),
    ("src/config.rs", "DEFAULT_DIMS", 2),
    ("src/edn_shim.rs", "EdnReadError::NoTypeRegistry", 1),
    ("src/edn_shim.rs", "value_to_edn", 1),
    ("src/freeze.rs", "RuntimeError::EvalVerificationFailed", 2),
    ("src/kernel/address.rs", "feedback_dont_build_the_forcing_function", 1),
    ("src/kernel/address.rs", "feedback_vended_primitives_never_deadlock", 2),
    ("src/kernel/listener.rs", "feedback_vended_primitives_never_deadlock", 1),
    ("src/load.rs", "LoadError::CycleDetected", 1),
    ("src/load.rs", "LoadError::DuplicateLoad", 1),
    ("src/load.rs", "LoadError::SetterInLoadedFile", 1),
    ("src/macros/expand.rs", "expand::expand_form", 1),
    ("src/macros/mod.rs", "ScopeId", 1),
    ("src/resolve/mod.rs", "RESERVED_PREFIXES", 1),
    ("src/resolve/mod.rs", "SymbolTable", 1),
    ("src/resolve/mod.rs", "check_form", 1),
    ("src/runtime.rs", "RuntimeError::ArityMismatch", 1),
    ("src/runtime.rs", "RuntimeError::DuplicateDefine", 1),
    ("src/services/client.rs", "docs/ZERO-MUTEX.md", 1),
    ("src/test_runner.rs", "crate::test_suite", 2),
    ("src/types.rs", "crate::macros::MacroRegistry::register_stdlib", 1),
    ("src/types.rs", "crate::resolve::RESERVED_PREFIXES", 1),
    ("src/value/environment.rs", "SymbolTable", 1),
    ("src/value/mod.rs", "must_use", 1),
    ("src/value/symbol_table.rs", "2", 1),
    ("src/value/symbol_table.rs", "RuntimeError::NoEncodingCtx", 1),
];

/// Wall-clock bound on the spawned doc build. **300s, against a worst OBSERVED 10.68s.**
///
/// Both numbers, because a bound chosen for roundness is a bound nobody can re-derive. Measured
/// 2026-09-01 on this host, release profile: **0–1s** when cargo replays cached rustdoc
/// diagnostics, **9–11s** with the doc units cold (the state a first floor run leaves), and
/// **10.68s** — the worst — running inside the full `wat::lint` binary under nextest's parallel
/// load. 300s is ~28x that worst case.
///
/// The headroom is deliberately large, and the asymmetry is the reason: this bound must be
/// UNCROSSABLE by legitimate slowness (a loaded CI box, a slower machine, a cold release
/// dependency graph) and crossable only by a wait that was never going to end. A false red here
/// would be a flake, and this tree does not have those. It is still far below the floor's own
/// multi-minute runtime, so an expiry surfaces inside a normal floor rather than after it.
/// ⛔ **THE RESIDUAL, AND THE ORCHESTRATOR'S RULING ON IT (arc 278, E3).** This bound converts a
/// hang into a red; it does NOT make the gate correct under contention. A cargo holding the target
/// lock for 40s → this passes. For 400s → **this reds on a tree with no broken links.** The rider
/// raised that against `CLAUDE.md`'s absolute *"there is no such thing as a known flake"*, and was
/// right to. **Ruled: keep the bound; do NOT give the doc build its own `CARGO_TARGET_DIR`.**
///
/// Three reasons, in order of weight:
///
/// 1. **Red-when-it-cannot-measure is the CORRECT answer.** The alternative is passing without
///    measuring, which is the failure this tree already has a name for — a check that reports
///    success without running. A gate that cannot see the tree must say so.
/// 2. **It is not a flake in the doctrine's sense.** A flake passes and fails on one tree for
///    UNKNOWN reasons, which is why the doctrine forbids re-running: the re-run destroys the only
///    evidence. This fails for a *stated, captured* reason and quotes cargo's own
///    `Blocking waiting for file lock …` line in the red. Resolving the named cause and
///    re-measuring is `extirpare` — fix the condition, then measure — not the forbidden
///    re-run-until-green.
/// 3. **The structural cure buys out a condition the operating discipline already forbids** (never
///    two cargos on this target dir) at the price of a full cold dependency compile on every fresh
///    clone. That trade is not worth it here.
///
/// **If this ever reds for a reason other than a held lock, that is a real finding** — the message
/// below tells the reader how to tell the two apart, and neither is a licence to dismiss it.
const DOC_BUILD_TIMEOUT_SECS: u32 = 300;

/// The bounding wrapper. Separate from the timeout value so the panic messages below can name the
/// binary they needed.
const TIMEOUT_BIN: &str = "timeout";

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

/// `--release` only when this test binary itself is a release build, so the doc build reuses the
/// dependency artifacts the current run already produced instead of compiling a second profile.
/// Derived from the executable's own path rather than `cfg!(debug_assertions)`, which is a
/// property of the assertion setting and not of the profile.
fn profile_flag() -> &'static [&'static str] {
    let release = std::env::current_exe()
        .map(|p| p.components().any(|c| c.as_os_str() == "release"))
        .unwrap_or(false);
    if release { &["--release"] } else { &[] }
}

#[test]
fn no_broken_intra_doc_link_outside_the_frozen_ledger() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // `timeout` wraps cargo; see this file's header for why an unbounded spawn is not an option.
    // GNU timeout puts the child in its own process group, so the KILL after `--kill-after` reaps
    // the rustdoc children too rather than orphaning them onto the floor.
    let out = std::process::Command::new(TIMEOUT_BIN)
        .current_dir(root)
        .env("RUSTDOCFLAGS", "-W rustdoc::broken_intra_doc_links")
        .arg("--kill-after=15s")
        .arg(format!("{DOC_BUILD_TIMEOUT_SECS}s"))
        .arg(env!("CARGO"))
        .args(["doc", "--no-deps", "--workspace", "--color=never"])
        .args(profile_flag())
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "could not spawn `{TIMEOUT_BIN}` to bound the doc build: {e}\n\n\
                 This gate will NOT run the doc build unbounded — it spawns cargo into the target \
                 directory the floor itself is using, and a cargo that cannot take the build lock \
                 waits forever instead of failing. Install coreutils `timeout`, or change this \
                 gate to bound the build some other way; do not delete the bound."
            )
        });

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let code = out.status.code();

    // ── ARM: the bound expired. 124 = TERM'd at the deadline; 137 = still alive, KILL'd after. ──
    let timed_out = code == Some(124) || code == Some(137);
    assert!(
        !timed_out,
        "THE DOC BUILD DID NOT FINISH IN {DOC_BUILD_TIMEOUT_SECS}s — it was killed, not failed.\n\n\
         The likeliest cause by far is ANOTHER CARGO HOLDING THE TARGET-DIRECTORY LOCK \
         (`target/<profile>/.cargo-lock`): a second cargo does not error, it prints \"Blocking \
         waiting for file lock on …\" and waits — cargo words the tail per lock (\"artifact \
         directory\", \"build directory\", \"package cache\"), so grep the stable prefix \
         \"Blocking waiting for file lock\" in the captured output below, and run `pgrep -af cargo` — a concurrent \
         interactive build or a second floor on this same target directory is the thing to find.\n\n\
         If nothing else was running, the doc build itself got {DOC_BUILD_TIMEOUT_SECS}s-slow, \
         which is ~28x its measured worst case and is a real finding — re-measure with the \
         instrument in this file's header BEFORE touching the bound.\n\n\
         Captured output:\n{combined}"
    );

    // ── ARM: `timeout` itself could not run cargo. Distinct from a doc build that ran and failed. ──
    let wrapper_failed = code == Some(125) || code == Some(126) || code == Some(127);
    assert!(
        !wrapper_failed,
        "`{TIMEOUT_BIN}` could not run cargo (exit {code:?}: 125 = timeout failed, 126 = cargo \
         found but not executable, 127 = cargo not found). The doc build never ran, so this gate \
         measured NOTHING — it is not reporting a clean tree:\n{combined}"
    );

    // ── ARM: the doc build ran to completion and failed. ──
    assert!(
        out.status.success(),
        "`cargo doc --no-deps --workspace` FAILED ({}). The gate cannot measure links from a doc \
         build that did not complete, and a failed doc build is itself a red:\n{combined}",
        out.status
    );

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
