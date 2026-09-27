//! Excursus 003 step 3b, gate G3 — one failure shape per `LociDiedError` variant: for each
//! variant, drive ONE real producer and assert the payload is a `Failure` whose `error` is a
//! record, not a string.
//!
//! **Producers driven, one per variant:**
//! - `Panic` — a spawned child's own `(:wat::kernel::assertion-failed! …)` (an
//!   `AssertionPayload` panic).
//! - `RuntimeError` — a spawned child's integer division by zero (the same producer gate G2
//!   drives independently, for the "typed across the boundary" claim specifically).
//! - `StartupError` — a direct `wat` binary invocation on a program using the retired colon
//!   spelling of `Option::Some` (`src/resolve`'s `UnresolvedReferences`) — the same trigger
//!   `probe_arc255_the_blanket_hides_a_phantom_head__colon_spelling_is_refused.wat` uses.
//! - `MainSignature` — a direct `wat` binary invocation on a program with no `:user::main` at
//!   all: `tests/cli/wat_cli.rs::missing_user_main_rejected`'s own scenario.
//!
//! StartupError and MainSignature are each driven via a direct binary invocation (exec), NOT
//! this file's `:wat::test::spawn-peer` (fork). Measured, not assumed: the exact colon-spelling
//! trigger above, and a peer with no `:user::main`, BOTH landed as a runtime `Panic` through
//! `:wat::test::spawn-peer`'s process locus instead of their own variant —
//! `validate_user_main_signature`/the freeze-time resolve wall run at the TOP-LEVEL CLI entry
//! (`finish_in_process` / `src/host/entry.rs`), not inside that locus's fork.
//!
//! **NOT drivable — named, not faked:**
//! - `EntryFormFailure` — measured by the previous executor of this excursus (step 3b's
//!   inherited state): no producer anywhere in this tree constructs this variant.
//! - `BadReturn` — `docs/excursus/2026/09/003-the-little-wat-findings/AUDIT-error-shapes-and-backtraces.md`
//!   line 58 records it "never observed in the corpus." Independently attempted here: a
//!   `:user::main [] -> :wat::core::nil` body that returns a non-nil value ought to reach
//!   `src/process/verbs.rs`'s `Ok(Ok(other)) => … process_died_error_bad_return_value(…)` arm,
//!   but every construction tried (a heterogeneous `if`, a literal wrong-typed body) is refused
//!   at CHECK time as a `StartupError` `TypeMismatch` before the program ever runs — e.g.
//!   `(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::if true 42 nil))` dies
//!   `":wat::core::if: parameter else-branch expects :wat::core::i64; got :wat::core::nil"` at
//!   `--check`, never reaching the runtime arm at all. No test here constructs this value by
//!   hand; the gate below covers exactly the four variants that have a real producer.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Call one of this file's co-located `:g3::*-report` fns and return its `:g3::Result` fields
/// as `(cause, message)`.
fn g3_report(fn_name: &str) -> (Value, String) {
    let v = call_beside_value(file!(), fn_name)
        .unwrap_or_else(|e| panic!("{fn_name} should run and return its Result value, never raise: {e:?}"));
    let result = match &v {
        Value::Aggregate(a) => a,
        other => panic!("{fn_name}: expected a :g3::Result record; got {other:?}"),
    };
    let cause = result.fields[0].clone();
    let message = match &result.fields[1] {
        Value::String(s) => s.to_string(),
        other => panic!("{fn_name}: expected :g3::Result.message to be a String; got {other:?}"),
    };
    (cause, message)
}

/// Assert `cause` is `:wat::kernel::LociDiedError.<variant>` and that its `failure.error` is a
/// typed record (never a `String` — the double-quoting mask this excursus exists to kill).
/// Returns the error record's own `class`, for a variant-specific follow-up assertion.
fn assert_failure_error_is_a_record(fn_name: &str, cause: &Value, variant: &str) -> String {
    let ev = match cause {
        Value::Enum(ev) => ev.as_ref(),
        other => panic!("{fn_name}: expected cause to be an enum; got {other:?}"),
    };
    assert_eq!(
        ev.type_path, ":wat::kernel::LociDiedError",
        "{fn_name}: cause must be a LociDiedError"
    );
    assert_eq!(
        ev.variant_name, variant,
        // rune:lint(one-variant-separator, display) — human-facing assertion-failure prose
        // naming the enum and its expected/actual variant; not a name composed or parsed
        // anywhere.
        "{fn_name}: expected LociDiedError::{variant}, got {} — a producer that silently \
         landed on the wrong variant proves nothing about the variant it was meant to drive",
        ev.variant_name
    );
    let failure = match &ev.fields[0] {
        Value::Aggregate(a) if a.class.as_ref() == "wat::kernel::Failure" => a,
        other => panic!("{fn_name}: expected {variant}.failure to be a Failure record; got {other:?}"),
    };
    let error = match &failure.fields[0] {
        Value::Aggregate(a) => a,
        Value::String(s) => panic!(
            "{fn_name}: MASK — Failure.error is a string, not a record: {s:?}"
        ),
        other => panic!("{fn_name}: expected Failure.error to be a typed record; got {other:?}"),
    };
    error.class.to_string()
}

