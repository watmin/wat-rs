//! wat-edn PARITY with the clj oracle on Unicode tokens.
//!
//! Grounded differential (Clojure 1.12.4, `clojure.edn/read-string`): Unicode is valid in symbol and
//! keyword position — clj reads `😀`/`é`/`λ`/`foo→bar` as Symbols and `:a😀`/`:λ` as Keywords. wat-edn
//! currently REFUSES them (`unexpected byte 0xNN`). Non-parity is an illegal state; clj is the oracle,
//! so wat-edn must ACCEPT them.
//!
//! Char literals stay BMP-only — clj refuses supplementary `\😀` ("Unsupported character"), so wat-edn
//! refusing it too is mutual parity (arc 218), not a wat quirk.
//!
//! RED at HEAD: `unicode_tokens_parse` fails (wat-edn refuses Unicode symbols/keywords clj accepts).

/// Every Unicode token clj accepts, wat-edn must accept.
#[test]
fn unicode_tokens_parse() {
    for src in ["😀", ":a😀", "é", ":aé", "λ", ":λ", "foo→bar"] {
        assert!(
            wat_edn::parse_owned(src).is_ok(),
            "{src:?} must parse (clj reads it as a Symbol/Keyword); got {:?}",
            wat_edn::parse_owned(src)
        );
    }
}

/// The mutual boundary: supplementary char literal refused by both; BMP char literal + UTF-8 string parse in both.
#[test]
fn char_and_string_boundary_matches_clj() {
    assert!(wat_edn::parse_owned("\\😀").is_err(), "\\😀 char literal refused (clj refuses it — mutual parity)");
    assert!(wat_edn::parse_owned("\\é").is_ok(),   "\\é (BMP) char literal parses (clj parses it)");
    assert!(wat_edn::parse_owned("\"a 😀 b\"").is_ok(), "UTF-8 string content parses (clj parses it)");
}
