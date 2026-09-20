//! The eight verbs the `the-little-wat` branch adds to the runtime, under test.
//!
//! Both groups were reached for by the-little-wat's x86-64 compiler and found missing, and
//! both were measured before they were asked for:
//!
//!   * **the seven bitwise ops** (F-134) — without them a packet-header parser must divide
//!     and modulo to reach a field, and it ran at **11.63x** a C loop. With them: **2.35x**,
//!     landing exactly on the division-free control, which is what confirms the attribution.
//!   * **`code-point-at`** (F-135) — without it, reading one character means
//!     `(subs s i (+ i 1))`, which ALLOCATES a one-character String and then needs a string
//!     compare to read it: **71 instructions and 56 cycles per byte scanned, 34.83x** a C
//!     `s[i] == 'e'` loop. With it: **3.41x**.
//!
//! Each case below is chosen to fail a plausible WRONG implementation, not merely to pass the
//! right one — a shift probed only at `1 << 10` cannot tell an arithmetic shift from a logical
//! one, nor a masked count from a saturating one.
//!
//! Run: `cargo test --test resolve probe_little_wat_bits_and_code_point`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture; drive it
// via `call_beside_value` and inspect the returned typed Value. Same shape as
// `probe_arc251_io_string_primitives`, with `i64` in place of `String`.
fn eval_i64(fn_name: &str) -> Result<i64, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_bit_and() {
    assert_eq!(eval_i64(":user::c01").expect("bit-and"), 8, "0b1100 & 0b1010");
}

#[test]
fn contract_02_bit_or() {
    assert_eq!(eval_i64(":user::c02").expect("bit-or"), 14, "0b1100 | 0b1010");
}

#[test]
fn contract_03_bit_xor() {
    assert_eq!(eval_i64(":user::c03").expect("bit-xor"), 6, "0b1100 ^ 0b1010");
}

#[test]
fn contract_04_bit_not_of_zero_is_minus_one() {
    assert_eq!(eval_i64(":user::c04").expect("bit-not"), -1);
}

#[test]
fn contract_05_shift_left() {
    assert_eq!(eval_i64(":user::c05").expect("bit-shift-left"), 1024);
}

/// The case that separates an arithmetic shift from a logical one. A logical `-7 >> 1` is
/// 9223372036854775804; clj's `bit-shift-right` sign-extends, so the answer is -4.
#[test]
fn contract_06_shift_right_sign_extends() {
    assert_eq!(eval_i64(":user::c06").expect("bit-shift-right"), -4);
}

/// The mirror case, and the whole reason the unsigned verb has its own name: `-1` is
/// sixty-four set bits, so a ZERO-FILLING shift by 60 leaves four of them. An arithmetic
/// shift would answer -1 and this test would catch it.
#[test]
fn contract_07_unsigned_shift_right_zero_fills() {
    assert_eq!(eval_i64(":user::c07").expect("unsigned-bit-shift-right"), 15);
}

/// **The shift count is masked to six bits**, so 64 means 0 — the behaviour x86 and the JVM
/// share and clj inherits. An implementation that saturated would answer 0 here, and one that
/// panicked in debug would not answer at all.
#[test]
fn contract_08_shift_count_masks_to_six_bits() {
    assert_eq!(eval_i64(":user::c08").expect("bit-shift-left 1 64"), 1);
}

#[test]
fn contract_09_code_point_at_indexes_by_character() {
    assert_eq!(eval_i64(":user::c09").expect("code-point-at"), 101, "'e' in \"hello\"");
}

#[test]
fn contract_10_code_point_at_zero() {
    assert_eq!(eval_i64(":user::c10").expect("code-point-at"), 65, "'A'");
}

/// Out of range is LOUD. A silent 0 would be indistinguishable from a NUL byte, which is
/// exactly the confusion a scanner would inherit.
#[test]
fn contract_11_code_point_at_past_the_end_errors() {
    let err = eval_i64(":user::c11").expect_err("index 99 of a 5-character string must error");
    let text = format!("{err:?}");
    assert!(
        text.contains("index out of range") && text.contains("char-length=5"),
        "the error must name the index and the length it exceeded; got {text}"
    );
}

/// A negative index is the same class and must not wrap into a huge `usize`.
#[test]
fn contract_12_code_point_at_negative_index_errors() {
    let err = eval_i64(":user::c12").expect_err("index -1 must error, not wrap");
    let text = format!("{err:?}");
    assert!(
        text.contains("index out of range"),
        "a negative index must be refused by the same path; got {text}"
    );
}
