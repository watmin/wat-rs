//! FM 2-bis probe — arc 251 Stone 251.5 FOUNDATION: the EDN round-trip is faithful.
//!
//! The 251.5 sweep is a wat-to-wat fixer: read .wat → `edn::read` → transform the
//! forms → `edn::write` → write .wat. The whole approach rests on ONE invariant:
//! reading a program to EDN and writing it back is an IDENTITY (modulo span). If it
//! is not, every transform bug hides behind a round-trip bug. This probe pins it,
//! using the same `edn::bridge` round-trip the `:wat::edn::read`/`write`
//! primitives wrap.
//!
//! C01: a representative program (defn, parametric type, match, vector, map, nested
//!      calls) survives `program_to_edn` → `edn_to_program` unchanged (WatAST eq is
//!      span-agnostic). GREEN already at HEAD — this is a FOUNDATION guard, not a
//!      RED→GREEN gate; it must hold before the fixer is trusted, and stay holding.
//!
//! Run: `cargo test --release --test probe_arc251_stone5_roundtrip`
//!
// rune:lint(no-inlined-wat) — this probe IS the reader/writer (raw wat
// program text in, via `parse_all!`, WatAST-equality out) — the FOUNDATION
// claim under test is that parsing a representative program and writing it
// back through the EDN bridge is an identity. There is no FrozenWorld/
// call_beside_value seam here: the subject is the parse/write round-trip itself,
// not evaluation of a program. Precedent: tests/collection/wat_arc167_vector_ast.rs.

use wat::edn::bridge::{edn_to_program, program_to_edn};

/// Parse → forms → EDN text → forms again; the two form-vecs must be equal.
fn roundtrips(src: &str) -> bool {
    let forms = wat::parse_all!(src).expect("parse ok");
    let edn = program_to_edn(&forms);
    let forms2 = edn_to_program(&edn).expect("edn_to_program ok");
    forms == forms2
}

#[test]
fn contract_01_edn_roundtrip_is_faithful() {
    // A representative slice of the surface the sweep touches. Arc 109 wave 2
    // "annihilate the angle bracket" — the parametric type on `xs` used to be spelled
    // `:wat::core::Vector<wat::core::i64>` (a single keyword); that spelling is
    // refused at the lexer now. The angle bracket was pure decoration for THIS test's
    // subject (an EDN round-trip identity over a representative program) — it never
    // tested anything angle-specific — so the input is migrated to the surviving `:-`
    // reference form, which is if anything MORE representative of "the surface the
    // sweep touches" post-migration. Class 3 (a): subject survives.
    let program = r#"
        (:wat::core::defn :user::inc [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+ x 1))
        (:wat::core::defn :user::sum [xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
          (:wat::core::foldl :wat::core::i64::+ 0 xs))
        (:wat::core::def :user::table
          (:wat::core::HashMap-new))
    "#;
    assert!(
        roundtrips(program),
        "program_to_edn → edn_to_program must be an identity (span-agnostic) — \
         the wat-to-wat fixer's read/write foundation"
    );
}

#[test]
fn contract_02_roundtrip_preserves_collections_and_quote() {
    // Maps, sets, vectors, and quoted data must survive the round-trip — these are
    // the EDN-native shapes the fixer must not corrupt.
    let program = r#"
        (:wat::core::def :user::m {:a 1 :b 2})
        (:wat::core::def :user::s #{1 2 3})
        (:wat::core::def :user::v [1 2 3])
        (:wat::core::def :user::q (:wat::core::quote (a b c)))
    "#;
    assert!(
        roundtrips(program),
        "maps/sets/vectors/quoted forms must survive the EDN round-trip unchanged"
    );
}
