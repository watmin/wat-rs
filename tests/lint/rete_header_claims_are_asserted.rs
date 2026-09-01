//! **A HEADER'S STRUCTURAL CLAIM IS AN ASSERTION HERE, OR IT IS NOT MADE.**
//!
//! ── THE CLASS ────────────────────────────────────────────────────────────────────────────────
//!
//! On 2026-08-30 a documentation pass took `src/rete` from 111 undocumented functions to 0, and
//! in the same commits wrote several claims about the tree's own LAYOUT. Two wards were cast at
//! that work the same day. `probare` checked 23 falsifiable claims in the prose and found 20 true
//! — the prose about MECHANISM is sound. `intueri` checked the claims about STRUCTURE and found
//! most of them false within hours of being written:
//!
//! - *"only these four `*_pass` in THIS file are `#[cfg(test)]`"* — six sites, not four, and
//!   `fire/pass/` has `#[cfg(test)]` items too. Wrong in both directions, in the sentence written
//!   to stop a reader inventing a wrong rule.
//! - *"the eleven session fields a join needs, plus the two that vary"* — `FireCtx` has fourteen.
//! - *"`kernel/tests.rs` is their only caller"* — that file was deleted by the same author, the
//!   same day (now gated by `tests/lint/no_stale_path_in_doc.rs`).
//!
//! Every one is CHEAP to check — a grep, a field count — and none was checked. That is the shape
//! `wat-rs/CLAUDE.md` records as having cost this repo a month: **an assertion no gate can check
//! rots undetected by construction.**
//!
//! ── WHY THIS SHAPE (the honest rung) ─────────────────────────────────────────────────────────
//!
//! A lint that PARSES the English and verifies it is not available — that is prose comprehension,
//! not a build gate. The rung the material allows is the inverse: **stop stating the fact in
//! prose and state it here, where it is executable.** The header then cites this file, and the
//! claim cannot rot without a red build. Prose keeps the WHY; the gate keeps the WHAT.
//!
//! ── WHAT THIS GATE CANNOT DO ─────────────────────────────────────────────────────────────────
//!
//! It pins the specific claims below and nothing else. It cannot find the NEXT unverifiable
//! totality someone writes. The standing rule that covers the residue is a discipline, not a
//! check, and it is written here because there is nowhere better: **if you cannot gate it, do not
//! assert a totality about it.** Say "most", say "at the time of writing", or add a row here.

use std::path::Path;

fn rete_source(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Lines carrying a bare `#[cfg(test)]` attribute, by 1-indexed line number.
fn cfg_test_sites(src: &str) -> Vec<usize> {
    src.lines()
        .enumerate()
        // Matched piecewise, not as one literal: `no_inlined_edn` flags any string opening with
        // `#`/`{`/`[`/`(`, and its rubric says an EDN look-alike is NOT a rune candidate —
        // restructure instead. The char literals and the `cfg(test)` fragment carry no opener.
        .filter(|(_, l)| {
            let t = l.trim();
            t.starts_with('#') && t.ends_with(']') && t.contains("cfg(test)") && t.len() == 12
        })
        .map(|(i, _)| i + 1)
        .collect()
}

#[test]
fn fire_mod_cfg_test_sites_are_exactly_the_documented_set() {
    let src = rete_source("src/rete/kernel/fire/mod.rs");
    let sites = cfg_test_sites(&src);

    // The header says the reliable discriminator is the ATTRIBUTE, not the `_pass` suffix and not
    // the file's location — because both of those rules were tried and both were false. This is
    // the count that sentence rests on.
    assert_eq!(
        sites.len(),
        9,
        "`fire/mod.rs` has {} `#[cfg(test)]` sites, expected 9 (found at lines {sites:?}). Either \
         a test-only item was added/removed — update this number and the module header together \
         — or a production item just became test-only, which is a defect.",
        sites.len()
    );

    // The four reference passes the header names, and the two non-`*_pass` test-only items that
    // falsified the earlier "only these four" claim. Named so the set cannot silently change shape
    // while keeping its size.
    for want in [
        "pub(crate) fn alpha_pass",
        "pub(crate) fn root_join_pass",
        "pub(crate) fn hash_join_pass",
        "pub(crate) fn production_pass",
        "fn keyed_join",
    ] {
        assert!(
            src.contains(want),
            "`fire/mod.rs` no longer defines `{want}` — the module header's account of which \
             items are test-only is now describing a file that does not exist"
        );
    }
}

#[test]
fn fire_ctx_field_count_matches_its_doc() {
    let src = rete_source("src/rete/kernel/fire/mod.rs");
    let start = src
        .find("pub(crate) struct FireCtx<'a> {")
        .expect("FireCtx struct");
    let body = &src[start..];
    let end = body.find("\n}").expect("FireCtx close");
    let fields = body[..end]
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("pub(crate) ") && t.contains(':') && !t.starts_with("pub(crate) struct")
        })
        .count();

    // The doc's persuasive force is entirely in this number — it exists to stop someone repeating
    // a refactor that was tried and reverted. It said "thirteen" while the struct held fourteen.
    assert_eq!(
        fields, 14,
        "`FireCtx` has {fields} fields; its doc says eleven session fields plus three that vary \
         (= 14). Update both together, including the `~84 lines` figure derived from it."
    );
}

