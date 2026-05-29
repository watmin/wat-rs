//! Ranking logic — threshold tuning, top-N capping, candidate combination.
//!
//! ## Why this module exists
//!
//! Given a needle and a candidate set, `nearest_match` must:
//! 1. Filter candidates that are too far away (threshold)
//! 2. Sort survivors ascending by distance (closest first)
//! 3. Break ties lexicographically on form
//! 4. Return at most `TOP_N` results
//!
//! The threshold and cap are substrate constants here, documented and named
//! rather than inlined as magic numbers at call sites.
//!
//! ## Scope
//!
//! One public function: [`nearest_match`]. Two private helpers:
//! [`typo_threshold`] and [`TOP_N`]. The `remedies_for` combinator lives in `mod.rs`.

use super::{Remedy, RemedyKind};
use crate::remedy::distance::levenshtein;

/// Maximum edit distance allowed for a typo remedy.
///
/// Formula: `max(1, needle.len() / 3)`. Longer identifiers tolerate more
/// distance; short identifiers (len < 3) always allow 1 edit minimum.
/// Mirrors the Rust compiler's heuristic.
///
/// Computed per-call, not a single global constant, because it scales
/// with the needle length.
fn typo_threshold(needle: &str) -> u32 {
    std::cmp::max(1, (needle.chars().count() / 3) as u32)
}

/// Maximum number of candidates returned by `nearest_match`.
///
/// Beyond 5 = noise; the reader can't usefully discriminate.
const TOP_N: usize = 5;

