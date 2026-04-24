//! `wat::test_runner` — library-callable entry for running `.wat`
//! test files with external-crate composition. Arc 015 slice 1.
//!
//! Closes the symmetry gap arc 013 left: `wat::compose_and_run` +
//! `wat::main!` let a consumer binary invoke `:user::main` with
//! external wat crates composed in; this module does the same for
//! the test runner that `wat test <path>` already ships.
//!
//! # Why this is a library module, not just a CLI subcommand
//!
//! The `wat` CLI binary deliberately does not link external wat
//! crates — that's the proof stance arc 013 holds (wat-rs root has
//! zero dependency on wat-lru). A consumer crate that wants to run
//! `.wat` tests referencing external symbols (`:wat::lru::*` etc. —
//! first-party workspace-member crates under arc 036's namespace
//! rule) cannot route through the CLI.
//!
//! This module exposes the same test-discovery + freeze + run logic
//! as a callable function that accepts `dep_sources` + `dep_registrars`.
//! The `wat::test_suite!` proc-macro (slice 2) wraps it in a `#[test]
//! fn` so `cargo test` picks up consumer-authored wat test suites
//! with zero ceremony.
//!
//! # Typical shape
//!
//! Direct library use:
//!
//! ```text
//! use std::path::Path;
//! let summary = wat::test_runner::run_tests_from_dir(
//!     Path::new("wat-tests"),
//!     &[wat_lru::wat_sources()],
//!     &[wat_lru::register],
//! );
//! assert_eq!(summary.failed, 0);
//! ```
//!
//! Via the macro (slice 2):
//!
//! ```text
//! wat::test_suite! {
//!     path: "wat-tests",
//!     deps: [wat_lru],
//! }
//! ```
//!
//! # Install semantics
//!
//! `rust_deps::install()` is a OnceLock — first-call-wins. A test
//! binary running `run_tests_from_dir` once against one dep set is
//! the intended shape. Callers running multiple `run_tests_from_dir`
//! invocations with *different* dep sets in one process will hit
//! the first-call-wins limitation documented in `compose_and_run`'s
//! docstring. Match each dep set to its own test binary (its own
//! `tests/*.rs` file) and Cargo handles the rest.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use crate::compose::DepRegistrar;
use crate::freeze::{startup_from_source, FrozenWorld};
use crate::load::{FsLoader, SourceLoader};
use crate::runtime::{apply_function, Function, Value};
use crate::rust_deps::{self, RustDepsBuilder};
use crate::source::{self, WatSource};
use crate::types::TypeExpr;

/// Aggregated result of running every `.wat` file under a path.
///
/// Returned by [`run_tests_from_dir`]; consumers that use the
/// library directly (not the [`crate::test_suite!`] macro) can
/// inspect fields and decide how to surface the outcome. The macro
/// route wraps this in [`run_and_assert`] which panics on any
/// failure.
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    /// One entry per failed test, pre-formatted for display.
    /// Includes the file name + test name + failure message.
    pub failure_summaries: Vec<String>,
    pub elapsed_ms: u128,
    /// True when the path resolved but contained no discoverable
    /// `test-` functions. Distinct from `total == 0` because the
    /// CLI reports this as exit-code 64 (no tests) rather than
    /// exit-code 1 (failures).
    pub no_tests_discovered: bool,
    /// Count of `.wat` files the directory walk turned up. Zero
    /// means the caller pointed at an empty directory (meaningful
    /// distinct diagnostic from "has files but no `test-` defines").
    /// Always `1` for a single-file input that exists.
    pub file_count: usize,
}

/// Run every `.wat` file under `path`, discover `test-` functions,
/// invoke each, aggregate results. Uses the full startup pipeline
/// with `dep_sources` + `dep_registrars` threaded through so
/// external wat crates' symbols are reachable from the test files.
///
/// # Discovery convention
///
/// A top-level `:wat::core::define` is a test iff:
/// 1. The path's final `::`-segment starts with `test-`.
/// 2. `param_types` is empty (zero-arg).
/// 3. `ret_type` is the plain path `:wat::kernel::RunResult`.
///
/// Tests within one file run in randomized order (Fisher-Yates,
/// nanos-seeded xorshift) to surface accidental inter-dependencies.
/// Tests across files stay grouped per file — each file's
/// FrozenWorld is distinct; re-freezing across files isn't worth
/// the cost.
///
/// # Path handling
///
/// `path` may be a single `.wat` file or a directory. Directory
/// traversal is recursive and deterministic (sorted by filesystem
/// path) — subdirectories like `wat-tests/std/*.wat` get picked up
/// by one invocation on the parent.
///
/// # Errors as data
///
/// Filesystem failures (missing path, unreadable file) and wat
/// startup failures (parse / check / resolve) populate
/// `failure_summaries` + increment `failed`. No panic, no
/// propagated Err. Callers that want panic-on-any-failure use
/// [`run_and_assert`].
pub fn run_tests_from_dir(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
) -> TestSummary {
    run_tests_from_dir_with_loader(
        path,
        dep_sources,
        dep_registrars,
        Arc::new(FsLoader),
    )
}

