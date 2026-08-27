//! Arc 170 — forms EDN cannot spell cross the wire VERBATIM, and the whole
//! corpus crosses losslessly.
//!
//! A wat keyword is not an EDN keyword, and a wat symbol is not an EDN symbol.
//! EDN's keyword is a flat `ns/name`; wat's is a small type/path language
//! (`::` segments, `Type/method` accessors, `<>` generics, `(A,B)` tuple types,
//! `Fn(A)->B` function types, trailing-`::` prefix markers). EDN's symbol has no
//! comma in it; wat's generic method heads (`mk<S,R>`) do — and EDN treats `,`
//! as WHITESPACE.
//!
//! Forcing either into EDN's native slot does not merely corrupt a value, it
//! changes the FORM'S ARITY — one token in, several nodes out. Both are carried
//! verbatim in a tagged record instead, exactly as a `.wat` source file carries
//! them. This is the SAME move `#wat.ast/ScopedSymbol` makes for hygiene
//! (`probe_arc170_edn_bridge_hygiene.rs`), applied to what EDN cannot spell.
//!
//! Temporary and self-disarming: it exists only because there are TWO readers
//! (`wat-reader`'s grammar is wider than `wat-edn`'s). When arc 300 retires the
//! rust-scheme surface, the encode-side round-trip test stops firing on its own.

use wat::edn::bridge::{edn_to_program, program_to_edn};

/// Cross a program and require an exact (span-agnostic) identity back.
/// ⛔ ARC 296 STONE M — this helper returns NO `Result`, deliberately.
///
/// It used to be `-> Result<Vec<String>, String>`, flattening two typed errors
/// (`ParseError`, `WatEdnBridgeError`) into one `String` because they have no common type.
/// The stone's usual cure — return the union, `StartupError` — does NOT apply here:
/// `WatEdnBridgeError` (`src/edn/bridge.rs:302`) has no `StartupError` variant, and inventing
/// one to satisfy a rule would be a LIE (a bridge failure is not a startup failure).
///
/// The real question is what the `Err` was FOR. Measured: both call sites `.expect(…)` it and
/// nothing in this file ever inspects it. A parse failure or a decode failure here is BROKEN
/// FIXTURE, not the property under test — the test is round-trip identity, and `Ok` already
/// carries the findings as `Vec<String>`. So the honest shape is no `Result` at all: panic on
/// a broken precondition, carrying each error's OWN typed `Debug` plus the EDN frame, which is
/// strictly more than the flattened string said.
fn crosses(src: &str, file: &str) -> Vec<String> {
    let forms = wat::parse_all_with_file(src, file)
        .unwrap_or_else(|e| panic!("fixture must parse — {file}: {e:?}"));
    let edn = program_to_edn(&forms);
    let back = edn_to_program(&edn)
        .unwrap_or_else(|e| panic!("fixture must decode — {e:?} — frame: {edn}"));
    let mut bad = Vec::new();
    for (i, (a, b)) in forms.iter().zip(back.iter()).enumerate() {
        if a != b {
            bad.push(format!("form #{i} crossed but CHANGED"));
        }
    }
    if forms.len() != back.len() {
        bad.push(format!("ARITY changed: {} forms in, {} out", forms.len(), back.len()));
    }
    bad
}

/// C01 — every class of wat lexeme EDN cannot spell crosses intact.
///
/// The exemplars live in a co-located `.wat` fixture (one form per class, each
/// labelled in the file) rather than inline strings — `no_inlined_wat_in_tests`,
/// and the fixture doubles as corpus the C03 sweep also covers.
#[test]
fn c01_every_unspellable_lexeme_crosses_intact() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/program/probe_arc170_edn_bridge_unspellable__lexemes.wat"
    );
    let src = std::fs::read_to_string(path).expect("the lexeme fixture must be readable");
    let bad = crosses(&src, path);
    assert!(
        bad.is_empty(),
        "wat lexemes that did not survive the wire. Each is ONE token to \
         wat-reader and none is spellable as a native EDN keyword/symbol, so \
         each must be carried VERBATIM — forcing it into EDN's native slot \
         changes the form's ARITY:\n  {}",
        bad.join("\n  ")
    );
}

