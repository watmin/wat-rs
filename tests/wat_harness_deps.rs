//! Integration: `wat::Harness::from_source_with_deps` — arc 013
//! slice 2.
//!
//! Exercises the composition-with-external-sources path. A throwaway
//! `&[StdlibFile]` with one simple define stands in for what an
//! external wat crate (like wat-lru, slice 4) will ship via its
//! `pub fn stdlib_sources()`. The user source calls into that define
//! from `:user::main`; the Outcome captures the resulting output.
//!
//! This is the substrate test — slice 4 will ship the first real
//! external crate consumer (wat-lru), and slice 5 will prove the full
//! user-shape through an examples/ binary.

use wat::harness::{Harness, Outcome};
use wat::stdlib::StdlibFile;

/// One "dep" source — a single in-memory wat file declaring one
/// define under `:user::*` (the namespace tier external crates live
/// under per arc 013's convention).
const DEP_GREETING: &[StdlibFile] = &[StdlibFile {
    path: "test-harness-deps/greeting.wat",
    source: r#"
        (:wat::core::define
          (:user::test::dep::greeting -> :String)
          "hello from dep")
    "#,
}];

/// User source that calls the dep-provided define from :user::main.
const USER_SRC: &str = r#"
    (:wat::config::set-dims! 1024)
    (:wat::config::set-capacity-mode! :error)

    (:wat::core::define (:user::main
                         (stdin  :wat::io::IOReader)
                         (stdout :wat::io::IOWriter)
                         (stderr :wat::io::IOWriter)
                         -> :())
      (:wat::io::IOWriter/println stdout (:user::test::dep::greeting)))
"#;

#[test]
fn harness_composes_user_source_with_one_dep() {
    let h = Harness::from_source_with_deps(USER_SRC, &[DEP_GREETING], &[]).expect("freeze");
    let Outcome { stdout, stderr } = h.run(&[]).expect("run");
    assert_eq!(stdout, vec!["hello from dep".to_string()]);
    assert!(stderr.is_empty(), "expected empty stderr; got {:?}", stderr);
}

#[test]
fn harness_with_zero_deps_matches_from_source() {
    // Passing &[] should behave identically to the no-deps entry.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::io::IOWriter/println stdout "no deps"))
    "#;
    let h_no_deps = Harness::from_source_with_deps(src, &[], &[]).expect("freeze-empty-deps");
    let h_ref = Harness::from_source(src).expect("freeze-from-source");
    let out_a = h_no_deps.run(&[]).expect("run-no-deps");
    let out_b = h_ref.run(&[]).expect("run-from-source");
    assert_eq!(out_a, out_b);
    assert_eq!(out_a.stdout, vec!["no deps".to_string()]);
}

#[test]
fn harness_composes_multiple_deps() {
    // Two deps contributing separate defines; user calls both.
    const DEP_A: &[StdlibFile] = &[StdlibFile {
        path: "test-harness-deps/a.wat",
        source: r#"
            (:wat::core::define
              (:user::test::dep-a::label -> :String)
              "A")
        "#,
    }];
    const DEP_B: &[StdlibFile] = &[StdlibFile {
        path: "test-harness-deps/b.wat",
        source: r#"
            (:wat::core::define
              (:user::test::dep-b::label -> :String)
              "B")
        "#,
    }];
    let user = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((_ :i64) (:wat::io::IOWriter/writeln stdout (:user::test::dep-a::label)))
             ((_ :i64) (:wat::io::IOWriter/writeln stdout (:user::test::dep-b::label))))
            ()))
    "#;
    let h = Harness::from_source_with_deps(user, &[DEP_A, DEP_B], &[]).expect("freeze");
    let out = h.run(&[]).expect("run");
    assert_eq!(out.stdout, vec!["A".to_string(), "B".to_string()]);
}

// The slice-4a probe test (`harness_accepts_dep_registrar_for_rust_shim`)
// retired in slice 4b. It tried to assert registrar effects through
// `rust_deps::get()`'s global OnceLock, which is first-call-wins and
// order-fragile across tests. Once slice 4b emptied
// `with_wat_rs_defaults()`, the probe's "either the baked default or
// our sentinel" fallback lost half its ground and the test became
// order-dependent. The honest proof — a registrar's symbols reaching
// real wat code through the full Harness path — lives in
// crates/wat-lru/tests/, where one dedicated process builds one
// registry with the full superset (exactly the pattern BACKLOG's
// sub-fog 4a-install prescribes for tests with specific dep sets).

#[test]
fn harness_dep_declaring_under_wat_namespace_is_rejected() {
    // Reserved-prefix gate must reject dep attempts to claim
    // `:wat::*`. Arc 013's namespace discipline: deps live under
    // `:user::*`; the runtime owns `:wat::*`.
    const BAD_DEP: &[StdlibFile] = &[StdlibFile {
        path: "test-harness-deps/bad.wat",
        source: r#"
            (:wat::core::define
              (:wat::std::bad::thing -> :String)
              "should not register")
        "#,
    }];
    let user = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          ())
    "#;
    let result = Harness::from_source_with_deps(user, &[BAD_DEP], &[]);
    assert!(
        result.is_err(),
        "dep declaring under :wat::* must fail reserved-prefix gate"
    );
}
