//! FM 2-bis probe — arc 237 Stone 237.1: :wat::core::typeunion substrate mint.
//!
//! Verifies the load-bearing API contract: NEW `TypeDef::Union` variant +
//! `UnionDef` struct + registration + cycle detection + member type-checking +
//! bounded-existential unification extension. Per arc 237 polymorphism
//! consolidation: typeunion is the type-level grouping primitive that
//! defclause (Stone 237.2) consumes for variadic-mixed-type dispatch.
//!
//! Doctrine (per docs/arc/2026/05/237-polymorphism-consolidation/):
//!   - Closed (not open-extension); fractal composition is the extension story
//!   - Bounded (explicit members; finite); preserves `:Any` ban
//!   - Type-only (no runtime artifact); no new Value variant
//!   - Departs from substrate's existing "named enum for closed heterogeneous
//!     sets" AnyBanned recommendation — justified by arithmetic UX (typeunion
//!     dispatches by actual type without wrapping; enum requires wrap)
//!
//! Probe contracts (14):
//!   1.  TypeDef::Union variant + UnionDef struct exist; register + read-back
//!   2.  Cycle detection at registration → CyclicUnion error
//!   3.  Empty members rejected → EmptyUnion error
//!   4.  Single-member rejected → SingleMemberUnion error (recommends typealias)
//!   5.  Fn member rejected → InvalidUnionMember
//!   6.  Var member rejected → InvalidUnionMember
//!   7.  Path member (concrete type) accepted
//!   8.  Parametric member accepted
//!   9.  Tuple member accepted
//!   10. Recursive union (typeunion-of-typeunions) accepted when acyclic
//!   11. wat source: typeunion declaration parses + registers
//!   12. wat source: typeunion-typed arg accepts member-typed value (unify Union vs Concrete)
//!   13. wat source: typeunion-typed arg rejects non-member value (unify fail)
//!   14. wat source: fractal — :Baz [:Foo :bool] where :Foo [:i64 :f64] accepts :bool, :i64, :f64
//!
//! Initial state: file does not compile — UnionDef/TypeError::CyclicUnion/etc. don't exist.
//! Post-stone 237.1: 14/14 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites
//! this file verbatim as "the working contract sonnet must satisfy."

use wat::freeze::startup_from_file;
use wat::types::{TypeDef, TypeEnv, TypeError, TypeErrorKind, TypeExpr, UnionDef};

// ─── helpers ────────────────────────────────────────────────────────────────

fn path(p: &str) -> TypeExpr {
    TypeExpr::Path(p.to_string())
}

fn fresh_env() -> TypeEnv {
    TypeEnv::new()
}

