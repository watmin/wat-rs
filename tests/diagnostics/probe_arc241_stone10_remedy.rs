//! FM 2-bis probe for Stone 241.10 — `src/remedy/` + ranked-remedy schema upgrade.
//!
//! The substrate's `hint: Option<String>` field upgrades to `remedies: Vec<Remedy>`.
//! Error messages render ranked candidates with kind annotation:
//!   - typo cases: `[typo, distance N]`
//!   - retirement cases: `[retirement replacement]`
//!
//! At HEAD: error messages do NOT carry the canonical "did you mean" phrasing
//! with structured kind annotations. The 241.8 hard-coded retirement prose
//! ("use ':wat::core::defstruct' instead") exists in `reason:` strings but is
//! UNSTRUCTURED, single-target, and pre-built-in (not produced by the remedy
//! home). The probe targets the STRUCTURED FORMAT, not the substitution content.
//!
//! HEAD-disconfirmation map:
//! - C01: typo case — "did you mean :wat::core::defenum" + "[typo, distance" annotation
//! - C02: retirement case — "did you mean :wat::core::defstruct" + "[retirement replacement]"
//! - C03: ranked multi-candidate case — multiple "[typo, distance" annotations
//! - C04: no-remedy case — error message does NOT carry "did you mean" (passes trivially at HEAD)
//! - C05: Display single-remedy single-line format
//! - C06: Display multi-remedy multi-line format
//! - C07: Display retirement-kind canonical annotation present
//! - C08: Display threshold honored — far-typo does NOT get remedy
//!
//! 7 of 8 contracts disconfirm cleanly at HEAD; C04 is the post-stone semantic
//! contract (consistent with arc 241 probe precedent).
//!
//! WAT fixtures: tests/diagnostics/probe_arc241_stone10_remedy_c{01,02,03,04,07,08}_bad.wat
//! C05 shares c02's fixture; C06 shares c03's fixture.
//!
//! Run: `cargo nextest run --release -E 'binary(diagnostics)' -F probe_arc241_stone10_remedy`

use wat::freeze::startup_from_file;

/// Display-format the error message (what the user sees).
fn display_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── Contracts 1-3: typo + retirement remedy production ───────────────────────
//
// Scope honesty: 241.10 wires remedies into EXISTING error paths. Unknown form
// HEADS at top level currently no-op at HEAD (substrate doesn't reject typo'd
// :wat::core::xxx keywords). Adding that rejection would be scope-expansion;
// 241.10 does not own it. The typo-detection contracts therefore target error
// paths that ALREADY error today: type-unknown errors (typo on a referenced
// type) and HARD-CUT retirement arms (struct, struct-restricted, enum).

// rune:complectens(assertion-sequence) — three properties of one rendered error msg; "did you mean"/form/annotation are the structured-remedy contract
#[test]
fn contract_01_typo_remedy_on_variant_constructor() {
    // User declares :my::Status with variants; then typos the constructor
    // :my::Status::Oks (distance 1 from :Ok). At HEAD: ReturnTypeMismatch
    // (constructor unknown → bare :wat::core::keyword type). Verified
    // disconfirming path per Stone 241.9 probe C08 precedent.
    // Post-stone: error contains 'did you mean :my::Status::Ok [typo, distance 1]'.
    // Uses :test::pick (non-main) — main signature retirement doesn't apply.
    // Fixture: probe_arc241_stone10_remedy_c01_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c01_bad.wat");
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/diagnostics/probe_arc241_stone10_remedy_c01_bad.wat:2:49: :test::pick: body produces :wat::core::keyword; signature declares :my::Status\n  did you mean:\n    :my::Status::Ok  [typo, distance 1]\n    :my::Status::Error  [typo, distance 5]\n",
        "variant-typo case: 'did you mean' with distance annotation"
    );
}

// rune:complectens(assertion-sequence) — three properties of one rendered error msg; "did you mean"/form/annotation are the structured-remedy contract
#[test]
fn contract_02_retirement_remedy_for_hard_cut_form() {
    // Legacy `:wat::core::struct` retired at Stone 241.8. The 241.8 hand-written
    // reason: string already names `:wat::core::defstruct` in prose, but the
    // STRUCTURED canonical phrasing ('did you mean: ... [retirement replacement]')
    // is the 241.10 shape.
    // Fixture: probe_arc241_stone10_remedy_c02_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c02_bad.wat");
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/diagnostics/probe_arc241_stone10_remedy_c02_bad.wat:1:2: malformed :wat::core::struct form: ':wat::core::struct' is retired (Stone 241.8)\n  did you mean: :wat::core::defstruct [replaces a retired form]\n",
        "retirement case: 'did you mean: :wat::core::defstruct [replaces a retired form]'"
    );
}

