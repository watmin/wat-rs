//! Stone 255.59. Ignored: the source reader does not track the clj oracle yet.
//! `cargo test -p wat-reader --test clj_oracle_source_parity -- --ignored`
//!
//! The golden stores a verdict, not a value (`clj_oracle/golden.txt`). Where both
//! sides accept one form, this also asks `wat_edn::parse_owned` — the data reader
//! that already matches those verdicts. A line the data reader refuses (multi-slash
//! symbols) is left to the score; it is not a failure here.
//! A red count is not the score's divergence count: `Symbol::try_new` also
//! rejects names clj accepts (`a:b`, `/`, `.1`), and those rows fail here too.

use std::borrow::Cow;
use wat_edn::value::{Keyword, Symbol};
use wat_edn::OwnedValue;
use wat_reader::{parse_all_with_file, WatAST};

const CORPUS: &str = include_str!("../../wat-edn/tests/clj_oracle/corpus.txt");
const GOLDEN: &str = include_str!("../../wat-edn/tests/clj_oracle/golden.txt");

#[test]
#[ignore = "255.59: wat-reader diverges from the clj oracle"]
fn source_reader_tracks_the_clj_oracle() {
    let mut fails: Vec<String> = Vec::new();
    for (input, gline) in CORPUS.lines().zip(GOLDEN.lines()) {
        if input.is_empty() {
            continue;
        }
        let (verdict, ginput) = gline.split_once('\t').expect("golden tab");
        assert_eq!(ginput, input, "golden input drifted");
        let reader = parse_all_with_file(input, "corpus");
        let golden_ok = verdict == "OK";
        let reader_ok = reader.is_ok();
        if reader_ok != golden_ok {
            if reader_ok && desugared_macro(&reader) {
                continue;
            }
            fails.push(format!(
                "verdict clj={verdict} reader={} {input:?}",
                if reader_ok { "OK" } else { "ERR" }
            ));
            continue;
        }
        if !golden_ok {
            continue;
        }
        let fs = reader.expect("reader ok");
        if fs.len() != 1 {
            fails.push(format!("forms {} {input:?}", fs.len()));
            continue;
        }
        if reader_macro_head(&fs[0]).is_some() {
            continue;
        }
        let Ok(expected) = wat_edn::parse_owned(input) else {
            continue;
        };
        match ast_to_value(&fs[0]) {
            Ok(got) if got == expected => {}
            Ok(_) => fails.push(format!("shape {input:?}")),
            Err(e) => fails.push(format!("unmap {e} {input:?}")),
        }
    }
    assert!(
        fails.is_empty(),
        "source reader vs clj oracle ({}):\n{}",
        fails.len(),
        fails.join("\n")
    );
}

fn desugared_macro(reader: &Result<Vec<WatAST>, wat_reader::ParseError>) -> bool {
    match reader {
        Ok(fs) if fs.len() == 1 => reader_macro_head(&fs[0]).is_some(),
        _ => false,
    }
}

fn reader_macro_head(a: &WatAST) -> Option<&str> {
    let WatAST::List(xs, _) = a else { return None };
    let WatAST::Keyword(h, _) = xs.first()? else { return None };
    match h.as_str() {
        ":wat::core::quote"
        | ":wat::core::quasiquote"
        | ":wat::core::unquote"
        | ":wat::core::unquote-splicing"
        | ":wat::holon::literal" => Some(h.as_str()),
        _ => None,
    }
}

fn ast_to_value(a: &WatAST) -> Result<OwnedValue, String> {
    use WatAST::*;
    Ok(match a {
        NilLit(_) => OwnedValue::Nil,
        BoolLit(b, _) => OwnedValue::Bool(*b),
        IntLit(n, _) => OwnedValue::Integer(*n),
        FloatLit(n, _) => OwnedValue::Float(*n),
        RationalLit(r, _) => OwnedValue::Rational(Box::new(r.clone())),
        BigIntLit(n, _) => OwnedValue::BigInt(Box::new(n.clone())),
        StringLit(s, _) => OwnedValue::String(Cow::Owned(s.clone())),
        CharLit(c, _) => OwnedValue::Char(*c),
        Keyword(k, _) => OwnedValue::Keyword(keyword_of(k)?),
        Symbol(id, _) => OwnedValue::Symbol(symbol_of(id.as_str())?),
        List(xs, _) => {
            if reader_macro_head(a).is_some() {
                return Err("reader-macro".into());
            }
            OwnedValue::List(xs.iter().map(ast_to_value).collect::<Result<Vec<_>, _>>()?)
        }
        Vector(xs, _) => {
            OwnedValue::Vector(xs.iter().map(ast_to_value).collect::<Result<Vec<_>, _>>()?)
        }
        Map(pairs, _) => {
            let mut out = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                out.push((ast_to_value(k)?, ast_to_value(v)?));
            }
            OwnedValue::Map(out)
        }
        Set(xs, _) => {
            OwnedValue::Set(xs.iter().map(ast_to_value).collect::<Result<Vec<_>, _>>()?)
        }
    })
}

fn keyword_of(stored: &str) -> Result<Keyword, String> {
    let body = stored
        .strip_prefix(':')
        .ok_or("keyword missing leading colon")?;
    if body.contains(':') {
        return Err(format!("colon in keyword body {stored:?}"));
    }
    if let Some((ns, name)) = body.split_once('/') {
        if name.contains('/') {
            return Err(format!("multi-slash keyword {stored:?}"));
        }
        return Keyword::try_ns(ns, name).map_err(|e| e.to_string());
    }
    Keyword::try_new(body).map_err(|e| e.to_string())
}

fn symbol_of(flat: &str) -> Result<Symbol, String> {
    if flat.contains(':') {
        return Err(format!("colon in symbol {flat:?}"));
    }
    if let Some((ns, name)) = flat.split_once('/') {
        if name.contains('/') || ns.is_empty() || name.is_empty() {
            return Err(format!("slash-edge symbol {flat:?}"));
        }
        return Symbol::try_ns(ns, name).map_err(|e| e.to_string());
    }
    Symbol::try_new(flat).map_err(|e| e.to_string())
}
