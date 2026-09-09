//! PROBE — a variant is not a type, and the two programs the builder wrote cannot be written.
//!
//! Measured on the green tree at `f85644b26`, using the builder's OWN examples as fixtures:
//!
//! ```text
//!   (defn :user::process-full-box :- [T] [full-box <- (:usr::Box::Full :- [:T])] -> :T
//!     (let [{:keys [inside]} full-box] inside))                       check=1  ⛔
//!
//!   (let [full-box (:usr::Box::Full {:inside 42})] (:user::takes-full full-box))
//!                                                                     check=1  ⛔
//!
//!   (match full-box [:usr::Box::Full {:inside inside} …] …)           check=0  already green
//! ```
//!
//! ★ **Only the annotation and the constructor are missing.** `match` on a variant already works;
//! this stone does not touch it.
//!
//! ## ⛔ THE ROW P-2a DID NOT HAVE, AND WHY IT WAS REVERTED
//!
//! P-2a registered variants as types and left the ctor erasing. All five of its rows passed — in
//! exactly the state its own DESIGN called *"strictly worse than today's refusal"* — because every
//! row ANNOTATED with the variant type and none CONSTRUCTED a value into it. The parameter was
//! accepted and uninhabitable: you write `Box.Full`, construct `Box.Full`, and are told you supplied
//! a `Box`.
//!
//! `ctor_carries_the_variant` is that missing row. **A probe that proves a type is USABLE must
//! construct into it.** `[[feedback_a_green_test_can_prove_nothing]]`
//!
//! ## What the measurements already settled, so this stone does not re-litigate them
//!
//! ```text
//!   the erasure          src/declare/register.rs — `ret_type: enum_type.clone()`. ONE LINE.
//!                        `infer_enum_map_ctor`'s fallback is NOT the live path; the registered
//!                        SCHEME is. (Measured: editing the fallback changed nothing.)
//!   the widening         head-level subtype edges ALREADY work — (derive Child Parent) lets a
//!                        Child<i64> reach a Parent<i64> slot, and args-differ is refused. A
//!                        variant edge is head-level, so NO new `assignable` arm is needed.
//!   {:keys}              asks `TypeDef::Aggregate`. ⛔ The fix is to widen the PREDICATE by SHAPE
//!                        — "does this carry named fields?" — exactly as Stone O widened it from
//!                        `nature == Struct`. NOT to register variants as aggregates: that is
//!                        shaping the TYPE to fit a PREDICATE. Builder's ruling.
//! ```
//!
//! The three subject tests are `#[ignore]`d and are UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_check(case: &str) -> (i32, String) {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_A2_a_variant_is_a_type__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check").arg(&p)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().expect("spawn wat --check");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), s)
}

fn check(case: &str) -> i32 { run_check(case).0 }

/// ⛔ THE WIDEST CONTROL. Every existing enum construction in the corpus flows where the ENUM is
/// expected. The ctor's type changes under this stone; if widening does not carry it, this row goes
/// red first and everything after it is noise.
#[test]
fn a_variant_value_still_flows_where_the_enum_is_expected() {
    assert_eq!(check("variant_widens_to_enum"), 0);
}

/// REGRESSION CONTROL — the builder's match example, green today.
#[test]
fn matching_on_a_variant_still_works() {
    assert_eq!(check("match_still_works"), 0);
}

/// ⛔ OVER-REACH DETECTOR — `:usr::Box::Nope` is not a variant of `Box`. P-1b is what made this
/// row meaningful; before it, a parametric head was never validated and this checked clean.
#[test]
fn a_nonexistent_variant_is_refused() {
    assert_eq!(check("nonexistent_variant"), 1);
}