/// Loader-parametric sibling of [`run_tests_from_dir`]. Same
/// contract; the caller supplies the [`SourceLoader`] used to
/// resolve `(:wat::load-file! ...)` from inside each test file's
/// freeze. The `wat::test_suite! { ..., loader: "path" }` form
/// (arc 017) expands to this function with a `ScopedLoader` rooted
/// at the given path. Passing `Arc::new(FsLoader)` reproduces the
/// default [`run_tests_from_dir`] behavior.
pub fn run_tests_from_dir_with_loader(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
    loader: Arc<dyn SourceLoader>,
) -> TestSummary {
    let mut summary = TestSummary::default();
    let run_start = Instant::now();

    // Install the wat panic hook — arc 016 slice 3. Writes Rust-
    // styled failure output to stderr when an assertion fires.
    // Must run BEFORE any wat code; idempotent if already installed.
    crate::panic_hook::install();

    // Install BOTH halves of the external-crate contract globally
    // — symmetric OnceLocks, first-call-wins. After install, every
    // test file's freeze and every nested `run-sandboxed-ast` /
    // fork child transparently sees dep wat sources + Rust shims.
    let mut builder = RustDepsBuilder::with_wat_rs_defaults();
    for registrar in dep_registrars {
        registrar(&mut builder);
    }
    let _ = rust_deps::install(builder.build());
    let _ = source::install_dep_sources(dep_sources.to_vec());

    // 1. Resolve input — file or directory.
    let files = match discover_wat_files(path) {
        Ok(fs) if fs.is_empty() => {
            summary.no_tests_discovered = true;
            summary.file_count = 0;
            return summary;
        }
        Ok(fs) => {
            summary.file_count = fs.len();
            fs
        }
        Err(e) => {
            summary
                .failure_summaries
                .push(format!("test-runner: read {}: {}", path.display(), e));
            summary.failed += 1;
            summary.elapsed_ms = run_start.elapsed().as_millis();
            return summary;
        }
    };

    // 2. Freeze each file against the composed dep_sources. A
    //    per-file startup failure surfaces as a single failure
    //    entry; the runner keeps going so the user sees all
    //    problems in one pass, cargo-test-style.
    //
    // **Entry vs. library** (arc 017). A `.wat` file in the test
    // directory is an **entry** iff it commits startup config (a
    // top-level `(:wat::config::set-*!)` form). Entries are frozen
    // here and scanned for `test-*` defines. Files without config
    // setters are **libraries** — intended to be `(:wat::load-file!
    // "...")`'d from entry files — and
    // test_runner silently skips them at freeze time. This mirrors
    // the binary-vs-library distinction `wat::main!` already uses
    // (the entry commits config, loaded files must not).
    let mut per_file: Vec<(PathBuf, FrozenWorld, Vec<String>)> = Vec::new();
    for file in &files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                summary.failure_summaries.push(format!(
                    "test-runner: read {}: {}",
                    file.display(),
                    e
                ));
                summary.failed += 1;
                continue;
            }
        };
        // Skip library files — defined as files without a top-level
        // config setter. A parse error here is left to the freeze
        // below so the user sees the real error with full context.
        if !source_has_config_setter(&src) {
            continue;
        }
        let canonical = std::fs::canonicalize(file)
            .ok()
            .map(|p| p.display().to_string());
        let frozen = match startup_from_source(
            &src,
            canonical.as_deref(),
            loader.clone(),
        ) {
            Ok(f) => f,
            Err(e) => {
                summary.failure_summaries.push(format!(
                    "test-runner: {}: startup: {}",
                    file.display(),
                    e
                ));
                summary.failed += 1;
                continue;
            }
        };
        let discovered = discover_tests(&frozen);
        summary.total += discovered.len();
        per_file.push((file.clone(), frozen, discovered));
    }

    if summary.total == 0 && summary.failed == 0 {
        summary.no_tests_discovered = true;
        summary.elapsed_ms = run_start.elapsed().as_millis();
        return summary;
    }

    println!("running {} tests", summary.total);

    // 3. Invoke each test. Randomize order per-file; tests across
    //    files stay grouped by file. Cargo-test-style per-test
    //    output (printed to stdout so both CLI and macro paths see
    //    it — the macro path's Cargo `#[test] fn` captures and
    //    surfaces on failure, or always with --nocapture).
    let mut rng = Xorshift64::seeded_from_clock();
    for (file, frozen, mut names) in per_file {
        shuffle(&mut names, &mut rng);
        let short_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.wat");
        for name in &names {
            let func = frozen
                .symbols()
                .get(name)
                .expect("discovered name must exist")
                .clone();
            let label = format!("test {} :: {}", short_name, strip_leading_colon(name));
            print!("{} ", label);
            let start = Instant::now();
            let invoke = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                apply_function(func, Vec::new(), frozen.symbols(), crate::rust_caller_span!())
            }));
            let elapsed_ms = start.elapsed().as_millis();
            match invoke {
                Ok(Ok(value)) => match extract_failure(&value) {
                    None => {
                        println!("... ok ({}ms)", elapsed_ms);
                        summary.passed += 1;
                    }
                    Some(fail) => {
                        println!("... FAILED ({}ms)", elapsed_ms);
                        summary.failure_summaries.push(format!("{}\n{}", label, fail));
                        summary.failed += 1;
                    }
                },
                Ok(Err(err)) => {
                    println!("... FAILED ({}ms)", elapsed_ms);
                    summary.failure_summaries.push(format!(
                        "{}\n  runtime: {}",
                        label, err
                    ));
                    summary.failed += 1;
                }
                Err(_) => {
                    println!("... FAILED ({}ms)", elapsed_ms);
                    summary.failure_summaries.push(format!(
                        "{}\n  panic escaped test body (assertion panics should be caught inside)",
                        label
                    ));
                    summary.failed += 1;
                }
            }
        }
    }

    summary.elapsed_ms = run_start.elapsed().as_millis();

    // Final summary — cargo-test-style. Emit on every completed
    // run so the CLI and macro paths produce consistent output.
    println!();
    if !summary.failure_summaries.is_empty() {
        println!("failures:");
        println!();
        for fail in &summary.failure_summaries {
            println!("{}", fail);
            println!();
        }
    }
    let overall = if summary.failed == 0 { "ok" } else { "FAILED" };
    println!(
        "test result: {}. {} passed; {} failed; finished in {}ms",
        overall, summary.passed, summary.failed, summary.elapsed_ms
    );

    summary
}

