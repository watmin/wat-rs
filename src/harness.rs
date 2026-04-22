//! `wat::Harness` — a thin ergonomic wrapper for Rust programs that
//! embed wat as a sub-language (arc 007 slice 5).
//!
//! Everything here is already possible with the raw public API — see
//! the ~20-line boilerplate that every in-crate integration test used
//! to hand-roll. Harness just captures the pattern. `from_source`
//! freezes; `run` builds StringIo stdio, invokes `:user::main`, and
//! returns captured stdout/stderr.
//!
//! # What Harness is NOT
//!
//! - A sandbox. No panic isolation, no scope enforcement beyond the
//!   caller-supplied loader. Callers that want panic containment use
//!   `:wat::kernel::run-sandboxed` from within their wat program.
//! - A test runner. That's `wat test <path>` (arc 007 slice 4).
//! - A `:user::main`-signature shim. Harness enforces the same
//!   three-IO contract the CLI enforces; programs with a different
//!   `:user::main` signature fail at `from_source`.
//!
//! # Typical shape
//!
//! ```no_run
//! use wat::harness::Harness;
//!
//! let h = Harness::from_source(r#"
//!     (:wat::config::set-dims! 1024)
//!     (:wat::config::set-capacity-mode! :error)
//!     (:wat::core::define (:user::main
//!                          (stdin  :wat::io::IOReader)
//!                          (stdout :wat::io::IOWriter)
//!                          (stderr :wat::io::IOWriter)
//!                          -> :())
//!       (:wat::io::IOWriter/println stdout "hello"))
//! "#)?;
//! let out = h.run(&[])?;
//! assert_eq!(out.stdout, vec!["hello".to_string()]);
//! # Ok::<(), wat::harness::HarnessError>(())
//! ```

use crate::freeze::{
    invoke_user_main, startup_from_source, startup_from_source_with_deps,
    validate_user_main_signature, FrozenWorld, StartupError,
};
use crate::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use crate::load::{InMemoryLoader, SourceLoader};
use crate::runtime::{RuntimeError, Value};
use crate::stdlib::StdlibFile;
use std::sync::Arc;

/// A frozen wat program ready to invoke. Clone is NOT derived: the
/// underlying `FrozenWorld` is intentionally share-only — hold the
/// Harness across invocations rather than cloning.
#[derive(Debug)]
pub struct Harness {
    world: FrozenWorld,
}

/// Captured output from one `:user::main` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

/// Errors surfaced across the Harness surface — one variant per pass,
/// so callers that just want a `?`-able `Result` see a single error
/// type.
#[derive(Debug)]
pub enum HarnessError {
    Startup(StartupError),
    MainSignature(String),
    Runtime(RuntimeError),
    StdioSnapshot(String),
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HarnessError::Startup(e) => write!(f, "startup: {}", e),
            HarnessError::MainSignature(m) => write!(f, ":user::main: {}", m),
            HarnessError::Runtime(e) => write!(f, "runtime: {}", e),
            HarnessError::StdioSnapshot(s) => write!(f, "stdio snapshot: {}", s),
        }
    }
}

impl std::error::Error for HarnessError {}

impl Harness {
    /// Freeze wat source with an `InMemoryLoader` — zero filesystem
    /// access. Use [`Self::from_source_with_loader`] when the program
    /// needs `load!` to reach disk.
    pub fn from_source(src: &str) -> Result<Self, HarnessError> {
        Self::from_source_with_loader(src, Arc::new(InMemoryLoader::new()))
    }

    /// Freeze wat source with a caller-supplied loader. Use `FsLoader`
    /// for production, `ScopedLoader::new(path)` for a capability-
    /// bounded subset, or a custom impl for programmatic fixtures.
    pub fn from_source_with_loader(
        src: &str,
        loader: Arc<dyn SourceLoader>,
    ) -> Result<Self, HarnessError> {
        let world = startup_from_source(src, None, loader).map_err(HarnessError::Startup)?;
        validate_user_main_signature(&world).map_err(HarnessError::MainSignature)?;
        Ok(Self { world })
    }

