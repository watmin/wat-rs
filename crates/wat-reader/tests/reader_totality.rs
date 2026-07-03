//! wat-reader TOTALITY — the wat *source* reader must never panic.
//!
//! wat-reader reads wat SOURCE, a narrower grammar than general EDN — it is NOT the clj-parity target
//! (that is `wat-edn`, which must *accept* Unicode tokens; the 300 convergence eventually feeds
//! wat-reader from wat-edn). A Unicode symbol is not wat source, so wat-reader doesn't know what to do
//! with it — and that is fine, PROVIDED it errors cleanly. Today its byte-wise `lex_symbol` PANICS
//! (`src[start..i]` slices a multi-byte char mid-boundary). The fix: a clean `LexError`, never a panic.
//!
//! RED at HEAD: `non_ascii_token_errs_not_panics` panics on `"😀"`/`"é"` in token position.

use wat_reader::parse_one_with_file;

/// A non-ASCII byte in TOKEN position must be a clean `Err` (wat-reader doesn't know what to do with a
/// Unicode symbol — that's wat-edn's job), NEVER a panic.
#[test]
fn non_ascii_token_errs_not_panics() {
    for src in ["😀", ":a😀", "é", ":aé", "λ", "foo→bar", "(:a😀)", "expected ∅, got x"] {
        match std::panic::catch_unwind(|| parse_one_with_file(src, "<totality>")) {
            Err(_) => panic!("parse_one_with_file PANICKED on {src:?} — must return a clean Err, not panic"),
            Ok(Ok(_)) => {} // parsing it is also acceptable (it just must not panic)
            Ok(Err(_)) => {} // clean refusal — the expected outcome (not wat source)
        }
    }
}

/// Unicode inside a STRING literal is valid wat source and must parse without panic — must not regress.
#[test]
fn unicode_inside_string_still_parses() {
    for src in ["\"héllo\"", "\"a 😀 b\"", "\"∅\""] {
        match std::panic::catch_unwind(|| parse_one_with_file(src, "<totality>")) {
            Err(_) => panic!("PANICKED on string {src:?}"),
            Ok(Ok(_)) => {}
            Ok(Err(e)) => panic!("string {src:?} should parse (UTF-8 content is valid wat source); got {e:?}"),
        }
    }
}
