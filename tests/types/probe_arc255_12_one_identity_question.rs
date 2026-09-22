//! Stone 255.12 — one identity question, asked with four `==`s.
//!
//! 255.8 made `wat.type/X` and `wat.core/X` one type and wired
//! `edn::render::type_denotation` into eight places. Four more sites asked the
//! SAME question with a raw comparison and therefore disagreed:
//!
//! | site | what it compared | how it failed |
//! |---|---|---|
//! | `types::TypeEnv::register_validated` | `TypeDef == TypeDef` | a re-declaration in the other spelling was `DuplicateType` |
//! | `types::register_stdlib_types_replacing` | `TypeDef != TypeDef` | (not in the brief) a re-spelling silently retracted-and-replaced |
//! | `edn::render::edn_to_typed_value_inner` | `p.as_str()` vs a literal table | a `wat.type`-spelled decode target matched no arm |
//! | `function::subsume::value_matches_type_by_name` | `p.as_str()` vs `val_type_path` | a `wat.type`-spelled clause never dispatched |
//!
//! ## ⛔ WHY THIS FILE IS MOSTLY NEGATIVE ROWS
//!
//! Every stone in this arc since 255.9 cured in the RESTRICTIVE direction — it
//! closed a bypass, so a regression shows up as something that stops being
//! refused. **255.12 runs the other way.** It makes walls ACCEPT MORE: more
//! re-declarations counted equivalent, more values counted matching. A
//! regression here looks like a green gate, which is why the accepting rows are
//! outnumbered by rows that pin what must STILL be refused. 255.11's rule:
//! *a wall that stops refusing is indistinguishable from a wall that never
//! fired.*
//!
//! ## ⭐ THE ROW THAT FOUND SOMETHING
//!
//! [`divergent_declaration_that_denotation_collapses_is_still_refused`] is the
//! brief's mandatory adversarial row — "construct a divergence `type_denotation`
//! might COLLAPSE" — and it caught a real defect in this stone's own first
//! draft: `:wat::type::Infer` is a marker with no `:wat::core::Infer` behind it,
//! `type_denotation` collapsed it anyway, and an ILLEGAL second declaration was
//! swallowed as a benign no-op. The cure is `types::denoted_type_path`, which
//! carries the carve-out `check::format_type_path` had already written down —
//! in a renderer, where no equality could see it.
//!
//! ## What is NOT cured here — site 2, the macro registry
//!
//! The brief's site 2 (`macros/registry.rs::ast_same_identity`) is REPORTED, not
//! cured, and the lead behind it is REFUTED. Its fixtures live in
//! `tests/macros/probe_arc255_12_macro_*`; the short version is that
//! `wat/holon/Ngram.wat`'s conversion failure is a `<-`/`->` → `:-` GRAMMAR flip
//! inside a quasiquoted template, not a namespace spelling, and no identity door
//! equates three different tokens.

use wat::freeze::{startup_from_file, FrozenWorld, StartupError};
use wat::types::TypeErrorKind;
use wat::value::Value;

// ─── helpers ────────────────────────────────────────────────────────────────

fn freeze_ok(path: &str, why: &str) -> FrozenWorld {
    match startup_from_file(path) {
        Ok(w) => w,
        Err(e) => panic!("{why}\n  freeze should have succeeded; got: {e}"),
    }
}

/// Assert STRUCTURALLY that freezing `path` fails with the type registry's own
/// duplicate verdict, on the name the fixture declares twice.
///
/// Matched on the `TypeErrorKind` variant AND its `name` field — never a
/// `contains()` over the rendered `Debug`. A loose check would pass on a
/// `DuplicateType` about some OTHER name, so it could not tell "the gate refused
/// this re-declaration" from "the fixture has a typo that collides with
/// something else"; and a bare `is_err()` would pass on any breakage at all —
/// the failure mode `[[retiring a name disarms every bare is_err test]]` names.
fn assert_duplicate_type(path: &str, name: &str, why: &str) {
    let err = match startup_from_file(path) {
        Ok(_) => panic!("{why}\n  ⛔ the declaration was ACCEPTED; this row has stopped measuring"),
        Err(e) => e,
    };
    let StartupError::Type(t) = &err else {
        panic!("{why}\n  expected a TYPE-registry failure; got a different StartupError: {err:?}")
    };
    match t.kind() {
        TypeErrorKind::DuplicateType { name: got } => assert_eq!(
            got, name,
            "{why}\n  the gate refused a duplicate, but of the wrong name — this row has stopped \
             measuring the fixture it names"
        ),
        other => panic!("{why}\n  expected DuplicateType({name}); the freeze DID fail, but on {other:?}"),
    }
}

