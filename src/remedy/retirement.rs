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
//! One crate-internal function: [`retirement_lookup`]. Called internally by
//! `remedies_for` in `mod.rs`; not re-exported on the module's public surface.
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
//! | `":wat::core::define"`            | 241.11 | define (function binding) | defn |
//! | `":wat::core::Char"`              | 242.1  | Char (PascalCase scalar)  | char (lowercase per Doctrine 2) |
//! | `":wat::runtime::define-alias"`   | 241.12 | runtime macro define-alias | defalias (native substrate form) |
//! | `":wat::core::define-dispatch"`   | 241.13 | dispatch entity kind       | defclause (Stone 237.2)          |

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
    // Stone 241.11 — defn replaces define.
    (":wat::core::define",            ":wat::core::defn"),
    // Stone 242.1 — char (lowercase) replaces Char (per Doctrine 2; scalar types lowercase).
    (":wat::core::Char",              ":wat::core::char"),
    // Stone 241.12 — defalias replaces runtime define-alias (native substrate form).
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
    // Stone 241.13 — defclause replaces define-dispatch.
    (":wat::core::define-dispatch",   ":wat::core::defclause"),
];

/// Look up `needle` in the retirement table.
///
/// Returns `Some(Remedy { kind: Retirement, score: 0, form: replacement })`
/// if the needle is a known retired form. Returns `None` if not retired.
///
/// The score for a retirement remedy is always 0 — a direct table hit has no
/// distance; it is an exact match on the retired form.
pub(super) fn retirement_lookup(needle: &str) -> Option<Remedy> {
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

    fn struct_retirement() -> Remedy {
        retirement_lookup(":wat::core::struct").unwrap()
    }

    fn struct_restricted_retirement() -> Remedy {
        retirement_lookup(":wat::core::struct-restricted").unwrap()
    }

    fn enum_retirement() -> Remedy {
        retirement_lookup(":wat::core::enum").unwrap()
    }

    #[test]
    fn struct_retires_to_defstruct_form() {
        assert_eq!(struct_retirement().form, ":wat::core::defstruct");
    }

    #[test]
    fn struct_retires_with_score_zero() {
        assert_eq!(struct_retirement().score, 0);
    }

    #[test]
    fn struct_retires_with_retirement_kind() {
        assert!(matches!(struct_retirement().kind, RemedyKind::Retirement));
    }

    #[test]
    fn struct_restricted_retires_to_defstruct_form() {
        assert_eq!(struct_restricted_retirement().form, ":wat::core::defstruct");
    }

    #[test]
    fn struct_restricted_retires_with_score_zero() {
        assert_eq!(struct_restricted_retirement().score, 0);
    }

    #[test]
    fn struct_restricted_retires_with_retirement_kind() {
        assert!(matches!(struct_restricted_retirement().kind, RemedyKind::Retirement));
    }

    #[test]
    fn enum_retires_to_defenum_form() {
        assert_eq!(enum_retirement().form, ":wat::core::defenum");
    }

    #[test]
    fn enum_retires_with_score_zero() {
        assert_eq!(enum_retirement().score, 0);
    }

    #[test]
    fn enum_retires_with_retirement_kind() {
        assert!(matches!(enum_retirement().kind, RemedyKind::Retirement));
    }

    #[test]
    fn known_replacement_defstruct_is_not_retired() {
        assert!(retirement_lookup(":wat::core::defstruct").is_none());
    }

    #[test]
    fn known_replacement_defenum_is_not_retired() {
        assert!(retirement_lookup(":wat::core::defenum").is_none());
    }

    #[test]
    fn arbitrary_unknown_form_returns_none() {
        assert!(retirement_lookup(":wat::core::completely-unknown").is_none());
    }

    // rune:complectens(property-over-table) — single contract enforced across all entries; loop is the structure, not multiple claims
    #[test]
    fn retirement_score_is_always_zero() {
        for (retired, _) in RETIREMENT_TABLE {
            let r = retirement_lookup(retired).unwrap();
            assert_eq!(r.score, 0, "retirement score must be 0 for {}", retired);
        }
    }
}
