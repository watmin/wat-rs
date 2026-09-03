//! **D11 — THE `:then` RHS TYPES ITS FIELD VALUES AT EVERY DEPTH, NOT JUST THE TOP ONE.**
//!
//! D10 closed the top level of a `:then` fact form: a value whose type is knowable and wrong is
//! refused. The identical flaw survived one level down. Driven at HEAD `f87bb070b` — the commit
//! immediately after D10's cure — against the CURED binary:
//!
//! ```text
//! :then [(:nh::Outer :i (:nh::Inner :n ?s))]     ?s : String, :nh::Inner.n : i64
//!   ->  "Outer count:" / 1
//!       #wat.core/PersistentVector [#wat.core/PersistentMap
//!         {"?f" #nh/Outer {:i #nh/Inner {:n "nested-string"}}}]
//! ```
//!
//! A wrong-typed value still reached the **fact set** — where joins, queries, the oracle and
//! `explain` all trust the declared schema. Same class, same consequence, one level deeper.
//!
//! ## The cause, and it was one missing parameter
//!
//! `walk_nested_constructors` (`src/rete/validate/mod.rs`) took `(operand, rule_name, types,
//! errors)` — **no `binds`**. `resolve_operand_type` needs `binds` to type a `?var`, so the walker
//! could only ever check field NAMES, arity and missing fields. Threading `binds` through its
//! seven call sites and calling D10's own `check_then_field_type` per kv pair is the whole cure:
//! no new error kind (`RhsFieldTypeMismatch` is the same claim at a different position), no
//! `typing.rs` change, no engine change.
//!
//! ## ★ The invariant this file pins
//!
//! > A `:then` field value whose type is **knowable** and does not match the destination field's
//! > declared type is refused at rule-compile time — **at ANY nesting depth**.
//!
//! ## ⛔ Why the not-knowable fixture is the load-bearing one
//!
//! Identical to D10's reason, now at depth. `OperandType` distinguishes *knowable-and-wrong* from
//! *not-knowable*, and a cure that refuses everything it cannot type passes **every refusal probe
//! below and the control too**, while stopping a corpus of legal rules from compiling. D10 proved
//! this is not theoretical: making `ComputedNotDerivableHere` a refusal took four pre-existing
//! corpus tests down with it. So `..._notknowable.wat` *constructs* five such nested operands and
//! requires them to compile, fire, and derive checked values.
//!
//! ## ⛔ And why the `match`-arm-body fixture is the second load-bearing one
//!
//! This walker is the one D5 (`strike-match-arm-is-not-a-call`) taught to skip a match arm's
//! **pattern** and recurse into its **body**. That recursion is one of the five internal sites
//! `binds` had to reach, and a cure that reached it with an empty map — or did not reach it —
//! leaves every other row here green. `..._match_body.wat.bad` is the only file that types a
//! `?var` inside an arm body, and `experiri-then-match.wat` (driven by
//! `probe_arc278_match_arm_is_not_a_call.rs`) is what still says the PATTERN is skipped rather
//! than typed.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately — a refusal's `Span` carries
/// `:file` verbatim, so an absolute path would make the diagnostic machine-dependent and no `.edn`
/// golden over it could ever be checked in. Same reason `probe_arc278_D10_then_field_types.rs`
/// and `probe_arc278_nested_wall.rs` state.
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
/// Values, not counts: a count of 1 is exactly what the D11 repro produced while the value nested
/// inside it was `"nested-string"`. Six well-typed nested shapes, one per producer the wall now
/// reaches — a bound `?var`, a literal, a computed operand, the nested POSITIONAL spelling, depth
/// 2, and a constructor inside a `match` arm body — and each must still compile, fire, and carry
/// the value it was given.
#[test]
fn every_well_typed_nested_then_value_still_compiles_and_derives() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D11_nested_then_field_types_ok.wat");
    assert!(
        ok,
        "the control must run — every NESTED `:then` value in it is well typed\n{out}{err}"
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec!["\"7\"", "\"42\"", "\"8\"", "\"7\"", "\"7\"", "\"100\""],
        "the control's DERIVED VALUES drifted — a nested `?var` (7), a nested literal (42), a \
         nested computed `i64::+` (8), the nested POSITIONAL spelling (7), depth 2 (7) and a \
         constructor inside a `match` arm body (100). A count would not have seen this; the \
         defect D11 cures is a wrong VALUE at the right count\n{out}{err}"
    );
}

/// ★ MUTATION 2 — the row that stands between this cure and one that refuses everything at depth.
///
/// Five CONSTRUCTED not-knowable nested operands, each a different reason:
///   nk1  a computed operand under a `Form` head (`cond`), nested — `ComputedNotDerivableHere`
///   nk2  a nested field declared as a RECORD — no rete segment to compare at
///   nk3  a constructor as the value of a nested constructor's field — depth 2, passing side
///   nk4  a `?var` bound from a DERIVED fact, consumed by the nested constructor — knowable and
///        RIGHT, and it must not be skipped
///   nk5  an enum-variant keyword nested — the class `rhs_operand_can_never_resolve` skips
///
/// All five must compile, fire, and derive the values asserted. If this test goes red while the
/// four refusals below stay green, the cure has started refusing what it merely cannot type — the
/// named failure of this strike.
#[test]
fn a_not_knowable_nested_then_operand_still_compiles_and_derives() {
    let (ok, out, err) =
        run("tests/rete/probe_arc278_D11_nested_then_field_types_notknowable.wat");
    assert!(
        ok,
        "NOT-KNOWABLE IS NOT WRONG, AT DEPTH TOO — a nested computed operand under a `Form` \
         head, a record-typed nested field, a depth-2 constructor, a `?var` bound from a derived \
         fact and a nested enum-variant keyword must all still compile. A refusal here is the \
         failure this strike is named for\n{out}{err}"
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec!["\"11\"", "\"held\"", "\"seed\"", "\"7\"", "\"#d11n.E/B []\""],
        "the not-knowable arms compiled but derived the wrong values — passing a check is not the \
         same as producing the right fact\n{out}{err}"
    );
}