fn call(world: &FrozenWorld, name: &str) -> Value {
    let f = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} should be defined by the fixture"));
    wat::runtime::apply_function(
        f.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .unwrap_or_else(|e| panic!("{name} should run; got: {e:?}"))
}

/// The `:wat::edn::Validation` variant `name` produced — `"Valid"` or `"Invalid"`.
fn validation_variant(v: &Value) -> String {
    match v {
        Value::Enum(e) if e.type_path == ":wat::edn::Validation" => e.variant_name.clone(),
        other => panic!("expected a :wat::edn::Validation enum value; got {other:?}"),
    }
}

fn as_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.as_ref().clone(),
        other => panic!("expected a String value; got {other:?}"),
    }
}

const CROSS: &str = "tests/types/probe_arc255_12_redeclare_cross_spelling.wat";
const RUNTIME: &str = "tests/types/probe_arc255_12_coerce_and_dispatch.wat";

// ─── site 1 — the registration gate ACCEPTS a re-spelling ───────────────────

/// THE CURE. Four declaration kinds — aggregate, parametric-field aggregate,
/// enum, typealias — each declared twice, once per spelling. Pre-cure this
/// fixture freezes with `DuplicateType` on the first pair; post-cure it is four
/// permitted no-ops.
///
/// This is the shape that broke `wat/source.wat` under conversion: the file is
/// baked into the binary by `include_str!` AND read off disk, so both copies
/// meet at `register_validated`, and once the on-disk one is converted its
/// `wat.core/String` field reads `wat.type/String`.
#[test]
fn a_re_declaration_in_the_other_spelling_is_a_no_op() {
    freeze_ok(
        CROSS,
        "a declaration re-delivered in the wat.type spelling is the SAME declaration",
    );
}

/// The unchanged half of the contract: a BYTE-identical re-declaration was
/// already a no-op (arc 054) and still is. Without this row, the row above
/// cannot distinguish "the denotation door works" from "the gate stopped
/// refusing anything at all".
#[test]
fn a_byte_identical_re_declaration_is_still_a_no_op() {
    freeze_ok(
        "tests/wat_lang/wat_idempotent_redeclare.wat",
        "arc 054's own fixture must keep freezing clean",
    );
}

// ─── site 1 — NON-VACUITY: what must STILL be refused ───────────────────────

/// The plainest divergence — same name, same field name, different field TYPE.
/// `type_denotation` does not collapse `i64` and `String`.
#[test]
fn divergent_field_type_is_still_a_duplicate() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_divergent_field_type.wat.bad",
        ":p255_12::Div",
        "i64 vs String is a DIVERGENT re-declaration",
    );
}

/// A divergence in a field the comparator never denotes — `AggregateDef.nature`.
/// This row is why `type_defs_same` is "normalize the `TypeExpr`s, then `==`"
/// rather than a hand-written field walk: a walk that forgot `nature` would pass
/// every other row in this file.
#[test]
fn divergent_nature_is_still_a_duplicate() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_divergent_nature.wat.bad",
        ":p255_12::Nat",
        "a record and a struct with identical fields are two declarations",
    );
}

#[test]
fn divergent_field_name_is_still_a_duplicate() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_divergent_field_name.wat.bad",
        ":p255_12::Fld",
        "a field NAME is a String in the fields vector and is never denoted",
    );
}

/// Same names, same types, different ORDER. `fields` is a `Vec` and `==` is
/// order-sensitive; normalizing the `TypeExpr`s does not sort it.
#[test]
fn divergent_field_order_is_still_a_duplicate() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_divergent_field_order.wat.bad",
        ":p255_12::Ord",
        "field order is part of a declaration",
    );
}