#[test]
fn contract_03_ranked_multi_candidate_variant_typo() {
    // Declare enum with two variants close in spelling; typo a constructor
    // close to both. Post-stone: ranked output names multiple candidates.
    // Fixture: probe_arc241_stone10_remedy_c03_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c03_bad.wat");
    let typo_annotation_count = msg.matches("[typo, distance").count();
    assert!(
        typo_annotation_count >= 2,
        "multi-candidate case should produce ≥2 '[typo, distance' annotations; got {}:\n{}",
        typo_annotation_count, msg
    );
}

// ─── Contract 4: no remedy case ───────────────────────────────────────────────

#[test]
fn contract_04_no_remedy_for_distant_unknown() {
    // `:wat::core::xyzzy` is far from any real form. No candidate within threshold.
    // Post-stone: error message renders without "did you mean" section.
    // Fixture: probe_arc241_stone10_remedy_c04_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c04_bad.wat");
    assert_eq!(
        msg,
        "<startup succeeded — no error to display>",
        "distant-unknown case should NOT produce 'did you mean'"
    );
}

// ─── Contracts 5-7: Display formatting ────────────────────────────────────────

// rune:complectens(assertion-sequence) — two properties of one extracted line; probe startup cost-of-split exceeds value
#[test]
fn contract_05_single_remedy_single_line_format() {
    // Single remedy → inline single-line "did you mean: <form> [annotation]".
    // Uses retirement path (always produces error post-241.8).
    // Fixture: probe_arc241_stone10_remedy_c02_bad.wat (same as C02)
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c02_bad.wat");
    let line = msg.lines().find(|l| l.contains("did you mean"))
        .unwrap_or_else(|| panic!("expected 'did you mean' line; got:\n{}", msg));
    assert_eq!(
        line,
        "  did you mean: :wat::core::defstruct [replaces a retired form]",
        "single-remedy line: exact format with form and annotation"
    );
}

#[test]
fn contract_06_multi_remedy_multi_line_format() {
    // Multiple remedies → "did you mean:" header on its own line; ranked candidates
    // each on their own subsequent line. Uses variant-constructor typo path.
    // Fixture: probe_arc241_stone10_remedy_c03_bad.wat (same as C03)
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c03_bad.wat");
    let lines: Vec<&str> = msg.lines().collect();
    let header_idx = lines.iter().position(|l| {
        l.trim_end().ends_with("did you mean:") || (l.contains("did you mean:") && !l.contains("[typo"))
    });
    assert!(
        header_idx.is_some(),
        "multi-remedy should have 'did you mean:' header on its own line; got:\n{}",
        msg
    );
}

#[test]
fn contract_07_retirement_kind_annotation_canonical() {
    // The retirement kind annotation is the LITERAL string `[retirement replacement]`.
    // No abbreviations; no variants. Exact phrase per D7.
    // Fixture: probe_arc241_stone10_remedy_c07_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c07_bad.wat");
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/diagnostics/probe_arc241_stone10_remedy_c07_bad.wat:1:2: malformed :wat::core::struct-restricted form: ':wat::core::struct-restricted' is retired (Stone 241.8); use ':wat::core::defstruct' with metadata-map: re-express ctor restriction as `{:restricted-to [<prefix-kw>...]}` and per-field restrictions as `{:field-metadata {field {:restricted-to [<prefix-kw>...]}}}` on the defstruct binding\n  did you mean: :wat::core::defstruct [replaces a retired form] — re-express the ctor restriction as `{:restricted-to [<prefix-kw>...]}` and per-field restrictions as `{:field-metadata {field {:restricted-to [<prefix-kw>...]}}}` on the defstruct binding\n",
        "retirement annotation must be exact '[replaces a retired form]'"
    );
}

// ─── Contract 8: threshold honored (distant-typo gets NO remedy) ──────────────

#[test]
fn contract_08_threshold_filters_far_typos() {
    // `:wat::core::definitelywrong` is far from any real form (distance >> needle.len()/3).
    // Post-stone: no remedy offered (threshold filter).
    // Fixture: probe_arc241_stone10_remedy_c08_bad.wat
    let msg = display_err("tests/diagnostics/probe_arc241_stone10_remedy_c08_bad.wat");
    assert_eq!(
        msg,
        "<startup succeeded — no error to display>",
        "distant-typo above threshold should not produce remedy"
    );
}
