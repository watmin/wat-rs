//! wat-reader TOTALITY — the source reader must never panic: for ANY input, `parse_one_with_file`
//! returns `Ok` or `Err`, never unwinds.
//!
//! The defect (surfaced by the arc-278 reader-based inline-wat gate force-feeding arbitrary strings):
//! a non-ASCII byte in TOKEN position makes the byte-wise `lex_symbol` advance mid-UTF-8-char and
//! `src[start..i]` slices on a non-char-boundary → PANIC. wat's deliberate stance (parity with
//! `wat-edn`'s ASCII-only token grammar; see `crates/wat-edn` — and note `clojure.edn` is *more*
//! permissive, accepting Unicode symbols) is that a non-ASCII token is INVALID — so the reader must
//! REFUSE it with a clean `LexError`, never panic. `wat-edn` already does exactly this (clean `Err`);
//! `wat-reader` panics — this probe pins that it must not.
//!
//! RED at HEAD: `non_ascii_in_token_position` panics on `"😀"`/`"é"` in symbol/keyword position.
//! GREEN when `lex_symbol` refuses a non-ASCII byte with a clean `LexError` instead of mid-slicing.

use wat_reader::parse_one_with_file;

/// Every input either parses or errors — it never panics. Non-ASCII in TOKEN position must be a
/// clean `Err` (wat tokens are ASCII); non-ASCII inside a STRING is valid and must parse.
#[test]
fn non_ascii_in_token_position() {
    // (input, must_parse) — string content is valid UTF-8 (parity with wat-edn + clojure.edn);
    // symbol/keyword content is ASCII-only, so a non-ASCII byte there is a clean refusal.
    let token_cases = ["😀", ":a😀", "é", ":aé", "(:a😀)", "foo∅bar", "expected ∅, got String"];
    for src in token_cases {
        match std::panic::catch_unwind(|| parse_one_with_file(src, "<totality>")) {
            Err(_) => panic!("parse_one_with_file PANICKED on {src:?} — a non-ASCII token must be a clean Err, not a panic"),
            Ok(Ok(_)) => panic!("{src:?} parsed OK — a non-ASCII byte in token position is not valid wat"),
            Ok(Err(_)) => {} // clean refusal — correct
        }
    }
}

/// Non-ASCII inside a STRING literal is valid (UTF-8) and must parse without panic — this must NOT
/// regress when the token-position guard lands.
#[test]
fn non_ascii_inside_string_still_parses() {
    for src in ["\"héllo\"", "\"a 😀 b\"", "\"∅\""] {
        match std::panic::catch_unwind(|| parse_one_with_file(src, "<totality>")) {
            Err(_) => panic!("PANICKED on string {src:?}"),
            Ok(Ok(_)) => {} // UTF-8 string content is valid — correct
            Ok(Err(e)) => panic!("string {src:?} should parse (UTF-8 content is valid); got Err {e:?}"),
        }
    }
}
