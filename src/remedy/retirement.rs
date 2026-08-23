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
//! it exactly. Future-vapor entries are forbidden — only shipped retirements
//! appear here.
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
//! | `":wat::core::struct-restricted"` | 241.8 | struct-restricted  | defstruct + metadata-map `{:restricted-to / :field-metadata}` |
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
//! | `":wat::core::Record::def"`             | 293.2  | base record decl macro          | defrecord (`:wat::core::defrecord`)  |
//! | `":wat::holon::Record::def"`      | 293.2  | holonic record decl macro       | defrecord (`:wat::holon::defrecord`) |
//! | `":wat::core::foldr"`             | 118.B6b | right fold (reverse+foldl wearing a Haskell name) | reduce (`:wat::core::reduce f init (:wat::core::reverse coll)`) |

use super::{Remedy, RemedyKind};

/// One retirement-table row: a retired form, its current replacement, and an
/// optional migration caveat. Named fields make a column swap a compile concern,
/// not a test-caught accident.
struct RetirementEntry {
    retired: &'static str,
    replacement: &'static str,
    note: Option<&'static str>,
}

/// Explicit retirement-form → replacement-form → optional migration note table.
///
/// Each entry carries `retired`, `replacement`, and `note`. HARD CUT stones append
/// entries at ship time. No future-vapor entries.
///
/// The note field carries a migration caveat for replacements that need more than
/// a form-swap; `None` for pure renames. The caveat is surfaced to programmatic
/// consumers via [`Remedy::note`].
const RETIREMENT_TABLE: &[RetirementEntry] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted.
    RetirementEntry { retired: ":wat::core::struct",            replacement: ":wat::core::defstruct", note: None },
    RetirementEntry { retired: ":wat::core::struct-restricted", replacement: ":wat::core::defstruct",
        note: Some("re-express the ctor restriction as `{:restricted-to [<prefix-kw>...]}` and per-field restrictions as `{:field-metadata {field {:restricted-to [<prefix-kw>...]}}}` on the defstruct binding") },
    // Stone 241.9 — defenum replaces enum.
    RetirementEntry { retired: ":wat::core::enum",              replacement: ":wat::core::defenum",   note: None },
    // Stone 241.11 — defn replaces define.
    RetirementEntry { retired: ":wat::core::define",            replacement: ":wat::core::defn",      note: None },
    // Stone 242.1 — char (lowercase) replaces Char (per Doctrine 2; scalar types lowercase).
    RetirementEntry { retired: ":wat::core::Char",              replacement: ":wat::core::char",      note: None },
    // Stone 241.12 — defalias replaces runtime define-alias (native substrate form).
    RetirementEntry { retired: ":wat::runtime::define-alias",   replacement: ":wat::core::defalias",  note: None },
    // Stone 241.13 — defclause replaces define-dispatch.
    RetirementEntry { retired: ":wat::core::define-dispatch",   replacement: ":wat::core::defclause", note: None },
    // Stone 241.14 — def + metadata-map replaces def-restricted; defn + metadata-map replaces defn-restricted.
    // The caller whitelist must be re-expressed as a {:restricted-to [...]} metadata-map
    // on the binding — carried in the remedy note for programmatic consumers.
    RetirementEntry { retired: ":wat::core::def-restricted",    replacement: ":wat::core::def",
        note: Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding") },
    RetirementEntry { retired: ":wat::core::defn-restricted",   replacement: ":wat::core::defn",
        note: Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding") },
    // Stone 241.15 — zombie purge: arc-109-slice-1j retirements now HARD CUT.
    RetirementEntry { retired: ":wat::core::try",               replacement: ":wat::core::Result/try",    note: None },
    RetirementEntry { retired: ":wat::core::option::expect",    replacement: ":wat::core::Option/expect", note: None },
    RetirementEntry { retired: ":wat::core::result::expect",    replacement: ":wat::core::Result/expect", note: None },
    // Arc 293.2-rename — defrecord replaces Record::def (the aggregate trio's final names).
    RetirementEntry { retired: ":wat::core::Record::def",        replacement: ":wat::core::defrecord",  note: None },
    RetirementEntry { retired: ":wat::holon::Record::def", replacement: ":wat::holon::defrecord", note: None },
    // Arc 293 K3-revise — to-struct + $struct are RETIRED. Projection is ONE-WAY UP (never
    // down to the impure tier). A surface now emits the PAIR: $core-record + $holon-record.
    // `to-struct` → use `:wat::core::to-record` (portable EDN) or `:wat::holon::to-record`
    //               (portable EDN + VSA hologram) to project up to the tier you need.
    // `$struct`   → no replacement — the type no longer exists; you do not need it.
    RetirementEntry { retired: ":wat::core::to-struct", replacement: ":wat::core::to-record",
        note: Some("projection is ONE-WAY UP (AGGREGATE-MODEL.md § to-record, 2026-06-29): choose :wat::core::to-record for portable EDN or :wat::holon::to-record for EDN + VSA hologram") },
    // Arc 296 remediation collapse — arc 109 / arc 170 prose-hint fns absorbed into the table.
    // Each entry was previously a prose `:hint` emitted by check.rs; now a structured Remedy.
    // Arc 109 slice 1f — vec retired (verb-equals-type playbook). Arc 118.2a note appended:
    // the CONSTRUCTOR use (`(vec :T 1 2 3)`) still redirects to `Vector`; the newer, more
    // common reason someone reaches for `vec` post-118.2a is clojure's "coerce a seqable/
    // Stream into a Vector" idiom — that's `(:wat::core::into [] coll)` (ratified: no new
    // name; `into []` is clojure's own materializer).
    RetirementEntry { retired: ":wat::core::vec", replacement: ":wat::core::Vector",
        note: Some("as a TYPE CONSTRUCTOR, rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same Vec<T> value. To materialize a seqable/Stream into a Vector (arc 118.2a), use `(:wat::core::into [] coll)` instead") },
    // Arc 109 slice 1g — list retired (was a duplicate of vec; both produced Vec<T>).
    RetirementEntry { retired: ":wat::core::list", replacement: ":wat::core::Vector",
        note: Some("rename `:wat::core::list` → `:wat::core::Vector` (was a duplicate of vec; arc 109 slice 1g); substrate produces the same Vec<T> value") },
    // Arc 109 slice 1g — tuple retired (verb-equals-type playbook).
    RetirementEntry { retired: ":wat::core::tuple", replacement: ":wat::core::Tuple",
        note: Some("rename `:wat::core::tuple` → `:wat::core::Tuple` (verb-equals-type, arc 109 slice 1g); the `:(T,U,V)` type spelling is ALSO retired (arc 109 \"the comma dies in the reader\") — use `(:wat::core::Tuple :- [T U V])`") },
    // Arc 109 slice 1h — bare `Some` retired (callable heads must be FQDN keywords).
    RetirementEntry { retired: "Some", replacement: ":wat::core::Some",
        note: Some("rename `(Some x)` → `(:wat::core::Some x)` at constructor sites; rename `((Some v) ...)` → `((:wat::core::Some v) ...)` at match-pattern sites (arc 109 slice 1h)") },
    // Arc 109 slice 1h — bare `:None` retired (substrate-provided keywords live under `:wat::core::*`).
    RetirementEntry { retired: ":None", replacement: ":wat::core::None",
        note: Some("rename `:None` → `:wat::core::None` at value-position sites; rename `(:None ...)` → `(:wat::core::None ...)` at match-pattern sites (arc 109 slice 1h)") },
    // Arc 109 slice 1i — bare `Ok` retired (callable heads must be FQDN keywords).
    RetirementEntry { retired: "Ok", replacement: ":wat::core::Ok",
        note: Some("rename `(Ok x)` → `(:wat::core::Ok x)` at constructor sites; rename `((Ok v) ...)` → `((:wat::core::Ok v) ...)` at match-pattern sites (arc 109 slice 1i)") },
    // Arc 109 slice 1i — bare `Err` retired (callable heads must be FQDN keywords).
    RetirementEntry { retired: "Err", replacement: ":wat::core::Err",
        note: Some("rename `(Err e)` → `(:wat::core::Err e)` at constructor sites; rename `((Err _e) ...)` → `((:wat::core::Err _e) ...)` at match-pattern sites (arc 109 slice 1i)") },
    // Arc 118.B6b — foldr retired: it was `reverse`+`foldl` wearing a name borrowed from
    // Haskell, where the verb is distinct only because it is LAZY, a property strict wat
    // cannot have. The operation is spelled from verbs that already exist; `reduce` already IS
    // `foldl` (`wat/seq.wat:308`), so nothing is renamed TO `reduce` — `foldr` simply stops
    // being a word.
    RetirementEntry { retired: ":wat::core::foldr", replacement: ":wat::core::reduce",
        note: Some("wat is STRICT, so a right fold is `(:wat::core::reduce f init (:wat::core::reverse coll))` — `foldr` was `reverse`+`foldl` wearing a name borrowed from Haskell, where the verb is distinct only because it is LAZY (arc 118.B6b)") },
];

/// Look up `needle` in the retirement table.
///
/// Returns `Some(Remedy)` for a known retired form — `form` is the replacement,
/// `note` carries any migration caveat, and `score()` is `0` (an exact table hit,
/// not a fuzzy distance). Returns `None` if `needle` is not retired.
///
/// For entries that carry a migration caveat (e.g. `def-restricted` / `defn-restricted`),
/// the `note` field carries the caveat in structured form so programmatic consumers receive
/// it — not just the prose in this module's doc.
pub(super) fn retirement_lookup(needle: &str) -> Option<Remedy> {
    RETIREMENT_TABLE
        .iter()
        .find(|e| e.retired == needle)
        .map(|e| Remedy {
            form: e.replacement.to_string(),
            kind: RemedyKind::Retirement,
            note: e.note.map(str::to_string),
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
        assert_eq!(struct_retirement().score(), 0);
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
        assert_eq!(struct_restricted_retirement().score(), 0);
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
        assert_eq!(enum_retirement().score(), 0);
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
        for entry in RETIREMENT_TABLE {
            let r = retirement_lookup(entry.retired).unwrap();
            assert_eq!(r.score(), 0, "retirement score must be 0 for {}", entry.retired);
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

    #[test]
    fn defn_restricted_retirement_note_is_some() {
        let r = retirement_lookup(":wat::core::defn-restricted").unwrap();
        assert!(r.note.is_some(), "defn-restricted retirement must carry a migration note");
    }
}
