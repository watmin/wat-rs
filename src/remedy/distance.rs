//! Levenshtein edit-distance — Wagner-Fischer table implementation.
//!
//! ## Why this module exists
//!
//! The substrate needs to rank candidate forms by edit distance from a
//! user-typed needle. String edit-distance is the right geometry: structural
//! similarity between keyword paths (`:wat::core::defenum` vs `:wat::core::defenmu`)
//! is captured as a count of single-character insertions, deletions, or
//! substitutions — not semantic distance.
//!
//! This is intentionally NOT VSA / `coincident?` territory. Levenshtein
//! operates on the raw string surface; VSA operates on vector embeddings.
//! The two concerns are orthogonal. At the error-construction layer, the
//! needle and candidates are substrate keyword strings; edit-distance is
//! honest geometry.
//!
//! ## Scope
//!
//! One crate-internal function: [`levenshtein`]. No caching, no Unicode
//! normalization, no SIMD. For identifier strings (~10-80 chars, candidate sets
//! of ≤200 items) the O(n×m) table is negligible.

/// Compute the Levenshtein edit distance between two strings.
///
/// Returns the minimum number of single-character insertions, deletions,
/// and substitutions to transform `a` into `b`. Both strings are treated
/// as sequences of bytes (`chars().count()` semantics via `collect::<Vec<char>>`).
///
/// # Performance note
///
/// For the substrate's use case (identifier keyword strings, ≤200 candidates),
/// the cost is negligible. Profile before optimizing.
pub(crate) fn levenshtein(a: &str, b: &str) -> u32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();

    // Degenerate cases — avoid allocating a table.
    if m == 0 {
        return n as u32;
    }
    if n == 0 {
        return m as u32;
    }

    // Two-row rolling Wagner-Fischer table.
    // `prev[j]` = cost to transform a[0..0] into b[0..j] (baseline: j deletions).
    let mut prev: Vec<u32> = (0..=(n as u32)).collect();
    let mut curr: Vec<u32> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i as u32; // cost to transform a[0..i] into "" = i deletions
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)             // deletion
                .min(curr[j - 1] + 1)           // insertion
                .min(prev[j - 1] + cost);        // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_are_zero() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn empty_a_returns_b_len() {
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn empty_b_returns_a_len() {
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(levenshtein("cat", "bat"), 1);
    }

    #[test]
    fn single_insertion() {
        assert_eq!(levenshtein("defenum", "defenmu"), 2); // transpose = 2 edits
    }

    #[test]
    fn keyword_path_typo() {
        // `:wat::core::defenum` vs `:wat::core::defenmu` — transposition at end
        assert_eq!(levenshtein(":wat::core::defenum", ":wat::core::defenmu"), 2);
    }

    #[test]
    fn variant_path_typo() {
        // `:my::Status::Ok` vs `:my::Status::Oks` — one insertion
        assert_eq!(levenshtein(":my::Status::Ok", ":my::Status::Oks"), 1);
    }
}
