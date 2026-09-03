//! **D10 — THE `:then` RHS TYPES ITS FIELD VALUES, LIKE THE REST OF THE LANGUAGE.**
//!
//! A rule's `:then` wrote values into a record's declared fields and never checked their types. The
//! same construction is a `#wat.check/TypeMismatch` everywhere else:
//!
//! ```text
//! ordinary   (:td::Bad :n "x")   ->  ":td::Bad: parameter #1 expects :wat::core::i64; got :wat::core::String"
//! in :then   (:tr::Bad :n ?s)    ->  compiled, fired, derived fact = #tr/Bad {:n "not-an-i64"}
//! ```
//!
//! Driven at HEAD `135b19c37` for a bound `?var` **and** a literal
//! (`wat-scripts/scratch-pad/d10-then-rhs-is-not-type-checked.wat`), each beside a well-typed
//! control that derived. A wrong-typed value entered the **fact set** — where joins, queries, the
//! oracle and `explain` all trust the declared schema.
//!
//! The four RHS walls that already existed — `RhsArityMismatch`, `RhsMissingFields`,
//! `RhsPositionalConstructionRetired`, `RhsUnresolvableOperand` — are every one of them
//! **structural**. None typed a value.
//!
//! ## ★ The invariant this file pins
//!
//! > A `:then` field value whose type is **knowable** and does not match the destination field's
//! > declared type is refused at rule-compile time.
//!
//! ## ⛔ Why the not-knowable fixture is the load-bearing one
//!
//! `OperandType` distinguishes *knowable-and-wrong* from *not-knowable*, and the `:when` side
//! carries `ComputedNotDerivableHere` precisely so the two cannot be confused. **A cure that
//! refuses every operand it cannot type passes every refusal probe below and still stops a corpus
//! of legal rules from compiling** — a `?var` bound from a derived fact, a computed operand whose
//! head is `Form`/`Redispatch`, a record-typed destination field. So
//! `probe_arc278_D10_then_field_types_notknowable.wat` *constructs* four such operands and requires
//! them to compile, fire, and derive checked values. Without that row the whole file scores full
//! marks for the failure it exists to prevent.
//!
//! ## Why five fixtures over three verdicts
//!
//! One source of `resolve_operand_type` can be right, wrong, or not-answerable, and only the
//! middle one is a refusal. The same computed operand appears three times on purpose: knowable and
//! RIGHT (`_ok.wat` row 3, derives 8), knowable and WRONG (`_computed.wat.bad`, refused), and
//! not knowable at all (`_notknowable.wat` nk1, derives 11). That resolver has already shipped a
//! false claim of exhaustiveness once — source 4 was missing while its doc called the list
//! complete, so wrapping an operand in a call made its type error *disappear* — and this is the
//! shape of drive that would have caught it.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately — a refusal's `Span` carries
/// `:file` verbatim, so an absolute path would make the diagnostic machine-dependent and no `.edn`
/// golden over it could ever be checked in. Same reason `probe_arc278_nested_wall.rs` states.
fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel} in {}: {e}", manifest.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// THE CONTROL, and it is load-bearing for all four refusals below.
///
/// Values, not counts: a count of 1 is exactly what the D10 repro produced while the value in it
/// was `"not-an-i64"`. Each line is a well-typed `:then` shape the wall now inspects — a bound
/// `?var`, a literal, a computed operand whose row declares its `ret`, and the positional
/// spelling — and each must still compile, fire, and carry the value it was given.
#[test]
fn every_well_typed_then_value_still_compiles_and_derives() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_ok.wat");
    assert!(ok, "the control must run — every `:then` value in it is well typed\n{out}{err}");
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec!["\"7 lit\"", "\"8 seed\"", "\"7 seed\""],
        "the control's DERIVED VALUES drifted — a bound `?var` (7), a literal (\"lit\"), a \
         computed `i64::+` (8) and the positional spelling (7 / \"seed\"). A count would not have \
         seen this; the defect D10 cures is a wrong VALUE at the right count\n{out}{err}"
    );
}

