# Arc 008 — wat IO substrate — INSCRIPTION

**Status:** shipped 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — pre-ship intent + decision
record.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — narrative of the three
slices, with the decision log.
**This file:** completion marker.

Same inscription discipline as arcs 003 / 004 / 005 / 006: DESIGN
was the intent; BACKLOG is the narrative; this INSCRIPTION is the
shipped contract. If DESIGN and INSCRIPTION disagree, INSCRIPTION
wins.

---

## What shipped

Three slices across one session:

### Slice 1 — `:u8` primitive type (commit `9f4920a`)

`Value::u8(u8)` variant with `type_name() = "u8"`. Range-checked
cast primitive `:wat::core::u8 :i64 -> :u8` — rejects values
outside 0..=255 at runtime with `RuntimeError::MalformedForm`.
`:Vec<u8>` construction works via existing parametric plumbing.
Polymorphic `=` extended to cover u8 pairs. 9 tests.

Deferred: `:wat::core::u8::+` / `-` / `*` / `/` arithmetic. No
caller demand during slice 2 design; stdlib-as-blueprint says ship
when a real caller needs it.

### Slice 2 — `:wat::io::IOReader` + `:wat::io::IOWriter` + StringIo (commit `b955901`)

Two opaque wat types, split per Rust's Read/Write trait model.
New module `src/io.rs`:

- **Traits:** `WatReader` + `WatWriter` (both `Send + Sync + Debug`).
- **Value variants:** `io__IOReader(Arc<dyn WatReader>)`,
  `io__IOWriter(Arc<dyn WatWriter>)`.
- **Four concrete impls:** `RealStdin` / `RealStdout` / `RealStderr`
  (wrap `Arc<std::io::*>` — Rust stdlib handles its own internal
  locking; wat-rs introduces no Mutex), `StringIoReader` /
  `StringIoWriter` (`ThreadOwnedCell`-backed; tier-2 discipline;
  zero Mutex; single-thread-owned; all ops synchronous on caller's
  thread — no channel round-trip, no driver spawn).
- **13 primitives** under the `<Type>/<method>` convention:
  - IOReader: `from-bytes`, `from-string`, `read`, `read-all`,
    `read-line`, `rewind`
  - IOWriter: `new`, `to-bytes`, `to-string`, `write`, `write-all`,
    `write-string`, `writeln`, `flush`
- **Bug caught**: parametric head strings in check-scheme registrations
  don't carry a leading colon — `format_type` prepends its own. My
  initial `head: ":Vec"` produced `"::Vec<u8>"` (double colon) in
  error messages; fixed to `head: "Vec"` matching the convention
  everywhere else in `check.rs`.

15 tests. Byte-level is the floor (fd-honest partial reads, write
counts); char-level layered on top as UTF-8 convenience.

### Slice 3 — `:user::main` contract migration (this commit)

The slice that makes arc 007 unblockable.

**CLI + validate_user_main_signature migrated.** `bin/wat.rs`
now wraps real OS stdio as `Value::io__IOReader` /
`Value::io__IOWriter`:

```rust
Arc::new(wat::io::RealStdin::new(Arc::new(io::stdin())))
```

...instead of the retired `Value::io__Stdin(Arc<std::io::Stdin>)`.
`validate_user_main_signature` expects
`(IOReader, IOWriter, IOWriter) -> ()`.

**Two unit-returning conveniences added.** Original sweep of the
CLI integration tests surfaced that `write-string` returns `:i64`
(fd-honest) but match arms returning `:()` (discarding the count)
add boilerplate everywhere. Added:

- `:wat::io::IOWriter/print writer s -> :()` — writes bytes, no
  newline, discards count. Ruby's `$stdout.print`.
- `:wat::io::IOWriter/println writer s -> :()` — writes `s` + `\n`,
  discards count. Ruby's `$stdout.puts`.

These are convenience wrappers over `write-all`; the fd-honest
count-returning primitives stay for callers who need them.

**Console.wat migrated.** `:wat::std::service::Console/loop` takes
`:wat::io::IOWriter` for stdout and stderr; dispatch uses
`:wat::io::IOWriter/write-string` (preserves no-implicit-newline
semantics the old `:wat::io::write` had).

**Retired:** `Value::io__Stdin` / `Value::io__Stdout` /
`Value::io__Stderr` variants; `eval_io_write` + `eval_io_read_line`
handlers; `:wat::io::write` + `:wat::io::read-line` dispatch arms
and check schemes. All references in the codebase swept.

