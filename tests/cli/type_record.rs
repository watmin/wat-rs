//! STONE 255.68 — `wat --check`'s type record, under `WAT_CHECK_TYPES=1`.
//!
//! Written fresh on `main`, ported from `the-little-wat`'s `bbfac2ee8` (read there, never
//! merged/checked-out/cherry-picked): same names (`crate::check::type_record`), same env var
//! (`WAT_CHECK_TYPES`), same output shape (`TYPE`/`UNRESOLVED`/`TYPES` lines on stdout). Unset,
//! nothing new runs — `--check` is byte-for-byte what it was before this stone
//! (`type_record_unset_output_is_unchanged`). Set, `wat --check` also prints, one line per
//! recorded node, the type the checker's `infer` gave it, through that inference root's FINAL
//! substitution (`type_record_records_element_type_from_call_site`).
//!
//! Fixture lives beside this file: `wat_cli__check_types.wat` declares `:user::takes-vec` with a
//! `(wat.type/PersistentVector :- [wat.type/i64])`-typed parameter, and `:user::main` calls it
//! with a bare, untyped `(:wat::core::PersistentVector)` — the argument's own text names no
//! element type. The recorded type at that node's span shows the checker inferring `i64` from
//! the call site, not from the text — the instrument STONE 255.68 measures the untyped
//! constructor corpus with.

use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_wat")
}

// Relative to the crate root, which is the cwd `cargo test` guarantees
// (`load::loader::span_display_path`'s doc comment) — the checker's recorded spans render this
// same relative spelling, so the fixture is addressed identically as an argv path and as a
// stdout needle.
fn fixture() -> &'static str {
    "tests/cli/wat_cli__check_types.wat"
}

#[test]
fn type_record_unset_output_is_unchanged() {
    // WAT_CHECK_TYPES absent — the recording path must not run at all. A well-typed program
    // still passes `--check` with rc 0 and empty stdout/stderr, exactly as it did before this
    // stone (mirrors `wat_cli::check_mode_exits_zero_on_good_program`, plus the stderr half that
    // test doesn't check, and this specific fixture).
    let output = Command::new(bin())
        .arg("--check")
        .arg(fixture())
        .env_remove("WAT_CHECK_TYPES")
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.stdout.is_empty(),
        "WAT_CHECK_TYPES unset must not print anything new; got stdout: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "a well-typed program must not print to stderr either way; got: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn type_record_records_element_type_from_call_site() {
    // WAT_CHECK_TYPES=1 — the untyped `(:wat::core::PersistentVector)` argument at line 11
    // col 23 (the fixture's own comment explains the shape) must record the element type the
    // checker inferred from the declared parameter type at the call site: i64.
    let output = Command::new(bin())
        .arg("--check")
        .arg(fixture())
        .env("WAT_CHECK_TYPES", "1")
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);

    let fixture_path = fixture();
    let needle = format!("{fixture_path}\t11\t23\t");
    let line = stdout
        .lines()
        .find(|l| l.contains(&needle))
        .unwrap_or_else(|| panic!("no recorded line at {fixture_path}:11:23; full stdout:\n{stdout}"));
    let fields: Vec<&str> = line.split('\t').collect();
    assert_eq!(fields.len(), 6, "TYPE/UNRESOLVED line must be tag/file/line/col/type/wide; got: {line}");

    let trailer = stdout
        .lines()
        .find(|l| l.starts_with("TYPES recorded"))
        .unwrap_or_else(|| panic!("no TYPES trailer line; full stdout:\n{stdout}"));
    let check_errors = trailer
        .rsplit(' ')
        .next()
        .unwrap_or_else(|| panic!("trailer line has no trailing field: {trailer}"));

    // The recorded node (tag/file/line/col/ty/wide) plus the recheck's own error count, as ONE
    // structured value compared by DATA equality against a captured golden — never a substring
    // check on the raw TAB line, which is what would pass on a reordered field, a wrong tag
    // (UNRESOLVED where TYPE was required), or a different element type entirely. The golden is
    // the proof: `:tag "TYPE"` (fully resolved — unified against the declared param type at the
    // call site, not left a unification variable), `:ty`/`:wide` both the i64-element vector (no
    // enclosing enum to widen to, so the two fields are identical), and `:check-errors 0` (the
    // second `check_program` pass over the frozen world found nothing wrong).
    // Built with `push(char)` braces rather than a `"{...}"` literal — a literal opening on `{`
    // is EDN-esque text inlined in the .rs source (`no_inlined_edn`'s own rubric), even though
    // this one is a format! scaffold, not a golden; a char literal carries no such reading.
    let mut recorded = String::new();
    recorded.push('{');
    recorded.push_str(&format!(
        ":tag {:?} :file {:?} :line {} :col {} :ty {:?} :wide {:?} :check-errors {}",
        fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], check_errors
    ));
    recorded.push('}');
    wat::assert_edn_matches_file!(recorded, "wat_cli__check_types__recorded_type.edn");
}
