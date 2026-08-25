//! Arc 170 execve step 2d — does the plain-EDN program bridge carry HYGIENE?
//!
//! `tests/resolve/probe_arc251_stone5_roundtrip.rs` and
//! `tests/program/probe_arc213_program_edn_roundtrip.rs` both pin
//! `program_to_edn` → `edn_to_program` as an identity, and both are green —
//! but every form they feed it comes from `parse_all!`, and the parser emits
//! `Identifier::bare` (EMPTY scope set). The whole existing corpus therefore
//! exercises exactly the case where there is nothing to lose, and is
//! structurally incapable of catching a dropped scope.
//!
//! `spawn-process` does NOT ship parser output. It takes `args[0]` — an
//! evaluated `Vector<WatAST>` (`src/kernel/spawn.rs:485`) — which a macro
//! routinely builds (`wat/bracket.wat`'s worker program is the live consumer
//! that fired `HygieneScopeDivergence` when step 2d shipped source text). The
//! expander calls `Identifier::add_scope` on every template-origin symbol, and
//! `env_key` (`src/scope/resolution.rs`) makes the scope set load-bearing at
//! every bind and lookup.
//!
//! ## The contract these probes pin
//!
//! A scoped symbol crosses as `#wat.ast/sym ["name" [ids…]]`, and the decode
//! side remaps each distinct wire id to a FRESH local scope. So the claim is
//! NOT raw-id identity — it is deliberately weaker and stronger at once:
//!
//! - **weaker**: the numbers change. They must. `ScopeId` has no public
//!   constructor from a `u64` because process-uniqueness is its entire
//!   contract; importing a sender's number would let it collide with a scope
//!   this process mints later — the capture hygiene exists to prevent.
//! - **stronger**: the STRUCTURE survives exactly — which identifiers share a
//!   scope and which do not. `hash_canonical_program` is precisely the oracle
//!   for that claim (it renumbers scopes to first-appearance order before
//!   hashing — "a RENUMBER, not a strip", `src/hash.rs`), so the round trip is
//!   asserted against a tool that already existed for this exact notion.
//!
//! C01 — CONTROL: a bare symbol round-trips byte-identically (the case the
//!       existing corpus covers; re-stated locally so a C02 failure cannot be
//!       blamed on this harness).
//! C02 — a scoped symbol keeps a non-empty scope set. THE disconfirming arm:
//!       RED before the bridge carried scopes (`left: {} right: {1}`).
//! C03 — DISCRIMINATION: shared-scope stays shared, distinct stays distinct,
//!       and the two structures do not collapse into each other.
//! C04 — the imported ids are FRESH, never the wire's — collision-safety.

use std::collections::BTreeSet;
use wat::ast::WatAST;
use wat::hash::hash_canonical_program;
use wat::scope::{fresh_scope, Identifier};
use wat::edn::bridge::{edn_to_program, program_to_edn};

fn sym(ident: Identifier) -> WatAST {
    WatAST::Symbol(ident, wat::rust_caller_span!())
}

/// A whole program through the bridge and back.
fn roundtrip(forms: &[WatAST]) -> Vec<WatAST> {
    let edn = program_to_edn(forms);
    edn_to_program(&edn).unwrap_or_else(|e| panic!("edn_to_program failed: {e} — frame was {edn}"))
}

fn scopes_of(a: &WatAST) -> BTreeSet<u64> {
    match a {
        WatAST::Symbol(id, _) => id.scopes().iter().map(|s| s.as_u64()).collect(),
        other => panic!("expected a Symbol, got {other:?}"),
    }
}

#[test]
fn c01_control_a_bare_symbol_round_trips_unchanged() {
    let forms = vec![sym(Identifier::bare("kwargs"))];
    let back = roundtrip(&forms);

    assert_eq!(
        scopes_of(&back[0]),
        BTreeSet::new(),
        "C01 CONTROL: a bare symbol has no scopes to lose"
    );
    assert_eq!(
        back, forms,
        "C01 CONTROL: a bare symbol must survive byte-identically — this is the \
         case the existing round-trip probes cover, and it must not regress"
    );
}

