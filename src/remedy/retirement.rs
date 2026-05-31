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
//! The entry format is `(retired_form, replacement_form, optional_note)`. Do NOT add entries
//! for forms that have not yet been retired — premature entries deceive the
//! substrate.
//!
//! The optional note carries a migration caveat for replacements that need more
//! than a form-swap (e.g. a retired restricted-def whose caller whitelist must be
//! re-expressed as a `{:restricted-to [...]}` metadata-map on the binding). This
//! caveat is carried in the remedy's `note` field so programmatic consumers
//! (LLM agents, IDEs) receive the structured guidance, not just the replacement form.
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
//! | `":wat::core::def-restricted"`    | 241.14 | def-restricted caller whitelist | def + metadata-map `:restricted-to` |
//! | `":wat::core::defn-restricted"`   | 241.14 | defn-restricted (wat macro)     | defn + metadata-map `:restricted-to` |
//! | `":wat::core::try"`               | 241.15 | lowercase try (arc 109 zombie)  | Result/try (PascalCase Type/method) |
//! | `":wat::core::option::expect"`    | 241.15 | lowercase option::expect zombie | Option/expect (PascalCase canonical) |
//! | `":wat::core::result::expect"`    | 241.15 | lowercase result::expect zombie | Result/expect (PascalCase canonical) |

use super::{Remedy, RemedyKind};

/// Explicit retirement-form → replacement-form → optional migration note table.
///
/// Each entry: `(retired, replacement, note)`. HARD CUT stones append entries
/// at ship time. No future-vapor entries.
///
/// The note field carries a migration caveat for replacements that need more than
/// a form-swap; `None` for pure renames. The caveat is surfaced to programmatic
/// consumers via [`Remedy::note`].
pub(crate) const RETIREMENT_TABLE: &[(&str, &str, Option<&str>)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted.
    (":wat::core::struct",            ":wat::core::defstruct", None),
    (":wat::core::struct-restricted", ":wat::core::defstruct",
        Some("re-express the ctor restriction as a `{:restricted-to [...]}` metadata-map, and per-field restrictions as `{:field-metadata {field {:restricted-to [...]}}}`, on the defstruct binding")),
    // Stone 241.9 — defenum replaces enum.
    (":wat::core::enum",              ":wat::core::defenum",   None),
    // Stone 241.11 — defn replaces define.
    (":wat::core::define",            ":wat::core::defn",      None),
    // Stone 242.1 — char (lowercase) replaces Char (per Doctrine 2; scalar types lowercase).
    (":wat::core::Char",              ":wat::core::char",      None),
    // Stone 241.12 — defalias replaces runtime define-alias (native substrate form).
    (":wat::runtime::define-alias",   ":wat::core::defalias",  None),
    // Stone 241.13 — defclause replaces define-dispatch.
    (":wat::core::define-dispatch",   ":wat::core::defclause", None),
    // Stone 241.14 — def + metadata-map replaces def-restricted; defn + metadata-map replaces defn-restricted.
    // The caller whitelist must be re-expressed as a {:restricted-to [...]} metadata-map
    // on the binding — carried in the remedy note for programmatic consumers.
    (":wat::core::def-restricted",    ":wat::core::def",
        Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding")),
    (":wat::core::defn-restricted",   ":wat::core::defn",
        Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding")),
    // Stone 241.15 — zombie purge: arc-109-slice-1j retirements now HARD CUT.
    (":wat::core::try",               ":wat::core::Result/try",    None),
    (":wat::core::option::expect",    ":wat::core::Option/expect", None),
    (":wat::core::result::expect",    ":wat::core::Result/expect", None),
];

/// Look up `needle` in the retirement table.
///
/// Returns `Some(Remedy { kind: Retirement, score: 0, form: replacement, note })`
/// if the needle is a known retired form. Returns `None` if not retired.
///
/// The score for a retirement remedy is always 0 — a direct table hit has no
/// distance; it is an exact match on the retired form. For entries that carry a
/// migration caveat (e.g. `def-restricted` / `defn-restricted`), the `note`
/// field carries the caveat in structured form so programmatic consumers receive
/// it — not just the prose in this module's doc.
pub(super) fn retirement_lookup(needle: &str) -> Option<Remedy> {
    RETIREMENT_TABLE
        .iter()
        .find(|(retired, _, _)| *retired == needle)
        .map(|(_, replacement, note)| Remedy {
            form: replacement.to_string(),
            score: 0,
            kind: RemedyKind::Retirement,
            note: note.map(str::to_string),
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
        for (retired, _, _) in RETIREMENT_TABLE {
            let r = retirement_lookup(retired).unwrap();
            assert_eq!(r.score, 0, "retirement score must be 0 for {}", retired);
        }
    }

    #[test]
    fn def_restricted_retirement_note_is_some() {
        let r = retirement_lookup(":wat::core::def-restricted").unwrap();
        assert!(r.note.is_some(), "def-restricted retirement must carry a migration note");
    }

    #[test]
    fn struct_restricted_retirement_note_is_some() {
        let r = retirement_lookup(":wat::core::struct-restricted").unwrap();
        assert!(r.note.is_some(), "struct-restricted retirement must carry a migration note");
    }
}