#[test]
fn export_lossy_fields_are_still_the_four_the_header_names() {
    let src = rete_source("src/rete/export.rs");
    // The header used to say "exactly one field is lossy". Three more are fabricated on import,
    // and they are the ones `arm.rs` needs to re-arm a network. These are the fabrication sites.
    let empty_pv_in_unpack = src.matches("empty_pv()").count();
    let dummy_ast_in_unpack = src.matches("dummy_ast(span)").count();
    assert!(
        empty_pv_in_unpack >= 1 && dummy_ast_in_unpack >= 2,
        "expected the three node-AST fabrication sites on the import path (1 `empty_pv()`, 2 \
         `dummy_ast(span)`); found {empty_pv_in_unpack} and {dummy_ast_in_unpack}. If they are \
         gone, the codec now carries those fields and the header's lossy list must shrink."
    );
    // ⛔ A `contains` on the header's PROSE stood here and `no_loose_string_assert` refused it —
    // correctly, and on a second ground the lint could not see: it was self-certification. This
    // gate exists because a claim checked only by reading a comment rots undetected; asserting
    // that the comment still contains its own sentence is that same defect one level up. The two
    // assertions above pin the CODE that makes the claim true, which is the checkable half.
}

/// The engine's alpha class lookup is a LINEAR SCAN, and a benchmark label said otherwise.
///
/// `accum_alpha_class_lookup_split` times three structures and, until 2026-08-30, labelled the
/// slowest of them — `std HashMap` — "(engine)". That was true the day
/// `DESIGN-STONE-alpha-class-lookup` was drafted (2026-08-19) and false the moment the stone
/// SHIPPED, because shipping it is what turned `roots` into a `Vec`. The label then described
/// the structure the engine had just stopped using, and went on printing beneath a green test
/// for eleven days.
///
/// **The class is the one this whole file exists for: a label that names a prior state.** Its
/// two siblings this arc are the `H−V` row that claimed a decomposition its arms did not have,
/// and `alloc_counter.rs`'s "NOTHING READS THESE COUNTERS YET" written after the fixpoint began
/// reading them. A benchmark label is the worst host for it — nobody re-derives a table's row
/// names, and the number beside it is right, which makes the row look checked.
///
/// So the claim moves here, where it is executable and OFF THE CLOCK: the ordering assertion in
/// the test itself can only speak while the timings hold, and a structure swapped back to a map
/// would be a compile-time fact, not a slow one.
#[test]
fn alpha_class_lookup_is_still_the_linear_scan_the_benchmark_calls_the_engine() {
    let src = rete_source("src/rete/alpha_tree.rs");

    // The type behind `roots`. A `Vec` of pairs is the stone's winner; a map of any flavour is
    // the decision reversed, and `accum_alpha_class_lookup_split`'s `L` row stops being the
    // production path the instant it changes.
    // ⛔ EXACT, NOT `contains` — `no_loose_string_assert` bans the loose form where a
    // deterministic value is available, and both values here are one rustfmt'd line. Its rubric
    // offers a rune for the legitimately-loose; neither of these qualifies, and the exact form is
    // the stronger gate anyway: a `Vec` swapped for a `SmallVec`, or a second field added to the
    // pair, changes what the benchmark's `L` arm models just as surely as a `HashMap` would.
    let alias = src
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("type AlphaRoots"))
        .unwrap_or_else(|| panic!("`type AlphaRoots` alias is gone from src/rete/alpha_tree.rs"));
    assert_eq!(
        alias, "type AlphaRoots = Vec<(String, Arc<AlphaDiscNode>)>;",
        "`AlphaRoots` changed shape. The alpha class lookup was interned to a linear scan by \
         `DESIGN-STONE-alpha-class-lookup`; if it is a map again the stone was reversed, and \
         `accum_alpha_class_lookup_split` must stop calling its `L` row THE ENGINE."
    );

    // And the lookup itself must still WALK it. A `Vec` reached through a side index would
    // satisfy the alias check while making the benchmark's `L` arm fiction again.
    let body_start = src.find("fn root_for(").unwrap_or_else(|| {
        panic!("`fn root_for(` is gone from src/rete/alpha_tree.rs — the class lookup was renamed")
    });
    let stmt = src[body_start..]
        .lines()
        .nth(1)
        .map(str::trim)
        .unwrap_or_else(|| panic!("`root_for` has no body line"));
    assert_eq!(
        stmt, "self.roots.iter().find(|(c, _)| c == class).map(|(_, n)| n)",
        "`root_for` no longer linear-scans `self.roots`. The benchmark's `L` arm models exactly \
         this `.iter().find()` over the class list — if the lookup changed shape, that arm is \
         measuring something the engine does not do."
    );
}

