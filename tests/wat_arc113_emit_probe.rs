//! Direct probe — does a forked child panicking with assertion-failed!
//! emit `#wat.kernel/ProcessPanics ...` on stderr? Bypasses hermetic.wat /
//! sandbox.wat to read raw stderr from the fork pipe directly.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

#[test]
fn child_assertion_writes_died_chain_to_stderr() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :Vec<wat::core::String>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::i64,wat::core::i64>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :wat::core::unit)
                    (:wat::test::assert-eq 1 2)))))
             ((joined :wat::core::Result<wat::core::unit,Vec<wat::kernel::ProcessDiedError>>)
              (:wat::kernel::Process/join-result proc))
             ((stderr-r :wat::io::IOReader) (:wat::kernel::Process/stderr proc))
             ((lines :Vec<wat::core::String>) (:wat::kernel::drain-lines stderr-r)))
            lines))
    "##;
    let v = run(src);
    let lines: Vec<String> = match v {
        Value::Vec(items) => items.iter().map(|x| match x {
            Value::String(s) => (**s).clone(),
            other => panic!("expected String got {:?}", other),
        }).collect(),
        other => panic!("expected Vec got {:?}", other),
    };
    eprintln!("STDERR_LINES: {:#?}", lines);
    assert!(
        lines.iter().any(|l| l.starts_with("#wat.kernel/ProcessPanics")),
        "expected a #wat.kernel/ProcessPanics marker line; got: {:?}",
        lines
    );
}

#[test]
fn child_plain_exit_writes_panic_marker_to_stderr() {
    // Sanity probe — fork + immediate runtime error. Tests whether
    // ANY stderr makes it back from the child. If this lands empty,
    // the issue isn't in slice 3's emit code; it's in stderr drain
    // ordering (the child writes ARE happening, just not where
    // we're reading).
    let src = r##"
        (:wat::core::define
          (:user::main -> :Vec<wat::core::String>)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::i64,wat::core::i64>)
              (:wat::kernel::fork-program-ast
                (:wat::test::program
                  (:wat::core::define (:user::main
                                       (stdin  :wat::io::IOReader)
                                       (stdout :wat::io::IOWriter)
                                       (stderr :wat::io::IOWriter)
                                       -> :wat::core::unit)
                    (:wat::io::IOWriter/println stderr "from-child-stderr")))))
             ((joined :wat::core::Result<wat::core::unit,Vec<wat::kernel::ProcessDiedError>>)
              (:wat::kernel::Process/join-result proc))
             ((stderr-r :wat::io::IOReader) (:wat::kernel::Process/stderr proc))
             ((lines :Vec<wat::core::String>) (:wat::kernel::drain-lines stderr-r)))
            lines))
    "##;
    let v = run(src);
    let lines: Vec<String> = match v {
        Value::Vec(items) => items.iter().map(|x| match x {
            Value::String(s) => (**s).clone(),
            other => panic!("expected String got {:?}", other),
        }).collect(),
        other => panic!("expected Vec got {:?}", other),
    };
    eprintln!("PLAIN_STDERR_LINES: {:#?}", lines);
    assert_eq!(
        lines,
        vec!["from-child-stderr".to_string()],
        "explicit child-stderr println should come back"
    );
}
