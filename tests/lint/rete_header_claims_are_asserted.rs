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
