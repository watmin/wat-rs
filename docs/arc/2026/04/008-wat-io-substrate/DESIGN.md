# wat IO substrate — `:u8` + `:wat::io::IOReader` + `:wat::io::IOWriter`

**Status:** planned. Opened 2026-04-21 as a prerequisite detour during
arc 007 slice 2a.
**Blocks:** arc 007 slice 2a onwards — `run-sandboxed` needs
substitutable stdio. Concrete `:rust::std::io::Stdin/Stdout/Stderr`
cannot be swapped for in-memory buffers; that's what this arc fixes.
**Motivation:** the Ruby StringIO model made operational in wat —
a wat program receives "an IO thing"; at production it's real stdio,
at test it's a string-buffer stand-in, same source runs in both.

---

## The gap this arc closes

Today `:user::main` takes concrete types:

```
(:user::main
  (stdin  :rust::std::io::Stdin)
  (stdout :rust::std::io::Stdout)
  (stderr :rust::std::io::Stderr)
  -> :())
```

Value variants hold `Arc<std::io::Stdin>` etc. Real OS handles.
For arc 007's `run-sandboxed` to capture stdio, we'd need to
substitute those values with in-memory backings — which isn't
possible because Value::io__Stdin specifically holds a real
`std::io::Stdin`.

The honest fix is an **abstract IO layer**. One wat-level type pair
(`IOReader` + `IOWriter`); multiple concrete backings (real stdio,
string-buffer stand-in). Main declares the abstract type; runtime
provides the concrete.

Ruby's StringIO is the conceptual model. Rust's `Read` + `Write`
traits are the structural precedent. wat introduces its own named
types because Rust's std has no unified "IO" type — these are
wat-owned abstractions built on Rust primitives, same way
`:wat::std::LocalCache` wraps `:rust::lru::LruCache`.

---

## Three slices

### Slice 1 — `:u8` primitive type

Byte-level IO (partial reads, write counts, fd-honest semantics)
needs a byte type. Today wat has `:i64` and `:f64` but no `:u8`.

Adds:

- **`:u8` primitive integer type.** Parser recognizes; type checker
  handles; `Value::u8(u8)` variant.
- **Literal parsing.** Annotated: `(:wat::core::u8 42)` or
  per-context coercion (numeric literal 0..=255 fits `:u8` when the
  expected type is `:u8`). Pick one discipline during slice 1.
- **`:Vec<u8>` parametric plumbing.** Consistency with existing
  `:Vec<T>` handling; no new abstraction, just working through the
  type/check/dispatch sites that need to know about the new primitive.
- **`:wat::core::u8::+ / - / * / /` arithmetic.** With wrapping or
  saturating semantics (pick wrapping — matches Rust's default for
  `u8`). Overflow/underflow behavior decided during implementation.

Tests cover literal parsing, Vec<u8> construction + access, type
check rejecting out-of-range literals.

Not shipping this slice: `:i8`, `:u16`, `:u32`, `:u64`, `:i32`.
Speculation. Add when a caller demands.

### Slice 2 — `:wat::io::IOReader` + `:wat::io::IOWriter` + primitives

Two wat-level types (split matches Rust's `Read` / `Write` trait
split; stdin is readable; stdout/stderr are writable; no type muddles
them together):

```
:wat::io::IOReader   ;; opaque wat type
:wat::io::IOWriter   ;; opaque wat type
```

Backed by Rust traits:

```rust
pub trait WatReader: Send + Sync {
    fn read(&self, n: usize) -> Option<Vec<u8>>;
    fn read_all(&self) -> Vec<u8>;
    fn read_line(&self) -> Option<String>;  // UTF-8 decode; None on EOF
    fn rewind(&self);                         // seek-to-start; no-op for real stdin
}

pub trait WatWriter: Send + Sync {
    fn write(&self, bytes: &[u8]) -> usize;
    fn write_all(&self, bytes: &[u8]);
    fn flush(&self);
}
```

