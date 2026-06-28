//! `first`/`second`/`third` become BARE, raising (forced forward from the 251 note; a Break Stuff HARD CUT).
//! Today they return `Option<T>` on runtime-length sequences (arc-047). This flips them to bare `T`, raising on
//! empty/out-of-range — like `nth` — with `get` as the lone `Option` safe path. RED at HEAD: using `(first xs)`
//! BARE (as `T`, no `Option/expect`) is a type error while `first` returns `Option`. GREEN when the flip lands.
//! Tuple-`first` is already bare (regression guard). Contract: DESIGN-STONE-first-bare-accessors.md.
//!
//! Run: cargo test --release -p wat --test probe_first_bare_accessors

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_file(path: &str) -> Result<Value, String> {
    let w = startup_from_file(path).map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!("(:p::f)").map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &w, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|tv| tv.value_owned())
}

/// BARE usage: the accessor's result is returned directly as `T` (no `Option/expect`). RED at HEAD.
fn expect_bare_i64(path: &str, want: i64) {
    match run_file(path) {
        Ok(Value::i64(n)) => assert_eq!(n, want, "value: got {n} want {want}"),
        Ok(other) => panic!("expected bare i64({want}); got {other:?}"),
        Err(e) => panic!("`first` must return BARE T (usable without Option/expect): {e}"),
    }
}

#[test]
fn first_vector_bare() {
    expect_bare_i64("tests/types/probe_first_bare_accessors_first_vector.wat", 10);
}

#[test]
fn first_persistent_vector_bare() {
    expect_bare_i64("tests/types/probe_first_bare_accessors_first_persistent_vector.wat", 10);
}

#[test]
fn first_list_bare() {
    expect_bare_i64("tests/types/probe_first_bare_accessors_first_list.wat", 10);
}

#[test]
fn third_vector_bare() {
    expect_bare_i64("tests/types/probe_first_bare_accessors_third_vector.wat", 30);
}

/// Regression: Tuple-`first` was always bare-total — must stay bare (green at HEAD and after).
#[test]
fn first_tuple_still_bare() {
    expect_bare_i64("tests/types/probe_first_bare_accessors_first_tuple.wat", 10);
}

/// Semantic guard: `first` on an EMPTY sequence RAISES (no value to return). After the flip this is a runtime
/// raise; at HEAD it's a type error — either way an Err, so this asserts the post-flip contract.
#[test]
fn first_empty_raises() {
    let r = run_file("tests/types/probe_first_bare_accessors_first_empty.wat");
    assert!(r.is_err(), "first on empty must NOT yield a value (raise); got {r:?}");
}
