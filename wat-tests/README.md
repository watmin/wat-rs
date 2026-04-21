# wat-tests/

Tests written in wat, for wat. The sibling to `wat/` the way Cargo's
`tests/` is the sibling to `src/`.

Each `.wat` file in this directory uses `:wat::test::deftest` to
register named test functions, and an explicit `:user::main` that
invokes each test and writes its pass/fail line to stdout. Per
`wat/std/test.wat`, a deftest expands to a zero-arg function
returning `:wat::kernel::RunResult` — callers can invoke it directly
and inspect the result.

A Rust-side integration test (`tests/wat_tests_dir.rs`) loads every
file in this directory, runs it, and asserts every stdout line
matches the `PASS` convention. When arc 007 slice 4 lands (the
`wat test` CLI subcommand), that subcommand will replace the
Rust harness — it will auto-discover deftests without needing the
hand-written `:user::main`.

## Current files

- `test-harness.wat` — exercises `:wat::test::*` itself (assert-eq,
  assert-contains, assert-stdout-is, deftest).

## Convention

- Every registered test has a keyword path under `:wat-tests::*`.
- Every test body writes "PASS" or the failure message to stdout.
- A test file's `:user::main` writes lines in the form
  `<test-name>:PASS` or `<test-name>:FAIL-<reason>`.
