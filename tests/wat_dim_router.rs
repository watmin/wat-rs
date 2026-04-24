//! End-to-end tests for `:wat::config::set-dim-router!` — arc 037
//! slice 4.
//!
//! The setter accepts any AST that evaluates to a function value.
//! Signature: `:fn(:wat::holon::HolonAST) -> :Option<i64>` — the
//! router sees the whole AST and decides what dim best suits its
//! surface. Freeze evaluates the AST against the finished frozen
//! world and installs a `WatLambdaRouter`.
//!
//! Entry-file discipline (set-*! before any non-setter) means the
//! setter's AST references symbols that don't exist yet at
//! collect-entry-file time. That's fine — the setter captures the
//! AST verbatim; freeze evaluates it after all defines are
//! registered.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

fn try_startup(src: &str) -> Result<(), StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new())).map(|_| ())
}

/// Emit `n` distinct `(:wat::holon::Atom "i")` calls inside a
/// `(:wat::core::list :wat::holon::HolonAST ...)` literal.
fn atoms_list(n: usize) -> String {
    let mut s = String::from("(:wat::core::list :wat::holon::HolonAST");
    for i in 0..n {
        s.push_str(&format!(" (:wat::holon::Atom \"atom-{}\")", i));
    }
    s.push(')');
    s
}

// ─── User router that introspects AST via statement-length ────────────

#[test]
fn user_router_introspects_via_statement_length() {
    // Router is the honest shape: takes AST, inspects surface arity,
    // decides dim. Always picks d=10000 regardless — but threads the
    // introspection primitive to prove it works.
    //
    // 200 atoms → sqrt(10000) = 100 → overflow at user's tier.
    // Default SizingRouter would have routed 200 → d=100000 and
    // succeeded. The user's router produces different behavior.
    let src = format!(
        r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((ast :wat::holon::HolonAST) -> :Option<i64>)
            (:wat::core::let*
              (((_n :i64) (:wat::holon::statement-length ast)))
              (Some 10000))))

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(200)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::holon::CapacityExceeded");
                match (&sv.fields[0], &sv.fields[1]) {
                    (Value::i64(cost), Value::i64(budget)) => {
                        assert_eq!(*cost, 200, "cost is the constituent count");
                        assert_eq!(*budget, 100, "budget = floor(sqrt(10000)) at user's d=10000");
                    }
                    other => panic!("expected (i64, i64) fields; got {:?}", other),
                }
            }
            other => panic!("expected Err(CapacityExceeded); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── User router that branches on AST surface arity ───────────────────

#[test]
fn user_router_branches_on_surface_arity() {
    // Router returns :None for Bundles with arity > 4, else Some(256).
    // Bundling 5 atoms triggers the overflow path via router's :None.
    let src = format!(
        r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((ast :wat::holon::HolonAST) -> :Option<i64>)
            (:wat::core::if (:wat::core::> (:wat::holon::statement-length ast) 4)
              -> :Option<i64>
              :None
              (Some 256))))

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(5)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Err(Value::Struct(sv)) => {
                assert_eq!(sv.type_name, ":wat::holon::CapacityExceeded");
                match (&sv.fields[0], &sv.fields[1]) {
                    (Value::i64(cost), Value::i64(budget)) => {
                        assert_eq!(*cost, 5);
                        assert_eq!(*budget, 0, "budget=0 signals router returned None");
                    }
                    other => panic!("expected (i64, i64); got {:?}", other),
                }
            }
            other => panic!("expected Err(CapacityExceeded); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── User router that succeeds ────────────────────────────────────────

#[test]
fn user_router_succeeds_when_within_picked_tier() {
    // Constant user router at d=10000. 50 atoms fits (sqrt=100).
    let src = format!(
        r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((_ast :wat::holon::HolonAST) -> :Option<i64>)
            (Some 10000)))

        (:wat::core::define (:user::main -> :wat::holon::BundleResult)
          (:wat::holon::Bundle {}))
        "#,
        atoms_list(50)
    );
    match run(&src) {
        Value::Result(r) => match &*r {
            Ok(Value::holon__HolonAST(_)) => {}
            other => panic!("expected Ok(wat::holon::HolonAST); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

// ─── set-dim-router! failure modes ────────────────────────────────────

#[test]
fn set_dim_router_with_non_function_fails_startup() {
    // AST evaluates to i64, not a function. Freeze rejects.
    let src = r#"
        (:wat::config::set-dim-router! 42)

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match try_startup(src) {
        Err(StartupError::DimRouter(msg)) => {
            assert!(
                msg.contains("function value"),
                "message should mention function value; got {}",
                msg
            );
        }
        other => panic!("expected StartupError::DimRouter, got {:?}", other),
    }
}

#[test]
fn set_dim_router_with_wrong_arity_fails_startup() {
    // 2-param lambda: freeze rejects at arity check.
    let src = r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((a :wat::holon::HolonAST)
                               (b :wat::holon::HolonAST)
                               -> :Option<i64>)
            (Some 256)))

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match try_startup(src) {
        Err(StartupError::DimRouter(msg)) => {
            assert!(
                msg.contains("exactly 1 argument"),
                "message should mention arity; got {}",
                msg
            );
        }
        other => panic!("expected StartupError::DimRouter, got {:?}", other),
    }
}

#[test]
fn set_dim_router_with_wrong_param_type_fails_startup() {
    // Param declared :i64 instead of :wat::holon::HolonAST. Freeze
    // rejects at signature type check.
    let src = r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((n :i64) -> :Option<i64>)
            (Some 256)))

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match try_startup(src) {
        Err(StartupError::DimRouter(msg)) => {
            assert!(
                msg.contains(":wat::holon::HolonAST"),
                "message should mention HolonAST param; got {}",
                msg
            );
        }
        other => panic!("expected StartupError::DimRouter, got {:?}", other),
    }
}

#[test]
fn set_dim_router_with_wrong_return_type_fails_startup() {
    // Return declared :i64 instead of :Option<i64>. Freeze rejects.
    let src = r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((ast :wat::holon::HolonAST) -> :i64)
            256))

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match try_startup(src) {
        Err(StartupError::DimRouter(msg)) => {
            assert!(
                msg.contains(":Option<i64>"),
                "message should mention Option<i64> return; got {}",
                msg
            );
        }
        other => panic!("expected StartupError::DimRouter, got {:?}", other),
    }
}

#[test]
fn duplicate_set_dim_router_rejected() {
    // Config setter discipline: one set-dim-router! per entry file.
    let src = r#"
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((a :wat::holon::HolonAST) -> :Option<i64>)
            (Some 256)))
        (:wat::config::set-dim-router!
          (:wat::core::lambda ((a :wat::holon::HolonAST) -> :Option<i64>)
            (Some 4096)))

        (:wat::core::define (:user::main -> :()) ())
    "#;
    match try_startup(src) {
        Err(StartupError::Config(_)) => {}
        other => panic!("expected StartupError::Config(DuplicateField); got {:?}", other),
    }
}
