//! # Remedy — ranked structured error remediation for the wat substrate.
//!
//! ## Why this module exists — the failure class being eliminated
//!
//! The substrate carried `hint: Option<String>` on error variants — a flat prose
//! string at the wrong abstraction layer. Strings are human-facing; programmatic
//! consumers (LLM agents, IDE integrations, telemetry) get no structure. The class
//! to eliminate: *structured remediation expressed as unstructured prose*.
//!
//! Per arc 233 (substrate-errors-as-values doctrine): errors are DATA. This
//! module extends that doctrine to remediation: ranked candidates are DATA.
//! The substrate refuses AND offers ranked candidates with kind annotation.
//!
//! ## What this module owns
//!
//! - [`Remedy`] — a single ranked candidate (form + score + kind)
//! - [`RemedyKind`] — discriminates typo remedies from retirement-table hits
//! - [`nearest_match`] — Levenshtein-ranked candidates from a candidate set
//! - [`remedies_for`] — convenience combinator: retirement (priority) + typo merged
//!
//! ## What this module does NOT own
//!
//! Error construction — each call site decides when to invoke `remedies_for` and
//! what candidate set to provide. This module is purely algorithmic. Per D10
//! (lazy invocation discipline): `remedies_for` is called ONLY at error
//! construction paths, never as a defensive pre-compute.
//!
//! VSA / `coincident?` / vector similarity — not this module's geometry.
//! Edit-distance is the right metric for identifier strings.
//!
//! ## Module layout
//!
//! ```text
//! src/remedy/
//! ├── mod.rs        — this file; public API surface
//! ├── distance.rs   — Levenshtein helper (~100 lines)
//! ├── retirement.rs — explicit retirement-form → replacement table
//! └── rank.rs       — threshold tuning, top-N capping, nearest_match
//! ```

mod distance;
mod retirement;
mod rank;

use retirement::retirement_lookup;
pub use rank::nearest_match;

/// A single ranked remedy offered to the user when their input is rejected.
///
/// Remedies are sorted ascending by `score` (closest first); ties broken
/// lexicographically on `form`. Use [`render_remedies`] to render a slice of
/// remedies as a human-readable "did you mean" section.
///
/// ## Kind semantics
///
/// - [`RemedyKind::Typo`] — edit-distance derived from a candidate set.
///   The `score` field carries the Levenshtein distance.
/// - [`RemedyKind::Retirement`] — explicit retirement-table hit. The substrate
///   has recorded that the needle was a valid form in a prior arc and was
///   HARD CUT to the remedy's form. `score` is always 0 (exact table hit, no
///   distance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remedy {
    /// The candidate form offered as a replacement.
    /// For typo remedies: the nearest known form by edit distance.
    /// For retirement remedies: the explicit replacement from the retirement table.
    pub form: String,
    /// Edit distance from the needle to this candidate.
    /// Always 0 for [`RemedyKind::Retirement`]; ≥ 1 for [`RemedyKind::Typo`].
    pub score: u32,
    /// Discriminates the remedy source: typo vs retirement-table hit.
    pub(crate) kind: RemedyKind,
}

/// Discriminates the source of a [`Remedy`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemedyKind {
    /// Levenshtein-derived from a candidate set — the user likely mistyped.
    Typo,
    /// Explicit retirement-table lookup — the form was valid in a prior arc
    /// and was HARD CUT. The replacement is the current canonical form.
    Retirement,
}

// ─── Ordering ────────────────────────────────────────────────────────────────
//
// Ascending by score; ties broken lexicographically on form.
// RemedyKind does not participate in ordering — kind is metadata,
// not a ranking axis.

impl PartialOrd for Remedy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Remedy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.form.cmp(&other.form))
    }
}

// ─── Display ─────────────────────────────────────────────────────────────────
//
// The Display impl renders a list of remedies as the "did you mean" section
// that gets appended to an error message.
//
// Format rules (per D7):
//   - 0 remedies → empty string (caller omits section)
//   - 1 remedy   → "  did you mean: <form> [<annotation>]"
//   - ≥2 remedies → "  did you mean:\n    <form>  [<annotation>]\n    ..."
//
// Kind annotations:
//   - Typo:       "[typo, distance N]"
//   - Retirement: "[retirement replacement]"

