//! Arc 251.8d-ii (EIGHTH DRAW) — **the macro member join.**
//!
//! A `defmacro` name is stored by `ns_to_wat_path` (`src/macros/parse.rs`'s name slot),
//! which writes `::` ALWAYS. A keyword-spelled declaration keeps its `/` verbatim (the
//! `Keyword` payload IS the identity), but its faithful-Clojure twin does not: `user.Box/of`
//! and a hypothetical `:user::Box::of` are the SAME Clojure symbol, so the declaration
//! registers under `:user::Box::of` while every keyword-spelled caller keeps asking for
//! `:user::Box/of`.
//!
//! `freeze::env::rekey_type_member_functions` is the compensating pass for exactly this,
//! and it CANNOT reach a macro: it walks `sym.functions_iter()`, and a macro is registered
//! (step 4) and expanded (step 5) before the `TypeEnv` is attached (step 6.97). So the
//! question has to be asked at the CONSULT — which is what the Symbol arm of the
//! macro-call dispatch in `src/macros/expand.rs` already did, and the Keyword arm did not.
//! The cure is that second question, in the Keyword arm, in ONE direction (`/` → `::`).
//!
//! Measured in situ, not inferred: with `wat/core.wat` converted ALONE and rebuilt, the
//! keyword call `(:wat::core::Fault/of "boom")` — the stdlib's own, at `wat/spawn.wat:475`
//! — becomes `#wat.resolve/UnresolvedReferences :path ":wat::core::Fault/of"` on every
//! program that loads the stdlib.
//!
//! FIVE rows, ONE test. Two of them are green on BOTH binaries, which is what makes them
//! a test of the WALL rather than of the cure.
//!
//! Wat sources: `probe_arc251_8d_macro_member_join.wat` (positive) and three
//! `…_*.wat.bad` negatives (a `.bad` fixture must fail startup, so it stays out of the
//! every-tracked-wat-loads gates).
//!
//! Run: cargo test --release -p wat --test macros -- macro_member_join

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::Value;

const CURE_FIXTURE: &str = "tests/macros/probe_arc251_8d_macro_member_join.wat";
const CONTROL_FIXTURE: &str = "tests/macros/probe_arc251_8d_macro_member_join_control.wat";

/// A `.wat.bad` fixture that must fail startup, and the name its refusal has to carry.
const REFUSALS: &[(&str, &str, &str)] = &[
    (
        "tests/macros/probe_arc251_8d_macro_member_join_unknown_member.wat.bad",
        ":user::Box/nope",
        "a member no macro declares, on a TYPE parent — the spelling widened, the population did not",
    ),
    (
        "tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad",
        ":user::helper::of",
        "THE DIRECTION CONTROL — a keyword author writing the OTHER join must stay refused",
    ),
    (
        "tests/macros/probe_arc251_8d_macro_member_join_non_type_parent.wat.bad",
        ":user::helper/nope",
        "a member no macro declares, on a NON-type parent — the wall is on the NAME",
    ),
];


/// Freeze one fixture and apply one zero-arg entry from it.
///
/// ⛔ NOT `call_beside_value`: that helper PANICS when the fixture fails to freeze, and
/// row 1's fixture does not freeze pre-cure — so every later row in this test would go
/// unmeasured on exactly the binary they exist to measure. A `Result` keeps all five rows
/// readable on BOTH binaries.
fn entry_value(path: &str, entry: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(path)?;
    // A missing entry is a BROKEN FIXTURE, not an error this test reports on: panic
    // rather than mint a `Result` arm no row discriminates (arc 296 Stone M's rubric).
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("fixture {path} no longer declares {entry}"))
        .clone();
    wat::runtime::apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))
}

#[test]
fn a_faithful_macro_name_answers_to_its_keyword_spelling() {
    let mut wrong: Vec<String> = Vec::new();

    // ⭐ ROW 1 — THE CURE. The macro is declared `(wat.core/defmacro user.Box/of …)` and
    // called `(:user::Box/of 7)`. PRE-CURE this is an UnresolvedReference on
    // `:user::Box/of`; POST it expands and the record's accessor reads 7 back.
    match entry_value(CURE_FIXTURE, ":user::cure") {
        Ok(Value::i64(7)) => {}
        other => wrong.push(format!(
            "  :user::cure -> {other:?}, want i64(7) — THE CURE: a FAITHFUL-spelled \
             defmacro name called in the KEYWORD spelling"
        )),
    }

    // ⛔ ROW 2 — THE CONTROL, in its OWN fixture. The identical shape declared in the
    // keyword surface. Green on both binaries: the cure widens a SPELLING, it may not
    // move a keyword-spelled program. It cannot live beside row 1 — row 1's fixture does
    // not freeze pre-cure, so a control inside it would be unreadable on exactly the
    // binary it exists to measure.
    match entry_value(CONTROL_FIXTURE, ":user::control") {
        Ok(Value::i64(9)) => {}
        other => wrong.push(format!(
            "  :user::control -> {other:?}, want i64(9) — CONTROL: a keyword-spelled \
             defmacro name must not move"
        )),
    }

    // ⛔ ROWS 3-5 — THE WALL, in one loop. Each must REFUSE, and the refusal must NAME
    // the offending head: a startup that fails for some other reason is not this wall.
    for (path, must_name, why) in REFUSALS {
        match startup_from_file(path) {
            Ok(_) => wrong.push(format!("  {path} -> STARTED, want REFUSED — {why}")),
            Err(e) => {
                let msg = format!("{e:?}");
                if !msg.contains(must_name) {
                    wrong.push(format!(
                        "  {path} -> refused, but the reason does not name {must_name} \
                         — {why}\n      got: {msg}"
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the macro member join answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
