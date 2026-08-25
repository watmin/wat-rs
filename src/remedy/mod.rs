//! vigilatum: 2026-05-31T08:24:29Z — vigilia 8-spell L1+L2=0
//!
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
//! - [`Remedy`] — a single ranked candidate (form + kind + note; `score()` derived from kind)
//! - [`RemedyKind`] — discriminates typo remedies from retirement-table hits
//! - [`nearest_matches`] — Levenshtein-ranked candidates from a candidate set
//! - [`remedies_for`] — convenience combinator: retirement (priority) + typo merged
//!
//! ## What this module does NOT own
//!
//! Error construction — each call site decides when to invoke `remedies_for` and
//! what candidate set to provide. This module is purely algorithmic.
//! Lazy invocation discipline: `remedies_for` is called ONLY at error
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
//! └── rank.rs       — threshold tuning, top-N capping, nearest_matches
//! ```

mod distance;
mod retirement;
mod rank;

use retirement::retirement_lookup;
pub use rank::nearest_matches;

/// Bridge for the end-to-end retirement-table reachability gate — see
/// `retirement::retirement_table_names`'s doc for why this exists and who the only
/// caller is. Re-bridged as `wat::retirement_table_names_for_gate` in `lib.rs` since
/// this module is `pub(crate)` and the gate lives in an external integration-test crate.
pub(crate) fn retirement_table_names() -> Vec<&'static str> {
    retirement::retirement_table_names()
}

/// A single ranked remedy offered to the user when their input is rejected.
///
/// Remedies are sorted ascending by `score()` (closest first); ties broken
/// lexicographically on `form`. Use [`render_remedies`] to render a slice of
/// remedies as a human-readable "did you mean" section.
///
/// ## Kind semantics
///
/// - [`RemedyKind::Typo`] — edit-distance derived from a candidate set.
///   The distance is carried inside the variant.
/// - [`RemedyKind::Retirement`] — explicit retirement-table hit. The substrate
///   has recorded that the needle was a valid form in a prior arc and was
///   HARD CUT to the remedy's form. Score is always 0 (exact table hit, no
///   distance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remedy {
    /// The candidate form offered as a replacement.
    /// For typo remedies: the nearest known form by edit distance.
    /// For retirement remedies: the explicit replacement from the retirement table.
    pub form: String,
    /// Discriminates the remedy source; for typos, carries the edit distance.
    pub(crate) kind: RemedyKind,
    /// Optional migration caveat for replacements that need more than a form-swap
    /// (e.g. a retired restricted-def whose whitelist must be re-expressed as a
    /// `{:restricted-to [...]}` metadata-map). `None` for pure renames + all typos.
    pub note: Option<String>,
}

impl Remedy {
    /// Ranking score: the Levenshtein distance for a typo; `0` for a retirement
    /// (an exact table hit — distance zero — which sorts ahead of every typo).
    pub fn score(&self) -> u32 {
        match self.kind {
            RemedyKind::Typo(distance) => distance.get(),
            RemedyKind::Retirement => 0,
        }
    }
}

/// Discriminates the source of a [`Remedy`].
///
/// Variant declaration order IS the Eq-consistency tiebreaker in `Remedy`'s `Ord`
/// (`Typo` before `Retirement`). The order carries ZERO ranking meaning — `score()`
/// + `form` decide all real cases — but DO NOT reorder these variants: it would
///   silently change tie resolution between otherwise-identical remedies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RemedyKind {
    /// Levenshtein-derived from a candidate set — the user likely mistyped.
    /// Carries the edit distance (always ≥ 1; exact matches are filtered upstream).
    Typo(std::num::NonZeroU32),
    /// Explicit retirement-table lookup — the form was valid in a prior arc and was
    /// HARD CUT. The replacement is the current canonical form. No distance: an exact
    /// table hit, not a fuzzy match.
    Retirement,
}

// ─── Ordering ────────────────────────────────────────────────────────────────
//
// Ranking is by score (ascending) then form (lexicographic). kind and note are
// appended as final tiebreakers ONLY for Eq-consistency (std contract: a==b iff
// cmp==Equal). They carry ZERO ranking meaning — score+form decide all real cases;
// kind+note only break ties between otherwise-identical remedies, which essentially
// never occurs in practice.

