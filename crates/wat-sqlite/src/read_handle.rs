//! `:rust::sqlite::ReadHandle` — read-only sqlite connection.
//!
//! Generic sibling of `:rust::sqlite::Db` (the read-write handle
//! in `lib.rs`). `Db` is what writers open; `ReadHandle` is what
//! readers open against an already-existing file. Telemetry-side
//! interrogation (arc 093) is the first consumer; any future
//! "read a sqlite file written by some other process" workflow
//! reuses this primitive without going through the
//! telemetry-specific layer.
//!
//! # Lifecycle
//!
//! - `open(path)` — opens the file with `SQLITE_OPEN_READ_ONLY`.
//!   Panics on rusqlite errors (missing file, permission denied,
//!   not-a-database) per the substrate's panic-vs-Option
//!   contract: filesystem / open errors are programmer-visible
//!   inputs, not data-flow returns.
//! - Drop closes the connection. No explicit close primitive
//!   shipping in slice 1; let*-bind the handle inside a
//!   `:user::main` body and the lexical end IS the close.
//!
//! # WAL coordination
//!
//! The writer side (`Db::open` + `Db::pragma "journal_mode" "WAL"`)
//! sets WAL on its connection; WAL persists across open/close
//! cycles, so readers inherit transparently. Long-running readers
//! don't block writers (and vice-versa) on a WAL-mode database —
//! that's the whole point of the reader-vs-writer separation
//! arc 093 builds on.
//!
//! # Why a distinct type from `Db`
//!
//! Two reasons:
//!
//! - **Capability honesty.** A `Db` carries write-side methods
//!   (`execute`, `execute_ddl`, `pragma`, `begin`, `commit`). A
//!   reader must not be able to use them. The type system
//!   enforces this — `ReadHandle` exposes only read primitives
//!   (slice 1c adds the cursor / step path; this slice ships
//!   just `open`).
//! - **Open-flag commitment.** `Db::open` opens read-write
//!   without flags; `ReadHandle::open` passes
//!   `SQLITE_OPEN_READ_ONLY`. The flag is set at open time and
//!   can't be revoked, so the type IS the proof of read-only
//!   intent.

use rusqlite::{Connection, OpenFlags};
use wat_macros::wat_dispatch;

/// Public registrar wrapping the macro-generated module so
/// `lib.rs::register` can wire ReadHandle in alongside the
/// `Db` shim. The generated `__wat_dispatch_ReadHandle` module
/// is private to this file by default; this function is the
/// documented public surface.
pub(crate) fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    __wat_dispatch_ReadHandle::register(builder);
}

/// `:rust::sqlite::ReadHandle` — read-only sqlite connection.
/// Thread-owned: open in the worker that will use it
/// (CIRCUIT.md rule 1, same discipline as `:rust::sqlite::Db`).
///
/// Stores both the open `Connection` (the validation that the
/// path is a real, openable sqlite file) AND the path itself.
/// Consumers that want to spawn cursors inside a wat
/// `spawn-producer` lambda (where the thread-owned ReadHandle
/// can't follow the lambda capture) call `path()` to get the
/// path back, capture that string, and re-open inside the
/// producer thread.
pub struct ReadHandle {
    /// `pub` (not `pub(crate)`) for the same reason `Db.conn` is
    /// `pub`: downstream cursor code in `wat-telemetry-sqlite`
    /// (slice 1c) may want direct `prepare` access in the future.
    /// Consumers outside the cursor layer should treat this as
    /// substrate-internal and reach for typed methods instead.
    #[allow(dead_code)]
    pub conn: Connection,
    /// The path passed to `open`. Stashed so consumers can hand
    /// the string off to a different thread (capture into a
    /// `spawn-producer` lambda) and re-open a fresh ReadHandle
    /// there. Connection itself doesn't expose `path()` cleanly
    /// for our flow, so we keep it ourselves.
    path: String,
}

#[wat_dispatch(
    path = ":rust::sqlite::ReadHandle",
    scope = "thread_owned"
)]
impl ReadHandle {
    /// `:rust::sqlite::ReadHandle::open path` — open an existing
    /// sqlite file at `path` in read-only mode. Panics on
    /// rusqlite errors (missing file, permission denied,
    /// not-a-database, etc.) — the substrate refuses to silently
    /// produce an Option for filesystem-shape failures per the
    /// panic-vs-Option contract.
    pub fn open(path: String) -> Self {
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap_or_else(|e| {
            panic!(":rust::sqlite::ReadHandle::open: cannot open {path}: {e}")
        });
        Self { conn, path }
    }

    /// `:rust::sqlite::ReadHandle::path handle` — borrow the path
    /// the handle was opened with. Used by downstream readers
    /// that need to spawn a producer in a different thread and
    /// re-open a fresh ReadHandle there (the thread_owned
    /// discipline forbids transferring this struct itself across
    /// the spawn boundary; transferring the path is fine).
    pub fn path(&self) -> String {
        self.path.clone()
    }
}
