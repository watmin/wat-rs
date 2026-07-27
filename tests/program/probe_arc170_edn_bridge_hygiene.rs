//! Disconfirming probe — arc 170 execve step 2d: does the plain-EDN program
//! bridge round-trip a MACRO-GENERATED form?
//!
//! `tests/resolve/probe_arc251_stone5_roundtrip.rs` and
//! `tests/program/probe_arc213_program_edn_roundtrip.rs` both pin
//! `program_to_edn` → `edn_to_program` as an identity — but every form they
//! feed it comes from `parse_all!`, and the parser emits `Identifier::bare`
//! (EMPTY scope set). So the whole existing round-trip corpus exercises
//! exactly the case where there is nothing to lose.
//!
//! `spawn-process` does NOT ship parser output. It takes `args[0]` — an
//! evaluated `Vector<WatAST>` (`src/kernel/spawn.rs:485`) — which a macro
//! routinely builds (`wat/bracket.wat`'s worker program is the live consumer
//! that fired `HygieneScopeDivergence` when step 2d shipped source text).
//! The macro expander calls `Identifier::add_scope` on every template-origin
//! symbol (`src/scope/resolution.rs` module doc), so those forms carry
//! hygiene scopes, and `env_key` makes the scope set load-bearing at every
//! bind and lookup.
//!
//! C01 — the CONTROL. A bare identifier round-trips. (Re-states the existing
//!       green claim locally so C02's failure cannot be blamed on the harness.)
//! C02 — the DISCONFIRMING arm. A SCOPED identifier round-trips.
//!
//! The two arms are the same transform over two inputs that differ in exactly
//! one dimension (the scope set), so whichever way C02 lands, it is the scope
//! set that decided it — not the printer, not the parser, not the codec.

use std::collections::BTreeSet;
use wat::ast::WatAST;
use wat::scope::{fresh_scope, Identifier};
use wat::wat_edn_bridge::{edn_to_program, program_to_edn};

/// One symbol, wrapped as a one-form program, through the bridge and back.
fn roundtrip_symbol(ident: Identifier) -> WatAST {
    let form = WatAST::Symbol(ident, wat::rust_caller_span!());
    let edn = program_to_edn(std::slice::from_ref(&form));
    let mut back = edn_to_program(&edn).expect("edn_to_program ok");
    assert_eq!(back.len(), 1, "one form in, one form out");
    back.pop().expect("one form")
}

fn scopes_of(a: &WatAST) -> BTreeSet<u64> {
    match a {
        WatAST::Symbol(id, _) => id.scopes().iter().map(|s| s.as_u64()).collect(),
        other => panic!("expected a Symbol, got {other:?}"),
    }
}

#[test]
fn c01_control_a_bare_identifier_round_trips() {
    let ident = Identifier::bare("kwargs");
    let back = roundtrip_symbol(ident.clone());

    assert_eq!(
        scopes_of(&back),
        BTreeSet::new(),
        "C01 CONTROL: a bare identifier has no scopes to lose"
    );
    assert_eq!(
        back,
        WatAST::Symbol(ident, wat::rust_caller_span!()),
        "C01 CONTROL: a bare identifier must survive the EDN bridge unchanged \
         (WatAST equality is span-agnostic) — this is the case the existing \
         round-trip probes cover"
    );
}

/// RED at HEAD — measured 2026-07-27: `left: {} right: {1}`. `watast_to_edn`
/// (`src/wat_edn_bridge.rs:104`) emits `Symbol::new(ident.as_str())` — the name
/// only — and `edn_to_watast` (`:169`) rebuilds with `Identifier::bare`, so the
/// scope set is dropped on encode and cannot be recovered on decode. Tracked as
/// the gate for execve step 2d; GREEN when the bridge carries scopes.
/// `#[ignore]` (not deleted) keeps the floor honest AND the gate visible — the
/// same posture `probe_arc278_self_scheduling` holds for item-(c).
#[test]
#[ignore = "RED gate — arc 170 execve step 2d: the EDN bridge drops Identifier.scopes"]
fn c02_a_scoped_identifier_round_trips() {
    let scope = fresh_scope();
    let ident = Identifier::bare("kwargs").add_scope(scope);

    // Precondition: the input really does carry the scope. If this fails the
    // probe is measuring nothing.
    assert_eq!(
        ident.scopes().iter().map(|s| s.as_u64()).collect::<BTreeSet<_>>(),
        BTreeSet::from([scope.as_u64()]),
        "C02 precondition: the identifier under test must actually be scoped"
    );

    let back = roundtrip_symbol(ident.clone());

    assert_eq!(
        scopes_of(&back),
        BTreeSet::from([scope.as_u64()]),
        "C02: a MACRO-GENERATED identifier's hygiene scope must survive \
         program_to_edn → edn_to_program. `spawn-process` ships exactly these \
         forms (src/kernel/spawn.rs:485), and env_key makes the scope set \
         load-bearing at every bind and lookup (src/scope/resolution.rs)"
    );
    assert_eq!(
        back,
        WatAST::Symbol(ident, wat::rust_caller_span!()),
        "C02: the round trip must be an identity for a scoped identifier, \
         exactly as C01 proves it is for a bare one"
    );
}
