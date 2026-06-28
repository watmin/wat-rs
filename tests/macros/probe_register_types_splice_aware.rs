//! Arc 170 slice 3 Gap J — regression probes for `register_types` splice-awareness.
//!
//! `register_types` previously only processed type declarations at the TOP LEVEL
//! of the form list. When a top-level `(:wat::core::do ...)` or
//! `(:wat::core::let ...)` form contained type declarations in its body, those
//! declarations were ABSENT from `TypeEnv` after startup.
//!
//! Gap J extends `register_types` and `register_stdlib_types` to recurse into
//! top-level do/let body forms and register any type declarations found there.
//!
//! All probes FAIL before Gap J ships; all PASS after.

use wat::freeze::startup_from_file;

#[test]
fn do_typealias_registers_in_type_env() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_do_typealias.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::MyAlias").is_some(), ":diag::MyAlias must be registered in TypeEnv after Gap J");
    assert!(world.symbols().get(":diag::alias-probe").is_some(), ":diag::alias-probe must be registered");
}

#[test]
fn do_struct_registers_in_type_env() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_do_struct.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::Point").is_some(), ":diag::Point must be registered in TypeEnv after Gap J");
    assert!(world.symbols().get(":diag::Point/new").is_some(), ":diag::Point/new accessor stub must be present");
    assert!(world.symbols().get(":diag::origin").is_some(), ":diag::origin must be registered");
}

#[test]
fn do_newtype_registers_in_type_env() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_do_newtype.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::UserId").is_some(), ":diag::UserId must be registered in TypeEnv after Gap J");
}

#[test]
fn do_enum_registers_in_type_env() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_do_enum.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::Color").is_some(), ":diag::Color must be registered in TypeEnv after Gap J");
}

#[test]
fn let_body_typealias_registers() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_let_typealias.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::LetAlias").is_some(), ":diag::LetAlias must be registered from let-body after Gap J");
}

#[test]
fn nested_do_typealias_registers() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_nested_do.wat")
        .expect("startup failed");
    assert!(world.types().get(":diag::NestedAlias").is_some(), ":diag::NestedAlias must be registered from do-within-do after Gap J");
}

#[test]
fn do_typealias_usage_typechecks() {
    let world = startup_from_file("tests/macros/probe_register_types_splice_aware_typealias_usage.wat")
        .expect("do_typealias_usage_typechecks: startup should succeed after Gap J");
    assert!(world.symbols().get(":diag::make-score").is_some(), ":diag::make-score must be registered");
    assert!(world.types().get(":diag::Score").is_some(), ":diag::Score must be in TypeEnv");
}
