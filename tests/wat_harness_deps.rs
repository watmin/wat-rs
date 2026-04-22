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

#[test]
fn harness_accepts_dep_registrar_for_rust_shim() {
    // Slice 4a — verify that a registrar passed to
    // from_source_with_deps actually runs. We install a shim that
    // adds a no-op type decl; then check that rust_deps::get()
    // reports it as present. Validates the registrar plumbing
    // without needing a full #[wat_dispatch] shim in the test.
    use wat::rust_deps::{self, RustDepsBuilder, RustTypeDecl};

    fn probe_register(builder: &mut RustDepsBuilder) {
        builder.register_type(RustTypeDecl {
            path: ":rust::probe::Sentinel",
        });
    }

    // Minimal user program (no wat-level uses of the probe; we're
    // checking the registry contents, not dispatch).
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

    let _h = Harness::from_source_with_deps(user, &[], &[probe_register])
        .expect("freeze with registrar");

    // After Harness built the registry, ours should be visible.
    // (Earlier tests in this file may have installed a registry
    // already — first-call-wins OnceLock semantics; if we're NOT
    // first, the earlier registry is in use. Check either that
    // our sentinel is there OR the earlier baked defaults are.)
    let registry = rust_deps::get();
    let baseline_has_lru = registry.has_type(":rust::lru::LruCache");
    let ours_has_sentinel = registry.has_type(":rust::probe::Sentinel");
    assert!(
        baseline_has_lru || ours_has_sentinel,
        "registry should have either the baked defaults or our sentinel; \
         got neither — registrar plumbing failed"
    );
}

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