/// ★ MUTATION 2 — the row that stands between this cure and one that refuses everything.
///
/// Four CONSTRUCTED not-knowable operands, each a different reason:
///   nk1  a computed operand under a `Form` head (`cond`) — `ComputedNotDerivableHere`
///   nk2  a destination field declared as a RECORD — no rete segment to compare at
///   nk3  a nested constructor as a value — a non-rete head, so `rete_op_for` declines
///   nk4  a `?var` bound from a DERIVED fact — knowable and right, and it must not be skipped
///
/// All four must compile, fire, and derive the values asserted. If this test goes red while the
/// four refusals below stay green, the cure has started refusing what it merely cannot type — the
/// named failure of this strike.
#[test]
fn a_not_knowable_then_operand_still_compiles_and_derives() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_notknowable.wat");
    assert!(
        ok,
        "NOT-KNOWABLE IS NOT WRONG — a computed operand under a `Form` head, a record-typed \
         destination field, a nested constructor, and a `?var` bound from a derived fact must all \
         still compile. A refusal here is the failure this strike is named for\n{out}{err}"
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec!["\"11\"", "\"held\"", "\"seed\"", "\"7\""],
        "the not-knowable arms compiled but derived the wrong values — passing a check is not the \
         same as producing the right fact\n{out}{err}"
    );
}

/// ARM 1/4 — a bound `?var` of the wrong declared type (`resolve_operand_type` source 2).
///
/// PRE (driven at HEAD `135b19c37`): compiled, fired, `#tr/Bad {:n "not-an-i64"}` in the fact set.
/// The caret must land on `?s` — the operand the author wrote — not on the whole `:then` form.
#[test]
fn a_then_value_bound_to_a_wrongly_typed_var_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_bound_var.wat.bad");
    assert!(
        !ok,
        "a String-bound `?var` written into an i64 field is a rule-compile refusal — if this \
         program RAN, the `:then` RHS is untyped again and a wrong-typed value is back in the \
         fact set\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D10_then_field_types__bound_var.edn",
        "`RhsFieldTypeMismatch` must be the ONLY finding; it must name BOTH types (declared \
         `:wat::core::i64`, operand `string`) and point at `?s`'s own extent"
    );
}

/// ARM 2/4 — a LITERAL of the wrong type (`resolve_operand_type` source 3).
///
/// A separate source from arm 1, so a separate fixture: a cure wired only to the `?var` path
/// would leave this green while `#tl/Bad {:n "LITERAL-STRING"}` still reached the fact set, which
/// is the second half of what was driven at HEAD.
#[test]
fn a_then_value_that_is_a_wrongly_typed_literal_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_literal.wat.bad");
    assert!(!ok, "a String literal written into an i64 field is a rule-compile refusal\n{out}{err}");
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D10_then_field_types__literal.edn",
        "`RhsFieldTypeMismatch` must be the ONLY finding, quoting the literal AS WRITTEN"
    );
}

/// ARM 3/4 — the POSITIONAL producer, which is a different call site in `validate_then_form`.
///
/// The two arguments are swapped, so both fields are wrong and the golden carries TWO findings —
/// batching every finding is this validator's contract, and a golden with one entry would not
/// notice the loop stopping at the first.
#[test]
fn a_positional_then_with_wrongly_typed_args_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_positional.wat.bad");
    assert!(
        !ok,
        "positional args are declaration order BY DEFINITION, so swapped types are a refusal — a \
         cure wired only into the kwargs branch leaves this half of the wall dark\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D10_then_field_types__positional.edn",
        "TWO `RhsFieldTypeMismatch` findings, one per swapped field, each at its own arg's span"
    );
}

/// ARM 4/4 — a COMPUTED operand whose row DECLARES its return type (`resolve_operand_type`
/// source 4).
///
/// ⚠ The source that has already gone missing once: its doc called the list exhaustive while this
/// source was absent, and a computed operand fell to a `_` arm meaning "unbound `?var`" — so
/// wrapping an operand in a call made its type error disappear (measured 2026-08-28, `:when`
/// side). D10 reuses the same resolver from the `:then` side, so the identical hole is one edit
/// away here, and nothing but this fixture would notice it opening.
#[test]
fn a_then_value_computed_to_the_wrong_type_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D10_then_field_types_computed.wat.bad");
    assert!(
        !ok,
        "`i64::+` declares `ret` i64 — a FACT about the row, since every rete row is `total` — so \
         writing it into a String field is a refusal. If this program RAN, wrapping an operand in \
         a call makes its type error vanish, which is exactly the defect the `:when` side already \
         paid for once\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D10_then_field_types__computed.edn",
        "`RhsFieldTypeMismatch` must name the computed operand's own type (`i64`), rendered as \
         wat source and not as Rust `Debug`"
    );
}
