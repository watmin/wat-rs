//! Arc 278 #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05).
//!
//! ★ INVERTED. `probe_arc278_repl_durable_forms_response_law.wat.bad` (formerly `wat-scripts/
//! scratch-pad/probe-repl-durable-forms.wat`) is R64's own probe: `EvalResponse` for op
//! `eval-src` is the divergent name R64 caught a guess getting wrong. Under the ruling, "a
//! response type's name need not echo its op's" is no longer true — the declaration itself is
//! the defect now, and this test is the proof that it is UNREPRESENTABLE: the surface is
//! REFUSED at registration, located, naming both the declared name and the required one.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

/// R64's defect, unrepresentable: `:probe::Repl::eval-src -> EvalResponse` must be REFUSED at
/// `defsurface` registration — located, naming both `:probe::Repl::EvalResponse` (declared) and
/// `:probe::Repl::EvalSrcResponse` (required).
#[test]
fn repl_eval_response_name_is_refused_at_registration() {
    match startup_from_file(
        "tests/services/probe_arc278_repl_durable_forms_response_law.wat.bad",
    ) {
        Ok(_) => panic!(
            "expected `:probe::Repl::eval-src -> EvalResponse` to be REFUSED at registration \
             (arc 278 #74 — `<Op>Response` is law); it froze clean instead, which means R64's \
             defect is representable again"
        ),
        Err(StartupError::Type(e)) => match e.into_kind() {
            TypeErrorKind::MalformedDecl { head, reason } => {
                assert_eq!(head, ":wat::core::defsurface");
                assert_eq!(
                    reason,
                    "op `eval-src` in surface :probe::Repl: response type name is LAW — \
                     declared `:probe::Repl::EvalResponse`, required \
                     `:probe::Repl::EvalSrcResponse` (arc 278 #74, builder ruling 2026-08-05: \
                     an op's response type IS `<Op>Response`; rename the declaration to match)"
                );
            }
            other => panic!(
                "expected MalformedDecl (the #74 response-name-law refusal); got {other:?}"
            ),
        },
        Err(other) => panic!(
            "expected StartupError::Type (the #74 response-name-law refusal); got {other:?}"
        ),
    }
}
