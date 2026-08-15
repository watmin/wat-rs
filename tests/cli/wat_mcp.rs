//! `wat --mcp` — the MCP server mode.
//!
//! Drives the REAL binary over a pipe (`env!("CARGO_BIN_EXE_wat")`, the `wat_cli.rs` /
//! `wat_repl.rs` pattern), because the thing under test is a mode of the CLI.
//!
//! ## What each test would have to break to go red
//!
//! An MCP server is trivially easy to gate VACUOUSLY — assert the process exits 0 and you
//! have proven a binary starts (R59 `NISI FRANGAS, NIHIL PROBAS`: a suite passed 4105/4105
//! for weeks over a protocol that had never once executed, because nothing in it DEPENDED on
//! the mechanism). So each of these depends on the mechanism it names:
//!
//! 1. `definitions_persist_across_turns` — the load-bearing property, and the whole point of
//!    holding a session at all. Cut the `Declared` arm's `session.defs.push(form)` in
//!    `distribution/mcp.rs` and the second call answers with an unresolved reference instead
//!    of `42`. Nothing else in this file can go red from that edit; this test can only pass
//!    if the definition genuinely crossed a call boundary.
//! 2. `reset_empties_the_session` — the same mechanism inverted, which is why a no-op `reset`
//!    cannot sneak through: it asserts the call SUCCEEDS before the reset and FAILS after.
//!    Delete `session.defs.clear()` and the post-reset call still answers `42` → red.
//! 3. `a_failed_evaluation_is_not_fatal` — failures are values, so the session survives them.
//!    If a bad form ended the process the following good call would never be answered.
//! 4. `the_payload_is_edn_not_json` — the ruling this mode exists to honour: the result rides
//!    as EDN text in a string slot, never converted to JSON. A record answer must come back
//!    as `#ns/Rec {…}`; if anything ever "helpfully" JSON-ified the payload, this goes red.
//! 5. `mcp_rejects_a_positional` — the mode's arity contract, as `--repl` has.

use std::io::Write;
use std::process::{Command, Stdio};

use wat_edn::OwnedValue;

/// The transcripts are JSON-RPC frames carrying wat forms, so they live in co-located
/// fixture files rather than Rust string literals (the `no_inlined_wat_in_tests` /
/// `no_inlined_edn` rubric — a `{`-opening literal full of wat forms is exactly what those
/// lints exist to keep out of `.rs`).
const PERSIST_IN: &str = include_str!("wat_mcp__persist.jsonl");
const MULTIFORM_IN: &str = include_str!("wat_mcp__multiform.jsonl");
const TOPLEVEL_EXPR_IN: &str = include_str!("wat_mcp__toplevel_expr.jsonl");
const RESET_IN: &str = include_str!("wat_mcp__reset.jsonl");
const BAD_THEN_GOOD_IN: &str = include_str!("wat_mcp__bad_then_good.jsonl");

/// Feed `input` to `wat --mcp` over stdin; return `(stdout, exit_code)`.
fn mcp(input: &str) -> (String, Option<i32>) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg("--mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat --mcp");
    child
        .stdin
        .take()
        .expect("mcp stdin")
        .write_all(input.as_bytes())
        .expect("write mcp input");
    let out = child.wait_with_output().expect("wait for wat --mcp");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code(),
    )
}

fn get<'a>(v: &'a OwnedValue, key: &str) -> Option<&'a OwnedValue> {
    match v {
        OwnedValue::Map(entries) => entries.iter().find_map(|(k, val)| match k {
            OwnedValue::String(s) if s.as_ref() == key => Some(val),
            _ => None,
        }),
        _ => None,
    }
}

/// `(text, is_error)` out of one reply frame — the two fields every `tools/call` answers
/// with. Parsed STRUCTURALLY, never matched by substring: a `.contains` would pass on the
/// right text under the wrong `isError`, or on a different frame entirely.
fn tool_reply(line: &str) -> (String, bool) {
    let v = wat_edn::from_json_string(line).expect("a reply frame must be JSON");
    let result = get(&v, "result").expect("a tools/call reply carries a result");
    let content = match get(result, "content") {
        Some(OwnedValue::Vector(items)) => items.clone(),
        other => panic!("content must be a vector; got {other:?}"),
    };
    let text = match content.first().and_then(|c| get(c, "text")) {
        Some(OwnedValue::String(s)) => s.to_string(),
        other => panic!("content[0].text must be a string; got {other:?}"),
    };
    let is_error = matches!(get(result, "isError"), Some(OwnedValue::Bool(true)));
    (text, is_error)
}

fn replies(out: &str) -> Vec<(String, bool)> {
    out.lines().map(tool_reply).collect()
}

#[test]
fn definitions_persist_across_turns() {
    // THE load-bearing property. Call 1 declares; call 2 uses it. Only a session that
    // actually grew its definition set can answer 42 here.
    let (out, code) = mcp(PERSIST_IN);
    let r = replies(&out);
    assert_eq!(r.len(), 2, "one reply per request");
    assert_eq!(r[0], ("nil".to_string(), false), "a declaration answers nil");
    assert_eq!(
        r[1],
        ("42".to_string(), false),
        "a definition from an earlier call must be live in a later one"
    );
    assert_eq!(code, Some(0));
}

