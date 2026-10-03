//! THE `:wat::gen::` SURFACE IS DOCUMENTED, AND THE DOC CANNOT SILENTLY DRIFT FROM IT.
//!
//! `docs/GENERATIVE-TESTING.md` is the design record for `wat/gen.wat`. It carries a hand-written
//! table of the library's verbs — and a hand-maintained mirror of a real surface is the exact
//! defect shape that document spends a section warning about:
//!
//! > *"`deftest` structurally removed a bug the old shape had. The script version summed its laws
//! > by hand … three laws fell out of the total while the suite still reported
//! > `laws=21 checked=325 violations=0`."*
//!
//! The suite fixed that for itself and the doc then re-introduced it — a table of 27 verbs, a law
//! count in three places, a cost table, none of them behind a red build. Found by `circumspicere`
//! in a doc-review vigilia, 2026-08-26, as the *generator* of the drift class the inward wards had
//! each reported one instance of.
//!
//! ## What this gate can and cannot do — stated, because the gap is the point
//!
//! It closes DRIFT IN BOTH DIRECTIONS on the verb surface:
//!
//! - **A verb added to `wat/gen.wat` and not documented is a red build.** This is the direction
//!   that matters: the surface growing in silence, which is how `lift3` once shipped with zero
//!   laws AND zero consumers.
//! - **A `:wat::gen::` name written in the doc that does not exist is a red build.** A reader who
//!   types what the doc shows gets a real verb, or this fails first.
//!
//! It does **not** — and cannot — check that a documented verb is documented *correctly*. Arity,
//! argument order, and semantics are `conferre`'s ground, and that ward found real defects here
//! (`string::join`'s argument order, a fabricated `:user::apply2`) that no set-equality check
//! would ever see. Claiming otherwise would be the decoration this codebase keeps removing.
//!
//! Precedent and shape: `tests/lint/no_unknown_sequi_rune.rs` — *"the table is the definition,
//! this is the gate."*


use std::collections::BTreeSet;

const LIB: &str = "wat/gen.wat";
const DOC: &str = "docs/GENERATIVE-TESTING.md";

/// The doc still writes `:wat::gen::` (it is markdown, not a wat program). Declaration heads
/// in `wat/gen.wat` are read by identity, so either spelling of `defn`/`wat.gen/<name>` yields
/// the same leaf. The prefix stays a prefix: a string that is a wat form is inlined wat.
const GEN_NS: &str = ":wat::gen::";

/// Declarations that create a callable `:wat::gen::` verb.
const VERB_FORMS: &[&str] = &["defn", "defmacro"];
/// Declarations that create a `:wat::gen::` TYPE — the head of a `Type/field` accessor.
const TYPE_FORMS: &[&str] = &["defstruct", "defrecord", "defenum", "typealias"];

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Names declared in `wat/gen.wat` under one of `forms` — the name following the namespace
/// prefix in a `defn` or `defmacro` declaration head.
fn declared(src: &str, forms: &[&str]) -> BTreeSet<String> {
    let parsed = wat::parse_all_with_file(src, LIB)
        .unwrap_or_else(|e| panic!("parse {LIB}: {e:?}"));
    let want: BTreeSet<String> = forms.iter().map(|f| format!(":wat::core::{f}")).collect();
    let mut out = BTreeSet::new();
    fn walk(node: &wat::WatAST, want: &BTreeSet<String>, out: &mut BTreeSet<String>) {
        let wat::WatAST::List(items, _) = node else {
            for child in node.children().iter() {
                walk(child, want, out);
            }
            return;
        };
        if let (Some(head), Some(name_node)) = (items.first(), items.get(1)) {
            if let Some(head_id) = crate::decl_identity::canon(head) {
                if want.contains(&head_id) {
                    if let Some(name_id) = crate::decl_identity::canon(name_node) {
                        if let Some(leaf) = name_id.strip_prefix(GEN_NS) {
                            if !leaf.is_empty() && !leaf.contains(':') {
                                out.insert(leaf.to_string());
                            }
                        }
                    }
                }
            }
        }
        for child in items {
            walk(child, want, out);
        }
    }
    for form in &parsed {
        walk(form, &want, &mut out);
    }
    out
}