#[test]
fn c02_a_scoped_symbol_keeps_its_hygiene() {
    let scope = fresh_scope();
    let forms = vec![sym(Identifier::bare("kwargs").add_scope(scope))];

    // Precondition — if the input is not scoped this probe measures nothing.
    assert_eq!(
        scopes_of(&forms[0]).len(),
        1,
        "C02 precondition: the symbol under test must actually be scoped"
    );

    let back = roundtrip(&forms);

    assert_eq!(
        scopes_of(&back[0]).len(),
        1,
        "C02: a MACRO-GENERATED symbol's hygiene scope must survive \
         program_to_edn → edn_to_program. `spawn-process` ships exactly these \
         forms (src/kernel/spawn.rs:485); erasing the scope on one side of a \
         wire manufactures HygieneScopeDivergence (check.rs:2035)"
    );
    assert_eq!(
        hash_canonical_program(&forms),
        hash_canonical_program(&back),
        "C02: the round trip must preserve scope STRUCTURE — equal up to an \
         order-preserving renaming, which is what hash_canonical_program measures"
    );
}

#[test]
fn c03_scope_structure_is_preserved_and_discriminated() {
    // SHARED: two symbols under one scope (a macro's binder and its reference).
    let one = fresh_scope();
    let shared = vec![
        sym(Identifier::bare("tmp").add_scope(one)),
        sym(Identifier::bare("tmp").add_scope(one)),
    ];
    // DISTINCT: same names, two different scopes (two separate expansions).
    let distinct = vec![
        sym(Identifier::bare("tmp").add_scope(fresh_scope())),
        sym(Identifier::bare("tmp").add_scope(fresh_scope())),
    ];

    let shared_back = roundtrip(&shared);
    let distinct_back = roundtrip(&distinct);

    assert_eq!(
        scopes_of(&shared_back[0]),
        scopes_of(&shared_back[1]),
        "C03: two symbols that SHARED a scope must still share one — this is \
         what makes a binder and its reference resolve to each other"
    );
    assert_ne!(
        scopes_of(&distinct_back[0]),
        scopes_of(&distinct_back[1]),
        "C03: two symbols under DISTINCT scopes must stay distinct — collapsing \
         them is the variable capture hygiene exists to prevent"
    );
    assert_ne!(
        hash_canonical_program(&shared_back),
        hash_canonical_program(&distinct_back),
        "C03 DISCRIMINATION: shared-one-scope and two-distinct-scopes are \
         different programs and must not round-trip into the same structure \
         (the guard hash.rs's own distinct_scope_structure_hashes_differently keeps)"
    );
}