**Every wat file declaring `:user::main` updated.** `Console.wat`
(stdlib) + `tests/wat_vm_cli.rs` (17 CLI integration tests, 56
references) + `tests/wat_vm_cache.rs` (1 test). Replaced
`:rust::std::io::Stdin/Stdout/Stderr` with `:wat::io::IOReader` /
`IOWriter`; replaced `:wat::io::write` with
`:wat::io::IOWriter/print`; replaced `:wat::io::read-line` with
`:wat::io::IOReader/read-line`.

**UTF-8 bug caught mid-migration and fixed** (commit included in
this slice). When writing a test for `write-string` returning byte
counts, I used `"héllo"` (6 UTF-8 bytes). Test returned a different
number. Root cause: `src/lexer.rs::lex_string` iterated
`src.as_bytes()` and did `bytes[i] as char` — treating each byte
as a Latin-1 char and UTF-8-re-encoding the result. `"héllo"` (6
source bytes) became 8 bytes in the lexed `String`. Fix: iterate
via `src.char_indices()` so UTF-8 structure is preserved. The
claim "wat's String is UTF-8" was silently false for any non-ASCII
literal before this arc. Now honest.

**Docs updated:** USER-GUIDE examples migrated to new primitives;
README prose + examples; INVENTORY rows for retired primitives
marked and new primitives added.

---

## Tests

All 25 test suites green:

- 499 unit tests (including 4 new lexer UTF-8 tests).
- 17 tests in `tests/wat_io.rs` (arc 008 slice 2 added).
- 17 tests in `tests/wat_vm_cli.rs` (migrated to new contract).
- 9 tests in `tests/wat_u8.rs` (arc 008 slice 1).
- Plus unchanged test suites from prior arcs.

Zero regressions across the sweep.

---

## Discipline locked

Every decision from DESIGN.md held:

- **Zero Mutex in wat-rs code.** `StringIoReader` / `StringIoWriter`
  use `ThreadOwnedCell`. Real stdio delegates to Rust stdlib's
  internal locking (transitive dep, opaque to wat).
- **Byte-level IO is the floor.** `read` / `write` take `:Vec<u8>`;
  partial-read and write-count semantics fd-honest. Char-level
  primitives (`read-line`, `writeln`, `print`, `println`) layered
  on top as UTF-8 decode/encode conveniences.
- **IOReader + IOWriter are separate types** — stdin typed
  IOReader, stdout/stderr typed IOWriter. Type system catches
  "write to stdin" at check time.
- **No thread spawn for StringIo.** All IO calls synchronous on
  caller's thread.
- **Concurrency profile divergence documented.** Real stdio is
  multi-thread-safe; StringIo is single-thread-owned. Multi-thread
  IO in tests requires real stdio.

---

## Lessons captured

**UTF-8 correctness as a cross-session lesson.** The lexer bug is
instructive: a primitive that *claims* to operate on UTF-8 must
actually preserve UTF-8 structure. Byte-at-a-time iteration through
a UTF-8 `&str` treated as `&[u8]` can corrupt silently — you only
notice when someone asserts on byte lengths. Every future primitive
that transforms `String` contents should be tested with multi-byte
input.

**Byte-honest IO is testable IO.** When `write-string` returns a
byte count, tests can assert exact byte counts against expected
UTF-8 encodings. That's a richer contract than the old
`:wat::io::write` that returned `:()`.

**The substitutability payoff is concrete.** Before this arc, a
wat program using `:user::main` couldn't be invoked from wat-level
code — the `:rust::std::io::Stdin` argument required a real OS
handle. After this arc, a wat program can be invoked from any wat
context by providing `StringIo` instances. That's what arc 007's
`run-sandboxed` needs; now it's buildable.

---

## What resumes after arc 008

Arc 007 slice 2a was paused at the discovery that `:user::main`'s
three concrete stdio types couldn't be substituted. With
`:wat::io::IOReader` + `:wat::io::IOWriter` as the new contract,
`:wat::kernel::run-sandboxed` becomes:

1. Parse wat source, freeze into an inner world.
2. Construct `StringIoReader::from_string(stdin_lines.join("\n"))`.
3. Construct two `StringIoWriter::new()`.
4. Wrap each in `Arc<dyn WatReader|WatWriter>` + Value.
5. `invoke_user_main(&inner_world, [rdr, w_out, w_err])`.
6. Snapshot the writers via `WatWriter::snapshot()`.
7. Return `RunResult { stdout, stderr }`.

The path is clean. Slice 2a resumes next.

---

**Arc 008 — complete.** The Ruby StringIO model made operational in
wat. Substrate for arc 007's self-hosted testing. Byte-honest IO
without Mutex. UTF-8 corruption bug exorcised from the lexer along
the way.

*these are very good thoughts.*

**PERSEVERARE.**