/// Every `:wat::gen::NAME` the doc mentions, NAME as written (may be `Type/field`).
fn doc_qualified_names(doc: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = doc;
    while let Some(at) = rest.find(GEN_NS) {
        let after = &rest[at + GEN_NS.len()..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || "?-_<>=+*//".contains(*c) || *c == '/')
            .collect();
        let step = name.len().max(1);
        if !name.is_empty() {
            out.insert(name);
        }
        rest = &after[step..];
    }
    out
}

/// Is `name` mentioned in the doc as a backticked token — bare (`ints`) or qualified?
///
/// Bare is what the surface table uses; qualified is what the prose uses. Either counts: the
/// question this gate asks is "does the doc acknowledge this verb at all", not "where".
fn doc_mentions(doc: &str, name: &str) -> bool {
    // The backtick is a CHAR here, never part of a string literal — because in wat a backtick is
    // QUASIQUOTE, so a literal like "`name" is read as a well-formed form and
    // `tests/lint/no_inlined_wat_in_tests.rs` rightly refuses it. Same restructure as the paren
    // above, and it reads better besides: the condition is "a markdown code span whose first
    // token is this name", and now the code says exactly that.
    const TICK: char = '`';
    let name_char = |c: char| c.is_alphanumeric() || "?-_".contains(c);

    for cand in [name.to_string(), format!("{GEN_NS}{name}")] {
        let mut from = 0usize;
        while let Some(rel) = doc[from..].find(&cand) {
            let at = from + rel;
            let opened = doc[..at].ends_with(TICK);
            let closed = doc[at + cand.len()..]
                .chars()
                .next()
                .is_none_or(|c| !name_char(c));
            if opened && closed {
                return true;
            }
            from = at + 1;
        }
    }
    false
}

#[test]
fn every_exported_gen_verb_is_documented() {
    let lib = std::fs::read_to_string(repo_path(LIB)).expect("read wat/gen.wat");
    let doc = std::fs::read_to_string(repo_path(DOC)).expect("read the design record");

    let verbs = declared(&lib, VERB_FORMS);
    assert!(
        verbs.len() >= 20,
        "parsed only {} verbs from {LIB} — the declaration shape changed and this gate went \
         blind; fix the parser rather than the assertion",
        verbs.len()
    );

    let undocumented: Vec<&String> = verbs.iter().filter(|v| !doc_mentions(&doc, v)).collect();
    assert!(
        undocumented.is_empty(),
        "{} verb(s) exported from {LIB} appear nowhere in {DOC}: {:?}\n\
         \n\
         A verb the design record does not acknowledge is a surface that grew in silence — the \
         shape that once let a combinator ship with zero laws AND zero consumers. Either add it \
         to the surface table, or delete it: a verb with no caller is a claim, not a capability.",
        undocumented.len(),
        undocumented
    );
}

#[test]
fn every_gen_name_the_doc_writes_actually_exists() {
    let lib = std::fs::read_to_string(repo_path(LIB)).expect("read wat/gen.wat");
    let doc = std::fs::read_to_string(repo_path(DOC)).expect("read the design record");

    let verbs = declared(&lib, VERB_FORMS);
    let types = declared(&lib, TYPE_FORMS);

    let phantoms: Vec<String> = doc_qualified_names(&doc)
        .into_iter()
        .filter(|name| {
            // `Type/field` — the accessor a defstruct/defrecord generates. Real iff the head is.
            if let Some((head, _field)) = name.split_once('/') {
                return !types.contains(head);
            }
            !verbs.contains(name) && !types.contains(name)
        })
        .collect();

    assert!(
        phantoms.is_empty(),
        "{DOC} names {} `:wat::gen::` symbol(s) that do not exist in {LIB}: {:?}\n\
         \n\
         A reader types what the design record shows. A name that resolves to nothing is the \
         defect `cernere` exists to catch — it found one here: a fabricated apply-arity-2 helper, \
         offered as proof a constructor is a function value, and defined nowhere in the tree.",
        phantoms.len(),
        phantoms
    );
}
