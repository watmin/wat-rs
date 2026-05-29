//! Retirement table — explicit mapping from retired forms to their replacements.
//!
//! ## Why this module exists
//!
//! When the substrate HARD CUTs a form (renames or removes a keyword), the
//! error at the old form should surface the replacement — not just a parse
//! failure. This table is the substrate's explicit memory of its own evolution.
//!
//! ## Scope
//!
//! One public function (re-exported from `mod.rs`): [`retirement_lookup`].
//! One private constant: [`RETIREMENT_TABLE`].
//!
//! The table is an EXPLICIT static mapping. No heuristic matching; no fuzzy
//! lookup. Retirement is a deliberate language history event — the table records
//! it exactly. Only shipped retirements appear here; future-vapor entries are
//! forbidden (per D6).
//!
//! ## Adding entries
//!
//! Each HARD CUT stone appends its retirement entry at the arc's ship time.
//! The entry format is `(retired_form, replacement_form)`. Do NOT add entries
//! for forms that have not yet been retired — premature entries deceive the
//! substrate.
//!
//! ## Arc history
//!
//! | Entry | Stone | Retired | Replacement |
//! |---|---|---|---|
//! | `":wat::core::struct"`            | 241.8 | struct (original) | defstruct |
//! | `":wat::core::struct-restricted"` | 241.8 | struct-restricted  | defstruct |
//! | `":wat::core::enum"`              | 241.9 | enum (original)    | defenum   |

use super::{Remedy, RemedyKind};

/// Explicit retirement-form → replacement-form table.
///
/// Each entry: `(retired, replacement)`. HARD CUT stones append entries
/// at ship time. No future-vapor entries.
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted.
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum.
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 entry added at 241.11 ship time; do NOT pre-emptively add.
];

/// Look up `needle` in the retirement table.
///
/// Returns `Some(Remedy { kind: Retirement, score: 0, form: replacement })`
/// if the needle is a known retired form. Returns `None` if not retired.
///
/// The score for a retirement remedy is always 0 — a direct table hit has no
/// distance; it is an exact match on the retired form.
pub fn retirement_lookup(needle: &str) -> Option<Remedy> {
    RETIREMENT_TABLE
        .iter()
        .find(|(retired, _)| *retired == needle)
        .map(|(_, replacement)| Remedy {
            form: replacement.to_string(),
            score: 0,
            kind: RemedyKind::Retirement,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_retires_to_defstruct() {
        let r = retirement_lookup(":wat::core::struct").unwrap();
        assert_eq!(r.form, ":wat::core::defstruct");
        assert_eq!(r.score, 0);
        assert!(matches!(r.kind, RemedyKind::Retirement));
    }

    #[test]
    fn struct_restricted_retires_to_defstruct() {
        let r = retirement_lookup(":wat::core::struct-restricted").unwrap();
        assert_eq!(r.form, ":wat::core::defstruct");
        assert_eq!(r.score, 0);
        assert!(matches!(r.kind, RemedyKind::Retirement));
    }

    #[test]
    fn enum_retires_to_defenum() {
        let r = retirement_lookup(":wat::core::enum").unwrap();
        assert_eq!(r.form, ":wat::core::defenum");
        assert_eq!(r.score, 0);
        assert!(matches!(r.kind, RemedyKind::Retirement));
    }

    #[test]
    fn unknown_form_returns_none() {
        assert!(retirement_lookup(":wat::core::defstruct").is_none());
        assert!(retirement_lookup(":wat::core::defenum").is_none());
        assert!(retirement_lookup(":wat::core::completely-unknown").is_none());
    }

    #[test]
    fn retirement_score_is_always_zero() {
        for (retired, _) in RETIREMENT_TABLE {
            let r = retirement_lookup(retired).unwrap();
            assert_eq!(r.score, 0, "retirement score must be 0 for {}", retired);
        }
    }
}
