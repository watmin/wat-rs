//! Strike (examinare probe) — 4.0: the faithful type-NAMESPACE fix (intueri-named).
//!
//! RED at HEAD: C03/C04/C05/C06 fail (mis-rendering).
//!
//! Run: `cargo test --release --test probe_arc251_type_namespace_fix`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_string(world: &wat::freeze::FrozenWorld, call: &str) -> Result<String, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn c01_core_fqdn_scalar_stays_wat_type() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c01a)"), Ok("wat.type/i64".into()));
    assert_eq!(eval_string(&world, "(:user::c01b)"), Ok("wat.type/String".into()));
}

#[test]
fn c02_core_parametric_stays_wat_type() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c02)"),
        Ok("(wat.type/Vector wat.type/i64)".into())
    );
}

#[test]
fn c03_bare_legacy_primitive_renders_core() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c03a)"), Ok("wat.type/i64".into()));
    assert_eq!(eval_string(&world, "(:user::c03b)"), Ok("wat.type/String".into()));
    assert_eq!(eval_string(&world, "(:user::c03c)"), Ok("wat.type/bool".into()));
}

#[test]
fn c04_user_type_preserves_namespace() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_string(&world, "(:user::c04)"),
        Ok("wat.kernel.services.StdErrService/Req".into())
    );
}

#[test]
fn c05_distinct_user_types_do_not_collide() {
    let world = startup_beside(file!()).expect("startup");
    let a = eval_string(&world, "(:user::c05a)");
    let b = eval_string(&world, "(:user::c05b)");
    assert!(a.is_ok() && b.is_ok(), "both must render: {a:?} {b:?}");
    assert_ne!(a, b, "distinct types must NOT render to the same faithful name (collision)");
}

#[test]
fn c06_user_type_two_segment_preserves_namespace() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c06)"), Ok("wat.holon/HolonAST".into()));
}

#[test]
fn c07_type_var_stays_bare() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c07a)"), Ok("T".into()));
    assert_eq!(eval_string(&world, "(:user::c07b)"), Ok("K".into()));
}

#[test]
fn c08_bare_head_parametric_errors_cleanly() {
    let world = startup_beside(file!()).expect("startup");
    let ast_a = wat::parse_one!("(:user::c08a)").expect("parse");
    assert!(
        eval_in_frozen(&ast_a, &world, &Environment::new()).is_err(),
        "bare parametric head must error cleanly, not panic"
    );
    let ast_b = wat::parse_one!("(:user::c08b)").expect("parse");
    assert!(
        eval_in_frozen(&ast_b, &world, &Environment::new()).is_err(),
        "higher-kinded head must error cleanly, not panic"
    );
}

#[test]
fn c09_trailing_colons_path_errors_cleanly() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::c09)").expect("parse");
    assert!(
        eval_in_frozen(&ast, &world, &Environment::new()).is_err(),
        "trailing-`::` path must error cleanly, not panic"
    );
}
