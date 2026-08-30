//! `:wat::io::TempFile/*`, `:wat::io::TempDir/*`, `:wat::io::read-file`,
//! `:wat::io::list-dir` intrinsics — arc 255 home #12 (255.1c-io-fs), the
//! third and last home in the `:wat::io::` family (`io/mod.rs`'s family
//! claim). Six verbs: the two RAII temp handles (`TempFile/new`,
//! `TempFile/path`, `TempDir/new`, `TempDir/path` — arc 093's
//! auto-deleting wrappers around Rust's `tempfile` crate; Drop unlinks the
//! file/dir when the wat value's Arc-count reaches zero) plus the two
//! filesystem one-shots (`read-file`, `list-dir` — open nothing the caller
//! keeps).
//!
//! This strike closes the family: after it, no `:wat::io::` literal-match
//! arm remains in `runtime.rs` — `grep '":wat::io::[^"]*" *=>' src/runtime.rs`
//! returns 0. `:wat::stdlib::sources`, which sat directly below these six
//! arms in `runtime.rs`, belongs to a different family (arc 275 Stone
//! 275.1's baked stdlib load order) and stays a literal-match arm — it was
//! never part of this family and is never carved here.
//!
//! Every one of the six delegates to a `crate::io::eval_io_*` fn that
//! already existed as a literal-match arm in `runtime.rs` — see `io/mod.rs`
//! for the family-wide "bodies do not live here" claim this home is an
//! instance of.
//!
//! ## Argument order — uniform, unlike `writer.rs`
//!
//! All six delegates take the SAME argument order — `(args, list_span,
//! env, sym)` — no split by verb, unlike `writer.rs`'s mixed argument
//! orders across its thirteen.
//!
//! ## Every `@Category` read at the body, not derived from the name
//!
//! The six straddle three clusters:
//!
//! - **`TempFile/new`/`TempDir/new`** (`io.rs:1703`/`1732`, delegating to
//!   `WatTempFile::new`/`WatTempDir::new` at `io.rs:1637`/`1668`, whose
//!   `NamedTempFile::new()`/`TempDir::new()` calls at `io.rs:1638`/`1669`
//!   are the actual acquisition) — mint a fresh, real file/directory on
//!   disk that Drop later unlinks (struct docs, `io.rs:1628`–`1629`/
//!   `1661`–`1662`). `:Resource`'s acquisition disjunct — the same
//!   mint-shape `writer.rs`'s `open-file`/`from-fd` and
//!   `kernel/resource.rs`'s `HandlePool::new`/`spawn-thread`/`spawn-process`
//!   give their own constructors, despite each minting an object with an
//!   unpredictable identity (a random temp name here, a random fd/tid
//!   there) — that identity is opaque to the Determinism axis, which tracks
//!   whether the call's own OUTCOME is fixed by its arguments, not the
//!   minted value's content. A real syscall, immediate return, no external
//!   actor's timing awaited, so `Effectful`/`Deterministic`.
//! - **`TempFile/path`/`TempDir/path`** (`io.rs:1722`/`1751`, delegating to
//!   `WatTempFile::path`/`WatTempDir::path` at `io.rs:1649`/`1680`, reading
//!   `f.path().display().to_string()`/`d.path().display().to_string()` at
//!   `io.rs:1651`/`1682`) — read a component (the stored path string) off a
//!   handle the caller already holds. No fit for `:Transform` (there is no
//!   value being reshaped, only a handle whose field is read off);
//!   `:Projection`'s own prose almost verbatim — "returns a COMPONENT of a
//!   compound value that was already there." `Pure` (a plain field read, no
//!   mutation, no external call) / `Deterministic` — unlike `writer.rs`'s
//!   `to-bytes`/`to-string` (whose buffer MUTATES across `write*` calls on
//!   the same handle, so two calls can disagree), a `NamedTempFile`'s path
//!   is fixed for the whole life of the handle: two calls on the SAME
//!   handle always agree, and nothing in this substrate can rename it out
//!   from under the caller. Per `purity_mandated_examples`, owes a RUNNABLE
//!   `@example`; the returned path string is itself unpredictable (a random
//!   suffix per construction), so the example wraps the call in a
//!   length-probe rather than asserting a literal path — the same
//!   "wrap the unpredictable value in a deterministic probe" convention
//!   `kernel/source.rs`'s `here`/`fn-forms` examples already use.
//! - **`read-file`/`list-dir`** (`io.rs:1785`/`1810`) — one-shot filesystem
//!   reads that open nothing the caller keeps: `read-file` routes through
//!   the SymbolTable's `SourceLoader` (`loader.fetch_source_file`) to read
//!   a whole file's content; `list-dir` calls `std::fs::read_dir` and
//!   collects each entry's full path. Both cross the process boundary IN —
//!   the read-side mirror of `reader.rs`'s stream family, applied to a
//!   whole-file/whole-directory one-shot rather than an incremental
//!   handle — `:Io`. `Effectful` (real filesystem access) /
//!   `Nondeterministic` (a file's content, or a directory's entries, can
//!   differ between two calls given the identical path argument if
//!   something else on the filesystem changed between them — the same
//!   ambient-state reasoning `reader.rs`/`writer.rs` give their own `:Io`
//!   members).
//!
//! ## Gate coverage — all six plain, registered `TypeScheme`s
//!
//! None of the six has a bespoke `infer_*` arm in `check.rs` — every row is
//! a plain registered `TypeScheme`, gate LIVE, `@ret` compared by the
//! compiler at every floor. No stub `TypeScheme`s were minted to
//! manufacture coverage, and none was touched.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::io::TempFile/new)` → `:wat::io::TempFile`. Creates a fresh,
/// empty, uniquely-named temp file under `std::env::temp_dir()`. Drop
/// unlinks it when the wat value's Arc-count reaches zero.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @ret     :wat::io::TempFile a fresh, auto-deleting temp file handle
/// @example-norun (:wat::io::TempFile/new) #=> #wat.io/TempFile{}
// Registered `TypeScheme` — `check.rs:15946` — gate LIVE. Nullary: zero
// `params`, so zero `@arg` lines above.
//
// Deciding line for `@Category Resource`: `src/io.rs:1703`
// `eval_io_temp_file_new` — `WatTempFile::new(list_span)?` (`io.rs:1637`)
// calls `NamedTempFile::new()` (`io.rs:1638`), which creates a real,
// distinctly-named file on disk that Drop later unlinks (`io.rs:1628`–
// `1629`) — a resource ACQUISITION whose lifetime is tracked outside value
// scope, the same mint-shape `writer.rs`'s `open-file`/`from-fd` give their
// own constructors.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: a
// real syscall with an observable OS-level effect (a new file on disk); no
// external actor's timing is awaited, so the outcome is deterministic
// given the call's own (zero) arguments — the minted file's random name is
// opaque to this axis, the same reasoning `open-file`/`from-fd`/
// `HandlePool::new` stand on despite minting objects with unpredictable
// identity.
#[wat_intrinsic(":wat::io::TempFile/new")]
pub(crate) fn eval_io_temp_file_new(
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_temp_file_new(&[], list_span, env, sym).map_err(Into::into)
}