/// Run tests via [`run_tests_from_dir`]; panic with the full
/// failure summary joined if any test failed or no tests were
/// discovered under the path. This is what
/// [`crate::test_suite!`] expands to — Cargo's `#[test] fn`
/// machinery captures the panic and surfaces it as a test
/// failure, so consumer-authored suites get cargo-test-style
/// output with zero boilerplate.
pub fn run_and_assert(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
) {
    run_and_assert_with_loader(
        path,
        dep_sources,
        dep_registrars,
        Arc::new(FsLoader),
    )
}

/// Loader-parametric sibling of [`run_and_assert`]. What
/// `wat::test_suite! { ..., loader: "path" }` expands to (arc 017).
/// Panics with the joined failure summary if any test fails; the
/// caller-supplied loader threads through every test file's freeze.
pub fn run_and_assert_with_loader(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
    loader: Arc<dyn SourceLoader>,
) {
    let summary =
        run_tests_from_dir_with_loader(path, dep_sources, dep_registrars, loader);
    if summary.no_tests_discovered {
        panic!(
            "wat test suite: no test- prefixed functions found under {}",
            path.display()
        );
    }
    if summary.failed > 0 {
        let mut msg = format!(
            "wat test suite: {} passed, {} failed ({}ms)\n",
            summary.passed, summary.failed, summary.elapsed_ms
        );
        for fail in &summary.failure_summaries {
            msg.push('\n');
            msg.push_str(fail);
            msg.push('\n');
        }
        panic!("{}", msg);
    }
}

// ─── Discovery helpers (lifted from src/bin/wat.rs) ─────────────────

/// Resolve a path into a list of `.wat` files.
/// - File → `vec![path]`.
/// - Directory → every `.wat` under it recursively, sorted.
fn discover_wat_files(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let meta = std::fs::metadata(path)?;
    if meta.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if meta.is_dir() {
        let mut out: Vec<PathBuf> = Vec::new();
        collect_wat_files_recursive(path, &mut out)?;
        out.sort();
        return Ok(out);
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "path is neither file nor directory",
    ))
}