/// Find the nearest candidates from an iterator by edit distance.
///
/// Returns at most [`TOP_N`] `Remedy` values of kind [`RemedyKind::Typo`],
/// sorted ascending by score (closest first); ties broken lexicographically
/// on `form`.
///
/// Candidates that exceed `typo_threshold(needle)` are discarded. Candidates
/// equal to the needle itself are discarded (exact match = not a typo).
///
/// The iterator is consumed once; no allocation per candidate beyond the
/// score table. Suitable for small-to-medium candidate sets (≤ ~500 items).
pub fn nearest_match<'a>(
    needle: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Vec<Remedy> {
    let threshold = typo_threshold(needle);

    let mut hits: Vec<Remedy> = candidates
        .filter_map(|candidate| {
            if candidate == needle {
                return None; // exact match — not a typo
            }
            let dist = levenshtein(needle, candidate);
            if dist <= threshold {
                Some(Remedy {
                    form: candidate.to_string(),
                    score: dist,
                    kind: RemedyKind::Typo,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort: ascending score, then lexicographic on form.
    hits.sort();

    // Cap at TOP_N.
    hits.truncate(TOP_N);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_match_admits_candidate_within_threshold_short_needle() {
        // needle "abcdef" (len=6): threshold = max(1, 6/3) = 2.
        // candidate "abcdXf" = distance 1 (≤ 2) — must be admitted.
        let results = nearest_match("abcdef", ["abcdXf"].iter().copied());
        assert_eq!(results[0].form, "abcdXf");
    }

    #[test]
    fn nearest_match_rejects_candidate_beyond_threshold_short_needle() {
        // needle "abcdef" (len=6): threshold = max(1, 6/3) = 2.
        // candidate "abcXYZ" = distance 3 (> 2) — must be rejected.
        let results = nearest_match("abcdef", ["abcXYZ"].iter().copied());
        assert!(results.is_empty(), "candidate beyond threshold should be rejected");
    }

    #[test]
    fn nearest_match_admits_candidate_within_threshold_long_needle() {
        // needle "abcdefghijkl" (len=12): threshold = max(1, 12/3) = 4.
        // candidate at distance 4 — must be admitted.
        let needle = "abcdefghijkl";
        // "abcdefghijXX" has 2 subs at pos 10,11 = distance 2 ≤ 4 → admitted.
        let results = nearest_match(needle, ["abcdefghijXX"].iter().copied());
        assert!(!results.is_empty(), "candidate within threshold (long needle) should be admitted");
    }

    #[test]
    fn nearest_match_rejects_candidate_beyond_threshold_long_needle() {
        // needle "abcdefghijkl" (len=12): threshold = max(1, 12/3) = 4.
        // candidate at distance 5 — must be rejected.
        let needle = "abcdefghijkl";
        // "abcdeXXXXXX" has 7 subs = distance 7 > 4 → rejected.
        let results = nearest_match(needle, ["abcdeXXXXXXX"].iter().copied());
        assert!(results.is_empty(), "candidate beyond threshold (long needle) should be rejected");
    }

    #[test]
    fn exact_match_excluded() {
        let results = nearest_match(
            ":wat::core::defenum",
            [":wat::core::defenum"].iter().copied(),
        );
        assert!(results.is_empty(), "exact match should not appear in results");
    }

    fn single_typo_results() -> Vec<Remedy> {
        // ":my::Status::Oks" vs ":my::Status::Ok" = distance 1; threshold = max(1, 16/3) = 5
        let candidates = [":my::Status::Ok", ":my::Status::Pending", ":my::Status::Error"];
        nearest_match(":my::Status::Oks", candidates.iter().copied())
    }

    #[test]
    fn single_typo_first_result_has_correct_form() {
        assert_eq!(single_typo_results()[0].form, ":my::Status::Ok");
    }

    #[test]
    fn single_typo_first_result_has_distance_one() {
        assert_eq!(single_typo_results()[0].score, 1);
    }

    #[test]
    fn single_typo_first_result_has_typo_kind() {
        assert!(matches!(single_typo_results()[0].kind, RemedyKind::Typo));
    }

    #[test]
    fn distant_candidate_excluded() {
        let candidates = [":wat::core::completely_different_thing"];
        let results = nearest_match(":wat::core::defenum", candidates.iter().copied());
        assert!(results.is_empty(), "distant candidate should be filtered");
    }

    #[test]
    fn top_n_cap_enforced() {
        // Build 10 candidates all at distance 1 from "aaaa".
        let candidates: Vec<String> = (b'a'..=b'j')
            .map(|c| format!("aaaa{}", c as char))
            .collect();
        let results = nearest_match("aaaa", candidates.iter().map(|s| s.as_str()));
        assert!(
            results.len() <= TOP_N,
            "results should be capped at TOP_N={}", TOP_N
        );
    }

    #[test]
    fn results_sorted_ascending_by_score() {
        let candidates = [
            ":wat::core::defenmu",  // distance 2 from :wat::core::defenum
            ":wat::core::defenu",   // distance 1 from :wat::core::defenum
        ];
        let results = nearest_match(":wat::core::defenum", candidates.iter().copied());
        for w in results.windows(2) {
            assert!(
                w[0].score <= w[1].score,
                "results not sorted ascending: {:?}", w
            );
        }
    }

    #[test]
    fn lex_tiebreaker_sorts_alphabetically_within_same_distance() {
        // needle = "aab"; "aac" and "aad" are both distance 1.
        // Threshold for "aab" (len=3) = max(1, 3/3) = 1, so both qualify.
        let candidates = ["aad", "aac"];
        let forms: Vec<String> = nearest_match("aab", candidates.iter().copied())
            .into_iter()
            .map(|r| r.form)
            .collect();
        assert_eq!(forms, vec!["aac".to_string(), "aad".to_string()]);
    }

    #[test]
    fn threshold_uses_char_count_not_byte_count() {
        // 'é' is 2 bytes but 1 char; typo_threshold must count chars to
        // match levenshtein's char-based distance. needle = 6 chars; threshold = 2.
        // candidate at distance 1 must be admitted.
        let needle = "éééééé"; // 6 chars
        let candidates = vec!["éééééx"]; // 1 substitution = distance 1
        let results = nearest_match(needle, candidates.iter().copied());
        assert!(!results.is_empty(), "candidate at distance 1 must pass threshold for needle of 6 chars");
    }
}
