//! Arc 209 — ENFORCE macro param types. A macro receives unevaluated SYNTAX, so every param
//! binds a `:wat::WatAST` form, never a runtime value. The mandatory-then-discarded param type
//! annotation must be CHECKED at macro-def time: a non-AST type (a lie like `x <- :i64`) is
//! rejected. BEFORE (HEAD): silently accepted (the bug). AFTER: a clean MalformedDefmacro.
//!
//! Run: cargo test --release -p wat --test probe_arc209_macro_param_type_enforced

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// A macro whose param claims `:wat::core::i64` — a lie: a macro param is always a form.
const LYING_PARAM: &str = r#"
(:wat::core::defmacro :user::bad [x <- :wat::core::i64] -> :wat::WatAST x)
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// RED at HEAD (measured: macro-def silently ACCEPTS a lying `<- :i64`). This is the gate for the
// QUEUED ENFORCE-macro-param-type stone — ignored so the suite stays green until the validator
// lands; un-ignore it when drawing that stone (it flips RED→GREEN).
#[ignore = "queued ENFORCE-macro-param-type stone: un-ignore when the validator lands"]
#[test]
fn lying_macro_param_type_is_rejected_at_macro_def() {
    let r = startup_from_source(LYING_PARAM, None, Arc::new(InMemoryLoader::new()));
    // AFTER (ENFORCE): macro-def must REJECT a non-:wat::WatAST param type.
    // BEFORE (HEAD): Ok — silently accepted (the mandatory-then-discarded bug). RED.
    match &r {
        Err(e) => {
            let msg = format!("{e}");
            // Measure the diagnostic: it must name the param + say macro params are forms.
            assert!(
                msg.contains(":wat::WatAST") && (msg.to_lowercase().contains("param") || msg.contains('x')),
                "rejection message should name the offending param + the required :wat::WatAST type; got:\n{msg}"
            );
        }
        Ok(_) => panic!(
            "BEFORE/RED: macro-def silently ACCEPTED `x <- :wat::core::i64` — a macro param always \
             binds a form; its type must be :wat::WatAST. The annotation is mandatory-then-discarded."
        ),
    }
}
