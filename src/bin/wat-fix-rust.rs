//! `wat-fix-rust` — arc 255 stone 255.80 (R1, item 3: "the driver"). Runs ANY recorded
//! `wat-scripts/fixes/*.wat` codemod over wat-shaped literals embedded in Rust string
//! literals, and splices the verified edits back into the `.rs` source — general tooling
//! (not special-cased to one codemod), since cutover stones 4b, 5 and 7 all need the same
//! reach (255.79's STOP-1).
//!
//! Usage:
//!   wat-fix-rust <codemod.wat> [--dry-run] [--wat-binary <path>] (--list <file> | <path.rs>...)
//!
//! `--list <file>` names a text file with one `.rs` path per line (same convention as
//! `scripts/replay/delta.sh`'s `--list`); extra positional `<path.rs>` args may be given
//! alongside or instead of it. `--dry-run` prints each file's edit/refusal count and a
//! line-level diff without writing anything. Idempotent: a file with nothing left to
//! convert reports 0 edits and is never rewritten (so re-running after a full apply is a
//! no-op on disk, not just a no-op semantically).
//!
//! Every edit comes from the recorded codemod's own output (`wat::codemod_driver`) — this
//! binary never rewrites `.rs` text itself; it only locates, verifies, and splices.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use wat::codemod_driver::{apply_codemod_to_rust_source, LiteralOutcome};

struct Args {
    codemod: PathBuf,
    dry_run: bool,
    wat_binary: Option<PathBuf>,
    list: Option<PathBuf>,
    paths: Vec<PathBuf>,
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut it = argv.into_iter();
    let _exe = it.next();
    let codemod = it.next().ok_or_else(|| "usage: wat-fix-rust <codemod.wat> [--dry-run] [--wat-binary <path>] (--list <file> | <path.rs>...)".to_string())?;
    let mut dry_run = false;
    let mut wat_binary = None;
    let mut list = None;
    let mut paths = Vec::new();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--wat-binary" => wat_binary = Some(PathBuf::from(it.next().ok_or("--wat-binary needs a path")?)),
            "--list" => list = Some(PathBuf::from(it.next().ok_or("--list needs a path")?)),
            other => paths.push(PathBuf::from(other)),
        }
    }
    Ok(Args { codemod: PathBuf::from(codemod), dry_run, wat_binary, list, paths })
}

/// The dry-run "diff": every individual leaf-level change (`old -> new`), one per line —
/// bounded and exact, unlike a unified text diff over a file with edits scattered across
/// many lines (which degenerates to one giant first-edit..last-edit hunk).
fn edits_report(result: &wat::codemod_driver::FileApplyResult) -> String {
    let mut out = String::new();
    for outcome in &result.per_literal {
        if let LiteralOutcome::Edited { changes } = outcome {
            for (old, new) in changes {
                out.push_str("    ");
                out.push_str(old);
                out.push_str(" -> ");
                out.push_str(new);
                out.push('\n');
            }
        }
    }
    out
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().collect()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("wat-fix-rust: {e}");
            return ExitCode::FAILURE;
        }
    };

    let wat_binary = args.wat_binary.clone().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("wat")))
            .unwrap_or_else(|| PathBuf::from("target/release/wat"))
    });
    if !wat_binary.is_file() {
        eprintln!("wat-fix-rust: no wat binary at {} (cargo build --release)", wat_binary.display());
        return ExitCode::FAILURE;
    }
    if !args.codemod.is_file() {
        eprintln!("wat-fix-rust: no codemod at {}", args.codemod.display());
        return ExitCode::FAILURE;
    }

    let mut paths = args.paths.clone();
    if let Some(list_path) = &args.list {
        let content = match std::fs::read_to_string(list_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("wat-fix-rust: cannot read --list {}: {e}", list_path.display());
                return ExitCode::FAILURE;
            }
        };
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                paths.push(PathBuf::from(line));
            }
        }
    }
    if paths.is_empty() {
        eprintln!("wat-fix-rust: no paths given (use --list <file> or positional <path.rs>...)");
        return ExitCode::FAILURE;
    }

    let mut total_files_changed = 0usize;
    let mut total_edits = 0usize;
    let mut total_refused = 0usize;
    let mut total_codemod_failed = 0usize;
    let mut refused_report: Vec<String> = Vec::new();
    let mut missing = Vec::new();

    for path in &paths {
        let raw_src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                missing.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        let result = match apply_codemod_to_rust_source(&wat_binary, &args.codemod, &raw_src) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("wat-fix-rust: {} — codemod run failed: {e}", path.display());
                return ExitCode::FAILURE;
            }
        };

        let codemod_failed = result
            .per_literal
            .iter()
            .filter(|o| matches!(o, LiteralOutcome::CodemodFailed { .. }))
            .count();
        if result.total_edits > 0 || result.total_refused > 0 || codemod_failed > 0 {
            println!(
                "[wat-fix-rust]{} {}: {} edit(s), {} refused, {} codemod-failed",
                if args.dry_run { " (dry-run)" } else { "" },
                path.display(),
                result.total_edits,
                result.total_refused,
                codemod_failed
            );
        }
        if args.dry_run && result.changed {
            print!("{}", edits_report(&result));
        }
        for outcome in &result.per_literal {
            if let LiteralOutcome::Refused(refusals) = outcome {
                for r in refusals {
                    refused_report.push(format!(
                        "{}: refused {:?} old={:?} raw={:?} ({})",
                        path.display(),
                        r.old_lo..r.old_hi,
                        r.old_decoded_text,
                        r.raw_text,
                        r.reason
                    ));
                }
            }
            if let LiteralOutcome::DiffFailed(e) = outcome {
                refused_report.push(format!("{}: DIFF FAILED: {e}", path.display()));
            }
            if let LiteralOutcome::CodemodFailed { raw_lo, raw_hi, first_error } = outcome {
                refused_report.push(format!(
                    "{}:{}..{}: {first_error}",
                    path.display(),
                    raw_lo,
                    raw_hi
                ));
            }
        }

        total_edits += result.total_edits;
        total_refused += result.total_refused;
        total_codemod_failed += result
            .per_literal
            .iter()
            .filter(|o| matches!(o, LiteralOutcome::CodemodFailed { .. }))
            .count();
        if result.changed {
            total_files_changed += 1;
            if !args.dry_run {
                if let Err(e) = std::fs::write(path, &result.new_src) {
                    eprintln!("wat-fix-rust: failed to write {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    if !missing.is_empty() {
        eprintln!("wat-fix-rust: {} path(s) could not be read:", missing.len());
        for m in &missing {
            eprintln!("  {m}");
        }
        return ExitCode::FAILURE;
    }

    println!(
        "[wat-fix-rust] {} file(s) scanned, {} changed, {} edit(s){}, {} refused, {} codemod-failed",
        paths.len(),
        total_files_changed,
        total_edits,
        // Stone 255.81 — a dry run COUNTS edits; it never WRITES one. The label said
        // "applied" unconditionally, including under `--dry-run`, where `std::fs::write`
        // above is never reached — a label that states the wrong conclusion about its own
        // run (255.80's SCORE named this; fixed here).
        if args.dry_run { " found" } else { " applied" },
        total_refused,
        total_codemod_failed
    );
    if !refused_report.is_empty() {
        println!("[wat-fix-rust] refused splices:");
        for r in &refused_report {
            println!("  {r}");
        }
    }

    let _ = std::io::stdout().flush();
    ExitCode::SUCCESS
}