/// C02 — the CONTROL: an ordinary program crosses as PLAIN EDN, unwrapped —
/// modulo the two DECLARED carriage tags (span carriage, stone J/296).
///
/// Without this, "wrap everything" would satisfy C01 while making every frame
/// unreadable and churning every golden. Asserted STRUCTURALLY — the frame is
/// parsed and walked for tagged nodes, never `.contains()` on the text
/// (`no_loose_string_assert`; a wat frame is EDN, so assert the structure).
///
/// Verbatim carriage (what C01/C03 pin) exists for wat lexemes EDN cannot
/// spell at all — a keyword path, a generic method head. Span carriage
/// (`#wat.ast/Spanned`, `#wat.ast/Program`, stone J) is a SEPARATE, declared
/// vocabulary: it wraps forms EDN spells just fine, to carry a `Span` that EDN
/// has no native slot for. Neither licenses the other — an unspellable lexeme
/// wrapper does not excuse a gratuitous span wrap elsewhere, and the span
/// carriage does not excuse wrapping some other lexeme "while we're at it".
/// So this control exempts EXACTLY those two known, declared tags — by exact
/// name, never a prefix or a namespace-wide skip — and keeps firing on
/// anything else, which is still the whole reason it exists.
#[test]
fn c02_control_ordinary_forms_stay_plain_edn() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/resolve/probe_arc251_fix_source_head_rule__contract-01-bare-call-head-inverted.wat"
    );
    let src = std::fs::read_to_string(path).expect("an ordinary corpus file must be readable");
    let forms = wat::parse_all_with_file(&src, path).expect("parse");
    let frame = program_to_edn(&forms);
    let parsed = wat_edn::parse_owned(&frame).expect("the frame must be valid EDN");

    fn tags(v: &wat_edn::OwnedValue, out: &mut Vec<String>) {
        use wat_edn::Value::*;
        match v {
            Tagged(t, b) => {
                out.push(format!("{}/{}", t.namespace(), t.name()));
                tags(b, out);
            }
            List(xs) | Vector(xs) | Set(xs) => xs.iter().for_each(|x| tags(x, out)),
            Map(kvs) => kvs.iter().for_each(|(k, v)| {
                tags(k, out);
                tags(v, out);
            }),
            _ => {}
        }
    }
    let mut found = Vec::new();
    tags(&parsed, &mut found);
    // The two declared span-carriage tags (stone J, arc 296) — exact names
    // only. Anything else surviving this filter is a DIFFERENT wrapper this
    // control must still catch (STOP-1: widening this beyond two exact names
    // would be silencing the alarm, not satisfying it).
    const SPAN_CARRIAGE_TAGS: [&str; 2] = ["wat.ast/Spanned", "wat.ast/Program"];
    found.retain(|t| !SPAN_CARRIAGE_TAGS.contains(&t.as_str()));
    assert_eq!(
        found,
        Vec::<String>::new(),
        "an ordinary program must cross as PLAIN EDN, modulo the two declared \
         span-carriage tags ({SPAN_CARRIAGE_TAGS:?}) — verbatim carriage is for \
         what EDN cannot spell, span carriage is a separate declared \
         vocabulary for what EDN CAN spell but has no slot for a Span on, and \
         neither licenses wrapping anything else. Frame: {frame}"
    );

    let bad = crosses(&src, path);
    assert!(bad.is_empty(), "and it must still round-trip: {bad:?}");
}

/// C03 — THE GATE: every `.wat` in the tree crosses the wire losslessly.
///
/// This is the measure nothing in the tree had. `program_to_edn`'s two existing
/// round-trip probes feed it only `parse_all!` snippets, so 618 of 1223 corpus
/// files were failing the crossing — 444 of them on `Type/method` accessors —
/// while every EDN test stood green. A sweep over the REAL corpus is the only
/// thing that catches a lexeme nobody thought to enumerate, and it DISCOVERS
/// rather than lists, so a new `.wat` anywhere is covered with no fixture list
/// to drift.
#[test]
fn c03_the_whole_corpus_crosses_the_wire() {
    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if !p.ends_with("target") {
                    collect(&p, out);
                }
            } else if p.extension().is_some_and(|x| x == "wat") {
                out.push(p);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for d in ["wat", "wat-tests", "wat-scripts", "tests", "examples", "docs", "crates"] {
        collect(&root.join(d), &mut files);
    }
    files.sort();

    let mut checked = 0usize;
    let mut broken = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        // Only files that PARSE are in scope — a deliberately-unparseable
        // negative fixture is not this gate's business.
        let Ok(forms) = wat::parse_all_with_file(&src, &f.to_string_lossy()) else { continue };
        checked += 1;
        let rel = f.strip_prefix(root).unwrap_or(f).display().to_string();
        let edn = program_to_edn(&forms);
        match edn_to_program(&edn) {
            Ok(back) if back == forms => {}
            Ok(_) => broken.push(format!("{rel}: crossed but CHANGED (silent corruption)")),
            Err(e) => broken.push(format!("{rel}: {e}")),
        }
    }

    assert!(
        checked > 1000,
        "the gate must actually walk the corpus; only {checked} files parsed — \
         the collector is probably pointed at the wrong root"
    );
    assert!(
        broken.is_empty(),
        "{} of {checked} corpus files do NOT survive program_to_edn → \
         edn_to_program. Every one is a program that cannot be shipped to a \
         child process. First failures:\n  {}",
        broken.len(),
        broken.iter().take(15).cloned().collect::<Vec<_>>().join("\n  ")
    );
}
