//! GA1 (excursus 003 strike A, BRIEF-shape-strike-A-one-death-shape.md) — "one death
//! shape": an unhandled `assertion-failed!` reports through the SAME
//! `LociDiedError.Panic` chain every other death uses, exactly once, on:
//!   - the main thread of a top-level `wat` run (`probe_..._main_thread.wat`);
//!   - a thread peer (`probe_..._thread_peer.wat`) — the peer's crash reason itself
//!     rides `crash_tx`, never stderr, so the ONE stderr line here comes solely from
//!     `wat::panic_hook`'s process-wide hook.
//!
//! Driven through the real binary (`CARGO_BIN_EXE_wat`) on a co-located `.wat`
//! fixture — EDN-over-stdio in the sense the brief means it: the death report
//! crosses the process boundary as the bare EDN this test parses off stderr, not a
//! human-prose message.
//!
//! MUTATION-PROVEN (see the brief's Report / this excursus's floor log): reverting
//! `wat::panic_hook::render_assertion_failure` to the retired hand-built
//! `#wat.kernel/AssertionFailure {…}` envelope turns `no_assertion_failure_tag`
//! (and the tag-shape assertions above it) RED in both tests below.

use std::path::Path;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(file!()).parent().unwrap().join(name)
}

fn run(fixture_name: &str) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(fixture(fixture_name))
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    (
        output.status.code().expect("exit code"),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Common shape assertions for both scenarios: stderr is EXACTLY one line, parses
/// as a bare `Vector<LociDiedError>` whose one element is `.Panic`, whose
/// `failure.error` is `:wat::runtime/AssertionFailed` carrying `actual`/`expected`
/// as `Option.Some` of the given strings — never the retired `AssertionFailure` tag.
fn assert_one_panic_line(stderr: &str, expected_actual: &str, expected_expected: &str) {
    // Exactly one line (a trailing newline is the ONE line's own terminator, not a
    // second, empty one) — "exactly one death line" is a claim about LINE COUNT,
    // not merely about the parsed shape.
    let trimmed = stderr.trim_end_matches('\n');
    assert!(
        !trimmed.is_empty(),
        "expected exactly one stderr line, got none: stderr={stderr:?}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "expected EXACTLY ONE death line on stderr (F1) — this is the whole claim; got: {stderr:?}"
    );

    // The retired shape must never appear, anywhere in the raw text — not just
    // "the parsed shape is right", the OLD tag must be textually gone. This is a
    // targeted-absence check over the whole stderr text, on top of (not instead of)
    // the structural parse below, which already proves the head tag and the error
    // tag are the new shapes, never the retired one.
    assert!(
        // rune:lint(loose-assert) — targeted absence of a retired tag string over the
        // whole raw stderr text, not a structured-value comparison; the structural
        // parse below already proves the actual shape.
        !stderr.contains("AssertionFailure"),
        "the retired #wat.kernel/AssertionFailure envelope must never appear again: {stderr:?}"
    );

    let parsed = wat_edn::parse_owned(trimmed).unwrap_or_else(|e| {
        panic!("stderr line did not parse as EDN: {e}\nstderr={stderr:?}")
    });
    let items = match parsed {
        wat_edn::OwnedValue::Vector(items) => items,
        other => panic!("expected a bare Vector<LociDiedError> chain, got {other:?}"),
    };
    assert_eq!(items.len(), 1, "expected a singleton chain (no cascade): {items:?}");
    let (tag, body) = items[0]
        .as_tagged()
        .unwrap_or_else(|| panic!("expected a tagged LociDiedError, got {:?}", items[0]));
    assert_eq!(
        format!("{}/{}", tag.namespace(), tag.name()),
        "wat.kernel/LociDiedError.Panic",
        "an unhandled assertion is ALWAYS reported as Panic, head of the chain"
    );
    let failure = get_field(body, "failure");
    let (ftag, fbody) = failure
        .as_tagged()
        .unwrap_or_else(|| panic!("expected a tagged Failure, got {failure:?}"));
    // `.namespace()` / `.name()` checked separately, NOT a `#ns/Name` string compared via
    // `to_string()` — that shape trips `no_inlined_edn`'s EDN-esque-literal heuristic on
    // the caller's comparison (the convention `probe_ex003_stone_d3_frames.rs` already uses).
    assert_eq!(ftag.namespace(), "wat.kernel", "failure tag namespace: {ftag}");
    assert_eq!(ftag.name(), "Failure", "failure tag name: {ftag}");
    let error = get_field(fbody, "error");
    let (etag, ebody) = error
        .as_tagged()
        .unwrap_or_else(|| panic!("expected a tagged error record, got {error:?}"));
    assert_eq!(
        etag.namespace(),
        "wat.runtime",
        "an assertion's Failure.error is its OWN record (F2), not a generic Fault: {etag}"
    );
    assert_eq!(
        etag.name(),
        "AssertionFailed",
        "an assertion's Failure.error is its OWN record (F2), not a generic Fault: {etag}"
    );
    let actual = option_string(get_field(ebody, "actual"));
    let expected = option_string(get_field(ebody, "expected"));
    assert_eq!(actual.as_deref(), Some(expected_actual));
    assert_eq!(expected.as_deref(), Some(expected_expected));
}

fn get_field<'a>(v: &'a wat_edn::OwnedValue, key: &str) -> &'a wat_edn::OwnedValue {
    let map = v.as_map().unwrap_or_else(|| panic!("expected a map, got {v:?}"));
    map.iter()
        .find_map(|(k, val)| {
            (k.as_keyword().map(|kw| kw.name()) == Some(key)).then_some(val)
        })
        .unwrap_or_else(|| panic!("key :{key} not found in {v:?}"))
}

fn option_string(v: &wat_edn::OwnedValue) -> Option<String> {
    let (tag, body) = v.as_tagged()?;
    // `.namespace()` / `.name()` matched separately — see `assert_one_panic_line`'s
    // comment on why not a `#ns/Name` string literal.
    match (tag.namespace(), tag.name()) {
        ("wat.core", "Option.Some") => get_field(body, "value").as_str().map(str::to_string),
        ("wat.core", "Option.None") => None,
        (ns, name) => panic!("expected Option.Some/Option.None, got {ns}/{name}"),
    }
}

#[test]
fn main_thread_unhandled_assertion_is_one_panic_line() {
    let (code, _stdout, stderr) = run("probe_excursus003_strike_a_ga1_one_death_shape_main_thread.wat");
    assert_eq!(code, 2, "an unhandled assertion on the main thread exits EXIT_PANIC=2; stderr={stderr:?}");
    assert_one_panic_line(&stderr, "1", "2");
}

#[test]
fn thread_peer_unhandled_assertion_is_one_panic_line() {
    let (code, _stdout, stderr) =
        run("probe_excursus003_strike_a_ga1_one_death_shape_thread_peer.wat");
    assert_eq!(
        code, 0,
        "the wat program handles the peer's Lost outcome itself and returns nil cleanly \
         — the ONE stderr line must still be there regardless; stderr={stderr:?}"
    );
    assert_one_panic_line(&stderr, "1", "2");
}