/// ⛔ THE ROW THAT CATCHES A SCOPE CUT. A STDLIB enum's variant. RELAND-1 exists because every
/// other fixture spells `:usr::Box`, so scoping `register_variant_types` to `!is_reserved_prefix`
/// passed all fourteen rows while excluding `Option`, `Result` and every service `Op`/`Reply` —
/// the population the capability is FOR.
#[test]
#[ignore = "arc 296 A-2 — the ctor erases; and a user-only scope would leave this red"]
fn a_stdlib_enums_variant_is_a_type_too() {
    let (code, out) = run_check("stdlib_enum_variant");
    assert_eq!(code, 0, "(:wat::core::Option::Some {{:value 42}}) must satisfy an \
        (:wat::core::Option::Some :- [i64]) parameter; got: {out}");
}

/// ⛔⛔ THE JOIN CONTROL — GREEN TODAY AND MUST STAY GREEN. The builder's canonical conditional
/// option: two SIBLING variants in one form. Neither is a subtype of the other; their join is
/// `Option`. It passes now only because the ctor ERASES both branches to the same type. **600 of
/// RELAND-0's 103 floor failures were this shape, and no fixture had it.** This row is not a
/// subject — it is the row that can only fail if the stone breaks it.
#[test]
fn two_sibling_variants_still_join_in_an_if() {
    let (code, out) = run_check("sibling_variants_join");
    assert_eq!(code, 0, "(if b (Option::Some …) (Option::None …)) is the canonical conditional \
        option; got: {out}");
}

/// ⛔ THE SAME GAP THROUGH `match` — 277 of the 600 arrived via match arms, not `if`.
#[test]
fn two_sibling_variants_still_join_across_match_arms() {
    let (code, out) = run_check("sibling_variants_join_in_match");
    assert_eq!(code, 0, "got: {out}");
}

/// SUBJECT — the builder's `process-full-box`: a function that takes ONLY full boxes and
/// destructures the payload directly, with no match ceremony to reach a field it has already proved
/// is there.
#[test]
#[ignore = "arc 296 A-2 — a variant is not a registered type, and {:keys} asks for an Aggregate"]
fn a_function_can_take_only_one_variant_and_destructure_it() {
    let (code, out) = run_check("process_full_box");
    assert_eq!(code, 0, "got: {out}");
}

/// ⛔⛔ SUBJECT, AND THE ROW P-2a LACKED. Constructs a value and passes it INTO a variant-typed
/// parameter. This is the only row that can tell "the type exists" from "the type is inhabited",
/// and its absence is why P-2a shipped an uninhabitable parameter and had to be reverted.
#[test]
#[ignore = "arc 296 A-2 — the ctor erases to the enum, so nothing can satisfy a variant parameter"]
fn the_constructor_carries_the_variant_type() {
    let (code, out) = run_check("ctor_carries_the_variant");
    assert_eq!(
        code, 0,
        "(:usr::Box::Full {{:inside 42}}) must satisfy a (:usr::Box::Full :- [i64]) parameter; \
         got: {out}"
    );
}

/// ⛔ SUBJECT **and** the row a defect satisfies. It exits 1 TODAY because the annotation names an
/// unregistered type. After the stone it must exit 1 for the opposite reason: a `Box` is not known
/// to be a `Box::Full`. Same exit code, different mechanism — so the bar is the MESSAGE. A stone
/// that registered variants as ALIASES of their enum would flip this to 0 and pass every other row.
#[test]
#[ignore = "arc 296 A-2 — refused today as an unknown type, not as a direction violation"]
fn an_enum_value_does_not_flow_into_a_variant_parameter() {
    let (code, out) = run_check("enum_does_not_narrow");
    assert_eq!(code, 1, "got: {out}");
    // rune:lint(loose-assert) — the value is a CheckErrors EDN blob carrying the fixture's ABSOLUTE
    // path and a span; it cannot be assert_eq!'d and a golden would bake in this machine's home
    // directory. Targeted ABSENCE over a per-run-varying output — the rubric's own exemption. The
    // exit code is 1 both before and after this stone, so WHICH diagnostic appears is the assertion.
    assert!(
        !out.contains("UnknownNamedType"),
        "after the stone this must be a direction mismatch, not an unknown-type refusal; got: {out}"
    );
}
