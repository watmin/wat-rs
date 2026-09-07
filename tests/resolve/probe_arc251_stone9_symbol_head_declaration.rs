//! PROBE — arc 251 stone 251.9: a symbol-headed declaration must not EVAPORATE.
//!
//! `src/declare/parse.rs:199` `is_declaration_form` reads a form's head as
//! `WatAST::Keyword(k, _) => k.as_str(), _ => return false`. A `Symbol` head takes the
//! catch-all, so the form is classified `FormOutcome::Evaluated` instead of `::Declared`:
//! it evaluates, yields a value, and grows the session by NOTHING. No error, no diagnostic.
//!
//! ⚠ EVERY ROW'S BAR IS THE KEYWORD-HEAD CONTROL, run in the same test — never a hand-written
//! expectation. The defect's signature is a PASS, so a row that asserted "expect success"
//! would be satisfied by the very defect it exists to catch.
//!
//! The two programs of a pair are byte-identical below line 1, asserted here rather than
//! promised in a comment — so a green row cannot come from the fixtures having drifted apart.
//!
//! ⚠ THE MALFORMATIONS ARE DIALECT-NEUTRAL, ON PURPOSE. The first draft used a bare-symbol
//! variant (`Red []`) as the negative control — which 251.5's target declaration makes CORRECT
//! (`(wat.core/defenum probe/Color wat.enum/Pure  Red :- [])`). That row would have pinned a
//! legacy rule as permanently true and forced 251.5 to violate its own arc's acceptance test.
//! Both controls now fail for a reason the clojure flip does not touch: a payload that is not a
//! vector, and a missing mandatory purity marker.
//!
//! Un-ignored by stone 251.9 — the catch-all that ate the declaration.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/resolve").join(name)
}

fn exit_code(path: &PathBuf) -> i32 {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    out.status.code().unwrap_or(-1)
}

/// Run a fixture PAIR, first proving the pair differs only in its head spelling.
fn pair(case: &str) -> (i32, i32) {
    let kw_path = fixture(&format!("probe_arc251_stone9__{case}_kw.wat"));
    let sym_path = fixture(&format!("probe_arc251_stone9__{case}_sym.wat"));
    let kw_src = std::fs::read_to_string(&kw_path).expect("kw fixture");
    let sym_src = std::fs::read_to_string(&sym_path).expect("sym fixture");
    let tail = |s: &str| s.lines().skip(1).map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(tail(&kw_src), tail(&sym_src), "{case}: the pair must differ ONLY on line 1");
    assert_ne!(
        kw_src.lines().next(),
        sym_src.lines().next(),
        "{case}: the pair's first lines are identical — the head spelling was never swapped"
    );
    (exit_code(&kw_path), exit_code(&sym_path))
}

#[test]
fn malformed_variant_is_refused_under_both_head_spellings() {
    let (kw, sym) = pair("malformed_variant");
    assert_ne!(kw, 0, "control: the keyword head MUST refuse a non-vector variant payload");
    assert_eq!(
        sym, kw,
        "a symbol-headed defenum swallowed a malformed variant payload the keyword head refuses \
         — the declaration parser never ran (declare/parse.rs:199 catch-all)"
    );
}

#[test]
fn missing_mandatory_purity_marker_is_refused_under_both_head_spellings() {
    let (kw, sym) = pair("missing_purity");
    assert_ne!(kw, 0, "control: the keyword head MUST refuse a missing purity marker");
    assert_eq!(
        sym, kw,
        "a symbol-headed defenum swallowed a MANDATORY marker's absence — the form \
         evaporated into FormOutcome::Evaluated"
    );
}

#[test]
fn a_symbol_headed_declaration_actually_declares() {
    let (kw, sym) = pair("declares");
    assert_eq!(kw, 0, "control: the keyword head declares and the reference resolves");
    assert_eq!(
        sym, kw,
        "a symbol-headed defenum did not register its type — the reference to \
         :probe::Color is unresolved"
    );
}