Value variants:

```rust
Value::io__IOReader(Arc<dyn WatReader + Send + Sync>)
Value::io__IOWriter(Arc<dyn WatWriter + Send + Sync>)
```

**Concrete impls, four of them:**

```rust
// Real stdio — wrap Rust's stdlib handles; Rust stdlib's internal
// locking is its own business, wat-rs introduces no Mutex.
struct RealStdin  { inner: Arc<std::io::Stdin>  }
struct RealStdout { inner: Arc<std::io::Stdout> }
struct RealStderr { inner: Arc<std::io::Stderr> }

// Test stand-ins — ThreadOwnedCell-backed, matches tier-2 discipline.
// Single-thread-owned; cross-thread use panics with owner-check error.
// Zero Mutex in wat-rs code.
struct StringIoReader {
    state: ThreadOwnedCell<ReaderState>,  // { bytes: Vec<u8>, cursor: usize }
}
struct StringIoWriter {
    state: ThreadOwnedCell<Vec<u8>>,
}
```

**Primitives (mirror Rust's Read / Write / BufRead method names):**

```
;; byte-level (the floor — fd-honest)
(:wat::io::read      reader n)        -> :Option<Vec<u8>>
(:wat::io::read-all  reader)          -> :Vec<u8>
(:wat::io::write     writer bytes)    -> :i64
(:wat::io::write-all writer bytes)    -> :()

;; char-level conveniences (UTF-8 encode/decode on top)
(:wat::io::read-line reader)          -> :Option<String>
(:wat::io::writeln   writer s)        -> :i64

;; common
(:wat::io::flush     writer)          -> :()
(:wat::io::rewind    reader)          -> :()
```

**Construction primitives (for test instances):**

```
(:wat::io::IOReader/from-bytes  bytes) -> :IOReader         ;; Vec<u8> backing
(:wat::io::IOReader/from-string s)     -> :IOReader         ;; String -> bytes
(:wat::io::IOWriter/new)               -> :IOWriter         ;; empty buffer
(:wat::io::IOWriter/to-bytes  writer)  -> :Vec<u8>          ;; clone collected bytes
(:wat::io::IOWriter/to-string writer)  -> :Option<String>   ;; UTF-8 decode; None on invalid
```

Real stdio instances are constructed by the CLI (wrap real handles)
and by `run-sandboxed` (StringIo variants). User code doesn't
construct real stdio — it receives them from its main params.

### Slice 3 — `:user::main` contract migration + INSCRIPTION

- **Update `:user::main` expected signature** from
  `(:Stdin, :Stdout, :Stderr) -> :()` to
  `(:IOReader, :IOWriter, :IOWriter) -> :()`. Update
  `validate_user_main_signature` in lib (moved per arc 007 slice 2
  plan; now lives alongside other sandbox plumbing).
- **CLI wraps real Rust stdio as IO values.** `bin/wat-vm.rs` builds
  `Arc<RealStdin>` / `Arc<RealStdout>` / `Arc<RealStderr>` and
  invokes main with the three Value::io__IOReader / io__IOWriter
  Values.
- **Retire `Value::io__Stdin` / `io__Stdout` / `io__Stderr`** — fully
  replaced by the IO variants. Same underlying Rust handles, new
  wrapper types.
- **Retire `:wat::io::write` / `:wat::io::read-line`** (the old
  primitives that dispatched on Value::io__Stdin et al) — replaced
  by the new byte- and line-oriented primitives from slice 2.
- **Migrate every wat file declaring `:user::main`:** trading lab's
  production main, test wat files under `wat/` in wat-rs. Sweep.
- **Arc 008 INSCRIPTION.** Dated record. Close arc. Then arc 007
  slice 2a resumes — `run-sandboxed` builds StringIo instances,
  invokes main, drains writer buffers into RunResult.stdout/stderr.

---

## Discipline

- **Zero Mutex in wat-rs code.** Real stdio wraps Rust's stdlib
  handles; Rust stdlib has its own internal locking (via
  `StdoutLock` etc.) — that's transitively downstream and opaque to
  wat, not wat-introduced. StringIo instances use
  `ThreadOwnedCell`, matching the tier-2 `LocalCache` discipline.
  No new `Mutex`, `RwLock`, or equivalent inside wat-rs.
- **No thread spawn for StringIo.** Every IO call is synchronous on
  the caller's thread — just like a real fd call. ThreadOwnedCell
  enforces single-thread-ownership; cross-thread use panics (same
  behavior LocalCache has today).
- **Byte-level is the floor.** `read` / `write` operate on
  `:Vec<u8>`; they carry fd-honest semantics (partial reads,
  write-count returned). Char-level primitives (`read-line`,
  `writeln`) are UTF-8 conveniences layered on top.
- **IOReader and IOWriter are separate types** — matches Rust's
  trait split. stdin is typed IOReader, stdout/stderr are typed
  IOWriter. The type system catches "write to stdin" at check time.

---

## Concurrency profile — divergence named

Real stdio is multi-thread-safe (Rust stdlib's internal locking).
StringIo is single-thread-owned (ThreadOwnedCell).

Consequence: a wat program that spawns IO-using worker threads and
runs fine in production (real stdio) may panic in a sandboxed test
(StringIo). That's the ThreadOwnedCell owner-check error — same
failure mode every tier-2 wat value has.

**This is intentional.** Multi-thread-safe StringIo would require
either Mutex (violates discipline) or a program-with-driver-thread
(violates "all calls synchronous on caller's thread"). The test
harness accepts single-thread IO; integration-test flows that need
multi-thread IO run against real stdio (via the CLI, not run-sandboxed).

Documented in the arc 008 INSCRIPTION; users coming from the Ruby
model know this limitation exists and read the discipline doc.

---

## What this scaffolds for arc 007

- **Slice 2a** — `run-sandboxed` constructs three StringIo instances
  (pre-seed IOReader from stdin lines; two empty IOWriters for
  stdout/stderr). Invokes sandboxed main with them. Drains the
  IOWriter buffers into `RunResult.stdout` / `stderr`.
- **Slice 3** — `:wat::test::assert-stdout-is` reads the IOWriter's
  captured output (via `to-string`), compares to expected.
- **Slice 4** — `wat-vm test` CLI runs tests in-process using the IO
  abstraction; hermetic-mode future arc can serialize IOWriter
  contents across process boundaries.

---

## Out of scope for this arc

- **Full Rust `Read` / `Write` / `BufRead` method coverage.** We
  ship what's used: read / read-all / read-line / rewind for
  readers; write / write-all / writeln / flush for writers. Add
  methods (`read_to_end`, `read_exact`, `seek` beyond rewind) when a
  caller demands.
- **File IO via IOReader / IOWriter.** Opening arbitrary files as
  IOReader — not yet demanded. `ScopedLoader` already handles
  startup loads and runtime `:wat::eval::file-path`; opening a file
  as an IOReader for stream-style reads is a different capability,
  future work.
- **Async / nonblocking IO.** wat's model is threads-and-channels;
  async doesn't fit. If a future caller needs nonblocking fd, that's
  an own arc with its own design.
- **Seekable writers.** StringIoWriter is append-only. Seeking would
  need a cursor; trivial to add when demanded.
- **`:i8` / `:u16` / `:u32` / `:i32` / `:u64` primitive types.** This
  arc introduces `:u8` specifically for byte IO. Other integer
  widths ship when callers demand.

---

## The thesis

Same language-level substitution Ruby's StringIO gives. Wat source
that reads stdin and writes stdout runs identically against real OS
handles (production) and in-memory buffers (tests). The IO is the
IO; what backs it is opaque. The substitutability is what arc 007's
self-hosted testing rides on.