/// ARM 1/4 — THE ORIGINAL D11 REPRO: a bound `?var` of the wrong declared type, one level down.
///
/// PRE (driven at HEAD `f87bb070b`, the commit after D10's cure): compiled, fired, and
/// `#nh/Outer {:i #nh/Inner {:n "nested-string"}}` entered the fact set. The caret must land on
/// `?s` — the operand the author wrote — and the fact type named must be the NESTED one
/// (`d11b::Inner`), not the outer item's.
#[test]
fn a_nested_then_value_bound_to_a_wrongly_typed_var_is_refused() {
    let (ok, out, err) =
        run("tests/rete/probe_arc278_D11_nested_then_field_types_bound_var.wat.bad");
    assert!(
        !ok,
        "a String-bound `?var` written into an i64 field of a NESTED constructor is a \
         rule-compile refusal — if this program RAN, the nested `:then` RHS is untyped again and \
         a wrong-typed value is back in the fact set\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D11_nested_then_field_types__bound_var.edn",
        "`RhsFieldTypeMismatch` must be the ONLY finding; it must name the NESTED fact type \
         (`d11b::Inner`), BOTH types (declared `:wat::core::i64`, operand `string`), and point at \
         `?s`'s own extent"
    );
}

/// ARM 2/4 — the nested POSITIONAL producer, a different arm of the walker from the kwargs one.
///
/// A cure wired only into the nested kwargs branch leaves this half dark, exactly as D10's own
/// positional fixture exists for the same split one level up. The shape is deliberately narrow:
/// the walker's positional arm is reached only for `args.len() <= 1`, every wider positional call
/// being refused above as `RhsPositionalConstructionRetired`, so a one-field record given one
/// positional arg is the only nested positional shape there is left to type.
#[test]
fn a_nested_positional_then_with_a_wrongly_typed_arg_is_refused() {
    let (ok, out, err) =
        run("tests/rete/probe_arc278_D11_nested_then_field_types_positional.wat.bad");
    assert!(
        !ok,
        "a nested positional arg is declaration order BY DEFINITION, so a String into an i64 \
         field is a refusal — a cure wired only into the nested kwargs branch leaves this half of \
         the wall dark\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D11_nested_then_field_types__positional.edn",
        "`RhsFieldTypeMismatch` at the nested positional arg's own span, naming `d11p::One`"
    );
}

/// ARM 3/4 — DEPTH 2, and TWO findings.
///
/// The claim is "at ANY nesting depth", not "one level down": a cure that peeked exactly one
/// level would pass arms 1, 2 and 4 and leave this compiling. Two wrongly-typed values in the
/// same innermost constructor, so the golden also proves the walk does not stop at the first
/// finding — batching every finding is this validator's contract, and a golden with one entry
/// would not notice the loop breaking early.
#[test]
fn a_wrongly_typed_value_two_levels_down_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D11_nested_then_field_types_deep.wat.bad");
    assert!(
        !ok,
        "the wall is unbounded-depth, mirroring the runtime's own recursive evaluation — a \
         one-level peek is not the claim\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D11_nested_then_field_types__deep.edn",
        "TWO `RhsFieldTypeMismatch` findings from the SAME innermost constructor, each at its own \
         operand's span"
    );
}

/// ARM 4/4 — a nested constructor inside a `match` ARM BODY.
///
/// ⛔ The only row that proves `binds` reached D5's recursion. A cure that threaded the parameter
/// but passed an empty map into the `match` arm — or never recursed there — leaves arms 1-3, the
/// control and the not-knowable set all green, because none of them types a `?var` inside an arm
/// body. Its counter-check is `..._ok.wat`'s okF row (same shape, well typed, derives 100) plus
/// `experiri-then-match.wat`, which must still load — that pair is what says the arm's PATTERN is
/// still skipped rather than typed.
#[test]
fn a_wrongly_typed_value_inside_a_match_arm_body_is_refused() {
    let (ok, out, err) =
        run("tests/rete/probe_arc278_D11_nested_then_field_types_match_body.wat.bad");
    assert!(
        !ok,
        "a `match` arm BODY is walked (D5 skips only the PATTERN), so a nested constructor inside \
         one is typed like any other — if this ran, `binds` never reached that recursion\n\
         {out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D11_nested_then_field_types__match_body.edn",
        "ONE `RhsFieldTypeMismatch`, from the SECOND arm's body only — the first arm's body is \
         well typed and the arm PATTERNS must contribute nothing at all"
    );
}
