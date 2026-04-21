# Arc 008 — wat IO substrate — Backlog

**Opened:** 2026-04-21 (detour during arc 007 slice 2a when discovered
that `:user::main` takes concrete stdio types that can't be
substituted for in-memory buffers).
**Design:** [`DESIGN.md`](./DESIGN.md).
**Blocks:** arc 007 slice 2a onward.

---

## Tracking

| Slice | Item | Status | Commit |
|---|---|---|---|
| 1 | `:u8` primitive type — parser | **done** (transparent via TypeExpr::Path) | this slice |
| 1 | `:u8` primitive type — type checker | **done** (scheme for `:wat::core::u8` registered) | this slice |
| 1 | `Value::u8` variant + type_name | **done** | this slice |
| 1 | `:Vec<u8>` parametric plumbing | **done** (works via existing `:Vec<T>` infra) | this slice |
| 1 | `:wat::core::u8` range-checked cast primitive | **done** | this slice |
| 1 | `:wat::core::u8::+/-/*//` arithmetic (wrapping) | **deferred** — no caller demand yet; stdlib-as-blueprint | — |
| 1 | slice 1 tests | **done** (9 tests in `tests/wat_u8.rs`) | this slice |
| 2 | `WatReader` + `WatWriter` traits (in new `src/io.rs`) | **done** | this slice |
| 2 | `Value::io__IOReader` + `Value::io__IOWriter` variants | **done** | this slice |
| 2 | `RealStdin` / `RealStdout` / `RealStderr` impls (wrap Rust handles) | **done** (not yet wired into CLI — slice 3) | this slice |
| 2 | `StringIoReader` + `StringIoWriter` impls (ThreadOwnedCell-backed) | **done** | this slice |
| 2 | byte-level primitives: read / read-all / write / write-all | **done** under `IOReader/` + `IOWriter/` prefix | this slice |
| 2 | char-level primitives: read-line / writeln | **done** | this slice |
| 2 | common: flush / rewind | **done** | this slice |
| 2 | construction: `IOReader/from-bytes` + `from-string`; `IOWriter/new` + `to-bytes` + `to-string` | **done** | this slice |
| 2 | type registration (`:wat::io::IOReader`, `:wat::io::IOWriter` as opaque types) | **done** (transparent via TypeExpr::Path) | this slice |
| 2 | type schemes in check.rs | **done** | this slice |
| 2 | slice 2 tests | **done** (15 tests in `tests/wat_io.rs`) | this slice |
| 3 | update `validate_user_main_signature` — new three-IO contract | pending | — |
| 3 | CLI (`bin/wat-vm.rs`) wraps real stdio as IO Values | pending | — |
| 3 | retire `Value::io__Stdin/Stdout/Stderr` variants | pending | — |
| 3 | retire old `:wat::io::write` / `:wat::io::read-line` primitives (replaced) | pending | — |
| 3 | migrate every wat file declaring `:user::main` (trading lab, test wat files) | pending | — |
| 3 | USER-GUIDE update — new IO section replaces old io::write/read-line | pending | — |
| 3 | arc 008 INSCRIPTION | pending | — |

---

## Decision log

- **2026-04-21** — Opened arc 008 as prerequisite to arc 007 slice 2a.
  Discovered that `:user::main` takes concrete `:rust::std::io::Stdin`
  / `Stdout` / `Stderr` — can't be substituted for in-memory buffers.
  ran-sandboxed needs substitutable stdio. Ruby StringIO is the
  conceptual model; Rust's Read/Write traits are the structural
  precedent; wat owns the abstraction (`:wat::io::IOReader` /
  `IOWriter` are wat-level types, no matching std type in Rust).
- **2026-04-21** — **Byte-level + char-level layered.** Option C
  chosen (byte-oriented floor). `:u8` is slice 1 prereq. Byte
  primitives (`read`, `write`) operate on `:Vec<u8>`; char primitives
  (`read-line`, `writeln`) are UTF-8 decode/encode conveniences on
  top. Rejected option B (char-only) — lost fd honesty and binary
  capability; rejected option A (line-only) — too narrow.
- **2026-04-21** — **IOReader + IOWriter split, NOT unified IO.**
  Mirrors Rust's Read / Write trait separation. stdin typed
  IOReader; stdout/stderr typed IOWriter. Rejected unified `IO` type
  because it muddled the capability boundary — a program could try
  to read from stdout and the type system wouldn't catch it.