#[test]
fn reset_empties_the_session() {
    // The same mechanism inverted — asserting BOTH sides so a `reset` that does nothing
    // cannot pass: it must work before, and it must stop working after.
    let (out, code) = mcp(RESET_IN);
    let r = replies(&out);
    assert_eq!(r.len(), 4);
    assert_eq!(r[1], ("42".to_string(), false), "live before the reset");
    assert_eq!(r[2], ("nil".to_string(), false), "reset answers nil");
    assert!(
        r[3].1,
        "after reset the definition is gone, so the call must FAIL: got {:?}",
        r[3]
    );
    assert_eq!(code, Some(0), "reset is not fatal");
}

#[test]
fn a_failed_evaluation_is_not_fatal() {
    // A failed evaluation is a SUCCESSFUL tool call reporting a failure — the session
    // survives it. If a bad form killed the process, the second reply would never arrive.
    let (out, code) = mcp(BAD_THEN_GOOD_IN);
    let r = replies(&out);
    assert_eq!(r.len(), 2, "the session must answer BOTH calls");
    assert!(r[0].1, "an unresolved reference is reported as isError");
    assert_eq!(
        r[1],
        ("7".to_string(), false),
        "the session must survive and answer the next call"
    );
    assert_eq!(code, Some(0));
}

#[test]
fn the_payload_is_edn_not_json() {
    // The ruling this mode exists to honour: EDN in, EDN out. A record answer comes back as
    // EDN TEXT in the string slot — `#ns/Rec {…}` — never converted into a JSON object.
    let (out, code) = mcp(include_str!("wat_mcp__record.jsonl"));
    let r = replies(&out);
    let (text, is_error) = r.last().expect("a reply").clone();
    assert!(!is_error, "the record form must evaluate: {text}");
    // Compared STRUCTURALLY against a captured golden, not by prefix: the claim is that the
    // payload is an EDN VALUE, and only parsing both sides proves that. A `starts_with` here
    // would pass on a truncated or malformed tail.
    wat::assert_edn_eq!(text, include_str!("wat_mcp__record.edn"));
    // Arc 296 G-2 — the golden USED TO record a defect, deliberately and visibly: the field
    // names came back as `:field-0`/`:field-1`, not the declared `:x`/`:y`, because the
    // renderer recovered names via a registry lookup that this session's symbol table (never
    // having seen the `defrecord`) could not satisfy. `AggregateValue` now carries its own
    // `names` at construction, so the value no longer depends on that lookup — the golden is
    // updated to the real declared names (`:x`/`:y`), and this comparison now checks what the
    // test's docstring always claimed: EDN in, EDN out, faithfully.
    assert_eq!(code, Some(0));
}

#[test]
fn every_form_in_a_payload_takes_effect() {
    // REGRESSION GATE for a shipped hidden failure: the first version evaluated only the
    // FIRST form of a payload and silently dropped the rest, answering `nil` + `isError:
    // false` — success — while the later definitions never existed. Nothing in the original
    // suite could see it, because every fixture sent exactly one form (R59's third face: a
    // gate whose success criteria never touch the mechanism).
    let (out, code) = mcp(MULTIFORM_IN);
    let r = replies(&out);
    assert_eq!(r.len(), 3);
    assert_eq!(r[0], ("nil".to_string(), false), "two declarations answer nil");
    // Only a session where BOTH defns landed can answer 3. Restore
    // `forms.into_iter().next()` in `eval_turn` and this goes red.
    assert_eq!(
        r[1],
        ("3".to_string(), false),
        "form 2 of the payload must take effect, not be dropped"
    );
    // …and a failure mid-payload is REPORTED, not swallowed behind an earlier success.
    assert!(
        r[2].1,
        "a failing form later in the payload must surface: got {:?}",
        r[2]
    );
    assert_eq!(code, Some(0));
}

#[test]
fn a_toplevel_let_or_do_answers_its_value() {
    // `let` and `do` are EXPRESSIONS. A top-level one used to be classified
    // `FormOutcome::Declared` — because one list was answering both "might this carry a def?"
    // (yes, they splice) and "is this a declaration?" (no) — so its value was discarded:
    // `--mcp` answered `nil`, `--repl` printed nothing. Reported by a zero-prior model driving
    // this tool live, then reproduced in both modes.
    //
    // Restore `is_runtime_declaration_head` at either site in `eval_form_against_defs` and the
    // first two of these go red. The third is the control that kept the bug hidden: a NESTED
    // `let` always worked, which is why "let is broken" looked wrong on inspection.
    let (out, code) = mcp(TOPLEVEL_EXPR_IN);
    let r = replies(&out);
    assert_eq!(r.len(), 3);
    assert_eq!(r[0], ("1".to_string(), false), "a top-level let answers its body");
    assert_eq!(r[1], ("3".to_string(), false), "a top-level do answers its last form");
    assert_eq!(
        r[2],
        ("\"wat::core::i64\"".to_string(), false),
        "the nested-let control must stay green"
    );
    assert_eq!(code, Some(0));
}

#[test]
fn mcp_rejects_a_positional() {
    // Same arity contract as `--repl`: the program is baked, so a path would be a silent lie
    // about what runs.
    let bin = env!("CARGO_BIN_EXE_wat");
    let out = Command::new(bin)
        .arg("--mcp")
        .arg("some_program.wat")
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat --mcp with a positional");
    assert_eq!(out.status.code(), Some(64), "usage error is EX_USAGE (64)");
}