/// `(:wat::io::TempFile/path temp-file)` → `:wat::core::String`. Returns
/// the on-disk path of `temp-file`. Errors if the handle has already been
/// dropped (defensive; unreachable via a `let`-scoped handle the caller
/// still holds).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     temp_file :wat::io::TempFile the temp file handle to read the path from
/// @ret     :wat::core::String the temp file's on-disk path
/// @example (:wat::i64::> (:wat::string::length (:wat::io::TempFile/path (:wat::io::TempFile/new))) 0) #=> true
// Registered `TypeScheme` — `check.rs:15955` — gate LIVE.
//
// Deciding line for `@Category Projection`: `src/io.rs:1722`
// `eval_io_temp_file_path` — `cell.with_ref(op, |f| f.path())??` delegates
// to `WatTempFile::path` (`io.rs:1649`–`1658`), which reads
// `f.path().display().to_string()` (`io.rs:1651`) off the handle's own
// already-stored `inner: Option<NamedTempFile>` field. A component read off
// a compound value that was already there, not a reshape — `:Projection`'s
// own prose, verbatim.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: a plain
// field read, no mutation, no external call. Unlike `writer.rs`'s
// `to-bytes`/`to-string` (whose buffer mutates across `write*` calls on the
// same handle), a `NamedTempFile`'s path is fixed for the handle's whole
// life — two calls on the SAME handle always agree, so `Deterministic`
// (not `Nondeterministic`, the axis `HandlePool::finish` was corrected
// onto for a live, externally-mutable queue depth — this field has no
// analogous external mutator). Per `purity_mandated_examples`, owes a
// RUNNABLE `@example`; since the path's own content is unpredictable (a
// random suffix picked at construction), the example wraps the call in a
// length-probe rather than asserting a literal path.
#[wat_intrinsic(":wat::io::TempFile/path")]
pub(crate) fn eval_io_temp_file_path(
    temp_file: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_temp_file_path(std::slice::from_ref(temp_file), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::TempDir/new)` → `:wat::io::TempDir`. Creates a fresh, empty,
/// uniquely-named temp directory under `std::env::temp_dir()`. Drop runs
/// `remove_dir_all` when the wat value's Arc-count reaches zero.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @ret     :wat::io::TempDir a fresh, auto-deleting temp directory handle
/// @example-norun (:wat::io::TempDir/new) #=> #wat.io/TempDir{}
// Registered `TypeScheme` — `check.rs:15964` — gate LIVE. Nullary: zero
// `params`, so zero `@arg` lines above.
//
// Deciding line for `@Category Resource`: `src/io.rs:1732`
// `eval_io_temp_dir_new` — `WatTempDir::new(list_span)?` (`io.rs:1668`)
// calls `TempDir::new()` (`io.rs:1669`), which creates a real, distinctly-
// named directory on disk that Drop later `remove_dir_all`s (`io.rs:1661`–
// `1662`) — same acquisition shape as `TempFile/new`, one directory instead
// of one file.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// identical reasoning to `TempFile/new` — a real syscall, immediate
// return, no external actor's timing awaited.
#[wat_intrinsic(":wat::io::TempDir/new")]
pub(crate) fn eval_io_temp_dir_new(
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_temp_dir_new(&[], list_span, env, sym).map_err(Into::into)
}

/// `(:wat::io::TempDir/path temp-dir)` → `:wat::core::String`. Returns the
/// on-disk path of `temp-dir`. Errors if the handle has already been
/// dropped (defensive; unreachable via a `let`-scoped handle the caller
/// still holds).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     temp_dir :wat::io::TempDir the temp dir handle to read the path from
/// @ret     :wat::core::String the temp dir's on-disk path
/// @example (:wat::i64::> (:wat::string::length (:wat::io::TempDir/path (:wat::io::TempDir/new))) 0) #=> true
// Registered `TypeScheme` — `check.rs:15973` — gate LIVE.
//
// Deciding line for `@Category Projection`: `src/io.rs:1751`
// `eval_io_temp_dir_path` — `cell.with_ref(op, |d| d.path())??` delegates
// to `WatTempDir::path` (`io.rs:1680`–`1689`), which reads
// `d.path().display().to_string()` (`io.rs:1682`) off the handle's own
// already-stored `inner: Option<TempDir>` field. Same component-read shape
// as `TempFile/path`.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`:
// identical reasoning to `TempFile/path` — a plain, immutable field read.
// Per `purity_mandated_examples`, owes a RUNNABLE `@example`, wrapped in a
// length-probe for the same reason as `TempFile/path`.
#[wat_intrinsic(":wat::io::TempDir/path")]
pub(crate) fn eval_io_temp_dir_path(
    temp_dir: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_temp_dir_path(std::slice::from_ref(temp_dir), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::read-file path)` → `:wat::core::String`. Returns the full
/// contents of the file at `path`, routed through the SymbolTable's
/// `SourceLoader` — the same capability discipline that gates
/// `:wat::load-file!`/`:wat::eval-file!`. Panics if no loader is attached
/// (the host didn't install one — programmer error, not data-flow).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     path :wat::core::String the path to read
/// @ret     :wat::core::String the file's full contents
/// @example-norun (:wat::io::read-file "/tmp/x.txt") #=> "hello"
// Registered `TypeScheme` — `check.rs:15988` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1785`
// `eval_io_read_file` — `loader.fetch_source_file(&path, None)` routes
// through the SymbolTable's attached `SourceLoader` (`FsLoader` /
// `ScopedLoader` / `InMemoryLoader`) to read the file's full content as a
// String. Data crosses the process boundary IN — the read-side mirror of
// `reader.rs`'s stream family, applied to a whole-file one-shot rather than
// an incremental handle.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// a real filesystem read; the same path argument can return different
// content across two calls if the file changed on disk between them — the
// same ambient-state reasoning `reader.rs`/`writer.rs` give their own
// `:Io` members.
#[wat_intrinsic(":wat::io::read-file")]
pub(crate) fn eval_io_read_file(
    path: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_read_file(std::slice::from_ref(path), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::list-dir path)` → `(:wat::core::Vector :- [wat::core::String])`.
/// Enumerates the directory at `path`; returns each entry as a full path
/// (`entry.path()` already joins the input path with the entry name).
/// Errors (`MalformedForm`) if `path` does not exist or is not a
/// directory.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     path :wat::core::String the directory to list
/// @ret     (:wat::core::Vector :- [:wat::core::String]) each entry's full path
/// @example-norun (:wat::io::list-dir "wat") #=> Vector["wat/fix.wat" "wat/core.wat" …]
// Registered `TypeScheme` — `check.rs:15997` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1810`
// `eval_io_list_dir` — `std::fs::read_dir(path.as_str())`, then collects
// each entry's `entry.path()` into a `Vec<Value::String>`. Data (the
// directory's entry list) crosses the process boundary IN — same `:Io`
// reasoning as `read-file`, one directory listing instead of one file's
// bytes.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// a real filesystem read; the same path argument can return a different
// entry set across two calls if the directory changed between them —
// identical reasoning to `read-file`.
#[wat_intrinsic(":wat::io::list-dir")]
pub(crate) fn eval_io_list_dir(
    path: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_io_list_dir(std::slice::from_ref(path), list_span, env, sym).map_err(Into::into)
}
