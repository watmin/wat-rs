//! strike-nested-wall — **AN ERROR KIND NOTHING CAN PRODUCE IS A PROMISE THE SYSTEM DOES NOT KEEP.**
//!
//! `walk_nested_constructors` (`src/rete/validate/mod.rs`) matched a record type as the **HEAD** of
//! a nested `:then` operand. `defrecord`'s companion macro lowers every record-constructor call
//! before freeze (`src/macros/parse.rs:343`, `(:wat::core::kwargs-construct ~_kc-type ~@call-args)`),
//! so the head the wall actually receives is the MACRO's and the type sits at **index 1**.
//! `types.get(":wat::core::kwargs-construct")` is `None`, the aggregate branch never opened, and
//! **four** error kinds were unreachable there: `UnknownField`, `RhsMissingFields`,
//! `RhsArityMismatch`, `RhsPositionalConstructionRetired`.
//!
//! ## The mechanism was ORPHANING, not oversight
//!
//! The walker was correct when written. The fix that made a nested constructor *work* introduced
//! the lowering that darkened it, and three sibling subsystems were re-pointed while this one was
//! not (`purity.rs:349`, `purity.rs:829`, `kernel/stratify.rs:517`, `expr_ir/mod.rs:547`). It never
//! looked dead because the walker's **enum-variant** sibling branch is live — an enum variant is
//! not lowered — so the function is exercised from outside and only the lowered arms were gone.
//!
//! ## Why four fixtures and not one
//!
//! **This file is the half that outlives the fix.** Re-pointing the walker closes today's hole and
//! leaves the *next* lowering free to open it again; nothing in the tree notices when an error kind
//! stops being producible. Each of the four kinds therefore gets a fixture that DRIVES it, tuned so
//! that kind fires **alone** — supply the declared field where only the undeclared one should be
//! reported, name only declared fields where only the missing one should be. A single fixture
//! driving two kinds would redden for both arms' mutations and prove neither separately.
//!
//! Every refusal is asserted as an `.edn` golden carrying the whole `Span` (`:line`, `:col`,
//! `:end`), following `probe_arc278_field_span.rs`: the kind alone would pass on a caret pointing
//! at the wrong text, and after a strike that makes a wall refuse for the first time, a
//! plausible-looking wrong span is the failure most likely to pass unnoticed.
//!
//! ## `:wat::core::aggregate-new` is deliberately absent — driven, not assumed
//!
//! `purity.rs` and `stratify.rs` both pair `kwargs-construct` with `aggregate-new`, so mirroring
//! them is the reflex. Every surface spelling was driven at this wall instead: the kwargs sugar,
//! the single-arg positional, the multi-arg positional, and a positionally-written OUTER item all
//! arrive as `kwargs-construct`; the positional prime `:T'` arrives **un-lowered** under its own
//! primed head, which `types.get` does not resolve. Nothing lowers to `aggregate-new` here. An arm
//! for it would be dead code minted fresh — and worse than dead, since `aggregate-new` *is* the
//! positional route, so `RhsPositionalConstructionRetired` would be an actively wrong refusal there.

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately — a refusal's `Span` carries
/// `:file` verbatim, so an absolute path would make the diagnostic machine-dependent and no `.edn`
/// golden over it could ever be checked in. Same reason `probe_arc278_field_span.rs` states.
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

/// THE CONTROL, and it is load-bearing for all four kind-probes below.
///
/// The same shape they misspell, spelled correctly: a two-field nested constructor written as
/// kwargs. Without it, "the fixture refused" is indistinguishable from "the fixture was malformed
/// in some way I did not intend" — and a strike whose whole effect is to make a wall start refusing
/// is exactly where that confusion passes for a result. It must COMPILE, FIRE, and derive one fact.
#[test]
fn a_correctly_spelled_nested_constructor_still_compiles_and_fires() {
    let (ok, out, err) = run("tests/rete/probe_arc278_nested_wall_ok.wat");
    assert!(ok, "the control must run — every nested field name in it is real\n{out}{err}");
    assert_eq!(
        out.trim(),
        "1",
        "one seeded `:nwo::Src` matches and derives one `:nwo::Outer` — a count that is not 1 \
         means the control drifted and the four arms below prove nothing\n{out}{err}"
    );
}

/// KIND 1/4 — `UnknownField` at the NESTED-CONSTRUCTOR producer.
///
/// PRE (measured at HEAD `26c79470c`): this program compiled and ran. The nested field `:nope` is
/// undeclared and nothing looked.
///
/// The fixture supplies the declared `x` on purpose, so `RhsMissingFields` has nothing to report
/// and this kind fires alone. The caret must be `:nope`'s own extent, not the nested form's —
/// `check_field_kw` takes the keyword NODE, and the lowering preserves source spans, so the span
/// asserted here is the one the author typed.
#[test]
fn a_nested_constructor_naming_an_undeclared_field_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_nested_wall_unknown_field.wat");
    assert!(
        !ok,
        "a nested constructor naming an undeclared field is a freeze refusal — if this program \
         RAN, the wall is reading `items[0]` again and the lowered head has orphaned it a second \
         time\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_nested_wall__unknown_field.edn",
        "`UnknownField` must be the ONLY finding, and its caret must be `:nope`'s own extent"
    );
}