/// Render a slice of remedies as the "did you mean" section.
///
/// Intended for embedding into `fmt::Display` impls on error variants.
/// Returns an empty string when `remedies` is empty — no section rendered.
pub fn render_remedies(remedies: &[Remedy]) -> String {
    match remedies.len() {
        0 => String::new(),
        1 => {
            let r = &remedies[0];
            format!("  did you mean: {} [{}]", r.form, kind_annotation(r))
        }
        _ => {
            let mut out = String::from("  did you mean:");
            for r in remedies {
                out.push_str(&format!("\n    {}  [{}]", r.form, kind_annotation(r)));
            }
            out
        }
    }
}

fn kind_annotation(r: &Remedy) -> String {
    match r.kind {
        RemedyKind::Typo       => format!("typo, distance {}", r.score),
        RemedyKind::Retirement => "retirement replacement".to_string(),
    }
}

// ─── Convenience combinator ───────────────────────────────────────────────────

/// Merge retirement (priority) + typo candidates into a single ranked list.
///
/// Retirement check runs FIRST. If the needle hits the retirement table, that
/// entry leads the result — retirement is an authoritative statement about the
/// form's history, not a distance estimate. Typo candidates follow, de-duplicated
/// against the retirement hit's form (a retirement replacement never appears again
/// as a distance-derived candidate).
///
/// Per D10 (lazy invocation): call ONLY at error construction paths.
pub fn remedies_for<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy> {
    let retirement = retirement_lookup(needle);
    let mut typos = nearest_match(needle, candidates);

    match retirement {
        None => typos,
        Some(ret) => {
            // De-duplicate: remove any typo candidate that matches the retirement form.
            typos.retain(|r| r.form != ret.form);
            let mut combined = vec![ret];
            combined.extend(typos);
            // No re-sort needed: retirement score=0 leads by construction (vec![ret]
            // prepended); typos from `nearest_match` are already sorted ascending by
            // score; exact matches filtered by `nearest_match` so no typo can have
            // score=0. The invariant is structural, not enforced by sort.
            combined
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Remedy ordering ─────────────────────────────────────────────────

    #[test]
    fn lower_score_sorts_first() {
        let a = Remedy { form: "beta".into(), score: 2, kind: RemedyKind::Typo };
        let b = Remedy { form: "alpha".into(), score: 1, kind: RemedyKind::Typo };
        let mut v = vec![a, b];
        v.sort();
        assert_eq!(v[0].score, 1);
    }

    #[test]
    fn lex_tiebreaker_on_equal_score() {
        let a = Remedy { form: "zeta".into(), score: 1, kind: RemedyKind::Typo };
        let b = Remedy { form: "alpha".into(), score: 1, kind: RemedyKind::Typo };
        let mut v = vec![a, b];
        v.sort();
        assert_eq!(v[0].form, "alpha");
    }

    #[test]
    fn retirement_leads_combined_list() {
        // Retirement score is 0 so it always sorts before typos.
        let candidates = [":wat::core::defstruct"]; // close to ":wat::core::struct"
        let remedies = remedies_for(":wat::core::struct", candidates.iter().copied());
        assert!(matches!(remedies[0].kind, RemedyKind::Retirement));
    }

    fn remedies_for_unknown_setup() -> Vec<Remedy> {
        // ":my::Status::Oks" is close to ":my::Status::Ok" (distance 1).
        let candidates = [":my::Status::Ok", ":my::Status::Pending"];
        remedies_for(":my::Status::Oks", candidates.iter().copied())
    }

    #[test]
    fn remedies_for_unknown_needle_first_typo_has_correct_form() {
        assert_eq!(remedies_for_unknown_setup()[0].form, ":my::Status::Ok");
    }

    #[test]
    fn remedies_for_unknown_needle_first_typo_has_typo_kind() {
        assert!(matches!(remedies_for_unknown_setup()[0].kind, RemedyKind::Typo));
    }

    #[test]
    fn no_duplicate_retirement_form_in_typos() {
        // When the retirement form also appears in the candidate iterator,
        // it should NOT appear twice in the output.
        let candidates = [":wat::core::defstruct", ":wat::core::defenum"];
        let remedies = remedies_for(":wat::core::struct", candidates.iter().copied());
        let defstruct_count = remedies.iter().filter(|r| r.form == ":wat::core::defstruct").count();
        assert_eq!(defstruct_count, 1, "retirement form should appear exactly once");
    }

    // ─── combined_remedy_setup: retirement + two typos at different distances ────
    //
    // needle = ":wat::core::struct" (len=18), threshold = max(1, 18/3) = 6.
    // ":wat::core::struXt" = dist 1 (1 substitution: c→X)
    // ":wat::core::strXXt" = dist 2 (2 substitutions: uc→XX)
    // ":wat::core::defstruct" deduped (= retirement form); not in candidates.
    fn combined_remedy_setup() -> Vec<Remedy> {
        let candidates = [":wat::core::struXt", ":wat::core::strXXt"];
        remedies_for(":wat::core::struct", candidates.iter().copied())
    }

    #[test]
    fn combined_retirement_leads() {
        let remedies = combined_remedy_setup();
        assert!(matches!(remedies[0].kind, RemedyKind::Retirement));
    }

    #[test]
    fn combined_retirement_has_score_zero() {
        let remedies = combined_remedy_setup();
        assert_eq!(remedies[0].score, 0);
    }

    #[test]
    fn combined_has_exactly_two_typos() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo)).collect();
        assert_eq!(typos.len(), 2);
    }

    #[test]
    fn combined_typos_sorted_ascending_by_score() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo)).collect();
        for w in typos.windows(2) {
            assert!(w[0].score <= w[1].score);
        }
    }

    #[test]
    fn combined_first_typo_has_score_one() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo)).collect();
        assert_eq!(typos[0].score, 1);
    }

    #[test]
    fn combined_second_typo_has_score_two() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo)).collect();
        assert_eq!(typos[1].score, 2);
    }

    // ─── render_remedies ─────────────────────────────────────────────────

    #[test]
    fn render_empty_is_empty_string() {
        assert_eq!(render_remedies(&[]), "");
    }

    // ─── render_single retirement — 4 focused tests ──────────────────────

    #[test]
    fn render_single_remedy_has_did_you_mean_prefix() {
        let r = Remedy { form: ":wat::core::defstruct".into(), score: 0, kind: RemedyKind::Retirement };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains("did you mean:"), "missing 'did you mean:' prefix");
    }

    #[test]
    fn render_single_remedy_contains_form() {
        let r = Remedy { form: ":wat::core::defstruct".into(), score: 0, kind: RemedyKind::Retirement };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains(":wat::core::defstruct"), "missing form in rendered output");
    }

    #[test]
    fn render_single_retirement_annotation_is_canonical() {
        let r = Remedy { form: ":wat::core::defstruct".into(), score: 0, kind: RemedyKind::Retirement };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains("[retirement replacement]"), "missing retirement annotation");
    }

    #[test]
    fn render_single_remedy_is_one_line() {
        let r = Remedy { form: ":wat::core::defstruct".into(), score: 0, kind: RemedyKind::Retirement };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered.lines().count(), 1, "single remedy should be one line");
    }

    // ─── render_single typo — 4 focused tests ────────────────────────────

    #[test]
    fn render_single_typo_has_did_you_mean_prefix() {
        let r = Remedy { form: ":my::Status::Ok".into(), score: 1, kind: RemedyKind::Typo };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains("did you mean:"), "missing 'did you mean:' prefix");
    }

    #[test]
    fn render_single_typo_contains_form() {
        let r = Remedy { form: ":my::Status::Ok".into(), score: 1, kind: RemedyKind::Typo };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains(":my::Status::Ok"), "missing form in rendered output");
    }

    #[test]
    fn render_single_typo_annotation_includes_distance() {
        let r = Remedy { form: ":my::Status::Ok".into(), score: 1, kind: RemedyKind::Typo };
        let rendered = render_remedies(&[r]);
        assert!(rendered.contains("[typo, distance 1]"), "missing typo annotation with distance");
    }

    #[test]
    fn render_single_typo_is_one_line() {
        let r = Remedy { form: ":my::Status::Ok".into(), score: 1, kind: RemedyKind::Typo };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered.lines().count(), 1, "single typo remedy should be one line");
    }

    #[test]
    fn render_multi_remedy_multi_line() {
        let remedies = vec![
            Remedy { form: ":my::Status::Ok".into(),      score: 1, kind: RemedyKind::Typo },
            Remedy { form: ":my::Status::Oke".into(),     score: 2, kind: RemedyKind::Typo },
        ];
        let rendered = render_remedies(&remedies);
        // Header "  did you mean:" on its own line; candidates on subsequent lines.
        let line_count = rendered.lines().count();
        assert!(line_count >= 3, "multi-remedy should have ≥3 lines; got {}", line_count);
    }

    #[test]
    fn render_remedies_typo_annotation_includes_exact_distance() {
        let r = Remedy { form: ":my::Status::Ok".into(), score: 3, kind: RemedyKind::Typo };
        let rendered = render_remedies(&[r]);
        assert!(
            rendered.contains("[typo, distance 3]"),
            "annotation should read '[typo, distance 3]'; got: {rendered:?}"
        );
    }

}
