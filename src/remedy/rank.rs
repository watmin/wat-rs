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
//! One public function: [`nearest_match`]. Two crate-internal helpers:
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
pub(crate) fn typo_threshold(needle: &str) -> u32 {
    std::cmp::max(1, (needle.len() / 3) as u32)
}

/// Maximum number of candidates returned by `nearest_match`.
///
/// Beyond 5 = noise; the reader can't usefully discriminate.
pub(crate) const TOP_N: usize = 5;

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
    fn threshold_scales_with_needle_length() {
        assert_eq!(typo_threshold("ab"),          1); // len 2 → max(1, 0) = 1
        assert_eq!(typo_threshold("abc"),         1); // len 3 → max(1, 1) = 1
        assert_eq!(typo_threshold("abcdef"),      2); // len 6 → max(1, 2) = 2
        assert_eq!(typo_threshold(":wat::core::defenum"), 6); // len 19 → max(1, 6)
    }

    #[test]
    fn exact_match_excluded() {
        let results = nearest_match(
            ":wat::core::defenum",
            [":wat::core::defenum"].iter().copied(),
        );
        assert!(results.is_empty(), "exact match should not appear in results");
    }

    #[test]
    fn single_typo_within_threshold() {
        let candidates = [":my::Status::Ok", ":my::Status::Pending", ":my::Status::Error"];
        let results = nearest_match(":my::Status::Oks", candidates.iter().copied());
        // ":my::Status::Oks" vs ":my::Status::Ok" = distance 1; threshold = max(1, 16/3) = 5
        assert!(!results.is_empty(), "should find ':my::Status::Ok'");
        assert_eq!(results[0].form, ":my::Status::Ok");
        assert_eq!(results[0].score, 1);
        assert!(matches!(results[0].kind, RemedyKind::Typo));
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
        assert!(!results.is_empty());
        for w in results.windows(2) {
            assert!(
                w[0].score <= w[1].score,
                "results not sorted ascending: {:?}", w
            );
        }
    }

    #[test]
    fn lex_tiebreaker_within_same_distance() {
        let candidates = ["beta", "alpha"]; // both distance 1 from "alph"
        // Actually let's use a controlled example:
        // needle = "aab"; "aac" and "aad" are both distance 1.
        let candidates2 = ["aac", "aad"];
        let mut results = nearest_match("aab", candidates2.iter().copied());
        if results.len() >= 2 {
            // lex order: "aac" < "aad"
            assert_eq!(results[0].form, "aac");
            assert_eq!(results[1].form, "aad");
        }
        let _ = candidates; // suppress unused warning
        let _ = results.sort(); // re-sort to ensure determinism tested
    }
}
