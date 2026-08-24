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

/// What the caller asked for. The MODE carries its own arity contract — a
/// closed set is an enum, and each variant holds exactly what that mode needs.
///
/// Arc 170: the single global `positional.len() != 1` this replaces was arc
/// 115's (`2b397cc0`), written to enforce `--check`'s grammar and applied to
/// every path — which is why the argv passthrough arc 170 built the ambient for
/// never worked. Verifying ONE file and RUNNING a program with arguments are
/// different contracts; giving each mode its own means a new mode (`--repl`,
/// which wants zero positionals) joins as a variant, not as a special case.
pub(super) enum Mode {
    /// `wat --check [--check-output edn|json] <entry.wat>` — freeze + type-check,
    /// no `:user::main`. EXACTLY one entry: checking two files at once has no
    /// defined output shape, so it stays a usage error.
    Check {
        entry_path: String,
        output_format: Option<CheckOutputFormat>,
    },
    /// `wat --repl` — the interactive read/eval/print loop. EXACTLY ZERO positionals:
    /// the REPL's program is baked into the binary, so there is no entry file to name and a
    /// trailing path would be a silent lie about what runs. This is the mode arc 170 split
    /// the global arity check FOR (see the note above) — it joins as a variant carrying its
    /// own contract, not as a special case threaded through Run's.
    Repl,
    /// `wat --mcp` — the MCP server: JSON-RPC 2.0 over stdio, one tool (`eval`). EXACTLY
    /// ZERO positionals, for `--repl`'s reason: the program is baked in, so a trailing path
    /// would be a silent lie about what runs.
    ///
    /// Semantically this IS `--repl` — read, eval against the accumulated definition set,
    /// print, loop. The only difference is the codec at each end: a JSON-RPC frame arrives
    /// carrying an EDN string, and a JSON-RPC frame leaves carrying an EDN string. The
    /// payload is EDN in both directions and is never converted to JSON — it rides inside a
    /// JSON string slot as characters.
    ///
    /// WHY THE LOOP IS DRIVEN IN RUST rather than by a `wat/mcp.wat`: JSON is not EDN, and
    /// wat's stdin/stdout are strict-EDN data channels by construction (R51, typed-Unix). A
    /// wat `println` of a JSON frame EDN-ENCODES it — the harness would receive an escaped
    /// string literal, not a JSON object (measured). That is the channel correctly refusing
    /// to carry a foreign format, not a gap to work around, so the bridge belongs where the
    /// other transport concerns already live: beside argv and the frame reader.
    Mcp,
    /// `wat <entry.wat> [args…]` — run the program. AT LEAST one positional;
    /// `positional[0]` is the entry and everything after it is the program's
    /// business, not the parser's. The trailing args are not carried in this
    /// variant because `set_argv` already receives the WHOLE unmodified argv —
    /// re-threading a slice of it here would be a second copy that could drift
    /// from the ambient the program actually reads.
    Run { entry_path: String },
    /// `echo '["a.wat" …]' | wat --grep <entry.wat>` — the grep mode
    /// (`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-mode.md`). EXACTLY one
    /// positional, same arity contract as `Run` — it has a real program that must be read and
    /// frozen. Unlike `Run`, that program declares `:user::grep` (rules), never `:user::main`;
    /// the dispatch tail (`src/distribution/mod.rs`) validates the mirror wall, not the main
    /// one, and hands the result to `:wat::grep::run` instead of invoking `:user::main`. The
    /// paths to search are NOT a positional/trailing arg — they arrive as an EDN vector on
    /// stdin, read by `:wat::grep::run` itself (`readln`, the codemods' shape); Rust never
    /// reads stdin for this mode.
    Grep { entry_path: String },
}

/// Parse `argv[1..]` into a [`Mode`]. `prog` is argv\[0\] (or `"wat"` if argv
/// is empty), used only for the usage message.
///
/// Returns `Err(exit_code)` on a usage violation.
pub(super) fn parse(argv: &[String], prog: &str) -> Result<Mode, ExitCode> {
    let mut check_only = false;
    let mut repl = false;
    let mut mcp = false;
    let mut grep = false;
    let mut output_format: Option<CheckOutputFormat> = None;
    let mut positional: Vec<&str> = Vec::new();
    let mut iter = argv.iter().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            // Flags are only recognised BEFORE the entry path. Once an entry is
            // in hand every remaining token belongs to the program — `wat
            // prog.wat --check` passes `--check` to the program, it does not
            // silently switch the cli into check mode.
            "--repl" if positional.is_empty() => repl = true,
            "--mcp" if positional.is_empty() => mcp = true,
            "--check" if positional.is_empty() => check_only = true,
            // `--grep`, not a bare `grep` subcommand — matches `--repl`/`--mcp`'s shape
            // (builder, 2026-08-24: "it must be --grep to match with --repl and --mcp").
            "--grep" if positional.is_empty() => grep = true,
            "--check-output" if positional.is_empty() => {
                match iter.next().map(String::as_str) {
                    Some("edn") => output_format = Some(CheckOutputFormat::Edn),
                    Some("json") => output_format = Some(CheckOutputFormat::Json),
                    Some(other) => {
                        eprintln!("wat: --check-output expects 'edn' or 'json'; got {:?}", other);
                        return Err(ExitCode::from(64));
                    }
                    None => {
                        eprintln!("wat: --check-output expects 'edn' or 'json'");
                        return Err(ExitCode::from(64));
                    }
                }
            }
            other => positional.push(other),
        }
    }

    if output_format.is_some() && !check_only {
        eprintln!("wat: --check-output requires --check");
        return Err(ExitCode::from(64));
    }

    let usage = |prog: &str| {
        eprintln!(
            "usage: {prog} [--check [--check-output edn|json]] <entry.wat> [args…]\n   or: {prog} --repl\n   or: {prog} --mcp\n   or: {prog} --grep <entry.wat>"
        );
        ExitCode::from(64) // EX_USAGE
    };

    // MCP mode: exactly zero positionals, same contract as --repl and for the same reason.
    // `--mcp --repl` is a usage error rather than a precedence rule — two different programs
    // were asked for, and picking one silently is the dishonest answer.
    if mcp {
        if !positional.is_empty() || check_only || output_format.is_some() || repl || grep {
            return Err(usage(prog));
        }
        return Ok(Mode::Mcp);
    }

    // REPL mode: exactly zero positionals. A path here means the caller asked for two
    // different programs at once; refusing is the honest answer, not "run one and ignore the
    // other". `--repl --check` is likewise a usage error: there is no entry file to check.
    if repl {
        if !positional.is_empty() || check_only || output_format.is_some() || grep {
            return Err(usage(prog));
        }
        return Ok(Mode::Repl);
    }

    // Grep mode: exactly one positional — the program declaring `:user::grep`, same arity
    // contract as Run. `--grep --check` is a usage error rather than a precedence rule: the
    // two verify DIFFERENT walls (`:user::main` vs `:user::grep`) and picking one silently
    // would misreport what was actually checked.
    if grep {
        if positional.len() != 1 || check_only || output_format.is_some() {
            return Err(usage(prog));
        }
        return Ok(Mode::Grep {
            entry_path: positional[0].to_string(),
        });
    }

    if check_only {
        // Check mode: exactly one.
        if positional.len() != 1 {
            return Err(usage(prog));
        }
        return Ok(Mode::Check {
            entry_path: positional[0].to_string(),
            output_format,
        });
    }

    // Run mode: at least one; the rest are the program's.
    match positional.first() {
        Some(entry) => Ok(Mode::Run { entry_path: entry.to_string() }),
        None => Err(usage(prog)),
    }
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
