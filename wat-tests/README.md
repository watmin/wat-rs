# wat-tests/

Tests written in wat, for wat. The sibling to `wat/` the way Cargo's
`tests/` is the sibling to `src/`.

Each `.wat` file uses `:wat::test::deftest` to register named test
functions. `cargo test` (via `tests/test.rs` and `wat::test! {}`)
auto-discovers every top-level `:wat::core::define` whose path's final
`::`-segment starts with `test-` and whose signature is `() ->
:wat::kernel::RunResult`, shuffles them, invokes each, and reports
cargo-test-style.

## Layout

Each `.wat` source file under `wat/<namespace>/` has a matching
test file under `wat-tests/<namespace>/`:

```
wat/holon/Subtract.wat         ↔ wat-tests/holon/Subtract.wat
wat/holon/Circular.wat         ↔ wat-tests/holon/Circular.wat
wat/holon/Reject.wat           ↔ wat-tests/holon/Reject.wat
wat/holon/Project.wat            (tested alongside Reject)
wat/holon/Sequential.wat       ↔ wat-tests/holon/Sequential.wat
wat/holon/Trigram.wat          ↔ wat-tests/holon/Trigram.wat
wat/test.wat                 ↔ wat-tests/test.wat
wat/std/service/Console.wat  ↔ wat-tests/std/service/Console.wat
wat/stream.wat               ↔ wat-tests/stream.wat
```

The stdlib module under test dictates the path. A future addition to
`wat/<ns>/X.wat` expects its tests at `wat-tests/<ns>/X.wat`.

**External crates ship their own `wat-tests/` trees.** Consumer
wat crates — both workspace members like `crates/wat-lru/` (arcs
013 + 036) and out-of-tree consumers — follow the same layout
under their own crate root: `<crate>/wat/<ns>/X.wat` paired with
`<crate>/wat-tests/<ns>/X.wat`, discovered and run via
`cargo test -p <crate>` through `wat::test! {}` in
`<crate>/tests/test.rs` (arc 015's external-crate test contract).
The pattern this README describes for wat-rs is portable to any
wat consumer; nothing about it is wat-rs-specific.

## Running

```
cargo test                          # full suite via tests/test.rs (wat::test! {})
cargo test -- --nocapture           # stream per-test output live
cargo test wat_suite -- --show-output  # print output after the suite completes
```

Cargo's libtest captures stdout from passing tests by default; the
`wat::test!` runner's per-test report lines appear only on failure
unless you opt into one of the flags above.

Discovery is recursive from the `wat-tests/` root. Random order
per-file (nanos-seeded xorshift Fisher-Yates) surfaces accidental
order-dependencies.

The `wat test <path>` CLI binary still ships — it was the original
pre-cargo-integration workflow — but `cargo test` is the canonical
entry point now.

## In-process vs hermetic

Most readers reach for the user-facing macros first:
`:wat::test::deftest` (in-process sandbox), `:wat::test::deftest-
hermetic` (forked subprocess), and `:wat::test::make-deftest` /
`make-deftest-hermetic` (factories whose default-prelude carries
shared loads/helpers across tests in a file — arc 029). Both
factory and direct shapes inherit the outer file's Config (arc
031) so neither takes per-test `mode` / `dims` arguments.

The substrate primitives the macros expand to:

- **`:wat::test::run`** — in-process sandbox with
  StringIoWriter-backed stdio. ThreadOwnedCell discipline means
  single-thread; reach for it directly when you need to drive
  the sandbox by hand.
- **`:wat::test::run-hermetic-ast`** — the wat stdlib wrapper
  that forks a child via `:wat::kernel::fork-program-ast` and
  runs the inner program with fd-backed thread-safe stdio
  (`PipeReader` / `PipeWriter`). Used for programs that spawn
  threads and write from them (Console, Cache). See
  `wat/std/hermetic.wat` for the implementation.

Default to the macros; drop down to the primitives when a
sandbox needs custom wiring.

## Naming

- Test function names: final segment **must** start with `test-` for
  auto-discovery. Fixture functions that shouldn't run as tests use
  a non-`test-` final segment.
- File names: hyphenated if multi-word (`test-harness.wat`, not
  `test_harness.wat`).