fn register_union(env: &mut TypeEnv, name: &str, members: Vec<TypeExpr>) -> Result<(), TypeError> {
    env.register(TypeDef::Union(UnionDef {
        name: name.to_string(),
        type_params: vec![],
        members,
    }))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_01_union_def_registers_and_reads_back() {
    let mut env = fresh_env();
    let result = register_union(
        &mut env,
        ":my::Numeric",
        vec![path(":wat::core::i64"), path(":wat::core::f64")],
    );
    assert!(result.is_ok(), "register typeunion :my::Numeric should succeed");

    let def = env.get(":my::Numeric").expect("typeunion :my::Numeric should be registered");
    match def {
        TypeDef::Union(u) => {
            assert_eq!(u.name, ":my::Numeric");
            assert_eq!(u.members.len(), 2);
            assert!(u.type_params.is_empty(), "arc 237 ships non-parametric only");
        }
        other => panic!("expected TypeDef::Union, got {:?}", other),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_cyclic_union_rejected_at_registration() {
    let mut env = fresh_env();
    // Register :A referring to :B (forward reference allowed at this point)
    register_union(
        &mut env,
        ":my::A",
        vec![path(":wat::core::i64"), path(":my::B")],
    )
    .expect("forward reference to unregistered :B is allowed pre-cycle");

    // Registering :B that closes the cycle (:B references :A) should fail
    let result = register_union(
        &mut env,
        ":my::B",
        vec![path(":wat::core::f64"), path(":my::A")],
    );
    match result.expect_err("cyclic typeunion should be rejected").kind() {
        TypeErrorKind::CyclicUnion { name } => {
            assert_eq!(name, ":my::B", "CyclicUnion should name the cycle-closing union");
        }
        other => panic!("expected TypeErrorKind::CyclicUnion, got {:?}", other),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_empty_union_rejected() {
    let mut env = fresh_env();
    let result = register_union(&mut env, ":my::Empty", vec![]);
    match result.expect_err("empty typeunion should be rejected").kind() {
        TypeErrorKind::EmptyUnion { name } => {
            assert_eq!(name, ":my::Empty");
        }
        other => panic!("expected TypeErrorKind::EmptyUnion, got {:?}", other),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_single_member_union_rejected_with_typealias_hint() {
    let mut env = fresh_env();
    let result = register_union(&mut env, ":my::Foo", vec![path(":wat::core::i64")]);
    match result.expect_err("single-member typeunion should be rejected").kind() {
        TypeErrorKind::SingleMemberUnion { name } => {
            assert_eq!(name, ":my::Foo");
            // Diagnostic message should recommend typealias (verified by SCORE
            // independently rendering the error and reading the recommendation)
        }
        other => panic!("expected TypeErrorKind::SingleMemberUnion, got {:?}", other),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_fn_member_rejected() {
    let mut env = fresh_env();
    let fn_type = TypeExpr::Fn {
        args: vec![path(":wat::core::i64")],
        ret: Box::new(path(":wat::core::bool")),
    };
    let result = register_union(
        &mut env,
        ":my::WithFn",
        vec![path(":wat::core::i64"), fn_type],
    );
    match result.expect_err("typeunion with Fn member should be rejected").kind() {
        TypeErrorKind::InvalidUnionMember { union_name, .. } => {
            assert_eq!(union_name, ":my::WithFn");
        }
        other => panic!("expected TypeErrorKind::InvalidUnionMember, got {:?}", other),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_var_member_rejected() {
    let mut env = fresh_env();
    let var = TypeExpr::Var(42);
    let result = register_union(
        &mut env,
        ":my::WithVar",
        vec![path(":wat::core::i64"), var],
    );
    match result.expect_err("typeunion with Var member should be rejected").kind() {
        TypeErrorKind::InvalidUnionMember { union_name, .. } => {
            assert_eq!(union_name, ":my::WithVar");
        }
        other => panic!("expected TypeErrorKind::InvalidUnionMember for Var, got {:?}", other),
    }
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_path_members_accepted() {
    let mut env = fresh_env();
    let result = register_union(
        &mut env,
        ":my::Primitives",
        vec![
            path(":wat::core::i64"),
            path(":wat::core::f64"),
            path(":wat::core::String"),
        ],
    );
    assert!(result.is_ok(), "typeunion with Path members should succeed");

    let def = env.get(":my::Primitives").expect("registered");
    if let TypeDef::Union(u) = def {
        assert_eq!(u.members.len(), 3);
    } else {
        panic!("expected TypeDef::Union");
    }
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_parametric_member_accepted() {
    let mut env = fresh_env();
    let vector_i64 = TypeExpr::Parametric {
        head: "wat::core::Vector".to_string(),
        args: vec![path(":wat::core::i64")],
    };
    let vector_f64 = TypeExpr::Parametric {
        head: "wat::core::Vector".to_string(),
        args: vec![path(":wat::core::f64")],
    };
    let result = register_union(&mut env, ":my::NumericVecs", vec![vector_i64, vector_f64]);
    assert!(result.is_ok(), "typeunion with Parametric members should succeed");
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_tuple_member_accepted() {
    let mut env = fresh_env();
    let tuple_ii = TypeExpr::Tuple(vec![path(":wat::core::i64"), path(":wat::core::i64")]);
    let tuple_ff = TypeExpr::Tuple(vec![path(":wat::core::f64"), path(":wat::core::f64")]);
    let result = register_union(&mut env, ":my::Pairs", vec![tuple_ii, tuple_ff]);
    assert!(result.is_ok(), "typeunion with Tuple members should succeed");
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_recursive_union_accepted_when_acyclic() {
    let mut env = fresh_env();
    // :Foo = {i64, f64}
    register_union(
        &mut env,
        ":my::Foo",
        vec![path(":wat::core::i64"), path(":wat::core::f64")],
    )
    .expect(":my::Foo registers");

    // :Baz = {:Foo, bool} — :Foo is itself a typeunion (acyclic recursion)
    let result = register_union(
        &mut env,
        ":my::Baz",
        vec![path(":my::Foo"), path(":wat::core::bool")],
    );
    assert!(
        result.is_ok(),
        "typeunion-of-typeunions should succeed when acyclic"
    );

    // Verify :Baz's members include :Foo (resolution-at-use-site, not flattened at registration)
    if let TypeDef::Union(u) = env.get(":my::Baz").unwrap() {
        assert_eq!(u.members.len(), 2);
        let has_foo_ref = u
            .members
            .iter()
            .any(|m| matches!(m, TypeExpr::Path(p) if p == ":my::Foo"));
        assert!(has_foo_ref, ":Baz should reference :Foo as a Path");
    } else {
        panic!("expected TypeDef::Union for :my::Baz");
    }
}

// ─── Probes 11-14: wat-source integration via startup_from_file ─────────────
//
// These probes exercise the FULL path: parse wat source → register typeunion →
// type-check defn with typeunion-typed args → unify Union vs Concrete in
// reduce/unify arms. Verifies the bounded-existential unification extension
// fires end-to-end. startup_from_file returns Err on type-check failure;
// each probe asserts on success / expected failure.

// ─── Probe 11 ───────────────────────────────────────────────────────────────
#[test]
fn probe_11_wat_source_typeunion_declaration_parses_and_registers() {
    startup_from_file("tests/types/probe_arc237_stone1_typeunion_substrate_probe11.wat")
        .expect("typeunion declaration should parse + register cleanly");
}

// ─── Probe 12 ───────────────────────────────────────────────────────────────
#[test]
fn probe_12_typeunion_arg_accepts_member_value() {
    startup_from_file("tests/types/probe_arc237_stone1_typeunion_substrate_probe12.wat")
        .expect("typeunion arg should accept i64 (member) and f64 (member) — bounded existential unify");
}

// ─── Probe 13 ───────────────────────────────────────────────────────────────
#[test]
fn probe_13_typeunion_arg_rejects_non_member_value() {
    let result = startup_from_file(
        "tests/types/probe_arc237_stone1_typeunion_substrate_probe13.wat.bad",
    );
    wat::assert_startup_error!(result, check
        wat::check::error::CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":my::identity"
            && param == "#1"
            && expected == ":my::IorF"
            && got == ":wat::core::String"
    );
}

// ─── Probe 14 ───────────────────────────────────────────────────────────────
#[test]
fn probe_14_fractal_typeunion_resolves_transitively() {
    // :Foo = {i64, f64}
    // :Baz = {Foo, bool}  → transitively {i64, f64, bool}
    // identity :Baz should accept i64, f64, bool
    startup_from_file("tests/types/probe_arc237_stone1_typeunion_substrate_probe14.wat")
        .expect("fractal typeunion should accept all transitively-resolved members (i64, f64, bool)");
}
