//! Arc 144 slice 2 — special-form registry.
//!
//! Arc 255 Stone "the three special-form tables" — this table used to duplicate the
//! intrinsic registry (`#[wat_special_form]`/`#[wat_intrinsic]`, `src/intrinsic/mod.rs`) for
//! ~30 names; those rows are DELETED, not migrated, because `eval_signature_of_defn`
//! (`src/reflect/verbs.rs`) and `eval_lookup_define` (`src/reflect/lookup.rs`) already
//! answer for every registered name before either ever reaches this table (via a
//! registered `@syntax` or non-empty `@arg` entries) — see `build_registry`'s own header
//! comment for the measured detail, including the one BRIEF instruction ("move the sketch
//! to `@syntax`" for the 9 names lacking one) that turned out to be impossible for 6 of the
//! 9 (the `#[wat_intrinsic]` doc parser does not recognize `@syntax` at all).
//!
//! What is left is exactly three names with **no registration site at all** to carry a
//! `@syntax`/`@arg`: `:wat::core::defstruct` (a stdlib macro — the FOURTH-registry fork),
//! and `:wat::core::unquote`/`:wat::core::unquote-splicing` (legal only inside a
//! `quasiquote` template; punctuation, not verbs —
//! `[[NOTE-unquote-is-punctuation-not-a-verb]]`). This table remains their only source for
//! `:wat::runtime::lookup-form`'s `Binding::SpecialForm` step
//! (`src/reflect/lookup.rs`'s step 6, `crate::special_forms::lookup_special_form`) and the
//! reflection surfaces built on it.
//!
//! # Sketch format
//!
//! Each `signature` is a `HolonAST::Bundle` whose first child is the
//! form's head as a Keyword (`HolonAST::keyword(":wat::core::if")`);
//! remaining children are bare-symbol placeholders for the syntactic
//! slots (`HolonAST::symbol("<cond>")`). Repeating slots use `<name>+`
//! (one or more) or `<name>*` (zero or more). The format is honest
//! about structure-not-types: each slot is a symbol naming the slot's
//! role, not a type. Consumers render this to a help string or AST.
//!
//! Forms registered as TypeScheme primitives (e.g., `:wat::core::Vector`,
//! `:wat::kernel::spawn-thread`, `:wat::kernel::send`) do NOT appear
//! here — they are reachable through `lookup_form`'s Primitive branch
//! (slice 3 territory) instead. User-defined wat helpers like
//! `:wat::kernel::run-sandboxed-ast` (defined in `wat/kernel/sandbox.wat`)
//! reach through the UserFunction branch.

use holon::HolonAST;
use std::collections::HashMap;
use std::sync::OnceLock;

/// One special-form entry. Owned data — cloned out at lookup time.
pub struct SpecialFormDef {
    pub name: String,
    pub signature: HolonAST,
    pub doc_string: Option<String>,
}

static REGISTRY: OnceLock<HashMap<String, SpecialFormDef>> = OnceLock::new();

/// Lookup by full keyword name. Returns `Some(&SpecialFormDef)` for
/// every known special form; `None` otherwise.
///
/// The first call lazily initializes the registry; subsequent calls
/// share the same `&'static HashMap` (no Mutex/RwLock — `OnceLock`
/// initialization is the substrate's permitted concurrency primitive
/// per `docs/ZERO-MUTEX.md`).
pub fn lookup_special_form(name: &str) -> Option<&'static SpecialFormDef> {
    REGISTRY.get_or_init(build_registry).get(name)
}

/// Build a `HolonAST::Bundle` whose first child is `head` as a
/// Keyword leaf and remaining children are `slots` as bare Symbol
/// leaves (each slot's name is a literal placeholder string like
/// `"<cond>"` or `"<body>+"`).
fn sketch(head: &str, slots: &[&str]) -> HolonAST {
    let mut children = Vec::with_capacity(1 + slots.len());
    children.push(HolonAST::keyword(head));
    for s in slots {
        children.push(HolonAST::symbol(*s));
    }
    HolonAST::bundle(children)
}

/// Insert one form into the registry. The signature head MUST equal
/// the lookup name; the helper enforces this by reusing `name` in
/// both positions.
fn insert(m: &mut HashMap<String, SpecialFormDef>, name: &str, slots: &[&str]) {
    let signature = sketch(name, slots);
    m.insert(
        name.to_string(),
        SpecialFormDef {
            name: name.to_string(),
            signature,
            doc_string: None,
        },
    );
}

