//! A `:then` OPERAND ERROR NAMES THE FORM AS WRITTEN — not as a Rust `Debug` dump.
//!
//! Found 2026-08-27 by probing the ward list's `conformare` x9 finding rather than trusting it.
//! That finding — *"a real wat span was in scope and discarded for `rust_caller_span!()`"* — is
//! STALE: neither cited file uses `rust_caller_span!` any more, and the error's `:location` points
//! at the user's own file. But the probe surfaced a live defect the finding did not name: the
//! operand in `:got` was rendered with Rust `Debug`, so an unbound `?var` in a `:then` showed
//!
//! ```text
//! got wat::core::String "Symbol(Identifier { name: \"?nope\", scopes: {} }, Span { file: … })"
//! ```
//!
//! Fixed by routing to `validate::render_form` — the structural printer `:wat::core::write-forms`
//! already uses — rather than growing a second renderer. Both halves of the compiled/interpreted
//! differential moved together, because `compiled_rhs`'s error is contracted to be BYTE-IDENTICAL
//! to `build_insert_fact`'s and changing one alone would have broken that silently.
//!
//! Asserted EXACTLY, by parsing the EDN and reading `:got`'s `:rendered`. A `contains` check would
//! have been the easy road — and `tests/lint/no_loose_string_assert.rs` refuses it, rightly: this
//! value is deterministic, so "the message mentions ?nope somewhere" is a weaker claim than the
//! one worth making.

use std::path::Path;
use std::process::{Command, Stdio};
use wat_edn::{Keyword, OwnedValue};

const FIXTURE: &str = "tests/rete/probe_arc278_then_operand_rendered_as_source.wat";

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

/// The 1-indexed line the offending `?nope` sits on, READ FROM THE FIXTURE.
///
/// Not a literal: the first version of this test hardcoded `8`, and adding a header comment to the
/// fixture moved the rule to line 20 and turned the assertion red. A line number written in a
/// second file is a fact that rots the moment anyone edits the first.
fn nope_line() -> i64 {
    let src = std::fs::read_to_string(fixture_path()).expect("read the fixture");
    // NON-COMMENT lines only. The fixture's own header quotes the old Debug output, which contains
    // `?nope` — so a naive scan matches the PROSE and reports line 6 while the span says 20. Third
    // time today a check has read a comment as evidence (the parity lint did it twice); the tell is
    // always the same: the file describes the thing it is also demonstrating.
    src.lines()
        .position(|l| l.contains("?nope") && !l.trim_start().starts_with(";;"))
        .map(|i| i as i64 + 1)
        .expect("the fixture must contain the offending `?nope` in CODE, not only in prose")
}

fn run(rel: &str) -> (bool, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let out = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The error map, parsed. Wire shape is nested: the outer `LociDiedError` carries a vector of
/// STRINGS, each the EDN text of one error — so it is parsed twice.
fn error_map(stderr: &str) -> Vec<(OwnedValue, OwnedValue)> {
    let outer = wat_edn::parse_owned(stderr.trim()).expect("the refusal must be EDN on stderr");
    let inner = match outer {
        OwnedValue::Vector(mut xs) if !xs.is_empty() => match xs.remove(0) {
            OwnedValue::Tagged(_, body) => match *body {
                OwnedValue::Vector(mut ss) if !ss.is_empty() => match ss.remove(0) {
                    OwnedValue::String(s) => s.to_string(),
                    other => panic!("expected the error as an EDN string; got {other:?}"),
                },
                other => panic!("expected a vector of error strings; got {other:?}"),
            },
            other => panic!("expected a tagged LociDiedError; got {other:?}"),
        },
        other => panic!("expected a vector at the top; got {other:?}"),
    };
    match wat_edn::parse_owned(&inner).expect("inner error must be EDN") {
        OwnedValue::Tagged(_, body) => match *body {
            OwnedValue::Map(m) => m,
            other => panic!("expected a map body; got {other:?}"),
        },
        other => panic!("expected a tagged error; got {other:?}"),
    }
}

fn field<'a>(m: &'a [(OwnedValue, OwnedValue)], name: &str) -> &'a OwnedValue {
    m.iter()
        .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new(name)))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("the error must carry :{name}; got {m:?}"))
}

#[test]
fn an_unbound_then_operand_is_named_as_source_not_as_rust_debug() {
    let (ok, stderr) = run("tests/rete/probe_arc278_then_operand_rendered_as_source.wat");
    assert!(!ok, "an unbound `?var` in a `:then` must not fire cleanly\n{stderr}");

    let e = error_map(&stderr);
    let got = match field(&e, "got") {
        OwnedValue::Map(m) => m.clone(),
        other => panic!(":got must be a snapshot map; got {other:?}"),
    };
    let rendered = match field(&got, "rendered") {
        OwnedValue::String(s) => s.to_string(),
        other => panic!(":rendered must be a String; got {other:?}"),
    };
    assert_eq!(
        rendered, "\"?nope\"",
        "the operand must be named AS WRITTEN. A Rust `Debug` rendering here shows the user \
         `Symbol(Identifier {{ name: \"?nope\", scopes: {{}} }}, Span {{ … }})` — hygiene scopes \
         and a nested span — for a typo."
    );
}

/// The `conformare` half, which was ALREADY correct and is pinned so it stays that way: the error
/// locates the user's own source, not wat-rs's Rust. The ward list claimed otherwise; it was stale.
#[test]
fn the_error_locates_the_users_file_not_the_engines() {
    let (_, stderr) = run("tests/rete/probe_arc278_then_operand_rendered_as_source.wat");
    let e = error_map(&stderr);
    let span = match field(&e, "location") {
        OwnedValue::Tagged(_, body) => match *body.clone() {
            OwnedValue::Map(m) => m,
            other => panic!(":location must carry a Span map; got {other:?}"),
        },
        other => panic!(":location must be a tagged Span; got {other:?}"),
    };
    let file = match field(&span, "file") {
        OwnedValue::String(s) => s.to_string(),
        other => panic!(":file must be a String; got {other:?}"),
    };
    // EXACT, not `ends_with`: the probe builds this path itself, so the expected value is known
    // precisely — and `tests/lint/no_loose_string_assert.rs` refuses the loose form where an exact
    // one is available, which it is here.
    // The span records the path REPO-RELATIVE for a file inside the tree — so the expected value
    // is exactly `FIXTURE`. (An absolute path appears when the file is outside the repo, which is
    // how this was first observed from a scratch dir.)
    assert_eq!(
        file, FIXTURE,
        "the span must point at the USER's file; a path under src/ would mean the engine's own \
         source was stamped instead"
    );
    assert_eq!(
        field(&span, "line"),
        &OwnedValue::Integer(nope_line()),
        "the span must point at the offending operand's own line"
    );
}