/// KIND 2/4 — `RhsMissingFields` at the NESTED-CONSTRUCTOR producer.
///
/// PRE: this program compiled and ran; the nested `:nwm::Inner` was built one field short and the
/// omission surfaced only if something later read `y` by name.
///
/// Every field NAME written here is declared, so `UnknownField` cannot fire and this kind stands
/// alone. Its span is the whole nested form and is asserted so **on purpose**: "missing `y`" is a
/// property of the form, not of any one keyword in it — the same distinction
/// `probe_arc278_field_span.rs` records for the top-level producer.
#[test]
fn a_nested_constructor_under_supplying_a_declared_field_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_nested_wall_missing_fields.wat");
    assert!(
        !ok,
        "a nested constructor that omits a declared field is a freeze refusal\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_nested_wall__missing_fields.edn",
        "`RhsMissingFields` must be the ONLY finding, naming `y`, spanning the whole nested form"
    );
}

/// KIND 3/4 — `RhsArityMismatch` at the NESTED-CONSTRUCTOR producer, which is the
/// **single-arg positional passthrough** arm (`args.len() <= 1`), mirroring
/// `eval_kwargs_construct`'s own `rest.len() <= 1` route to `construct_aggregate`.
///
/// ⚠ This kind has TWO producers in this one walker. The other is the **enum-variant** branch,
/// which was always live because an enum variant is not lowered — which is precisely why the
/// function looked exercised while its aggregate arms were dark. This fixture drives the
/// AGGREGATE one. A mutation to either producer must leave the other's probe green; that is what
/// makes them two producers rather than one, and it is checked by mutating each in turn.
///
/// The compiled RHS path independently refuses this shape at fire (`expr_ir::lower_construct`:
/// `fields.len() != names.len()`), so here the two agree and the wall merely moves the refusal
/// earlier, from fire to freeze, with a message that names the rule and the type.
#[test]
fn a_nested_single_positional_arg_against_a_wider_record_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_nested_wall_arity.wat");
    assert!(
        !ok,
        "one positional value cannot construct a two-field record — a freeze refusal\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_nested_wall__arity.edn",
        "`RhsArityMismatch` must be the ONLY finding, expected 2 / got 1, at the AGGREGATE \
         producer — the enum-variant producer of the same kind must not be what fired"
    );
}

/// KIND 4/4 — `RhsPositionalConstructionRetired` at the NESTED-CONSTRUCTOR producer.
///
/// ⛔ **THIS KIND'S OWN DOC IS FALSIFIED BY A DRIVE, AND THAT IS RECORDED HERE RATHER THAN FIXED.**
///
/// `validate/error.rs` says of this variant: *"Once #1 wires a nested constructor to actually reach
/// `:wat::core::kwargs-construct`'s dispatch (`eval_kwargs_construct`, runtime.rs), that dispatch
/// unconditionally retires multi-arg RAW POSITIONAL construction."* Driven at HEAD `26c79470c`, the
/// fixture's exact form **compiled, fired, and derived a correctly-valued fact** — a nested
/// `(:T ?k 99)` produced `y = 99`, checked by value and not by count. Rete fire never reaches that
/// dispatch: `rhs_must_compile` (`kernel/arm.rs`) refuses to walk `build_insert_fact` at all, and
/// the compiled path lowers through `expr_ir::lower_construct`, whose `rete_kwargs_value_asts`
/// treats positional args as declaration order (*"positional is already declaration order BY
/// DEFINITION"*) and constructs happily. The retirement arm is on the INTERPRETER path, which rete
/// fire does not take.
///
/// So this refusal is not a wall catching what fire would catch anyway — it is this wall enforcing
/// a doctrine the rete fire path does not. That is a real behaviour change and it is deliberate:
/// the variant keeps its shape (the strike that wired it affirmatively cut changing what the kinds
/// MEAN). A corpus sweep of all 1650 `.wat` files — 460 `:then` clauses — found **zero** existing
/// uses of this shape, so nothing in the tree relies on the acceptance being withdrawn.
#[test]
fn a_nested_multi_arg_positional_construction_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_nested_wall_positional_retired.wat");
    assert!(
        !ok,
        "multi-arg positional construction at a bare aggregate name is retired doctrine — this \
         wall is the only place on the rete path that enforces it, so if this program RAN the \
         enforcement is gone entirely\n{out}{err}"
    );
    wat::assert_edn_matches_file!(
        err.trim().to_string(),
        "probe_arc278_nested_wall__positional_retired.edn",
        "`RhsPositionalConstructionRetired` must be the ONLY finding — in particular NOT \
         `RhsArityMismatch`, whose \"expected N\" framing would misstate a refusal that stands \
         even when the count is right"
    );
}
