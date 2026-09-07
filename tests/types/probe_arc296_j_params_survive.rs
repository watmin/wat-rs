//! PROBE — arc 296 stone J: every parametric outcome enum keeps its TYPE PARAMS
//! (and their order) across the move to wat.
//!
//! H-3's probe is the model. Nine parametric types moved this stone; arity ≥ 2
//! (`AcceptOutcome [R, S]`, `ConnectOutcome [S, R]`) is where a swapped pair
//! or a fixed-offset binder bug is silent at arity 1.

use wat::types::{TypeDef, TypeEnv};

fn params_of(env: &TypeEnv, path: &str) -> Vec<String> {
    match env.get(path) {
        Some(TypeDef::Enum(e)) => e.type_params.clone(),
        other => panic!("{path} must be a registered builtin enum; got {other:?}"),
    }
}

#[test]
fn nine_parametric_outcomes_keep_params_in_declaration_order() {
    let env = TypeEnv::with_builtins();
    let cases: &[(&str, &[&str])] = &[
        (":wat::eval::WalkStep", &["A"]),
        (":wat::edn::ReadJsonOutcome", &["T"]),
        (":wat::edn::ReadForeignOutcome", &["T"]),
        (":wat::kernel::ReadlnOutcome", &["T"]),
        (":wat::eval::FormOutcome", &["T"]),
        (":wat::kernel::RecvOutcome", &["O"]),
        (":wat::stream::NextOutcome", &["T"]),
        (":wat::kernel::AcceptOutcome", &["R", "S"]),
        (":wat::kernel::ConnectOutcome", &["S", "R"]),
    ];
    for (path, want) in cases {
        let got = params_of(&env, path);
        let want: Vec<String> = want.iter().map(|s| (*s).to_string()).collect();
        assert_eq!(
            got, want,
            "{path} lost or reordered type params — Accept is [R,S], Connect is [S,R]"
        );
    }
}
