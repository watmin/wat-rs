//! Arc 053 slice 2 — OnlineSubspace as native wat value.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

#[test]
fn subspace_construct_dim_k_n_zero() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((s :wat::holon::OnlineSubspace) (:wat::holon::OnlineSubspace/new 10000 16))
             ((d :wat::core::i64) (:wat::holon::OnlineSubspace/dim s))
             ((k :wat::core::i64) (:wat::holon::OnlineSubspace/k s))
             ((n :wat::core::i64) (:wat::holon::OnlineSubspace/n s)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if
                (:wat::core::and (:wat::core::= d 10000)
                  (:wat::core::and (:wat::core::= k 16) (:wat::core::= n 0))) -> :wat::core::String
                "ok" "wrong"))))
    "##;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

#[test]
fn subspace_update_increments_n_and_returns_residual() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((s :wat::holon::OnlineSubspace) (:wat::holon::OnlineSubspace/new 10000 4))
             ((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x")))
             ((residual :wat::core::f64) (:wat::holon::OnlineSubspace/update s v))
             ((n :wat::core::i64) (:wat::holon::OnlineSubspace/n s)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= n 1) -> :wat::core::String "incremented" "stuck"))))
    "##;
    assert_eq!(run(src), vec!["incremented".to_string()]);
}

#[test]
fn subspace_eigenvalues_returns_k_floats() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((s :wat::holon::OnlineSubspace) (:wat::holon::OnlineSubspace/new 10000 8))
             ((eigs :wat::core::Vector<wat::core::f64>) (:wat::holon::OnlineSubspace/eigenvalues s))
             ((len :wat::core::i64) (:wat::core::length eigs)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::if (:wat::core::= len 8) -> :wat::core::String "k-eigs" "wrong-len"))))
    "##;
    assert_eq!(run(src), vec!["k-eigs".to_string()]);
}
