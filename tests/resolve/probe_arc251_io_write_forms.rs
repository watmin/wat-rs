//! Strike (examinare probe) — the wat-level IO conveniences: `write-file` + `with-open-file`.
//!
//! Run: `cargo test --release --test probe_arc251_io_write_forms`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::compute-cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside`. The claim under test is the file bytes written, inspected
// in-process via std::fs — no process boundary.

#[test]
fn contract_01_write_file() {
    let p = "/tmp/wat-iowriteforms-c01.txt";
    let _ = std::fs::remove_file(p);
    let r = call_beside(file!(), ":user::compute-c01");
    assert!(r.is_ok(), "write-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-write-file");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_02_with_open_file() {
    let p = "/tmp/wat-iowriteforms-c02.txt";
    let _ = std::fs::remove_file(p);
    let r = call_beside(file!(), ":user::compute-c02");
    assert!(r.is_ok(), "with-open-file eval: {r:?}");
    assert_eq!(std::fs::read_to_string(p).unwrap(), "hello-with-open");
    let _ = std::fs::remove_file(p);
}

#[test]
fn contract_03_with_open_file_returns_body_result() {
    let p = "/tmp/wat-iowriteforms-c03.txt";
    let _ = std::fs::remove_file(p);
    let r = call_beside(file!(), ":user::compute-c03").map_err(|e| format!("{e:?}"));
    assert_eq!(r, Ok(Value::i64(99)));
    let _ = std::fs::remove_file(p);
}
