//! wat-edn TOKEN GRAMMAR is ASCII — the deliberate "no wide chars in tokens" stance, pinned + given
//! a clear diagnostic.
//!
//! Grounded differential vs `clojure.edn` (Clojure 1.12.4), U+1F600 (😀, supplementary) and U+00E9
//! (é, BMP) by position:
//!
//! | position          | clojure.edn        | wat-edn (this stance)        |
//! |-------------------|--------------------|------------------------------|
//! | sym `😀` / `é`     | PARSED (Symbol)    | REFUSED — non-ASCII in token  |
//! | kw  `:a😀`/`:aé`  | PARSED (Keyword)   | REFUSED — non-ASCII in token  |
//! | string `"😀"`     | PARSED (String)    | PARSED (UTF-8 content OK)     |
//! | char `\😀`        | REFUSED            | REFUSED (BMP-only, arc 218)   |
//! | char `\é`         | PARSED             | PARSED                        |
//!
//! wat-edn is deliberately STRICTER than clojure.edn on symbols/keywords (ASCII-only tokens) — that is
//! the "no wide chars in tokens" choice. It already refuses CLEANLY (never panics). The true-up: the
//! refusal error must NAME the reason (non-ASCII in token; wat tokens are ASCII), not fall through to
//! the generic `unexpected byte 0xNN`.
//!
//! RED at HEAD: the symbol-position refusal is `ErrorKind::UnexpectedByte` → "unexpected byte 0xf0".
//! GREEN when it carries a clear non-ASCII-in-token diagnostic.

fn err_of(src: &str) -> String {
    match wat_edn::parse_owned(src) {
        Ok(v) => panic!("{src:?} unexpectedly PARSED -> {v:?}"),
        Err(e) => e.to_string(),
    }
}

/// Non-ASCII in token position is refused with a CLEAR message (not the generic unexpected-byte).
#[test]
fn non_ascii_token_error_is_clear() {
    for src in ["😀", ":a😀", "é", ":aé"] {
        let msg = err_of(src).to_lowercase();
        assert!(
            (msg.contains("non-ascii") || msg.contains("ascii")) && (msg.contains("token") || msg.contains("symbol")),
            "refusal for {src:?} must name the reason (non-ASCII in token/symbol); got: {}",
            err_of(src)
        );
    }
}

/// The stance itself, pinned: UTF-8 strings parse; ASCII symbols parse; BMP char literals parse;
/// supplementary char literals refuse; non-ASCII tokens refuse.
#[test]
fn token_ascii_stance_pinned() {
    // valid: UTF-8 string content, ASCII symbol, BMP char literal
    for ok in ["\"a 😀 b\"", "\"héllo\"", "foo-bar", ":a-kw", "\\é", "\\newline"] {
        assert!(wat_edn::parse_owned(ok).is_ok(), "{ok:?} should parse");
    }
    // refused: non-ASCII tokens (both BMP + supplementary), supplementary char literal
    for bad in ["😀", ":a😀", "é", ":aé", "\\😀"] {
        assert!(wat_edn::parse_owned(bad).is_err(), "{bad:?} should refuse (ASCII tokens / BMP char literals)");
    }
}
