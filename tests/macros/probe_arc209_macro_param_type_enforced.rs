//! Arc 209 — ENFORCE macro param types. A macro receives unevaluated SYNTAX, so every param
//! binds a `:wat::WatAST` form, never a runtime value. The mandatory-then-discarded param type
//! annotation must be CHECKED at macro-def time: a non-AST type (a lie like `x <- :i64`) is
//! rejected. BEFORE (HEAD): silently accepted (the bug). AFTER: a clean MalformedDefmacro.
//!
//! Wat source lives in the NEGATIVE fixture:
//! tests/macros/probe_arc209_macro_param_type_enforced.wat.bad
//! (loaded via startup_from_file — must fail).
//!
//! Run: cargo test --release -p wat --test probe_arc209_macro_param_type_enforced

use wat::freeze::startup_from_file;

// ENFORCE landed (arc 251.5 / 209): macro-def now REJECTS a lying `<- :i64` at definition
// time. This gate flipped RED→GREEN when the validator landed in src/macros/parse.rs.
#[test]
fn lying_macro_param_type_is_rejected_at_macro_def() {
    let r = startup_from_file(
        "tests/macros/probe_arc209_macro_param_type_enforced.wat.bad",
    );
    // AFTER (ENFORCE): macro-def must REJECT a non-:wat::WatAST param type.
    // BEFORE (HEAD): Ok — silently accepted (the mandatory-then-discarded bug). RED.
    match &r {
        Err(e) => {
            let msg = format!("{e}");
            // Measure the diagnostic: it must name the param + say macro params are forms.
            wat::assert_edn_matches_file!(
                msg,
                "probe_arc209_macro_param_type_enforced__lying_macro_param_type_is_rejected_at_macro_def.edn",
                "arc209: lying macro param type golden"
            );
        }
        Ok(_) => panic!(
            "BEFORE/RED: macro-def silently ACCEPTED `x <- :wat::core::i64` — a macro param always \
             binds a form; its type must be :wat::WatAST. The annotation is mandatory-then-discarded."
        ),
    }
}