/// Arc 017 — a `.wat` file is an ENTRY (commits config + hosts tests)
/// iff it has at least one top-level `(:wat::config::set-*!)` form
/// OR any top-level `(:wat::test::*)` form (deftest, make-deftest,
/// etc.). Files with only defines / loads are LIBRARIES and get
/// skipped at freeze time.
///
/// Arc 037 (2026-04-24): loosened the setter-only signal. Under the
/// arc 037 contract, set-dims! is a no-op and set-capacity-mode!
/// defaults to :error — entry-file preambles are often empty. A
/// file's intent to host tests is better signaled by the presence
/// of `:wat::test::*` forms.
///
/// Implementation: parse the file's top-level forms with the lexer +
/// parser and check each form's head keyword. Parse errors are NOT
/// reported here — the caller proceeds to freeze, where the error
/// surfaces with full diagnostic context. Treating parse-failed files
/// as "not an entry" (and skipping) would mask real errors.
fn source_has_config_setter(src: &str) -> bool {
    let forms = match crate::parser::parse_all(src) {
        Ok(f) => f,
        // Parse error — let the caller's freeze path report it.
        // Return `true` so we proceed to freeze.
        Err(_) => return true,
    };
    forms.iter().any(|form| {
        if let crate::ast::WatAST::List(items, _) = form {
            if let Some(crate::ast::WatAST::Keyword(k, _)) = items.first() {
                return (k.starts_with(":wat::config::set-") && k.ends_with('!'))
                    || k.starts_with(":wat::test::");
            }
        }
        false
    })
}

fn collect_wat_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_wat_files_recursive(&path, out)?;
        } else if file_type.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("wat")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn discover_tests(frozen: &FrozenWorld) -> Vec<String> {
    let mut out = Vec::new();
    for (name, func) in &frozen.symbols().functions {
        if is_test_function(name, func) {
            out.push(name.clone());
        }
    }
    out.sort();
    out
}

fn is_test_function(name: &str, func: &Arc<Function>) -> bool {
    if !func.param_types.is_empty() {
        return false;
    }
    match &func.ret_type {
        TypeExpr::Path(p) if p == ":wat::kernel::RunResult" => {}
        _ => return false,
    }
    let bare = strip_leading_colon(name);
    let last = bare.rsplit("::").next().unwrap_or("");
    last.starts_with("test-")
}

fn strip_leading_colon(s: &str) -> &str {
    s.strip_prefix(':').unwrap_or(s)
}

fn extract_failure(v: &Value) -> Option<String> {
    let sv = match v {
        Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        _ => return Some("  test did not return :wat::kernel::RunResult".into()),
    };
    let failure_field = sv.fields.get(2)?;
    let failure_opt = match failure_field {
        Value::Option(opt) => opt,
        _ => return Some("  malformed RunResult.failure slot".into()),
    };
    let failure = match &**failure_opt {
        Some(v) => v,
        None => return None,
    };
    let fv = match failure {
        Value::Struct(s) if s.type_name == ":wat::kernel::Failure" => s,
        _ => return Some("  failure slot is not :wat::kernel::Failure".into()),
    };
    let message = match fv.fields.first() {
        Some(Value::String(s)) => (**s).clone(),
        _ => "<missing message>".to_string(),
    };
    let actual = fv.fields.get(3).and_then(option_string_field);
    let expected = fv.fields.get(4).and_then(option_string_field);
    let mut out = format!("  failure: {}", message);
    if let Some(a) = actual {
        out.push_str(&format!("\n  actual:   {}", a));
    }
    if let Some(e) = expected {
        out.push_str(&format!("\n  expected: {}", e));
    }
    Some(out)
}

fn option_string_field(v: &Value) -> Option<String> {
    match v {
        Value::Option(opt) => match &**opt {
            Some(Value::String(s)) => Some((**s).clone()),
            _ => None,
        },
        _ => None,
    }
}

// ─── Xorshift64 — tiny deterministic shuffle source ─────────────────────
//
// Not cryptographic. Seeds from clock nanos so order varies across runs
// without pulling in the `rand` crate as a dependency.

struct Xorshift64(u64);

impl Xorshift64 {
    fn seeded_from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xdead_beef_1234_5678);
        Xorshift64(if nanos == 0 { 1 } else { nanos })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn shuffle<T>(items: &mut [T], rng: &mut Xorshift64) {
    if items.len() < 2 {
        return;
    }
    for i in (1..items.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        items.swap(i, j);
    }
}