- **2026-04-21** — **ThreadOwnedCell for StringIo, no Mutex.**
  Matches wat's zero-Mutex discipline. Rejected `Mutex<Vec<u8>>`
  (laziness; wat has a tier-2 pattern for interior mutability).
  Rejected program-with-driver-thread (adds channel round-trip per
  IO call; overkill for single-thread test use). StringIo is
  single-thread-owned; cross-thread use panics with the owner-check
  error — same behavior `:wat::std::LocalCache` has. Multi-thread
  stdio in tests requires real stdio (via CLI, not run-sandboxed)
  or explicit channel-based IO — documented divergence.
- **2026-04-21** — **No thread spawn for StringIo.** All IO calls
  synchronous on caller's thread. Builder's direction: "closure
  that captures state, blocking calls." Matches real fd semantics —
  a read call blocks the caller until data or EOF.
- **2026-04-21** — **Rust stdlib's internal locking is opaque to
  wat.** Real stdio uses `std::io::StdoutLock` / equivalent
  internally when you call `lock()`. That's Rust's stdlib's
  concurrency story, not wat's. Wat claims zero-Mutex in its own
  code; transitive deps have their own disciplines.
- **2026-04-21** — Slice 1 shipped. `:u8` exists as a primitive
  type. Cast primitive `:wat::core::u8` takes `:i64` and
  range-checks at runtime (0..=255 or MalformedForm). Comparison
  works via existing polymorphic `=`. `:Vec<u8>` construction works
  via existing parametric plumbing. **`:wat::core::u8::+` and
  siblings deferred** — no caller demand during slice 2 design;
  ship when demanded. Adds zero lines to the primitive zoo today;
  arithmetic is one edit away when needed. 9 tests passing; 498+
  unit tests + all integration tests green.
- **2026-04-21** — Slice 2 shipped. `:wat::io::IOReader` +
  `:wat::io::IOWriter` exist as opaque wat types.
  `Value::io__IOReader` / `io__IOWriter` variants hold
  `Arc<dyn WatReader|WatWriter + Send + Sync + Debug>`. Four concrete
  impls in `src/io.rs`: `RealStdin`/`Stdout`/`Stderr` (wrap Rust
  handles; not yet wired to CLI — that's slice 3),
  `StringIoReader`/`StringIoWriter` (ThreadOwnedCell-backed; zero
  Mutex; single-thread-owned).
  **Primitives use `<Type>/<method>` naming** for this slice — chose
  this over short-form (`:wat::io::read`) because short-form conflicts
  with the existing `:wat::io::write` / `:wat::io::read-line` that
  operate on the old `io__Stdin`/`Stdout`/`Stderr` variants. Slice 3
  may rename once old primitives retire. 13 primitives: IOReader
  construction (from-bytes, from-string) + ops (read, read-all,
  read-line, rewind); IOWriter construction (new) + snapshots
  (to-bytes, to-string) + ops (write, write-all, writeln, flush).
  Byte-level is the floor; char-level uses UTF-8 encode/decode.
  **WatWriter trait has `snapshot() -> Option<Vec<u8>>`** — defaults
  None (real stdio can't snapshot); StringIoWriter overrides Some.
  Let the trait answer "can I be captured?" rather than downcasting
  at dispatch.
  **Bug caught in check schemes:** had `head: ":Vec"` with leading
  colon; format_type prepends its own colon, producing `"::Vec<u8>"`
  in error messages. Fix: head strings in check.rs schemes don't
  include the leading colon. Existing `"Vec"` / `"Option"` usage
  elsewhere in check.rs confirmed as convention. 15 tests passing;
  full suite (25 suites, 498+ unit + integration) green.

---

## What resumes after arc 008 closes

Arc 007's slice 2a was paused when arc 008 scope was surfaced. After
arc 008 ships:

1. Slice 2a resumes — `run-sandboxed` constructs three StringIo
   instances, invokes main with IOReader + IOWriter + IOWriter,
   drains buffers into RunResult.stdout / stderr.
2. Slices 2b / 3 / 4 / 5 / INSCRIPTION proceed per arc 007 backlog.

This arc stays open only for its own work; arc 007 tracks its own
completion independently.
