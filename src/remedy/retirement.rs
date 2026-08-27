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
//! | `":wat::core::Uuid/v4"`               | 255 | Uuid v4 constructor, junk-drawer home         | `:wat::uuid::v4` |
//! | `":wat::core::Uuid/v5"`               | 255 | Uuid v5 constructor, junk-drawer home         | `:wat::uuid::v5` |
//! | `":wat::core::Uuid/from-string"`      | 255 | Uuid parse-safe constructor, junk-drawer home | `:wat::uuid::from-string` |
//! | `":wat::core::Uuid/to-string"`        | 255 | Uuid render, junk-drawer home                 | `:wat::uuid::to-string` |
//! | `":wat::core::Uuid/nil"`              | 255 | Uuid nil sentinel, junk-drawer home           | `:wat::uuid::nil` |
//! | `":wat::core::Uuid/version"`          | 255 | Uuid version-nibble accessor, junk-drawer home | `:wat::uuid::version` |
//! | `":wat::core::Uuid/rfc4122-variant?"` | 255 | Uuid variant probe, junk-drawer home          | `:wat::uuid::rfc4122-variant?` |
//! | `":wat::core::regex::matches?"`       | 255 | regex match predicate, junk-drawer home       | `:wat::regex::matches?` |
//! | `":wat::core::List/of"`               | 255 | List constructor's redundant `/of` suffix     | `:wat::core::List` (finishing, not starting) |
//! | `":wat::core::char/of"`               | 255 | char constructor's redundant `/of` suffix     | `:wat::core::char` (finishing, not starting) |
//! | `":wat::core::i64::*"` (17 ops)       | 255 Stone C | per-type i64 verbs, junk-drawer home | `:wat::i64::*` |
//! | `":wat::core::f64::*"` (19 ops)       | 255 Stone C | per-type f64 verbs, junk-drawer home | `:wat::f64::*` (`max-of`/`min-of` also change calling convention — see the table) |
//! | `":wat::core::bigint::*"` (6 ops)     | 255 Stone D | per-type bigint verbs, junk-drawer home   | `:wat::bigint::*` |
//! | `":wat::core::rational::*"` (5 ops) + `":wat::core::rational/*"` (2 ops) | 255 Stone D | per-type rational verbs, junk-drawer home | `:wat::rational::*` (the two slash-form accessors also become `::` verbs — see the table) |
//! | `":wat::core::PersistentMap/*"` (8 ops) | 255 Stone E-i | per-type PersistentMap verbs, junk-drawer home | `:wat::map::*` (the UNMARKED home — never moves again once the persistent-backend swap lands) |
//! | `":wat::core::HashMap/*"` (8 ops)       | 255 Stone E-i | per-type HashMap verbs, junk-drawer home       | `:wat::hashmap::*` (the flavor-marked home) |
//! | `":wat::core::PersistentVector/*"` (6 ops) | 255 Stone E-ii | per-type PersistentVector verbs, junk-drawer home | `:wat::vector::*` (the UNMARKED home — never moves again once the persistent-backend swap lands) |
//! | `":wat::core::Vector/*"` (7 ops)        | 255 Stone E-ii | per-type Vector verbs, junk-drawer home        | `:wat::vec::*` (the flavor-marked home; `extend` is Vector-only, no PersistentVector twin) |
//! | `":wat::core::HashSet/*"` (4 ops)       | 255 Stone E-iii | per-type HashSet verbs, junk-drawer home     | `:wat::hashset::*` (the flavor-marked home; `:wat::set::` stays free for the persistent sibling) |
//! | `":wat::core::List/*"` (5 ops)          | 255 Stone E-iii | per-type List verbs, junk-drawer home        | `:wat::linkedlist::*` (the flavor-marked home; `:wat::list::` stays free for the persistent sibling) |
//! | `":wat::core::keyword/*"` (5 ops)       | 255 Stone E-iv | keyword verbs, junk-drawer home; the LAST scalar without a home | `:wat::keyword::*` (the plain, unmarked home — `keyword` has only one flavor) |

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
        note: Some("as a TYPE CONSTRUCTOR, rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same (Vector :- [T]) value. To materialize a seqable/Stream into a Vector (arc 118.2a), use `(:wat::core::into [] coll)` instead") },
    // Arc 109 slice 1g — list retired (was a duplicate of vec; both produced Vec<T>).
    RetirementEntry { retired: ":wat::core::list", replacement: ":wat::core::Vector",
        note: Some("rename `:wat::core::list` → `:wat::core::Vector` (was a duplicate of vec; arc 109 slice 1g); substrate produces the same (Vector :- [T]) value") },
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
    // Arc 255 — "the four that got homes": ten Rust-implemented verbs move off the
    // `:wat::core::` junk-drawer to the namespace each earns. Handler bodies untouched;
    // name-only. `List/of` and `char/of` FINISH a migration — both types already had their
    // own constructor names, and these two verbs simply drop the redundant `/of` suffix.
    RetirementEntry { retired: ":wat::core::Uuid/v4",               replacement: ":wat::uuid::v4",               note: None },
    RetirementEntry { retired: ":wat::core::Uuid/v5",               replacement: ":wat::uuid::v5",               note: None },
    RetirementEntry { retired: ":wat::core::Uuid/from-string",      replacement: ":wat::uuid::from-string",      note: None },
    RetirementEntry { retired: ":wat::core::Uuid/to-string",        replacement: ":wat::uuid::to-string",        note: None },
    RetirementEntry { retired: ":wat::core::Uuid/nil",              replacement: ":wat::uuid::nil",              note: None },
    RetirementEntry { retired: ":wat::core::Uuid/version",          replacement: ":wat::uuid::version",          note: None },
    RetirementEntry { retired: ":wat::core::Uuid/rfc4122-variant?", replacement: ":wat::uuid::rfc4122-variant?", note: None },
    RetirementEntry { retired: ":wat::core::regex::matches?",       replacement: ":wat::regex::matches?",        note: None },
    RetirementEntry { retired: ":wat::core::List/of",               replacement: ":wat::core::List",
        note: Some("finishing, not starting — every other collection type is already its own constructor") },
    RetirementEntry { retired: ":wat::core::char/of",               replacement: ":wat::core::char",
        note: Some("finishing, not starting — the type is already named `:wat::core::char`; the constructor drops the redundant `/of`") },
    // Arc 255 Stone C — "the numerics get their homes": the per-type i64/f64 verbs
    // move off the `:wat::core::` junk-drawer to their own top-level namespace
    // (`:wat::i64::*` / `:wat::f64::*`), adjacent to `:wat::string::*`. Name-only;
    // handler bodies untouched (DESIGN-STONE-the-numerics-get-their-homes.md).
    // `max-of` / `min-of` are the one pair where the replacement ALSO changes
    // calling convention: the retired form took a single `(Vector :- [f64])`
    // argument; the replacement is variadic (`(:wat::f64::max-of 1.0 2.0 3.0)`).
    RetirementEntry { retired: ":wat::core::i64::+",           replacement: ":wat::i64::+",           note: None },
    RetirementEntry { retired: ":wat::core::i64::-",           replacement: ":wat::i64::-",           note: None },
    RetirementEntry { retired: ":wat::core::i64::*",           replacement: ":wat::i64::*",           note: None },
    RetirementEntry { retired: ":wat::core::i64::/",           replacement: ":wat::i64::/",           note: None },
    RetirementEntry { retired: ":wat::core::i64::<",           replacement: ":wat::i64::<",           note: None },
    RetirementEntry { retired: ":wat::core::i64::<=",          replacement: ":wat::i64::<=",          note: None },
    RetirementEntry { retired: ":wat::core::i64::>",           replacement: ":wat::i64::>",           note: None },
    RetirementEntry { retired: ":wat::core::i64::>=",          replacement: ":wat::i64::>=",          note: None },
    RetirementEntry { retired: ":wat::core::i64::=",           replacement: ":wat::i64::=",           note: None },
    RetirementEntry { retired: ":wat::core::i64::not=",        replacement: ":wat::i64::not=",        note: None },
    RetirementEntry { retired: ":wat::core::i64::mod",         replacement: ":wat::i64::mod",         note: None },
    RetirementEntry { retired: ":wat::core::i64::quot",        replacement: ":wat::i64::quot",        note: None },
    RetirementEntry { retired: ":wat::core::i64::rem",         replacement: ":wat::i64::rem",         note: None },
    RetirementEntry { retired: ":wat::core::i64::to-bigint",   replacement: ":wat::i64::to-bigint",   note: None },
    RetirementEntry { retired: ":wat::core::i64::to-f64",      replacement: ":wat::i64::to-f64",      note: None },
    RetirementEntry { retired: ":wat::core::i64::to-rational", replacement: ":wat::i64::to-rational", note: None },
    RetirementEntry { retired: ":wat::core::i64::to-string",   replacement: ":wat::i64::to-string",   note: None },
    RetirementEntry { retired: ":wat::core::f64::+",           replacement: ":wat::f64::+",           note: None },
    RetirementEntry { retired: ":wat::core::f64::-",           replacement: ":wat::f64::-",           note: None },
    RetirementEntry { retired: ":wat::core::f64::*",           replacement: ":wat::f64::*",           note: None },
    RetirementEntry { retired: ":wat::core::f64::/",           replacement: ":wat::f64::/",           note: None },
    RetirementEntry { retired: ":wat::core::f64::<",           replacement: ":wat::f64::<",           note: None },
    RetirementEntry { retired: ":wat::core::f64::<=",          replacement: ":wat::f64::<=",          note: None },
    RetirementEntry { retired: ":wat::core::f64::>",           replacement: ":wat::f64::>",           note: None },
    RetirementEntry { retired: ":wat::core::f64::>=",          replacement: ":wat::f64::>=",          note: None },
    RetirementEntry { retired: ":wat::core::f64::=",           replacement: ":wat::f64::=",           note: None },
    RetirementEntry { retired: ":wat::core::f64::not=",        replacement: ":wat::f64::not=",        note: None },
    RetirementEntry { retired: ":wat::core::f64::abs",         replacement: ":wat::f64::abs",         note: None },
    RetirementEntry { retired: ":wat::core::f64::clamp",       replacement: ":wat::f64::clamp",       note: None },
    RetirementEntry { retired: ":wat::core::f64::max",         replacement: ":wat::f64::max",         note: None },
    RetirementEntry { retired: ":wat::core::f64::max-of",      replacement: ":wat::f64::max-of",
        note: Some("calling convention changed, not just the name: the retired form took a single `(Vector :- [f64])` argument; `:wat::f64::max-of` is variadic — `(:wat::f64::max-of 1.0 2.0 3.0)`, no Vector wrapper") },
    RetirementEntry { retired: ":wat::core::f64::min",         replacement: ":wat::f64::min",         note: None },
    RetirementEntry { retired: ":wat::core::f64::min-of",      replacement: ":wat::f64::min-of",
        note: Some("calling convention changed, not just the name: the retired form took a single `(Vector :- [f64])` argument; `:wat::f64::min-of` is variadic — `(:wat::f64::min-of 1.0 2.0 3.0)`, no Vector wrapper") },
    RetirementEntry { retired: ":wat::core::f64::round",       replacement: ":wat::f64::round",       note: None },
    RetirementEntry { retired: ":wat::core::f64::to-i64",      replacement: ":wat::f64::to-i64",      note: None },
    RetirementEntry { retired: ":wat::core::f64::to-string",   replacement: ":wat::f64::to-string",   note: None },
    // Stone D (arc 255) — bigint/rational, the numeric tower's last two verb families.
    RetirementEntry { retired: ":wat::core::bigint::+",             replacement: ":wat::bigint::+",             note: None },
    RetirementEntry { retired: ":wat::core::bigint::-",             replacement: ":wat::bigint::-",             note: None },
    RetirementEntry { retired: ":wat::core::bigint::*",             replacement: ":wat::bigint::*",             note: None },
    RetirementEntry { retired: ":wat::core::bigint::/",             replacement: ":wat::bigint::/",             note: None },
    RetirementEntry { retired: ":wat::core::bigint::to-f64",        replacement: ":wat::bigint::to-f64",        note: None },
    RetirementEntry { retired: ":wat::core::bigint::to-rational",   replacement: ":wat::bigint::to-rational",   note: None },
    RetirementEntry { retired: ":wat::core::rational::+",           replacement: ":wat::rational::+",           note: None },
    RetirementEntry { retired: ":wat::core::rational::-",           replacement: ":wat::rational::-",           note: None },
    RetirementEntry { retired: ":wat::core::rational::*",           replacement: ":wat::rational::*",           note: None },
    RetirementEntry { retired: ":wat::core::rational::/",           replacement: ":wat::rational::/",           note: None },
    RetirementEntry { retired: ":wat::core::rational::to-f64",      replacement: ":wat::rational::to-f64",      note: None },
    RetirementEntry { retired: ":wat::core::rational/numerator",    replacement: ":wat::rational::numerator",
        note: Some("the slash-form accessor becomes an ordinary `::` verb (arc 255's `:wat::core::Uuid/v4 -> :wat::uuid::v4` precedent), not just a namespace move") },
    RetirementEntry { retired: ":wat::core::rational/denominator",  replacement: ":wat::rational::denominator",
        note: Some("the slash-form accessor becomes an ordinary `::` verb (arc 255's `:wat::core::Uuid/v4 -> :wat::uuid::v4` precedent), not just a namespace move") },
    // Arc 255 Stone E-i — "the maps get their homes": PersistentMap moves to the UNMARKED
    // `:wat::map::*` home (it never moves again once the persistent-backend swap lands, "probably
    // a week or two" out per the builder); HashMap moves to the flavor-marked `:wat::hashmap::*`
    // home. Both flavors survive — this is a spelling migration, not a backend decision. Each
    // slash-form op becomes an ordinary `::` verb (same shape as the Uuid/v4 and rational/numerator
    // precedents above). Name-only; handler bodies untouched (they already lived in
    // `src/collection/eval.rs`, unmoved by this stone).
    RetirementEntry { retired: ":wat::core::PersistentMap/length",         replacement: ":wat::map::length",         note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/empty?",         replacement: ":wat::map::empty?",         note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/contains-key?",  replacement: ":wat::map::contains-key?",  note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/get",            replacement: ":wat::map::get",            note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/assoc",          replacement: ":wat::map::assoc",          note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/dissoc",         replacement: ":wat::map::dissoc",         note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/keys",           replacement: ":wat::map::keys",           note: None },
    RetirementEntry { retired: ":wat::core::PersistentMap/values",         replacement: ":wat::map::values",         note: None },
    RetirementEntry { retired: ":wat::core::HashMap/length",               replacement: ":wat::hashmap::length",     note: None },
    RetirementEntry { retired: ":wat::core::HashMap/empty?",               replacement: ":wat::hashmap::empty?",     note: None },
    RetirementEntry { retired: ":wat::core::HashMap/contains-key?",        replacement: ":wat::hashmap::contains-key?", note: None },
    RetirementEntry { retired: ":wat::core::HashMap/get",                  replacement: ":wat::hashmap::get",        note: None },
    RetirementEntry { retired: ":wat::core::HashMap/assoc",                replacement: ":wat::hashmap::assoc",      note: None },
    RetirementEntry { retired: ":wat::core::HashMap/dissoc",               replacement: ":wat::hashmap::dissoc",     note: None },
    RetirementEntry { retired: ":wat::core::HashMap/keys",                 replacement: ":wat::hashmap::keys",       note: None },
    RetirementEntry { retired: ":wat::core::HashMap/values",               replacement: ":wat::hashmap::values",     note: None },
    // Arc 255 Stone E-ii — "the vectors get their homes": PersistentVector moves to the
    // UNMARKED `:wat::vector::*` home (it never moves again once the persistent-backend swap
    // lands); Vector moves to the flavor-marked `:wat::vec::*` home. Both flavors survive —
    // this is a spelling migration, not a backend decision. Verb sets are NOT symmetric:
    // `extend` exists only on Vector. Name-only; handler bodies untouched (they already lived
    // in `src/collection/eval.rs`, unmoved by this stone).
    RetirementEntry { retired: ":wat::core::PersistentVector/length",   replacement: ":wat::vector::length",   note: None },
    RetirementEntry { retired: ":wat::core::PersistentVector/empty?",   replacement: ":wat::vector::empty?",   note: None },
    RetirementEntry { retired: ":wat::core::PersistentVector/contains?", replacement: ":wat::vector::contains?", note: None },
    RetirementEntry { retired: ":wat::core::PersistentVector/get",      replacement: ":wat::vector::get",      note: None },
    RetirementEntry { retired: ":wat::core::PersistentVector/conj",     replacement: ":wat::vector::conj",     note: None },
    RetirementEntry { retired: ":wat::core::PersistentVector/concat",   replacement: ":wat::vector::concat",   note: None },
    RetirementEntry { retired: ":wat::core::Vector/length",             replacement: ":wat::vec::length",      note: None },
    RetirementEntry { retired: ":wat::core::Vector/empty?",             replacement: ":wat::vec::empty?",      note: None },
    RetirementEntry { retired: ":wat::core::Vector/contains?",          replacement: ":wat::vec::contains?",   note: None },
    RetirementEntry { retired: ":wat::core::Vector/get",                replacement: ":wat::vec::get",         note: None },
    RetirementEntry { retired: ":wat::core::Vector/conj",               replacement: ":wat::vec::conj",        note: None },
    RetirementEntry { retired: ":wat::core::Vector/concat",             replacement: ":wat::vec::concat",      note: None },
    RetirementEntry { retired: ":wat::core::Vector/extend",             replacement: ":wat::vec::extend",      note: None },
    // Arc 255 Stone E-iii — "set + list get their homes": both HashSet and List are the
    // copy-on-write flavor (same axis-side as HashMap/Vector), so BOTH take a MARKED name —
    // `:wat::set::`/`:wat::list::` stay free for the persistent-backed siblings the builder has
    // ruled are coming, same reason `:wat::map::`/`:wat::vector::` stayed free above. Verb sets
    // are NOT symmetric: HashSet has no `get` (its "get-by-equality" is `contains?`); List has
    // no `concat`/`extend`. Name-only; handler bodies untouched (they already lived in
    // `src/collection/eval.rs`, unmoved by this stone).
    RetirementEntry { retired: ":wat::core::HashSet/length",   replacement: ":wat::hashset::length",   note: None },
    RetirementEntry { retired: ":wat::core::HashSet/empty?",   replacement: ":wat::hashset::empty?",   note: None },
    RetirementEntry { retired: ":wat::core::HashSet/contains?", replacement: ":wat::hashset::contains?", note: None },
    RetirementEntry { retired: ":wat::core::HashSet/conj",     replacement: ":wat::hashset::conj",     note: None },
    RetirementEntry { retired: ":wat::core::List/length",      replacement: ":wat::linkedlist::length",   note: None },
    RetirementEntry { retired: ":wat::core::List/empty?",      replacement: ":wat::linkedlist::empty?",   note: None },
    RetirementEntry { retired: ":wat::core::List/contains?",   replacement: ":wat::linkedlist::contains?", note: None },
    RetirementEntry { retired: ":wat::core::List/get",         replacement: ":wat::linkedlist::get",      note: None },
    RetirementEntry { retired: ":wat::core::List/conj",        replacement: ":wat::linkedlist::conj",     note: None },
    // Arc 255 Stone E-iv — "keyword gets its home": the LAST scalar without one. One flavor,
    // so the plain unmarked name (contrast E-iii's hashset/linkedlist, both marked). Name-only;
    // handler bodies untouched (`eval_keyword_to_string`/`eval_keyword_from_string` stay in
    // `runtime.rs`; `eval_keyword_to_symbol`/`eval_keyword_to_type_form`/
    // `eval_keyword_to_type_form_colon` stay in `edn/render.rs`).
    RetirementEntry { retired: ":wat::core::keyword/to-string",         replacement: ":wat::keyword::to-string",         note: None },
    RetirementEntry { retired: ":wat::core::keyword/from-string",       replacement: ":wat::keyword::from-string",       note: None },
    RetirementEntry { retired: ":wat::core::keyword/to-symbol",         replacement: ":wat::keyword::to-symbol",         note: None },
    RetirementEntry { retired: ":wat::core::keyword/to-type-form",      replacement: ":wat::keyword::to-type-form",      note: None },
    RetirementEntry { retired: ":wat::core::keyword/to-type-form-colon", replacement: ":wat::keyword::to-type-form-colon", note: None },
    // Arc 255 Stone F — the `String/` verbs leave the `extend-type`-generated instance-method
    // namespace. `:wat::core::String` (the bare TYPE, no trailing `/`) is UNCHANGED and remains
    // the home `extend-type` mints real instance methods into (e.g. `String/tag`); only these
    // five plain functions — never methods — move. Name-only; handler bodies untouched
    // (`intrinsic/string.rs`'s `eval_string_{concat,starts_with,ends_with,contains,empty}`,
    // four of which already backed the old spelling via `runtime.rs`'s deleted alias arms).
    RetirementEntry { retired: ":wat::core::String/concat",       replacement: ":wat::string::concat",       note: None },
    RetirementEntry { retired: ":wat::core::String/starts-with?", replacement: ":wat::string::starts-with?", note: None },
    RetirementEntry { retired: ":wat::core::String/ends-with?",   replacement: ":wat::string::ends-with?",   note: None },
    RetirementEntry { retired: ":wat::core::String/contains?",    replacement: ":wat::string::contains?",    note: None },
    RetirementEntry { retired: ":wat::core::String/empty?",       replacement: ":wat::string::empty?",       note: None },
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

/// Every retired-form name, in table order — walks `RETIREMENT_TABLE` directly.
///
/// Arc 255 STONE-retirement-table-becomes-mechanism: the ONLY caller is the
/// end-to-end reachability gate (`tests/cli/retirement_table_reachable.rs`, bridged
/// through `crate::remedy::retirement_table_names` and `wat::retirement_table_names_for_gate`
/// in `lib.rs`), which must iterate the table ITSELF rather than a hand-maintained
/// copy of its names — a copy would be exactly the defect this stone fixes, one
/// level up (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`). Not for
/// production use.
pub(super) fn retirement_table_names() -> Vec<&'static str> {
    RETIREMENT_TABLE.iter().map(|e| e.retired).collect()
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