impl PartialOrd for Remedy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Remedy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score()
            .cmp(&other.score())
            .then_with(|| self.form.cmp(&other.form))
            // Final tiebreakers: kind, then note — carry ZERO ranking meaning
            // (score+form decide all real cases); present solely so the total
            // order is consistent with the derived Eq (std contract: a==b iff
            // cmp==Equal). Without them, two remedies equal on score+form but
            // differing in kind/note would compare Equal yet be Eq-unequal.
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.note.cmp(&other.note))
    }
}

// ─── Arc 296 D1 — structured EDN form ────────────────────────────────────────
//
// `Remedy` gets a `ToEdn` impl so error serializers can embed remedies as a
// structured `Vector` of tagged maps rather than a `render_remedies()` prose blob.

impl crate::edn::contract::ToEdn for Remedy {
    /// `#wat.kernel/Remedy {:form "…" :kind :typo|:retirement :score N :note "…"|nil}`
    ///
    /// `:kind` is a keyword (`:typo` or `:retirement`) derived from `RemedyKind`.
    /// `:score` is the integer Levenshtein distance (0 for retirement hits).
    /// `:note` is the migration caveat string, or `nil` when `None`.
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::{edn_int, edn_kw, edn_str, edn_tag};
        use wat_edn::OwnedValue;
        let kind_kw = match self.kind {
            RemedyKind::Typo(_) => edn_kw("typo"),
            RemedyKind::Retirement => edn_kw("retirement"),
        };
        let note_val = match &self.note {
            Some(n) => edn_str(n),
            None => OwnedValue::Nil,
        };
        edn_tag("Remedy", OwnedValue::Map(vec![
            (edn_kw("form"), edn_str(&self.form)),
            (edn_kw("kind"), kind_kw),
            (edn_kw("score"), edn_int(self.score() as i64)),
            (edn_kw("note"), note_val),
        ]))
    }
}

/// Serialize a slice of remedies as a `Vector` of `#wat.kernel/Remedy` tagged maps.
///
/// Returns an empty `Vector` (`[]`) for an empty slice so the EDN field is always
/// structurally consistent — never a String, never absent. Used by the serializers
/// for `ReturnTypeMismatch`, `MalformedForm`, and `MalformedVariant`.
pub(crate) fn remedies_to_edn(remedies: &[Remedy]) -> wat_edn::OwnedValue {
    use crate::edn::contract::ToEdn;
    wat_edn::OwnedValue::Vector(remedies.iter().map(|r| r.to_edn()).collect())
}

// ─── Display ─────────────────────────────────────────────────────────────────
//
// The Display impl renders a list of remedies as the "did you mean" section
// that gets appended to an error message.
//
// Format rules:
//   - 0 remedies → empty string (caller omits section)
//   - 1 remedy   → "  did you mean: <form> [<annotation>]"
//   - ≥2 remedies → "  did you mean:\n    <form>  [<annotation>]\n    ..."
//
// Kind annotations:
//   - Typo:       "[typo, distance N]"
//   - Retirement: "[replaces a retired form]"

/// Render a slice of remedies as the "did you mean" section.
///
/// Intended for embedding into `fmt::Display` impls on error variants.
/// Returns an empty string when `remedies` is empty — no section rendered.
///
/// When a remedy carries `Some(note)`, the note is appended to that remedy's
/// rendered line (e.g. `… [replaces a retired form] — <note>`).
pub fn render_remedies(remedies: &[Remedy]) -> String {
    match remedies.len() {
        0 => String::new(),
        1 => {
            let r = &remedies[0];
            let mut line = format!("  did you mean: {} [{}]", r.form, kind_annotation(r));
            line.push_str(&note_suffix(r));
            line
        }
        _ => {
            let mut out = String::from("  did you mean:");
            for r in remedies {
                let mut entry = format!("\n    {}  [{}]", r.form, kind_annotation(r));
                entry.push_str(&note_suffix(r));
                out.push_str(&entry);
            }
            out
        }
    }
}

fn kind_annotation(r: &Remedy) -> String {
    match r.kind {
        RemedyKind::Typo(distance) => format!("typo, distance {distance}"),
        RemedyKind::Retirement     => "replaces a retired form".to_string(),
    }
}