fn build_registry() -> HashMap<String, SpecialFormDef> {
    let mut m = HashMap::new();

    // Arc 255 Stone "the three special-form tables" — the 32 rows that duplicated the
    // intrinsic registry are DELETED, not migrated: `eval_signature_of_defn`
    // (`src/reflect/verbs.rs`) already answers every one of those 32 names from the
    // registry BEFORE it ever reaches this table (23 via a registered `@syntax`, arm
    // `!entry.syntax.is_empty()`; the other 9 already carried `@arg` doc entries and were
    // ALREADY answered by the `!entry.args.is_empty()` arm, ahead of this table's fallback
    // — measured live, not assumed: the pre-edit `signature-of-defn` rendering for those 9
    // matched their `entry.args`, not this file's stale sketch text, which disagreed with
    // the live rendering for 8 of the 9).
    //
    // Only 3 of those 9 (`and`/`or`/`if`, all `#[wat_special_form]`) also gained their own
    // `@syntax` at the registration site, transcribing the LIVE `entry.args` rendering.
    // The other 5 (`Option/expect`/`Option/try`/`Result/expect`/`Result/try`/
    // `:wat::form::matches?`) are declared with
    // `#[wat_intrinsic(...)]`, and that macro's doc parser (`wat_doc::parse`, distinct from
    // `#[wat_special_form]`'s `parse_special_form`) does not recognize `@syntax` at all —
    // attempting it is a hard `compile_error!`, not a rendering mismatch. See the stone's
    // report: this is a real blocker on the BRIEF's literal instruction, not an oversight;
    // it costs nothing behaviourally because `entry.args` already renders these 6
    // identically with no `@syntax` at all, both before and after this deletion.
    //
    // ⛔ CORRECTED by the stone that followed (`:wat::holon::literal` is a special form):
    // that list said SIX and named `:wat::holon::literal` among them. It is now FIVE. The
    // verb captures its argument unevaluated (its own check arm: "DATA captured without
    // evaluation, exactly as `:wat::core::quote`"), so it was misdeclared `#[wat_intrinsic]`;
    // it is `#[wat_special_form]` now and carries a real `@syntax`. The rule the paragraph
    // above states is still right — an intrinsic cannot carry `@syntax`, by design
    // (`@arg` for positional forms, `@syntax` for structural ones) — but this verb was never
    // positional, and the count was reading a mis-kinded row as evidence about the rule.
    //
    // What remains is exactly the DESIGN's third bucket — names with NO registration site
    // to carry a `@syntax`/`@arg` at all:
    insert(&mut m, ":wat::core::defstruct", &["<name>", "[<field> <- <type>]+"]);
    // `unquote` and `unquote-splicing` are only legal INSIDE a
    // quasiquote template; at the top level they return None from
    // expression-position inference (`src/check.rs:3401-3402`).
    // Registered here for uniform reflection.
    insert(&mut m, ":wat::core::unquote", &["<expr>"]);
    insert(&mut m, ":wat::core::unquote-splicing", &["<expr>"]);

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_some_for_defstruct() {
        // Arc 255 Stone "the three special-form tables" — `if` moved to the intrinsic
        // registry's `@arg`-derived answer (already live before this stone; now also
        // carries its own `@syntax`) and its row here was deleted along with the other 31
        // duplicated rows. `defstruct` is one of the three survivors: a stdlib macro with
        // no registration site (the FOURTH-registry fork), so this table is still its only
        // reflection source.
        let def = lookup_special_form(":wat::core::defstruct").expect("defstruct");
        assert_eq!(def.name, ":wat::core::defstruct");
        assert!(def.doc_string.is_none());
        match &def.signature {
            HolonAST::Bundle(children) => {
                // head + 2 slots (name, fields)
                assert_eq!(children.len(), 3);
                // Arc 221 Stone 221.3 (holon-rs fa48b39): HolonAST::keyword() now returns
                // HolonAST::Keyword (stripped of leading colon). as_keyword() returns
                // content WITHOUT colon; as_symbol() → None.
                assert_eq!(
                    children[0].as_keyword(),
                    Some("wat::core::defstruct"),
                    "first child should be the keyword head (HolonAST::Keyword after arc 221 Stone 221.3)"
                );
                // Slot children are still Symbol (HolonAST::symbol(...) unchanged).
                assert_eq!(children[1].as_symbol(), Some("<name>"));
                assert_eq!(children[2].as_symbol(), Some("[<field> <- <type>]+"));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup_special_form(":wat::core::not-a-special-form").is_none());
    }

    #[test]
    fn registry_covers_audited_forms() {
        // Arc 255 Stone "the three special-form tables" — the 32 rows duplicating the
        // intrinsic registry were deleted (see `build_registry`'s header comment); only the
        // three names with no registration site remain.
        for name in [
            ":wat::core::defstruct",
            ":wat::core::unquote",
            ":wat::core::unquote-splicing",
        ] {
            assert!(
                lookup_special_form(name).is_some(),
                "expected {} in registry",
                name
            );
        }
    }
}
