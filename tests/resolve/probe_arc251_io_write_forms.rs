//! Strike (examinare probe) — the wat-level IO conveniences: `write-file` + `with-open-file`.
//!
//! Run: `cargo test --release --test probe_arc251_io_write_forms`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn contract_01_write_file() {
    let p = "/tmp/wat-iowriteforms-c01.txt";
    let _ = std::fs::remove_file(p);
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-c01)").expect("parse");
    let r = eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned());
    assert!(r.is_ok(), "write-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-write-file");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_02_with_open_file() {
    let p = "/tmp/wat-iowriteforms-c02.txt";
    let _ = std::fs::remove_file(p);
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-c02)").expect("parse");
    let r = eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned());
    assert!(r.is_ok(), "with-open-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-with-open");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_03_with_open_file_returns_body_result() {
    let p = "/tmp/wat-iowriteforms-c03.txt";
    let _ = std::fs::remove_file(p);
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-c03)").expect("parse");
    let r = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("{e:?}"));
    assert_eq!(r, Ok(Value::i64(99)));
    let _ = std::fs::remove_file(p);
}