/// The ` — <note>` suffix for a remedy that carries a migration caveat; empty when None.
fn note_suffix(r: &Remedy) -> String {
    match &r.note {
        Some(note) => format!(" — {note}"),
        None => String::new(),
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
/// Lazy invocation discipline — call ONLY at error construction paths.
pub fn remedies_for<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy> {
    let retirement = retirement_lookup(needle);
    let mut typos = nearest_matches(needle, candidates);

    match retirement {
        None => typos,
        Some(ret) => {
            // De-duplicate: remove any typo candidate that matches the retirement form.
            typos.retain(|r| r.form != ret.form);
            let mut combined = vec![ret];
            combined.extend(typos);
            // No re-sort needed: retirement score=0 leads by construction (vec![ret]
            // prepended); typos from `nearest_matches` are already sorted ascending by
            // score; exact matches filtered by `nearest_matches` so no typo can have
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
        let a = Remedy { form: "beta".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(2).unwrap()), note: None };
        let b = Remedy { form: "alpha".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let mut v = [a, b];
        v.sort();
        assert_eq!(v[0].score(), 1);
    }

    #[test]
    fn lex_tiebreaker_on_equal_score() {
        let a = Remedy { form: "zeta".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let b = Remedy { form: "alpha".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let mut v = [a, b];
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
        assert!(matches!(remedies_for_unknown_setup()[0].kind, RemedyKind::Typo(_)));
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
        assert_eq!(remedies[0].score(), 0);
    }

    #[test]
    fn combined_has_exactly_two_typos() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo(_))).collect();
        assert_eq!(typos.len(), 2);
    }

    #[test]
    fn combined_typos_sorted_ascending_by_score() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo(_))).collect();
        for w in typos.windows(2) {
            assert!(w[0].score() <= w[1].score());
        }
    }

    #[test]
    fn combined_first_typo_has_score_one() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo(_))).collect();
        assert_eq!(typos[0].score(), 1);
    }

    #[test]
    fn combined_second_typo_has_score_two() {
        let remedies = combined_remedy_setup();
        let typos: Vec<&Remedy> = remedies.iter().filter(|r| matches!(r.kind, RemedyKind::Typo(_))).collect();
        assert_eq!(typos[1].score(), 2);
    }

    // ─── render_remedies ─────────────────────────────────────────────────

    #[test]
    fn render_empty_is_empty_string() {
        assert_eq!(render_remedies(&[]), "");
    }

    // ─── render_single retirement — 4 focused tests ──────────────────────

    #[test]
    fn render_single_remedy_has_did_you_mean_prefix() {
        let r = Remedy { form: ":wat::core::defstruct".into(), kind: RemedyKind::Retirement, note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :wat::core::defstruct [replaces a retired form]");
    }

    #[test]
    fn render_single_remedy_contains_form() {
        let r = Remedy { form: ":wat::core::defstruct".into(), kind: RemedyKind::Retirement, note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :wat::core::defstruct [replaces a retired form]");
    }

    #[test]
    fn render_single_retirement_annotation_is_canonical() {
        let r = Remedy { form: ":wat::core::defstruct".into(), kind: RemedyKind::Retirement, note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :wat::core::defstruct [replaces a retired form]");
    }

    #[test]
    fn render_single_remedy_is_one_line() {
        let r = Remedy { form: ":wat::core::defstruct".into(), kind: RemedyKind::Retirement, note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered.lines().count(), 1, "single remedy should be one line");
    }

    // ─── render_single typo — 4 focused tests ────────────────────────────

    #[test]
    fn render_single_typo_has_did_you_mean_prefix() {
        let r = Remedy { form: ":my::Status::Ok".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :my::Status::Ok [typo, distance 1]");
    }

    #[test]
    fn render_single_typo_contains_form() {
        let r = Remedy { form: ":my::Status::Ok".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :my::Status::Ok [typo, distance 1]");
    }

    #[test]
    fn render_single_typo_annotation_includes_distance() {
        let r = Remedy { form: ":my::Status::Ok".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :my::Status::Ok [typo, distance 1]");
    }

    #[test]
    fn render_single_typo_is_one_line() {
        let r = Remedy { form: ":my::Status::Ok".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered.lines().count(), 1, "single typo remedy should be one line");
    }

    #[test]
    fn render_multi_remedy_multi_line() {
        let remedies = vec![
            Remedy { form: ":my::Status::Ok".into(),  kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()), note: None },
            Remedy { form: ":my::Status::Oke".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(2).unwrap()), note: None },
        ];
        let rendered = render_remedies(&remedies);
        // Header "  did you mean:" on its own line; candidates on subsequent lines.
        let line_count = rendered.lines().count();
        assert!(line_count >= 3, "multi-remedy should have ≥3 lines; got {}", line_count);
    }

    #[test]
    fn render_remedies_typo_annotation_includes_exact_distance() {
        let r = Remedy { form: ":my::Status::Ok".into(), kind: RemedyKind::Typo(std::num::NonZeroU32::new(3).unwrap()), note: None };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :my::Status::Ok [typo, distance 3]");
    }

    #[test]
    fn render_remedy_with_note_appends_note_suffix() {
        let r = Remedy {
            form: ":wat::core::defstruct".into(),
            kind: RemedyKind::Retirement,
            note: Some("X".into()),
        };
        let rendered = render_remedies(&[r]);
        assert_eq!(rendered, "  did you mean: :wat::core::defstruct [replaces a retired form] — X");
    }

    #[test]
    fn render_multi_remedy_note_suffix_appears_on_noted_entry() {
        // Covers note_suffix's second call site: the multi-branch loop in render_remedies.
        let remedies = vec![
            Remedy {
                form: ":wat::core::defstruct".into(),
                kind: RemedyKind::Retirement,
                note: Some("migrate the ctor restriction".into()),
            },
            Remedy {
                form: ":wat::core::defenum".into(),
                kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()),
                note: None,
            },
        ];
        let rendered = render_remedies(&remedies);
        assert_eq!(rendered, "  did you mean:\n    :wat::core::defstruct  [replaces a retired form] — migrate the ctor restriction\n    :wat::core::defenum  [typo, distance 1]");
    }

    // ─── Arc 296 D1 — ToEdn impl + remedies_to_edn ──────────────────────

    #[test]
    fn remedy_to_edn_typo_is_wat_kernel_remedy_tagged() {
        use crate::edn::contract::ToEdn;
        let r = Remedy {
            form: ":my::Status::Ok".into(),
            kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()),
            note: None,
        };
        let edn = r.to_edn();
        let s = wat_edn::write(&edn);
        assert_eq!(s, r#"#wat.kernel/Remedy {:form ":my::Status::Ok" :kind :typo :score 1 :note nil}"#);
        // Must be valid EDN.
        wat_edn::parse_owned(&s).expect("must be valid EDN");
    }

    #[test]
    fn remedy_to_edn_retirement_kind_is_keyword() {
        use crate::edn::contract::ToEdn;
        let r = Remedy {
            form: ":wat::core::defstruct".into(),
            kind: RemedyKind::Retirement,
            note: None,
        };
        let edn = r.to_edn();
        let s = wat_edn::write(&edn);
        assert_eq!(s, r#"#wat.kernel/Remedy {:form ":wat::core::defstruct" :kind :retirement :score 0 :note nil}"#);
    }

    #[test]
    fn remedy_to_edn_note_some_is_string() {
        use crate::edn::contract::ToEdn;
        let r = Remedy {
            form: ":wat::core::defstruct".into(),
            kind: RemedyKind::Retirement,
            note: Some("update ctor restrictions".into()),
        };
        let edn = r.to_edn();
        let s = wat_edn::write(&edn);
        assert_eq!(s, r#"#wat.kernel/Remedy {:form ":wat::core::defstruct" :kind :retirement :score 0 :note "update ctor restrictions"}"#);
    }

    #[test]
    fn remedies_to_edn_empty_slice_is_empty_vector() {
        let edn = remedies_to_edn(&[]);
        assert!(
            matches!(edn, wat_edn::OwnedValue::Vector(ref v) if v.is_empty()),
            "remedies_to_edn([]) must be an empty Vector; got: {:?}",
            edn
        );
    }

    #[test]
    fn remedies_to_edn_nonempty_produces_tagged_remedy_items() {
        let remedies = vec![
            Remedy {
                form: ":my::Status::Ok".into(),
                kind: RemedyKind::Typo(std::num::NonZeroU32::new(1).unwrap()),
                note: None,
            },
            Remedy {
                form: ":my::Status::Okay".into(),
                kind: RemedyKind::Typo(std::num::NonZeroU32::new(2).unwrap()),
                note: None,
            },
        ];
        let edn = remedies_to_edn(&remedies);
        let s = wat_edn::write(&edn);
        assert_eq!(s, r#"[#wat.kernel/Remedy {:form ":my::Status::Ok" :kind :typo :score 1 :note nil} #wat.kernel/Remedy {:form ":my::Status::Okay" :kind :typo :score 2 :note nil}]"#);
        // Must be a Vector.
        assert!(
            matches!(edn, wat_edn::OwnedValue::Vector(ref v) if v.len() == 2),
            "remedies_to_edn must be a Vector with 2 items; got: {:?}",
            edn
        );
        wat_edn::parse_owned(&s).expect("must be valid EDN");
    }

}