/// The termination verifier's REACH — the claim `arm.rs` used to make in prose and get wrong.
///
/// That comment said `compile-all` is *"the one door EVERY rule passes"*, unqualified, while
/// `stratify.rs`'s own module doc said the opposite from its side: an imported Export carries no
/// rule AST, so there is nothing there to analyse. Both cannot be true, and a reader landing on
/// either had no route to the correction. The prose is now qualified; the two facts it rests on
/// are here, where a future hand who wires the verifier into a second door — or unwires it from
/// this one — gets a red build instead of a comment that quietly stops describing the tree.
///
/// This gate cannot check that the two doors named in that comment are ALL of them. It checks the
/// count of call sites, which is the half that is decidable.
#[test]
fn the_termination_verifier_still_has_exactly_one_call_site() {
    fn rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                rs_files(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }

    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files(&src_root, &mut files);
    files.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 213 .rs file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        files.len() > 100,
        "the termination-verifier call-site walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    // Call sites only: the definition line (`fn refuse_non_terminating`) and prose mentions inside
    // `//` comments are not callers, and counting them would make this gate green for the wrong
    // reason the first time someone documents the function twice.
    //
    // ⛔ AND NOT THE PROBES. `src/rete/kernel/tests/` drives the verifier directly — that is what a
    // probe for a `pub(crate)` fn IS — and the first draft of this gate counted those seven calls
    // as doors, going red the moment it was written. The claim is about which ENGINE paths reach
    // the verifier; a test that calls it reaches nothing.
    let mut callers: Vec<String> = Vec::new();
    for f in &files {
        if f.components().any(|c| c.as_os_str() == "tests") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("fn refuse_non_terminating") {
                continue;
            }
            if line.contains("refuse_non_terminating(") && !line.contains("pub(crate) fn ") {
                let rel = f.strip_prefix(&src_root).unwrap_or(f).display().to_string();
                callers.push(format!("src/{rel}:{}", i + 1));
            }
        }
    }

    assert_eq!(
        callers.len(),
        1,
        "the termination verifier has {} call sites ({callers:?}), expected 1. `arm.rs`'s comment \
         above the call names the doors that reach an arm WITHOUT it; a second caller means that \
         account is now wrong, and a zero means the verifier is dead.",
        callers.len()
    );
    assert_eq!(
        callers[0].split(':').next().unwrap_or(""),
        "src/rete/kernel/arm.rs",
        "the one call site moved: {callers:?}. It belongs at `arm-session`, which is the door the \
         comment there describes."
    );
}

/// The import door does NOT call the verifier — stated in `arm.rs`'s comment, and deliberately so.
///
/// An imported Export carries no rule AST (`rules_lack_ast`), so a call there could only ever
/// answer `NotAnalysable`; the runtime round cap is the real answer on that path. This row exists
/// so the comment's *"NO hit"* is a fact the build checks rather than a grep someone once ran.
#[test]
fn the_import_door_still_does_not_call_the_termination_verifier() {
    let src = rete_source("src/rete/export.rs");
    let hits =
        src.matches("refuse_non_terminating").count() + src.matches("verify_termination").count();
    assert_eq!(
        hits, 0,
        "`src/rete/export.rs` now mentions the termination verifier ({hits} hits). If the import \
         path genuinely gained a call, `arm.rs`'s comment above the one call site must stop naming \
         import as the door that skips it — and the call itself needs a reason, because with no \
         AST to walk it can only answer `NotAnalysable`."
    );
}
