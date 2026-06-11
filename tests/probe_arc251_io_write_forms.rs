//! Strike (examinare probe) — the wat-level IO conveniences: `write-file` + `with-open-file`.
//! Both are wat HOFs (blessed wat/io.wat) over the live Rust IOWriter primitives. The `with-`
//! naming law: managed scope = framework owns open+close, caller owns usage.
//!
//! NOTE: `:wat::io::read-file` routes through the module LOADER (the test's InMemoryLoader
//! can't see the real fs), so we verify the written bytes with Rust `std::fs` directly.
//!
//! C01 write-file        : (write-file p "x") writes "x" to the real file
//! C02 with-open-file     : writes via the body-fn callback; the file holds the bytes
//! C03 with-open-file ret : the form returns body-fn's result (generic <T>), not the writer
//!
//! Run: `cargo test --release --test probe_arc251_io_write_forms`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run(body: &str, ret_ty: &str) -> Result<Value, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> {ret_ty} {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))
}

#[test]
fn contract_01_write_file() {
    let p = "/tmp/wat-iowriteforms-c01.txt";
    let _ = std::fs::remove_file(p);
    let r = run(&format!("(:wat::io::write-file \"{p}\" \"hello-write-file\")"), ":wat::core::nil");
    assert!(r.is_ok(), "write-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-write-file");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_02_with_open_file() {
    let p = "/tmp/wat-iowriteforms-c02.txt";
    let _ = std::fs::remove_file(p);
    let r = run(
        &format!(
            "(:wat::io::with-open-file \"{p}\" \
               (:wat::core::fn [w <- :wat::io::IOWriter] -> :wat::core::i64 \
                  (:wat::io::IOWriter/write-string w \"hello-with-open\")))"
        ),
        ":wat::core::i64",
    );
    assert!(r.is_ok(), "with-open-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-with-open");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_03_with_open_file_returns_body_result() {
    // Generic <T>: the form returns body-fn's value (99 : i64), not the writer or close result.
    let p = "/tmp/wat-iowriteforms-c03.txt";
    let _ = std::fs::remove_file(p);
    let r = run(
        &format!(
            "(:wat::io::with-open-file \"{p}\" \
               (:wat::core::fn [w <- :wat::io::IOWriter] -> :wat::core::i64 \
                  (:wat::core::do (:wat::io::IOWriter/write-string w \"x\") 99)))"
        ),
        ":wat::core::i64",
    );
    assert_eq!(r, Ok(Value::i64(99)));
    let _ = std::fs::remove_file(p);
}