#[test]
fn divergent_enum_variant_is_still_a_duplicate() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_divergent_variant.wat.bad",
        ":p255_12::Var",
        "a changed variant name is a divergent enum",
    );
}

/// ⭐ **THE ADVERSARIAL ROW — the stone.**
///
/// A divergence that `edn::render::type_denotation` genuinely DOES collapse:
/// `:wat::type::Infer` vs `:wat::core::Infer`. `Infer` is a MARKER
/// (`types::INFER_TYPE_PATH`), not a `wat.type` member — there is no
/// `:wat::core::Infer`, and declaring a field of that type is `UnknownNamedType`
/// standing alone. `type_denotation` is a blind prefix rewrite and collapses it
/// anyway.
///
/// This stone's first draft therefore called the two declarations equivalent and
/// SWALLOWED the illegal one as a benign no-op — the file checked clean, measured
/// on two binaries. The cure is `types::denoted_type_path`, the one door that
/// carries the `Infer` carve-out that had until now lived only inside
/// `check::format_type_path`, a renderer.
///
/// ⛔ If this row ever goes green, the identity door has been widened past what
/// the substrate itself says the marker means.
#[test]
fn divergent_declaration_that_denotation_collapses_is_still_refused() {
    assert_duplicate_type(
        "tests/types/probe_arc255_12_redeclare_collapsible_divergence.wat.bad",
        ":p255_12::Inf",
        "the Infer marker must not denote to a :wat::core:: name that does not exist",
    );
}

// ─── site 3 — the EDN coerce table ──────────────────────────────────────────

/// Sites 3 and 4 in one freeze. ⚠ Neither is visible to `--check`: this fixture
/// type-checks clean on the PRE-cure binary too. The defect is at eval.
#[test]
fn the_runtime_tables_read_the_denotation() {
    let w = freeze_ok(RUNTIME, "the runtime fixture type-checks in both spellings");

    // ── site 3, the cure ──
    assert_eq!(
        validation_variant(&call(&w, ":p255_12::coerce-core-ok")),
        "Valid",
        "the control: 42 against the wat.core spelling was never in doubt"
    );
    assert_eq!(
        validation_variant(&call(&w, ":p255_12::coerce-type-ok")),
        "Valid",
        "⛔ 42 against wat.type/i64 — the 8d-ii STOP. Pre-cure this answered Invalid \
         with `:expected \":wat::core::i64\" :got \"Integer\"`, a diagnostic that \
         contradicted itself because format_type denotes on the way OUT while the \
         comparison did not"
    );
    assert_eq!(
        validation_variant(&call(&w, ":p255_12::coerce-type-parametric-ok")),
        "Valid",
        "the parametric half — head AND argument re-spelled"
    );

    // ── site 3, NON-VACUITY: a value of the WRONG type is still refused ──
    assert_eq!(
        validation_variant(&call(&w, ":p255_12::coerce-type-wrong")),
        "Invalid",
        "⛔ a String is still not an i64 — denoting the TARGET must not make the \
         coerce table accept anything"
    );
    assert_eq!(
        validation_variant(&call(&w, ":p255_12::coerce-type-parametric-wrong")),
        "Invalid",
        "⛔ a Vector of String is still not a Vector of i64 — the walk still \
         recurses to the leaf"
    );

    // ── site 4, the cure ──
    assert_eq!(
        call(&w, ":p255_12::dispatch-type-ok"),
        Value::i64(42),
        "⛔ a defclause keyed on wat.type/i64 must take a plain 42. Pre-cure this \
         was NoMatchingClause, reporting `expected :wat::core::i64, got \
         wat::core::i64` — the same self-contradicting diagnostic, one tier over"
    );

    // ── site 4, NON-VACUITY: the matcher is not a wildcard ──
    assert_eq!(
        as_string(&call(&w, ":p255_12::dispatch-picks-i64")),
        "i64-clause",
        "two clauses, two spellings, two types: the i64 call selects the i64 clause"
    );
    assert_eq!(
        as_string(&call(&w, ":p255_12::dispatch-picks-string")),
        "string-clause",
        "⛔ and the String call must still select the STRING clause — if the cure \
         had over-collapsed, the first clause would swallow both"
    );
}
