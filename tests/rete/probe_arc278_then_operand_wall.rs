//! Arc 278 — the `:then` operand wall's PERMANENT red-proof.
//!
//! `validate_and_reorder_then` already checked a `:then` insert's SHAPE — head, fact type,
//! field names, positional arity — and stopped short of looking INSIDE a value-position
//! argument. So `(:nf::Out bogus-bare-symbol)` passed `--check` with arity 1 == 1 field and
//! then raised mid-fire, once per derived fact, for a property of the RULE that no fact could
//! change — with a Rust `Debug` dump of the AST as the user-facing `got`.
//!
//! This is arc 278's third statement of one ruling: a negation cycle is a compile error (R18),
//! a lying `extend-type` is a compile error (R28), `()` stopped being a value (arc 179). An
//! illegal expression must be illegal, not merely diagnosed politely at runtime.
//!
//! ★ Arc 278 Stone B narrowed this wall's SCOPE (DESIGN-STONE-then-is-a-vector-of-singular-facts.md
//! § "Stone B") — a NESTED FORM (a call, e.g. the original fixture's `(:wat::core::+ ?a 1)`) is no
//! longer categorically unresolvable: it may be a fenced expression, judged by the wat-side
//! `then-item-fence` instead of this Rust wall. This fixture was narrowed to a BARE non-`?` symbol
//! — a shape no fence at any layer can ever resolve — so the permanent red-proof keeps proving
//! what is still true, rather than proving something Stone B deliberately made legal.
//!
//! WHY THIS FILE EXISTS rather than a hand-run probe: the wall was proven red once, by hand, in
//! `/tmp`. That is an anecdote — the instrument supplying its own result. Committed, the red is
//! permanent: un-arm the predicate and this test goes green-when-it-should-be-red, which is the
//! failure `feedback_a_green_test_can_prove_nothing` names.

use wat::freeze::{startup_from_file, StartupError};

#[test]
fn bare_symbol_in_then_is_a_compile_error() {
    let err = startup_from_file("tests/rete/probe_arc278_then_operand_wall.wat.bad")
        .expect_err("a bare non-`?` symbol in a :then value position must fail check, not fire");
    // The rete wall registers through the generic freeze-validator hook (`inventory::submit!`
    // in `src/rete/validate.rs`), so its findings surface as `Validator`, not a rete-specific
    // variant. `src/freeze/env.rs`'s comment claiming `StartupError::Rete(..)` is stale.
    let StartupError::Validator(errs) = &err else {
        panic!("expected StartupError::Validator (the defrule wall's hook), got {err:?}");
    };
    let rendered = format!("{errs}");

    // ⚠ WHY THESE ARE `contains` AND RUNED, rather than the `.edn` golden the rubric prefers:
    // the error embeds its `Span`, whose `:file` is an ABSOLUTE path that differs on every
    // machine and in CI. A structure-exact golden would be non-deterministic by construction —
    // exactly the "value that varies per run: path/pid/hash/timestamp" case
    // `no_loose_string_assert`'s own message names as the exemption. Each site below asserts one
    // load-bearing FIELD of the diagnostic, not a vague substring of the whole blob.
    //
    // The operand is checked WITHOUT its surrounding parens on purpose: a parenthesised wat form
    // in a Rust string literal is inlined wat, which `no_inlined_wat_in_tests` correctly rejects.
    // rune:lint(loose-assert) — the error embeds an absolute path (Span :file); an exact golden
    // cannot be deterministic. Asserts the error KIND.
    assert!(rendered.contains("RhsUnresolvableOperand"), "wrong error kind:\n{rendered}");
    // rune:lint(loose-assert) — same span/path reason. Asserts the rule is NAMED.
    assert!(rendered.contains("nf::compute"), "error does not name the rule:\n{rendered}");
    // rune:lint(loose-assert) — same span/path reason. Asserts the operand renders as WAT SOURCE
    // (`render_form`) and not as a Rust `Debug` dump of the AST, which is the defect this wall was
    // built to stop; a Debug rendering would read `Symbol(Identifier {...})` and would NOT
    // contain this.
    assert!(rendered.contains("bogus-bare-symbol"),
        "the operand must render as wat source, not Rust Debug:\n{rendered}");
    // rune:lint(loose-assert) — same span/path reason. Asserts the error TEACHES (R29) rather
    // than merely refusing, mirroring how `UnknownField` carries `available_fields`.
    assert!(rendered.contains("bound by this rule's :when"),
        "the error must teach what a RHS operand may be:\n{rendered}");
}