/// C05 — the WIRE SHAPE is the ruled record form, asserted STRUCTURALLY (the
/// frame is parsed and its shape matched; never a substring of a Debug string —
/// `no_loose_string_assert`'s rule, and "every wat stdio is an edn form").
///
/// Pins the builder's ruling: `["name" [ids]]` is a vector of non-uniform types
/// and reads as a two-field enum variant; a record body keeps the arc's one
/// rule intact — **record → `{field-map}`, variant → `[field-vec]`**.
///
/// The subject — a scoped symbol crosses as a `Tagged` whose body is a record,
/// not a tuple — is untouched by stone J's span carriage. Only the frame's
/// REACH changed, and on TWO levels (the second one deeper than the brief that
/// prompted this update described): `program_to_edn` now wraps the whole
/// program in `#wat.ast/Program {:origins […] :forms […]}` rather than handing
/// back a bare `Vector`, AND — because a delivered program is executed and any
/// node may end up in a `Fault`'s location — `program_to_edn` ALWAYS wraps
/// every individual node (`Carriage::Transport`, unconditionally, not only for
/// call forms) in `#wat.ast/Spanned {:node … :origin N :line N :col N …}`. So
/// this destructures `Program`, takes `:forms`, unwraps that per-node `Spanned`
/// carriage, and applies the SAME assertions to the `:node` it carries — it
/// still fails if the frame stops being the `Program` record, or the form
/// stops being `Spanned`, or the node stops being the ruled `ScopedSymbol`
/// record (none of that is weakened to "any Tagged").
#[test]
fn c05_the_wire_form_is_a_record_not_a_tuple() {
    let forms = vec![sym(Identifier::bare("kwargs").add_scope(fresh_scope()))];
    let frame = program_to_edn(&forms);
    let parsed = wat_edn::parse_owned(&frame).expect("the frame must be valid EDN");

    let (tag, body) = match &parsed {
        wat_edn::Value::Tagged(t, b) => (t, b.as_ref()),
        other => panic!(
            "program frame must be #wat.ast/Program {{:origins […] :forms […]}}, got {}",
            other.type_name()
        ),
    };
    assert_eq!(
        (tag.namespace(), tag.name()),
        ("wat.ast", "Program"),
        "C05: the program frame's own wrapper must be the ruled Program record"
    );
    let program_fields = match body {
        wat_edn::Value::Map(fields) => fields,
        other => panic!(
            "C05: #wat.ast/Program's body must be a RECORD map, not a {}",
            other.type_name()
        ),
    };
    let items = program_fields
        .iter()
        .find_map(|(k, v)| match k {
            wat_edn::Value::Keyword(kw) if kw.namespace().is_none() && kw.name() == "forms" => {
                Some(v)
            }
            _ => None,
        })
        .and_then(|v| match v {
            wat_edn::Value::Vector(items) => Some(items),
            _ => None,
        })
        .unwrap_or_else(|| panic!("C05: #wat.ast/Program must have a :forms Vector — frame: {frame}"));
    assert_eq!(items.len(), 1, "one form in, one form out");

    // Unwrap the per-node span carriage (stone J) — a SEPARATE declared
    // wrapper from the subject under test here, but always present, since
    // `program_to_edn` spans every node unconditionally.
    let (span_tag, span_body) = match &items[0] {
        wat_edn::Value::Tagged(t, b) => (t, b.as_ref()),
        other => panic!(
            "every form must cross Spanned-wrapped (stone J), got {} — frame: {frame}",
            other.type_name()
        ),
    };
    assert_eq!(
        (span_tag.namespace(), span_tag.name()),
        ("wat.ast", "Spanned"),
        "C05: the per-node span wrapper must be the ruled Spanned record"
    );
    let span_fields = match span_body {
        wat_edn::Value::Map(fields) => fields,
        other => panic!("C05: #wat.ast/Spanned's body must be a RECORD map, not a {}", other.type_name()),
    };
    let node = span_fields
        .iter()
        .find_map(|(k, v)| match k {
            wat_edn::Value::Keyword(kw) if kw.namespace().is_none() && kw.name() == "node" => Some(v),
            _ => None,
        })
        .unwrap_or_else(|| panic!("C05: #wat.ast/Spanned must have a :node field — frame: {frame}"));

    let (tag, body) = match node {
        wat_edn::Value::Tagged(t, b) => (t, b.as_ref()),
        other => panic!(
            "a scoped symbol must cross as a TAGGED literal, got {} — frame: {frame}",
            other.type_name()
        ),
    };
    assert_eq!(
        (tag.namespace(), tag.name()),
        ("wat.ast", "ScopedSymbol"),
        "C05: the ruled tag"
    );

    let fields = match body {
        wat_edn::Value::Map(fields) => fields,
        other => panic!(
            "C05: the body must be a RECORD map, not a {} — a heterogeneous \
             tuple reads as an enum variant and breaks body-shape dispatch \
             (record → {{field-map}}, variant → [field-vec]). Frame: {frame}",
            other.type_name()
        ),
    };
    let key_names: Vec<&str> = fields
        .iter()
        .map(|(k, _)| match k {
            wat_edn::Value::Keyword(kw) => {
                assert!(kw.namespace().is_none(), "C05: field keys are bare keywords");
                kw.name()
            }
            other => panic!("C05: field key must be a Keyword, got {}", other.type_name()),
        })
        .collect();
    assert_eq!(
        key_names,
        vec!["name", "scopes"],
        "C05: exactly the two named fields — frame: {frame}"
    );
    assert!(
        matches!(&fields[0].1, wat_edn::Value::String(s) if s.as_ref() == "kwargs"),
        "C05: :name carries the bare name — frame: {frame}"
    );
    assert!(
        matches!(&fields[1].1, wat_edn::Value::Vector(ids)
                 if ids.len() == 1 && matches!(ids[0], wat_edn::Value::Integer(_))),
        "C05: :scopes carries a Vector of integer ids — frame: {frame}"
    );
}

#[test]
fn c04_imported_scopes_are_fresh_not_the_wire_s() {
    let scope = fresh_scope();
    let forms = vec![sym(Identifier::bare("kwargs").add_scope(scope))];
    let back = roundtrip(&forms);

    let imported: Vec<u64> = scopes_of(&back[0]).into_iter().collect();
    assert_eq!(imported.len(), 1, "C04: exactly one scope came back");
    assert_ne!(
        imported[0],
        scope.as_u64(),
        "C04: the decoded scope must be FRESH, not the sender's number. \
         ScopeId has no public u64 constructor precisely because \
         process-uniqueness is its contract — importing the wire's number \
         would let it collide with a scope this process mints later, \
         reintroducing capture through the transport"
    );
    assert!(
        imported[0] > scope.as_u64(),
        "C04: fresh_scope() is monotonic, so an id minted after `scope` must \
         exceed it — evidence the id came from the allocator, not the wire"
    );
}