#[test]
fn panic_carries_a_record_error() {
    let (cause, message) = g3_report(":g3::panic-report");
    let class = assert_failure_error_is_a_record(":g3::panic-report", &cause, "Panic");
    // An AssertionPayload panic's Failure builder wraps the assertion message as a
    // `:wat::core::Fault` (item 2 of the brief: "with an AssertionPayload, use the existing
    // Failure builder").
    assert_eq!(class, "wat::core::Fault", "an assertion-failed! panic's error record class");
    assert_eq!(message, "g3-panic-sentinel", "LociDiedError/message must be the assertion text");
}

#[test]
fn runtime_error_carries_a_record_error() {
    let (cause, message) = g3_report(":g3::runtime-error-report");
    let class = assert_failure_error_is_a_record(":g3::runtime-error-report", &cause, "RuntimeError");
    assert_eq!(class, "wat::runtime::DivisionByZero", "a division-by-zero's error record class");
    assert_eq!(message, "division by zero", "LociDiedError/message must be the headline");
}

// ─── StartupError / MainSignature: driven via a direct `wat` binary invocation ─────────────

/// Field-name lookup on a parsed EDN map — mirrors
/// `tests/rete/probe_arc278_then_operand_rendered_as_source.rs`'s `field` helper.
fn edn_field<'a>(m: &'a [(wat_edn::OwnedValue, wat_edn::OwnedValue)], name: &str) -> &'a wat_edn::OwnedValue {
    m.iter()
        .find(|(k, _)| *k == wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new(name)))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("missing :{name}; got {m:?}"))
}

fn edn_tagged_map(v: &wat_edn::OwnedValue) -> (&wat_edn::Tag, &[(wat_edn::OwnedValue, wat_edn::OwnedValue)]) {
    match v {
        wat_edn::OwnedValue::Tagged(tag, body) => match body.as_ref() {
            wat_edn::OwnedValue::Map(m) => (tag, m),
            other => panic!("expected a tagged Map; got a tagged {other:?}"),
        },
        other => panic!("expected a tagged value; got {other:?}"),
    }
}

fn tag_path(tag: &wat_edn::Tag) -> String {
    format!("{}/{}", tag.namespace(), tag.name())
}

/// Run the `wat` binary directly on `program`, assert its exit code, and return the parsed
/// `(:failure error-tag, :error error-tag)` pair for the ONE death in its stderr chain.
fn run_and_parse_death(program: &str, want_exit: i32, name_hint: &str) -> (String, wat_edn::OwnedValue) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = std::env::temp_dir().join(format!(
        "g3-{name_hint}-{}-{}.wat",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, program).expect("write temp program");
    let output = std::process::Command::new(bin)
        .arg(&path)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        output.status.code(),
        Some(want_exit),
        "expected exit {want_exit}; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed = wat_edn::parse_owned(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr must be one EDN document: {e}\nstderr:\n{stderr}"));
    let died = match parsed {
        wat_edn::OwnedValue::Vector(mut xs) if xs.len() == 1 => xs.remove(0),
        other => panic!("expected a one-element death chain; got {other:?}"),
    };
    let (died_tag, died_map) = edn_tagged_map(&died);
    let died_tag_path = tag_path(died_tag);
    let (failure_tag, failure_map) = edn_tagged_map(edn_field(died_map, "failure"));
    assert_eq!(tag_path(failure_tag), "wat.kernel/Failure", "the payload must be a Failure record");
    // THE GATE: :error is a tagged RECORD, never a String (the mask this excursus kills).
    let error = edn_field(failure_map, "error").clone();
    match &error {
        wat_edn::OwnedValue::Tagged(..) => {}
        wat_edn::OwnedValue::String(s) => panic!("MASK — Failure.error is a string, not a record: {s:?}"),
        other => panic!("expected Failure.error to be a tagged record; got {other:?}"),
    }
    (died_tag_path, error)
}

#[test]
fn startup_error_carries_a_record_error() {
    let program = include_str!("probe_excursus003_g3_startup_error.wat");
    let (died_tag_path, error) = run_and_parse_death(program, 3, "startup-error");
    assert_eq!(
        died_tag_path, "wat.kernel/LociDiedError.StartupError",
        "the retired colon spelling must refuse as LociDiedError::StartupError"
    );
    let (error_tag, _) = edn_tagged_map(&error);
    assert_eq!(
        tag_path(error_tag), "wat.resolve/UnresolvedReferences",
        "a retired colon-spelling call head's error record class"
    );
}

#[test]
fn main_signature_carries_a_record_error() {
    let (died_tag_path, error) = run_and_parse_death("", 4, "main-signature");
    assert_eq!(
        died_tag_path, "wat.kernel/LociDiedError.MainSignature",
        "a program with no :user::main must refuse as LociDiedError::MainSignature"
    );
    let (error_tag, _) = edn_tagged_map(&error);
    // MainSignature is a FLAT-message producer (item 2: "SendError::Failed's reason,
    // main-signature, bad-return, and OS-level failures" all synthesize a `:wat::core::Fault`)
    // — its error is a record (a Fault), never a class specific to "no :user::main".
    assert_eq!(tag_path(error_tag), "wat.core/Fault", "a flat-message producer's error record class");
}
