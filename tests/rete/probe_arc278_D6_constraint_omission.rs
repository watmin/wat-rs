//! **D6 — `explain` MUST NOT SILENTLY DROP A CONSTRAINT.**
//!
//! `eval_step_payload` builds the `constraints` field of a `DerivationStep` — the user-facing
//! *why did this fire* surface — under a doc that promised *"the rule's satisfied predicates with
//! bound values substituted"* and, in bold, **"Faithfulness by construction"**. At HEAD
//! `c9bb8044b` a rule with two satisfied constraints produced a payload with one, and nothing
//! anywhere said so.
//!
//! ## The two stacked gates, and why one arm cannot separate them
//!
//! The enum constraint was dropped by the FIRST of two silent `continue`s:
//!
//! 1. the payload builder passed `sym: None` to `resolve_operand`, so a unit-variant keyword in
//!    direct operand position (`:d6u::Grade::Hi`) resolved to no value at all — `resolve_operand`
//!    needs the `SymbolTable` to tell a unit variant from a plain keyword;
//! 2. `value_to_ast_literal` had no `Value::Enum` arm, so a variant that DID resolve still had no
//!    spelling and was dropped at the very next line.
//!
//! Thread the symbol table and stop, and the constraint is still absent — the drop has moved one
//! line down. That is why the assertions here are over the PAYLOAD and never over which internal
//! path was taken: only the payload can tell a two-gate fix from a one-gate one.
//!
//! ## The class, not just the instance
//!
//! The deeper defect is the bare `continue`: a caller cannot distinguish a shortened vector from a
//! rule that genuinely had fewer constraints. The cure is positional — every inline constraint
//! clause gets exactly one entry, and one that cannot be rendered is spelled
//! `(:wat::rete::explain::constraint-not-rendered <op> <operand-index> "<why>")` instead of
//! vanishing. `a_tagged_enum_operand_is_named_not_dropped` is the arm that drives it, over the one
//! residue that is still deliberately unrenderable (a TAGGED enum variant: never a literal the
//! author wrote, and with two defensible spellings and nothing here to choose between them).
//!
//! That "one residue" is a claim, so it has its own arm.
//! `a_non_comparable_operand_is_walled_at_freeze_not_left_to_the_payload` drives the case that
//! would widen it — a bound var of a type rete has no comparator for — and pins that a freeze
//! wall refuses the rule outright. The rete equality surface is i64/f64/string/bool/keyword/enum;
//! five of those six render, and the sixth splits into unit (renders) and tagged (marker). That is
//! the whole space, and each half of it is driven here.
//!
//! ## Why goldens and not a length check
//!
//! A `constraints.length == 2` assertion passes on two markers, on the wrong operator, and on an
//! unsubstituted `?g`. The whole rendered vector is deterministic — no path, no pid, no span in
//! the face — so the exact `.edn` golden is available and is what is asserted.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, matching `probe_arc278_field_span.rs`:
/// keeping the invocation machine-independent is what lets an exact golden be checked in at all.
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

/// ★ THE ROW. A condition carrying `i64::>` and `enum::=` puts BOTH in the payload, each with its
/// bound value substituted.
///
/// The `i64` clause is the control and the enum clause is the subject, on the SAME condition and
/// in the SAME payload — so "the probe reached the wrong step" and "the engine dropped one" are
/// not confusable. The enum operands render as the keyword path the author wrote, which
/// `expr_ir::keyword_value` reads straight back to the same `Value::Enum`.
///
/// Mutation-proved both ways: reverting the `sym` thread OR the `Value::Enum` arm (either one
/// alone, keeping the other) turns this RED with the enum constraint gone.
#[test]
fn a_unit_enum_constraint_reaches_the_explain_payload() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D6_constraint_omission_unit.wat");
    assert!(ok, "the D6 unit-variant fixture must run to completion\n{out}{err}");
    wat::assert_edn_matches_file!(
        out.trim().to_string(),
        "probe_arc278_D6_constraint_omission__unit.edn",
        "both satisfied constraints must be in the payload with their bound values substituted — \
         at HEAD only the i64 one was, and the enum one vanished with no diagnostic"
    );
}

/// THE RESIDUE ARM — a constraint that still cannot be rendered is NAMED, never dropped.
///
/// A tagged enum variant bound from a fact field has no literal spelling, so the payload keeps the
/// clause's position with the omission marker carrying the op, the failing operand's index and the
/// reason. Length stays 2; the `i64::>` control still renders normally beside it.
///
/// This is the arm that proves the strike bought a PROPERTY and not one operand type: the next
/// unrenderable `Value` variant lands here too, by construction, instead of disappearing.
#[test]
fn a_tagged_enum_operand_is_named_not_dropped() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D6_constraint_omission_tagged.wat");
    assert!(ok, "the D6 tagged-variant fixture must run to completion\n{out}{err}");
    wat::assert_edn_matches_file!(
        out.trim().to_string(),
        "probe_arc278_D6_constraint_omission__tagged.edn",
        "an unrenderable constraint must hold its position with a marker naming op, operand and \
         reason — a shorter vector is indistinguishable from a rule with fewer constraints"
    );
}

/// THE BOUND ON THE RESIDUE — and the arm that keeps the class claim honest.
///
/// The tagged enum variant is described above as *the* live residue. That is a claim about what
/// else can reach `value_to_ast_literal` unspellable, and `validate/typing.rs`'s header reads as
/// though a great deal can: a constraint operand is schema-checked only when it is a `:field`
/// reference, so "a `?var` (free or bound) and a literal are left alone".
///
/// Driven, a separate freeze wall — `ConstraintTypeNotComparable` — refuses a bound `?v` whose
/// declared type has no rete comparator, before any payload is built. That wall is what makes the
/// residue exactly one case rather than open-ended, so it is asserted here rather than assumed.
/// If it is ever relaxed this arm goes RED, and the tagged arm alone would stop being the whole
/// story.
#[test]
fn a_non_comparable_operand_is_walled_at_freeze_not_left_to_the_payload() {
    let (ok, out, err) = run("tests/rete/probe_arc278_D6_constraint_omission_nonenum.wat.bad");
    assert!(
        !ok,
        "a bound var whose declared type has no rete comparator must be a freeze refusal — if \
         this program runs, an unspellable non-enum value now reaches the explain payload and the \
         residue is wider than the tagged-enum arm above claims\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_D6_constraint_omission__nonenum_refusal.edn",
        "the refusal must be ConstraintTypeNotComparable naming the operand and its declared type \
         — any other refusal means this fixture is being rejected for an unrelated reason and \
         bounds nothing"
    );
}