    /// Freeze wat source composed with external dep sources. Arc 013
    /// slice 2.
    ///
    /// `dep_sources` is a slice of `&[StdlibFile]` — one inner slice
    /// per dep crate. `wat::main!` (slice 3) expands to a call
    /// through this entry, passing `&[wat_lru::stdlib_sources(), …]`.
    /// Uses `InMemoryLoader` (no filesystem); callers that need a
    /// filesystem-capable loader use
    /// [`Self::from_source_with_deps_and_loader`].
    ///
    /// Dep forms join the user tier at the reserved-prefix gate —
    /// deps must declare under `:user::*` (typically
    /// `:user::wat::std::<crate>::*` per the arc 013 convention) or
    /// registration fails loud.
    pub fn from_source_with_deps(
        src: &str,
        dep_sources: &[&[StdlibFile]],
    ) -> Result<Self, HarnessError> {
        Self::from_source_with_deps_and_loader(
            src,
            dep_sources,
            Arc::new(InMemoryLoader::new()),
        )
    }

    /// Full-form entry with both external dep sources and a
    /// caller-supplied loader. The other three `from_source*`
    /// variants on Harness are sugar over this one.
    pub fn from_source_with_deps_and_loader(
        src: &str,
        dep_sources: &[&[StdlibFile]],
        loader: Arc<dyn SourceLoader>,
    ) -> Result<Self, HarnessError> {
        let world = startup_from_source_with_deps(src, dep_sources, None, loader)
            .map_err(HarnessError::Startup)?;
        validate_user_main_signature(&world).map_err(HarnessError::MainSignature)?;
        Ok(Self { world })
    }

    /// Borrow the frozen world for callers that want direct access
    /// (e.g., to invoke non-`:user::main` defined functions by name).
    pub fn world(&self) -> &FrozenWorld {
        &self.world
    }

    /// Invoke `:user::main` with pre-seeded stdin lines and return
    /// captured stdout + stderr. Lines are joined with `\n` between
    /// (no trailing newline added); the StringIoReader delivers the
    /// joined buffer to the wat program's IOReader.
    ///
    /// # Panic semantics
    ///
    /// Panics inside the wat program (assertion failures, explicit
    /// `panic!`-able primitives, etc.) are NOT caught here. If panic
    /// containment matters, have the wat program invoke
    /// `:wat::kernel::run-sandboxed` on itself and return the
    /// structured `RunResult`.
    pub fn run(&self, stdin: &[&str]) -> Result<Outcome, HarnessError> {
        let stdin_data = stdin.join("\n");
        let reader: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(stdin_data));

        let stdout_writer = Arc::new(StringIoWriter::new());
        let stderr_writer = Arc::new(StringIoWriter::new());
        let stdout_dyn: Arc<dyn WatWriter> = stdout_writer.clone();
        let stderr_dyn: Arc<dyn WatWriter> = stderr_writer.clone();

        let args = vec![
            Value::io__IOReader(reader),
            Value::io__IOWriter(stdout_dyn),
            Value::io__IOWriter(stderr_dyn),
        ];

        invoke_user_main(&self.world, args).map_err(HarnessError::Runtime)?;

        let stdout = snapshot_lines(&stdout_writer)?;
        let stderr = snapshot_lines(&stderr_writer)?;
        Ok(Outcome { stdout, stderr })
    }
}

fn snapshot_lines(writer: &Arc<StringIoWriter>) -> Result<Vec<String>, HarnessError> {
    let bytes = writer
        .snapshot_bytes()
        .map_err(|e| HarnessError::StdioSnapshot(format!("{:?}", e)))?;
    let s = String::from_utf8(bytes)
        .map_err(|e| HarnessError::StdioSnapshot(format!("utf-8: {}", e)))?;
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    Ok(lines)
}
