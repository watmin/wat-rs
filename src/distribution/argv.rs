//! Argv handling for the wat CLI entry point: cargo-subcommand
//! stripping (`cargo-wat`'s dispatch shim) and the `[--check
//! [--check-output edn|json]] <entry.wat>` flag grammar. Split out of
//! `distribution/mod.rs` (arc 170) — the parsing concern, distinct
//! from battery composition and the fork/proxy/reap run path.

use std::process::ExitCode;

use super::check_output::CheckOutputFormat;

/// Strip cargo's injected subcommand token from argv.
///
/// `cargo X ...args...` invokes `cargo-X X ...args...`; the repeated
/// subcommand name at argv\[1\] is an artifact of cargo's dispatch
/// convention. This helper removes it so the resulting argv matches
/// what a direct invocation would produce.
///
/// No-op if argv\[1\] != `sub` (direct invocation or already stripped).
pub fn strip_cargo_subcommand(mut argv: Vec<String>, sub: &str) -> Vec<String> {
    if argv.get(1).map(String::as_str) == Some(sub) {
        argv.remove(1);
    }
    argv
}

/// The parsed result of the CLI's argv grammar: `[--check
/// [--check-output edn|json]] <entry.wat>`.
pub(super) struct ParsedArgv {
    pub(super) check_only: bool,
    pub(super) check_output_format: Option<CheckOutputFormat>,
    pub(super) entry_path: String,
}

/// Parse `argv[1..]` into the CLI's flag grammar. `prog` is argv\[0\]
/// (or `"wat"` if argv is empty), used only for the usage message.
///
/// Returns `Err(exit_code)` on any usage violation — the exact
/// `eprintln!` message + exit code the pre-split `run_with_args`
/// produced inline; behavior-preserving extraction (arc 170), not a
/// grammar change.
pub(super) fn parse(argv: &[String], prog: &str) -> Result<ParsedArgv, ExitCode> {
    let mut check_only = false;
    let mut check_output_format: Option<CheckOutputFormat> = None;
    let mut positional: Vec<&str> = Vec::new();
    let mut iter = argv.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--check" => check_only = true,
            "--check-output" => match iter.next().map(String::as_str) {
                Some("edn") => check_output_format = Some(CheckOutputFormat::Edn),
                Some("json") => check_output_format = Some(CheckOutputFormat::Json),
                Some(other) => {
                    eprintln!(
                        "wat: --check-output expects 'edn' or 'json'; got {:?}",
                        other
                    );
                    return Err(ExitCode::from(64));
                }
                None => {
                    eprintln!("wat: --check-output expects 'edn' or 'json'");
                    return Err(ExitCode::from(64));
                }
            },
            other => positional.push(other),
        }
    }
    if check_output_format.is_some() && !check_only {
        eprintln!("wat: --check-output requires --check");
        return Err(ExitCode::from(64));
    }
    if positional.len() != 1 {
        eprintln!(
            "usage: {} [--check [--check-output edn|json]] <entry.wat>",
            prog
        );
        return Err(ExitCode::from(64)); // EX_USAGE
    }
    Ok(ParsedArgv {
        check_only,
        check_output_format,
        entry_path: positional[0].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::strip_cargo_subcommand;

    #[test]
    fn strip_removes_injected_subcommand() {
        // `cargo wat foo.wat` → cargo invokes `cargo-wat wat foo.wat`
        // strip_cargo_subcommand must drop the injected "wat" at argv[1].
        let argv = vec![
            "cargo-wat".to_string(),
            "wat".to_string(),
            "foo.wat".to_string(),
        ];
        assert_eq!(
            strip_cargo_subcommand(argv, "wat"),
            vec!["cargo-wat".to_string(), "foo.wat".to_string()],
        );
    }

    #[test]
    fn strip_is_noop_when_argv1_is_not_subcommand() {
        // Direct invocation: `./cargo-wat foo.wat` — argv[1] is the
        // file path, not the subcommand name; must be left unchanged.
        let argv = vec!["wat".to_string(), "foo.wat".to_string()];
        assert_eq!(
            strip_cargo_subcommand(argv.clone(), "wat"),
            vec!["wat".to_string(), "foo.wat".to_string()],
        );
    }
}
