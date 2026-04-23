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
wat/std/test.wat             ↔ wat-tests/std/test.wat
wat/std/service/Console.wat  ↔ wat-tests/std/service/Console.wat
wat/std/stream.wat           ↔ wat-tests/std/stream.wat
```

The stdlib module under test dictates the path. A future addition to
`wat/<ns>/X.wat` expects its tests at `wat-tests/<ns>/X.wat`.

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

Simple tests use `:wat::test::run` (in-process sandbox with
StringIoWriter-backed stdio — ThreadOwnedCell discipline means
single-thread). Programs that spawn threads and write from them
(Console, Cache) use `:wat::test::run-hermetic-ast` — the wat
stdlib wrapper that forks a child via `:wat::kernel::fork-with-forms`
and runs the inner program with fd-backed thread-safe stdio
(`PipeReader` / `PipeWriter`). See `wat/std/hermetic.wat` for the
implementation.

## Naming

- Test function names: final segment **must** start with `test-` for
  auto-discovery. Fixture functions that shouldn't run as tests use
  a non-`test-` final segment.
- File names: hyphenated if multi-word (`test-harness.wat`, not
  `test_harness.wat`).
