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

/// For ASCII a byte index and a character index are the same number — which is the property a
/// consumer restricted to ASCII relies on.
#[test]
fn contract_13_byte_at_matches_code_point_for_ascii() {
    assert_eq!(eval_i64(":user::c13").expect("byte-at"), 101, "'e'");
    assert_eq!(eval_i64(":user::c13").unwrap(), eval_i64(":user::c09").unwrap());
}

#[test]
fn contract_14_byte_length_of_ascii_is_its_length() {
    assert_eq!(eval_i64(":user::c14").expect("byte-length"), 5);
}

/// **And where they diverge, they diverge deliberately.** "aé" is two characters in three bytes.
/// A verb that silently conflated the two would pass every ASCII test and be wrong here.
#[test]
fn contract_15_byte_length_counts_bytes_not_characters() {
    assert_eq!(eval_i64(":user::c15").expect("byte-length"), 3, "\"aé\" is 3 bytes");
    assert_eq!(eval_i64(":user::c16").expect("length"), 2, "\"aé\" is 2 characters");
}

/// Byte 1 of "aé" is the UTF-8 lead byte 0xC3, not a character. byte-at reports the byte.
#[test]
fn contract_16_byte_at_reports_a_utf8_lead_byte() {
    assert_eq!(eval_i64(":user::c17").expect("byte-at"), 0xC3);
}

#[test]
fn contract_17_byte_at_past_the_end_errors() {
    let err = eval_i64(":user::c18").expect_err("index 99 of a 5-byte string must error");
    let text = format!("{err:?}");
    assert!(
        text.contains("index out of range") && text.contains("byte-length=5"),
        "the error must name the index and the length it exceeded; got {text}"
    );
}

#[test]
fn contract_18_byte_at_negative_index_errors() {
    let err = eval_i64(":user::c19").expect_err("index -1 must error, not wrap");
    assert!(format!("{err:?}").contains("index out of range"));
}

#[test]
fn contract_19_byte_length_of_empty_is_zero() {
    assert_eq!(eval_i64(":user::c20").expect("byte-length"), 0);
}

fn eval_str(fn_name: &str) -> Result<String, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_20_byte_subs_slices() {
    assert_eq!(eval_str(":user::c21").expect("byte-subs"), "el");
}

#[test]
fn contract_21_byte_subs_empty_range() {
    assert_eq!(eval_str(":user::c22").expect("byte-subs"), "");
}

/// A multi-byte character survives when the range is on its boundaries.
#[test]
fn contract_22_byte_subs_keeps_a_whole_character() {
    assert_eq!(eval_str(":user::c23").expect("byte-subs"), "é");
}

/// **And it REFUSES to cut one in half** rather than returning something that is not a String.
/// `str::get` answers None off a boundary, which is the whole reason it is the right primitive.
#[test]
fn contract_23_byte_subs_refuses_a_split_character() {
    let err = eval_str(":user::c24").expect_err("[1,2) splits a 2-byte character");
    let text = format!("{err:?}");
    assert!(
        text.contains("not on a character boundary"),
        "the error must say why, not just that it failed; got {text}"
    );
}

#[test]
fn contract_24_byte_subs_out_of_range_errors() {
    let err = eval_str(":user::c25").expect_err("end 99 of a 5-byte string must error");
    assert!(format!("{err:?}").contains("byte-length=5"));
}
