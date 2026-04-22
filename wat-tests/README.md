# wat-tests/

Tests written in wat, for wat. The sibling to `wat/` the way Cargo's
`tests/` is the sibling to `src/`.

Each `.wat` file uses `:wat::test::deftest` to register named test
functions. `wat test wat-tests/` auto-discovers every top-level
`:wat::core::define` whose path's final `::`-segment starts with
`test-` and whose signature is `() -> :wat::kernel::RunResult`,
shuffles them, invokes each, and reports cargo-test-style.

## Layout

Mirrors `wat/std/` one-to-one:

```
wat/std/Subtract.wat         ↔ wat-tests/std/Subtract.wat
wat/std/Circular.wat         ↔ wat-tests/std/Circular.wat
wat/std/Reject.wat           ↔ wat-tests/std/Reject.wat
wat/std/Project.wat            (tested alongside Reject)
wat/std/Sequential.wat       ↔ wat-tests/std/Sequential.wat
wat/std/Trigram.wat          ↔ wat-tests/std/Trigram.wat
wat/std/test.wat             ↔ wat-tests/std/test.wat
wat/std/service/Console.wat  ↔ wat-tests/std/service/Console.wat
wat/std/service/Cache.wat    ↔ wat-tests/std/service/Cache.wat
```

The stdlib module under test dictates the path. A future addition to
`wat/std/X.wat` expects its tests at `wat-tests/std/X.wat`.

## Running

```
wat test wat-tests/               # every .wat in tree, cargo-style report
wat test wat-tests/std/           # just stdlib tests
wat test wat-tests/std/test.wat   # single file
```

Discovery is recursive. Random order per-file (nanos-seeded xorshift
Fisher-Yates) surfaces accidental order-dependencies.

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
