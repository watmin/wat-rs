//! FM-2-bis DIAGNOSTIC PROBE — Stone 249.5d (ArgSpec carries the Identifier).
//!
//! Does a macro-generated `defclause` WITH a rest param (`& rest <- :T`) resolve
//! its fixed params at call time? This is the disconfirming contract for the
//! ArgSpec strip-and-re-walk root fix.
//!
//! Wat source lives in the co-located fixture: probe_argspec_rest_param_hygiene.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release --test probe_argspec_rest_param_hygiene -- --nocapture

use wat::freeze::call_beside;
use wat::runtime::Value;

/// REST-PARAM HYGIENE GUARD — a macro-generated defclause WITH a rest param must
/// resolve its scope-tagged fixed params at call time.
///
/// At HEAD the `% 3` guard in `scoped_arg_names` ejects the rest-binder argspec,
/// baring the fixed-param bind keys while the body looks them up scoped →
/// `UnboundSymbol`. Stone 249.5d (ArgSpec carries the Identifier; `env_key` over
/// the parsed identifiers; the re-walk deleted) makes bind-key == lookup-key.
///
/// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
/// call_beside — no inline wat driver expression.
#[test]
fn macro_generated_defclause_with_rest_resolves_params() {
    let result = call_beside(file!(), ":user::compute").expect(
        "macro-generated defclause WITH a rest param must freeze without UnboundSymbol; \
         failure = the `% 3` guard bared the fixed params while \
         the scope-tagged body looked them up scoped (Stone 249.5d root fix)",
    );
    assert_eq!(
        result,
        Value::i64(10),
        "REST HYGIENE: macro-generated defclause body must resolve x, y AND rest \
         (1 + 2 + 3 + 4 = 10). Got {:?}",
        result
    );
}
